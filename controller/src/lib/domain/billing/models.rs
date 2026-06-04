use crate::domain::notifiers::models::NotifierKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Pro,
}

impl Plan {
    pub fn as_str(&self) -> &'static str {
        match self {
            Plan::Free => "free",
            Plan::Pro => "pro",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "free" => Some(Self::Free),
            "pro" => Some(Self::Pro),
            _ => None,
        }
    }

    pub fn limits(&self) -> PlanLimits {
        match self {
            Plan::Free => PlanLimits {
                max_projects: Some(2),
                allowed_notifier_kinds: vec![
                    NotifierKind::Telegram,
                    NotifierKind::Discord,
                    NotifierKind::DiscordWebhook,
                ],
            },
            Plan::Pro => PlanLimits {
                max_projects: None,
                allowed_notifier_kinds: vec![
                    NotifierKind::Slack,
                    NotifierKind::Telegram,
                    NotifierKind::Discord,
                    NotifierKind::DiscordWebhook,
                    NotifierKind::Gotify,
                    NotifierKind::Email,
                ],
            },
        }
    }
}

/// Quotas for a plan. `None` on a numeric field means unlimited.
#[derive(Debug, Clone, Serialize)]
pub struct PlanLimits {
    pub max_projects: Option<i64>,
    pub allowed_notifier_kinds: Vec<NotifierKind>,
}

impl PlanLimits {
    pub fn allows_notifier_kind(&self, kind: NotifierKind) -> bool {
        self.allowed_notifier_kinds.contains(&kind)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Usage {
    pub projects: i64,
    pub notifiers_by_kind: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanStatus {
    pub plan: Plan,
    pub limits: PlanLimits,
    pub usage: Usage,
}

#[derive(Debug, Error)]
pub enum PlanError {
    #[error("Unknown error")]
    UnknownError,
}

/// Billing-side rejection from a write handler. Maps to HTTP 402.
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeRequired {
    pub message: String,
    pub required_plan: Plan,
}
