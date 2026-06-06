//! Bridge from `Vec<Notifier>` to chatterbox dispatchers.
//!
//! chatterbox's `Sender` has at-most-one transport of each kind. The
//! controller's data model lets a user have many of the same kind (e.g.
//! two Slack channels), so we build a small one-kind `Sender` per notifier
//! and dispatch each independently. Errors are logged and swallowed —
//! a broken notifier mustn't block the deployment POST.

use crate::domain::notifiers::models::{Notifier, NotifierConfig};
use chatterbox::dispatcher::Sender;
use chatterbox::message::{Dispatcher, Message};
use log::warn;

/// Controller-wide email delivery settings (Resend). Sourced from env at
/// startup — email notifiers store only the recipient, never SMTP/API
/// credentials. When `None`, dispatching an email notifier is an error.
#[derive(Clone, Debug)]
pub struct EmailDispatchConfig {
    pub resend_api_key: String,
    pub from: String,
}

/// Dispatch `message` to every enabled notifier. Synchronous chatterbox
/// calls are run in a `spawn_blocking` so an unresponsive notifier
/// endpoint doesn't tie up the tokio worker — caller stays in async
/// context.
pub async fn dispatch_to_all(
    notifiers: Vec<Notifier>,
    message: Message,
    email: Option<EmailDispatchConfig>,
) {
    for notifier in notifiers {
        if !notifier.enabled {
            continue;
        }
        let kind = notifier.kind;
        let msg = message.clone();
        let email = email.clone();
        let res =
            tokio::task::spawn_blocking(move || dispatch_one(notifier, msg, email.as_ref())).await;
        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => warn!("notifier {kind:?} dispatch failed: {e}"),
            Err(e) => warn!("notifier {kind:?} dispatch task panicked: {e}"),
        }
    }
}

fn dispatch_one(
    notifier: Notifier,
    message: Message,
    email: Option<&EmailDispatchConfig>,
) -> Result<(), String> {
    let sender = sender_for(notifier.config, email)?;
    let dispatcher = Dispatcher::new(sender);
    dispatcher.dispatch(&message).map_err(|e| e.to_string())
}

/// Like `dispatch_one` but async (wraps the blocking dispatcher call) and
/// returns the error to the caller instead of swallowing it. Used by the
/// "test notifier" handler so the user sees why a misconfigured channel
/// failed; not used by deployment-event dispatch, which still fans out
/// fire-and-forget via [`dispatch_to_all`].
pub async fn dispatch_one_async(
    notifier: Notifier,
    message: Message,
    email: Option<EmailDispatchConfig>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || dispatch_one(notifier, message, email.as_ref()))
        .await
        .map_err(|e| format!("dispatch task panicked: {e}"))?
}

fn sender_for(
    config: NotifierConfig,
    email: Option<&EmailDispatchConfig>,
) -> Result<Sender, String> {
    let mut sender = Sender::default();
    match config {
        NotifierConfig::Slack(s) => {
            sender.slack = Some(chatterbox::dispatcher::slack::Slack {
                webhook_url: s.webhook,
                channel: s.channel,
            });
        }
        NotifierConfig::Telegram(t) => {
            sender.telegram = Some(chatterbox::dispatcher::telegram::Telegram {
                bot_token: t.bot_token,
                chat_id: t.chat_id,
            });
        }
        NotifierConfig::Discord(d) => {
            sender.discord = Some(chatterbox::dispatcher::discord::Discord {
                bot_token: d.bot_token,
                channel_id: d.channel_id,
            });
        }
        NotifierConfig::DiscordWebhook(d) => {
            sender.discord_webhook =
                Some(chatterbox::dispatcher::discord_webhook::DiscordWebhook {
                    webhook_url: d.webhook,
                    username: None,
                    avatar_url: None,
                });
        }
        NotifierConfig::Teams(t) => {
            sender.teams = Some(chatterbox::dispatcher::teams::Teams {
                webhook_url: t.webhook,
            });
        }
        NotifierConfig::Gotify(g) => {
            let server_url = url::Url::parse(&g.server)
                .map_err(|e| format!("invalid gotify server url: {e}"))?;
            sender.gotify = Some(chatterbox::dispatcher::gotify::Gotify {
                server_url,
                app_token: g.token,
            });
        }
        NotifierConfig::Email(e) => {
            let cfg = email.ok_or_else(|| {
                "email notifier configured but the controller has no Resend \
                 credentials (HOISTER_CONTROLLER_RESEND_API_KEY / \
                 HOISTER_CONTROLLER_EMAIL_FROM)"
                    .to_string()
            })?;
            sender.resend = Some(chatterbox::dispatcher::resend::Resend {
                api_key: cfg.resend_api_key.clone(),
                from: cfg.from.clone(),
                to: e.recipient,
            });
        }
        NotifierConfig::Ntfy(n) => {
            let server_url =
                url::Url::parse(&n.server).map_err(|e| format!("invalid ntfy server url: {e}"))?;
            sender.ntfy = Some(chatterbox::dispatcher::ntfy::Ntfy {
                server_url,
                topic: n.topic,
                access_token: n.access_token,
            });
        }
        NotifierConfig::Pushover(p) => {
            sender.pushover = Some(chatterbox::dispatcher::pushover::Pushover {
                token: p.token,
                user: p.user,
                device: p.device,
            });
        }
        NotifierConfig::Matrix(m) => {
            let homeserver_url = url::Url::parse(&m.homeserver)
                .map_err(|e| format!("invalid matrix homeserver url: {e}"))?;
            sender.matrix = Some(chatterbox::dispatcher::matrix::Matrix {
                homeserver_url,
                access_token: m.access_token,
                room_id: m.room_id,
            });
        }
        NotifierConfig::Webhook(w) => {
            sender.webhook = Some(chatterbox::dispatcher::webhook::Webhook {
                url: w.url,
                headers: w.headers,
            });
        }
    }
    Ok(sender)
}
