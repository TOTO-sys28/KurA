use super::message::*;
use crate::{
    events::CoreContext,
    model::{
        payload::{Heartbeat, Speaking},
        CloseCode as VoiceCloseCode,
        Event as GatewayEvent,
        FromPrimitive,
        SpeakingState,
    },
    ws::{Error as WsError, WsStream},
    ConnectionInfo,
};
use flume::Receiver;
use rand::{distr::Uniform, Rng};
#[cfg(feature = "receive")]
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    select,
    time::{sleep_until, Instant},
};
#[cfg(feature = "tungstenite")]
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tracing::{debug, info, instrument, trace, warn};

pub(crate) struct AuxNetwork {
    rx: Receiver<WsMessage>,
    ws_client: WsStream,
    dont_send: bool,

    ssrc: u32,
    heartbeat_interval: Duration,

    speaking: SpeakingState,
    last_heartbeat_nonce: Option<u64>,

    attempt_idx: usize,
    info: ConnectionInfo,

    #[cfg(feature = "receive")]
    ssrc_signalling: Arc<SsrcTracker>,

    #[cfg(feature = "dave")]
    dave_session: Option<std::sync::Arc<std::sync::Mutex<davey::DaveSession>>>,
}

impl AuxNetwork {
    pub(crate) fn new(
        evt_rx: Receiver<WsMessage>,
        ws_client: WsStream,
        ssrc: u32,
        heartbeat_interval: f64,
        attempt_idx: usize,
        info: ConnectionInfo,
        #[cfg(feature = "receive")] ssrc_signalling: Arc<SsrcTracker>,
        #[cfg(feature = "dave")] dave_session: Option<std::sync::Arc<std::sync::Mutex<davey::DaveSession>>>,
    ) -> Self {
        Self {
            rx: evt_rx,
            ws_client,
            dont_send: false,

            ssrc,
            heartbeat_interval: Duration::from_secs_f64(heartbeat_interval / 1000.0),

            speaking: SpeakingState::empty(),
            last_heartbeat_nonce: None,

            attempt_idx,
            info,

            #[cfg(feature = "receive")]
            ssrc_signalling,

            #[cfg(feature = "dave")]
            dave_session,
        }
    }

    #[instrument(skip(self))]
    async fn run(&mut self, interconnect: &mut Interconnect) {
        let mut next_heartbeat = Instant::now() + self.heartbeat_interval;

        loop {
            let mut ws_error = false;
            let mut should_reconnect = false;
            let mut ws_reason = None;

            let hb = sleep_until(next_heartbeat);

            select! {
                biased;
                () = hb => {
                    info!("Sending heartbeat");
                    ws_error = match self.send_heartbeat().await {
                        Err(e) => {
                            should_reconnect = ws_error_is_not_final(&e);
                            ws_reason = Some((&e).into());
                            true
                        },
                        _ => false,
                    };
                    info!("Heartbeat sent ok");
                    next_heartbeat = self.next_heartbeat();
                }
                ws_msg = self.ws_client.recv_raw(), if !self.dont_send => {
                    ws_error = match ws_msg {
                        Err(e) => {
                            should_reconnect = ws_error_is_not_final(&e);
                            ws_reason = Some((&e).into());
                            true
                        },
                        Ok(Some(crate::ws::RawMessage::Text(text))) => {
                            let mut ws_val = serde_json::from_str::<serde_json::Value>(&text);
                            let mut processed = false;

                            if let Ok(v) = ws_val.as_mut() {
                                if let Some(s) = v.get("seq").and_then(|x| x.as_u64()) {
                                    self.ws_client.last_inbound_seq = s;
                                }

                                let op = v.get("op").and_then(|x| x.as_u64());

                                // Strip 'seq' so Songbird's internal model can parse the rest
                                if let Some(obj) = v.as_object_mut() {
                                    obj.remove("seq");
                                }

                                if let Ok(ev) = serde_json::from_value::<GatewayEvent>(v.clone()) {
                                    self.process_ws(interconnect, ev);
                                    processed = true;
                                } else if let Some(op) = op {
                                    match op {
                                        6 => {
                                            self.last_heartbeat_nonce.take();
                                            trace!("Heartbeat ACK received (manual)");
                                            processed = true;
                                        },
                                        #[cfg(feature = "dave")]
                                        24 => {
                                            let epoch = v.pointer("/d/epoch").and_then(|x| x.as_u64()).unwrap_or(0);
                                            if epoch == 1 {
                                                if let Some(sess) = &self.dave_session {
                                                    let kp = {
                                                        let mut l = sess.lock().unwrap();
                                                        let _ = l.reset().map_err(|e| warn!("DAVE: session reset on prepare_epoch: {:?}", e));
                                                        l.create_key_package()
                                                    };
                                                    if let Ok(kp) = kp {
                                                        if let Err(e) = crate::driver::dave::send_client_binary(&mut self.ws_client, 26, &kp).await {
                                                            tracing::warn!("Failed to send key package: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                            processed = true;
                                        },
                                        _ => {}
                                    }
                                }
                            }

                            if !processed {
                                tracing::debug!("Unexpected JSON or DAVE packet. Payload: {text}");
                            }
                            false
                        },
                        Ok(Some(crate::ws::RawMessage::Binary(data))) => {
                            #[cfg(feature = "dave")]
                            if let Some(session_mutex) = &self.dave_session {
                                if let Some((_seq, op, payload)) = crate::driver::dave::split_server_binary(&data) {
                                    match op {
                                        25 => {
                                            let kp = {
                                                let mut session = session_mutex.lock().unwrap();
                                                if let Err(e) = session.set_external_sender(payload) {
                                                    warn!("DAVE: set_external_sender failed: {:?}", e);
                                                    Err(())
                                                } else {
                                                    session.create_key_package().map_err(|_| ())
                                                }
                                            };
                                            if let Ok(kp) = kp {
                                                if let Err(e) = crate::driver::dave::send_client_binary(&mut self.ws_client, 26, &kp).await {
                                                    warn!("DAVE: failed to send key package: {:?}", e);
                                                }
                                            }
                                        },
                                        27 => {
                                            if !payload.is_empty() {
                                                let op_type = match payload[0] {
                                                    0 => davey::ProposalsOperationType::APPEND,
                                                    1 | _ => davey::ProposalsOperationType::REVOKE,
                                                };
                                                let cw = {
                                                    let mut session = session_mutex.lock().unwrap();
                                                    session.process_proposals(op_type, &payload[1..], None)
                                                };
                                                if let Ok(Some(cw)) = cw {
                                                    let mut frame = Vec::with_capacity(1 + cw.commit.len() + 8);
                                                    frame.push(28);
                                                    frame.extend_from_slice(&cw.commit);
                                                    if let Some(w) = cw.welcome {
                                                        frame.extend_from_slice(&w);
                                                    }
                                                    if let Err(e) = self.ws_client.send_binary(&frame).await {
                                                        warn!("DAVE: failed to send commit/welcome: {:?}", e);
                                                    }
                                                }
                                            }
                                        },
                                        29 => {
                                            if payload.len() >= 2 {
                                                let transition_id = u16::from_be_bytes([payload[0], payload[1]]);
                                                let ok = {
                                                    let mut session = session_mutex.lock().unwrap();
                                                    session.process_commit(&payload[2..]).is_ok()
                                                };
                                                if ok {
                                                    if let Err(e) = crate::driver::dave::send_transition_ready(&mut self.ws_client, transition_id).await {
                                                        warn!("DAVE: failed to send transition ready: {:?}", e);
                                                    }
                                                }
                                            }
                                        },
                                        30 => {
                                            if payload.len() >= 2 {
                                                let transition_id = u16::from_be_bytes([payload[0], payload[1]]);
                                                let ok = {
                                                    let mut session = session_mutex.lock().unwrap();
                                                    session.process_welcome(&payload[2..]).is_ok()
                                                };
                                                if ok {
                                                    if let Err(e) = crate::driver::dave::send_transition_ready(&mut self.ws_client, transition_id).await {
                                                        warn!("DAVE: failed to send transition ready: {:?}", e);
                                                    }
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            false
                        },
                        _ => false,
                    };
                }
                inner_msg = self.rx.recv_async() => {
                    match inner_msg {
                        Ok(WsMessage::Ws(data)) => {
                            self.ws_client = *data;
                            next_heartbeat = self.next_heartbeat();
                            self.dont_send = false;
                        },
                        Ok(WsMessage::ReplaceInterconnect(i)) => {
                            *interconnect = i;
                        },
                        Ok(WsMessage::SetKeepalive(keepalive)) => {
                            self.heartbeat_interval = Duration::from_secs_f64(keepalive / 1000.0);
                            next_heartbeat = self.next_heartbeat();
                        },
                        Ok(WsMessage::Speaking(is_speaking)) => {
                            if self.speaking.contains(SpeakingState::MICROPHONE) != is_speaking && !self.dont_send {
                                self.speaking.set(SpeakingState::MICROPHONE, is_speaking);
                                info!("Changing to {:?}", self.speaking);

                                let ssu_status = self.ws_client
                                    .send_json(&GatewayEvent::from(Speaking {
                                        delay: Some(0),
                                        speaking: self.speaking,
                                        ssrc: self.ssrc,
                                        user_id: None,
                                    }))
                                    .await;

                                ws_error |= match ssu_status {
                                    Err(e) => {
                                        should_reconnect = ws_error_is_not_final(&e);
                                        ws_reason = Some((&e).into());
                                        true
                                    },
                                    _ => false,
                                }
                            }
                        },
                        Ok(WsMessage::Deliver(msg)) => {
                            self.process_ws(interconnect, msg);
                        },
                        Ok(WsMessage::GetPrivacyCode(tx)) => {
                            let mut code = None;

                            #[cfg(feature = "dave")]
                            if let Some(sess) = &self.dave_session {
                                let l = sess.lock().unwrap();
                                code = l.voice_privacy_code().map(|c| c.to_string());
                            }

                            let _ = tx.send(code);
                        },
                        Err(flume::RecvError::Disconnected) => {
                            break;
                        },
                    }
                }
            }

            if ws_error {
                self.dont_send = true;

                if should_reconnect {
                    drop(interconnect.core.send(CoreMessage::Reconnect));
                } else {
                    drop(interconnect.core.send(CoreMessage::SignalWsClosure(
                        self.attempt_idx,
                        self.info.clone(),
                        ws_reason,
                    )));
                    break;
                }
            }
        }
    }

    fn next_heartbeat(&self) -> Instant {
        Instant::now() + self.heartbeat_interval
    }

    async fn send_heartbeat(&mut self) -> Result<(), WsError> {
        // Discord have suddenly, mysteriously, started rejecting
        // ints-as-strings. Keep JS happy here, I suppose...
        const JS_MAX_INT: u64 = (1u64 << 53) - 1;
        let nonce_range =
            Uniform::new(0, JS_MAX_INT).expect("uniform range is finite and nonempty");
        let nonce = rand::rng().sample(nonce_range);
        self.last_heartbeat_nonce = Some(nonce);

        trace!("Sent heartbeat {:?}", self.speaking);

        if !self.dont_send {
            self.ws_client
                .send_json(&GatewayEvent::from(Heartbeat { nonce }))
                .await?;
        }

        Ok(())
    }

    fn process_ws(&mut self, interconnect: &Interconnect, value: GatewayEvent) {
        match value {
            GatewayEvent::Speaking(ev) => {
                #[cfg(feature = "receive")]
                if let Some(user_id) = &ev.user_id {
                    self.ssrc_signalling.user_ssrc_map.insert(*user_id, ev.ssrc);
                }

                drop(interconnect.events.send(EventMessage::FireCoreEvent(
                    CoreContext::SpeakingStateUpdate(ev),
                )));
            },
            GatewayEvent::ClientConnect(ev) => {
                debug!("Received discontinued ClientConnect: {:?}", ev);
            },
            GatewayEvent::ClientDisconnect(ev) => {
                #[cfg(feature = "receive")]
                {
                    self.ssrc_signalling.disconnected_users.insert(ev.user_id);
                }

                drop(interconnect.events.send(EventMessage::FireCoreEvent(
                    CoreContext::ClientDisconnect(ev),
                )));
            },
            GatewayEvent::HeartbeatAck(ev) => {
                if let Some(nonce) = self.last_heartbeat_nonce.take() {
                    if ev.nonce == nonce {
                        trace!("Heartbeat ACK received.");
                    } else {
                        warn!(
                            "Heartbeat nonce mismatch! Expected {}, saw {}.",
                            nonce, ev.nonce
                        );
                    }
                }
            },
            other => {
                trace!("Received other websocket data: {:?}", other);
            },
        }
    }
}

#[instrument(skip(interconnect, aux))]
pub(crate) async fn runner(mut interconnect: Interconnect, mut aux: AuxNetwork) {
    trace!("WS thread started.");
    aux.run(&mut interconnect).await;
    trace!("WS thread finished.");
}

fn ws_error_is_not_final(err: &WsError) -> bool {
    match err {
        #[cfg(feature = "tungstenite")]
        WsError::WsClosed(Some(frame)) => match frame.code {
            CloseCode::Library(l) =>
                if let Some(code) = VoiceCloseCode::from_u16(l) {
                    code.should_resume()
                } else {
                    true
                },
            _ => true,
        },
        #[cfg(feature = "tws")]
        WsError::WsClosed(Some(code)) => match (*code).into() {
            code @ 4000..=4999_u16 =>
                if let Some(code) = VoiceCloseCode::from_u16(code) {
                    code.should_resume()
                } else {
                    true
                },
            _ => true,
        },
        e => {
            debug!("Error sending/receiving ws {:?}.", e);
            true
        },
    }
}
