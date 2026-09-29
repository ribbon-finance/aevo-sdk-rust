use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{SinkExt, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::client::AevoClient;
use crate::error::{AevoError, Result};
use crate::models::SignedOrder;
use crate::signing;

type InnerStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsChannel {
    Orderbook(String),
    Ticker(String),
    Trades(String),
    Orders,
    Fills,
    Positions,
    Raw(String),
}

impl WsChannel {
    pub fn as_string(&self) -> String {
        match self {
            Self::Orderbook(instrument) => format!("orderbook:{instrument}"),
            Self::Ticker(instrument) => format!("ticker:{instrument}"),
            Self::Trades(instrument) => format!("trades:{instrument}"),
            Self::Orders => "orders".to_string(),
            Self::Fills => "fills".to_string(),
            Self::Positions => "positions".to_string(),
            Self::Raw(value) => value.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsMessage {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub op: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug)]
pub struct AevoWebSocket {
    inner: InnerStream,
    next_id: u64,
}

impl AevoWebSocket {
    pub async fn connect(client: &AevoClient) -> Result<Self> {
        let (inner, _) = connect_async(client.ws_url())
            .await
            .map_err(|err| AevoError::WebSocket(format!("failed to connect: {err}")))?;
        Ok(Self { inner, next_id: 1 })
    }

    pub async fn auth(&mut self, key: &str, secret: &str) -> Result<()> {
        let id = self.next_request_id();
        self.send_value(auth_frame(id, key, secret)).await
    }

    pub async fn signed_auth(&mut self, key: &str, secret: &str) -> Result<()> {
        let timestamp = signing::current_unix_timestamp_ns().to_string();
        let signature = signing::websocket_hmac_signature(key, secret, &timestamp, "auth", "")?;
        let id = self.next_request_id();
        self.send_value(signed_auth_frame(id, key, &timestamp, &signature))
            .await
    }

    pub async fn subscribe(&mut self, channels: &[WsChannel]) -> Result<()> {
        self.send_value(subscribe_frame(
            None,
            channels
                .iter()
                .map(WsChannel::as_string)
                .collect::<Vec<String>>(),
        ))
        .await
    }

    pub async fn unsubscribe(&mut self, channels: &[WsChannel]) -> Result<()> {
        self.send_value(unsubscribe_frame(
            None,
            channels
                .iter()
                .map(WsChannel::as_string)
                .collect::<Vec<String>>(),
        ))
        .await
    }

    pub async fn publish_create_order(&mut self, order: &SignedOrder) -> Result<()> {
        let id = self.next_request_id();
        self.send_value(create_order_frame(id, order, None)?).await
    }

    pub async fn publish_edit_order(&mut self, order_id: &str, order: &SignedOrder) -> Result<()> {
        let id = self.next_request_id();
        self.send_value(edit_order_frame(id, order_id, order, None)?)
            .await
    }

    pub async fn publish_cancel_order(&mut self, order_id: &str) -> Result<()> {
        let id = self.next_request_id();
        self.send_value(cancel_order_frame(id, order_id)).await
    }

    pub async fn publish_cancel_all(&mut self) -> Result<()> {
        let id = self.next_request_id();
        self.send_value(json!({
            "id": id,
            "op": "cancel_all_orders",
            "data": {}
        }))
        .await
    }

    pub async fn ping(&mut self) -> Result<()> {
        let id = self.next_request_id();
        self.send_value(json!({
            "id": id,
            "op": "ping"
        }))
        .await
    }

    pub async fn close(mut self) -> Result<()> {
        self.inner
            .close(None)
            .await
            .map_err(|err| AevoError::WebSocket(format!("failed to close: {err}")))
    }

    async fn send_value(&mut self, value: Value) -> Result<()> {
        self.inner
            .send(Message::Text(value.to_string()))
            .await
            .map_err(|err| AevoError::WebSocket(format!("failed to send: {err}")))
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }
}

impl Stream for AevoWebSocket {
    type Item = Result<WsMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let Some(message) = futures_util::ready!(Pin::new(&mut self.inner).poll_next(cx))
            else {
                return Poll::Ready(None);
            };
            match message {
                Ok(Message::Text(text)) => {
                    return Poll::Ready(Some(parse_message(&text)));
                }
                Ok(Message::Binary(bytes)) => {
                    let text = String::from_utf8_lossy(&bytes);
                    return Poll::Ready(Some(parse_message(&text)));
                }
                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                Ok(Message::Close(_)) => return Poll::Ready(None),
                Ok(_) => continue,
                Err(err) => {
                    return Poll::Ready(Some(Err(AevoError::WebSocket(format!(
                        "read failed: {err}"
                    )))));
                }
            }
        }
    }
}

pub fn auth_frame(id: u64, key: &str, secret: &str) -> Value {
    json!({
        "id": id,
        "op": "auth",
        "data": {
            "key": key,
            "secret": secret,
        }
    })
}

pub fn signed_auth_frame(id: u64, key: &str, timestamp: &str, signature: &str) -> Value {
    json!({
        "id": id,
        "op": "auth",
        "data": {
            "key": key,
            "timestamp": timestamp,
            "signature": signature,
        }
    })
}

pub fn request_auth(key: &str, secret: &str, operation: &str, body: &str) -> Result<Value> {
    let timestamp = signing::current_unix_timestamp_ns().to_string();
    let signature = signing::websocket_hmac_signature(key, secret, &timestamp, operation, body)?;
    Ok(json!({
        "key": key,
        "timestamp": timestamp,
        "signature": signature,
    }))
}

pub fn subscribe_frame(id: Option<u64>, channels: Vec<String>) -> Value {
    let mut frame = json!({
        "op": "subscribe",
        "data": channels,
    });
    if let Some(id) = id {
        frame["id"] = json!(id);
    }
    frame
}

pub fn unsubscribe_frame(id: Option<u64>, channels: Vec<String>) -> Value {
    let mut frame = json!({
        "op": "unsubscribe",
        "data": channels,
    });
    if let Some(id) = id {
        frame["id"] = json!(id);
    }
    frame
}

pub fn create_order_frame(id: u64, order: &SignedOrder, auth: Option<Value>) -> Result<Value> {
    let data = serde_json::to_value(order)?;
    let mut frame = json!({
        "id": id,
        "op": "create_order",
        "data": data,
    });
    if let Some(auth) = auth {
        frame["auth"] = auth;
    }
    Ok(frame)
}

pub fn edit_order_frame(
    id: u64,
    order_id: &str,
    order: &SignedOrder,
    auth: Option<Value>,
) -> Result<Value> {
    let mut frame = create_order_frame(id, order, auth)?;
    frame["op"] = json!("edit_order");
    if let Some(data) = frame["data"].as_object_mut() {
        data.insert("order_id".to_string(), json!(order_id));
    }
    Ok(frame)
}

pub fn cancel_order_frame(id: u64, order_id: &str) -> Value {
    json!({
        "id": id,
        "op": "cancel_order",
        "data": { "order_id": order_id }
    })
}

pub fn parse_message(text: &str) -> Result<WsMessage> {
    Ok(serde_json::from_str(text)?)
}
