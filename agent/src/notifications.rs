use crate::HoisterError;
use crate::config::Config;
use chatterbox::message::{Dispatcher, Message};
use hoister_shared::{
    CreateDeployment, DeploymentStatus, HostName, ImageDigest, ImageName, ProjectName, ServiceName,
};
use log::{debug, error, info, warn};
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum NotificationError {
    #[error("failed to send notification: {0}")]
    SendError(#[from] reqwest::Error),
    #[error("failed to send notification: {0:?}")]
    BroadcastSendError(#[from] SendError<String>),
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
}

pub struct DeploymentResultHandler {
    tx: Sender<CreateDeployment>,
    hostname: HostName,
}

impl DeploymentResultHandler {
    pub(crate) fn new(tx: Sender<CreateDeployment>, hostname: HostName) -> Self {
        Self { tx, hostname }
    }

    pub(crate) async fn inform_container_failed(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest,
            status: DeploymentStatus::Failed,
            hostname: self.hostname.clone(),
        })
        .await;
    }

    pub(crate) async fn inform_rollback_complete(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest,
            status: DeploymentStatus::RollbackFinished,
            hostname: self.hostname.clone(),
        })
        .await;
    }

    pub(crate) async fn inform_update_success(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest,
            status: DeploymentStatus::Success,
            hostname: self.hostname.clone(),
        })
        .await;
    }

    pub(crate) async fn test_message(&self) {
        self.send(CreateDeployment::test()).await;
    }

    /// Best-effort enqueue onto the in-process notification channel. The
    /// receiver runs on its own task; if it has died (panicked, shut down)
    /// we log and drop the event rather than panicking the agent itself.
    async fn send(&self, message: CreateDeployment) {
        if let Err(e) = self.tx.send(message).await {
            error!("notification channel closed, dropping event: {e:?}");
        }
    }
}

pub(crate) async fn send_pending_update_to_controller(
    config: &Config,
    client: &reqwest::Client,
    hostname: &HostName,
    project: &ProjectName,
    service: &ServiceName,
    image: &ImageName,
    digest: &ImageDigest,
) -> Result<(), NotificationError> {
    let controller = match &config.controller {
        Some(c) => c,
        None => {
            info!("No controller configured, skipping pending update notification");
            return Ok(());
        }
    };

    let url = controller.url.join("/pending-updates")?;
    let token = controller.token.as_deref().unwrap_or_default();

    #[derive(serde::Serialize)]
    struct PendingUpdateRequest<'a> {
        hostname: &'a HostName,
        project_name: &'a ProjectName,
        service_name: &'a ServiceName,
        image_name: &'a ImageName,
        new_digest: &'a ImageDigest,
    }

    let body = PendingUpdateRequest {
        hostname,
        project_name: project,
        service_name: service,
        image_name: image,
        new_digest: digest,
    };

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await;
    debug!("pending update response: {res:?}");
    Ok(())
}

pub(super) async fn start_notification_handler(
    config: &Config,
    mut rx: Receiver<CreateDeployment>,
    dispatcher: Option<Dispatcher>,
    client: reqwest::Client,
) {
    while let Some(deployment_message) = rx.recv().await {
        send(config, &deployment_message, dispatcher.as_ref(), &client).await;
    }
}

async fn send_to_controller(
    deployment_message: &CreateDeployment,
    config: &Config,
    client: &reqwest::Client,
) -> Result<(), NotificationError> {
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
        .header("Authorization", format!("Bearer {token}"))
        .json(&deployment_message)
        .send()
        .await;
    debug!("response: {res:?}");
    Ok(())
}

async fn send_to_chatterbox(
    deployment_message: &CreateDeployment,
    dispatcher: Option<&Dispatcher>,
) -> Result<(), NotificationError> {
    let Some(dispatcher) = dispatcher else {
        return Ok(());
    };
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
    dispatcher: Option<&Dispatcher>,
    client: &reqwest::Client,
) {
    debug!("sending deployment request");
    let (result1, result2) = tokio::join!(
        send_to_controller(deployment_message, config, client),
        send_to_chatterbox(deployment_message, dispatcher)
    );

    if let Err(e) = result1 {
        error!("Failed to send to controller: {e:?}");
    }
    if let Err(e) = result2 {
        error!("Failed to send to chatterbox: {e:?}");
    }
}

pub(crate) fn setup_dispatcher(config: &Config) -> Option<Dispatcher> {
    if std::env::var("HOISTER_SLACK_WEBHOOK_URL").is_ok()
        || std::env::var("HOISTER_SLACK_CHANNEL").is_ok()
        || std::env::var("HOISTER_TELEGRAM_BOT_TOKEN").is_ok()
        || std::env::var("HOISTER_TELEGRAM_CHAT_ID").is_ok()
        || std::env::var("HOISTER_DISCORD_BOT_TOKEN").is_ok()
    {
        warn!(
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
            sender_name: e.from.unwrap_or("hoister".to_string()),
        }
    });

    let sender = chatterbox::dispatcher::Sender {
        slack,
        telegram,
        discord,
        gotify,
        email,
        // Resend is a hosted-controller delivery path; the standalone agent
        // doesn't expose it as a configurable dispatcher.
        resend: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn sample_handler() -> (DeploymentResultHandler, mpsc::Receiver<CreateDeployment>) {
        let (tx, rx) = mpsc::channel(8);
        (DeploymentResultHandler::new(tx, HostName::default()), rx)
    }

    fn sample_args() -> (ProjectName, ServiceName, ImageName, ImageDigest) {
        (
            ProjectName::new("p"),
            ServiceName::new("s"),
            ImageName::new("img:latest"),
            ImageDigest::new("sha256:1"),
        )
    }

    // Regression for the panic at notifications.rs:50: every inform_* method
    // used to `.unwrap()` the channel send, so a dropped receiver crashed the
    // agent the next time we tried to report a deploy result.
    #[tokio::test]
    async fn inform_methods_do_not_panic_when_receiver_is_dropped() {
        let (handler, rx) = sample_handler();
        drop(rx);

        let (p, s, i, d) = sample_args();
        handler
            .inform_container_failed(p.clone(), s.clone(), i.clone(), d.clone())
            .await;
        handler
            .inform_rollback_complete(p.clone(), s.clone(), i.clone(), d.clone())
            .await;
        handler.inform_update_success(p, s, i, d).await;
        handler.test_message().await;
    }

    // Regression for the silent break in main.rs: with no chatterbox
    // dispatcher configured, send_to_chatterbox previously required a
    // Dispatcher reference. Verify the None path is a no-op so the receiver
    // can be spawned unconditionally.
    #[tokio::test]
    async fn send_to_chatterbox_is_noop_when_dispatcher_is_none() {
        let (p, s, i, d) = sample_args();
        let msg = CreateDeployment {
            project: p,
            service: s,
            image: i,
            digest: d,
            status: DeploymentStatus::Success,
            hostname: HostName::default(),
        };
        assert!(send_to_chatterbox(&msg, None).await.is_ok());
    }
}
