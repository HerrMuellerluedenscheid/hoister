#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use hoister_shared::{
        CreateDeployment, DeploymentStatus, ImageDigest, ImageName, ProjectName, ServiceName,
    };
    use std::collections::HashMap;

    use controller::config::Config;
    use controller::domain::deployments::models::deployment::{
        CreateDeploymentRequest, Deployment,
    };
    use controller::domain::deployments::ports::DeploymentsService;
    use controller::domain::deployments::service::Service;
    use controller::inbound::server::{ApiResponse, AppState, create_app};
    use controller::outbound::sqlite::Sqlite;
    use controller::sse::ControllerEvent;
    use std::sync::Arc;
    use tokio::sync::{RwLock, broadcast};
    use tower::ServiceExt;
    // for `oneshot` and `ready`

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

    async fn setup_test_app() -> (Router, Config) {
        let config = Config {
            api_secret: Some("tests-secret".to_string()),
            port: 3034,
            database_path: unique_db_path(),
        };
        let (event_tx, _) = broadcast::channel::<ControllerEvent>(100);

        let deployments_service = get_service(&config).await;
        let state = AppState {
            deployments_service: Arc::new(deployments_service),
            container_state: Arc::new(RwLock::new(HashMap::new())),
            api_secret: config.api_secret.clone(),
            event_tx,
        };

        let app = create_app(state).await;
        (app, config)
    }

    #[tokio::test]
    async fn test_health_endpoint_no_auth_required() {
        let (app, _config) = setup_test_app().await;

        let response = app
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
    async fn test_get_deployments_requires_auth() {
        let (app, _config) = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/deployments")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_deployments_with_valid_auth() {
        let (app, _config) = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/deployments")
                    .header("Authorization", "Bearer tests-secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        //     .await
        //     .unwrap();
        // let response: ApiResponse<Vec<Deployment>> =
        //     serde_json::from_slice(&body).unwrap();
        //
        // assert!(response.success);
        // assert!(response.data.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_create_and_get_deployment() {
        let (app, _config) = setup_test_app().await;

        // Create a deployment
        let payload = CreateDeployment {
            project: ProjectName::new("tests-project"),
            service: ServiceName::new("tests-service"),
            image: ImageName::new("nginx:latest"),
            digest: ImageDigest::new("sha256:abc123"),
            status: DeploymentStatus::Pending,
        };

        let response = app
            .clone()
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
        //
        // let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        //     .await
        //     .unwrap();
        // let create_response: ApiResponse<Deployment> =
        //     serde_json::from_slice(&body).unwrap();
        //
        // assert!(create_response.success);
        // let created_deployment = create_response.data.unwrap();
        // assert_eq!(created_deployment.digest.as_str(), "sha256:abc123");
        //
        // // Get all deployments
        // let response = app
        //     .oneshot(
        //         Request::builder()
        //             .uri("/deployments")
        //             .header("Authorization", "Bearer tests-secret")
        //             .body(Body::empty())
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();
        //
        // assert_eq!(response.status(), StatusCode::OK);
        //
        // let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        //     .await
        //     .unwrap();
        // let list_response: ApiResponse<Vec<Deployment>> =
        //     serde_json::from_slice(&body).unwrap();
        //
        // assert!(list_response.success);
        // let deployments = list_response.data.unwrap();
        // assert_eq!(deployments.len(), 1);
        // assert_eq!(deployments[0].id, created_deployment.id);
    }

    #[tokio::test]
    async fn test_get_deployment_by_image() {
        let (app, config) = setup_test_app().await;
        let image_name = ImageName::new("aaa");
        let service_name = ServiceName::new("tests-service");
        let project_name = ProjectName::new("tests-project");
        // Create a deployment first

        let database_service = get_service(&config).await;
        let req = CreateDeploymentRequest {
            project_name: project_name.clone(),
            service_name: service_name.clone(),
            image_name: image_name.clone(),
            image_digest: ImageDigest::new("sha256:abc123"),
            deployment_status: DeploymentStatus::Pending,
        };
        database_service.create_deployment(&req).await.unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/deployments/{}/{}",
                        project_name.as_str(),
                        service_name.as_str()
                    ))
                    .header("Authorization", "Bearer tests-secret")
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
        let (app, _config) = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
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
        let (app, _config) = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
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
