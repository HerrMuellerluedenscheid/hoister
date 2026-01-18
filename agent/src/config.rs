use serde::Deserialize;
use std::path::Path;

use figment2::{
    Figment,
    providers::{Env, Format, Toml},
};

#[derive(Deserialize, Debug)]
struct Registries {
    pub(crate) ghcr: Option<GithubRegistry>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct GithubRegistry {
    pub(crate) username: String,
    pub(crate) token: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Schedule {
    pub(crate) interval: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) send_test_message: bool,
    pub(crate) schedule: Schedule,
    pub(crate) registries: Option<Registries>,
}

pub(crate) async fn load_config(config_path: &Path) -> Config {
    let config: Config = Figment::new()
        .merge(Toml::file(config_path))
        .merge(Env::prefixed("HOISTER_").split("_"))
        .extract()
        .expect("Failed to load config");
    config
}
#[test]
fn test_load_config() {
    Figment::Jail::expect_with(|jail: &mut Figment::Jail| {
        jail.create_file(
            "config-test.toml",
            r#"
            [schedule]
            interval=10

            [registries.ghcr]
            username="foo"
            token="ghc_asdfasdf"
            "#,
        )?;

        jail.set_env("HOISTER_registries_ghcr_username", "bar");
        jail.set_env("HOISTER_schedule_name", "bar");
        let config_path = "config-test.toml";

        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = rt.block_on(load_config(config_path.as_ref()));

        assert_eq!(config.registries.unwrap().ghcr.unwrap().username, "bar");

        Ok(())
    });
}
