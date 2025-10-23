use trading_engine::monitoring::telegram::{TelegramAlerter, AlertLevel};

#[tokio::test]
#[ignore] // 需要真实的bot token
async fn test_send_telegram_message() {
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap();
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap();

    let alerter = TelegramAlerter::new(bot_token, chat_id);

    let result = alerter.send_alert(
        AlertLevel::Info,
        "Test message from trading engine".to_string()
    ).await;

    assert!(result.is_ok());
}
