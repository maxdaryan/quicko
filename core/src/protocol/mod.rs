//! Wire protocol for Quicko2 messaging.

pub mod codec;
pub mod frame;

pub use codec::FrameCodec;
pub use frame::{Frame, MessageType};
