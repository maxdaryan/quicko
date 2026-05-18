//! QuickoKey — Unified 128-bit identity and key system.
//!
//! A single 128-bit key that serves as:
//! - **Identity** — unique user identifier (like a phone number)
//! - **Crypto seed** — deterministically derives X25519 keypairs
//! - **Address** — registered on the relay server for peer discovery
//!
//! Format: `QK-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX` (hex, grouped in 4s)
//!
//! The server stores a directory mapping QuickoKey → public key + online status,
//! acting as a "phone book". Messages are never stored — still fully peer-to-peer.

pub mod directory;
pub mod key;
pub mod seed;

pub use key::QuickoKey;
pub use seed::{generate_seed_phrase, seed_phrase_to_key};
