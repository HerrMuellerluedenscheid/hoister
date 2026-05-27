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

/// Dispatch `message` to every enabled notifier. Synchronous chatterbox
/// calls are run in a `spawn_blocking` so an unresponsive notifier
/// endpoint doesn't tie up the tokio worker — caller stays in async
/// context.
pub async fn dispatch_to_all(notifiers: Vec<Notifier>, message: Message) {
    for notifier in notifiers {
        if !notifier.enabled {
            continue;
        }
        let kind = notifier.kind;
        let msg = message.clone();
        let res = tokio::task::spawn_blocking(move || dispatch_one(notifier, msg)).await;
        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => warn!("notifier {kind:?} dispatch failed: {e}"),
            Err(e) => warn!("notifier {kind:?} dispatch task panicked: {e}"),
        }
    }
}

fn dispatch_one(notifier: Notifier, message: Message) -> Result<(), String> {
    let sender = sender_for(notifier.config)?;
    let dispatcher = Dispatcher::new(sender);
    dispatcher.dispatch(&message).map_err(|e| e.to_string())
}

/// Like `dispatch_one` but async (wraps the blocking dispatcher call) and
/// returns the error to the caller instead of swallowing it. Used by the
/// "test notifier" handler so the user sees why a misconfigured channel
/// failed; not used by deployment-event dispatch, which still fans out
/// fire-and-forget via [`dispatch_to_all`].
pub async fn dispatch_one_async(notifier: Notifier, message: Message) -> Result<(), String> {
    tokio::task::spawn_blocking(move || dispatch_one(notifier, message))
        .await
        .map_err(|e| format!("dispatch task panicked: {e}"))?
}

fn sender_for(config: NotifierConfig) -> Result<Sender, String> {
    let mut sender = Sender {
        slack: None,
        telegram: None,
        discord: None,
        gotify: None,
        email: None,
    };
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
        NotifierConfig::Gotify(g) => {
            let server_url = url::Url::parse(&g.server)
                .map_err(|e| format!("invalid gotify server url: {e}"))?;
            sender.gotify = Some(chatterbox::dispatcher::gotify::Gotify {
                server_url,
                app_token: g.token,
            });
        }
        NotifierConfig::Email(e) => {
            sender.email = Some(chatterbox::dispatcher::email::Email {
                smtp_user: e.smtp_user.clone(),
                smtp_password: e.smtp_password,
                smtp_server: e.smtp_server,
                smtp_port: 587,
                receiver_address: e.recipient,
                sender_address: e.smtp_user,
                sender_name: e.from.unwrap_or_else(|| "hoister".to_string()),
            });
        }
    }
    Ok(sender)
}
