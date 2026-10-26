# ramble-daemon

Native Linux desktop notification daemon for Ramble.

ramble-daemon shows desktop notifications for [Ramble](https://github.com/slastra/ramble) chat messages while you're away from the web app.

## Install

### Requirements

- Linux with D-Bus notification support (dunst, mako, notification-daemon, etc.)
- A Rust toolchain for building from source

### From source

```bash
git clone https://github.com/slastra/ramble-daemon.git
cd ramble-daemon
cargo build --release
cp target/release/ramble-daemon ~/.local/bin/
```

Or use the install script, which builds, copies the binary to `/usr/local/bin` (or `~/.local/bin` when that isn't writable) and writes an example config:

```bash
./install.sh
```

## Usage

Start the daemon:

```bash
ramble-daemon
ramble-daemon --config path/to/config.toml   # default: ~/.config/ramble/config.toml
ramble-daemon setup                          # write an example config file
```

### Auto-start with systemd

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

## Configuration

Create `~/.config/ramble/config.toml`:

```toml
username = "your-username"
server_url = "https://ramble.chat"
token = "the-server-EVENTS_TOKEN"  # must match EVENTS_TOKEN on the Ramble server
local_port = 9001
log_level = "info"
reconnect_delay = 5
notification_timeout = 5
```

`username`, `server_url` and `token` are required; the rest default to the values shown. `ramble-daemon setup` writes this file for you to edit.

## How it works

The daemon connects to the Ramble server via Server-Sent Events (SSE) to receive real-time messages, and notifies for every message from someone else. The web app communicates its activity status to the daemon via HTTP on localhost (`/active`, `/inactive`, `/heartbeat`, and `/status` to read it back); the daemon tracks that status but does not yet use it to suppress notifications.

## Contributing

Contributions are welcome. Please open an issue or pull request.

## License

MIT © 2025 Ramble Contributors. See [LICENSE](LICENSE).
