use figment2::{Figment, providers::Env};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    #[cfg(feature = "self-hosted")]
    #[serde(default)]
    pub api_secret: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_internal_port")]
    pub internal_port: u16,
    /// Bind address for the internal (BFF-facing) listener. Defaults to
    /// loopback so the internal router cannot be reached from anywhere other
    /// than the same host. For docker-compose deployments where the frontend
    /// runs in a sibling container, set this to `0.0.0.0`.
    #[serde(default = "default_internal_bind_addr")]
    pub internal_bind_addr: String,
    /// Shared secret required from the BFF as the `X-Internal-Auth` header.
    /// When the internal listener has to bind to a non-loopback interface
    /// (e.g. docker bridge), this is the only thing standing between the
    /// internal router and any other host/container that can reach the
    /// port — set it to a long random value in production.
    #[serde(default)]
    pub internal_secret: Option<String>,
    /// Server-side pepper combined via HMAC-SHA256 with every agent token
    /// before storage. A DB dump alone is then insufficient to verify a
    /// stolen `hst_` token — the attacker also needs this value.
    ///
    /// Optional in dev (controller logs a warning and falls back to an
    /// unsalted hash). Required in production.
    #[serde(default)]
    pub token_pepper: Option<String>,
    /// Base64-encoded 32-byte AES-256-GCM key used to encrypt notifier
    /// `config` blobs at rest. Optional — when missing, configs are stored
    /// in plaintext (the controller logs a warning at startup). Required
    /// for cloud / multi-tenant deployments. See `outbound::secrets`.
    #[serde(default)]
    pub notifier_key: Option<String>,
    pub database_path: String,
    #[serde(default)]
    pub tls_cert_path: Option<PathBuf>,
    #[serde(default)]
    pub tls_key_path: Option<PathBuf>,
}

impl Config {
    /// Returns `Some((cert_path, key_path))` if both TLS paths are set, `None` if neither is set.
    /// Panics if only one of the two is configured.
    pub fn tls_config(&self) -> Option<(&PathBuf, &PathBuf)> {
        match (&self.tls_cert_path, &self.tls_key_path) {
            (Some(cert), Some(key)) => Some((cert, key)),
            (None, None) => None,
            _ => panic!(
                "Both HOISTER_CONTROLLER_TLS_CERT_PATH and HOISTER_CONTROLLER_TLS_KEY_PATH must be set, or neither"
            ),
        }
    }
}

fn default_port() -> u16 {
    3033
}

fn default_internal_port() -> u16 {
    3034
}

fn default_internal_bind_addr() -> String {
    "127.0.0.1".to_string()
}

pub fn get_config() -> Config {
    let config: Config = Figment::new()
        .merge(Env::prefixed("HOISTER_CONTROLLER_"))
        .extract()
        .expect("Failed to read configuration from environment variables.");
    config
}
