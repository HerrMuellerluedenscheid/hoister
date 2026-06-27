use crate::docker::DockerHandler;
use hoister_shared::wire::{ControllerEvent, PostContainerLogsRequest};
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::{info, warn};
use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};
use tokio_stream::StreamExt;
use url::Url;

pub struct SSEHandler {
    docker: Arc<DockerHandler>,
    rx: mpsc::Receiver<ControllerEvent>,
    hostname: HostName,
    /// Mirror of `HOISTER_REPORT_LOGS`. Log requests are ignored unless set,
    /// since container logs can carry secrets keyword redaction won't catch.
    report_logs: bool,
    /// Used to ship requested logs back to the controller.
    client: Client,
    /// Base controller URL (e.g. `https://api.hoister.io/`), the same one the
    /// monitor POSTs container state to.
    controller_url: Url,
    token: Option<String>,
}

impl SSEHandler {
    pub(crate) fn new(
        docker: Arc<DockerHandler>,
        rx: mpsc::Receiver<ControllerEvent>,
        hostname: HostName,
        report_logs: bool,
        client: Client,
        controller_url: Url,
        token: Option<String>,
    ) -> Self {
        Self {
            docker,
            rx,
            hostname,
            report_logs,
            client,
            controller_url,
            token,
        }
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
                ControllerEvent::ApplyUpdate((target_host, project_name, service_name)) => {
                    if target_host == self.hostname {
                        if let Some(container_id) = self
                            .docker
                            .find_container_by_service(&project_name, &service_name)
                            .await
                        {
                            if let Err(e) = self
                                .docker
                                .apply_update_container(&project_name, &container_id)
                                .await
                            {
                                warn!("Failed to apply update for {}: {e}", service_name.as_str());
                            }
                        } else {
                            warn!(
                                "ApplyUpdate: no container found for service {} in project {}",
                                service_name.as_str(),
                                project_name.as_str()
                            );
                        }
                    }
                }
                ControllerEvent::RequestLogs((target_host, project_name, service_name)) => {
                    if target_host == self.hostname {
                        self.handle_log_request(&project_name, &service_name).await;
                    }
                }
            }
        }
    }

    /// Honour an on-demand `RequestLogs` event: fetch the service's current log
    /// tail and ship it to the controller's in-memory store. Gated on
    /// `report_logs` so an operator who never opted in leaks nothing, even if
    /// someone triggers a request from the dashboard.
    async fn handle_log_request(&self, project_name: &ProjectName, service_name: &ServiceName) {
        if !self.report_logs {
            info!(
                "Ignoring log request for service {} — set HOISTER_REPORT_LOGS=true to enable",
                service_name.as_str()
            );
            return;
        }

        // Always answer the request, even with an empty body, so the dashboard
        // can distinguish "no logs" from "still waiting".
        let logs = self
            .docker
            .fetch_service_logs(project_name, service_name)
            .await
            .unwrap_or_default();

        if let Err(e) = self
            .post_requested_logs(project_name, service_name, logs)
            .await
        {
            warn!(
                "Failed to ship requested logs for service {}: {e}",
                service_name.as_str()
            );
        }
    }

    async fn post_requested_logs(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        logs: String,
    ) -> Result<(), SSEError> {
        let url = self
            .controller_url
            .join(&format!(
                "container/logs/{}/{}/{}",
                self.hostname.as_str(),
                project_name.as_str(),
                service_name.as_str()
            ))
            .expect("controller log URL should be valid");

        let mut req = self
            .client
            .post(url)
            .json(&PostContainerLogsRequest { logs });
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req.send().await?.error_for_status()?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub(super) enum SSEError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

pub(crate) async fn consume_sse(
    url: &str,
    token: Option<String>,
    tx_sse: Sender<ControllerEvent>,
    client: Client,
) -> Result<(), SSEError> {
    loop {
        info!("Connecting to SSE...");

        match try_consume_stream(&client, url, token.as_deref(), &tx_sse).await {
            Ok(_) => info!("Stream ended normally"),
            Err(e) => warn!("Stream error: {e}"),
        }

        println!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn try_consume_stream(
    client: &Client,
    url: &str,
    token: Option<&str>,
    tx_sse: &Sender<ControllerEvent>,
) -> Result<(), SSEError> {
    let mut req = client.get(url);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let response = req.send().await?;
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
                    println!("Received: {event:?}");
                    tx_sse.send(event).await.unwrap();
                }
            }
        }
    }

    Ok(())
}
