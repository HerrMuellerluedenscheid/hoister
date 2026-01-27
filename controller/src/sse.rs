use crate::server::AppState;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use hoister_shared::ProjectName;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

pub type ContainerID = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ControllerEvent {
    Retry((ProjectName, ContainerID)),
}

pub(crate) async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            if let Ok(sse_event) = Event::default().json_data(event) {
                yield Ok(sse_event);
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
