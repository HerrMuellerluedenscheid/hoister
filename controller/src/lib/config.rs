use figment2::{Figment, providers::Env};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_secret: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
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

pub fn get_config() -> Config {
    let config: Config = Figment::new()
        .merge(Env::prefixed("HOISTER_CONTROLLER_"))
        .extract()
        .expect("Failed to read configuration from environment variables.");
    config
}
