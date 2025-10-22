# 币安高频量化交易系统 - 技术设计文档

## 1. 项目概述

### 1.1 项目目标
构建一个轻量级、高性能的加密货币量化交易系统，专注于币安交易所的现货市场，实现持续自动化交易获利。

### 1.2 核心特性
- **低延迟交易：** 目标订单延迟 10-50ms
- **风险可控：** 单笔亏损<1%，日亏损<3%
- **策略灵活：** 基于Pine Script DSL的策略系统
- **24/7运行：** 自动化监控和告警
- **可扩展：** 从中频到高频的平滑升级路径

---

## 2. 系统架构

### 2.1 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                     Strategy Development                     │
│  ┌────────────┐    ┌──────────────┐    ┌────────────────┐  │
│  │ Freqtrade  │ -> │ Pine Script  │ -> │ Strategy DSL   │  │
│  │  (Python)  │    │  Converter   │    │  (JSON/TOML)   │  │
│  └────────────┘    └──────────────┘    └────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Rust Core Trading Engine (Tokyo)                │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Market Data Module                                     │ │
│  │  - WebSocket订阅 (币安现货)                              │ │
│  │  - L2订单簿维护                                          │ │
│  │  - K线数据聚合                                           │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Strategy Engine                                        │ │
│  │  - DSL解析器 (Pine Script子集)                          │ │
│  │  - 技术指标计算 (SMA, RSI, MACD等)                       │ │
│  │  - 信号生成                                              │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Risk Manager                                           │ │
│  │  - 仓位控制 (70%资金投入)                                │ │
│  │  - 止损/止盈                                             │ │
│  │  - 单笔亏损<1%, 日亏损<3%                                │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Order Executor                                         │ │
│  │  - 币安API集成                                           │ │
│  │  - 订单路由优化                                          │ │
│  │  - 延迟监控                                              │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  State Manager                                          │ │
│  │  - 持仓状态                                              │ │
│  │  - 订单状态                                              │ │
│  │  - 余额追踪                                              │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Monitoring & Storage                      │
│  ┌──────────────┐   ┌──────────────┐   ┌────────────────┐  │
│  │  ElastiCache │->│  Timestream   │   │  Telegram Bot  │  │
│  │    Redis     │   │  (历史数据)   │   │    (告警)      │  │
│  └──────────────┘   └──────────────┘   └────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Web Dashboard (Axum + Svelte)                          ││
│  │  - 实时PnL曲线                                           ││
│  │  - 持仓/订单监控                                         ││
│  │  - 系统性能指标                                          ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块详解

#### 2.2.1 Market Data Module
**职责：** 实时市场数据获取和处理

**实现细节：**
- 使用`tokio-tungstenite`建立WebSocket连接到币安
- 订阅频道：
  - `<symbol>@trade` - 实时成交
  - `<symbol>@depth@100ms` - 订单簿更新
  - `<symbol>@kline_1m` - 1分钟K线
- 维护本地订单簿副本（减少API调用）
- 数据结构：
  ```rust
  struct MarketData {
      orderbook: Arc<RwLock<OrderBook>>,
      trades: Arc<RwLock<Vec<Trade>>>,
      klines: Arc<RwLock<HashMap<String, Vec<Kline>>>>,
  }
  ```

**性能要求：**
- WebSocket消息处理延迟 < 1ms
- 订单簿更新频率：100ms

#### 2.2.2 Strategy Engine
**职责：** 策略执行和信号生成

**Pine Script DSL支持的核心功能：**
```pine
// 示例策略DSL
//@version=5
strategy("Mean Reversion", overlay=true)

// 参数
length = input(20, "MA Length")
std_dev = input(2.0, "Std Dev")

// 指标计算
basis = ta.sma(close, length)
upper = basis + std_dev * ta.stdev(close, length)
lower = basis - std_dev * ta.stdev(close, length)

// 交易逻辑
if close < lower
    strategy.entry("Long", strategy.long)
if close > upper
    strategy.close("Long")
```

**实现架构：**
```rust
// DSL解析器
pub struct PineScriptParser {
    ast: StrategyAST,
}

// 策略执行器
pub struct StrategyExecutor {
    indicators: IndicatorRegistry,
    state: StrategyState,
}

// 技术指标库
pub mod indicators {
    pub fn sma(data: &[f64], period: usize) -> f64;
    pub fn rsi(data: &[f64], period: usize) -> f64;
    pub fn bollinger_bands(data: &[f64], period: usize, std_dev: f64)
        -> (f64, f64, f64);
}
```

#### 2.2.3 Risk Manager
**职责：** 实时风险控制

**风控规则：**
1. **仓位控制：** 总资金70%投入
2. **单笔止损：** 每笔交易最大亏损1%本金
3. **日亏损限制：** 当日累计亏损达3%停止交易
4. **交易对分散：** 单交易对不超过总仓位30%

**实现：**
```rust
pub struct RiskManager {
    config: RiskConfig,
    daily_pnl: f64,
    positions: HashMap<String, Position>,
}

impl RiskManager {
    pub fn check_order(&self, order: &Order) -> Result<(), RiskError> {
        // 检查日亏损限制
        if self.daily_pnl < -self.config.max_daily_loss {
            return Err(RiskError::DailyLossExceeded);
        }

        // 检查仓位限制
        let position_value = self.calculate_position_value(order);
        if position_value > self.config.max_position_ratio {
            return Err(RiskError::PositionLimitExceeded);
        }

        Ok(())
    }
}
```

#### 2.2.4 Order Executor
**职责：** 订单执行和延迟优化

**关键优化：**
1. **连接池管理：** 预建立HTTP/2连接
2. **订单批处理：** 合并同时段订单
3. **智能重试：** 指数退避 + 抖动
4. **延迟监控：** 记录每个订单的端到端延迟

**实现：**
```rust
pub struct OrderExecutor {
    client: BinanceClient,
    latency_tracker: LatencyTracker,
}

impl OrderExecutor {
    pub async fn place_order(&self, order: Order)
        -> Result<OrderResponse, ExecutionError> {
        let start = Instant::now();

        // 执行订单
        let response = self.client
            .new_order(order)
            .timeout(Duration::from_millis(100))
            .await?;

        // 记录延迟
        let latency = start.elapsed();
        self.latency_tracker.record(latency);

        Ok(response)
    }
}
```

---

## 3. 技术栈详解

### 3.1 Rust核心依赖

```toml
[dependencies]
# 异步运行时
tokio = { version = "1.35", features = ["full"] }
tokio-tungstenite = "0.21" # WebSocket

# HTTP客户端
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# 币安API
binance = "1.0"

# 数据库
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Web框架
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "fs"] }

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 时间处理
chrono = "0.4"

# 配置管理
config = "0.14"

# 性能监控
prometheus = "0.13"
```

### 3.2 Pine Script DSL解析

**实现方案：**
- 使用`pest`或`nom`进行词法/语法分析
- 生成AST（抽象语法树）
- 转换为可执行的Rust代码或解释执行

**支持的核心语法：**
```pine
// 变量声明
length = input(20, "Period")
price = close

// 技术指标
ma = ta.sma(close, length)
rsi_val = ta.rsi(close, 14)

// 条件判断
if rsi_val < 30
    strategy.entry("Long", strategy.long)
else if rsi_val > 70
    strategy.close("Long")
```

**语法定义（Pest示例）：**
```pest
// pine.pest
strategy_def = { "strategy(" ~ string ~ ")" }
assignment = { identifier ~ "=" ~ expression }
if_statement = { "if" ~ condition ~ block }
indicator_call = { "ta." ~ identifier ~ "(" ~ args ~ ")" }
```

### 3.3 Freqtrade集成

**策略转换器架构：**
```python
# freqtrade_to_pine.py
class StrategyConverter:
    def __init__(self, strategy_class):
        self.strategy = strategy_class

    def convert_to_pine(self) -> str:
        """
        提取Freqtrade策略逻辑，转换为Pine Script
        """
        indicators = self._extract_indicators()
        entry_logic = self._extract_entry_signals()
        exit_logic = self._extract_exit_signals()

        return self._generate_pine_script(
            indicators, entry_logic, exit_logic
        )

    def _extract_indicators(self) -> Dict:
        # 分析populate_indicators方法
        pass

    def _extract_entry_signals(self) -> str:
        # 分析populate_entry_trend方法
        pass
```

**使用流程：**
1. 在Freqtrade中开发策略
2. 使用内置回测功能验证
3. 运行转换器生成Pine Script DSL
4. Rust引擎加载DSL文件
5. 纸面交易验证一致性

---

## 4. 数据存储设计

### 4.1 AWS架构

**部署区域：** ap-northeast-1 (Tokyo)

**核心服务：**
```
┌──────────────────────────────────────────────┐
│  EC2 Instance (c7gn.xlarge)                  │
│  - Graviton3 CPU (ARM64)                     │
│  - 增强网络 (100 Gbps)                        │
│  - Nitro System                              │
│                                              │
│  运行组件:                                    │
│  └─ Rust交易引擎                              │
│     └─ Axum Web服务                          │
└──────────────────────────────────────────────┘
           │                    │
           │                    │
     ┌─────▼─────┐        ┌────▼────┐
     │ElastiCache│        │Timestream│
     │  Redis    │        │   DB    │
     │(r7g.large)│        │         │
     └───────────┘        └─────────┘
```

### 4.2 数据流设计

**实时数据路径：**
```
交易事件 (订单、成交、持仓变化)
    │
    ▼
内存聚合 (每秒批量)
    │
    ├─> ElastiCache Redis (实时查询, TTL=1小时)
    │       │
    │       └─> Web Dashboard (WebSocket推送)
    │
    └─> AWS Timestream (长期存储)
            │
            └─> 历史分析/回测
```

**Redis数据结构：**
```redis
# 实时指标 (Sorted Set)
ZADD trading:latency <timestamp> <latency_ms>
ZADD trading:pnl <timestamp> <pnl_value>

# 持仓状态 (Hash)
HSET position:BTCUSDT quantity avg_price unrealized_pnl

# 订单记录 (Stream)
XADD orders:stream * symbol BTCUSDT side BUY price 50000 ...

# 系统状态 (String with TTL)
SETEX system:health 60 "OK"
```

**Timestream表结构：**
```sql
-- 交易记录表
CREATE TABLE trades (
    time TIMESTAMP,
    symbol VARCHAR,
    side VARCHAR,
    price DOUBLE,
    quantity DOUBLE,
    commission DOUBLE,
    order_id VARCHAR,
    latency_ms BIGINT
);

-- 持仓快照表
CREATE TABLE positions (
    time TIMESTAMP,
    symbol VARCHAR,
    quantity DOUBLE,
    avg_price DOUBLE,
    current_price DOUBLE,
    unrealized_pnl DOUBLE,
    realized_pnl DOUBLE
);

-- 性能指标表
CREATE TABLE metrics (
    time TIMESTAMP,
    metric_name VARCHAR,
    value DOUBLE,
    tags VARCHAR
);
```

---

## 5. 监控系统设计

### 5.1 三层监控架构

#### Layer 1: 核心指标（微秒级）
**监控项：**
- 订单延迟分布 (P50, P95, P99)
- WebSocket消息延迟
- 订单成功率
- 策略信号生成延迟

**实现：**
```rust
pub struct MetricsCollector {
    latency_histogram: Histogram,
    order_counter: Counter,
}

impl MetricsCollector {
    pub fn record_order_latency(&self, duration: Duration) {
        self.latency_histogram.observe(duration.as_micros() as f64);
    }
}
```

#### Layer 2: 风控告警（毫秒级）
**告警触发条件：**
- 单笔亏损 > 0.8% (警告)
- 日亏损 > 2.5% (严重警告)
- 订单失败率 > 5%
- WebSocket断连
- 余额不足

**Telegram Bot实现：**
```rust
pub struct TelegramAlerter {
    bot_token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramAlerter {
    pub async fn send_alert(&self, level: AlertLevel, message: String) {
        let text = format!("🚨 [{:?}] {}", level, message);
        self.client
            .post(&format!(
                "https://api.telegram.org/bot{}/sendMessage",
                self.bot_token
            ))
            .json(&json!({
                "chat_id": self.chat_id,
                "text": text,
                "parse_mode": "Markdown"
            }))
            .send()
            .await
            .ok();
    }
}
```

#### Layer 3: Web Dashboard（秒级）
**技术栈：**
- 后端：Axum (Rust)
- 前端：Svelte + Chart.js
- 通信：WebSocket

**实时数据推送：**
```rust
// Axum WebSocket处理器
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let metrics = state.redis
            .get_latest_metrics()
            .await
            .unwrap();

        socket.send(Message::Text(
            serde_json::to_string(&metrics).unwrap()
        )).await.ok();
    }
}
```

**前端Svelte组件：**
```svelte
<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';

  let pnlData = [];
  let ws;

  onMount(() => {
    ws = new WebSocket('ws://localhost:8080/ws');
    ws.onmessage = (event) => {
      const metrics = JSON.parse(event.data);
      updateChart(metrics);
    };
  });
</script>

<div>
  <canvas id="pnlChart"></canvas>
  <div class="metrics">
    <span>实时PnL: {currentPnl}</span>
    <span>订单延迟: {latency}ms</span>
  </div>
</div>
```

---

## 6. 安全与容错设计

### 6.1 API密钥管理
```rust
// 使用AWS Secrets Manager
pub struct SecretsManager {
    client: aws_sdk_secretsmanager::Client,
}

impl SecretsManager {
    pub async fn get_binance_credentials(&self)
        -> Result<BinanceCredentials, Error> {
        let secret = self.client
            .get_secret_value()
            .secret_id("prod/binance/api-keys")
            .send()
            .await?;

        serde_json::from_str(&secret.secret_string().unwrap())
    }
}
```

### 6.2 断线重连机制
```rust
pub struct ResilientWebSocket {
    url: String,
    ws: Option<WebSocketStream>,
    reconnect_attempts: u32,
}

impl ResilientWebSocket {
    pub async fn connect_with_retry(&mut self) -> Result<()> {
        let max_attempts = 10;
        let mut delay = Duration::from_secs(1);

        for attempt in 0..max_attempts {
            match self.try_connect().await {
                Ok(ws) => {
                    self.ws = Some(ws);
                    self.reconnect_attempts = 0;
                    return Ok(());
                }
                Err(e) => {
                    warn!("连接失败 (尝试 {}/{}): {}",
                          attempt + 1, max_attempts, e);
                    tokio::time::sleep(delay).await;
                    delay *= 2; // 指数退避
                }
            }
        }

        Err(anyhow!("达到最大重连次数"))
    }
}
```

### 6.3 状态持久化
```rust
pub struct StatePersistence {
    redis: RedisClient,
}

impl StatePersistence {
    // 每30秒保存一次状态
    pub async fn snapshot_state(&self, state: &TradingState) {
        let serialized = serde_json::to_string(state).unwrap();
        self.redis
            .set_ex("state:snapshot", serialized, 300)
            .await
            .ok();
    }

    // 启动时恢复状态
    pub async fn restore_state(&self) -> Option<TradingState> {
        let data: Option<String> = self.redis
            .get("state:snapshot")
            .await
            .ok()?;
        serde_json::from_str(&data?).ok()
    }
}
```

---

## 7. 性能优化策略

### 7.1 网络层优化
1. **HTTP/2连接复用：**
   ```rust
   let client = reqwest::Client::builder()
       .http2_prior_knowledge()
       .pool_max_idle_per_host(10)
       .build()?;
   ```

2. **TCP优化（系统层）：**
   ```bash
   # /etc/sysctl.conf
   net.ipv4.tcp_fastopen = 3
   net.core.rmem_max = 134217728
   net.core.wmem_max = 134217728
   ```

3. **DNS缓存：**
   - 使用Route 53 Resolver
   - 本地DNS缓存（systemd-resolved）

### 7.2 计算优化
1. **零拷贝序列化：**
   ```rust
   use serde_json::value::RawValue;

   // 避免不必要的JSON解析
   let raw: &RawValue = message.get();
   ```

2. **SIMD技术指标计算：**
   ```rust
   use packed_simd::f64x4;

   pub fn simd_sma(data: &[f64], period: usize) -> Vec<f64> {
       // 使用SIMD指令加速计算
   }
   ```

3. **内存池：**
   ```rust
   use bumpalo::Bump;

   let arena = Bump::new();
   let orders = arena.alloc_slice_fill_copy(100, Order::default());
   ```

### 7.3 并发优化
```rust
// 使用无锁数据结构
use crossbeam::queue::ArrayQueue;

pub struct OrderQueue {
    queue: Arc<ArrayQueue<Order>>,
}

// 多生产者单消费者模式
tokio::spawn(async move {
    while let Some(order) = order_queue.pop() {
        executor.execute(order).await;
    }
});
```

---

## 8. 测试策略

### 8.1 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_manager_daily_loss_limit() {
        let mut risk_mgr = RiskManager::new(RiskConfig {
            max_daily_loss: 0.03,
            ..Default::default()
        });

        risk_mgr.daily_pnl = -0.035; // 已亏损3.5%

        let order = Order::new_market_buy("BTCUSDT", 0.1);
        assert!(risk_mgr.check_order(&order).is_err());
    }
}
```

### 8.2 集成测试
```rust
// 使用币安Testnet
#[tokio::test]
#[ignore] // 需要网络连接
async fn test_order_execution_flow() {
    let config = Config::testnet();
    let engine = TradingEngine::new(config).await.unwrap();

    let order = Order::new_limit_buy("BTCUSDT", 50000.0, 0.001);
    let result = engine.place_order(order).await;

    assert!(result.is_ok());
}
```

### 8.3 负载测试
```bash
# 使用k6进行WebSocket负载测试
k6 run --vus 100 --duration 1m websocket_test.js
```

---

## 9. 部署指南

### 9.1 AWS资源配置

**Terraform脚本：**
```hcl
# main.tf
provider "aws" {
  region = "ap-northeast-1"
}

resource "aws_instance" "trading_engine" {
  ami           = "ami-xxxxx" # Graviton3 AMI
  instance_type = "c7gn.xlarge"

  vpc_security_group_ids = [aws_security_group.trading_sg.id]

  user_data = file("setup.sh")

  tags = {
    Name = "crypto-trading-engine"
  }
}

resource "aws_elasticache_cluster" "redis" {
  cluster_id           = "trading-redis"
  engine               = "redis"
  node_type            = "cache.r7g.large"
  num_cache_nodes      = 1
  parameter_group_name = "default.redis7"
  port                 = 6379

  subnet_group_name = aws_elasticache_subnet_group.redis_subnet.name
}

resource "aws_timestreamwrite_database" "trading_db" {
  database_name = "trading_metrics"
}
```

### 9.2 Docker部署
```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/trading-engine /usr/local/bin/

CMD ["trading-engine"]
```

**Docker Compose（本地测试）：**
```yaml
version: '3.8'
services:
  trading-engine:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

### 9.3 监控配置
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'trading-engine'
    static_configs:
      - targets: ['localhost:9090']
```

---

## 10. 配置文件示例

### 10.1 主配置文件
```toml
# config.toml
[general]
environment = "production"
log_level = "info"

[binance]
api_endpoint = "https://api.binance.com"
ws_endpoint = "wss://stream.binance.com:9443"
# API密钥从AWS Secrets Manager读取

[trading]
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "DOGEUSDT", "ZECUSDT"]
capital_allocation = 0.7  # 70%资金投入

[risk]
max_position_per_symbol = 0.3  # 单交易对最大30%
max_single_loss = 0.01  # 单笔最大1%
max_daily_loss = 0.03  # 日最大3%
stop_loss_multiplier = 1.5  # 止损倍数

[strategy]
dsl_file = "/etc/trading/strategy.pine"
update_interval_sec = 60  # 策略更新频率

[monitoring]
redis_url = "redis://trading-redis.cache.amazonaws.com:6379"
timestream_db = "trading_metrics"
telegram_bot_token = "..."  # 从环境变量读取
telegram_chat_id = "..."

[performance]
max_order_latency_ms = 50
worker_threads = 4
```

### 10.2 策略配置示例
```pine
// strategy.pine
//@version=5
strategy("Crypto Mean Reversion", overlay=true,
         initial_capital=10000, commission_type=strategy.commission.percent,
         commission_value=0.1)

// 输入参数
bb_length = input.int(20, "Bollinger Bands Length", minval=1)
bb_mult = input.float(2.0, "BB StdDev Multiplier", step=0.1)
rsi_period = input.int(14, "RSI Period", minval=1)
rsi_overbought = input.int(70, "RSI Overbought", minval=50, maxval=100)
rsi_oversold = input.int(30, "RSI Oversold", minval=0, maxval=50)

// 指标计算
[bb_middle, bb_upper, bb_lower] = ta.bb(close, bb_length, bb_mult)
rsi = ta.rsi(close, rsi_period)

// 入场条件
long_condition = close < bb_lower and rsi < rsi_oversold
short_condition = close > bb_upper and rsi > rsi_overbought

// 执行交易
if long_condition
    strategy.entry("Long", strategy.long)

if short_condition
    strategy.close("Long")

// 可视化
plot(bb_middle, color=color.blue)
plot(bb_upper, color=color.red)
plot(bb_lower, color=color.green)
```

---

## 11. 应急预案

### 11.1 故障场景应对

| 故障场景 | 检测方式 | 应对措施 | 恢复时间 |
|---------|---------|---------|---------|
| WebSocket断连 | 心跳超时 | 自动重连（指数退避） | <30秒 |
| API限流 | HTTP 429错误 | 降低请求频率，等待重置 | 1-60秒 |
| 订单失败率高 | 失败率>5% | 暂停交易，发送告警 | 手动介入 |
| 服务器宕机 | 健康检查失败 | AWS Auto Scaling启动备份实例 | <5分钟 |
| 日亏损超限 | 实时PnL监控 | 立即平仓所有持仓，停止交易 | <10秒 |
| Redis故障 | 连接超时 | 切换到内存模式，告警 | 即时 |

### 11.2 紧急止损脚本
```bash
#!/bin/bash
# emergency_stop.sh

echo "🛑 执行紧急停止..."

# 1. 停止交易引擎
systemctl stop trading-engine

# 2. 平仓所有持仓（调用币安API）
curl -X POST "https://api.binance.com/api/v3/order" \
  -H "X-MBX-APIKEY: $BINANCE_API_KEY" \
  -d "symbol=BTCUSDT&side=SELL&type=MARKET&quantity=..."

# 3. 发送Telegram通知
curl -X POST "https://api.telegram.org/bot$TELEGRAM_TOKEN/sendMessage" \
  -d "chat_id=$CHAT_ID&text=🚨 紧急停止已执行"

echo "✅ 已完成"
```

---

## 12. 附录

### 12.1 关键性能指标（KPI）

| 指标 | 目标值 | 测量方式 |
|-----|--------|---------|
| 订单延迟 (P95) | <50ms | Prometheus histogram |
| WebSocket延迟 | <5ms | 时间戳差值 |
| 订单成功率 | >99% | Counter比率 |
| 日收益率 | >0.1% | 每日PnL统计 |
| 最大回撤 | <5% | 历史净值曲线 |
| 系统可用性 | >99.9% | 运行时间/总时间 |

### 12.2 成本估算（月度）

| 项目 | 配置 | 预估成本（USD） |
|-----|------|----------------|
| EC2 (c7gn.xlarge) | On-Demand, 24/7 | ~$280 |
| ElastiCache Redis (r7g.large) | 单节点 | ~$150 |
| AWS Timestream | 10GB写入, 1GB存储 | ~$60 |
| 数据传输 | 100GB/月 | ~$10 |
| **总计** | | **~$500/月** |

*注：预留实例可节省约40%成本*

### 12.3 参考资源

- **币安API文档：** https://binance-docs.github.io/apidocs/spot/en/
- **Rust异步编程：** https://tokio.rs/
- **Pine Script参考：** https://www.tradingview.com/pine-script-docs/
- **Freqtrade文档：** https://www.freqtrade.io/
- **AWS最佳实践：** https://aws.amazon.com/architecture/

---

## 文档变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|-----|------|---------|------|
| 1.0 | 2025-01-22 | 初始版本 | Claude |

---

**文档状态：** ✅ 已审核
**下一步：** 查看 `DEVELOPMENT_TODO.md` 开始开发
