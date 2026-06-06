use crate::HoisterError;
use crate::config::Config;
use chatterbox::message::{Dispatcher, Message};
use hoister_shared::{
    CreateDeployment, DeploymentStatus, HostName, ImageDigest, ImageName, ProjectName, ServiceName,
};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

/// A failed image pull recurs on every scheduled check (often once a minute)
/// until the operator fixes the credentials or image reference. Reporting each
/// one would write a `Failed` deployment row and fire every notifier every
/// minute. Collapse repeated identical pull failures to at most one report per
/// this interval; a successful update for the same service re-arms reporting.
const PULL_FAILURE_REPORT_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Throttle key for a pull failure: the (project, service, image) it concerns.
/// `ImageName` isn't `Hash`/`Eq`, so we fold the parts into one string.
fn pull_failure_key(project: &ProjectName, service: &ServiceName, image: &ImageName) -> String {
    format!(
        "{}\u{0}{}\u{0}{}",
        project.as_str(),
        service.as_str(),
        image.as_str()
    )
}

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
    /// Last time a pull failure was reported, keyed by [`pull_failure_key`].
    /// In-memory only: a restart re-arms reporting, which is acceptable since
    /// it bounds notifications to roughly one per agent lifetime per failure.
    pull_failure_last_reported: Mutex<HashMap<String, Instant>>,
}

impl DeploymentResultHandler {
    pub(crate) fn new(tx: Sender<CreateDeployment>, hostname: HostName) -> Self {
        Self {
            tx,
            hostname,
            pull_failure_last_reported: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn inform_container_failed(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
        logs: Option<String>,
    ) {
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest,
            status: DeploymentStatus::Failed,
            hostname: self.hostname.clone(),
            logs,
        })
        .await;
    }

    pub(crate) async fn inform_rollback_complete(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
        logs: Option<String>,
    ) {
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest,
            status: DeploymentStatus::RollbackFinished,
            hostname: self.hostname.clone(),
            logs,
        })
        .await;
    }

    /// Report that pulling a new image failed (e.g. registry unauthorized,
    /// manifest not found) so the dashboard can show the operator what went
    /// wrong when checking for an update. No digest is known at this point, so
    /// it is left empty and the error text is carried in `logs`.
    pub(crate) async fn inform_pull_failed(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        error: String,
    ) {
        if !self.should_report_pull_failure(&project, &service, &image) {
            debug!(
                "Suppressing repeated pull-failure report for {} / {} ({}); already reported within the last {}h",
                project.as_str(),
                service.as_str(),
                image.as_str(),
                PULL_FAILURE_REPORT_INTERVAL.as_secs() / 3600,
            );
            return;
        }
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest: ImageDigest::new(String::new()),
            status: DeploymentStatus::Failed,
            hostname: self.hostname.clone(),
            logs: Some(error),
        })
        .await;
    }

    /// Returns `true` when a pull failure for this (project, service, image) has
    /// not been reported within [`PULL_FAILURE_REPORT_INTERVAL`], recording the
    /// current time when it does. Collapses the per-check failure storm.
    fn should_report_pull_failure(
        &self,
        project: &ProjectName,
        service: &ServiceName,
        image: &ImageName,
    ) -> bool {
        let key = pull_failure_key(project, service, image);
        let now = Instant::now();
        let mut last = self
            .pull_failure_last_reported
            .lock()
            .expect("pull-failure throttle mutex poisoned");
        match last.get(&key) {
            Some(prev) if now.duration_since(*prev) < PULL_FAILURE_REPORT_INTERVAL => false,
            _ => {
                last.insert(key, now);
                true
            }
        }
    }

    /// Forget any pull-failure throttle for this (project, service, image) so a
    /// *new* failure after a recovery is reported promptly instead of being
    /// silenced for the rest of the interval.
    fn clear_pull_failure(&self, project: &ProjectName, service: &ServiceName, image: &ImageName) {
        let key = pull_failure_key(project, service, image);
        self.pull_failure_last_reported
            .lock()
            .expect("pull-failure throttle mutex poisoned")
            .remove(&key);
    }

    pub(crate) async fn inform_update_success(
        &self,
        project: ProjectName,
        service: ServiceName,
        image: ImageName,
        digest: ImageDigest,
    ) {
        // A successful update means any prior registry/image problem is
        // resolved; re-arm reporting so the next failure is not throttled.
        self.clear_pull_failure(&project, &service, &image);
        self.send(CreateDeployment {
            project,
            service,
            image,
            digest,
            status: DeploymentStatus::Success,
            hostname: self.hostname.clone(),
            logs: None,
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
    let discord_webhook = dispatcher_config.discord_webhook.map(|d| {
        info!("Using Discord webhook dispatcher");
        chatterbox::dispatcher::discord_webhook::DiscordWebhook {
            webhook_url: d.webhook.to_string(),
            username: d.username,
            avatar_url: d.avatar_url.map(|u| u.to_string()),
        }
    });

    let teams = dispatcher_config.teams.map(|t| {
        info!("Using Teams webhook dispatcher");
        chatterbox::dispatcher::teams::Teams {
            webhook_url: t.webhook.to_string(),
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

    let ntfy = dispatcher_config.ntfy.map(|n| {
        info!("Using ntfy dispatcher");
        chatterbox::dispatcher::ntfy::Ntfy {
            server_url: n.server,
            topic: n.topic,
            access_token: n.access_token,
        }
    });

    let pushover = dispatcher_config.pushover.map(|p| {
        info!("Using Pushover dispatcher");
        chatterbox::dispatcher::pushover::Pushover {
            token: p.token,
            user: p.user,
            device: p.device,
        }
    });

    let matrix = dispatcher_config.matrix.map(|m| {
        info!("Using Matrix dispatcher");
        chatterbox::dispatcher::matrix::Matrix {
            homeserver_url: m.homeserver,
            access_token: m.access_token,
            room_id: m.room_id,
        }
    });

    let webhook = dispatcher_config.webhook.map(|w| {
        info!("Using webhook dispatcher");
        chatterbox::dispatcher::webhook::Webhook {
            url: w.url.to_string(),
            headers: w.headers,
        }
    });

    let sender = chatterbox::dispatcher::Sender {
        slack,
        telegram,
        discord,
        discord_webhook,
        teams,
        gotify,
        email,
        // Resend is a hosted-controller delivery path; the standalone agent
        // doesn't expose it as a configurable dispatcher.
        resend: None,
        ntfy,
        pushover,
        matrix,
        webhook,
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
            .inform_container_failed(p.clone(), s.clone(), i.clone(), d.clone(), None)
            .await;
        handler
            .inform_rollback_complete(p.clone(), s.clone(), i.clone(), d.clone(), None)
            .await;
        handler.inform_update_success(p, s, i, d).await;
        handler.test_message().await;
    }

    // A failed pull recurs on every scheduled check; only the first within the
    // interval should be enqueued, otherwise the controller (and every
    // notifier) is hit once a minute.
    #[tokio::test]
    async fn repeated_pull_failure_is_throttled_within_interval() {
        let (handler, mut rx) = sample_handler();
        let (p, s, i, _d) = sample_args();

        handler
            .inform_pull_failed(p.clone(), s.clone(), i.clone(), "unauthorized".into())
            .await;
        handler
            .inform_pull_failed(p.clone(), s.clone(), i.clone(), "unauthorized".into())
            .await;

        assert!(rx.try_recv().is_ok(), "first failure should be reported");
        assert!(
            rx.try_recv().is_err(),
            "second identical failure within the interval should be suppressed"
        );
    }

    // Distinct services failing must each be reported — the throttle is keyed
    // per (project, service, image), not global.
    #[tokio::test]
    async fn pull_failures_for_different_services_are_reported() {
        let (handler, mut rx) = sample_handler();
        let (p, _s, i, _d) = sample_args();

        handler
            .inform_pull_failed(p.clone(), ServiceName::new("a"), i.clone(), "x".into())
            .await;
        handler
            .inform_pull_failed(p.clone(), ServiceName::new("b"), i.clone(), "x".into())
            .await;

        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
    }

    // A successful update re-arms reporting so a genuinely new failure after a
    // recovery is not silenced for the rest of the interval.
    #[tokio::test]
    async fn successful_update_rearms_pull_failure_reporting() {
        let (handler, mut rx) = sample_handler();
        let (p, s, i, d) = sample_args();

        handler
            .inform_pull_failed(p.clone(), s.clone(), i.clone(), "unauthorized".into())
            .await;
        let _ = rx.try_recv(); // consume the failure
        handler
            .inform_update_success(p.clone(), s.clone(), i.clone(), d)
            .await;
        let _ = rx.try_recv(); // consume the success

        handler
            .inform_pull_failed(p, s, i, "unauthorized".into())
            .await;
        assert!(
            rx.try_recv().is_ok(),
            "a failure after a successful update should be reported again"
        );
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
            logs: None,
        };
        assert!(send_to_chatterbox(&msg, None).await.is_ok());
    }
}
