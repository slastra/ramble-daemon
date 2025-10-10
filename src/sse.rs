//! SSE client for receiving events from the Ramble server

use crate::protocol::{ChatMessage, ServerEvent, SystemMessage};
use crate::state::AppState;
use anyhow::{Context, Result};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::sleep;

/// SSE client configuration
pub struct SseClient {
    server_url: String,
    username: String,
    reconnect_delay: Duration,
    state: AppState,
}

impl SseClient {
    /// Create a new SSE client
    pub fn new(server_url: String, username: String, reconnect_delay: u64, state: AppState) -> Self {
        Self {
            server_url,
            username,
            reconnect_delay: Duration::from_secs(reconnect_delay),
            state,
        }
    }

    /// Connect to the SSE endpoint and start receiving events
    pub async fn connect<F>(&self, mut on_event: F) -> Result<()>
    where
        F: FnMut(ServerEvent) -> Result<()>,
    {
        let url = format!("{}/api/events?username={}", self.server_url, self.username);
        tracing::info!("Connecting to SSE endpoint: {}", url);

        let mut backoff = ExponentialBackoff::new(self.reconnect_delay);

        loop {
            match self.try_connect(&url, &mut on_event).await {
                Ok(_) => {
                    tracing::info!("SSE connection closed normally");
                    backoff.reset();
                    self.state.set_server_connected(false).await;
                }
                Err(e) => {
                    tracing::error!("SSE connection error: {}", e);
                    self.state.set_server_connected(false).await;

                    let delay = backoff.next_delay();
                    tracing::info!("Reconnecting in {:?}...", delay);
                    sleep(delay).await;
                }
            }
        }
    }

    /// Attempt to connect and process events
    async fn try_connect<F>(&self, url: &str, on_event: &mut F) -> Result<()>
    where
        F: FnMut(ServerEvent) -> Result<()>,
    {
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to connect to SSE endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        tracing::info!("Connected to SSE stream");
        self.state.set_server_connected(true).await;

        let mut stream = response.bytes_stream().eventsource();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    if let Err(e) = self.handle_event(event, on_event) {
                        tracing::error!("Error handling event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading event: {}", e);
                    return Err(anyhow::anyhow!("Stream error: {}", e));
                }
            }
        }

        Ok(())
    }

    /// Handle a single SSE event
    fn handle_event<F>(
        &self,
        event: eventsource_stream::Event,
        on_event: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ServerEvent) -> Result<()>,
    {
        let event_type = event.event.as_str();
        let data = event.data;

        tracing::debug!("Received SSE event: type={}, data={}", event_type, data);

        let server_event = match event_type {
            "message" => {
                let message: ChatMessage = serde_json::from_str(&data)
                    .context("Failed to parse chat message")?;
                ServerEvent::Message(message)
            }
            "system" => {
                let message: SystemMessage = serde_json::from_str(&data)
                    .context("Failed to parse system message")?;
                ServerEvent::System(message)
            }
            "presence" => {
                // Presence updates are optional, just log for now
                tracing::debug!("Received presence update: {}", data);
                return Ok(());
            }
            _ => {
                tracing::warn!("Unknown event type: {}", event_type);
                return Ok(());
            }
        };

        on_event(server_event)?;

        Ok(())
    }
}

/// Exponential backoff for reconnection attempts
struct ExponentialBackoff {
    base_delay: Duration,
    current_delay: Duration,
    max_delay: Duration,
}

impl ExponentialBackoff {
    fn new(base_delay: Duration) -> Self {
        Self {
            base_delay,
            current_delay: base_delay,
            max_delay: Duration::from_secs(60),
        }
    }

    fn next_delay(&mut self) -> Duration {
        let delay = self.current_delay;
        self.current_delay = std::cmp::min(self.current_delay * 2, self.max_delay);
        delay
    }

    fn reset(&mut self) {
        self.current_delay = self.base_delay;
    }
}
