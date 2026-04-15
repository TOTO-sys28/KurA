//! DAVE / MLS handshake for Discord voice gateway v8 (E2EE-capable sessions).
//!
//! After `SelectProtocol`, the gateway exchanges MLS messages over JSON and binary WebSocket
//! frames; see Discord's voice documentation and the DAVE protocol whitepaper.
//!
//! Transport encryption keys still come from opcode 4 `SessionDescription`; this module drives the
//! MLS exchange until that payload is received (or returns the transport key early if parsed here).

use crate::{
    driver::{
        connection::error::{Error, Result},
        tasks::message::WsMessage,
    },
    model::Event as GatewayEvent,
    ws::{RawMessage, WsStream},
};
use flume::Sender;
use serde_json::json;
use tracing::{debug, info, warn};

/// Server-originated binary frame: big-endian sequence, opcode, then opcode-specific payload.
pub(crate) fn split_server_binary(data: &[u8]) -> Option<(u16, u8, &[u8])> {
    if data.len() < 3 {
        return None;
    }
    let seq = u16::from_be_bytes([data[0], data[1]]);
    let op = data[2];
    Some((seq, op, &data[3..]))
}

pub(crate) async fn send_client_binary(client: &mut WsStream, opcode: u8, body: &[u8]) -> Result<()> {
    let mut frame = Vec::with_capacity(1 + body.len());
    frame.push(opcode);
    frame.extend_from_slice(body);
    client.send_binary(&frame).await.map_err(Error::from)
}

pub(crate) async fn send_transition_ready(client: &mut WsStream, transition_id: u16) -> Result<()> {
    let msg = json!({
        "op": 23,
        "d": { "transition_id": transition_id }
    });
    client
        .send_json_text(&msg.to_string())
        .await
        .map_err(Error::from)
}

fn transport_key_from_session_description(v: &serde_json::Value) -> Option<Vec<u8>> {
    let key = v.get("d")?.get("secret_key")?.as_array()?;
    let mut out = Vec::with_capacity(key.len());
    for n in key {
        out.push(u8::try_from(n.as_u64()?).ok()?);
    }
    Some(out)
}

pub(crate) fn try_deliver_raw_json(tx: &Sender<WsMessage>, text: &str) {
    if let Ok(ev) = serde_json::from_str::<GatewayEvent>(text) {
        let _ = tx.send(WsMessage::Deliver(ev));
    }
}



#[cfg(feature = "dave")]
pub(crate) async fn perform_dave_handshake(
    client: &mut WsStream,
    info: &crate::info::ConnectionInfo,
    deliver: &Sender<WsMessage>,
) -> Result<(Option<Vec<u8>>, std::sync::Arc<std::sync::Mutex<davey::DaveSession>>)> {
    use core::num::NonZeroU16;
    use davey::{DAVE_PROTOCOL_VERSION, DaveSession, ProposalsOperationType};

    info!("DAVE: MLS handshake starting");

    let Some(channel) = info.channel_id else {
        return Err(Error::Dave(
            "voice channel id missing from ConnectionInfo (required for DAVE)",
        ));
    };

    let proto = NonZeroU16::new(DAVE_PROTOCOL_VERSION)
        .ok_or(Error::Dave("invalid DAVE_PROTOCOL_VERSION"))?;
    let user_id_u64 = info.user_id.0.get();
    let channel_id_u64 = channel.0.get();

    let mut session: DaveSession = DaveSession::new(proto, user_id_u64, channel_id_u64, None)
        .map_err(|e| {
            warn!("DAVE: DaveSession::new failed: {:?}", e);
            Error::Dave("DaveSession::new failed")
        })?;

    let mut cached_external_sender: Option<Vec<u8>> = None;
    let mut steps: u32 = 0;

    let transport_key = loop {
        steps = steps.saturating_add(1);
        if steps > 50_000 {
            return Err(Error::Dave("MLS handshake exceeded message limit"));
        }

        let incoming = match client.recv_raw().await? {
            Some(m) => m,
            None => continue,
        };

        match incoming {
            RawMessage::Text(text) => {
                let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
                    debug!("DAVE: non-json text frame: {e}");
                    Error::Json(e)
                })?;

                let op = match v.get("op").and_then(|x| x.as_u64()) {
                    Some(o) => o as u8,
                    None => {
                        try_deliver_raw_json(deliver, &text);
                        continue;
                    },
                };

                match op {
                    4 => {
                        if let Some(key) = transport_key_from_session_description(&v) {
                            info!("DAVE: received SessionDescription (transport key)");
                            break Some(key);
                        }
                        warn!("DAVE: opcode 4 missing secret_key");
                        return Err(Error::Dave("SessionDescription missing secret_key"));
                    },
                    21 | 22 => {
                        debug!("DAVE: JSON transition opcode {}", op);
                    },
                    24 => {
                        let epoch = v.pointer("/d/epoch").and_then(|x| x.as_u64()).unwrap_or(0);
                        if epoch == 1 {
                            session.reset().map_err(|e| {
                                warn!("DAVE: session reset on prepare_epoch: {:?}", e);
                                Error::Dave("MLS reset failed")
                            })?;
                            let Some(ext) = cached_external_sender.as_deref() else {
                                warn!("DAVE: prepare_epoch epoch=1 before external sender cache");
                                continue;
                            };
                            session.set_external_sender(ext).map_err(|e| {
                                warn!("DAVE: set_external_sender after reset: {:?}", e);
                                Error::Dave("set_external_sender after reset failed")
                            })?;
                            let kp = session.create_key_package().map_err(|e| {
                                warn!("DAVE: key package after epoch reset: {:?}", e);
                                Error::Dave("create_key_package after reset failed")
                            })?;
                            send_client_binary(client, 26, &kp).await?;
                            debug!("DAVE: sent new key package after prepare_epoch (epoch=1)");
                        }
                    },
                    _ => {
                        try_deliver_raw_json(deliver, &text);
                    },
                }
            },
            RawMessage::Binary(data) => {
                let Some((_seq, op, payload)) = split_server_binary(&data) else {
                    debug!("DAVE: short binary frame");
                    continue;
                };

                match op {
                    25 => {
                        cached_external_sender = Some(payload.to_vec());
                        session.set_external_sender(payload).map_err(|e| {
                            warn!("DAVE: set_external_sender failed: {:?}", e);
                            Error::Dave("set_external_sender failed")
                        })?;

                        let kp = session.create_key_package().map_err(|e| {
                            warn!("DAVE: create_key_package failed: {:?}", e);
                            Error::Dave("create_key_package failed after external sender")
                        })?;

                        send_client_binary(client, 26, &kp).await?;
                        debug!("DAVE: sent key package ({} bytes)", kp.len());
                    },
                    27 => {
                        if payload.is_empty() {
                            continue;
                        }
                        let op_type = match payload[0] {
                            0 => ProposalsOperationType::APPEND,
                            1 => ProposalsOperationType::REVOKE,
                            _ => {
                                warn!("DAVE: unknown proposals operation {}", payload[0]);
                                continue;
                            },
                        };
                        let body = &payload[1..];
                        match session.process_proposals(op_type, body, None) {
                            Ok(Some(cw)) => {
                                let mut frame = Vec::with_capacity(1 + cw.commit.len() + 8);
                                frame.push(28);
                                frame.extend_from_slice(&cw.commit);
                                if let Some(w) = cw.welcome {
                                    frame.extend_from_slice(&w);
                                }
                                client.send_binary(&frame).await.map_err(Error::from)?;
                                debug!("DAVE: sent commit/welcome");
                            },
                            Ok(None) => {},
                            Err(e) => {
                                warn!("DAVE: process_proposals failed: {:?}", e);
                            },
                        }
                    },
                    29 => {
                        if payload.len() < 2 {
                            continue;
                        }
                        let transition_id = u16::from_be_bytes([payload[0], payload[1]]);
                        let commit = &payload[2..];
                        if let Err(e) = session.process_commit(commit) {
                            warn!("DAVE: process_commit failed: {:?}", e);
                            continue;
                        }
                        send_transition_ready(client, transition_id).await?;
                        debug!("DAVE: processed announced commit; sent transition ready");
                    },
                    30 => {
                        if payload.len() < 2 {
                            continue;
                        }
                        let transition_id = u16::from_be_bytes([payload[0], payload[1]]);
                        let welcome = &payload[2..];
                        if let Err(e) = session.process_welcome(welcome) {
                            warn!("DAVE: process_welcome failed: {:?}", e);
                            continue;
                        }
                        send_transition_ready(client, transition_id).await?;
                        debug!("DAVE: processed welcome; sent transition ready");
                    },
                    _ => {
                        debug!("DAVE: unhandled binary opcode {}", op);
                    },
                }
            },
        }
    };

    Ok((transport_key, std::sync::Arc::new(std::sync::Mutex::new(session))))
}

#[cfg(not(feature = "dave"))]
pub(crate) async fn perform_dave_handshake(
    _client: &mut WsStream,
    _info: &crate::info::ConnectionInfo,
    _deliver: &Sender<WsMessage>,
) -> Result<Option<Vec<u8>>> {
    Ok(None)
}
