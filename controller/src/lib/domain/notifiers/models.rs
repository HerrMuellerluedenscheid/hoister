use serde::{Deserialize, Serialize};
use thiserror::Error;

/// One notifier kind per chatterbox transport we expose to users. The
/// `kind` is stored as a separate text column so the DB can filter by it;
/// the variant-specific fields live in the JSON `config` blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifierKind {
    Slack,
    Telegram,
    Discord,
    DiscordWebhook,
    Teams,
    Gotify,
    Email,
    Ntfy,
    Pushover,
    Matrix,
    Mattermost,
    RocketChat,
    GoogleChat,
    Webhook,
}

impl NotifierKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotifierKind::Slack => "slack",
            NotifierKind::Telegram => "telegram",
            NotifierKind::Discord => "discord",
            NotifierKind::DiscordWebhook => "discord_webhook",
            NotifierKind::Teams => "teams",
            NotifierKind::Gotify => "gotify",
            NotifierKind::Email => "email",
            NotifierKind::Ntfy => "ntfy",
            NotifierKind::Pushover => "pushover",
            NotifierKind::Matrix => "matrix",
            NotifierKind::Mattermost => "mattermost",
            NotifierKind::RocketChat => "rocketchat",
            NotifierKind::GoogleChat => "google_chat",
            NotifierKind::Webhook => "webhook",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "slack" => Some(Self::Slack),
            "telegram" => Some(Self::Telegram),
            "discord" => Some(Self::Discord),
            "discord_webhook" => Some(Self::DiscordWebhook),
            "teams" => Some(Self::Teams),
            "gotify" => Some(Self::Gotify),
            "email" => Some(Self::Email),
            "ntfy" => Some(Self::Ntfy),
            "pushover" => Some(Self::Pushover),
            "matrix" => Some(Self::Matrix),
            "mattermost" => Some(Self::Mattermost),
            "rocketchat" => Some(Self::RocketChat),
            "google_chat" => Some(Self::GoogleChat),
            "webhook" => Some(Self::Webhook),
            _ => None,
        }
    }
}

/// Variant-specific config payload. The shape we accept on the wire and
/// store in the DB. Secrets (webhook URLs, tokens, passwords) live here.
///
/// TODO: encrypt at rest. See follow-up issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NotifierConfig {
    Slack(SlackConfig),
    Telegram(TelegramConfig),
    Discord(DiscordConfig),
    DiscordWebhook(DiscordWebhookConfig),
    Teams(TeamsConfig),
    Gotify(GotifyConfig),
    Email(EmailConfig),
    Ntfy(NtfyConfig),
    Pushover(PushoverConfig),
    Matrix(MatrixConfig),
    Mattermost(MattermostConfig),
    RocketChat(RocketChatConfig),
    GoogleChat(GoogleChatConfig),
    Webhook(WebhookConfig),
}

impl NotifierConfig {
    pub fn kind(&self) -> NotifierKind {
        match self {
            NotifierConfig::Slack(_) => NotifierKind::Slack,
            NotifierConfig::Telegram(_) => NotifierKind::Telegram,
            NotifierConfig::Discord(_) => NotifierKind::Discord,
            NotifierConfig::DiscordWebhook(_) => NotifierKind::DiscordWebhook,
            NotifierConfig::Teams(_) => NotifierKind::Teams,
            NotifierConfig::Gotify(_) => NotifierKind::Gotify,
            NotifierConfig::Email(_) => NotifierKind::Email,
            NotifierConfig::Ntfy(_) => NotifierKind::Ntfy,
            NotifierConfig::Pushover(_) => NotifierKind::Pushover,
            NotifierConfig::Matrix(_) => NotifierKind::Matrix,
            NotifierConfig::Mattermost(_) => NotifierKind::Mattermost,
            NotifierConfig::RocketChat(_) => NotifierKind::RocketChat,
            NotifierConfig::GoogleChat(_) => NotifierKind::GoogleChat,
            NotifierConfig::Webhook(_) => NotifierKind::Webhook,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub channel_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordWebhookConfig {
    /// Discord incoming-webhook URL
    /// (`https://discord.com/api/webhooks/{id}/{token}`). The target channel
    /// is fixed when the webhook is created; no bot token is involved.
    pub webhook: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    /// Microsoft Teams incoming-webhook URL. The target channel is fixed when
    /// the webhook is created; no app registration is involved. Works with both
    /// Workflows (`*.logic.azure.com`) and legacy connector
    /// (`*.webhook.office.com`) webhooks.
    pub webhook: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotifyConfig {
    pub server: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// Destination address only. The sender identity and delivery
    /// credentials are controller-wide (Resend) and supplied via env, so
    /// users no longer enter SMTP details per notifier.
    pub recipient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtfyConfig {
    /// Base URL of the ntfy server (e.g. `https://ntfy.sh`). User-supplied, so
    /// SSRF-validated at create time.
    pub server: String,
    pub topic: String,
    /// Optional access token for reserved/protected topics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushoverConfig {
    /// Application API token and recipient user/group key. Delivery is to
    /// Pushover's fixed API host, so there is no user-supplied host to check.
    pub token: String,
    pub user: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixConfig {
    /// Homeserver base URL (e.g. `https://matrix.org`). User-supplied, so
    /// SSRF-validated at create time.
    pub homeserver: String,
    pub access_token: String,
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostConfig {
    /// Mattermost incoming-webhook URL. Self-hosted, so user-supplied and
    /// SSRF-validated at create time.
    pub webhook: String,
    /// Optional channel override (e.g. `town-square` or `@username`); only
    /// honoured if the webhook allows channel overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Optional display-name override for the posting bot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocketChatConfig {
    /// Rocket.Chat incoming-webhook URL. Self-hosted, so user-supplied and
    /// SSRF-validated at create time.
    pub webhook: String,
    /// Optional channel override (e.g. `#general` or `@username`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Optional alias (display name) override for the posting bot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleChatConfig {
    /// Google Chat incoming-webhook URL. The target space is fixed when the
    /// webhook is created; the URL carries the `key`/`token` pair, so it is a
    /// secret. Host is fixed (`chat.googleapis.com`), pinned at create time.
    pub webhook: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Arbitrary HTTP endpoint to POST events to. User-supplied, so
    /// SSRF-validated at create time. Headers carry auth and are secret.
    pub url: String,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub headers: std::collections::HashMap<String, String>,
}

/// A persisted notifier. The full `config` (with secrets) stays
/// server-side; the dashboard receives [`NotifierSummary`] instead.
#[derive(Debug, Clone)]
pub struct Notifier {
    pub id: i64,
    pub user_id: String,
    pub kind: NotifierKind,
    pub config: NotifierConfig,
    pub enabled: bool,
    pub created_at: String,
}

/// Wire-shape returned by `GET /notifiers` and `POST /notifiers`. Carries
/// only non-secret display fields plus `*_set: true` markers, so a stolen
/// session or browser XSS doesn't yield the user's Slack webhook URL,
/// bot tokens, or SMTP password. To rotate a secret, the user deletes the
/// notifier and creates a new one.
#[derive(Debug, Clone, Serialize)]
pub struct NotifierSummary {
    pub id: i64,
    pub kind: NotifierKind,
    pub config: NotifierSummaryConfig,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NotifierSummaryConfig {
    Slack {
        channel: String,
        webhook_set: bool,
    },
    Telegram {
        chat_id: u64,
        bot_token_set: bool,
    },
    Discord {
        channel_id: u64,
        bot_token_set: bool,
    },
    DiscordWebhook {
        webhook_set: bool,
    },
    Teams {
        webhook_set: bool,
    },
    Gotify {
        /// Gotify server host (origin) only — query/path stripped so a
        /// custom auth path in the URL doesn't leak back to the browser.
        server_host: String,
        token_set: bool,
    },
    Email {
        recipient: String,
    },
    Ntfy {
        /// ntfy server host (origin) only — path/query stripped.
        server_host: String,
        topic: String,
        access_token_set: bool,
    },
    Pushover {
        token_set: bool,
        user_set: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        device: Option<String>,
    },
    Matrix {
        /// Homeserver host (origin) only — path/query stripped.
        homeserver_host: String,
        room_id: String,
        access_token_set: bool,
    },
    Mattermost {
        /// Webhook host (origin) only — the `/hooks/<token>` path is secret.
        webhook_host: String,
        webhook_set: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel: Option<String>,
    },
    RocketChat {
        /// Webhook host (origin) only — the `/hooks/...` path is secret.
        webhook_host: String,
        webhook_set: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel: Option<String>,
    },
    GoogleChat {
        /// Fixed host only — the `key`/`token` query pair is secret.
        webhook_host: String,
        webhook_set: bool,
    },
    Webhook {
        /// Endpoint host (origin) only — path/query stripped so URL-embedded
        /// secrets don't leak back to the browser.
        url_host: String,
        headers_set: bool,
    },
}

impl From<&Notifier> for NotifierSummary {
    fn from(n: &Notifier) -> Self {
        let config = match &n.config {
            NotifierConfig::Slack(c) => NotifierSummaryConfig::Slack {
                channel: c.channel.clone(),
                webhook_set: !c.webhook.is_empty(),
            },
            NotifierConfig::Telegram(c) => NotifierSummaryConfig::Telegram {
                chat_id: c.chat_id,
                bot_token_set: !c.bot_token.is_empty(),
            },
            NotifierConfig::Discord(c) => NotifierSummaryConfig::Discord {
                channel_id: c.channel_id,
                bot_token_set: !c.bot_token.is_empty(),
            },
            NotifierConfig::DiscordWebhook(c) => NotifierSummaryConfig::DiscordWebhook {
                webhook_set: !c.webhook.is_empty(),
            },
            NotifierConfig::Teams(c) => NotifierSummaryConfig::Teams {
                webhook_set: !c.webhook.is_empty(),
            },
            NotifierConfig::Gotify(c) => NotifierSummaryConfig::Gotify {
                server_host: url_host_only(&c.server),
                token_set: !c.token.is_empty(),
            },
            NotifierConfig::Email(c) => NotifierSummaryConfig::Email {
                recipient: c.recipient.clone(),
            },
            NotifierConfig::Ntfy(c) => NotifierSummaryConfig::Ntfy {
                server_host: url_host_only(&c.server),
                topic: c.topic.clone(),
                access_token_set: c.access_token.as_ref().is_some_and(|t| !t.is_empty()),
            },
            NotifierConfig::Pushover(c) => NotifierSummaryConfig::Pushover {
                token_set: !c.token.is_empty(),
                user_set: !c.user.is_empty(),
                device: c.device.clone().filter(|d| !d.is_empty()),
            },
            NotifierConfig::Matrix(c) => NotifierSummaryConfig::Matrix {
                homeserver_host: url_host_only(&c.homeserver),
                room_id: c.room_id.clone(),
                access_token_set: !c.access_token.is_empty(),
            },
            NotifierConfig::Mattermost(c) => NotifierSummaryConfig::Mattermost {
                webhook_host: url_host_only(&c.webhook),
                webhook_set: !c.webhook.is_empty(),
                channel: c.channel.clone().filter(|s| !s.is_empty()),
            },
            NotifierConfig::RocketChat(c) => NotifierSummaryConfig::RocketChat {
                webhook_host: url_host_only(&c.webhook),
                webhook_set: !c.webhook.is_empty(),
                channel: c.channel.clone().filter(|s| !s.is_empty()),
            },
            NotifierConfig::GoogleChat(c) => NotifierSummaryConfig::GoogleChat {
                webhook_host: url_host_only(&c.webhook),
                webhook_set: !c.webhook.is_empty(),
            },
            NotifierConfig::Webhook(c) => NotifierSummaryConfig::Webhook {
                url_host: url_host_only(&c.url),
                headers_set: !c.headers.is_empty(),
            },
        };
        Self {
            id: n.id,
            kind: n.kind,
            config,
            enabled: n.enabled,
            created_at: n.created_at.clone(),
        }
    }
}

/// Reduce a URL to `scheme://host[:port]`, dropping path/query so a
/// custom auth path or URL-embedded secret never leaks back to the browser in
/// a notifier summary. Falls back to the raw string when it doesn't parse.
fn url_host_only(raw: &str) -> String {
    match url::Url::parse(raw) {
        Ok(u) => {
            let scheme = u.scheme();
            let host = u.host_str().unwrap_or("");
            match (u.port(), host.is_empty()) {
                (_, true) => raw.to_string(),
                (Some(p), _) => format!("{scheme}://{host}:{p}"),
                (None, _) => format!("{scheme}://{host}"),
            }
        }
        Err(_) => raw.to_string(),
    }
}

#[derive(Debug, Error)]
pub enum NotifierError {
    #[error("Notifier not found")]
    NotFound,
    #[error("Invalid notifier config: {0}")]
    InvalidConfig(String),
    #[error("Unknown error")]
    UnknownError,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notifier(kind: NotifierKind, config: NotifierConfig) -> Notifier {
        Notifier {
            id: 1,
            user_id: "u".into(),
            kind,
            config,
            enabled: true,
            created_at: "now".into(),
        }
    }

    // The summary is the only notifier shape the browser receives. Secrets
    // (webhook URL path/query, auth headers, ntfy access token) must never
    // appear in it — only host + `*_set` markers.
    #[test]
    fn webhook_summary_strips_url_path_and_hides_headers() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer s3cret".to_string());
        let n = notifier(
            NotifierKind::Webhook,
            NotifierConfig::Webhook(WebhookConfig {
                url: "https://example.com/hooks/abc?token=s3cret".to_string(),
                headers,
            }),
        );
        match NotifierSummary::from(&n).config {
            NotifierSummaryConfig::Webhook {
                url_host,
                headers_set,
            } => {
                assert_eq!(url_host, "https://example.com");
                assert!(headers_set);
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn ntfy_summary_marks_token_set_without_leaking_it() {
        let n = notifier(
            NotifierKind::Ntfy,
            NotifierConfig::Ntfy(NtfyConfig {
                server: "https://ntfy.sh/some/path".to_string(),
                topic: "alerts".to_string(),
                access_token: Some("tk_secret".to_string()),
            }),
        );
        match NotifierSummary::from(&n).config {
            NotifierSummaryConfig::Ntfy {
                server_host,
                topic,
                access_token_set,
            } => {
                assert_eq!(server_host, "https://ntfy.sh");
                assert_eq!(topic, "alerts");
                assert!(access_token_set);
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn kind_roundtrips_through_str_for_new_kinds() {
        for k in [
            NotifierKind::Ntfy,
            NotifierKind::Pushover,
            NotifierKind::Matrix,
            NotifierKind::Mattermost,
            NotifierKind::RocketChat,
            NotifierKind::GoogleChat,
            NotifierKind::Webhook,
        ] {
            assert_eq!(NotifierKind::parse(k.as_str()), Some(k));
        }
    }
}
