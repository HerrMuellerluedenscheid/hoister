use controller::config::get_config;
use controller::domain::container_state::service::Service as ContainerStateService;
use controller::domain::deployments::service::Service as DeploymentsService;
use controller::inbound::server::{AppState, create_app};
use controller::outbound::sqlite::Sqlite;
use controller::outbound::state_memory::StateMemory;
use controller::sse::ControllerEvent;
use env_logger::Env;
use log::info;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = get_config();

    let (event_tx, _) = broadcast::channel::<ControllerEvent>(100);
    let deployments_repo = Sqlite::new(&config.database_path)
        .await
        .expect("Failed to connect to database");
    deployments_repo.migrate().await?;
    let deployments_service = DeploymentsService::new(deployments_repo);

    let container_state_repo = StateMemory::default();
    let container_state_service = ContainerStateService::new(container_state_repo);
    let state = AppState {
        deployments_service: Arc::new(deployments_service),
        container_state_service: Arc::new(container_state_service),
        api_secret: config.api_secret.clone(),
        event_tx,
    };

    let app = create_app(state).await;
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;

    if let Some((cert_path, key_path)) = config.tls_config() {
        use hyper_util::rt::{TokioExecutor, TokioIo};
        use hyper_util::server::conn::auto::Builder;
        use rustls::ServerConfig;
        use rustls::crypto::aws_lc_rs;
        use std::io::BufReader;
        use tokio_rustls::TlsAcceptor;

        aws_lc_rs::default_provider()
            .install_default()
            .expect("Failed to install default CryptoProvider");

        let cert_file = std::fs::File::open(cert_path)
            .unwrap_or_else(|e| panic!("Failed to open TLS cert file {cert_path:?}: {e}"));
        let key_file = std::fs::File::open(key_path)
            .unwrap_or_else(|e| panic!("Failed to open TLS key file {key_path:?}: {e}"));

        let certs: Vec<_> =
            rustls_pemfile::certs(&mut BufReader::new(cert_file)).collect::<Result<_, _>>()?;
        let key = rustls_pemfile::private_key(&mut BufReader::new(key_file))?
            .expect("No private key found in key file");

        let tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

        info!("Server running on https://0.0.0.0:{}", config.port);
        info!(
            "Health check: https://0.0.0.0:{}/health (no auth required)",
            config.port
        );
        log_endpoints();

        loop {
            let (tcp_stream, remote_addr) = listener.accept().await?;
            let tls_acceptor = tls_acceptor.clone();
            let app = app.clone();

            tokio::spawn(async move {
                let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        log::debug!("TLS handshake failed from {remote_addr}: {e}");
                        return;
                    }
                };

                let service = hyper::service::service_fn(move |req| {
                    let mut svc = app.clone();
                    async move {
                        // Poll readiness before calling
                        <_ as tower::Service<_>>::call(&mut svc, req).await
                    }
                });

                if let Err(e) = Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(tls_stream), service)
                    .await
                {
                    log::debug!("Connection error from {remote_addr}: {e}");
                }
            });
        }
    } else {
        info!("Server running on http://0.0.0.0:{}", config.port);
        info!(
            "Health check: http://0.0.0.0:{}/health (no auth required)",
            config.port
        );
        log_endpoints();

        axum::serve(listener, app).await?;
    }

    Ok(())
}

fn log_endpoints() {
    info!("Protected API endpoints (require Authorization: Bearer <secret>):");
    info!("  GET    /sse                   - server side events");
    info!("  GET    /deployments           - Get all deployments");
    info!("  POST   /deployments           - Create deployment");
    info!("  GET    /deployments/:id       - Get deployment by ID");
    info!("  PUT    /deployments/:id       - Update deployment");
}
