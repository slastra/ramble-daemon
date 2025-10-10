//! Configuration management for Ramble daemon

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// User's display name in Ramble
    pub username: String,

    /// Ramble server URL (e.g., "https://ramble.chat")
    pub server_url: String,

    /// Local HTTP server port
    #[serde(default = "default_local_port")]
    pub local_port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Seconds between reconnection attempts
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay: u64,

    /// Notification display timeout in seconds
    #[serde(default = "default_notification_timeout")]
    pub notification_timeout: i32,
}

fn default_local_port() -> u16 {
    9001
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_reconnect_delay() -> u64 {
    5
}

fn default_notification_timeout() -> i32 {
    5
}

impl Config {
    /// Load configuration from the default location (~/.config/ramble/config.toml)
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        Self::load_from_path(&path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;

        config.validate()?;

        Ok(config)
    }

    /// Get the default config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?;

        Ok(config_dir.join("ramble").join("config.toml"))
    }

    /// Create the config directory if it doesn't exist
    pub fn ensure_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("ramble");

        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory: {}", config_dir.display()))?;

        Ok(config_dir)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        if self.username.is_empty() {
            anyhow::bail!("username cannot be empty");
        }

        if self.server_url.is_empty() {
            anyhow::bail!("server_url cannot be empty");
        }

        if !self.server_url.starts_with("http://") && !self.server_url.starts_with("https://") {
            anyhow::bail!("server_url must start with http:// or https://");
        }

        Ok(())
    }

    /// Create an example configuration file
    pub fn create_example(path: &PathBuf) -> Result<()> {
        let example = Config {
            username: "your-username".to_string(),
            server_url: "https://ramble.chat".to_string(),
            local_port: default_local_port(),
            log_level: default_log_level(),
            reconnect_delay: default_reconnect_delay(),
            notification_timeout: default_notification_timeout(),
        };

        let content = toml::to_string_pretty(&example)
            .context("Failed to serialize example config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
}
