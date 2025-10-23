use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Copy)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl AlertLevel {
    fn emoji(&self) -> &str {
        match self {
            AlertLevel::Info => "ℹ️",
            AlertLevel::Warning => "⚠️",
            AlertLevel::Critical => "🚨",
        }
    }
}

pub struct TelegramAlerter {
    bot_token: String,
    chat_id: String,
    client: Client,
    enabled: bool,
}

impl TelegramAlerter {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        let enabled = !bot_token.is_empty() && !chat_id.is_empty();

        if !enabled {
            warn!("Telegram alerter disabled: missing bot token or chat ID");
        }

        Self {
            bot_token,
            chat_id,
            client: Client::new(),
            enabled,
        }
    }

    pub async fn send_alert(&self, level: AlertLevel, message: String) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let formatted_message = format!(
            "{} *[{:?}]* {}\n\n_时间: {}_",
            level.emoji(),
            level,
            message,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        info!("Sending Telegram alert: {:?} - {}", level, message);

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let response = self.client
            .post(&url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": formatted_message,
                "parse_mode": "Markdown"
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Telegram API error: {}", error_text);
            anyhow::bail!("Failed to send Telegram message: {}", error_text);
        }

        Ok(())
    }

    pub async fn send_trade_alert(&self, symbol: &str, side: &str, price: f64, quantity: f64) {
        let message = format!(
            "📊 *交易执行*\n\n\
            交易对: `{}`\n\
            方向: *{}*\n\
            价格: `{:.2}`\n\
            数量: `{:.6}`\n\
            金额: `{:.2}`",
            symbol, side, price, quantity, price * quantity
        );

        self.send_alert(AlertLevel::Info, message).await.ok();
    }

    pub async fn send_risk_alert(&self, message: String) {
        self.send_alert(AlertLevel::Warning, message).await.ok();
    }

    pub async fn send_error_alert(&self, error: String) {
        self.send_alert(AlertLevel::Critical, error).await.ok();
    }

    pub async fn send_daily_summary(&self, pnl: f64, trades_count: usize) {
        let emoji = if pnl > 0.0 { "📈" } else { "📉" };
        let message = format!(
            "{} *每日总结*\n\n\
            盈亏: `{:.2}` ({:.2}%)\n\
            交易次数: `{}`\n\
            日期: {}",
            emoji,
            pnl,
            pnl, // TODO: 计算百分比
            trades_count,
            chrono::Utc::now().format("%Y-%m-%d")
        );

        self.send_alert(AlertLevel::Info, message).await.ok();
    }
}
