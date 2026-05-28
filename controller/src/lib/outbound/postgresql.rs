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
use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError, NotifierKind};
use crate::domain::notifiers::ports::NotifierRepository;
use crate::domain::tokens::models::{ApiToken, TokenError};
use crate::domain::tokens::ports::TokenRepository;
use hoister_shared::{DeploymentStatus, HostName, ImageName, ProjectName, ServiceName};
use log::error;
use sqlx::{Error as SqlxError, PgPool, Row};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Clone)]
pub struct Postgresql {
    pool: PgPool,
    /// Server-side pepper combined with every agent token via HMAC-SHA256
    /// before storage. See `crate::domain::tokens::hash::hash_token`.
    token_pepper: std::sync::Arc<Vec<u8>>,
    /// Envelope-AEAD for notifier configs at rest. See `outbound::secrets`.
    aead: crate::outbound::secrets::Aead,
}

/// Best-effort parse of a postgres `timestamptz::text` value (e.g.
/// `2026-05-28 09:12:01.234567+00`) into a UTC `DateTime`. Falls back to
/// `Utc::now()` if the format drifts — the on-disk timestamp is informational
/// only (UI sort key), not load-bearing for correctness.
fn parse_pg_timestamp(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f%#z")
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(s))
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

fn status_from_i16(val: i16) -> DeploymentStatus {
    match val {
        0 => DeploymentStatus::Pending,
        1 => DeploymentStatus::Started,
        2 => DeploymentStatus::Success,
        3 => DeploymentStatus::RollbackFinished,
        4 => DeploymentStatus::NoUpdate,
        5 => DeploymentStatus::Failed,
        6 => DeploymentStatus::TestMessage,
        _ => DeploymentStatus::Pending,
    }
}

impl Postgresql {
    pub async fn new(
        database_url: &str,
        token_pepper: Vec<u8>,
        aead: crate::outbound::secrets::Aead,
    ) -> Result<Self, SqlxError> {
        info!("Connecting to database: {database_url}");
        let pool = PgPool::connect(database_url).await?;
        Ok(Self {
            pool,
            token_pepper: std::sync::Arc::new(token_pepper),
            aead,
        })
    }

    /// Run embedded database migrations.
    pub async fn migrate(&self) -> Result<(), SqlxError> {
        info!("Running database migrations");
        sqlx::migrate!("migrations/postgres")
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
                    d.created_at::text as created_at,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                LEFT JOIN host h ON d.host_id = h.id
                WHERE p.user_id = $1
                ORDER BY d.created_at DESC
                LIMIT 50",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get("id")),
                digest: row.get("digest"),
                status: status_from_i16(row.get("status")),
                service_id: row.get("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
                hostname: HostName::new(row.get::<String, _>("hostname")),
            })
            .collect();

        Ok(deployments)
    }

    /// Upsert a project by name, returning its ID.
    /// Sets user_id only on insert; existing projects keep their user_id.
    pub async fn upsert_project(
        &self,
        name: &ProjectName,
        user_id: &str,
    ) -> Result<i64, SqlxError> {
        let result = sqlx::query(
            r#"
            INSERT INTO project (name, user_id) VALUES ($1, $2)
            ON CONFLICT(name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(name.as_str())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Upsert a service, returning its ID
    pub async fn upsert_service(
        &self,
        project_id: i64,
        name: &ServiceName,
        image: &ImageName,
    ) -> Result<i64, SqlxError> {
        let result = sqlx::query(
            r#"
            INSERT INTO service (project_id, name, image) VALUES ($1, $2, $3)
            ON CONFLICT(project_id, name) DO UPDATE SET image = EXCLUDED.image
            RETURNING id
            "#,
        )
        .bind(project_id)
        .bind(name.as_str())
        .bind(image.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Upsert a host by hostname, returning its UUID bytes.
    /// Sets user_id only on insert; existing hosts keep their user_id.
    pub async fn upsert_host(
        &self,
        hostname: &HostName,
        user_id: &str,
    ) -> Result<Vec<u8>, SqlxError> {
        let id = uuid::Uuid::new_v4().as_bytes().to_vec();
        let result = sqlx::query(
            r#"
            INSERT INTO host (id, hostname, user_id) VALUES ($1, $2, $3)
            ON CONFLICT(hostname) DO UPDATE SET hostname = EXCLUDED.hostname
            RETURNING id
            "#,
        )
        .bind(&id)
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
            SELECT id, name, created_at::text as created_at FROM project WHERE project.name = $1
            "#,
        )
        .bind(project_name.as_str())
        .fetch_one(&self.pool)
        .await?;

        let project = Project {
            id: ProjectId(row.get("id")),
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
            SELECT id, name, project_id, created_at::text as created_at
            FROM service WHERE service.name = $1 AND service.project_id = $2
            "#,
        )
        .bind(service_name.as_str())
        .bind(project.id.0)
        .fetch_one(&self.pool)
        .await?;

        let result = Service {
            id: ServiceId(row.get("id")),
            name: ServiceName(row.get("name")),
            project_id: ProjectId(row.get("project_id")),
            created_at: row.get("created_at"),
        };
        Ok(result)
    }

    async fn clear_last_no_update_deployment(&self, service_id: i64) -> Result<(), SqlxError> {
        sqlx::query("DELETE FROM deployment WHERE status = $1 AND service_id = $2")
            .bind(DeploymentStatus::NoUpdate as i16)
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
                    d.created_at::text as created_at,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                LEFT JOIN host h ON d.host_id = h.id
                WHERE d.id = $1 AND p.user_id = $2",
        )
        .bind(id.0)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let deployment = Deployment {
            id: DeploymentId(row.get("id")),
            digest: row.get("digest"),
            status: status_from_i16(row.get("status")),
            service_id: row.get("service_id"),
            created_at: row.get("created_at"),
            service_name: ServiceName(row.get("service_name")),
            project_name: ProjectName(row.get("project_name")),
            hostname: HostName::new(row.get::<String, _>("hostname")),
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
                    d.created_at::text as created_at,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                    JOIN service s ON d.service_id = s.id
                    JOIN project p ON s.project_id = p.id
                    LEFT JOIN host h ON d.host_id = h.id
                WHERE d.service_id = $1 AND p.user_id = $2
                ORDER BY d.created_at DESC LIMIT 50",
        )
        .bind(service.id.0)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get("id")),
                digest: row.get("digest"),
                status: status_from_i16(row.get("status")),
                service_id: row.get("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
                hostname: HostName::new(row.get::<String, _>("hostname")),
            })
            .collect();

        Ok(deployments)
    }

    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, SqlxError> {
        let user_id = req.user_id.as_str();
        let project_id = self.upsert_project(&req.project_name, user_id).await?;
        let service_id = self
            .upsert_service(project_id, &req.service_name, &req.image_name)
            .await?;
        let host_id = self.upsert_host(&req.hostname, user_id).await?;

        if matches!(req.deployment_status, DeploymentStatus::NoUpdate) {
            self.clear_last_no_update_deployment(service_id).await?;
            debug!(
                "deleted {} - {}",
                req.image_digest.as_str(),
                req.deployment_status
            )
        }

        let result = sqlx::query(
            "INSERT INTO deployment (digest, status, service_id, host_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(req.image_digest.as_str())
        .bind(req.deployment_status.clone() as i16)
        .bind(service_id)
        .bind(&host_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DeploymentId(result.get("id")))
    }
}

impl TokenRepository for Postgresql {
    async fn list_tokens(&self, user_id: &str) -> Result<Vec<ApiToken>, TokenError> {
        let rows = sqlx::query(
            "SELECT id, user_id, token_prefix, comment, created_at::text AS created_at
                FROM api_token
                WHERE user_id = $1
                ORDER BY created_at DESC, id DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TokenError::UnknownError)?;

        Ok(rows
            .iter()
            .map(|r| ApiToken {
                id: r.get("id"),
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
        let token = format!("hst_{}", uuid::Uuid::new_v4().simple());
        let token_hash = crate::domain::tokens::hash::hash_token(&token, &self.token_pepper);
        let token_prefix = token[..12].to_string();
        let row = sqlx::query(
            "INSERT INTO api_token (user_id, token_hash, token_prefix, comment)
                VALUES ($1, $2, $3, $4)
                RETURNING id, created_at::text AS created_at",
        )
        .bind(user_id)
        .bind(&token_hash)
        .bind(&token_prefix)
        .bind(&comment)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| TokenError::UnknownError)?;

        Ok(ApiToken {
            id: row.get("id"),
            user_id: user_id.to_string(),
            token: Some(token),
            token_prefix,
            comment,
            created_at: row.get("created_at"),
        })
    }

    async fn delete_token(&self, user_id: &str, token_id: i64) -> Result<bool, TokenError> {
        let result = sqlx::query("DELETE FROM api_token WHERE id = $1 AND user_id = $2")
            .bind(token_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| TokenError::UnknownError)?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_user_by_token(&self, token: &str) -> Option<String> {
        let token_hash = crate::domain::tokens::hash::hash_token(token, &self.token_pepper);
        sqlx::query_scalar::<_, String>("SELECT user_id FROM api_token WHERE token_hash = $1")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }
}

impl DeploymentsRepository for Postgresql {
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

impl NotifierRepository for Postgresql {
    async fn list_notifiers(&self, user_id: &str) -> Result<Vec<Notifier>, NotifierError> {
        let rows = sqlx::query(
            "SELECT id, user_id, kind, config::text AS config, enabled, created_at::text AS created_at
                FROM notifier
                WHERE user_id = $1
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
            out.push(Notifier {
                id: r.get("id"),
                user_id: r.get("user_id"),
                kind,
                config,
                enabled: r.get("enabled"),
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
        let row = sqlx::query(
            "INSERT INTO notifier (user_id, kind, config) VALUES ($1, $2, $3::jsonb)
                RETURNING id, created_at::text AS created_at",
        )
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
            id: row.get("id"),
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
        notifier_id: i64,
    ) -> Result<bool, NotifierError> {
        let result = sqlx::query("DELETE FROM notifier WHERE id = $1 AND user_id = $2")
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
        notifier_id: i64,
        enabled: bool,
    ) -> Result<bool, NotifierError> {
        let result = sqlx::query("UPDATE notifier SET enabled = $1 WHERE id = $2 AND user_id = $3")
            .bind(enabled)
            .bind(notifier_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| NotifierError::UnknownError)?;
        Ok(result.rows_affected() > 0)
    }
}

impl PlanRepository for Postgresql {
    async fn get_plan(&self, user_id: &str) -> Result<Plan, PlanError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT plan FROM user_plan WHERE user_id = $1")
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
            "INSERT INTO user_plan (user_id, plan, updated_at) VALUES ($1, $2, NOW())
                 ON CONFLICT(user_id) DO UPDATE SET plan = EXCLUDED.plan, updated_at = NOW()",
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
}

impl ContainerStateRepository for Postgresql {
    async fn get_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<HostProjectState> {
        let row: (String, String) = sqlx::query_as(
            "SELECT services::text, last_updated::text FROM container_state
                WHERE user_id = $1 AND hostname = $2 AND project_name = $3",
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
            last_updated: parse_pg_timestamp(&last_updated),
        })
    }

    async fn get_container_states(&self, user_id: &str) -> ContainerStateData {
        let rows: Vec<(String, String, String, String)> = match sqlx::query_as(
            "SELECT hostname, project_name, services::text, last_updated::text
                FROM container_state WHERE user_id = $1",
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
                    last_updated: parse_pg_timestamp(&last_updated),
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
        if let Err(e) = sqlx::query(
            "INSERT INTO container_state (user_id, hostname, project_name, services, last_updated)
                 VALUES ($1, $2, $3, $4::jsonb, NOW())
                 ON CONFLICT(user_id, hostname, project_name) DO UPDATE SET
                     services = EXCLUDED.services,
                     last_updated = NOW()",
        )
        .bind(&req.user_id)
        .bind(req.hostname.as_str())
        .bind(req.project_name.as_str())
        .bind(&services_json)
        .execute(&self.pool)
        .await
        {
            error!("add_container_state failed: {e:?}");
        }
    }
}
