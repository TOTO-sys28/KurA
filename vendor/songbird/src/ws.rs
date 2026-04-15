use crate::{error::JsonError, model::Event};

use bytes::Bytes;
use futures::{SinkExt, StreamExt, TryStreamExt};
use tokio::{
    net::TcpStream,
    time::{timeout, Duration},
};
#[cfg(feature = "tungstenite")]
use tokio_tungstenite::{
    tungstenite::{
        error::Error as TungsteniteError,
        protocol::{CloseFrame, WebSocketConfig as Config},
        Message,
    },
    MaybeTlsStream,
    WebSocketStream,
};
#[cfg(feature = "tws")]
use tokio_websockets::{
    CloseCode,
    Error as TwsError,
    Limits,
    MaybeTlsStream,
    Message,
    WebSocketStream,
};
use tracing::{debug, instrument};
use url::Url;

pub struct WsStream {
    pub(crate) stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub(crate) seq: u64,
    pub(crate) last_inbound_seq: u64,
}

impl WsStream {
    #[instrument]
    pub(crate) async fn connect(url: Url) -> Result<Self> {
        #[cfg(feature = "tungstenite")]
        let (stream, _) = tokio_tungstenite::connect_async_with_config::<Url>(
            url,
            Some(
                Config::default()
                    .max_message_size(None)
                    .max_frame_size(None),
            ),
            true,
        )
        .await?;
        #[cfg(feature = "tws")]
        let (stream, _) = tokio_websockets::ClientBuilder::new()
            .limits(Limits::unlimited())
            .uri(url.as_str())
            .unwrap() // Any valid URL is a valid URI.
            .connect()
            .await?;

        Ok(Self { stream, seq: 0, last_inbound_seq: 0 })
    }

    pub(crate) async fn recv_json(&mut self) -> Result<Option<Event>> {
        const TIMEOUT: Duration = Duration::from_millis(500);

        let ws_message = match timeout(TIMEOUT, self.stream.next()).await {
            Ok(Some(Ok(v))) => Some(v),
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => None,
        };

        convert_ws_message(ws_message)
    }

    pub(crate) async fn recv_json_no_timeout(&mut self) -> Result<Option<Event>> {
        convert_ws_message(self.stream.try_next().await?)
    }

    pub(crate) async fn send_json(&mut self, value: &Event) -> Result<()> {
        let mut v = crate::json::to_value(value)?;
        if let Some(obj) = v.as_object_mut() {
            let op = obj.get("op").and_then(|x| x.as_u64());

            // In V8, Heartbeats (Opcode 3) must NOT have a top-level 'seq'.
            if op != Some(3) {
                self.seq += 1;
                obj.insert("seq".to_string(), crate::json::Value::Number(self.seq.into()));
            }

            // V8 heartbeat: transform `"d": <nonce>` into `"d": {"t": <nonce>, "seq_ack": N}`
            if op == Some(3) {
                let nonce = obj.get("d").cloned().unwrap_or(crate::json::Value::Null);
                let mut d_obj = serde_json::Map::new();
                d_obj.insert("t".to_string(), nonce);
                d_obj.insert("seq_ack".to_string(), crate::json::Value::Number(self.last_inbound_seq.into()));
                obj.insert("d".to_string(), crate::json::Value::Object(d_obj));
            }
        }
        let text = crate::json::to_string(&v)?;
        Ok(self.stream.send(Message::text(text)).await?)
    }

    /// Send a pre-serialized JSON text frame (used when the payload is not representable as
    /// [`Event`], e.g. DAVE `Identify` with `max_dave_protocol_version`).
    pub(crate) async fn send_json_text(&mut self, json: &str) -> Result<()> {
        let mut v: serde_json::Value = serde_json::from_str(json).map_err(JsonError::from)?;
        if let Some(obj) = v.as_object_mut() {
            let op = obj.get("op").and_then(|x| x.as_u64());

            if op != Some(3) {
                self.seq += 1;
                obj.insert("seq".to_string(), crate::json::Value::Number(self.seq.into()));
            }

            if op == Some(3) {
                let nonce = obj.get("d").cloned().unwrap_or(crate::json::Value::Null);
                let mut d_obj = serde_json::Map::new();
                d_obj.insert("t".to_string(), nonce);
                d_obj.insert("seq_ack".to_string(), crate::json::Value::Number(self.last_inbound_seq.into()));
                obj.insert("d".to_string(), crate::json::Value::Object(d_obj));
            }
        }
        let text = crate::json::to_string(&v).map_err(JsonError::from)?;
        Ok(self.stream.send(Message::text(text)).await?)
    }

    /// Send a binary WebSocket frame (DAVE MLS opcodes 26–28 use binary client payloads).
    pub(crate) async fn send_binary(&mut self, payload: &[u8]) -> Result<()> {
        Ok(self.stream.send(Message::binary(payload.to_vec())).await?)
    }

    /// Receive the next WebSocket message without the short recv timeout used by [`Self::recv_json`].
    pub(crate) async fn recv_raw(&mut self) -> Result<Option<RawMessage>> {
        #[cfg(feature = "tungstenite")]
        {
            match self.stream.next().await {
                Some(Ok(Message::Text(t))) => Ok(Some(RawMessage::Text(t.to_string()))),
                Some(Ok(Message::Binary(b))) => Ok(Some(RawMessage::Binary(b.to_vec()))),
                Some(Ok(Message::Close(frame))) => Err(Error::WsClosed(frame)),
                Some(Err(e)) => Err(e.into()),
                Some(Ok(_)) | None => Ok(None),
            }
        }
        #[cfg(all(feature = "tws", not(feature = "tungstenite")))]
        {
            match self.stream.next().await {
                Some(Ok(m)) if m.is_text() => Ok(Some(RawMessage::Text(
                    m.as_text().unwrap_or_default().to_string(),
                ))),
                Some(Ok(m)) if m.is_binary() => {
                    Ok(Some(RawMessage::Binary(m.into_payload().into())))
                },
                Some(Ok(m)) if m.is_close() => {
                    Err(Error::WsClosed(m.as_close().map(|(c, _)| c)))
                },
                Some(Err(e)) => Err(e.into()),
                Some(Ok(_)) | None => Ok(None),
            }
        }
    }
}

/// Payload from [`WsStream::recv_raw`].
pub(crate) enum RawMessage {
    Text(String),
    Binary(Vec<u8>),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Json(JsonError),

    /// The discord voice gateway does not support or offer zlib compression.
    /// As a result, only text messages are expected.
    UnexpectedBinaryMessage(Bytes),

    #[cfg(feature = "tungstenite")]
    Ws(TungsteniteError),
    #[cfg(feature = "tws")]
    Ws(TwsError),

    #[cfg(feature = "tungstenite")]
    WsClosed(Option<CloseFrame>),
    #[cfg(feature = "tws")]
    WsClosed(Option<CloseCode>),
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Error {
        Error::Json(e)
    }
}

#[cfg(feature = "tungstenite")]
impl From<TungsteniteError> for Error {
    fn from(e: TungsteniteError) -> Error {
        Error::Ws(e)
    }
}

#[cfg(feature = "tws")]
impl From<TwsError> for Error {
    fn from(e: TwsError) -> Self {
        Error::Ws(e)
    }
}

#[inline]
pub(crate) fn convert_ws_message(message: Option<Message>) -> Result<Option<Event>> {
    #[cfg(feature = "tungstenite")]
    let text = match message {
        Some(Message::Text(ref payload)) => payload,
        Some(Message::Binary(bytes)) => {
            debug!("Unexpected binary message (len {}), ignoring.", bytes.len());
            return Ok(None);
        },
        Some(Message::Close(Some(frame))) => {
            return Err(Error::WsClosed(Some(frame)));
        },
        // Ping/Pong message behaviour is internally handled by tungstenite.
        _ => return Ok(None),
    };
    #[cfg(feature = "tws")]
    let text = match message {
        Some(ref message) if message.is_text() =>
            if let Some(text) = message.as_text() {
                text
            } else {
                return Ok(None);
            },
        Some(message) if message.is_binary() => {
            debug!("Unexpected binary message, ignoring.");
            return Ok(None);
        },
        Some(message) if message.is_close() => {
            return Err(Error::WsClosed(message.as_close().map(|(c, _)| c)));
        },
        // ping/pong; will also be internally handled by tokio-websockets.
        _ => return Ok(None),
    };

    Ok(serde_json::from_str(text)
        .map_err(|e| {
            debug!("Unexpected JSON: {e}. Payload: {text}");
            e
        })
        .ok())
}
