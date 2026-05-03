//! Message relay — routes encrypted messages between clients.
//!
//! The relay is zero-knowledge: it never decrypts message content,
//! only reads the frame header to determine routing.

use std::sync::Arc;
use tokio::sync::mpsc;

use quicko2_core::protocol::frame::{MessageType, payloads};
use quicko2_core::protocol::codec::FrameCodec;
use quicko2_core::error::QuickoError;

use crate::AppState;

/// Handle an incoming binary message from a client.
pub async fn handle_message(
    data: &[u8],
    client_tx: &mpsc::Sender<Vec<u8>>,
    state: &Arc<AppState>,
    session_id: &mut Option<String>,
) -> Result<(), QuickoError> {
    let (frame, _) = FrameCodec::decode(data)?;

    match frame.msg_type {
        MessageType::Hello => {
            let hello: payloads::HelloPayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::info!(
                "Client hello: {} ({})",
                hello.display_name,
                &hello.session_id[..8]
            );

            // Register the client
            *session_id = Some(hello.session_id.clone());
            state
                .registry
                .register(hello.session_id, hello.display_name, client_tx.clone());

            // Send HelloAck
            let ack = payloads::HelloAckPayload {
                accepted: true,
                server_time: chrono::Utc::now().timestamp(),
            };
            let ack_payload = FrameCodec::serialize_payload(&ack)?;
            let ack_frame = quicko2_core::protocol::frame::Frame::new(
                MessageType::HelloAck,
                ack_payload,
            )?;
            let _ = client_tx.send(FrameCodec::encode(&ack_frame)).await;
        }

        MessageType::Join => {
            let join: payloads::JoinPayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::info!(
                "Client {} joining session {}",
                &join.session_id[..8],
                join.invite_code
            );

            state
                .registry
                .join_session(&join.session_id, &join.invite_code);

            // Notify existing peers
            let peer_event = payloads::PeerEventPayload {
                session_id: join.session_id.clone(),
                display_name: join.display_name,
                public_key: Some(join.public_key),
            };
            let event_payload = FrameCodec::serialize_payload(&peer_event)?;
            let event_frame = quicko2_core::protocol::frame::Frame::new(
                MessageType::PeerJoined,
                event_payload,
            )?;
            state
                .registry
                .broadcast(
                    &join.session_id,
                    &join.invite_code,
                    FrameCodec::encode(&event_frame),
                )
                .await;
        }

        MessageType::Message => {
            // Relay encrypted message to recipient — we never decrypt
            let msg: payloads::MessagePayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::debug!(
                "Relaying message {} → {}",
                &msg.sender_id[..8],
                &msg.recipient_id[..8]
            );

            // Forward the original frame data as-is
            state.registry.send_to(&msg.recipient_id, data.to_vec()).await;
        }

        MessageType::KeyExchange => {
            // Relay key exchange to peers in the same session
            let kex: payloads::KeyExchangePayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            // Broadcast the key exchange to all peers via the sender's session
            if let Some(ref sid) = session_id {
                // Use the sender_id from the payload to find their invite code
                state
                    .registry
                    .broadcast_from(sid, data.to_vec())
                    .await;
            }
            let _ = kex; // suppress unused warning
        }

        MessageType::Ping => {
            // Respond with pong
            let pong = quicko2_core::protocol::frame::Frame::pong();
            let _ = client_tx.send(FrameCodec::encode(&pong)).await;
        }

        MessageType::Leave => {
            if let Some(ref sid) = session_id {
                tracing::info!("Client {} leaving", &sid[..8.min(sid.len())]);
                state.registry.unregister(sid);
                *session_id = None;
            }
        }

        _ => {
            tracing::debug!("Unhandled message type: {:?}", frame.msg_type);
        }
    }

    Ok(())
}
