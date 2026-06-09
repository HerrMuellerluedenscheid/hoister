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

    use controller::domain::billing::service::Service as BillingService;
    use controller::domain::container_state::service::Service as ContainerStateService;
    use controller::domain::deployments::models::deployment::{
        CreateDeploymentRequest, Deployment,
    };
    use controller::domain::deployments::ports::DeploymentsService as _;
    use controller::domain::deployments::service::Service as DeploymentsService;
    use controller::domain::metrics::service::Service as MetricsService;
    use controller::domain::notifiers::service::Service as NotifierService;
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
            metrics_service: Arc::new(MetricsService::new(db)),
            #[cfg(feature = "self-hosted")]
            api_secret: Some("tests-secret".to_string()),
            event_tx,
            pending_updates: Default::default(),
            email: None,
            dashboard_url: "https://hoister.io".to_string(),
        };
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
}
