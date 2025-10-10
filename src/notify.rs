//! Desktop notification management via D-Bus

use crate::protocol::ChatMessage;
use anyhow::Result;
use notify_rust::{Notification, Timeout};

/// Notification manager for desktop notifications
pub struct NotificationManager {
    timeout: i32,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new(timeout: i32) -> Self {
        Self { timeout }
    }

    /// Show a notification for a chat message
    pub fn show_message(&self, message: &ChatMessage) -> Result<()> {
        let body = self.truncate_message(&message.content);

        Notification::new()
            .summary(&format!("Ramble - {}", message.author))
            .body(&body)
            .appname("Ramble")
            .timeout(Timeout::Milliseconds((self.timeout * 1000) as u32))
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to show notification: {}", e))?;

        tracing::debug!("Notification shown for message from {}", message.author);

        Ok(())
    }

    /// Show a system notification
    pub fn show_system(&self, content: &str) -> Result<()> {
        Notification::new()
            .summary("Ramble")
            .body(content)
            .appname("Ramble")
            .timeout(Timeout::Milliseconds((self.timeout * 1000) as u32))
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to show notification: {}", e))?;

        tracing::debug!("System notification shown: {}", content);

        Ok(())
    }

    /// Show a connection status notification
    pub fn show_connection_status(&self, connected: bool) -> Result<()> {
        let message = if connected {
            "Connected to Ramble server"
        } else {
            "Disconnected from Ramble server"
        };

        Notification::new()
            .summary("Ramble Daemon")
            .body(message)
            .appname("Ramble")
            .timeout(Timeout::Milliseconds((self.timeout * 1000) as u32))
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to show notification: {}", e))?;

        tracing::info!("Connection status notification: {}", message);

        Ok(())
    }

    /// Truncate message content if too long
    fn truncate_message(&self, content: &str) -> String {
        const MAX_LENGTH: usize = 200;

        if content.len() <= MAX_LENGTH {
            content.to_string()
        } else {
            format!("{}...", &content[..MAX_LENGTH])
        }
    }
}

/// Determine if a notification should be shown
pub fn should_notify(
    message: &ChatMessage,
    current_username: &str,
) -> bool {
    // Never notify for own messages
    if message.author == current_username {
        tracing::trace!("Skipping notification for own message");
        return false;
    }

    // Always show notifications for other users
    true
}
