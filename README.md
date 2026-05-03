# Quicko2

> Lightweight, high-performance ephemeral messaging for macOS & Android.

## Architecture

- **Core**: Rust — messaging engine, E2E encryption, protocol, networking
- **macOS UI**: Python (PyQt6) via PyO3 FFI bridge
- **Android UI**: Kotlin (Jetpack Compose) via UniFFI bridge
- **Server**: Rust relay server (axum + WebSocket)

## Building

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

## Project Structure

```
quicko2/
├── core/          # Shared Rust core library
├── core-ffi/      # PyO3 + UniFFI bridge
├── server/        # Relay server
├── ui-macos/      # Python macOS UI (PyQt6)
├── ui-android/    # Kotlin Android UI (future)
└── utils/         # Python utilities
```

## Security

- X25519 ECDH key exchange
- HKDF-SHA256 key derivation
- AES-256-GCM authenticated encryption with AAD
- Ephemeral sessions — no data persists after session ends
- Zero-knowledge relay — server never sees plaintext
