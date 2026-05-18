//! Message relay — routes encrypted messages between clients.
//!
//! The relay is zero-knowledge: it never decrypts message content,
//! only reads the frame header to determine routing.
//! Also handles the QuickoKey directory (register, lookup, call).

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
                state.directory.set_offline_by_session(sid);
                *session_id = None;
            }
        }

        // --- QuickoKey Directory Operations ---

        MessageType::RegisterKey => {
            let reg: payloads::RegisterKeyPayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::info!("RegisterKey: {}", &reg.quicko_key);

            let sid = session_id.clone().unwrap_or_default();
            state.directory.register(
                reg.quicko_key.clone(),
                reg.public_key,
                reg.display_name,
                sid,
            );

            // Send ack
            let ack = payloads::RegisterKeyAckPayload {
                accepted: true,
                error: None,
            };
            let ack_payload = FrameCodec::serialize_payload(&ack)?;
            let ack_frame = quicko2_core::protocol::frame::Frame::new(
                MessageType::RegisterKey,
                ack_payload,
            )?;
            let _ = client_tx.send(FrameCodec::encode(&ack_frame)).await;
        }

        MessageType::LookupKey => {
            let lookup: payloads::LookupKeyPayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::debug!("LookupKey: {}", &lookup.quicko_key);

            let response = if let Some(entry) = state.directory.lookup(&lookup.quicko_key) {
                payloads::LookupResponsePayload {
                    quicko_key: lookup.quicko_key,
                    found: true,
                    public_key: Some(entry.public_key),
                    display_name: Some(entry.display_name),
                    is_online: entry.is_online,
                }
            } else {
                payloads::LookupResponsePayload {
                    quicko_key: lookup.quicko_key,
                    found: false,
                    public_key: None,
                    display_name: None,
                    is_online: false,
                }
            };

            let resp_payload = FrameCodec::serialize_payload(&response)?;
            let resp_frame = quicko2_core::protocol::frame::Frame::new(
                MessageType::LookupResponse,
                resp_payload,
            )?;
            let _ = client_tx.send(FrameCodec::encode(&resp_frame)).await;
        }

        MessageType::UnregisterKey => {
            let unreg: payloads::UnregisterKeyPayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::info!("UnregisterKey: {}", &unreg.quicko_key);
            state.directory.unregister(&unreg.quicko_key);
        }

        MessageType::CallPeer => {
            let call: payloads::CallPeerPayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::info!("CallPeer: {} → {}", &call.caller_key, &call.callee_key);

            // Look up the callee's session and forward the call
            if let Some(callee_sid) = state.directory.get_session_id(&call.callee_key) {
                state.registry.send_to(&callee_sid, data.to_vec()).await;
            } else {
                // Callee not online — send error back to caller
                let err = payloads::CallResponsePayload {
                    caller_key: call.caller_key,
                    callee_key: call.callee_key,
                    accepted: false,
                    responder_public_key: None,
                    responder_display_name: None,
                };
                let err_payload = FrameCodec::serialize_payload(&err)?;
                let err_frame = quicko2_core::protocol::frame::Frame::new(
                    MessageType::CallResponse,
                    err_payload,
                )?;
                let _ = client_tx.send(FrameCodec::encode(&err_frame)).await;
            }
        }

        MessageType::CallResponse => {
            let resp: payloads::CallResponsePayload =
                FrameCodec::deserialize_payload(&frame.payload)?;

            tracing::info!(
                "CallResponse: {} → {} (accepted: {})",
                &resp.callee_key, &resp.caller_key, resp.accepted
            );

            // Forward the response to the caller
            if let Some(caller_sid) = state.directory.get_session_id(&resp.caller_key) {
                state.registry.send_to(&caller_sid, data.to_vec()).await;
            }
        }

        _ => {
            tracing::debug!("Unhandled message type: {:?}", frame.msg_type);
        }
    }

    Ok(())
}

