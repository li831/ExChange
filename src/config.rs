use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub binance: BinanceConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    pub environment: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub api_endpoint: String,
    pub ws_endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradingConfig {
    pub symbols: Vec<String>,
    pub capital_allocation: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskConfig {
    pub max_position_per_symbol: f64,
    pub max_single_loss: f64,
    pub max_daily_loss: f64,
    pub stop_loss_multiplier: f64,
}

impl AppConfig {
    pub fn load(config_file: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(config_file).required(false))
            .add_source(File::with_name("config.local").required(false))
            .add_source(config::Environment::with_prefix("TRADING"))
            .build()?;

        config.try_deserialize()
    }

    pub fn default_config() -> Self {
        Self {
            general: GeneralConfig {
                environment: "development".to_string(),
                log_level: "info".to_string(),
            },
            binance: BinanceConfig {
                api_endpoint: "https://testnet.binance.vision".to_string(),
                ws_endpoint: "wss://testnet.binance.vision/ws".to_string(),
            },
            trading: TradingConfig {
                symbols: vec!["BTCUSDT".to_string()],
                capital_allocation: 0.7,
            },
            risk: RiskConfig {
                max_position_per_symbol: 0.3,
                max_single_loss: 0.01,
                max_daily_loss: 0.03,
                stop_loss_multiplier: 1.5,
            },
        }
    }
}
