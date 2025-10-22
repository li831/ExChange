use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};

pub struct BinanceWebSocket {
    url: String,
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl BinanceWebSocket {
    /// Create a new WebSocket connection instance
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            stream: None,
        }
    }

    /// Connect to the Binance WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebSocket: {}", self.url);

        let (ws_stream, response) = connect_async(&self.url)
            .await
            .map_err(|e| anyhow!("Failed to connect: {}", e))?;

        debug!("WebSocket handshake completed: {:?}", response);
        self.stream = Some(ws_stream);

        info!("WebSocket connected successfully");
        Ok(())
    }

    /// Subscribe to multiple streams
    pub async fn subscribe(&mut self, streams: Vec<String>) -> Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow!("WebSocket not connected"))?;

        let subscribe_msg = json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": 1
        });

        let msg_str = serde_json::to_string(&subscribe_msg)?;
        info!("Subscribing to streams: {:?}", streams);

        stream
            .send(Message::Text(msg_str))
            .await
            .map_err(|e| anyhow!("Failed to send subscribe message: {}", e))?;

        debug!("Subscribe message sent");
        Ok(())
    }

    /// Receive the next message from WebSocket
    pub async fn next_message(&mut self) -> Result<String> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow!("WebSocket not connected"))?;

        loop {
            match stream.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => {
                        debug!("Received text message: {} bytes", text.len());
                        return Ok(text);
                    }
                    Message::Binary(data) => {
                        debug!("Received binary message: {} bytes", data.len());
                        let text = String::from_utf8(data)
                            .map_err(|e| anyhow!("Failed to decode binary message: {}", e))?;
                        return Ok(text);
                    }
                    Message::Ping(payload) => {
                        debug!("Received ping, sending pong");
                        stream.send(Message::Pong(payload)).await?;
                    }
                    Message::Pong(_) => {
                        debug!("Received pong");
                    }
                    Message::Close(frame) => {
                        warn!("WebSocket closed: {:?}", frame);
                        return Err(anyhow!("WebSocket connection closed"));
                    }
                    Message::Frame(_) => {
                        // Raw frames are not normally received
                        continue;
                    }
                },
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    return Err(anyhow!("WebSocket error: {}", e));
                }
                None => {
                    warn!("WebSocket stream ended");
                    return Err(anyhow!("WebSocket stream ended"));
                }
            }
        }
    }

    /// Unsubscribe from streams
    pub async fn unsubscribe(&mut self, streams: Vec<String>) -> Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow!("WebSocket not connected"))?;

        let unsubscribe_msg = json!({
            "method": "UNSUBSCRIBE",
            "params": streams,
            "id": 2
        });

        let msg_str = serde_json::to_string(&unsubscribe_msg)?;
        info!("Unsubscribing from streams: {:?}", streams);

        stream
            .send(Message::Text(msg_str))
            .await
            .map_err(|e| anyhow!("Failed to send unsubscribe message: {}", e))?;

        debug!("Unsubscribe message sent");
        Ok(())
    }

    /// Close the WebSocket connection gracefully
    pub async fn close(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            info!("Closing WebSocket connection");
            stream
                .close(None)
                .await
                .map_err(|e| anyhow!("Failed to close WebSocket: {}", e))?;
            self.stream = None;
            info!("WebSocket closed successfully");
        }
        Ok(())
    }
}

impl Drop for BinanceWebSocket {
    fn drop(&mut self) {
        if self.stream.is_some() {
            warn!("WebSocket dropped without explicit close");
        }
    }
}
