use controller::sse::ControllerEvent;
use reqwest::Client;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};
use tokio_stream::StreamExt;

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
        println!("Connecting to SSE...");

        match try_consume_stream(&client, url, &tx_sse).await {
            Ok(_) => println!("Stream ended normally"),
            Err(e) => eprintln!("Stream error: {}", e),
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
