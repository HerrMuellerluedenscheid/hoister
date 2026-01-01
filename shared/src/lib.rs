use chatterbox::message::Message;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt::{Display, Formatter};
use ts_rs::TS;

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
            DeploymentStatus::Pending => write!(f, "Pending"),
            DeploymentStatus::Started => write!(f, "Started"),
            DeploymentStatus::Success => write!(f, "Success"),
            DeploymentStatus::RollbackFinished => write!(f, "Rolled back"),
            &DeploymentStatus::NoUpdate => write!(f, "NoUpdate"),
            &DeploymentStatus::Failed => write!(f, "Failed"),
            &DeploymentStatus::TestMessage => write!(f, "Test Message"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateDeployment {
    pub image: String,
    pub container_id: String,
    pub status: DeploymentStatus,
}

impl From<&CreateDeployment> for Message {
    fn from(val: &CreateDeployment) -> Self {
        Message::new(val.status.to_string(), val.image.clone())
    }
}
