use std::collections::HashMap;
use thiserror::Error;
use tracing::{warn, info};

#[derive(Debug, Error)]
pub enum RiskError {
    #[error("Daily loss limit exceeded: {0:.2}%")]
    DailyLossExceeded(f64),

    #[error("Position limit exceeded for {symbol}: current={current:.2}, limit={limit:.2}")]
    PositionLimitExceeded {
        symbol: String,
        current: f64,
        limit: f64,
    },

    #[error("Total position ratio exceeded: {0:.2}% > {1:.2}%")]
    TotalPositionExceeded(f64, f64),

    #[error("Single trade loss would exceed limit: {0:.2}%")]
    SingleLossExceeded(f64),
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_single_loss: f64,    // 单笔最大亏损比例 (0.01 = 1%)
    pub max_daily_loss: f64,     // 日最大亏损比例 (0.03 = 3%)
    pub max_position_ratio: f64, // 最大仓位比例 (0.7 = 70%)
    pub max_position_per_symbol: f64, // 单币种最大仓位 (0.3 = 30%)
}

pub struct RiskManager {
    config: RiskConfig,
    initial_capital: f64,
    daily_pnl: f64,
    positions: HashMap<String, f64>, // symbol -> position value
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_capital: f64) -> Self {
        Self {
            config,
            initial_capital,
            daily_pnl: 0.0,
            positions: HashMap::new(),
        }
    }

    pub fn check_can_trade(&self, _symbol: &str) -> Result<(), RiskError> {
        // 检查日亏损限制
        let daily_loss_pct = self.daily_pnl / self.initial_capital;
        if daily_loss_pct < -self.config.max_daily_loss {
            warn!(
                "❌ Daily loss limit exceeded: {:.2}% < -{:.2}%",
                daily_loss_pct * 100.0,
                self.config.max_daily_loss * 100.0
            );
            return Err(RiskError::DailyLossExceeded(daily_loss_pct * 100.0));
        }

        // 检查总仓位比例
        let total_position: f64 = self.positions.values().sum();
        let position_ratio = total_position / self.initial_capital;
        if position_ratio > self.config.max_position_ratio {
            warn!(
                "❌ Total position ratio exceeded: {:.2}% > {:.2}%",
                position_ratio * 100.0,
                self.config.max_position_ratio * 100.0
            );
            return Err(RiskError::TotalPositionExceeded(
                position_ratio * 100.0,
                self.config.max_position_ratio * 100.0,
            ));
        }

        Ok(())
    }

    pub fn check_position_limit(&self, symbol: &str, additional_value: f64) -> Result<(), RiskError> {
        let current_position = self.positions.get(symbol).copied().unwrap_or(0.0);
        let new_position = current_position + additional_value;
        let position_ratio = new_position / self.initial_capital;

        if position_ratio > self.config.max_position_per_symbol {
            warn!(
                "❌ Position limit exceeded for {}: {:.2}% > {:.2}%",
                symbol,
                position_ratio * 100.0,
                self.config.max_position_per_symbol * 100.0
            );
            return Err(RiskError::PositionLimitExceeded {
                symbol: symbol.to_string(),
                current: position_ratio * 100.0,
                limit: self.config.max_position_per_symbol * 100.0,
            });
        }

        Ok(())
    }

    pub fn update_daily_pnl(&mut self, pnl_change: f64) {
        self.daily_pnl += pnl_change;
        info!("Daily PnL updated: {:.2} ({:.2}%)",
            self.daily_pnl,
            (self.daily_pnl / self.initial_capital) * 100.0
        );
    }

    pub fn update_position(&mut self, symbol: &str, position_value: f64) {
        self.positions.insert(symbol.to_string(), position_value);
        info!("Position updated: {} = {:.2}", symbol, position_value);
    }

    pub fn remove_position(&mut self, symbol: &str) {
        self.positions.remove(symbol);
        info!("Position removed: {}", symbol);
    }

    pub fn reset_daily_pnl(&mut self) {
        info!("Resetting daily PnL from {:.2} to 0.0", self.daily_pnl);
        self.daily_pnl = 0.0;
    }

    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    pub fn total_position_value(&self) -> f64 {
        self.positions.values().sum()
    }
}
