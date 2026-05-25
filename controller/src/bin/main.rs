use controller::config::get_config;
use controller::domain::container_state::service::Service as ContainerStateService;
use controller::domain::deployments::service::Service as DeploymentsService;
use controller::domain::tokens::service::Service as TokenService;
use controller::inbound::server::{AppState, create_agent_router, create_internal_router};
use controller::outbound::Database;
use controller::outbound::pending_updates_memory::PendingUpdatesMemory;
use controller::outbound::state_memory::StateMemory;
use controller::sse::UserScopedEvent;
use env_logger::Env;
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = get_config();

    let (event_tx, _) = broadcast::channel::<UserScopedEvent>(100);
    let db = Database::connect(&config.database_path).await?;

    let pending_updates = PendingUpdatesMemory::default();
    let state = AppState {
        deployments_service: Arc::new(DeploymentsService::new(db.clone())),
        container_state_service: Arc::new(ContainerStateService::new(StateMemory::default())),
        token_service: Arc::new(TokenService::new(db)),
        #[cfg(feature = "self-hosted")]
        api_secret: config.api_secret.clone(),
        event_tx,
        pending_updates,
    };
    let agent_app = create_agent_router(state.clone()).await;
    let internal_app = create_internal_router(state).await;
    info!(
        "starting internal listener on {}:{}",
        config.internal_bind_addr, config.internal_port
    );
    let internal_listener = TcpListener::bind(format!(
        "{}:{}",
        config.internal_bind_addr, config.internal_port
    ))
    .await
    .map_err(|e| {
        warn!("Failed to bind internal listener: {}", e);
        e
    })?;

    info!("starting agent listener on 0.0.0.0:{}", config.port);
    let agent_listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .map_err(|e| {
            warn!("Failed to bind agent listener: {}", e);
            e
        })?;

    info!(
        "Internal router on http://{}:{} (VPC-only, no auth)",
        config.internal_bind_addr, config.internal_port
    );

    let internal_server = async {
        axum::serve(internal_listener, internal_app)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    };

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

        info!("Agent router on https://0.0.0.0:{} (TLS)", config.port);
        log_endpoints(&config);

        let agent_server = async move {
            loop {
                let (tcp_stream, remote_addr) = agent_listener.accept().await?;
                let tls_acceptor = tls_acceptor.clone();
                let app = agent_app.clone();

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
                        async move { <_ as tower::Service<_>>::call(&mut svc, req).await }
                    });

                    if let Err(e) = Builder::new(TokioExecutor::new())
                        .serve_connection(TokioIo::new(tls_stream), service)
                        .await
                    {
                        log::debug!("Connection error from {remote_addr}: {e}");
                    }
                });
            }
            #[allow(unreachable_code)]
            Ok::<(), Box<dyn std::error::Error>>(())
        };

        tokio::try_join!(internal_server, agent_server)?;
    } else {
        info!("Agent router on http://0.0.0.0:{}", config.port);
        log_endpoints(&config);

        let agent_server = async {
            axum::serve(agent_listener, agent_app)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        };

        tokio::try_join!(internal_server, agent_server)?;
    }

    Ok(())
}

fn log_endpoints(config: &controller::config::Config) {
    info!("Agent router endpoints (require Authorization: Bearer hst_<token>):");
    info!("  GET    /health                 - health check (no auth)");
    info!("  GET    /sse                    - server-sent events");
    info!("  POST   /deployments            - create deployment");
    info!("  POST   /container/state/...    - update container state");
    info!(
        "Internal router endpoints (VPC-only, http://0.0.0.0:{}, X-User-Id header):",
        config.internal_port
    );
    info!("  GET    /health                 - health check");
    info!("  GET    /token                  - get or create agent token");
    info!("  GET    /deployments            - list deployments");
    info!("  GET    /deployments/:p/:s      - list deployments by service");
    info!("  GET    /container/state        - all container states");
    info!("  GET    /container/state/...    - container state by service");
}
