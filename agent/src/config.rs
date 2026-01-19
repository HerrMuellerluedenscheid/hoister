use serde::Deserialize;
use std::path::Path;

use figment2::{
    Figment,
    providers::{Env, Format, Toml},
};
use reqwest::Url;
use shared::ProjectName;

type ChannelId = u64;
type ChannelName = String;
type ChatId = u64;
type BotToken = String;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Registry {
    pub(crate) ghcr: Option<GithubRegistry>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GithubRegistry {
    pub(crate) username: String,
    pub(crate) token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Dispatcher {
    pub(crate) telegram: Option<Telegram>,
    pub(crate) discord: Option<Discord>,
    pub(crate) slack: Option<Slack>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Telegram {
    pub(crate) chat: ChatId,
    pub(crate) token: BotToken,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Discord {
    pub(crate) token: BotToken,
    pub(crate) channel: ChannelId,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Slack {
    pub(crate) webhook: Url,
    pub(crate) channel: ChannelName,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Schedule {
    pub(crate) interval: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Controller {
    pub(crate) url: Url,
    pub(crate) token: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) project: Option<ProjectName>,
    #[serde(default)]
    pub(crate) send_test_message: bool,
    pub(crate) schedule: Schedule,
    pub(crate) registry: Option<Registry>,
    pub(crate) controller: Option<Controller>,
    pub(crate) dispatcher: Option<Dispatcher>,
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
    use figment2::Jail;
    Jail::expect_with(|jail: &mut Jail| {
        jail.create_file(
            "config-test.toml",
            r#"
            [schedule]
            interval=10

            [registry.ghcr]
            username="foo"
            token="ghc_asdfasdf"

            [dispatcher.telegram]
            token="123456789:qwertyuiopasdfghjkl"
            chat=123456789

            [dispatcher.slack]
            webhook="https://hooks.slack.com/xxx/xx"
            channel="channel-name"

            [dispatcher.discord]
            token="foo"
            channel="getsoverriddenbyenvvar"
            "#,
        )?;

        jail.set_env("HOISTER_registry_ghcr_username", "xxx");
        jail.set_env("HOISTER_schedule_name", "bar");
        jail.set_env("HOISTER_CONTROLLER_URL", "http://foobar:3033");
        jail.set_env("HOISTER_DISPATCHER_discord_token", "discord_token");
        jail.set_env("HOISTER_dispatcher_discord_channel", "123123");

        let config_path = "config-test.toml";

        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = rt.block_on(load_config(config_path.as_ref()));

        let dispatcher = config.dispatcher.unwrap();
        assert_eq!(config.registry.unwrap().ghcr.unwrap().username, "xxx");
        assert_eq!(
            config.controller.unwrap().url,
            Url::from("http://foobar:3033".parse().unwrap())
        );
        assert_eq!(
            dispatcher.discord.as_ref().unwrap().token,
            "discord_token".to_string()
        );
        assert_eq!(dispatcher.discord.as_ref().unwrap().channel, 123123,);
        assert_eq!(
            dispatcher.slack.as_ref().unwrap().channel,
            "channel-name".to_string()
        );

        Ok(())
    });
}
