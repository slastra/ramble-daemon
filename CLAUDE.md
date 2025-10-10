# Ramble Daemon Development Guide

## Project Overview

**Ramble** is an ephemeral, RAM-based chat application. This daemon provides native Linux desktop notifications for Ramble users.

**Core Philosophy:**
- No persistent storage
- No user accounts
- Ephemeral by design
- Privacy-focused (no third-party notification services)

## Daemon Purpose

A lightweight Rust daemon that:
1. Connects to Ramble's Nitro server via Server-Sent Events (SSE)
2. Runs a local HTTP server for the web app to communicate with
3. Shows native Linux notifications via D-Bus
4. Intelligently suppresses notifications when the user is active in the web app

## Architecture

```
Ramble Web App (Browser)
    ↓ https:// → Nitro Server (ramble.chat)
    ↓ POST http://localhost:9001 → Ramble Daemon (local)

Ramble Daemon
    ↑ SSE https://ramble.chat/events → Nitro Server (receives messages)
    ↑ HTTP localhost:9001 ← Web App (receives status)
    ↓ D-Bus → Linux Notification System
```

## Technical Requirements

### Core Dependencies

**HTTP & SSE:**
- `reqwest` with `stream` feature for SSE client
- `axum` or `warp` for local HTTP server
- `tokio` for async runtime

**Notifications:**
- `notify-rust` for D-Bus desktop notifications
- Alternative: `zbus` for lower-level D-Bus control

**Serialization:**
- `serde` and `serde_json` for message parsing

**Configuration:**
- `toml` or `config` crate for config file parsing
- Config location: `~/.config/ramble/config.toml`

**Logging:**
- `tracing` or `env_logger` for structured logging

### Platform Support

**Target:** Linux (primary focus on systemd-based distros)

**Notification systems supported:**
- Desktop Notification Specification (D-Bus)
- Works with: dunst, mako, notification-daemon, etc.

## Configuration File Structure

**Location:** `~/.config/ramble/config.toml`

**Required fields:**
- `username` - User's chosen display name
- `server_url` - Ramble server WebSocket URL (e.g., `wss://ramble.chat`)
- `local_port` - Local WebSocket server port (default: 9001)

**Optional fields:**
- `log_level` - Logging verbosity
- `reconnect_delay` - Seconds between reconnection attempts
- `notification_timeout` - How long notifications stay visible

## Core Components

### 1. Configuration Manager

**Responsibilities:**
- Load config from `~/.config/ramble/config.toml`
- Validate required fields
- Provide defaults for optional fields
- Create config directory if missing

**Error handling:**
- Clear error messages if config missing
- Helpful prompts for first-time setup

### 2. Server SSE Client

**Responsibilities:**
- Connect to Ramble Nitro server SSE endpoint
- Subscribe with username as query parameter
- Receive chat messages as SSE events
- Handle reconnection on disconnect
- Exponential backoff on failures

**SSE Endpoint:**
- `GET https://ramble.chat/events?username={username}`
- Server keeps connection open and pushes events

**Event types to handle:**
- `event: message` - Chat messages
- `event: system` - System messages  
- `event: presence` - Presence updates (optional)

**Event data format:**
```
event: message
data: {"author": "alice", "content": "Hello!", "room": "general", "timestamp": 1234567890}

event: system
data: {"content": "alice joined the room"}
```

**Connection management:**
- SSE has built-in reconnection
- Additional exponential backoff on persistent failures
- Log connection state changes
- Handle network interruptions gracefully

### 3. Local HTTP Server

**Responsibilities:**
- Listen on `localhost:{local_port}`
- Accept POST requests from web app
- Receive status updates
- Respond with simple JSON acknowledgments

**Endpoints:**

`POST /active` - Web app is open and focused
```json
Request: {}
Response: {"status": "ok"}
```

`POST /inactive` - Web app lost focus or closed
```json
Request: {}
Response: {"status": "ok"}
```

`POST /heartbeat` - Keep-alive ping from web app
```json
Request: {}
Response: {"status": "ok"}
```

`GET /status` - Health check (optional)
```json
Response: {
  "daemon": "running",
  "server_connected": true,
  "web_app_active": false
}
```

**State tracking:**
- Track whether web app is currently active
- Reset to inactive if heartbeat times out (30s)
- Simple in-memory state, no persistence needed

### 4. Notification Manager

**Responsibilities:**
- Show desktop notifications via D-Bus
- Handle notification display logic
- Manage notification lifecycle

**Notification logic:**
```
IF message.author == config.username:
    SKIP (never notify own messages)
ELSE IF web_app_is_active:
    SKIP (user is already in the app)
ELSE:
    SHOW notification
```

**Notification format:**
- Title: Author's username or "Ramble"
- Body: Message content (truncate if too long)
- Icon: Ramble logo (optional)
- Urgency: Normal
- Timeout: Configurable (default: 5 seconds)

**Click handling:**
- Optional: Open Ramble in browser when clicked
- Use `xdg-open` or similar

### 5. State Manager

**In-memory state:**
- `web_app_active: bool` - Is web app currently focused/active
- `username: String` - Current user's name
- `connected_to_server: bool` - Server connection status
- `last_heartbeat: Option<Instant>` - Last ping from web app

**State transitions:**
- Web app connects → set active
- Web app sends "inactive" → set inactive
- Web app disconnects → set inactive
- No heartbeat for 30s → set inactive (connection lost)

## Message Protocol

### Server → Daemon (SSE from Nitro)

**SSE Connection:**
```
GET https://ramble.chat/events?username=alice
```

**Chat message event:**
```
event: message
data: {"author": "alice", "content": "Hello world!", "room": "general", "timestamp": 1234567890}
```

**System message event:**
```
event: system
data: {"content": "alice joined the room"}
```

### Web App → Daemon (HTTP POST to localhost)

**Activity status:**
```
POST http://localhost:9001/active
Content-Type: application/json
{}
```

```
POST http://localhost:9001/inactive
Content-Type: application/json
{}
```

**Heartbeat:**
```
POST http://localhost:9001/heartbeat
Content-Type: application/json
{}
```

## Error Handling

**Connection errors:**
- Log clearly when SSE connection fails
- SSE has automatic reconnection built-in
- Implement additional exponential backoff for persistent failures (1s, 2s, 4s, 8s, max 60s)
- Show notification when connection restored

**Configuration errors:**
- Fail fast with helpful error messages
- Suggest running `ramble-daemon --setup` for first-time config

**D-Bus errors:**
- Log but don't crash if notification fails
- Gracefully handle missing notification daemon

**Local HTTP server errors:**
- If port already in use, suggest killing old process or changing port
- Clear error message about firewall if binding fails

**SSE stream errors:**
- Handle malformed SSE events gracefully
- Log parse errors without crashing
- Continue processing next event

## CLI Interface

**Commands:**
- `ramble-daemon` - Start the daemon
- `ramble-daemon --setup` - Interactive config creation
- `ramble-daemon --config <path>` - Use custom config file
- `ramble-daemon --version` - Show version
- `ramble-daemon --help` - Show usage

**Logging:**
- Default: INFO level to stdout
- Use `RUST_LOG` environment variable for control
- Log connection events, state changes, errors

## Installation & Distribution

**Build:**
- Single static binary
- Optimize for size with `strip` and `upx` (optional)

**Package managers:**
- Provide install script for manual installation
- AUR package (PKGBUILD)
- Consider: .deb, .rpm, Flatpak later

**Installation locations:**
- Binary: `/usr/local/bin/ramble-daemon` or `~/.local/bin/ramble-daemon`
- Config: `~/.config/ramble/config.toml`

**Systemd service (optional):**
- User service for auto-start
- `~/.config/systemd/user/ramble-daemon.service`

## Testing Strategy

**Unit tests:**
- Config parsing
- Message deserialization
- State transitions

**Integration tests:**
- Mock SSE server
- Test reconnection logic
- Test notification suppression logic
- Mock HTTP requests from web app

**Manual testing:**
- Test with actual Ramble server
- Test with web app open/closed
- Test reconnection scenarios
- Test on different notification daemons (dunst, mako, etc.)

## Development Phases

### Phase 1: Core Functionality
1. Configuration management
2. SSE client for server connection
3. Basic notification display
4. Connect to server and receive message events

### Phase 2: Web App Integration
5. Local HTTP server
6. Activity tracking from web app POST requests
7. Smart notification suppression

### Phase 3: Polish
8. Reconnection logic with backoff
9. Comprehensive error handling
10. CLI improvements
11. Logging and debugging tools

### Phase 4: Distribution
12. Installation scripts
13. Documentation
14. Package for AUR
15. Systemd service file

## Code Structure

```
ramble-daemon/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── config.rs         # Configuration management
│   ├── sse.rs            # SSE client for server events
│   ├── http.rs           # Local HTTP server
│   ├── notify.rs         # Notification manager
│   ├── state.rs          # State management
│   └── protocol.rs       # Message types/event parsing
├── config.example.toml   # Example configuration
├── install.sh            # Installation script
└── README.md            # User documentation
```

## Security Considerations

**Local HTTP server:**
- Bind to `127.0.0.1` only (never `0.0.0.0`)
- No authentication needed (localhost only)
- Validate request paths and methods
- Simple CORS headers if needed for browser requests

**Server connection:**
- Use HTTPS for SSE connection
- Validate server certificates
- No sensitive data stored

**Configuration:**
- Config file readable only by user (`chmod 600`)
- No passwords/tokens stored (stateless design)

## Performance Targets

**Resource usage:**
- Memory: < 10MB typical
- CPU: Minimal when idle
- Network: Single SSE connection + occasional HTTP POST responses

**Responsiveness:**
- Notifications appear within 100ms of SSE event receipt
- Local HTTP server responds to web app within 10ms

## Future Enhancements (Out of Scope for MVP)

- Support for multiple rooms/channels
- Rich notifications with inline reply
- Message history cache (configurable)
- Sound effects for notifications
- Custom notification rules/filters
- MacOS support via Notification Center
- Windows support via Windows Notifications

## Success Criteria

The daemon is successful when:
1. ✅ Users receive notifications for new messages
2. ✅ No notifications when web app is active
3. ✅ No notifications for own messages
4. ✅ Reconnects automatically after network issues
5. ✅ Easy to install and configure
6. ✅ Minimal resource usage
7. ✅ Works with common Linux notification daemons

## Notes for Claude Code Agent

- Focus on correctness and simplicity over premature optimization
- Use idiomatic Rust patterns (Result/Option, error handling)
- Prefer standard library when possible, minimize dependencies
- Write clear error messages that help users debug issues
- Comment complex logic, especially state transitions
- Use structured logging for debugging
- Test thoroughly with different notification daemons
- Consider that many users will run this on tiling window managers (i3, sway, etc.)

## Why SSE + HTTP?

This architecture is simpler than WebSockets because:
- **SSE**: One-way push from server is all we need for receiving messages. Built-in reconnection.
- **HTTP**: Web app just needs to POST status updates occasionally, no persistent connection needed.
- **Less code**: No WebSocket handshaking, framing, or bidirectional message handling.
- **More robust**: HTTP is stateless, SSE handles reconnection automatically.

The daemon is essentially a notification bridge: SSE in from server, HTTP in from web app, D-Bus out to Linux notifications.
