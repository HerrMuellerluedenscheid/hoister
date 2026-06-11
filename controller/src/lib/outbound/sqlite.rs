use crate::domain::billing::models::{Plan, PlanError};
use crate::domain::billing::ports::PlanRepository;
use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState, ServiceState,
};
use crate::domain::container_state::port::ContainerStateRepository;
use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project, ProjectId,
};
use crate::domain::deployments::models::service::{Service, ServiceId};
use crate::domain::deployments::ports::DeploymentsRepository;
use crate::domain::metrics::models::{
    AddMetricsRequest, LatestMetric, MetricPoint, RETENTION_DAYS,
};
use crate::domain::metrics::port::MetricsRepository;
use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError, NotifierKind};
use crate::domain::notifiers::ports::NotifierRepository;
use crate::domain::tokens::models::{ApiToken, TokenError};
use crate::domain::tokens::ports::TokenRepository;
use hoister_shared::{DeploymentStatus, HostName, ImageName, ProjectName, ServiceName};
use log::error;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Error as SqlxError, Row, SqlitePool};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Clone)]
pub struct Sqlite {
    pool: SqlitePool,
    /// Server-side pepper combined with every agent token via HMAC-SHA256
    /// before storage. See `crate::domain::tokens::hash::hash_token`.
    token_pepper: std::sync::Arc<Vec<u8>>,
    /// Envelope-AEAD for notifier configs at rest. Pass-through when no
    /// `HOISTER_CONTROLLER_NOTIFIER_KEY` is set (dev / self-hosted).
    aead: crate::outbound::secrets::Aead,
}

impl Sqlite {
    pub async fn new(
        database_url: &str,
        token_pepper: Vec<u8>,
        aead: crate::outbound::secrets::Aead,
    ) -> Result<Self, SqlxError> {
        info!("Connecting to database: {database_url}");
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }

        // SQLite enforces foreign keys only when `foreign_keys` is turned on,
        // and the pragma is per-connection — so set it on every pooled
        // connection. Without this the `ON DELETE CASCADE` from
        // service_metrics → service, compose_state → project, and the
        // deployment FKs would be silently ignored.
        let pool = SqlitePoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(database_url)
            .await?;

        Ok(Self {
            pool,
            token_pepper: std::sync::Arc::new(token_pepper),
            aead,
        })
    }

    /// Run embedded database migrations.
    pub async fn migrate(&self) -> Result<(), SqlxError> {
        info!("Running database migrations");
        sqlx::migrate!("migrations/sqlite")
            .run(&self.pool)
            .await
            .map_err(|e| SqlxError::Migrate(Box::new(e)))?;
        Ok(())
    }

    /// Get all deployments owned by `user_id`.
    pub async fn get_all_deployments(&self, user_id: &str) -> Result<Vec<Deployment>, SqlxError> {
        let rows = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at,
                    d.logs,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                LEFT JOIN host h ON d.host_id = h.id
                WHERE p.user_id = ?
                ORDER BY d.created_at DESC
                LIMIT 50",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get::<uuid::Uuid, _>("id")),
                digest: row.get("digest"),
                status: row.get("status"),
                service_id: row.get::<uuid::Uuid, _>("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
                hostname: HostName::new(row.get::<String, _>("hostname")),
                logs: row.get("logs"),
            })
            .collect();

        Ok(deployments)
    }

    /// Upsert a project by name, returning its UUID.
    pub async fn upsert_project(
        &self,
        name: &ProjectName,
        user_id: &str,
        host_id: uuid::Uuid,
    ) -> Result<uuid::Uuid, SqlxError> {
        let id = uuid::Uuid::new_v4();
        let result = sqlx::query(
            r#"
            INSERT INTO project (id, name, user_id, host_id) VALUES (?, ?, ?, ?)
            ON CONFLICT(user_id, name) DO UPDATE SET name = name
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(name.as_str())
        .bind(user_id)
        .bind(host_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Upsert a service, returning its UUID.
    pub async fn upsert_service(
        &self,
        project_id: uuid::Uuid,
        name: &ServiceName,
        image: &ImageName,
    ) -> Result<uuid::Uuid, SqlxError> {
        let id = uuid::Uuid::new_v4();
        let result = sqlx::query(
            r#"
            INSERT INTO service (id, project_id, name, image) VALUES (?, ?, ?, ?)
            ON CONFLICT(project_id, name) DO UPDATE SET image = excluded.image
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(name.as_str())
        .bind(image.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Upsert a host by hostname, returning its UUID.
    pub async fn upsert_host(
        &self,
        hostname: &HostName,
        user_id: &str,
    ) -> Result<uuid::Uuid, SqlxError> {
        let id = uuid::Uuid::new_v4();
        let result = sqlx::query(
            r#"
            INSERT INTO host (id, hostname, user_id) VALUES (?, ?, ?)
            ON CONFLICT(user_id, hostname) DO UPDATE SET hostname = hostname
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(hostname.as_str())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Get a project by name
    pub async fn get_project(&self, project_name: &ProjectName) -> Result<Project, SqlxError> {
        let row = sqlx::query(
            r#"
            SELECT * FROM project WHERE project.name = ?
            "#,
        )
        .bind(project_name.as_str())
        .fetch_one(&self.pool)
        .await?;

        let project = Project {
            id: ProjectId(row.get::<uuid::Uuid, _>("id")),
            name: ProjectName(row.get("name")),
            created_at: row.get("created_at"),
        };
        Ok(project)
    }

    pub async fn get_service(
        &self,
        project: &Project,
        service_name: &ServiceName,
    ) -> Result<Service, SqlxError> {
        let row = sqlx::query(
            r#"
            SELECT * FROM service WHERE service.name = ? AND service.project_id = ?
            "#,
        )
        .bind(service_name.as_str())
        .bind(project.id.0)
        .fetch_one(&self.pool)
        .await?;

        let result = Service {
            id: ServiceId(row.get::<uuid::Uuid, _>("id")),
            name: ServiceName(row.get("name")),
            project_id: ProjectId(row.get::<uuid::Uuid, _>("project_id")),
            created_at: row.get("created_at"),
        };
        Ok(result)
    }

    async fn clear_last_no_update_deployment(
        &self,
        service_id: uuid::Uuid,
    ) -> Result<(), SqlxError> {
        sqlx::query("DELETE FROM deployment WHERE status = ? AND service_id = ?")
            .bind(DeploymentStatus::NoUpdate as u8)
            .bind(service_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get a deployment by ID, scoped to `user_id`. Returns RowNotFound if
    /// the deployment exists but belongs to another tenant.
    pub async fn get_deployment(
        &self,
        id: DeploymentId,
        user_id: &str,
    ) -> Result<Deployment, SqlxError> {
        let row = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at,
                    d.logs,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                LEFT JOIN host h ON d.host_id = h.id
                WHERE d.id = ? AND p.user_id = ?",
        )
        .bind(id.0)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let deployment = Deployment {
            id: DeploymentId(row.get("id")),
            digest: row.get("digest"),
            status: row.get("status"),
            service_id: row.get("service_id"),
            created_at: row.get("created_at"),
            service_name: ServiceName(row.get("service_name")),
            project_name: ProjectName(row.get("project_name")),
            hostname: HostName::new(row.get::<String, _>("hostname")),
            logs: row.get("logs"),
        };

        Ok(deployment)
    }

    /// Get deployments by service for a specific user.
    pub async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: &str,
    ) -> Result<Vec<Deployment>, SqlxError> {
        let project = self.get_project(project_name).await?;
        let service = self.get_service(&project, service_name).await?;
        let rows = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at,
                    d.logs,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                    JOIN service s ON d.service_id = s.id
                    JOIN project p ON s.project_id = p.id
                    LEFT JOIN host h ON d.host_id = h.id
                WHERE d.service_id = ? AND p.user_id = ?
                ORDER BY d.created_at DESC LIMIT 50",
        )
        .bind(service.id.0)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get::<uuid::Uuid, _>("id")),
                digest: row.get("digest"),
                status: row.get("status"),
                service_id: row.get::<uuid::Uuid, _>("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
                hostname: HostName::new(row.get::<String, _>("hostname")),
                logs: row.get("logs"),
            })
            .collect();

        Ok(deployments)
    }

    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, SqlxError> {
        let user_id = req.user_id.as_str();
        let host_id = self.upsert_host(&req.hostname, user_id).await?;
        let project_id = self
            .upsert_project(&req.project_name, user_id, host_id)
            .await?;
        let service_id = self
            .upsert_service(project_id, &req.service_name, &req.image_name)
            .await?;

        if matches!(req.deployment_status, DeploymentStatus::NoUpdate) {
            self.clear_last_no_update_deployment(service_id).await?;
            debug!(
                "deleted {} - {}",
                req.image_digest.as_str(),
                req.deployment_status
            )
        }

        let id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO deployment (id, digest, status, service_id, host_id, logs) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(req.image_digest.as_str())
        .bind(&req.deployment_status)
        .bind(service_id)
        .bind(host_id)
        .bind(req.logs.as_deref())
        .execute(&self.pool)
        .await
        .expect("Failed to insert deployment");

        Ok(DeploymentId(id))
    }
}

impl TokenRepository for Sqlite {
    async fn list_tokens(&self, user_id: &str) -> Result<Vec<ApiToken>, TokenError> {
        let rows = sqlx::query(
            "SELECT id, user_id, token_prefix, comment, created_at
                FROM api_token
                WHERE user_id = ?
                ORDER BY created_at DESC, id DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TokenError::UnknownError)?;

        Ok(rows
            .iter()
            .map(|r| ApiToken {
                id: r.get::<uuid::Uuid, _>("id"),
                user_id: r.get("user_id"),
                token: None,
                token_prefix: r.get("token_prefix"),
                comment: r.get("comment"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn create_token(
        &self,
        user_id: &str,
        comment: Option<String>,
    ) -> Result<ApiToken, TokenError> {
        let id = uuid::Uuid::new_v4();
        let token = format!("hst_{}", uuid::Uuid::new_v4().simple());
        let token_hash = crate::domain::tokens::hash::hash_token(&token, &self.token_pepper);
        let token_prefix = token[..12].to_string();
        let row = sqlx::query(
            "INSERT INTO api_token (id, user_id, token_hash, token_prefix, comment)
                VALUES (?, ?, ?, ?, ?)
                RETURNING created_at",
        )
        .bind(id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(&token_prefix)
        .bind(&comment)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| TokenError::UnknownError)?;

        Ok(ApiToken {
            id,
            user_id: user_id.to_string(),
            token: Some(token),
            token_prefix,
            comment,
            created_at: row.get("created_at"),
        })
    }

    async fn delete_token(&self, user_id: &str, token_id: uuid::Uuid) -> Result<bool, TokenError> {
        let result = sqlx::query("DELETE FROM api_token WHERE id = ? AND user_id = ?")
            .bind(token_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| TokenError::UnknownError)?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_user_by_token(&self, token: &str) -> Option<String> {
        let token_hash = crate::domain::tokens::hash::hash_token(token, &self.token_pepper);
        sqlx::query_scalar::<_, String>("SELECT user_id FROM api_token WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }
}

impl DeploymentsRepository for Sqlite {
    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, CreateDeploymentError> {
        self.create_deployment(req).await.map_err(|e| {
            error!("Failed to create deployment: {e:?}");
            CreateDeploymentError::UnknownError
        })
    }

    async fn get_all_deployments(
        &self,
        user_id: &str,
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.get_all_deployments(user_id).await.map_err(|e| {
            error!("Failed to get all deployments: {e:?}");
            GetDeploymentError::UnknownError
        })
    }

    async fn get_deployment(
        &self,
        deployment_id: DeploymentId,
        user_id: &str,
    ) -> Result<Deployment, GetDeploymentError> {
        self.get_deployment(deployment_id, user_id)
            .await
            .map_err(|e| {
                error!("Failed to get deployment: {e:?}");
                match e {
                    sqlx::error::Error::RowNotFound => GetDeploymentError::DeploymentNotFound,
                    _ => GetDeploymentError::UnknownError,
                }
            })
    }

    async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: &str,
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.get_deployments_of_service(project_name, service_name, user_id)
            .await
            .map_err(|e| match e {
                sqlx::error::Error::RowNotFound => GetDeploymentError::DeploymentNotFound,
                _ => {
                    error!(
                        "Failed to get deployments of service: {project_name:?} {service_name:?} | {e:?}"
                    );
                    GetDeploymentError::UnknownError
                }
            })
    }

    async fn get_project(&self, project_name: &ProjectName) -> Result<Project, GetProjectError> {
        self.get_project(project_name).await.map_err(|e| {
            error!("Failed to get project: {e:?}");
            GetProjectError::UnknownError
        })
    }
}

impl NotifierRepository for Sqlite {
    async fn list_notifiers(&self, user_id: &str) -> Result<Vec<Notifier>, NotifierError> {
        let rows = sqlx::query(
            "SELECT id, user_id, kind, config, enabled, created_at
                FROM notifier
                WHERE user_id = ?
                ORDER BY created_at DESC, id DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("list_notifiers failed: {e:?}");
            NotifierError::UnknownError
        })?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let kind_str: String = r.get("kind");
            let kind = NotifierKind::parse(&kind_str)
                .ok_or_else(|| NotifierError::InvalidConfig(format!("unknown kind {kind_str}")))?;
            let stored: String = r.get("config");
            let config_str = self.aead.decrypt_or_plaintext(&stored).map_err(|e| {
                error!("notifier config decrypt failed: {e:?}");
                NotifierError::UnknownError
            })?;
            let config: NotifierConfig = serde_json::from_str(&config_str)
                .map_err(|e| NotifierError::InvalidConfig(e.to_string()))?;
            let enabled_int: i64 = r.get("enabled");
            out.push(Notifier {
                id: r.get::<uuid::Uuid, _>("id"),
                user_id: r.get("user_id"),
                kind,
                config,
                enabled: enabled_int != 0,
                created_at: r.get("created_at"),
            });
        }
        Ok(out)
    }

    async fn create_notifier(
        &self,
        user_id: &str,
        config: NotifierConfig,
    ) -> Result<Notifier, NotifierError> {
        let kind = config.kind();
        let config_json = serde_json::to_string(&config)
            .map_err(|e| NotifierError::InvalidConfig(e.to_string()))?;
        let to_store = self.aead.encrypt(&config_json).map_err(|e| {
            error!("notifier config encrypt failed: {e:?}");
            NotifierError::UnknownError
        })?;
        let id = uuid::Uuid::new_v4();
        let row = sqlx::query(
            "INSERT INTO notifier (id, user_id, kind, config) VALUES (?, ?, ?, ?)
                RETURNING created_at",
        )
        .bind(id)
        .bind(user_id)
        .bind(kind.as_str())
        .bind(&to_store)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("create_notifier failed: {e:?}");
            NotifierError::UnknownError
        })?;
        Ok(Notifier {
            id,
            user_id: user_id.to_string(),
            kind,
            config,
            enabled: true,
            created_at: row.get("created_at"),
        })
    }

    async fn delete_notifier(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
    ) -> Result<bool, NotifierError> {
        let result = sqlx::query("DELETE FROM notifier WHERE id = ? AND user_id = ?")
            .bind(notifier_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| NotifierError::UnknownError)?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_enabled(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<bool, NotifierError> {
        let result = sqlx::query("UPDATE notifier SET enabled = ? WHERE id = ? AND user_id = ?")
            .bind(if enabled { 1_i64 } else { 0 })
            .bind(notifier_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| NotifierError::UnknownError)?;
        Ok(result.rows_affected() > 0)
    }
}

impl PlanRepository for Sqlite {
    async fn get_plan(&self, user_id: &str) -> Result<Plan, PlanError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT plan FROM user_plan WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("get_plan failed: {e:?}");
                PlanError::UnknownError
            })?;
        Ok(row.and_then(|(s,)| Plan::parse(&s)).unwrap_or(Plan::Free))
    }

    async fn set_plan(&self, user_id: &str, plan: Plan) -> Result<(), PlanError> {
        sqlx::query(
            "INSERT INTO user_plan (user_id, plan, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)
                 ON CONFLICT(user_id) DO UPDATE SET plan = excluded.plan, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(user_id)
        .bind(plan.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("set_plan failed: {e:?}");
            PlanError::UnknownError
        })?;
        Ok(())
    }

    async fn upsert_user(&self, user_id: &str) {
        if let Err(e) = sqlx::query("INSERT OR IGNORE INTO users(id) VALUES (?)")
            .bind(user_id)
            .execute(&self.pool)
            .await
        {
            error!("upsert_user failed for {user_id}: {e:?}");
        }
    }

    async fn delete_user(&self, user_id: &str) -> bool {
        match sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
        {
            Ok(r) => r.rows_affected() > 0,
            Err(e) => {
                error!("delete_user failed for {user_id}: {e:?}");
                false
            }
        }
    }
}

impl ContainerStateRepository for Sqlite {
    async fn get_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<HostProjectState> {
        let row: (String, String) = sqlx::query_as(
            "SELECT cs.services, cs.last_updated
                FROM compose_state cs
                JOIN project p ON cs.project_id = p.id
                JOIN host h ON p.host_id = h.id
                WHERE p.user_id = ? AND h.hostname = ? AND p.name = ?",
        )
        .bind(user_id)
        .bind(hostname.as_str())
        .bind(project_name.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| error!("get_container_state failed: {e:?}"))
        .ok()??;

        let (services_json, last_updated) = row;
        let mut services: HashMap<ServiceName, ServiceState> = serde_json::from_str(&services_json)
            .map_err(|e| error!("services blob decode failed: {e:?}"))
            .ok()?;
        services.retain(|k, _| k == service_name);
        if services.is_empty() {
            return None;
        }
        Some(HostProjectState {
            services,
            last_updated: parse_ts(&last_updated),
        })
    }

    async fn get_container_states(&self, user_id: &str) -> ContainerStateData {
        let rows: Vec<(String, String, String, String)> = match sqlx::query_as(
            "SELECT h.hostname, p.name, cs.services, cs.last_updated
                FROM compose_state cs
                JOIN project p ON cs.project_id = p.id
                JOIN host h ON p.host_id = h.id
                WHERE p.user_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        {
            Ok(rs) => rs,
            Err(e) => {
                error!("get_container_states failed: {e:?}");
                return ContainerStateData::default();
            }
        };

        let mut out: ContainerStateData = HashMap::new();
        for (hostname, project_name, services_json, last_updated) in rows {
            let services: HashMap<ServiceName, ServiceState> =
                match serde_json::from_str(&services_json) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("services blob decode failed: {e:?}");
                        continue;
                    }
                };
            out.entry(HostName::new(hostname)).or_default().insert(
                ProjectName::new(project_name),
                HostProjectState {
                    services,
                    last_updated: parse_ts(&last_updated),
                },
            );
        }
        out
    }

    async fn add_container_state(&self, req: AddContainerStateRequest) {
        let services_json = match serde_json::to_string(&req.services) {
            Ok(s) => s,
            Err(e) => {
                error!("encode services for {} failed: {e:?}", req.user_id);
                return;
            }
        };
        let host_id = match self.upsert_host(&req.hostname, &req.user_id).await {
            Ok(id) => id,
            Err(e) => {
                error!("add_container_state upsert_host failed: {e:?}");
                return;
            }
        };
        let project_id = match self
            .upsert_project(&req.project_name, &req.user_id, host_id)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                error!("add_container_state upsert_project failed: {e:?}");
                return;
            }
        };
        let now = chrono::Utc::now().to_rfc3339();
        if let Err(e) = sqlx::query(
            "INSERT INTO compose_state (project_id, services, last_updated)
                 VALUES (?, ?, ?)
                 ON CONFLICT(project_id) DO UPDATE SET
                     services = excluded.services,
                     last_updated = excluded.last_updated",
        )
        .bind(project_id)
        .bind(&services_json)
        .bind(&now)
        .execute(&self.pool)
        .await
        {
            error!("add_container_state failed: {e:?}");
        }
    }

    async fn delete_project(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
    ) -> bool {
        match sqlx::query(
            "DELETE FROM project
                WHERE user_id = ? AND name = ?
                  AND EXISTS (
                    SELECT 1 FROM host WHERE id = project.host_id AND hostname = ?
                  )",
        )
        .bind(user_id)
        .bind(project_name.as_str())
        .bind(hostname.as_str())
        .execute(&self.pool)
        .await
        {
            Ok(result) => result.rows_affected() > 0,
            Err(e) => {
                error!("delete_project failed: {e:?}");
                false
            }
        }
    }
}

/// Parse a timestamp stored either as RFC3339 (our writes) or SQLite's
/// `CURRENT_TIMESTAMP` (`%Y-%m-%d %H:%M:%S`) default. Mirrors the fallback
/// used by the container_state reads above.
fn parse_ts(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc().fixed_offset())
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .unwrap_or_else(|_| chrono::Utc::now())
}

impl MetricsRepository for Sqlite {
    async fn add_metrics(&self, req: AddMetricsRequest) {
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();
        let mut tx = match self.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("add_metrics begin tx failed: {e:?}");
                return;
            }
        };
        for (service_name, sample) in &req.samples {
            // Resolve service_id via JOIN; if the service row doesn't exist yet
            // the SELECT returns nothing and the INSERT is a no-op, which is
            // the same guard the old container_state FK provided.
            if let Err(e) = sqlx::query(
                "INSERT OR IGNORE INTO service_metrics
                    (service_id, recorded_at, cpu_pct, mem_bytes, mem_limit_bytes)
                 SELECT s.id, ?, ?, ?, ?
                 FROM service s
                 JOIN project p ON s.project_id = p.id
                 JOIN host h ON p.host_id = h.id
                 WHERE p.user_id = ? AND h.hostname = ? AND p.name = ? AND s.name = ?",
            )
            .bind(&now_str)
            .bind(sample.cpu_pct)
            .bind(sample.mem_bytes as i64)
            .bind(sample.mem_limit_bytes as i64)
            .bind(&req.user_id)
            .bind(req.hostname.as_str())
            .bind(req.project_name.as_str())
            .bind(service_name.as_str())
            .execute(&mut *tx)
            .await
            {
                error!("add_metrics insert failed: {e:?}");
            }
        }

        // Opportunistic retention: keep at most RETENTION_DAYS of samples for
        // this user. Runs at most once per agent report (~per minute).
        let cutoff = (now - chrono::Duration::days(RETENTION_DAYS)).to_rfc3339();
        if let Err(e) = sqlx::query(
            "DELETE FROM service_metrics
                WHERE recorded_at < ?
                  AND service_id IN (
                    SELECT s.id FROM service s
                    JOIN project p ON s.project_id = p.id
                    WHERE p.user_id = ?
                  )",
        )
        .bind(&cutoff)
        .bind(&req.user_id)
        .execute(&mut *tx)
        .await
        {
            error!("add_metrics prune failed: {e:?}");
        }

        if let Err(e) = tx.commit().await {
            error!("add_metrics commit failed: {e:?}");
        }
    }

    async fn get_service_metrics(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Vec<MetricPoint> {
        let rows: Vec<(String, f64, i64, i64)> = match sqlx::query_as(
            "SELECT sm.recorded_at, sm.cpu_pct, sm.mem_bytes, sm.mem_limit_bytes
                FROM service_metrics sm
                JOIN service s ON sm.service_id = s.id
                JOIN project p ON s.project_id = p.id
                JOIN host h ON p.host_id = h.id
                WHERE p.user_id = ? AND h.hostname = ? AND p.name = ? AND s.name = ?
                  AND sm.recorded_at >= ?
                ORDER BY sm.recorded_at ASC",
        )
        .bind(user_id)
        .bind(hostname.as_str())
        .bind(project_name.as_str())
        .bind(service_name.as_str())
        .bind(since.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        {
            Ok(rs) => rs,
            Err(e) => {
                error!("get_service_metrics failed: {e:?}");
                return Vec::new();
            }
        };

        rows.into_iter()
            .map(
                |(recorded_at, cpu_pct, mem_bytes, mem_limit_bytes)| MetricPoint {
                    recorded_at: parse_ts(&recorded_at),
                    cpu_pct,
                    mem_bytes: mem_bytes.max(0) as u64,
                    mem_limit_bytes: mem_limit_bytes.max(0) as u64,
                },
            )
            .collect()
    }

    async fn get_latest_metrics(&self, user_id: &str) -> Vec<LatestMetric> {
        let rows: Vec<(String, String, String, String, f64, i64, i64)> = match sqlx::query_as(
            "SELECT h.hostname, p.name, s.name, sm.recorded_at,
                    sm.cpu_pct, sm.mem_bytes, sm.mem_limit_bytes
                FROM service_metrics sm
                JOIN service s ON sm.service_id = s.id
                JOIN project p ON s.project_id = p.id
                JOIN host h ON p.host_id = h.id
                WHERE p.user_id = ?
                  AND sm.recorded_at = (
                    SELECT MAX(sm2.recorded_at) FROM service_metrics sm2
                    WHERE sm2.service_id = sm.service_id
                  )",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        {
            Ok(rs) => rs,
            Err(e) => {
                error!("get_latest_metrics failed: {e:?}");
                return Vec::new();
            }
        };

        rows.into_iter()
            .map(
                |(
                    hostname,
                    project_name,
                    service_name,
                    recorded_at,
                    cpu_pct,
                    mem_bytes,
                    mem_limit_bytes,
                )| {
                    LatestMetric {
                        hostname: HostName::new(hostname),
                        project_name: ProjectName::new(project_name),
                        service_name: ServiceName::new(service_name),
                        point: MetricPoint {
                            recorded_at: parse_ts(&recorded_at),
                            cpu_pct,
                            mem_bytes: mem_bytes.max(0) as u64,
                            mem_limit_bytes: mem_limit_bytes.max(0) as u64,
                        },
                    }
                },
            )
            .collect()
    }
}
