use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{self, Stream};
use log::info;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;

pub type ContainerID = String;

#[derive(Debug, Deserialize, Serialize)]
pub enum ControllerEvent {
    Retry(ContainerID),
}

pub(super) async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| {
        info!("senthi");
        let x = ControllerEvent::Retry("hi".to_string());
        Event::default().json_data(x).unwrap()
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
