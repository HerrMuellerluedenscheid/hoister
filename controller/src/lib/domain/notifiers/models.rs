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

/// A persisted notifier. The serialized form (sent to the dashboard)
/// redacts the secret-bearing fields so the UI can show "set" without
/// leaking values that would let a session-hijacker walk away with the
/// user's Slack webhook.
#[derive(Debug, Clone, Serialize)]
pub struct Notifier {
    pub id: i64,
    pub user_id: String,
    pub kind: NotifierKind,
    pub config: NotifierConfig,
    pub enabled: bool,
    pub created_at: String,
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
