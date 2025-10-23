use trading_engine::risk::manager::{RiskManager, RiskConfig, RiskError};

#[test]
fn test_risk_manager_daily_loss_exceeded() {
    let config = RiskConfig {
        max_single_loss: 0.01,
        max_daily_loss: 0.03,
        max_position_ratio: 0.7,
        max_position_per_symbol: 0.3,
    };

    let mut manager = RiskManager::new(config, 10000.0);
    manager.update_daily_pnl(-350.0); // -3.5%

    let result = manager.check_can_trade("BTCUSDT");
    assert!(matches!(result, Err(RiskError::DailyLossExceeded(_))));
}

#[test]
fn test_risk_manager_position_limit() {
    let config = RiskConfig {
        max_single_loss: 0.01,
        max_daily_loss: 0.03,
        max_position_ratio: 0.7,
        max_position_per_symbol: 0.3,
    };

    let mut manager = RiskManager::new(config, 10000.0);

    // 添加持仓到30%限制
    manager.update_position("BTCUSDT", 3000.0);

    // 尝试再增加持仓应该被拒绝
    let result = manager.check_position_limit("BTCUSDT", 100.0);
    assert!(matches!(result, Err(RiskError::PositionLimitExceeded { .. })));
}

#[test]
fn test_risk_manager_allows_valid_trade() {
    let config = RiskConfig {
        max_single_loss: 0.01,
        max_daily_loss: 0.03,
        max_position_ratio: 0.7,
        max_position_per_symbol: 0.3,
    };

    let manager = RiskManager::new(config, 10000.0);

    let result = manager.check_can_trade("BTCUSDT");
    assert!(result.is_ok());
}
