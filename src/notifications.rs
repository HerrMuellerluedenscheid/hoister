use crate::HoisterError;
use chatterbox::message::{Dispatcher, Message};
use log::{debug, error, info};

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
    let sender = chatterbox::dispatcher::Sender {
        slack,
        telegram,
        email: None,
    };

    Dispatcher::new(sender)
}

impl From<HoisterError> for Option<Message<'_>> {
    fn from(value: HoisterError) -> Self {
        match value {
            HoisterError::NoUpdateAvailable => {
                debug!("no update available");
                None
            }
            HoisterError::UpdateFailed(e) => Some(Message::new_now(
                "update failed",
                format!("failed to update image {}", e),
            )),
            _ => {
                error!("unexpected error: {:?}", value);
                None
            }
        }
    }
}
