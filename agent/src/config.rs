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
    pub(crate) dockerhub: Option<DockerHubRegistry>,
    pub(crate) ecr: Option<EcrRegistry>,
    pub(crate) acr: Option<AcrRegistry>,
    pub(crate) gcr: Option<GcrRegistry>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GithubRegistry {
    pub(crate) username: String,
    pub(crate) token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct DockerHubRegistry {
    pub(crate) username: String,
    pub(crate) password: String,
}

/// AWS Elastic Container Registry. Credentials are used to dynamically fetch
/// a short-lived auth token from the ECR API via AWS Signature V4.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct EcrRegistry {
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
    pub(crate) region: String,
}

/// Azure Container Registry.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AcrRegistry {
    pub(crate) username: String,
    pub(crate) password: String,
}

/// Google Container Registry / Google Artifact Registry.
/// Use `_json_key` as username and the service account JSON as password,
/// or `oauth2accesstoken` as username with a short-lived access token.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GcrRegistry {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Dispatcher {
    pub(crate) telegram: Option<Telegram>,
    pub(crate) discord: Option<Discord>,
    pub(crate) discord_webhook: Option<DiscordWebhook>,
    pub(crate) teams: Option<Teams>,
    pub(crate) slack: Option<Slack>,
    pub(crate) gotify: Option<Gotify>,
    pub(crate) email: Option<Email>,
    pub(crate) ntfy: Option<Ntfy>,
    pub(crate) pushover: Option<Pushover>,
    pub(crate) matrix: Option<Matrix>,
    pub(crate) mattermost: Option<Mattermost>,
    pub(crate) rocketchat: Option<RocketChat>,
    pub(crate) google_chat: Option<GoogleChat>,
    pub(crate) webhook: Option<Webhook>,
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

/// Discord delivery via an incoming webhook
/// (`https://discord.com/api/webhooks/{id}/{token}`) — no bot token, and the
/// target channel is fixed when the webhook is created. `username` and
/// `avatar_url` optionally override the webhook's default identity.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct DiscordWebhook {
    pub(crate) webhook: Url,
    pub(crate) username: Option<String>,
    pub(crate) avatar_url: Option<Url>,
}

/// Microsoft Teams delivery via an incoming webhook — no app registration, and
/// the target channel is fixed when the webhook is created. Works with both the
/// Workflows (Power Automate) webhooks and the legacy connector webhooks.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Teams {
    pub(crate) webhook: Url,
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

/// ntfy delivery to a (possibly self-hosted) ntfy server. `access_token` is
/// only needed for reserved/protected topics.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Ntfy {
    pub(crate) server: Url,
    pub(crate) topic: String,
    pub(crate) access_token: Option<String>,
}

/// Pushover delivery. `token` is the application API token, `user` the
/// recipient user or group key; `device` optionally targets one device.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Pushover {
    pub(crate) token: String,
    pub(crate) user: String,
    pub(crate) device: Option<String>,
}

/// Matrix delivery via a homeserver access token to a joined room.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Matrix {
    pub(crate) homeserver: Url,
    pub(crate) access_token: String,
    pub(crate) room_id: String,
}

/// Mattermost delivery via an incoming webhook. `channel` and `username`
/// optionally override the webhook's defaults; a `channel` override only works
/// if the webhook was created with "allow channel override" enabled.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Mattermost {
    pub(crate) webhook: Url,
    pub(crate) channel: Option<String>,
    pub(crate) username: Option<String>,
}

/// Rocket.Chat delivery via an incoming webhook. `channel` and `alias`
/// optionally override the webhook's defaults.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RocketChat {
    pub(crate) webhook: Url,
    pub(crate) channel: Option<String>,
    pub(crate) alias: Option<String>,
}

/// Google Chat delivery via an incoming webhook — the target space is fixed
/// when the webhook is created (the URL carries the `key`/`token` pair).
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct GoogleChat {
    pub(crate) webhook: Url,
}

/// Generic webhook delivery — POSTs each event to `url`. `headers` carries any
/// auth headers and defaults to empty.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Webhook {
    pub(crate) url: Url,
    #[serde(default)]
    pub(crate) headers: std::collections::HashMap<String, String>,
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
    /// Controller URL. Defaults to the hosted instance at hoister.io so
    /// users only need to set `token` to enable the cloud dashboard. Override
    /// when running the controller yourself.
    #[serde(default = "default_controller_url")]
    pub(crate) url: Url,
    pub(crate) token: Option<String>,
    pub(crate) ca_cert_path: Option<String>,
}

pub(crate) fn default_controller_url() -> Url {
    Url::parse(DEFAULT_CONTROLLER_URL).expect("default controller URL parses")
}

pub(crate) const DEFAULT_CONTROLLER_URL: &str = "https://api.hoister.io";

fn default_true() -> bool {
    true
}

/// Parse a boolean-ish environment variable. Returns `None` when the variable
/// is unset (so the caller keeps the config-file / default value) and `Some`
/// when it is set to a recognised truthy/falsy value.
pub(crate) fn env_bool(name: &str) -> Option<bool> {
    let raw = std::env::var(name).ok()?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) project: Option<ProjectName>,
    #[serde(default)]
    pub(crate) hostname: HostName,
    #[serde(default)]
    pub(crate) send_test_message: bool,
    #[serde(default = "default_true")]
    pub(crate) auto_update: bool,
    /// Forward container log tails to the controller when a container is
    /// in a non-running state. Disabled by default — logs can contain
    /// secrets the keyword-based env redaction doesn't catch. Opt in by
    /// setting `HOISTER_REPORT_LOGS=true`.
    #[serde(default)]
    pub(crate) report_logs: bool,
    /// Collect per-container CPU/memory usage and forward it to the controller
    /// for time-series graphing. Enabled by default. Disable it by setting
    /// `report_metrics = false` in the config file or `HOISTER_REPORT_METRICS=false`.
    #[serde(default = "default_true")]
    pub(crate) report_metrics: bool,
    /// Extra case-insensitive substrings that mark an env-var key as sensitive,
    /// on top of the built-in heuristic list. Lets operators redact
    /// project-specific secrets (e.g. `license`, `pin`) the defaults miss.
    /// Set as a TOML array (`redact_keywords = ["license", "pin"]`) or as a
    /// comma-separated `HOISTER_REDACT_KEYWORDS` env var. Loaded at startup.
    #[serde(default)]
    pub(crate) redact_keywords: Vec<String>,
    pub(crate) schedule: Schedule,
    pub(crate) registry: Option<Registry>,
    pub(crate) controller: Option<Controller>,
    pub(crate) dispatcher: Option<Dispatcher>,
}

pub(crate) fn build_http_client(controller: &Option<Controller>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();

    if let Some(controller) = controller
        && let Some(ca_path) = &controller.ca_cert_path
    {
        let pem = std::fs::read(ca_path)
            .unwrap_or_else(|e| panic!("Failed to read CA cert at {ca_path}: {e}"));
        let cert = reqwest::Certificate::from_pem(&pem)
            .unwrap_or_else(|e| panic!("Failed to parse CA cert at {ca_path}: {e}"));
        builder = builder.add_root_certificate(cert);
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
    if let Some(ref mut controller) = config.controller
        && controller.ca_cert_path.is_none()
    {
        controller.ca_cert_path = std::env::var("HOISTER_CONTROLLER_CA_CERT_PATH").ok();
    }

    // Same quirk for the top-level snake_case flags: figment's split("_")
    // mangles HOISTER_REPORT_LOGS / HOISTER_REPORT_METRICS, so read them here.
    // When present the env var is authoritative (overrides the file/default),
    // and may turn a setting either on or off.
    if let Some(v) = env_bool("HOISTER_REPORT_LOGS") {
        config.report_logs = v;
    }
    if let Some(v) = env_bool("HOISTER_REPORT_METRICS") {
        config.report_metrics = v;
    }

    // `redact_keywords` is a list, which figment's split("_") env provider can't
    // populate cleanly, so accept a comma-separated HOISTER_REDACT_KEYWORDS that
    // extends whatever the config file declared.
    if let Ok(raw) = std::env::var("HOISTER_REDACT_KEYWORDS") {
        config.redact_keywords.extend(
            raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
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

            [dispatcher.discord_webhook]
            webhook="https://discord.com/api/webhooks/123/abc"
            username="hoister"

            [dispatcher.teams]
            webhook="https://example.webhook.office.com/webhookb2/abc/IncomingWebhook/def/ghi"
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
            let webhook = dispatcher.discord_webhook.as_ref().unwrap();
            assert_eq!(
                webhook.webhook,
                "https://discord.com/api/webhooks/123/abc".parse().unwrap()
            );
            assert_eq!(webhook.username.as_deref(), Some("hoister"));
            assert_eq!(
                dispatcher.teams.as_ref().unwrap().webhook,
                "https://example.webhook.office.com/webhookb2/abc/IncomingWebhook/def/ghi"
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                dispatcher.slack.as_ref().unwrap().channel,
                "channel-name".to_string()
            );

            Ok(())
        });
    }

    #[test]
    fn test_report_metrics_defaults_on_and_can_be_disabled() {
        use figment2::Jail;

        // Default: metrics on, logs off.
        Jail::expect_with(|jail: &mut Jail| {
            jail.create_file("config-test.toml", "[schedule]\ninterval=10\n")?;
            let rt = tokio::runtime::Runtime::new().unwrap();
            let config = rt.block_on(load_config("config-test.toml".as_ref()));
            assert!(config.report_metrics, "metrics should default to on");
            assert!(!config.report_logs, "logs should default to off");
            Ok(())
        });

        // Disable metrics via the config file.
        Jail::expect_with(|jail: &mut Jail| {
            jail.create_file(
                "config-test.toml",
                "report_metrics=false\n[schedule]\ninterval=10\n",
            )?;
            let rt = tokio::runtime::Runtime::new().unwrap();
            let config = rt.block_on(load_config("config-test.toml".as_ref()));
            assert!(
                !config.report_metrics,
                "report_metrics=false should disable"
            );
            Ok(())
        });

        // Env var overrides and can disable metrics / enable logs.
        Jail::expect_with(|jail: &mut Jail| {
            jail.create_file("config-test.toml", "[schedule]\ninterval=10\n")?;
            jail.set_env("HOISTER_REPORT_METRICS", "false");
            jail.set_env("HOISTER_REPORT_LOGS", "true");
            let rt = tokio::runtime::Runtime::new().unwrap();
            let config = rt.block_on(load_config("config-test.toml".as_ref()));
            assert!(!config.report_metrics, "env should disable metrics");
            assert!(config.report_logs, "env should enable logs");
            Ok(())
        });
    }

    #[test]
    fn test_controller_token_alone_defaults_to_hosted_url() {
        use figment2::Jail;
        Jail::expect_with(|jail: &mut Jail| {
            jail.create_file(
                "config-test.toml",
                r#"
            [schedule]
            interval=10
            "#,
            )?;

            jail.set_env("HOISTER_CONTROLLER_TOKEN", "hst_test_token");

            let rt = tokio::runtime::Runtime::new().unwrap();
            let config = rt.block_on(load_config("config-test.toml".as_ref()));

            let controller = config.controller.expect("controller should be populated");
            assert_eq!(
                controller.url.as_str(),
                "https://api.hoister.io/",
                "default URL should be the hosted instance"
            );
            assert_eq!(controller.token.as_deref(), Some("hst_test_token"));
            Ok(())
        });
    }

    #[test]
    fn test_no_controller_env_means_standalone() {
        use figment2::Jail;
        Jail::expect_with(|jail: &mut Jail| {
            jail.create_file(
                "config-test.toml",
                r#"
            [schedule]
            interval=10
            "#,
            )?;

            let rt = tokio::runtime::Runtime::new().unwrap();
            let config = rt.block_on(load_config("config-test.toml".as_ref()));

            assert!(
                config.controller.is_none(),
                "no controller env vars should leave controller=None (standalone mode)"
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
