use trading_engine::binance::BinanceClient;
use trading_engine::config::AppConfig;

#[tokio::test]
async fn test_binance_client_creation() {
    let config = AppConfig::load("config").unwrap();
    let client = BinanceClient::new(
        &config.binance.api_endpoint,
        config.binance.api_key.as_deref(),
        config.binance.secret_key.as_deref(),
    );

    assert_eq!(client.endpoint(), &config.binance.api_endpoint);
}

#[tokio::test]
async fn test_get_server_time() {
    let config = AppConfig::load("config").unwrap();
    let client = BinanceClient::new(
        &config.binance.api_endpoint,
        config.binance.api_key.as_deref(),
        config.binance.secret_key.as_deref(),
    );

    let result = client.get_server_time().await;
    assert!(result.is_ok(), "Should successfully get server time");

    let server_time = result.unwrap();
    assert!(server_time > 0, "Server time should be positive");
}

#[tokio::test]
async fn test_get_exchange_info() {
    let config = AppConfig::load("config").unwrap();
    let client = BinanceClient::new(
        &config.binance.api_endpoint,
        config.binance.api_key.as_deref(),
        config.binance.secret_key.as_deref(),
    );

    let result = client.get_exchange_info().await;
    assert!(result.is_ok(), "Should successfully get exchange info");
}

#[tokio::test]
async fn test_get_account_info() {
    let config = AppConfig::load("config").unwrap();

    // This test requires API keys
    if config.binance.api_key.is_none() || config.binance.secret_key.is_none() {
        println!("Skipping test_get_account_info: API keys not configured");
        return;
    }

    let client = BinanceClient::new(
        &config.binance.api_endpoint,
        config.binance.api_key.as_deref(),
        config.binance.secret_key.as_deref(),
    );

    let result = client.get_account().await;
    assert!(result.is_ok(), "Should successfully get account info with valid keys");
}

#[tokio::test]
async fn test_get_symbol_price() {
    let config = AppConfig::load("config").unwrap();
    let client = BinanceClient::new(
        &config.binance.api_endpoint,
        config.binance.api_key.as_deref(),
        config.binance.secret_key.as_deref(),
    );

    let result = client.get_symbol_price("BTCUSDT").await;
    assert!(result.is_ok(), "Should successfully get BTCUSDT price");

    let price = result.unwrap();
    assert!(price > 0.0, "Price should be positive");
}
