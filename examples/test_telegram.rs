use trading_engine::{config::AppConfig, monitoring::telegram::*};

#[tokio::main]
async fn main() {
    let config = AppConfig::load("config").expect("Failed to load config");

    let alerter = TelegramAlerter::new(
        config.monitoring.telegram_bot_token,
        config.monitoring.telegram_chat_id,
    );

    println!("Sending test alert...");
    alerter.send_alert(
        AlertLevel::Info,
        "Trading engine test alert".to_string()
    ).await.unwrap();

    println!("Sending trade alert...");
    alerter.send_trade_alert("BTCUSDT", "BUY", 50000.0, 0.001).await;

    println!("Done! Check your Telegram");
}
