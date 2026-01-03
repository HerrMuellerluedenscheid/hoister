use crate::HoisterError;
use chatterbox::message::{Dispatcher, Message};
use log::{debug, error, info};
use shared::{
    CreateDeployment, DeploymentStatus, ImageDigest, ImageName, ProjectName, ServiceName,
};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub struct DeploymentResultHandler {
    tx: Sender<CreateDeployment>,
}

impl DeploymentResultHandler {
    pub(crate) fn new(tx: Sender<CreateDeployment>) -> Self {
        Self { tx }
    }

    pub(crate) async fn inform_container_failed(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        self.tx
            .send(CreateDeployment {
                project,
                service,
                image,
                digest,
                status: DeploymentStatus::Failed,
            })
            .await
            .unwrap();
    }

    pub(crate) async fn inform_rollback_complete(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        self.tx
            .send(CreateDeployment {
                project,
                service,
                image,
                digest,
                status: DeploymentStatus::RollbackFinished,
            })
            .await
            .unwrap();
    }

    pub(crate) async fn inform_update_success(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        self.tx
            .send(CreateDeployment {
                project,
                service,
                image,
                digest,
                status: DeploymentStatus::Success,
            })
            .await
            .unwrap();
    }

    pub(crate) async fn test_message(&self) {
        self.tx.send(CreateDeployment::test()).await.unwrap();
    }
}

pub(super) async fn start_notification_handler(
    mut rx: Receiver<CreateDeployment>,
    dispatcher: Dispatcher,
) {
    while let Some(message) = rx.recv().await {
        send(&message, &dispatcher).await;
    }
}

async fn send_to_controller(result: &CreateDeployment) {
    let client = reqwest::Client::new();
    let url = std::env::var("HOISTER_CONTROLLER_URL");
    if url.is_err() {
        info!("HOISTER_CONTROLLER_URL not defined");
        return;
    }
    let token = std::env::var("HOISTER_CONTROLLER_TOKEN").unwrap_or_default();

    let mut url = url.unwrap();
    url.push_str("/deployments");
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token})"))
        .json(&result)
        .send()
        .await;
    debug!("response: {:?}", res);
}

async fn send_to_chatterbox(result: &CreateDeployment, dispatcher: &Dispatcher) {
    match result.status {
        DeploymentStatus::NoUpdate => {}
        _ => {
            let message: Message = result.into();
            _ = dispatcher
                .dispatch(&message)
                .inspect_err(|e| error!("failed to dispatch message: {e}"));
        }
    }
}

pub(crate) async fn send(result: &CreateDeployment, dispatcher: &Dispatcher) {
    debug!("sending deployment request");
    send_to_controller(result).await;
    send_to_chatterbox(result, dispatcher).await;
}

pub(crate) fn setup_dispatcher() -> Dispatcher {
    let slack = match std::env::var("HOISTER_SLACK_WEBHOOK_URL") {
        Ok(webhook_url) => {
            info!("Using Slack dispatcher");
            let channel =
                std::env::var("HOISTER_SLACK_CHANNEL").expect("HOISTER_SLACK_CHANNEL not defined");
            Some(chatterbox::dispatcher::slack::Slack {
                webhook_url,
                channel,
            })
        }
        Err(_) => {
            info!("HOISTER_SLACK_WEBHOOK_URL not defined");
            None
        }
    };
    let telegram = match std::env::var("HOISTER_TELEGRAM_BOT_TOKEN") {
        Ok(bot_token) => {
            info!("Using Telegram dispatcher");
            let chat_id = std::env::var("HOISTER_TELEGRAM_CHAT_ID")
                .expect("HOISTER_TELEGRAM_CHAT_ID not defined");
            Some(chatterbox::dispatcher::telegram::Telegram { bot_token, chat_id })
        }
        Err(_) => {
            info!("HOISTER_TELEGRAM_BOT_TOKEN not defined");
            None
        }
    };
    let discord = match std::env::var("HOISTER_DISCORD_BOT_TOKEN") {
        Ok(bot_token) => {
            info!("Using Discord dispatcher");
            let channel_id = std::env::var("HOISTER_DISCORD_CHANNEL_ID")
                .expect("HOISTER_DISCORD_CHANNEL_ID not defined");
            Some(chatterbox::dispatcher::discord::Discord {
                bot_token,
                channel_id,
            })
        }
        Err(_) => {
            info!("HOISTER_DISCORD_BOT_TOKEN not defined");
            None
        }
    };
    let sender = chatterbox::dispatcher::Sender {
        slack,
        telegram,
        discord,
        email: None,
    };

    Dispatcher::new(sender)
}

impl From<HoisterError> for Option<Message> {
    fn from(value: HoisterError) -> Self {
        match value {
            HoisterError::NoUpdateAvailable => {
                debug!("no update available");
                None
            }
            HoisterError::UpdateFailed(e) => Some(Message::new(
                "update failed".to_string(),
                format!("failed to update image {e}"),
            )),
            _ => {
                error!("unexpected error: {value:?}");
                None
            }
        }
    }
}
