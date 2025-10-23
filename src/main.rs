use trading_engine::{config::AppConfig, engine::TradingEngine, logging};
use tracing::info;

#[tokio::main]
async fn main() {
    // 加载配置
    let config = AppConfig::load("config").unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
        eprintln!("Using default config");
        AppConfig::default_config()
    });

    // 初始化日志
    logging::init_tracing(&config.general.log_level);

    info!("🚀 Trading Engine Starting");
    info!("Environment: {}", config.general.environment);
    info!("Trading symbols: {:?}", config.trading.symbols);

    // 创建并启动交易引擎
    let mut engine = TradingEngine::new(config);

    if let Err(e) = engine.start().await {
        eprintln!("Engine error: {}", e);
        std::process::exit(1);
    }
}
