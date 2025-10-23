use crate::{
    binance::BinanceClient,
    config::AppConfig,
    monitoring::telegram::{AlertLevel, TelegramAlerter},
    orderbook::OrderBook,
    risk::manager::{RiskConfig, RiskManager},
    strategy::{dual_ma::DualMAStrategy, Signal, Strategy},
    websocket::BinanceWebSocket,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

pub struct TradingEngine {
    config: AppConfig,
    #[allow(dead_code)]
    binance_client: BinanceClient,
    websocket: BinanceWebSocket,
    #[allow(dead_code)]
    orderbooks: Arc<RwLock<HashMap<String, OrderBook>>>,
    strategy: Box<dyn Strategy + Send + Sync>,
    risk_manager: Arc<RwLock<RiskManager>>,
    alerter: TelegramAlerter,
    price_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl TradingEngine {
    pub fn new(config: AppConfig) -> Self {
        let binance_client = BinanceClient::new(
            &config.binance.api_endpoint,
            config.binance.api_key.as_deref(),
            config.binance.secret_key.as_deref(),
        );

        let websocket = BinanceWebSocket::new(&config.binance.ws_endpoint);

        let risk_config = RiskConfig {
            max_single_loss: config.risk.max_single_loss,
            max_daily_loss: config.risk.max_daily_loss,
            max_position_ratio: config.trading.capital_allocation,
            max_position_per_symbol: config.risk.max_position_per_symbol,
        };

        // TODO: 从配置读取初始资金
        let risk_manager = RiskManager::new(risk_config, 10000.0);

        let alerter = TelegramAlerter::new(
            config.monitoring.telegram_bot_token.clone(),
            config.monitoring.telegram_chat_id.clone(),
        );

        // 使用双均线策略 (5, 20)
        let strategy = Box::new(DualMAStrategy::new(5, 20));

        Self {
            config,
            binance_client,
            websocket,
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            risk_manager: Arc::new(RwLock::new(risk_manager)),
            alerter,
            price_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting trading engine...");

        // 发送启动通知
        self.alerter
            .send_alert(
                AlertLevel::Info,
                "Trading engine started successfully".to_string(),
            )
            .await
            .ok();

        // 连接WebSocket
        self.websocket.connect().await?;

        // 订阅交易对的成交流
        let streams: Vec<String> = self
            .config
            .trading
            .symbols
            .iter()
            .map(|s| format!("{}@trade", s.to_lowercase()))
            .collect();

        self.websocket.subscribe(streams).await?;

        info!("📡 Subscribed to market data streams");

        // 主循环
        self.run_main_loop().await?;

        Ok(())
    }

    async fn run_main_loop(&mut self) -> Result<()> {
        let mut strategy_check_interval =
            tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                // 接收WebSocket消息
                msg_result = self.websocket.next_message() => {
                    if let Ok(msg) = msg_result {
                        self.process_market_data(&msg).await;
                    }
                }

                // 定期检查策略信号
                _ = strategy_check_interval.tick() => {
                    self.check_strategy_signals().await;
                }
            }
        }
    }

    async fn process_market_data(&self, message: &str) {
        // 解析成交数据
        if let Ok(trade) = serde_json::from_str::<serde_json::Value>(message) {
            if let Some(symbol) = trade["s"].as_str() {
                if let Some(price) = trade["p"].as_str() {
                    if let Ok(price_f64) = price.parse::<f64>() {
                        debug!("{} - Price: {}", symbol, price_f64);

                        // 更新价格历史
                        let mut history = self.price_history.write().await;
                        let prices = history.entry(symbol.to_string()).or_insert_with(Vec::new);

                        prices.push(price_f64);

                        // 保留最近100个价格
                        if prices.len() > 100 {
                            prices.remove(0);
                        }
                    }
                }
            }
        }
    }

    async fn check_strategy_signals(&self) {
        for symbol in &self.config.trading.symbols {
            let history = self.price_history.read().await;

            if let Some(prices) = history.get(symbol) {
                if prices.len() < 20 {
                    debug!(
                        "{}: Insufficient price data ({})",
                        symbol,
                        prices.len()
                    );
                    continue;
                }

                let signal = self.strategy.generate_signal(prices);

                match signal {
                    Signal::Long => {
                        info!("📊 {} - LONG signal generated", symbol);
                        self.execute_trade(symbol, "BUY").await;
                    }
                    Signal::Short => {
                        info!("📊 {} - SHORT signal generated", symbol);
                        self.execute_trade(symbol, "SELL").await;
                    }
                    Signal::None => {
                        // 无操作
                    }
                    _ => {}
                }
            }
        }
    }

    async fn execute_trade(&self, symbol: &str, side: &str) {
        // 风控检查
        let risk_check = self.risk_manager.read().await.check_can_trade(symbol);

        if let Err(e) = risk_check {
            warn!("❌ Trade rejected by risk manager: {}", e);
            self.alerter
                .send_risk_alert(format!("Trade rejected: {}", e))
                .await;
            return;
        }

        info!(
            "✅ Risk check passed, executing {} order for {}",
            side, symbol
        );

        // TODO: 实际下单逻辑
        // let order = NewOrderRequest { ... };
        // self.binance_client.new_order(order).await

        // 发送交易通知
        self.alerter
            .send_trade_alert(symbol, side, 50000.0, 0.001)
            .await;
    }
}
