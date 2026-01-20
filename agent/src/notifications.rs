use crate::HoisterError;
use crate::config::Config;
use chatterbox::message::{Dispatcher, Message};
use log::{debug, error, info};
use shared::{
    CreateDeployment, DeploymentStatus, ImageDigest, ImageName, ProjectName, ServiceName,
};
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
enum NotificationError {
    #[error("failed to send notification: {0}")]
    SendError(#[from] reqwest::Error),
    #[error("failed to send notification: {0:?}")]
    BroadcastSendError(#[from] SendError<String>),
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
}

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
    config: &Config,
    mut rx: Receiver<CreateDeployment>,
    dispatcher: Dispatcher,
) {
    while let Some(deployment_message) = rx.recv().await {
        send(config, &deployment_message, &dispatcher).await;
    }
}

async fn send_to_controller(
    deployment_message: &CreateDeployment,
    config: &Config,
) -> Result<(), NotificationError> {
    let client = reqwest::Client::new();
    let (url, token) = match config.controller {
        Some(ref controller) => {
            debug!(
                "sending deployment request to controller: {}",
                &controller.url
            );
            let token = controller.token.as_deref().unwrap_or_default();
            (&controller.url, token)
        }
        None => {
            info!("HOISTER_CONTROLLER_URL not defined");
            return Ok(());
        }
    };

    let url = url.join("/deployments")?;
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token})"))
        .json(&deployment_message)
        .send()
        .await;
    debug!("response: {:?}", res);
    Ok(())
}

async fn send_to_chatterbox(
    deployment_message: &CreateDeployment,
    dispatcher: &Dispatcher,
) -> Result<(), NotificationError> {
    match deployment_message.status {
        DeploymentStatus::NoUpdate => Ok(()),
        _ => {
            let message = deployment_message.to_message();
            dispatcher.dispatch(&message)?;
            Ok(())
        }
    }
}

pub(crate) async fn send(
    config: &Config,
    deployment_message: &CreateDeployment,
    dispatcher: &Dispatcher,
) {
    debug!("sending deployment request");
    let (result1, result2) = tokio::join!(
        send_to_controller(deployment_message, config),
        send_to_chatterbox(deployment_message, dispatcher)
    );

    if let Err(e) = result1 {
        error!("Failed to send to controller: {:?}", e);
    }
    if let Err(e) = result2 {
        error!("Failed to send to chatterbox: {:?}", e);
    }
}

pub(crate) fn setup_dispatcher(config: &Config) -> Option<Dispatcher> {
    if std::env::var("HOISTER_SLACK_WEBHOOK_URL").is_ok()
        || std::env::var("HOISTER_SLACK_CHANNEL").is_ok()
        || std::env::var("HOISTER_TELEGRAM_BOT_TOKEN").is_ok()
        || std::env::var("HOISTER_TELEGRAM_CHAT_ID").is_ok()
        || std::env::var("HOISTER_DISCORD_BOT_TOKEN").is_ok()
    {
        error!(
            "The following environment variables are deprecated: HOISTER_SLACK_WEBHOOK_URL, HOISTER_SLACK_CHANNEL, HOISTER_TELEGRAM_BOT_TOKEN, HOISTER_TELEGRAM_CHAT_ID, HOISTER_DISCORD_BOT_TOKEN. Please change the prefix to HOISTER_DISPATCHERS instead."
        )
    };

    let dispatcher_config = config.dispatcher.clone()?;
    let slack = dispatcher_config.slack.map(|s| {
        info!("Using Slack dispatcher");
        chatterbox::dispatcher::slack::Slack {
            webhook_url: s.webhook.to_string(),
            channel: s.channel,
        }
    });
    let telegram = dispatcher_config.telegram.map(|t| {
        info!("Using Telegram dispatcher");
        chatterbox::dispatcher::telegram::Telegram {
            bot_token: t.token,
            chat_id: t.chat,
        }
    });
    let discord = dispatcher_config.discord.map(|d| {
        info!("Using Discord dispatcher");
        chatterbox::dispatcher::discord::Discord {
            bot_token: d.token,
            channel_id: d.channel,
        }
    });

    let gotify = dispatcher_config.gotify.map(|g| {
        info!("Using Gotify dispatcher");
        chatterbox::dispatcher::gotify::Gotify {
            server_url: g.server,
            app_token: g.token,
        }
    });

    let email = dispatcher_config.email.map(|e| {
        info!("Using email dispatcher");
        chatterbox::dispatcher::email::Email {
            smtp_user: e.smtp.user.clone(),
            smtp_password: e.smtp.password,
            smtp_server: e.smtp.server,
            smtp_port: 587,
            receiver_address: e.recipient,
            sender_address: e.smtp.user,
            sender_name: e.from.unwrap_or("no-reply".to_string()),
        }
    });

    let sender = chatterbox::dispatcher::Sender {
        slack,
        telegram,
        discord,
        gotify,
        email,
    };

    Some(Dispatcher::new(sender))
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
