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

    use controller::config::Config;
    use controller::domain::container_state::service::Service as ContainerStateService;
    use controller::domain::deployments::models::deployment::{
        CreateDeploymentRequest, Deployment,
    };
    use controller::domain::deployments::ports::DeploymentsService;
    use controller::domain::deployments::service::Service;
    use controller::domain::tokens::service::Service as TokenService;
    use controller::inbound::server::{
        ApiResponse, AppState, create_agent_router, create_internal_router,
    };
    use controller::outbound::sqlite::Sqlite;
    use controller::outbound::state_memory::StateMemory;
    use controller::sse::ControllerEvent;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tower::ServiceExt;
    // for `oneshot` and `ready`

    /// The agent router maps the static api_secret to this synthetic user id,
    /// so deployments created via the agent API are owned by "local".
    const TEST_USER: &str = "local";

    fn unique_db_path() -> String {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/hoister_test_{}_{}", std::process::id(), id)
    }

    async fn get_service(config: &Config) -> Service<Sqlite> {
        let repo = Sqlite::new(&config.database_path)
            .await
            .expect("Failed to connect to database");
        repo.migrate().await.expect("Failed to run migrations");
        Service::new(repo)
    }

    /// Builds both routers (agent + internal) over a shared, migrated database.
    ///
    /// `main` split the old combined `create_app` into two routers with
    /// distinct auth: the agent router authenticates via the `Bearer` api
    /// secret, the internal router trusts the `X-User-Id` header (VPC-only).
    async fn setup_test_app() -> (Router, Router, Config) {
        let config = Config {
            api_secret: Some("tests-secret".to_string()),
            port: 3033,
            internal_port: 3034,
            database_path: unique_db_path(),
            tls_cert_path: None,
            tls_key_path: None,
        };
        let (event_tx, _) = broadcast::channel::<ControllerEvent>(100);

        let repo = Sqlite::new(&config.database_path)
            .await
            .expect("Failed to connect to database");
        repo.migrate().await.expect("Failed to run migrations");

        let deployments_service = Service::new(repo.clone());
        let token_service = TokenService::new(repo);
        let container_state_service = ContainerStateService::new(StateMemory::default());
        let state = AppState {
            deployments_service: Arc::new(deployments_service),
            container_state_service: Arc::new(container_state_service),
            token_service: Arc::new(token_service),
            api_secret: config.api_secret.clone(),
            event_tx,
            pending_updates: Default::default(),
        };

        let agent = create_agent_router(state.clone()).await;
        let internal = create_internal_router(state).await;
        (agent, internal, config)
    }

    #[tokio::test]
    async fn test_health_endpoint_no_auth_required() {
        let (agent, _internal, _config) = setup_test_app().await;

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
        let (agent, _internal, _config) = setup_test_app().await;

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
        let (_agent, internal, _config) = setup_test_app().await;

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
        let (agent, _internal, _config) = setup_test_app().await;

        let payload = CreateDeployment {
            project: ProjectName::new("tests-project"),
            service: ServiceName::new("tests-service"),
            image: ImageName::new("nginx:latest"),
            digest: ImageDigest::new("sha256:abc123"),
            status: DeploymentStatus::Pending,
            hostname: HostName::new("test-host"),
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
        let (_agent, internal, config) = setup_test_app().await;
        let image_name = ImageName::new("aaa");
        let service_name = ServiceName::new("tests-service");
        let project_name = ProjectName::new("tests-project");

        // Seed a deployment owned by TEST_USER directly through the repository.
        let database_service = get_service(&config).await;
        let req = CreateDeploymentRequest {
            project_name: project_name.clone(),
            service_name: service_name.clone(),
            image_name: image_name.clone(),
            image_digest: ImageDigest::new("sha256:abc123"),
            deployment_status: DeploymentStatus::Pending,
            hostname: HostName::new("test-host"),
            user_id: Some(TEST_USER.to_string()),
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
        let (agent, _internal, _config) = setup_test_app().await;

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
        let (agent, _internal, _config) = setup_test_app().await;

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
