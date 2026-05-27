use crate::domain::billing::ports::BillingService;
use crate::domain::container_state::port::ContainerStateService;
use crate::domain::deployments::ports::DeploymentsService;
use crate::domain::notifiers::ports::NotifierService;
use crate::domain::tokens::ports::TokenService;
use crate::inbound::server::{AppState, UserId};
use axum::Extension;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use std::convert::Infallible;

pub use hoister_shared::ContainerID;
pub use hoister_shared::wire::ControllerEvent;

/// Internal broadcast payload. The first element is the owning user_id —
/// `sse_handler` uses it to deliver each event only to that user's
/// subscribers. The wire format sent to agents is just `ControllerEvent`.
pub type UserScopedEvent = (String, ControllerEvent);

pub(crate) async fn sse_handler<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS>>,
    Extension(UserId(subscriber_user_id)): Extension<UserId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok((event_user_id, event)) = rx.recv().await {
            if event_user_id != subscriber_user_id {
                continue;
            }
            if let Ok(sse_event) = Event::default().json_data(event) {
                yield Ok(sse_event);
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
