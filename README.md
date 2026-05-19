# Quicko2

> Lightweight, high-performance ephemeral messaging for macOS & Android.  
> **8,169 lines of code** across Rust, Python, Kotlin, and TOML.

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
┌──────────────────────────────────────────────────────────────────────────────────┐
│                                 Quicko2 Platform                                 │
├────────────────┬───────────────────┬──────────────┬───────────────┬──────────────┤
│   ui-macos     │   ui-android      │   core-ffi   │     core      │    server    │
│   (PyQt6)      │   (Compose)       │   (UniFFI)   │    (Rust)     │    (axum)    │
│                │                   │              │               │              │
│  main_window   │  MainActivity     │ QuickoClient │  Client       │  Relay       │
│  sidebar       │  QuickoViewModel  │ SessionInfo  │  Crypto       │  Registry    │
│  chat_input    │  QuickoApp        │ KeyInfo      │  Protocol     │  KeyDirectory│
│  message_bubble│                   │ Message      │  Network      │  RateLimiter │
│  session_panel │                   │              │  Messaging    │  Listener    │
│  theme         │                   │              │  QuickoKey    │  Config      │
│                │                   │              │  Store        │              │
└────────────────┴───────────────────┴──────────────┴───────────────┴──────────────┘
```

| Layer | Language | Purpose |
|-------|----------|---------|
| **core** | Rust | Encryption, protocol, networking, identity, storage |
| **server** | Rust | WebSocket relay, session registry, key directory |
| **core-ffi** | Rust | PyO3 (macOS) and UniFFI (Android) bindings |
| **ui-macos** | Python | Native macOS desktop interface (PyQt6) |
| **ui-android**| Kotlin | Native Android application (Jetpack Compose) |

---

## Project Structure

```
quicko2/
├── core/                       # Shared Rust core library
├── core-ffi/                   # FFI bridge (PyO3 & UniFFI)
│   ├── src/uniffi_bridge.rs    # Android UniFFI exports
│   └── uniffi.toml             # Kotlin binding configuration
├── server/                     # Relay server (axum + tokio)
├── ui-macos/                   # macOS UI (PyQt6)
├── ui-android/                 # Android App (Jetpack Compose)
│   └── app/src/main/java/
│       └── dev/quicko/
│           ├── app/            # UI, ViewModel, MainActivity
│           └── core/           # Generated Rust bindings
├── scripts/
│   └── build_android.sh        # Android build & binding generation script
├── DEVELOPMENT.md              # Progress, architecture notes, and task list
├── Cargo.toml                  # Workspace configuration
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

- **`keys.rs`** — X25519 ECDH key pair generation and deterministic derivation.
- **`kdf.rs`** — HKDF-SHA256 key derivation with domain-separated contexts.
- **`encrypt.rs`** — AES-256-GCM authenticated encryption with AAD.
- **`session_keys.rs`** — Per-session key management and rotation.

---

## Android UI (ui-android)

Modern Android application built with **Jetpack Compose** and **MVVM**.

- **`QuickoViewModel`** — Reactive bridge between the Rust core and Compose UI using Kotlin `StateFlow`.
- **`MainActivity`** — Single-activity entry point with dynamic theme support.
- **UniFFI Bridge** — Direct native calls to the Rust core with zero overhead.

---

## macOS UI (ui-macos)

Native desktop interface built with **PyQt6** and a custom dark theme.

- **`main_window.py`** — Stacked views for navigation.
- **`bridge.py`** — Rust ↔ Python bridge calls using PyO3.
- **`theme.py`** — Curated dark mode color system.

---

## Building & Running

### Prerequisites

- **Rust** 1.70+
- **Python** 3.10+ (for macOS UI)
- **Android SDK & NDK** (for Android build)
- **cargo-ndk** (for cross-compilation)

### Quick Start (macOS)

```bash
./run_mac.sh
```

### Quick Start (Android)

```bash
# Generate bindings and build for Android
./scripts/build_android.sh
```

---

## License

MIT License - Copyright (c) 2026 maxdaryan
