//! Message types and protocol definitions for Ramble events

use serde::{Deserialize, Serialize};

/// SSE event types from the Ramble server
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseEventType {
    Message,
    System,
    Presence,
}

/// Chat message from the server
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub author: String,
    pub content: String,
    pub room: String,
    pub timestamp: u64,
    /// Message type (ignored, we use SSE event type instead)
    #[serde(rename = "type")]
    #[serde(skip_serializing)]
    pub message_type: Option<String>,
}

/// System message from the server
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemMessage {
    pub content: String,
}

/// Presence update from the server
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresenceUpdate {
    pub username: String,
    pub status: String,
}

/// All possible event types from the server
#[derive(Debug, Clone)]
pub enum ServerEvent {
    Message(ChatMessage),
    System(SystemMessage),
    Presence(PresenceUpdate),
}

/// Activity status from the web app
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityStatus {
    #[serde(default)]
    pub active: bool,
}

/// HTTP response for web app requests
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiResponse {
    pub status: String,
}

impl ApiResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
        }
    }
}

/// Daemon status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DaemonStatus {
    pub daemon: String,
    pub server_connected: bool,
    pub web_app_active: bool,
}
