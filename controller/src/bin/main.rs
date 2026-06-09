use controller::config::get_config;
use controller::domain::billing::service::Service as BillingServiceImpl;
use controller::domain::container_state::service::Service as ContainerStateService;
use controller::domain::deployments::service::Service as DeploymentsService;
use controller::domain::metrics::service::Service as MetricsService;
use controller::domain::notifiers::service::Service as NotifierService;
use controller::domain::tokens::service::Service as TokenService;
use controller::inbound::server::{
    AppState, InternalSecret, create_agent_router, create_internal_router,
};
use controller::outbound::Database;
use controller::outbound::pending_updates_memory::PendingUpdatesMemory;
use controller::sse::UserScopedEvent;
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing-subscriber (not env_logger) so per-request spans propagate their
    // fields — e.g. user_id from the audit middleware — onto every event logged
    // while serving that request. `.init()` also installs the tracing-log
    // bridge, so existing `log::` calls keep flowing. Honours RUST_LOG, default
    // `info`, same as before.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let config = get_config();

    let (event_tx, _) = broadcast::channel::<UserScopedEvent>(100);

    // HOISTER_CONTROLLER_TOKEN_PEPPER is the HMAC key for agent-token storage.
    // Without it, hashes degrade to unsalted SHA-256, so a DB read alone is
    // enough to verify a stolen token. The cloud build fails closed; the
    // self-hosted build only warns (single-tenant local dev).
    let token_pepper = config.token_pepper.clone().unwrap_or_default();
    if token_pepper.is_empty() {
        #[cfg(not(feature = "self-hosted"))]
        return Err(
            "Refusing to start: HOISTER_CONTROLLER_TOKEN_PEPPER is unset. Agent \
             tokens would be stored under an unsalted SHA-256 hash, so a read of \
             the database alone would be enough to verify a stolen token. Set \
             the env var to a long random value."
                .into(),
        );

        #[cfg(feature = "self-hosted")]
        warn!(
            "HOISTER_CONTROLLER_TOKEN_PEPPER is not set. Agent tokens will be \
             stored under an unsalted SHA-256 hash — a DB read alone is then \
             enough to verify a stolen token. Set the env var to a long random \
             value in production."
        );
    }
    let aead = controller::outbound::secrets::Aead::from_base64_or_passthrough(
        config.notifier_key.as_deref(),
    )?;
    if !aead.is_active() {
        warn!(
            "HOISTER_CONTROLLER_NOTIFIER_KEY is not set. Notifier secrets \
             (Slack webhook URLs, Telegram/Discord bot tokens, SMTP passwords, \
             etc.) will be stored in plaintext. Set the env var to a base64 \
             32-byte random value in production; back the key up offline."
        );
    }
    let db = Database::connect(&config.database_path, token_pepper.into_bytes(), aead).await?;

    let pending_updates = PendingUpdatesMemory::default();

    // Email (Resend) delivery is controller-wide: users supply only a
    // recipient. Both the API key and the From identity must be present; a
    // half-configured pair disables email rather than failing at dispatch.
    let email = match (config.resend_api_key.clone(), config.email_from.clone()) {
        (Some(api_key), Some(from)) if !api_key.is_empty() && !from.is_empty() => Some(
            controller::outbound::notification_dispatch::EmailDispatchConfig {
                resend_api_key: api_key,
                from,
            },
        ),
        (None, None) => None,
        _ => {
            warn!(
                "Email notifier delivery is disabled: set BOTH \
                 HOISTER_CONTROLLER_RESEND_API_KEY and \
                 HOISTER_CONTROLLER_EMAIL_FROM to enable it."
            );
            None
        }
    };

    let state = AppState {
        deployments_service: Arc::new(DeploymentsService::new(db.clone())),
        container_state_service: Arc::new(ContainerStateService::new(db.clone())),
        token_service: Arc::new(TokenService::new(db.clone())),
        notifier_service: Arc::new(NotifierService::new(db.clone())),
        billing_service: Arc::new(BillingServiceImpl::new(db.clone())),
        metrics_service: Arc::new(MetricsService::new(db)),
        #[cfg(feature = "self-hosted")]
        api_secret: config.api_secret.clone(),
        event_tx,
        pending_updates,
        email,
        dashboard_url: config.dashboard_url.clone(),
    };
    // Treat an empty secret as unset everywhere downstream (the middleware's
    // X-Internal-Auth gate would otherwise just require an empty, trivially
    // forged header).
    let internal_secret = InternalSecret(config.internal_secret.clone().filter(|s| !s.is_empty()));

    // Fail closed: the internal router trusts the caller-supplied X-User-Id
    // header, so an X-Internal-Auth secret is the ONLY thing protecting it
    // once it is reachable beyond loopback. If the listener binds to a
    // non-loopback interface (e.g. 0.0.0.0 for a docker-compose sibling) we
    // refuse to start without a secret rather than silently exposing every
    // tenant to anything that can reach the port.
    let bind_is_loopback = config
        .internal_bind_addr
        .parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false);
    if !bind_is_loopback && internal_secret.0.as_deref().unwrap_or_default().is_empty() {
        return Err(format!(
            "Refusing to start: internal listener binds to {addr} (non-loopback) \
             but HOISTER_CONTROLLER_INTERNAL_SECRET is unset. Anything that can \
             reach {addr}:{port} could impersonate any user via the X-User-Id \
             header. Set a long random secret, or bind internal_bind_addr to \
             127.0.0.1.",
            addr = config.internal_bind_addr,
            port = config.internal_port,
        )
        .into());
    }

    let internal_auth_enabled = internal_secret.0.is_some();
    let agent_app = create_agent_router(state.clone()).await;
    let internal_app = create_internal_router(state, internal_secret).await;
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
        "Internal router on http://{}:{} ({})",
        config.internal_bind_addr,
        config.internal_port,
        if internal_auth_enabled {
            "X-Internal-Auth required"
        } else {
            "loopback-only, no auth"
        }
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
    info!("  POST   /container/metrics/...  - report container metrics (opt-in)");
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
    info!("  GET    /container/metrics      - latest metrics per container");
    info!("  GET    /container/metrics/...  - metric time series by service");
}
