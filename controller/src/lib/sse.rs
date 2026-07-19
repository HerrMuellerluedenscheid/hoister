use crate::domain::billing::ports::BillingService;
use crate::domain::container_state::port::ContainerStateService;
use crate::domain::deployments::ports::DeploymentsService;
use crate::domain::metrics::port::MetricsService;
use crate::domain::notifiers::ports::NotifierService;
use crate::domain::tokens::ports::TokenService;
use crate::inbound::server::{AppState, UserId};
use axum::Extension;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;

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
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(subscriber_user_id)): Extension<UserId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok((event_user_id, event)) => {
                    if event_user_id != subscriber_user_id {
                        continue;
                    }
                    if let Ok(sse_event) = Event::default().json_data(event) {
                        yield Ok(sse_event);
                    }
                }
                // A slow consumer of this SSE stream — or a burst of events from
                // any tenant, since all tenants share one broadcast channel —
                // can push messages out of the ring buffer before this task
                // reads them. `recv` then returns `Lagged(n)` for the n dropped
                // messages. The previous `while let Ok(..)` treated this exactly
                // like `Closed` and ended the stream silently, so the agent
                // stopped receiving ApplyUpdate/RequestLogs commands until it
                // happened to reconnect. Log it and keep going: `recv` resumes
                // from the oldest still-buffered message on the next call.
                Err(RecvError::Lagged(skipped)) => {
                    log::warn!(
                        "SSE subscriber {subscriber_user_id} lagged; dropped \
                         {skipped} broadcast event(s). Some ApplyUpdate/RequestLogs \
                         commands for this user may have been missed."
                    );
                    continue;
                }
                // The sender was dropped (controller shutting down); no further
                // events will ever arrive, so end the stream.
                Err(RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
