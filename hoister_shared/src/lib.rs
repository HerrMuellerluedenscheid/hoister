use chatterbox::message::Message;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt::{Display, Formatter};
use ts_rs::TS;

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

#[derive(TS, Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq)]
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
}

impl Display for CreateDeployment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let body = format!(
            "image {} update to {}\nfinished with status {:?}\n(project {} | service {})",
            self.image.as_str(),
            self.digest.as_str(),
            self.status,
            self.project.as_str(),
            self.service.as_str()
        );

        write!(f, "{}", body)
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
        }
    }

    pub fn to_message(&self) -> Message {
        Message::new(self.status.to_string(), self.to_string())
    }
}

impl From<&CreateDeployment> for Message {
    fn from(val: &CreateDeployment) -> Self {
        Message::new(val.status.to_string(), val.to_string())
    }
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
}
