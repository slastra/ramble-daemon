//! Ramble Daemon - Native Linux desktop notifications for Ramble chat

mod config;
mod http;
mod notify;
mod protocol;
mod sse;
mod state;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use notify::NotificationManager;
use protocol::ServerEvent;
use sse::SseClient;
use state::AppState;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ramble-daemon")]
#[command(about = "Native Linux desktop notifications for Ramble chat", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to config file (default: ~/.config/ramble/config.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an example configuration file
    Setup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(Commands::Setup) = cli.command {
        return setup_config().await;
    }

    // Load configuration
    let config = if let Some(config_path) = cli.config {
        Config::load_from_path(&config_path)?
    } else {
        Config::load().with_context(|| {
            format!(
                "Failed to load config. Run 'ramble-daemon setup' to create a config file at {}",
                Config::config_path().unwrap_or_default().display()
            )
        })?
    };

    // Initialize logging
    init_logging(&config.log_level)?;

    tracing::info!("Starting Ramble daemon v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Username: {}", config.username);
    tracing::info!("Server: {}", config.server_url);
    tracing::info!("Local port: {}", config.local_port);

    // Create shared state
    let state = AppState::new(config.username.clone());

    // Create notification manager
    let notify_manager = NotificationManager::new(config.notification_timeout);

    // Create SSE client
    let sse_client = SseClient::new(
        config.server_url.clone(),
        config.username.clone(),
        config.token.clone(),
        config.reconnect_delay,
        state.clone(),
    );

    // Spawn heartbeat timeout checker
    let state_clone = state.clone();
    tokio::spawn(async move {
        heartbeat_checker(state_clone).await;
    });

    // Spawn HTTP server
    let http_state = state.clone();
    let http_port = config.local_port;
    tokio::spawn(async move {
        if let Err(e) = http::start_server(http_port, http_state).await {
            tracing::error!("HTTP server error: {}", e);
            std::process::exit(1);
        }
    });

    // Start SSE client (this blocks)
    let username = config.username.clone();
    let state_for_events = state.clone();

    sse_client
        .connect(move |event| {
            handle_server_event(event, &username, &state_for_events, &notify_manager)
        })
        .await?;

    Ok(())
}

/// Handle events from the server
fn handle_server_event(
    event: ServerEvent,
    username: &str,
    _state: &AppState,
    notify_manager: &NotificationManager,
) -> Result<()> {
    match event {
        ServerEvent::Message(message) => {
            if notify::should_notify(&message, username) {
                if let Err(e) = notify_manager.show_message(&message) {
                    tracing::error!("Failed to show notification: {}", e);
                }
            }
        }
        ServerEvent::System(message) => {
            // Optionally show system messages as notifications
            tracing::info!("System: {}", message.content);
        }
        ServerEvent::Presence(_) => {
            // Presence updates are logged in the SSE client
        }
    }

    Ok(())
}

/// Periodically check for heartbeat timeout
async fn heartbeat_checker(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;
        state.check_heartbeat_timeout().await;
    }
}

/// Initialize logging based on log level
fn init_logging(log_level: &str) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    Ok(())
}

/// Setup configuration interactively
async fn setup_config() -> Result<()> {
    use std::io::{self, Write};

    println!("Ramble Daemon Setup");
    println!("==================\n");

    let config_dir = Config::ensure_config_dir()?;
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        print!("Config file already exists at {}. Overwrite? [y/N] ", config_path.display());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Setup cancelled.");
            return Ok(());
        }
    }

    Config::create_example(&config_path)?;

    println!("✓ Created example config at: {}", config_path.display());
    println!("\nPlease edit this file and set your username, server URL, and token.");
    println!("Then run 'ramble-daemon' to start the daemon.");

    Ok(())
}
