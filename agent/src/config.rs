use chrono::Utc;
use cron::Schedule as CronSchedule;
use figment2::{
    Figment,
    providers::{Env, Format, Toml},
};
use hoister_shared::{HostName, ProjectName};
use reqwest::Url;
use serde::Deserialize;
use std::path::Path;

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
    pub(crate) gotify: Option<Gotify>,
    pub(crate) email: Option<Email>,
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
pub(crate) struct Gotify {
    pub(crate) server: Url,
    pub(crate) token: String,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct SMTP {
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) server: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Email {
    pub(crate) smtp: SMTP,
    pub(crate) from: Option<String>,
    pub(crate) recipient: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Schedule {
    pub(crate) interval: Option<u64>,
    pub(crate) cron: Option<CronSchedule>,
}

impl Schedule {
    pub(crate) fn sleep(&self) -> std::time::Duration {
        let seconds = match &self.cron {
            None => self
                .interval
                .expect("Either `interval` or `cron` has to be defined"),
            Some(schedule) => {
                let upcoming = schedule.upcoming(Utc).next().unwrap();
                upcoming
                    .signed_duration_since(Utc::now())
                    .num_seconds()
                    .max(0) as u64
            }
        };

        std::time::Duration::from_secs(seconds)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Controller {
    pub(crate) url: Url,
    pub(crate) token: Option<String>,
    pub(crate) ca_cert_path: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) project: Option<ProjectName>,
    #[serde(default)]
    pub(crate) hostname: HostName,
    #[serde(default)]
    pub(crate) send_test_message: bool,
    pub(crate) schedule: Schedule,
    pub(crate) registry: Option<Registry>,
    pub(crate) controller: Option<Controller>,
    pub(crate) dispatcher: Option<Dispatcher>,
}

pub(crate) fn build_http_client(controller: &Option<Controller>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();

    if let Some(controller) = controller {
        if let Some(ca_path) = &controller.ca_cert_path {
            let pem = std::fs::read(ca_path)
                .unwrap_or_else(|e| panic!("Failed to read CA cert at {ca_path}: {e}"));
            let cert = reqwest::Certificate::from_pem(&pem)
                .unwrap_or_else(|e| panic!("Failed to parse CA cert at {ca_path}: {e}"));
            builder = builder.add_root_certificate(cert);
        }
    }

    builder.build().expect("Failed to build HTTP client")
}

pub(crate) async fn load_config(config_path: &Path) -> Config {
    let mut config: Config = Figment::new()
        .merge(Toml::file(config_path))
        .merge(Env::prefixed("HOISTER_").split("_"))
        .extract()
        .expect("Failed to load config");

    // Workaround: figment's split("_") splits HOISTER_CONTROLLER_CA_CERT_PATH into
    // controller.ca.cert.path instead of controller.ca_cert_path.
    if let Some(ref mut controller) = config.controller {
        if controller.ca_cert_path.is_none() {
            controller.ca_cert_path = std::env::var("HOISTER_CONTROLLER_CA_CERT_PATH").ok();
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_load_config() {
        use figment2::Jail;
        Jail::expect_with(|jail: &mut Jail| {
            jail.create_file(
                "config-test.toml",
                r#"
            [schedule]
            interval=10
            cron="0 * * * * * *"

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
                "http://foobar:3033".parse().unwrap()
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

    #[test]
    fn test_schedule_cron() {
        let expression = "0 * * * * * *";
        let schedule = Schedule {
            interval: Some(10),
            cron: Some(CronSchedule::from_str(expression).unwrap()),
        };
        assert!(schedule.sleep().as_secs() < 60); // any schedule should be less than 60 seconds
    }
}
