use chatterbox::message::Message;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt::{Display, Formatter};
use ts_rs::TS;

pub mod wire;

pub type ContainerID = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImageDigest(pub String);

impl ImageDigest {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The name of an image as used in docker in the form `repo:tag`. E.g. "emrius11/example:latest"
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImageName(pub String);

impl ImageName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn split(&self) -> (&str, &str) {
        let split: Vec<&str> = self.0.split(':').collect();
        (split[0], split.get(1).unwrap_or(&"latest"))
    }
}

#[derive(TS, Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq, Type)]
#[ts(export)]
pub struct ServiceName(pub String);

impl ServiceName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(TS, Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq)]
#[ts(export)]
pub struct ProjectName(pub String);

impl ProjectName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(TS, Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[ts(export)]
pub struct HostName(pub String);

impl HostName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for HostName {
    fn default() -> Self {
        Self(String::from("undefined"))
    }
}

#[derive(TS, Deserialize, Debug, Clone, Serialize, Type)]
#[ts(export)]
#[repr(u8)]
pub enum DeploymentStatus {
    Pending = 0,
    Started = 1,
    Success = 2,
    RollbackFinished = 3,
    NoUpdate = 4,
    Failed = 5,
    TestMessage = 6,
    UpdateAvailable = 7,
}

impl Display for DeploymentStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentStatus::Pending => write!(f, "Deployment Pending"),
            DeploymentStatus::Started => write!(f, "Deployment Started"),
            DeploymentStatus::Success => write!(f, "Deployment Successful ✅"),
            DeploymentStatus::RollbackFinished => write!(f, "Deployment rolled back 🔁"),
            &DeploymentStatus::NoUpdate => write!(f, "NoUpdate"),
            &DeploymentStatus::Failed => write!(f, "Deployment Failed ❌"),
            &DeploymentStatus::TestMessage => write!(f, "Test Message"),
            &DeploymentStatus::UpdateAvailable => write!(f, "Update Available"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateDeployment {
    pub project: ProjectName,
    pub service: ServiceName,
    pub image: ImageName,
    pub digest: ImageDigest,
    pub status: DeploymentStatus,
    pub hostname: HostName,
    /// Redacted log tail of the failed container, attached on rollback/failure
    /// so the dashboard can show why an update was rolled back. `None` for
    /// successful deployments and when the agent has not opted into log
    /// forwarding (`HOISTER_REPORT_LOGS`). `#[serde(default)]` keeps older
    /// agents that don't send this field wire-compatible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logs: Option<String>,
}

impl Display for CreateDeployment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let body = format!(
            "image {} update to {}\nfinished with status {:?}\n(project {} | service {} | host {})",
            self.image.as_str(),
            self.digest.as_str(),
            self.status,
            self.project.as_str(),
            self.service.as_str(),
            self.hostname.as_str()
        );

        write!(f, "{body}")
    }
}

impl CreateDeployment {
    pub fn test() -> Self {
        Self {
            project: ProjectName::new("tests-project"),
            service: ServiceName::new("tests-service"),
            image: ImageName::new("tests:latest"),
            digest: ImageDigest::new("sha256:tests"),
            status: DeploymentStatus::TestMessage,
            hostname: HostName::default(),
            logs: None,
        }
    }

    pub fn to_message(&self) -> Message {
        Message::new(self.status.to_string(), self.to_string()).with_subject(
            deployment_email_subject(self.image.as_str(), self.hostname.as_str()),
        )
    }
}

impl From<&CreateDeployment> for Message {
    fn from(val: &CreateDeployment) -> Self {
        val.to_message()
    }
}

/// Stable email subject / thread key for every notification about one image's
/// deployment history on a host. Keyed by the deployment target — never the
/// event type — so a mail client groups update-available notices, successes,
/// rollbacks and failures for the same image into a single conversation,
/// instead of bucketing everything by status into a few giant threads. Keeping
/// the host in the key stops the same image on different machines from merging.
pub fn deployment_email_subject(image_name: &str, hostname: &str) -> String {
    format!("Hoister deployment: {image_name} on {hostname}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_name_split_with_tag() {
        let image = ImageName::new("emrius11/example:latest");
        assert_eq!(image.split(), ("emrius11/example", "latest"));
    }

    #[test]
    // Ensure that the default tag is "latest" if no tag is specified
    fn test_image_name_split_no_tag() {
        let image = ImageName::new("emrius11/example");
        assert_eq!(image.split(), ("emrius11/example", "latest"));
    }

    fn deployment(status: DeploymentStatus, image: &str, host: &str) -> CreateDeployment {
        CreateDeployment {
            project: ProjectName::new("p"),
            service: ServiceName::new("s"),
            image: ImageName::new(image),
            digest: ImageDigest::new("sha256:1"),
            status,
            hostname: HostName::new(host),
            logs: None,
        }
    }

    // Regression: the email subject used to be the status string, so every
    // notification collapsed into ~3 threads. The subject must be keyed by the
    // deployment target (image + host), not the event type, so one image's
    // history threads together — while the title still carries the status for
    // chat dispatchers.
    #[test]
    fn email_subject_is_keyed_by_image_and_host_not_status() {
        let success = deployment(DeploymentStatus::Success, "myapp:latest", "web-01");
        let rollback = deployment(DeploymentStatus::RollbackFinished, "myapp:latest", "web-01");
        let other_image = deployment(DeploymentStatus::Success, "other:latest", "web-01");
        let other_host = deployment(DeploymentStatus::Success, "myapp:latest", "web-02");

        // Same image+host, different status -> one thread.
        assert_eq!(success.to_message().subject, rollback.to_message().subject);

        // The subject must not leak the status, or clients would split the thread.
        let subject = success.to_message().subject.expect("subject is set");
        assert!(
            !subject.contains("Success") && !subject.contains('✅'),
            "subject must not depend on status: {subject}"
        );

        // A different image or host is a different thread.
        assert_ne!(
            success.to_message().subject,
            other_image.to_message().subject
        );
        assert_ne!(
            success.to_message().subject,
            other_host.to_message().subject
        );

        // The title still carries the status for chat dispatchers.
        assert!(success.to_message().title.contains("Successful"));
    }
}
