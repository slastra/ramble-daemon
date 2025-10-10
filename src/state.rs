//! Shared application state management

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
}

#[derive(Debug)]
struct AppStateInner {
    /// Username of the current user
    username: String,

    /// Is the web app currently active/focused?
    web_app_active: bool,

    /// Is the daemon connected to the server?
    server_connected: bool,

    /// Last heartbeat received from web app
    last_heartbeat: Option<Instant>,

    /// Heartbeat timeout in seconds
    heartbeat_timeout: u64,
}

impl AppState {
    /// Create new application state
    pub fn new(username: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                username,
                web_app_active: false,
                server_connected: false,
                last_heartbeat: None,
                heartbeat_timeout: 30, // 30 seconds
            })),
        }
    }

    /// Get the username
    pub async fn username(&self) -> String {
        self.inner.read().await.username.clone()
    }

    /// Set web app as active
    pub async fn set_active(&self) {
        let mut inner = self.inner.write().await;
        inner.web_app_active = true;
        inner.last_heartbeat = Some(Instant::now());
        tracing::info!("Web app set to active");
    }

    /// Set web app as inactive
    pub async fn set_inactive(&self) {
        let mut inner = self.inner.write().await;
        inner.web_app_active = false;
        tracing::info!("Web app set to inactive");
    }

    /// Update heartbeat timestamp
    pub async fn update_heartbeat(&self) {
        let mut inner = self.inner.write().await;
        inner.last_heartbeat = Some(Instant::now());
        tracing::trace!("Heartbeat updated");
    }

    /// Check if web app is active
    pub async fn is_web_app_active(&self) -> bool {
        let inner = self.inner.read().await;

        // If web app is marked inactive, return false
        if !inner.web_app_active {
            return false;
        }

        // Check if heartbeat has timed out
        if let Some(last_heartbeat) = inner.last_heartbeat {
            let elapsed = last_heartbeat.elapsed().as_secs();
            if elapsed > inner.heartbeat_timeout {
                tracing::warn!("Heartbeat timeout ({}s), marking web app as inactive", elapsed);
                return false;
            }
        }

        true
    }

    /// Set server connection status
    pub async fn set_server_connected(&self, connected: bool) {
        let mut inner = self.inner.write().await;
        inner.server_connected = connected;
        tracing::info!("Server connection status: {}", connected);
    }

    /// Check if connected to server
    pub async fn is_server_connected(&self) -> bool {
        self.inner.read().await.server_connected
    }

    /// Check if heartbeat has timed out (and update state if needed)
    pub async fn check_heartbeat_timeout(&self) {
        let mut inner = self.inner.write().await;

        if !inner.web_app_active {
            return;
        }

        if let Some(last_heartbeat) = inner.last_heartbeat {
            let elapsed = last_heartbeat.elapsed().as_secs();
            if elapsed > inner.heartbeat_timeout {
                tracing::warn!("Heartbeat timeout ({}s), marking web app as inactive", elapsed);
                inner.web_app_active = false;
            }
        }
    }
}
