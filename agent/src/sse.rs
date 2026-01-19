use crate::docker::DockerHandler;
use controller::sse::ControllerEvent;
use log::{info, warn};
use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};
use tokio_stream::StreamExt;

pub struct SSEHandler {
    docker: Arc<DockerHandler>,
    rx: mpsc::Receiver<ControllerEvent>,
}

impl SSEHandler {
    pub(crate) fn new(docker: Arc<DockerHandler>, rx: mpsc::Receiver<ControllerEvent>) -> Self {
        Self { docker, rx }
    }

    pub(crate) async fn start(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                ControllerEvent::Retry((project_name, container_id)) => {
                    self.docker
                        .update_container(&project_name, &container_id)
                        .await
                        .expect("TODO: panic message");
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub(super) enum SSEError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

pub(crate) async fn consume_sse(
    url: &str,
    tx_sse: Sender<ControllerEvent>,
) -> Result<(), SSEError> {
    let client = Client::new();

    loop {
        info!("Connecting to SSE...");

        match try_consume_stream(&client, url, &tx_sse).await {
            Ok(_) => info!("Stream ended normally"),
            Err(e) => warn!("Stream error: {}", e),
        }

        println!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn try_consume_stream(
    client: &Client,
    url: &str,
    tx_sse: &Sender<ControllerEvent>,
) -> Result<(), SSEError> {
    let response = client.get(url).send().await?;
    let response = response.error_for_status()?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = StreamExt::next(&mut stream).await {
        let bytes = chunk?;
        let text = String::from_utf8_lossy(&bytes);
        buffer.push_str(&text);

        while let Some(pos) = buffer.find("\n\n") {
            let message = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            for line in message.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    let event: ControllerEvent = serde_json::from_str(data).unwrap();
                    println!("Received: {:?}", event);
                    tx_sse.send(event).await.unwrap();
                }
            }
        }
    }

    Ok(())
}
