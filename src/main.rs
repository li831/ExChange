use trading_engine::{config::AppConfig, logging};
use tracing::info;

#[tokio::main]
async fn main() {
    // 加载配置
    let config = AppConfig::load("config")
        .unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}", e);
            AppConfig::default_config()
        });

    // 初始化日志
    logging::init_tracing(&config.general.log_level);

    info!("Trading engine starting...");
    info!("Environment: {}", config.general.environment);
    info!("Binance API: {}", config.binance.api_endpoint);
    info!("Trading symbols: {:?}", config.trading.symbols);

    // TODO: 启动交易引擎

    info!("Trading engine initialized successfully");
}
