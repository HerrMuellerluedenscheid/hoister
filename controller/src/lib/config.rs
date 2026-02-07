use figment2::{Figment, providers::Env};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_secret: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_path: String,
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
