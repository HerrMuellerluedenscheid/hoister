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
    Gotify,
    Email,
}

impl NotifierKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotifierKind::Slack => "slack",
            NotifierKind::Telegram => "telegram",
            NotifierKind::Discord => "discord",
            NotifierKind::Gotify => "gotify",
            NotifierKind::Email => "email",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "slack" => Some(Self::Slack),
            "telegram" => Some(Self::Telegram),
            "discord" => Some(Self::Discord),
            "gotify" => Some(Self::Gotify),
            "email" => Some(Self::Email),
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
    Gotify(GotifyConfig),
    Email(EmailConfig),
}

impl NotifierConfig {
    pub fn kind(&self) -> NotifierKind {
        match self {
            NotifierConfig::Slack(_) => NotifierKind::Slack,
            NotifierConfig::Telegram(_) => NotifierKind::Telegram,
            NotifierConfig::Discord(_) => NotifierKind::Discord,
            NotifierConfig::Gotify(_) => NotifierKind::Gotify,
            NotifierConfig::Email(_) => NotifierKind::Email,
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
pub struct GotifyConfig {
    pub server: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub from: Option<String>,
    pub recipient: String,
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
    Gotify {
        /// Gotify server host (origin) only — query/path stripped so a
        /// custom auth path in the URL doesn't leak back to the browser.
        server_host: String,
        token_set: bool,
    },
    Email {
        smtp_server: String,
        smtp_user: String,
        recipient: String,
        from: Option<String>,
        smtp_password_set: bool,
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
            NotifierConfig::Gotify(c) => NotifierSummaryConfig::Gotify {
                server_host: gotify_host_only(&c.server),
                token_set: !c.token.is_empty(),
            },
            NotifierConfig::Email(c) => NotifierSummaryConfig::Email {
                smtp_server: c.smtp_server.clone(),
                smtp_user: c.smtp_user.clone(),
                recipient: c.recipient.clone(),
                from: c.from.clone(),
                smtp_password_set: !c.smtp_password.is_empty(),
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

fn gotify_host_only(raw: &str) -> String {
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
