use hoister_shared::{HostName, ProjectName};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// Longest accepted invitee address (the RFC 5321 path limit).
pub const MAX_EMAIL_LEN: usize = 320;
/// Co-maintainers a project may have besides its owner.
pub const MAX_MEMBERS_PER_PROJECT: usize = 25;
/// Pending invitations per project. Every invitation sends an email, so this
/// bounds how much mail one project can generate.
pub const MAX_PENDING_INVITATIONS_PER_PROJECT: usize = 20;

/// What a user is to a project. The owner is the account whose agent reports
/// the project (`project.user_id`); members were invited to co-maintain it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectRole {
    Owner,
    Member,
}

/// One project as seen by one user. Only ever handed out after the access
/// check, so holding a `ProjectAccess` means the viewer may read the project;
/// `role` decides whether they may also administer it.
#[derive(Debug, Clone)]
pub struct ProjectAccess {
    pub id: uuid::Uuid,
    pub name: ProjectName,
    pub hostname: HostName,
    /// The owning account. Container state, metrics, logs and pending updates
    /// are all stored under this user id, so project-scoped reads go through it.
    pub owner_id: String,
    pub role: ProjectRole,
    pub created_at: String,
    pub member_count: i64,
    pub notifier_count: i64,
}

impl ProjectAccess {
    pub fn is_owner(&self) -> bool {
        self.role == ProjectRole::Owner
    }
}

/// A co-maintainer. The owner is not stored as a member row.
#[derive(Debug, Clone)]
pub struct ProjectMember {
    pub user_id: String,
    pub invited_by: Option<String>,
    pub created_at: String,
}

/// A pending invitation for someone who has no account yet.
#[derive(Debug, Clone)]
pub struct ProjectInvitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub invited_by: String,
    pub created_at: String,
}

#[derive(Debug, Error)]
pub enum ProjectsError {
    /// The project doesn't exist or the user has no access to it. The two are
    /// deliberately indistinguishable so project ids can't be probed.
    #[error("Project not found")]
    NotFound,
    #[error("Only the project owner can do this")]
    Forbidden,
    #[error("{0}")]
    Invalid(String),
    #[error("Unknown error")]
    UnknownError,
}

/// Normalise an invitee address for storage and matching: trimmed and
/// lower-cased. Returns `None` for anything that is clearly not a single
/// email address. Deliverability is the identity provider's problem; this
/// only keeps garbage and header-injection attempts out of the table.
pub fn normalize_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_lowercase();
    if email.is_empty() || email.len() > MAX_EMAIL_LEN {
        return None;
    }
    if email.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') || !domain.contains('.') {
        return None;
    }
    if domain.starts_with('.') || domain.ends_with('.') {
        return None;
    }
    Some(email)
}

#[cfg(test)]
mod tests {
    use super::normalize_email;

    #[test]
    fn normalize_email_trims_and_lowercases() {
        assert_eq!(
            normalize_email("  Alice@Example.COM "),
            Some("alice@example.com".to_string())
        );
    }

    #[test]
    fn normalize_email_rejects_malformed_addresses() {
        for bad in [
            "",
            "alice",
            "@example.com",
            "alice@",
            "alice@localhost",
            "a@b@example.com",
            "alice@.example.com",
            "alice@example.com.",
            "alice smith@example.com",
            "alice@example.com\nBcc: eve@example.com",
        ] {
            assert_eq!(normalize_email(bad), None, "{bad:?} should be rejected");
        }
    }

    #[test]
    fn normalize_email_rejects_overlong_addresses() {
        let long = format!("{}@example.com", "a".repeat(320));
        assert_eq!(normalize_email(&long), None);
    }
}
