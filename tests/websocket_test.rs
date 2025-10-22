use trading_engine::websocket::BinanceWebSocket;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_websocket_can_connect() {
    let mut ws = BinanceWebSocket::new("wss://stream.binance.com:9443/ws");
    let result = timeout(Duration::from_secs(5), ws.connect()).await;
    assert!(result.is_ok(), "WebSocket should connect within timeout");
}

#[tokio::test]
async fn test_websocket_can_subscribe() {
    let mut ws = BinanceWebSocket::new("wss://stream.binance.com:9443/ws");
    ws.connect().await.unwrap();

    let result = ws.subscribe(vec!["btcusdt@depth20@100ms".to_string()]).await;
    assert!(result.is_ok(), "Should successfully subscribe to stream");
}

#[tokio::test]
async fn test_websocket_receives_messages() {
    let mut ws = BinanceWebSocket::new("wss://stream.binance.com:9443/ws");
    ws.connect().await.unwrap();
    ws.subscribe(vec!["btcusdt@depth20@100ms".to_string()]).await.unwrap();

    let result = timeout(Duration::from_secs(10), ws.next_message()).await;
    assert!(result.is_ok(), "Should receive at least one message");
}
