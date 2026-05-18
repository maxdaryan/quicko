# Quicko2

> Lightweight, high-performance ephemeral messaging for macOS & Android.  
> **5,497 lines of code** across Rust, Python, and TOML.

---

## Overview

Quicko2 is a zero-persistence, end-to-end encrypted messaging platform built with a Rust core and native UI layers. Messages exist only in memory — when a session ends, everything is gone. The relay server is zero-knowledge: it routes encrypted blobs without ever seeing plaintext.

### Key Principles

- **Ephemeral** — No data persists to disk. Ever.
- **Encrypted** — E2E encryption using X25519 + HKDF-SHA256 + AES-256-GCM.
- **Fast** — Rust core, async networking, binary protocol (MessagePack).
- **Zero-Knowledge Relay** — Server never decrypts, never stores.
- **Unified Identity** — 128-bit QuickoKey with seed phrase recovery.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Quicko2 Platform                          │
├────────────────┬───────────────────┬──────────────┬─────────────────┤
│   ui-macos     │    core-ffi       │    core      │    server       │
│   (PyQt6)      │    (PyO3)         │   (Rust)     │   (axum)        │
│                │                   │              │                 │
│  main_window   │  QuickoClient     │  Client      │  Relay          │
│  sidebar       │  PySessionInfo    │  Crypto      │  Registry       │
│  chat_input    │  PyQuickoKeyInfo  │  Protocol    │  KeyDirectory   │
│  message_bubble│  PyMessage        │  Network     │  RateLimiter    │
│  session_panel │                   │  Messaging   │  Listener       │
│  theme         │                   │  QuickoKey   │  Config         │
│                │                   │  Store       │                 │
└────────────────┴───────────────────┴──────────────┴─────────────────┘
```

| Layer | Language | Lines | Purpose |
|-------|----------|------:|---------|
| **core** | Rust | 3,121 | Encryption, protocol, networking, identity, storage |
| **server** | Rust | 756 | WebSocket relay, session registry, key directory |
| **core-ffi** | Rust (PyO3) | 214 | Python bindings for macOS UI |
| **ui-macos** | Python (PyQt6) | 1,017 | Native macOS desktop interface |
| **utils** | Python | 83 | Keygen and benchmarking tools |

---

## Project Structure

```
quicko2/
├── core/                       # Shared Rust core library (3,121 LOC)
│   └── src/
│       ├── lib.rs              # Client entry point, ClientConfig
│       ├── error.rs            # Unified error types (QuickoError)
│       ├── crypto/
│       │   ├── keys.rs         # X25519 key pair generation & DH
│       │   ├── encrypt.rs      # AES-256-GCM encryption with AAD
│       │   ├── kdf.rs          # HKDF-SHA256 key derivation
│       │   └── session_keys.rs # Per-session key management
│       ├── protocol/
│       │   ├── frame.rs        # Wire frame format & message types
│       │   └── codec.rs        # MessagePack encode/decode
│       ├── network/
│       │   ├── connection.rs   # WebSocket with exponential backoff
│       │   ├── heartbeat.rs    # Ping/pong health monitoring
│       │   └── transport.rs    # Transport event types
│       ├── session/
│       │   ├── identity.rs     # Ephemeral identity generation
│       │   ├── lifecycle.rs    # Session state machine
│       │   └── peer.rs         # Peer management
│       ├── messaging/
│       │   ├── message.rs      # Message data structure
│       │   ├── queue.rs        # Outbound message buffering
│       │   └── delivery.rs     # Delivery status tracking
│       ├── quickokey/
│       │   ├── key.rs          # 128-bit unified identity key
│       │   ├── seed.rs         # 16-word seed phrase generation/recovery
│       │   └── directory.rs    # Client-side directory operation types
│       └── store/
│           └── ephemeral.rs    # Ring buffer message store with TTL
│
├── core-ffi/                   # PyO3 bridge (214 LOC)
│   └── src/lib.rs              # QuickoClient, PySessionInfo, PyQuickoKeyInfo
│
├── server/                     # Relay server (756 LOC)
│   └── src/
│       ├── main.rs             # Server entry point (tokio + axum)
│       ├── relay.rs            # Message routing & directory ops
│       ├── registry.rs         # Session registry (DashMap)
│       ├── directory.rs        # Key directory (QuickoKey → public key)
│       ├── listener.rs         # WebSocket upgrade & routing
│       ├── rate_limit.rs       # Token bucket rate limiter
│       └── config.rs           # Server configuration
│
├── ui-macos/                   # macOS UI (1,017 LOC)
│   └── src/quicko_ui/
│       ├── app.py              # Application entry point
│       ├── main_window.py      # Main window (stacked views)
│       ├── bridge.py           # Rust ↔ Python bridge calls
│       ├── theme.py            # Dark theme color system
│       └── widgets/
│           ├── sidebar.py      # Chat list with search
│           ├── session_panel.py# Create/join session panel
│           ├── chat_input.py   # Message input with send button
│           ├── message_bubble.py# Sent/received message bubbles
│           └── status_bar.py   # Connection status indicator
│
├── utils/                      # Utilities (83 LOC)
│   ├── keygen.py               # QuickoKey generation script
│   └── bench.py                # Performance benchmarks
│
├── Cargo.toml                  # Workspace configuration
├── run_mac.sh                  # One-command macOS launcher
└── README.md
```

---

## Core Modules

### Cryptography (`core/src/crypto/`)

**End-to-end encryption pipeline:**

```
X25519 Key Exchange → Raw Shared Secret
         ↓
HKDF-SHA256 Key Derivation (domain-separated)
         ↓
     ┌───┴───┐
  msg_key  auth_key
     ↓
AES-256-GCM Encrypt (with AAD: sender || recipient || timestamp)
     ↓
nonce (12B) || ciphertext || tag (16B)
```

- **`keys.rs`** — X25519 ECDH key pair generation, Diffie-Hellman shared secret computation, and deterministic keypair derivation from QuickoKey.
- **`kdf.rs`** — HKDF-SHA256 key derivation with domain-separated contexts (`quicko2-msg-v1`, `quicko2-auth-v1`, `quicko2-qk-x25519-v1`). Derives message encryption keys, auth keys, and X25519 secrets from QuickoKey.
- **`encrypt.rs`** — AES-256-GCM authenticated encryption with AAD. AAD binds `sender_id:recipient_id:timestamp` to prevent replay attacks. Random 96-bit nonce per message.
- **`session_keys.rs`** — Per-session key management and rotation.

### Wire Protocol (`core/src/protocol/`)

**Binary frame format:**

```
┌─────────┬──────────┬─────────┬──────────────────────┐
│ Version │  Type    │ Length  │       Payload        │
│ 1 byte  │ 1 byte  │ 4 bytes │  Variable (msgpack)  │
└─────────┴──────────┴─────────┴──────────────────────┘
```

**18 message types** covering the full protocol:

| Code | Type | Direction | Purpose |
|------|------|-----------|---------|
| `0x01` | Hello | Client → Server | Initial handshake |
| `0x02` | HelloAck | Server → Client | Handshake acknowledgment |
| `0x10` | Message | Client ↔ Client | Encrypted chat message |
| `0x11` | Ack | Bidirectional | Delivery acknowledgment |
| `0x20` | Ping | Client → Server | Heartbeat ping |
| `0x21` | Pong | Server → Client | Heartbeat pong |
| `0x30` | Join | Client → Server | Join session by invite code |
| `0x31` | Leave | Client → Server | Leave session |
| `0x32` | PeerJoined | Server → Client | Peer joined notification |
| `0x33` | PeerLeft | Server → Client | Peer left notification |
| `0x40` | KeyExchange | Client ↔ Client | X25519 key exchange |
| `0x50` | RegisterKey | Client → Server | Register QuickoKey on directory |
| `0x51` | LookupKey | Client → Server | Look up a QuickoKey |
| `0x52` | LookupResponse | Server → Client | Lookup result |
| `0x53` | UnregisterKey | Client → Server | Remove QuickoKey from directory |
| `0x54` | CallPeer | Client → Server | Initiate connection to a QuickoKey |
| `0x55` | CallResponse | Client → Client | Accept/reject a call |
| `0xFF` | Error | Server → Client | Error notification |

- Max payload: 1 MB
- Serialization: MessagePack (via `rmp-serde`)

### QuickoKey Identity System (`core/src/quickokey/`)

**128-bit unified identity key** (2¹²⁸ ≈ 3.4×10³⁸ possible keys):

```
Format: QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890
         ↑   └──── 8 groups of 4 hex chars (128 bits) ────┘
      prefix
```

- **`key.rs`** — QuickoKey generation, parsing, formatting. Derives deterministic X25519 secrets, session salts, and human-readable display names (`"Swift Falcon #A1B2"`). Constant-time equality comparison to prevent timing attacks.
- **`seed.rs`** — 16-word seed phrase for key backup/recovery. Uses a 256-word list where each word encodes exactly 8 bits. Lossless roundtrip: `generate → words → recover`.
- **`directory.rs`** — Client-side payload types for directory operations (register, lookup, unregister, call, call response).

### Networking (`core/src/network/`)

- **`connection.rs`** — WebSocket connection manager with a robust background supervisor loop and exponential backoff reconnection (100ms → 30s cap, 2× multiplier). Transparently manages connection state and provides persistent channels to the application.
- **`heartbeat.rs`** — Ping/pong health monitoring with configurable interval (15s default) and timeout (5s). Measures round-trip latency.
- **`transport.rs`** — Transport event types (Connected, Disconnected, MessageReceived, PingReceived, Reconnecting, ReconnectionFailed).

### Session Management (`core/src/session/`)

- **`identity.rs`** — Ephemeral session identity: 128-bit random session ID, human-readable display name (`"Adjective Animal #XXXX"`), 8-character Base32 invite code (`"KBQW-E3TU"`), X25519 keypair, TTL with expiry tracking. Supports both random generation and deterministic derivation from QuickoKey.
- **`lifecycle.rs`** — Session state machine management.
- **`peer.rs`** — Peer tracking within sessions.

### Messaging (`core/src/messaging/`)

- **`message.rs`** — Message data structure with ID, sender, recipient, content, timestamp.
- **`queue.rs`** — Outbound message queue (ring buffer, 500 max) for offline buffering. Drops oldest messages when full.
- **`delivery.rs`** — Delivery status tracking: `Pending → Sent → Delivered → Read` (or `Failed`).

### Ephemeral Store (`core/src/store/`)

- **`ephemeral.rs`** — In-memory message store using a ring buffer with TTL eviction. Bounded capacity (default 1000 messages). Supports recent messages, peer-filtered queries, and ID lookups. All data lost on process exit — **by design**.

---

## Server

The relay server is a zero-knowledge message router built with **axum** and **tokio**.

### Components

- **`relay.rs`** — Core message handler. Routes all 18 message types: handshakes, encrypted messages, key exchanges, directory operations, and heartbeats. Never decrypts message content.
- **`registry.rs`** — Session registry using `DashMap` for lock-free concurrent access. Manages client connections, session membership, broadcast, and unicast routing.
- **`directory.rs`** — In-memory QuickoKey directory ("phone book"). Maps `QuickoKey → public key + display name + online status`. Enables peer discovery by QuickoKey and call initiation.
- **`rate_limit.rs`** — Per-session token bucket rate limiter (30 msgs/sec default, burst capacity).
- **`listener.rs`** — HTTP/WebSocket listener with axum routing and upgrade handling.
- **`config.rs`** — Server configuration (bind address, max connections, buffer TTL, rate limits, max payload size).

### Server Defaults

| Setting | Default |
|---------|---------|
| Bind address | `0.0.0.0:9900` |
| Max connections | 10,000 |
| Buffer TTL | 60 seconds |
| Rate limit | 30 msgs/sec |
| Max payload | 1 MB |

---

## FFI Bridge (core-ffi)

PyO3 bindings exposing the Rust core to Python:

| Python Class | Purpose |
|-------------|---------|
| `QuickoClient` | Main client handle — manage connection, create sessions, manage keys |
| `PyTransportEvent`| Networking events (Connected, MessageReceived, Reconnecting, etc) |
| `PySessionInfo` | Session identity data (session_id, display_name, invite_code) |
| `PyQuickoKeyInfo` | QuickoKey data (formatted_key, display_name, seed_phrase) |
| `PyMessage` | Message data (id, sender, recipient, content, timestamp, status) |

**Key FFI methods:**
- `connect()` — Start the background connection supervisor
- `disconnect()` — Stop the connection and clear state
- `poll_event()` — Poll for the next networking event (non-blocking)
- `send_raw(data)` — Send a binary message to the relay
- `create_session()` — Create a new ephemeral session
- `destroy_session()` — Zeroize all session data
- `generate_quickokey()` — Generate a new QuickoKey with seed phrase
- `recover_quickokey(seed_phrase)` — Recover key from 16-word seed phrase

---

## macOS UI

Native desktop interface built with **PyQt6** and a custom dark theme.

### Views

1. **Welcome View** — Landing page with Quicko2 branding
2. **Session Panel** — Create new session or join by invite code
3. **Chat View** — Active messaging with header, message list, and input

### Widgets

- **Sidebar** — Scrollable chat list with search, session selection, and new chat button
- **MessageBubble** — Sent/received message styling with timestamps and asymmetric border radius
- **ChatInput** — Multi-line text input with send button (Enter to send, Shift+Enter for newline)
- **StatusBar** — Connection status indicator (connected/disconnected/connecting)

### Theme

Dark theme with curated color palette:
- Background: layered surfaces (`#1a1a2e` → `#16213e` → `#0f3460`)
- Accent: `#e94560` (action buttons, highlights)
- Messages: distinct sent/received bubble colors

---

## Security Model

```
┌─────────┐                    ┌─────────────┐                    ┌─────────┐
│  Alice   │                   │   Relay      │                    │   Bob   │
│          │   X25519 DH       │   Server     │    X25519 DH      │         │
│  keypair ├──────────────────►│              │◄───────────────────┤ keypair │
│          │                   │  zero-know-  │                    │         │
│ HKDF →   │   AES-256-GCM    │  ledge only  │   AES-256-GCM     │ HKDF →  │
│ msg_key  ├──────────────────►│  routes blobs│───────────────────►│ msg_key │
│          │   (nonce+ct+tag)  │              │   (nonce+ct+tag)  │         │
└─────────┘                    └─────────────┘                    └─────────┘
```

| Property | Implementation |
|----------|---------------|
| Key Exchange | X25519 ECDH (Curve25519) |
| Key Derivation | HKDF-SHA256 with domain-separated contexts |
| Encryption | AES-256-GCM with 96-bit random nonce |
| Authentication | AEAD with AAD (sender + recipient + timestamp) |
| Anti-Replay | Timestamp binding in AAD |
| Anti-Tamper | GCM 128-bit authentication tag |
| Identity | 128-bit QuickoKey with constant-time comparison |
| Recovery | 16-word deterministic seed phrase (256-word list) |
| Ephemeral | All data in-memory only, zeroized on session destroy |
| Zero-Knowledge | Relay server never sees plaintext or keys |
| Rate Limiting | Per-session token bucket algorithm |

---

## Building & Running

### Prerequisites

- **Rust** 1.70+ with `cargo`
- **Python** 3.10+ with `PyQt6`
- **maturin** (for building Python bindings)

### Build

```bash
# Build all Rust crates
cargo build --release

# Run tests
cargo test

# Start relay server
cargo run -p quicko2-server

# Build Python bindings (requires maturin)
cd core-ffi && maturin develop --release
```

### Quick Start (macOS)

```bash
# One-command launcher: builds core, starts server, launches UI
./run_mac.sh
```

### Run Components Individually

```bash
# Terminal 1: Start relay server
cargo run -p quicko2-server
# Server binds to 0.0.0.0:9900

# Terminal 2: Start macOS UI
export PYTHONPATH=ui-macos/src
python3 -m quicko_ui.app
```

---

## Dependencies

### Rust Crates

| Category | Crate | Version | Purpose |
|----------|-------|---------|---------|
| Crypto | `x25519-dalek` | 2.0 | ECDH key exchange |
| Crypto | `aes-gcm` | 0.10 | AES-256-GCM encryption |
| Crypto | `hkdf` | 0.12 | Key derivation |
| Crypto | `sha2` | 0.10 | SHA-256 hash |
| Crypto | `rand` | 0.8 | Cryptographic RNG (OsRng) |
| Serialization | `serde` | 1.0 | Serialization framework |
| Serialization | `rmp-serde` | 1.3 | MessagePack codec |
| Async | `tokio` | 1.0 | Async runtime |
| Async | `tokio-tungstenite` | 0.24 | WebSocket client |
| Async | `futures-util` | 0.3 | Stream/Sink utilities |
| Server | `axum` | 0.7 | HTTP/WebSocket server |
| Server | `tower` | 0.5 | Middleware |
| Server | `tower-http` | 0.6 | CORS + tracing middleware |
| Utility | `thiserror` | 2.0 | Error derive macros |
| Utility | `tracing` | 0.1 | Structured logging |
| Utility | `dashmap` | 6.0 | Lock-free concurrent map |
| Utility | `uuid` | 1.0 | UUID generation |
| Utility | `base32` | 0.5 | Invite code encoding |
| Utility | `chrono` | 0.4 | Timestamps |
| FFI | `pyo3` | 0.22 | Python bindings |
| FFI | `uniffi` | 0.28 | Cross-platform FFI (Android) |

### Python

| Package | Purpose |
|---------|---------|
| `PyQt6` | macOS native UI framework |

---

## License

MIT
