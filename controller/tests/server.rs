#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use hoister_shared::{
        CreateDeployment, DeploymentStatus, HostName, ImageDigest, ImageName, ProjectName,
        ServiceName,
    };

    use controller::domain::alerts::service::Service as AlertsService;
    use controller::domain::billing::ports::BillingService as _;
    use controller::domain::billing::service::Service as BillingService;
    use controller::domain::container_state::service::Service as ContainerStateService;
    use controller::domain::deployments::models::deployment::{
        CreateDeploymentRequest, Deployment,
    };
    use controller::domain::deployments::ports::DeploymentsService as _;
    use controller::domain::deployments::service::Service as DeploymentsService;
    use controller::domain::metrics::service::Service as MetricsService;
    use controller::domain::notifiers::service::Service as NotifierService;
    use controller::domain::projects::service::Service as ProjectsService;
    use controller::domain::tokens::service::Service as TokenService;
    use controller::inbound::server::{
        ApiResponse, AppState, InternalSecret, create_agent_router, create_internal_router,
    };
    use controller::outbound::Database;
    use controller::outbound::secrets::Aead;
    use controller::sse::UserScopedEvent;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tower::ServiceExt;

    /// The agent router maps the static api secret to this synthetic user id,
    /// so deployments created via the agent API are owned by "local".
    const TEST_USER: &str = "local";

    fn unique_db_path() -> String {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/hoister_test_{}_{}", std::process::id(), id)
    }

    /// Opens a database at `path`. `Database::connect` runs migrations, and the
    /// passthrough AEAD keeps notifier secrets in plaintext (fine for tests).
    async fn connect_db(path: &str) -> Database {
        let aead = Aead::from_base64_or_passthrough(None).expect("aead");
        Database::connect(path, b"tests-pepper".to_vec(), aead)
            .await
            .expect("connect database")
    }

    /// Builds both routers (agent + internal) over a shared, migrated database.
    ///
    /// The agent router authenticates via the `Bearer` api secret; the internal
    /// router trusts the `X-User-Id` header (`InternalSecret` is `None` in tests,
    /// so the shared-secret check is skipped).
    async fn setup_test_app() -> (Router, Router, String) {
        let db_path = unique_db_path();
        let db = connect_db(&db_path).await;
        let (event_tx, _) = broadcast::channel::<UserScopedEvent>(100);
        let state = AppState {
            deployments_service: Arc::new(DeploymentsService::new(db.clone())),
            container_state_service: Arc::new(ContainerStateService::new(db.clone())),
            token_service: Arc::new(TokenService::new(db.clone())),
            notifier_service: Arc::new(NotifierService::new(db.clone())),
            billing_service: Arc::new(BillingService::new(db.clone())),
            metrics_service: Arc::new(MetricsService::new(db.clone())),
            alerts_service: Arc::new(AlertsService::new(db.clone())),
            projects_service: Arc::new(ProjectsService::new(db)),
            #[cfg(feature = "self-hosted")]
            api_secret: Some("tests-secret".to_string()),
            event_tx,
            pending_updates: Default::default(),
            logs: Default::default(),
            email: None,
            dashboard_url: "https://hoister.io".to_string(),
        };
        // Seed the synthetic "local" user. In production the auth middleware
        // upserts the user on every request (see `upsert_user` calls in
        // server.rs); tests that hit the repository directly bypass that, and
        // with `PRAGMA foreign_keys = ON` the `host.user_id -> users(id)` FK
        // would otherwise reject any host/project insert.
        state.billing_service.upsert_user(TEST_USER).await;
        let agent = create_agent_router(state.clone()).await;
        let internal = create_internal_router(state, InternalSecret(None)).await;
        (agent, internal, db_path)
    }

    #[tokio::test]
    async fn test_health_endpoint_no_auth_required() {
        let (agent, _internal, _db) = setup_test_app().await;

        let response = agent
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn test_agent_router_requires_auth() {
        let (agent, _internal, _db) = setup_test_app().await;

        // No Authorization header → the agent auth middleware rejects before
        // routing.
        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/deployments")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_internal_list_deployments() {
        let (_agent, internal, _db) = setup_test_app().await;

        let response = internal
            .oneshot(
                Request::builder()
                    .uri("/deployments")
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: ApiResponse<Vec<Deployment>> = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        assert!(response.data.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_create_deployment_via_agent() {
        let (agent, _internal, _db) = setup_test_app().await;

        let payload = CreateDeployment {
            project: ProjectName::new("tests-project"),
            service: ServiceName::new("tests-service"),
            image: ImageName::new("nginx:latest"),
            digest: ImageDigest::new("sha256:abc123"),
            status: DeploymentStatus::Pending,
            hostname: HostName::new("test-host"),
            logs: None,
        };

        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/deployments")
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_deployment_by_service() {
        let (_agent, internal, db_path) = setup_test_app().await;
        let image_name = ImageName::new("aaa");
        let service_name = ServiceName::new("tests-service");
        let project_name = ProjectName::new("tests-project");

        // Seed a deployment owned by TEST_USER directly through the repository.
        let database_service = DeploymentsService::new(connect_db(&db_path).await);
        let req = CreateDeploymentRequest {
            project_name: project_name.clone(),
            service_name: service_name.clone(),
            image_name: image_name.clone(),
            image_digest: ImageDigest::new("sha256:abc123"),
            deployment_status: DeploymentStatus::Pending,
            hostname: HostName::new("test-host"),
            logs: None,
            user_id: TEST_USER.to_string(),
        };
        database_service.create_deployment(&req).await.unwrap();

        let response = internal
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/deployments/{}/{}",
                        project_name.as_str(),
                        service_name.as_str()
                    ))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: ApiResponse<Vec<Deployment>> = serde_json::from_slice(&body).unwrap();
        assert!(response.success);
        response.data.unwrap();
    }

    #[tokio::test]
    async fn test_auth_with_invalid_token() {
        let (agent, _internal, _db) = setup_test_app().await;

        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/deployments")
                    .header("Authorization", "Bearer wrong-secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_without_bearer_prefix() {
        let (agent, _internal, _db) = setup_test_app().await;

        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/deployments")
                    .header("Authorization", "tests-secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// GET /container/metrics (latest-per-service) as a raw JSON array.
    async fn latest_metrics(internal: &Router) -> Vec<serde_json::Value> {
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/container/metrics")
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice::<serde_json::Value>(&body)
            .unwrap()
            .as_array()
            .cloned()
            .unwrap()
    }

    #[tokio::test]
    async fn test_delete_project_cascades_metrics_and_is_idempotent() {
        let (agent, internal, _db) = setup_test_app().await;
        let host = "test-host";
        let project = "tests-project";

        // Seed a project with a "web" service by reporting container state via
        // the agent. The service row must exist for the metrics insert below to
        // resolve it (the insert is a no-op for unknown services).
        let state_body = serde_json::json!({
            "project_name": project,
            "payload": { "web": { "inspect": {} } }
        });
        let response = agent
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/container/state/{host}/{project}"))
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(state_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Report a metrics sample for the same (host, project). With the state
        // row present, the FK-guarded insert stores it.
        let metrics_body = serde_json::json!({
            "project_name": project,
            "payload": { "web": { "cpu_pct": 1.5, "mem_bytes": 1000, "mem_limit_bytes": 2000 } }
        });
        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/container/metrics/{host}/{project}"))
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(metrics_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(latest_metrics(&internal).await.len(), 1);

        // Deleting the project succeeds and frees the slot.
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/container/state/{host}/{project}"))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // The container state is gone...
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/container/state")
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let states: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(states.as_array().map(|a| a.len()), Some(0));

        // ...and so are its metrics, removed by the ON DELETE CASCADE rather
        // than a second application-level delete.
        assert_eq!(latest_metrics(&internal).await.len(), 0);

        // Deleting a project that no longer exists is a 404.
        let response = internal
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/container/state/{host}/{project}"))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_metrics_without_state_row_are_dropped() {
        let (agent, internal, _db) = setup_test_app().await;
        let host = "test-host";
        let project = "orphan-project";

        // Report metrics for a project that has no container_state row. The
        // FK-guarded insert skips them rather than violating the foreign key.
        let metrics_body = serde_json::json!({
            "project_name": project,
            "payload": { "web": { "cpu_pct": 1.5, "mem_bytes": 1000, "mem_limit_bytes": 2000 } }
        });
        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/container/metrics/{host}/{project}"))
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(metrics_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(latest_metrics(&internal).await.len(), 0);
    }

    /// GET the alert history as the given login session.
    async fn alert_history(internal: &Router, session: &str) -> serde_json::Value {
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/alerts/events?session={session}"))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        parsed["data"].clone()
    }

    /// A fired alert lands in the history with its host/project/service, and is
    /// flagged new only once the dashboard comes back under a new login.
    #[tokio::test]
    async fn test_alert_history_records_firings_and_flags_them_per_login() {
        let (agent, internal, _db) = setup_test_app().await;

        // First visit of the first session: an empty history and no cutoff.
        let history = alert_history(&internal, "session-a").await;
        assert_eq!(history["events"].as_array().map(|a| a.len()), Some(0));
        assert!(history["new_since"].is_null());
        assert_eq!(history["new_count"], 0);

        let rule = serde_json::json!({
            "metric": "cpu_pct",
            "threshold": 80.0,
            "for_seconds": 0,
            "service": "web",
        });
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/alerts")
                    .header("X-User-Id", TEST_USER)
                    .header("Content-Type", "application/json")
                    .body(Body::from(rule.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Ingest a breaching sample so the rule fires (for_seconds = 0).
        let state_body = serde_json::json!({
            "project_name": "history-project",
            "payload": { "web": { "inspect": {} } }
        });
        let response = agent
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/container/state/history-host/history-project")
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(state_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let metrics_body = serde_json::json!({
            "project_name": "history-project",
            "payload": { "web": { "cpu_pct": 95.0, "mem_bytes": 1000, "mem_limit_bytes": 2000 } }
        });
        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/container/metrics/history-host/history-project")
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(metrics_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Still the same login: the firing is listed but not flagged as new.
        let history = alert_history(&internal, "session-a").await;
        let events = history["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["kind"], "fired");
        assert_eq!(events[0]["metric"], "cpu_pct");
        assert_eq!(events[0]["value"], 95.0);
        assert_eq!(events[0]["hostname"], "history-host");
        assert_eq!(events[0]["project"], "history-project");
        assert_eq!(events[0]["service"], "web");
        assert_eq!(events[0]["is_new"], false);
        assert_eq!(history["new_count"], 0);

        // A new login: it fired while they were away, so it is highlighted —
        // and stays highlighted while they are on this session.
        let history = alert_history(&internal, "session-b").await;
        assert_eq!(history["events"][0]["is_new"], true);
        assert_eq!(history["new_count"], 1);
        assert!(history["new_since"].is_string());
        let again = alert_history(&internal, "session-b").await;
        assert_eq!(again["new_since"], history["new_since"]);
        assert_eq!(again["new_count"], 1);

        // Deleting the rule must not erase what it already reported; the entry
        // survives with its rule reference cleared.
        let rule_id = again["events"][0]["rule_id"].as_str().unwrap().to_string();
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/alerts/{rule_id}"))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let history = alert_history(&internal, "session-b").await;
        assert_eq!(history["events"].as_array().map(|a| a.len()), Some(1));
        assert!(history["events"][0]["rule_id"].is_null());
        assert_eq!(history["events"][0]["service"], "web");
    }

    /// Alert-rule CRUD over the internal router, plus a metrics POST through
    /// the agent router while a rule exists — evaluation runs on that path and
    /// must never disturb ingestion. (The firing side-effect is a notifier
    /// dispatch, which has no observable endpoint here; the evaluator itself
    /// is covered by the domain unit tests.)
    #[tokio::test]
    async fn test_alert_rule_crud_and_evaluation_on_ingestion() {
        let (agent, internal, _db) = setup_test_app().await;

        let body = serde_json::json!({
            "metric": "cpu_pct",
            "threshold": 80.0,
            "for_seconds": 0,
            "service": "web",
        });
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/alerts")
                    .header("X-User-Id", TEST_USER)
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(created["data"]["metric"], "cpu_pct");
        assert_eq!(created["data"]["enabled"], true);
        let rule_id = created["data"]["id"].as_str().unwrap().to_string();

        // The rule shows up in the listing.
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/alerts")
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let listed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(1));

        // Invalid rules are rejected with the validation message.
        let bad = serde_json::json!({ "metric": "cpu_pct", "threshold": -5.0 });
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/alerts")
                    .header("X-User-Id", TEST_USER)
                    .header("Content-Type", "application/json")
                    .body(Body::from(bad.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Ingest a breaching sample through the agent router: evaluation runs
        // on this request (the rule fires with for_seconds=0) and ingestion
        // must still store the sample. Seed the state row first so the metric
        // insert resolves the service.
        let state_body = serde_json::json!({
            "project_name": "alerts-project",
            "payload": { "web": { "inspect": {} } }
        });
        let response = agent
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/container/state/test-host/alerts-project")
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(state_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let metrics_body = serde_json::json!({
            "project_name": "alerts-project",
            "payload": { "web": { "cpu_pct": 95.0, "mem_bytes": 1000, "mem_limit_bytes": 2000 } }
        });
        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/container/metrics/test-host/alerts-project")
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(metrics_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(latest_metrics(&internal).await.len(), 1);

        // Disable, delete, and confirm the second delete 404s.
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/alerts/{rule_id}/enabled"))
                    .header("X-User-Id", TEST_USER)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"enabled": false}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let response = internal
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/alerts/{rule_id}"))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let response = internal
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/alerts/{rule_id}"))
                    .header("X-User-Id", TEST_USER)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ── Projects and sharing ─────────────────────────────────────────────────

    /// A co-maintainer and an unrelated user, both only ever seen through the
    /// internal router's `X-User-Id`.
    const MEMBER: &str = "user_member";
    const OUTSIDER: &str = "user_outsider";

    /// One internal-router call as `user`; returns the status and the JSON
    /// body (`Null` for empty bodies).
    async fn call(
        internal: &Router,
        method: &str,
        uri: &str,
        user: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("X-User-Id", user);
        let body = match body {
            Some(json) => {
                request = request.header("Content-Type", "application/json");
                Body::from(json.to_string())
            }
            None => Body::empty(),
        };
        let response = internal
            .clone()
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    /// Report a running "web" service for `project` through the agent router
    /// (owned by TEST_USER) and return the project id from the overview.
    async fn seed_project(agent: &Router, internal: &Router, project: &str) -> String {
        let state_body = serde_json::json!({
            "project_name": project,
            "payload": { "web": { "inspect": {
                "State": { "Status": "running", "Health": { "Status": "healthy" } },
                "Config": { "Image": "nginx:latest" }
            } } }
        });
        let response = agent
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/container/state/test-host/{project}"))
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(state_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let (status, projects) = call(internal, "GET", "/projects", TEST_USER, None).await;
        assert_eq!(status, StatusCode::OK);
        projects["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == project)
            .expect("seeded project is listed")["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    /// Invite existing user `member` to the project (as the owner) and accept
    /// the invitation as them.
    async fn share_with(internal: &Router, project_id: &str, member: &str) {
        let (status, invited) = call(
            internal,
            "POST",
            &format!("/projects/{project_id}/invitations"),
            TEST_USER,
            Some(
                serde_json::json!({ "email": format!("{member}@example.com"), "user_id": member }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let invitation_id = invited["data"]["invitation"]["id"].as_str().unwrap();
        let (status, _) = call(
            internal,
            "POST",
            &format!("/invitations/{invitation_id}/accept"),
            member,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_existing_users_must_accept_invitations() {
        let (agent, internal, _db) = setup_test_app().await;
        let id = seed_project(&agent, &internal, "shop").await;
        let invitations = format!("/projects/{id}/invitations");
        let invite = serde_json::json!({ "email": "member@example.com", "user_id": MEMBER });

        let (status, invited) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(invite.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(invited["data"]["created"], true);
        assert_eq!(invited["data"]["invitation"]["user_id"], MEMBER);
        let (_, again) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(invite.clone()),
        )
        .await;
        assert_eq!(again["data"]["created"], false);

        // Being invited grants nothing until the invitee agrees.
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(0));
        let (status, _) = call(&internal, "GET", &format!("/projects/{id}"), MEMBER, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (_, received) = call(&internal, "GET", "/invitations", MEMBER, None).await;
        let invitation_id = received["data"][0]["id"].as_str().unwrap().to_string();
        let (_, others) = call(&internal, "GET", "/invitations", OUTSIDER, None).await;
        assert_eq!(others["data"].as_array().map(|a| a.len()), Some(0));

        // Declining drops the invitation, and only the addressee can do it.
        let decline = format!("/invitations/{invitation_id}/decline");
        let (status, _) = call(&internal, "POST", &decline, OUTSIDER, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = call(&internal, "POST", &decline, MEMBER, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (_, received) = call(&internal, "GET", "/invitations", MEMBER, None).await;
        assert_eq!(received["data"].as_array().map(|a| a.len()), Some(0));
        let (_, detail) = call(
            &internal,
            "GET",
            &format!("/projects/{id}"),
            TEST_USER,
            None,
        )
        .await;
        assert_eq!(
            detail["data"]["invitations"].as_array().map(|a| a.len()),
            Some(0)
        );
        let (status, _) = call(
            &internal,
            "POST",
            &format!("/invitations/{invitation_id}/accept"),
            MEMBER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Invited again, they accept and get in; inviting a member again is
        // refused, as is inviting the owner.
        share_with(&internal, &id, MEMBER).await;
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"][0]["role"], "member");
        let (status, _) = call(&internal, "POST", &invitations, TEST_USER, Some(invite)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let (status, _) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(serde_json::json!({ "email": "owner@example.com", "user_id": TEST_USER })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        // An address invited before its owner signed up gets addressed to the
        // account once the owner re-invites it as an existing user.
        let (_, first) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(serde_json::json!({ "email": "later@example.com" })),
        )
        .await;
        assert_eq!(
            first["data"]["invitation"]["user_id"],
            serde_json::Value::Null
        );
        let (_, second) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(serde_json::json!({ "email": "later@example.com", "user_id": OUTSIDER })),
        )
        .await;
        assert_eq!(second["data"]["created"], false);
        assert_eq!(second["data"]["invitation"]["user_id"], OUTSIDER);
        let (_, received) = call(&internal, "GET", "/invitations", OUTSIDER, None).await;
        assert_eq!(received["data"].as_array().map(|a| a.len()), Some(1));
    }

    #[tokio::test]
    async fn test_project_overview_summarises_services_and_latest_rollout() {
        let (agent, internal, _db) = setup_test_app().await;
        let id = seed_project(&agent, &internal, "shop").await;

        let deployment = CreateDeployment {
            project: ProjectName::new("shop"),
            service: ServiceName::new("web"),
            image: ImageName::new("nginx:latest"),
            digest: ImageDigest::new("sha256:new"),
            status: DeploymentStatus::Success,
            hostname: HostName::new("test-host"),
            logs: None,
        };
        let response = agent
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/deployments")
                    .header("Authorization", "Bearer tests-secret")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&deployment).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let (status, body) = call(&internal, "GET", "/projects", TEST_USER, None).await;
        assert_eq!(status, StatusCode::OK);
        let projects = body["data"].as_array().unwrap();
        assert_eq!(projects.len(), 1);
        let project = &projects[0];
        assert_eq!(project["id"], id.as_str());
        assert_eq!(project["hostname"], "test-host");
        assert_eq!(project["role"], "owner");
        assert_eq!(project["services"][0]["name"], "web");
        assert_eq!(project["services"][0]["status"], "running");
        assert_eq!(project["services"][0]["health"], "healthy");
        assert_eq!(project["services"][0]["image"], "nginx:latest");
        assert_eq!(project["latest_deployment"]["digest"], "sha256:new");
        assert_eq!(project["pending_updates"], 0);
        assert_eq!(project["member_count"], 0);

        // Project-scoped service and deployment reads work for the owner.
        let (status, services) = call(
            &internal,
            "GET",
            &format!("/projects/{id}/services"),
            TEST_USER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(services.as_array().map(|a| a.len()), Some(1));
        let (status, deployments) = call(
            &internal,
            "GET",
            &format!("/projects/{id}/services/web/deployments"),
            TEST_USER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(deployments["data"].as_array().map(|a| a.len()), Some(1));
    }

    #[tokio::test]
    async fn test_shared_project_is_readable_by_members_but_administered_by_the_owner() {
        let (agent, internal, _db) = setup_test_app().await;
        let id = seed_project(&agent, &internal, "shop").await;
        let project = format!("/projects/{id}");

        // Before sharing, the project is invisible to anyone else — and
        // indistinguishable from a project that doesn't exist.
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(0));
        let (status, _) = call(&internal, "GET", &project, MEMBER, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        share_with(&internal, &id, MEMBER).await;

        // The member sees it, with the owner's live state.
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        let listed = listed["data"].as_array().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["role"], "member");
        assert_eq!(listed[0]["services"][0]["status"], "running");
        let (status, service) = call(
            &internal,
            "GET",
            &format!("{project}/services/web"),
            MEMBER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(service["data"]["service_name"], "web");
        let (status, detail) = call(&internal, "GET", &project, MEMBER, None).await;
        assert_eq!(status, StatusCode::OK);
        let members = detail["data"]["members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0]["user_id"], TEST_USER);
        assert_eq!(members[0]["role"], "owner");
        assert_eq!(members[1]["user_id"], MEMBER);

        // …but may neither invite others nor delete it.
        let (status, _) = call(
            &internal,
            "POST",
            &format!("{project}/invitations"),
            MEMBER,
            Some(serde_json::json!({ "email": "outsider@example.com", "user_id": OUTSIDER })),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        let (status, _) = call(
            &internal,
            "POST",
            &format!("{project}/invitations"),
            MEMBER,
            Some(serde_json::json!({ "email": "someone@example.com" })),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        let (status, _) = call(&internal, "DELETE", &project, MEMBER, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // Outsiders still can't see it, and a member can't remove others.
        let (status, _) = call(&internal, "GET", &project, OUTSIDER, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = call(
            &internal,
            "DELETE",
            &format!("{project}/members/{TEST_USER}"),
            MEMBER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);

        // A member can leave.
        let (status, _) = call(
            &internal,
            "DELETE",
            &format!("{project}/members/{MEMBER}"),
            MEMBER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(0));
    }

    #[tokio::test]
    async fn test_invitations_are_claimed_by_email_once() {
        let (agent, internal, _db) = setup_test_app().await;
        let id = seed_project(&agent, &internal, "shop").await;
        let invitations = format!("/projects/{id}/invitations");

        let (status, invited) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(serde_json::json!({ "email": " Member@Example.com " })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(invited["data"]["created"], true);
        assert_eq!(invited["data"]["invitation"]["email"], "member@example.com");

        // Re-inviting the same address doesn't create (or re-send) anything.
        let (_, again) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(serde_json::json!({ "email": "member@example.com" })),
        )
        .await;
        assert_eq!(again["data"]["created"], false);

        let (status, _) = call(
            &internal,
            "POST",
            &invitations,
            TEST_USER,
            Some(serde_json::json!({ "email": "not-an-address" })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        // Signing up with a different address claims nothing.
        let (_, claimed) = call(
            &internal,
            "POST",
            "/invitations/claim",
            OUTSIDER,
            Some(serde_json::json!({ "emails": ["outsider@example.com"] })),
        )
        .await;
        assert_eq!(
            claimed["data"]["project_ids"].as_array().map(|a| a.len()),
            Some(0)
        );

        // Signing up with the invited (verified) address addresses the
        // invitation to the new account — but grants nothing yet.
        let (status, claimed) = call(
            &internal,
            "POST",
            "/invitations/claim",
            MEMBER,
            Some(serde_json::json!({ "emails": ["MEMBER@example.com"] })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(claimed["data"]["project_ids"][0], id.as_str());
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(0));
        let (_, received) = call(&internal, "GET", "/invitations", MEMBER, None).await;
        let received = received["data"].as_array().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0]["project_name"], "shop");
        assert_eq!(received[0]["invited_by"], TEST_USER);
        let invitation_id = received[0]["id"].as_str().unwrap().to_string();

        // Only the addressee can answer it.
        let (status, _) = call(
            &internal,
            "POST",
            &format!("/invitations/{invitation_id}/accept"),
            OUTSIDER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, accepted) = call(
            &internal,
            "POST",
            &format!("/invitations/{invitation_id}/accept"),
            MEMBER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(accepted["data"]["project_id"], id.as_str());
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(1));

        // The invitation is used up.
        let (_, detail) = call(
            &internal,
            "GET",
            &format!("/projects/{id}"),
            TEST_USER,
            None,
        )
        .await;
        assert_eq!(
            detail["data"]["invitations"].as_array().map(|a| a.len()),
            Some(0)
        );
        let (_, claimed) = call(
            &internal,
            "POST",
            "/invitations/claim",
            OUTSIDER,
            Some(serde_json::json!({ "emails": ["member@example.com"] })),
        )
        .await;
        assert_eq!(
            claimed["data"]["project_ids"].as_array().map(|a| a.len()),
            Some(0)
        );
    }

    #[tokio::test]
    async fn test_project_notifiers_are_shared_with_members_and_kept_off_the_account_list() {
        let (agent, internal, _db) = setup_test_app().await;
        let id = seed_project(&agent, &internal, "shop").await;
        let notifiers = format!("/projects/{id}/notifiers");
        let telegram = serde_json::json!({ "kind": "telegram", "bot_token": "t", "chat_id": 1 });

        share_with(&internal, &id, MEMBER).await;

        // Owner and member each add one; both land on the project.
        let (status, _) = call(
            &internal,
            "POST",
            &notifiers,
            TEST_USER,
            Some(telegram.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (status, created) = call(
            &internal,
            "POST",
            &notifiers,
            MEMBER,
            Some(telegram.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let member_notifier = created["data"]["id"].as_str().unwrap().to_string();

        let (_, listed) = call(&internal, "GET", &notifiers, MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(2));

        // Project notifiers don't show up among (or get managed through) the
        // owner's account-wide notifiers.
        let (_, account) = call(&internal, "GET", "/notifiers", TEST_USER, None).await;
        assert_eq!(account["data"].as_array().map(|a| a.len()), Some(0));
        let (status, _) = call(
            &internal,
            "DELETE",
            &format!("/notifiers/{member_notifier}"),
            TEST_USER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Outsiders can neither list nor add.
        let (status, _) = call(&internal, "GET", &notifiers, OUTSIDER, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = call(&internal, "POST", &notifiers, OUTSIDER, Some(telegram)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Members can toggle and delete project notifiers.
        let (status, _) = call(
            &internal,
            "PATCH",
            &format!("{notifiers}/{member_notifier}/enabled"),
            MEMBER,
            Some(serde_json::json!({ "enabled": false })),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, _) = call(
            &internal,
            "DELETE",
            &format!("{notifiers}/{member_notifier}"),
            MEMBER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // Deleting the project takes its notifiers and memberships with it.
        let (status, _) = call(
            &internal,
            "DELETE",
            &format!("/projects/{id}"),
            TEST_USER,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (_, listed) = call(&internal, "GET", "/projects", MEMBER, None).await;
        assert_eq!(listed["data"].as_array().map(|a| a.len()), Some(0));
        let (status, _) = call(&internal, "GET", &notifiers, TEST_USER, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
