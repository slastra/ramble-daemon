# Ramble Daemon

Native Linux desktop notifications for [Ramble](https://ramble.chat) - an ephemeral, RAM-based chat application.

## What It Does

Ramble Daemon provides desktop notifications for chat messages while you're away from the web app. It intelligently suppresses notifications when you're actively using Ramble, so you only get notified when needed.

## Installation

### Prerequisites

- Linux with D-Bus notification support (dunst, mako, notification-daemon, etc.)
- Rust 1.70+ for building from source

### Build and Install

```bash
git clone https://github.com/yourusername/ramble-daemon.git
cd ramble-daemon
cargo build --release
cp target/release/ramble-daemon ~/.local/bin/
```

Or use the install script:

```bash
./install.sh
```

## Configuration

Create `~/.config/ramble/config.toml`:

```toml
username = "your-username"
server_url = "https://ramble.chat"
local_port = 9001
log_level = "info"
reconnect_delay = 5
notification_timeout = 5
```

Or run the setup wizard:

```bash
ramble-daemon setup
```

## Usage

Start the daemon:

```bash
ramble-daemon
```

### Auto-Start with Systemd

Create `~/.config/systemd/user/ramble-daemon.service`:

```ini
[Unit]
Description=Ramble Daemon
After=network.target

[Service]
Type=simple
ExecStart=%h/.local/bin/ramble-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user enable --now ramble-daemon
```

## How It Works

The daemon connects to the Ramble server via Server-Sent Events (SSE) to receive real-time messages. The web app communicates its activity status to the daemon via HTTP on localhost. Notifications only appear when the web app is inactive and the message is from someone else.

## License

MIT

## Contributing

Contributions are welcome. Please open an issue or pull request.
