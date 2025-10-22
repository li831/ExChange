use trading_engine::config::AppConfig;

#[test]
fn test_load_config_from_file() {
    let config = AppConfig::load("config").unwrap();
    assert_eq!(config.general.environment, "development");
}

#[test]
fn test_config_has_binance_section() {
    let config = AppConfig::load("config").unwrap();
    assert!(!config.binance.api_endpoint.is_empty());
}

#[test]
fn test_config_has_risk_limits() {
    let config = AppConfig::load("config").unwrap();
    assert!(config.risk.max_single_loss > 0.0);
    assert!(config.risk.max_single_loss < 1.0);
}
