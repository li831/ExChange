# 币安量化交易系统 - 开发任务清单

## 📋 总览

本文档按照四个开发阶段组织任务，每个任务都标注了：
- **优先级：** P0 (关键路径) / P1 (重要) / P2 (优化)
- **预计时间**
- **依赖关系**

---

## 🚀 Phase 1: MVP核心引擎 (4-6周)

### 1.1 项目初始化 (3天)

#### Task 1.1.1: 创建Rust项目结构
- **优先级：** P0
- **预计时间：** 4小时
- **依赖：** 无

**子任务：**
- [ ] 创建Cargo workspace
  ```bash
  cargo new --lib trading-engine
  cd trading-engine
  ```
- [ ] 创建模块目录结构：
  ```
  src/
  ├── main.rs
  ├── lib.rs
  ├── market_data/
  │   ├── mod.rs
  │   ├── websocket.rs
  │   └── orderbook.rs
  ├── strategy/
  │   ├── mod.rs
  │   ├── indicators.rs
  │   └── executor.rs
  ├── risk/
  │   ├── mod.rs
  │   └── manager.rs
  ├── execution/
  │   ├── mod.rs
  │   ├── binance_client.rs
  │   └── order_router.rs
  ├── state/
  │   ├── mod.rs
  │   └── manager.rs
  └── monitoring/
      ├── mod.rs
      └── metrics.rs
  ```
- [ ] 配置Cargo.toml依赖（参考技术设计文档3.1节）
- [ ] 设置.gitignore
- [ ] 创建README.md

**验收标准：**
- `cargo build` 成功编译
- 目录结构清晰
- 基本文档就绪

---

#### Task 1.1.2: 配置管理系统
- **优先级：** P0
- **预计时间：** 3小时
- **依赖：** Task 1.1.1

**子任务：**
- [ ] 创建`config.toml`模板（参考设计文档10.1节）
- [ ] 实现配置加载模块：
  ```rust
  // src/config.rs
  use config::{Config, ConfigError, File};
  use serde::Deserialize;

  #[derive(Debug, Deserialize)]
  pub struct AppConfig {
      pub general: GeneralConfig,
      pub binance: BinanceConfig,
      pub trading: TradingConfig,
      pub risk: RiskConfig,
  }

  impl AppConfig {
      pub fn load() -> Result<Self, ConfigError> {
          Config::builder()
              .add_source(File::with_name("config"))
              .build()?
              .try_deserialize()
      }
  }
  ```
- [ ] 添加环境变量覆盖支持
- [ ] 创建本地开发配置`config.local.toml`

**验收标准：**
- 能够从文件加载配置
- 环境变量可以覆盖配置
- 单元测试通过

---

#### Task 1.1.3: 日志系统设置
- **优先级：** P1
- **预计时间：** 2小时
- **依赖：** Task 1.1.1

**子任务：**
- [ ] 配置tracing订阅器：
  ```rust
  use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

  pub fn init_tracing() {
      tracing_subscriber::registry()
          .with(
              tracing_subscriber::EnvFilter::try_from_default_env()
                  .unwrap_or_else(|_| "trading_engine=debug".into()),
          )
          .with(tracing_subscriber::fmt::layer())
          .init();
  }
  ```
- [ ] 添加结构化日志宏
- [ ] 配置日志级别（dev: debug, prod: info）
- [ ] 测试日志输出

**验收标准：**
- 日志格式清晰可读
- 可通过环境变量控制级别
- 包含时间戳和模块信息

---

### 1.2 币安WebSocket集成 (5天)

#### Task 1.2.1: WebSocket连接管理
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 1.1.2

**子任务：**
- [ ] 实现基础WebSocket客户端：
  ```rust
  // src/market_data/websocket.rs
  use tokio_tungstenite::{connect_async, tungstenite::Message};
  use futures_util::{StreamExt, SinkExt};

  pub struct BinanceWebSocket {
      url: String,
      stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
  }

  impl BinanceWebSocket {
      pub async fn connect(&mut self) -> Result<()> {
          let (ws_stream, _) = connect_async(&self.url).await?;
          self.stream = Some(ws_stream);
          Ok(())
      }

      pub async fn subscribe(&mut self, streams: Vec<String>)
          -> Result<()> {
          let subscribe_msg = json!({
              "method": "SUBSCRIBE",
              "params": streams,
              "id": 1
          });
          self.send(subscribe_msg).await
      }
  }
  ```
- [ ] 实现心跳机制（ping/pong）
- [ ] 添加连接状态跟踪
- [ ] 编写单元测试（mock WebSocket）

**验收标准：**
- 能成功连接到币安测试网WebSocket
- 心跳正常工作
- 测试覆盖率>80%

---

#### Task 1.2.2: 订单簿数据维护
- **优先级：** P0
- **预计时间：** 2天
- **依赖：** Task 1.2.1

**子任务：**
- [ ] 定义订单簿数据结构：
  ```rust
  // src/market_data/orderbook.rs
  use std::collections::BTreeMap;

  pub struct OrderBook {
      symbol: String,
      bids: BTreeMap<OrderedFloat<f64>, f64>, // 价格 -> 数量
      asks: BTreeMap<OrderedFloat<f64>, f64>,
      last_update_id: u64,
  }

  impl OrderBook {
      pub fn update_bids(&mut self, updates: Vec<(f64, f64)>) {
          for (price, qty) in updates {
              if qty == 0.0 {
                  self.bids.remove(&OrderedFloat(price));
              } else {
                  self.bids.insert(OrderedFloat(price), qty);
              }
          }
      }

      pub fn best_bid(&self) -> Option<(f64, f64)> {
          self.bids.iter().next_back()
              .map(|(p, q)| (p.0, *q))
      }

      pub fn mid_price(&self) -> Option<f64> {
          let best_bid = self.best_bid()?.0;
          let best_ask = self.best_ask()?.0;
          Some((best_bid + best_ask) / 2.0)
      }
  }
  ```
- [ ] 实现增量更新处理
- [ ] 添加订单簿快照恢复
- [ ] 性能测试（100ms内处理1000条更新）

**验收标准：**
- 订单簿数据准确
- 能正确处理币安depth消息
- 性能达标

---

#### Task 1.2.3: K线数据聚合
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 1.2.1

**子任务：**
- [ ] 订阅K线流（1m, 5m, 15m）
- [ ] 实现K线数据结构和存储
- [ ] 添加实时K线更新逻辑
- [ ] 提供历史K线查询接口

**验收标准：**
- 能接收并解析K线数据
- 支持多时间周期
- 数据完整无遗漏

---

#### Task 1.2.4: 断线重连机制
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 1.2.1

**子任务：**
- [ ] 实现指数退避重连策略
- [ ] 添加重连计数器和限制
- [ ] 状态恢复逻辑（重新订阅频道）
- [ ] 告警集成（重连失败时通知）

**验收标准：**
- 网络中断后能自动重连
- 重连后数据流正常
- 压力测试通过（模拟频繁断连）

---

### 1.3 币安REST API集成 (4天)

#### Task 1.3.1: HTTP客户端封装
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 1.1.2

**子任务：**
- [ ] 创建BinanceClient结构：
  ```rust
  // src/execution/binance_client.rs
  use reqwest::{Client, Response};
  use hmac::{Hmac, Mac};
  use sha2::Sha256;

  pub struct BinanceClient {
      api_key: String,
      secret_key: String,
      client: Client,
      base_url: String,
  }

  impl BinanceClient {
      pub fn new(api_key: String, secret_key: String) -> Self {
          let client = Client::builder()
              .http2_prior_knowledge()
              .pool_max_idle_per_host(10)
              .timeout(Duration::from_millis(5000))
              .build()
              .unwrap();

          Self {
              api_key,
              secret_key,
              client,
              base_url: "https://testnet.binance.vision".into(),
          }
      }

      fn sign(&self, query: &str) -> String {
          let mut mac = Hmac::<Sha256>::new_from_slice(
              self.secret_key.as_bytes()
          ).unwrap();
          mac.update(query.as_bytes());
          hex::encode(mac.finalize().into_bytes())
      }
  }
  ```
- [ ] 实现请求签名逻辑
- [ ] 添加请求头（API Key, User-Agent）
- [ ] 配置连接池和超时

**验收标准：**
- 能成功发送签名请求
- 请求头正确
- 连接复用工作正常

---

#### Task 1.3.2: 账户信息查询
- **优先级：** P0
- **预计时间：** 0.5天
- **依赖：** Task 1.3.1

**子任务：**
- [ ] 实现GET /api/v3/account
- [ ] 解析余额信息
- [ ] 缓存账户信息（30秒TTL）
- [ ] 错误处理（API限流、网络错误）

**验收标准：**
- 能获取账户余额
- 缓存机制工作
- 优雅处理错误

---

#### Task 1.3.3: 下单接口实现
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 1.3.1

**子任务：**
- [ ] 实现POST /api/v3/order（限价单）
- [ ] 实现POST /api/v3/order（市价单）
- [ ] 定义Order数据结构：
  ```rust
  pub struct Order {
      pub symbol: String,
      pub side: OrderSide, // Buy/Sell
      pub order_type: OrderType, // Limit/Market
      pub quantity: f64,
      pub price: Option<f64>,
      pub time_in_force: Option<TimeInForce>,
  }

  pub enum OrderSide {
      Buy,
      Sell,
  }

  pub enum OrderType {
      Limit,
      Market,
  }
  ```
- [ ] 添加订单响应解析
- [ ] 延迟监控（记录请求耗时）

**验收标准：**
- 能成功下限价单和市价单
- 订单响应正确解析
- 延迟指标被记录

---

#### Task 1.3.4: 订单查询和取消
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 1.3.3

**子任务：**
- [ ] 实现GET /api/v3/order（查询订单）
- [ ] 实现GET /api/v3/openOrders（查询所有挂单）
- [ ] 实现DELETE /api/v3/order（取消订单）
- [ ] 批量取消接口

**验收标准：**
- 各接口功能正常
- 错误场景处理完善

---

### 1.4 简单策略实现 (3天)

#### Task 1.4.1: 双均线策略（硬编码）
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 1.2.3

**子任务：**
- [ ] 实现SMA指标计算：
  ```rust
  // src/strategy/indicators.rs
  pub fn sma(data: &[f64], period: usize) -> Option<f64> {
      if data.len() < period {
          return None;
      }
      let sum: f64 = data.iter().rev().take(period).sum();
      Some(sum / period as f64)
  }
  ```
- [ ] 实现双均线策略逻辑：
  ```rust
  // src/strategy/simple_ma.rs
  pub struct DualMAStrategy {
      fast_period: usize,
      slow_period: usize,
  }

  impl Strategy for DualMAStrategy {
      fn generate_signal(&self, klines: &[Kline]) -> Option<Signal> {
          let closes: Vec<f64> = klines.iter()
              .map(|k| k.close).collect();

          let fast_ma = sma(&closes, self.fast_period)?;
          let slow_ma = sma(&closes, self.slow_period)?;

          if fast_ma > slow_ma {
              Some(Signal::Long)
          } else if fast_ma < slow_ma {
              Some(Signal::Short)
          } else {
              None
          }
      }
  }
  ```
- [ ] 集成到主交易循环
- [ ] 添加日志记录信号生成

**验收标准：**
- 策略逻辑正确
- 能生成交易信号
- 通过历史数据测试

---

#### Task 1.4.2: 策略执行框架
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 1.4.1, Task 1.3.3

**子任务：**
- [ ] 定义Strategy trait：
  ```rust
  pub trait Strategy: Send + Sync {
      fn generate_signal(&self, data: &MarketData)
          -> Option<Signal>;
      fn name(&self) -> &str;
  }

  pub enum Signal {
      Long,
      Short,
      CloseLong,
      CloseShort,
  }
  ```
- [ ] 实现StrategyExecutor：
  ```rust
  pub struct StrategyExecutor {
      strategy: Box<dyn Strategy>,
      order_executor: Arc<OrderExecutor>,
      position_manager: Arc<PositionManager>,
  }

  impl StrategyExecutor {
      pub async fn run(&self) {
          let mut interval = tokio::time::interval(
              Duration::from_secs(60)
          );

          loop {
              interval.tick().await;
              let data = self.get_market_data().await;

              if let Some(signal) = self.strategy.generate_signal(&data) {
                  self.execute_signal(signal).await;
              }
          }
      }
  }
  ```
- [ ] 实现信号到订单的转换
- [ ] 添加执行日志

**验收标准：**
- 策略能定时执行
- 信号正确转换为订单
- 日志完整

---

#### Task 1.4.3: 持仓管理
- **优先级：** P0
- **预计时间：** 0.5天
- **依赖：** Task 1.3.2

**子任务：**
- [ ] 实现PositionManager：
  ```rust
  pub struct PositionManager {
      positions: Arc<RwLock<HashMap<String, Position>>>,
  }

  pub struct Position {
      pub symbol: String,
      pub side: PositionSide,
      pub quantity: f64,
      pub avg_entry_price: f64,
      pub unrealized_pnl: f64,
  }

  impl PositionManager {
      pub async fn update_position(&self, symbol: &str,
                                    trade: &Trade) {
          // 更新持仓信息
      }

      pub fn get_position(&self, symbol: &str)
          -> Option<Position> {
          // 查询持仓
      }
  }
  ```
- [ ] 实现持仓更新逻辑（基于成交回报）
- [ ] 计算未实现盈亏

**验收标准：**
- 持仓信息准确
- 盈亏计算正确

---

### 1.5 基础风控 (3天)

#### Task 1.5.1: 风控管理器实现
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 1.1.2

**子任务：**
- [ ] 实现RiskManager核心逻辑（参考设计文档2.2.3节）
- [ ] 单笔亏损检查（<1%）
- [ ] 日亏损限制检查（<3%）
- [ ] 仓位比例检查（总投入70%，单币种30%）
- [ ] 订单前置校验hook

**验收标准：**
- 所有风控规则正确执行
- 违规订单被拒绝
- 单元测试覆盖所有场景

---

#### Task 1.5.2: 止损止盈逻辑
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 1.5.1

**子任务：**
- [ ] 实现基于价格的止损：
  ```rust
  pub struct StopLossManager {
      stop_orders: HashMap<String, StopOrder>,
  }

  pub struct StopOrder {
      symbol: String,
      trigger_price: f64,
      order_side: OrderSide,
      quantity: f64,
  }

  impl StopLossManager {
      pub async fn check_triggers(&self, current_price: f64) {
          for (symbol, stop) in &self.stop_orders {
              if self.should_trigger(stop, current_price) {
                  self.execute_stop_loss(stop).await;
              }
          }
      }
  }
  ```
- [ ] 添加止盈逻辑（目标收益2%）
- [ ] 集成到主循环

**验收标准：**
- 止损触发及时（<5秒）
- 止损价格准确

---

#### Task 1.5.3: 紧急平仓功能
- **优先级：** P1
- **预计时间：** 0.5天
- **依赖：** Task 1.5.1

**子任务：**
- [ ] 实现一键平仓函数
- [ ] 添加SIGTERM信号处理（优雅关闭）
- [ ] 创建紧急平仓脚本（Bash）

**验收标准：**
- 能快速平掉所有持仓
- 程序退出前自动平仓

---

### 1.6 Telegram告警 (2天)

#### Task 1.6.1: Telegram Bot集成
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 1.1.2

**子任务：**
- [ ] 实现TelegramAlerter（参考设计文档5.1节）
- [ ] 添加告警级别（Info, Warning, Critical）
- [ ] 消息格式化（Markdown）
- [ ] 测试消息发送

**验收标准：**
- 能成功发送Telegram消息
- 消息格式美观

---

#### Task 1.6.2: 告警规则配置
- **优先级：** P1
- **预计时间：** 0.5天
- **依赖：** Task 1.6.1

**子任务：**
- [ ] 配置告警触发条件：
  - 单笔亏损>0.8%
  - 日亏损>2.5%
  - WebSocket断连
  - 订单失败率>5%
- [ ] 集成到风控和监控模块
- [ ] 添加告警频率限制（防止刷屏）

**验收标准：**
- 告警触发准确
- 无重复告警

---

#### Task 1.6.3: 交互命令支持
- **优先级：** P2
- **预计时间：** 0.5天
- **依赖：** Task 1.6.1

**子任务：**
- [ ] 实现Bot命令监听
- [ ] 支持命令：
  - `/status` - 查看系统状态
  - `/positions` - 查看当前持仓
  - `/pnl` - 查看今日盈亏
  - `/stop` - 紧急停止交易

**验收标准：**
- 命令响应及时
- 返回信息准确

---

### 1.7 集成测试 (3天)

#### Task 1.7.1: 币安Testnet端到端测试
- **优先级：** P0
- **预计时间：** 2天
- **依赖：** 所有前置任务

**子任务：**
- [ ] 配置Testnet环境
- [ ] 获取Testnet API密钥
- [ ] 编写完整交易流程测试：
  ```rust
  #[tokio::test]
  #[ignore]
  async fn test_full_trading_cycle() {
      // 1. 启动引擎
      let engine = TradingEngine::new(testnet_config()).await;

      // 2. 连接WebSocket
      engine.start_market_data().await;

      // 3. 等待数据稳定
      tokio::time::sleep(Duration::from_secs(10)).await;

      // 4. 触发策略信号（手动）
      let signal = Signal::Long;
      engine.execute_signal("BTCUSDT", signal).await;

      // 5. 验证订单执行
      let positions = engine.get_positions().await;
      assert!(positions.contains_key("BTCUSDT"));

      // 6. 平仓
      engine.close_all_positions().await;
  }
  ```
- [ ] 运行24小时稳定性测试
- [ ] 记录所有异常

**验收标准：**
- 端到端流程无报错
- 24小时运行稳定
- 无内存泄漏

---

#### Task 1.7.2: 性能基准测试
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 1.7.1

**子任务：**
- [ ] 测试订单延迟（目标<100ms）
- [ ] 测试WebSocket消息处理延迟
- [ ] 测试策略计算耗时
- [ ] 生成性能报告

**验收标准：**
- 订单延迟<100ms（Phase 1目标）
- 报告清晰展示瓶颈

---

### Phase 1 交付清单

- [ ] ✅ 能连接币安Testnet并接收实时数据
- [ ] ✅ 双均线策略正常运行
- [ ] ✅ 风控规则生效（亏损限制、仓位限制）
- [ ] ✅ Telegram告警正常
- [ ] ✅ 24小时稳定性测试通过
- [ ] ✅ 基础文档完成（README, 配置说明）

---

## 🧩 Phase 2: 策略系统 (3-4周)

### 2.1 Pine Script DSL设计 (5天)

#### Task 2.1.1: DSL语法定义
- **优先级：** P0
- **预计时间：** 2天
- **依赖：** Phase 1完成

**子任务：**
- [ ] 研究Pine Script v5语法
- [ ] 定义支持的核心语法子集：
  - 变量声明和赋值
  - 技术指标函数（ta.sma, ta.rsi, ta.bb等）
  - 条件语句（if/else）
  - 策略函数（strategy.entry, strategy.close）
- [ ] 编写BNF语法规范文档
- [ ] 创建示例策略文件

**验收标准：**
- 语法规范清晰完整
- 示例策略覆盖主要功能

---

#### Task 2.1.2: 词法分析器实现
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 2.1.1

**子任务：**
- [ ] 选择解析库（推荐pest）
- [ ] 编写Pest语法文件：
  ```pest
  // pine.pest
  strategy = { strategy_declaration ~ statement* }

  strategy_declaration = {
      "strategy" ~ "(" ~ string ~ ("," ~ param)* ~ ")"
  }

  statement = {
      assignment | if_statement | strategy_call
  }

  assignment = { identifier ~ "=" ~ expression }

  expression = {
      number | identifier | function_call | binary_op
  }

  function_call = {
      namespace? ~ identifier ~ "(" ~ (expression ~ ("," ~ expression)*)? ~ ")"
  }
  ```
- [ ] 实现Tokenizer
- [ ] 单元测试（解析各类表达式）

**验收标准：**
- 能解析示例策略文件
- 测试覆盖率>90%

---

#### Task 2.1.3: 语法树生成
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 2.1.2

**子任务：**
- [ ] 定义AST节点类型：
  ```rust
  pub enum ASTNode {
      Strategy(StrategyNode),
      Assignment(String, Box<ASTNode>),
      IfStatement(ConditionNode, Vec<ASTNode>),
      FunctionCall(String, Vec<ASTNode>),
      Literal(Value),
  }

  pub struct StrategyNode {
      pub name: String,
      pub params: HashMap<String, Value>,
      pub body: Vec<ASTNode>,
  }
  ```
- [ ] 实现Parser（Pest Pairs -> AST）
- [ ] 添加语义检查（变量未定义等）
- [ ] 可视化AST（调试用）

**验收标准：**
- 能生成正确的AST
- 语义错误能被捕获

---

### 2.2 技术指标库 (4天)

#### Task 2.2.1: 基础指标实现
- **优先级：** P0
- **预计时间：** 2天
- **依赖：** Phase 1

**子任务：**
- [ ] 实现以下指标（参考TA-Lib）：
  - SMA（简单移动平均）
  - EMA（指数移动平均）
  - RSI（相对强弱指数）
  - MACD（指数平滑异同移动平均线）
  - Bollinger Bands（布林带）
  - ATR（平均真实波幅）
  - Stochastic（随机指标）
- [ ] 优化计算性能（考虑SIMD）
- [ ] 单元测试（与TA-Lib结果对比）

**验收标准：**
- 所有指标计算正确
- 性能测试通过（1000数据点<1ms）

---

#### Task 2.2.2: 指标注册系统
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 2.2.1

**子任务：**
- [ ] 实现IndicatorRegistry：
  ```rust
  pub struct IndicatorRegistry {
      indicators: HashMap<String, Box<dyn Indicator>>,
  }

  pub trait Indicator: Send + Sync {
      fn calculate(&self, data: &[f64], params: &[f64])
          -> Result<Vec<f64>>;
      fn name(&self) -> &str;
  }

  impl IndicatorRegistry {
      pub fn register<I: Indicator + 'static>(&mut self, indicator: I) {
          self.indicators.insert(
              indicator.name().to_string(),
              Box::new(indicator)
          );
      }

      pub fn call(&self, name: &str, data: &[f64], params: &[f64])
          -> Result<Vec<f64>> {
          self.indicators.get(name)
              .ok_or_else(|| anyhow!("指标不存在"))?
              .calculate(data, params)
      }
  }
  ```
- [ ] 注册所有已实现的指标
- [ ] 添加动态调用接口

**验收标准：**
- 能通过名称调用指标
- 支持运行时注册新指标

---

#### Task 2.2.3: 指标缓存优化
- **优先级：** P2
- **预计时间：** 1天
- **依赖：** Task 2.2.2

**子任务：**
- [ ] 实现指标结果缓存
- [ ] 增量更新（新K线到来时只计算差量）
- [ ] LRU缓存策略
- [ ] 性能对比测试

**验收标准：**
- 缓存命中率>90%
- 计算速度提升10倍以上

---

### 2.3 DSL解释器 (5天)

#### Task 2.3.1: AST执行器
- **优先级：** P0
- **预计时间：** 2天
- **依赖：** Task 2.1.3, Task 2.2.2

**子任务：**
- [ ] 实现ASTInterpreter：
  ```rust
  pub struct ASTInterpreter {
      registry: IndicatorRegistry,
      variables: HashMap<String, Value>,
      market_data: Arc<MarketData>,
  }

  impl ASTInterpreter {
      pub fn execute(&mut self, node: &ASTNode)
          -> Result<Option<Signal>> {
          match node {
              ASTNode::Assignment(name, expr) => {
                  let value = self.eval_expression(expr)?;
                  self.variables.insert(name.clone(), value);
                  Ok(None)
              }
              ASTNode::IfStatement(cond, body) => {
                  if self.eval_condition(cond)? {
                      for stmt in body {
                          if let Some(sig) = self.execute(stmt)? {
                              return Ok(Some(sig));
                          }
                      }
                  }
                  Ok(None)
              }
              ASTNode::FunctionCall(name, args) => {
                  self.call_function(name, args)
              }
              _ => Ok(None)
          }
      }
  }
  ```
- [ ] 实现表达式求值
- [ ] 实现条件判断
- [ ] 添加运行时错误处理

**验收标准：**
- 能执行简单策略DSL
- 运行时错误有清晰提示

---

#### Task 2.3.2: 策略函数实现
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 2.3.1

**子任务：**
- [ ] 实现`strategy.entry()`
- [ ] 实现`strategy.close()`
- [ ] 实现`strategy.exit()`（止损止盈）
- [ ] 添加参数验证
- [ ] 生成信号对象

**验收标准：**
- 策略函数正确生成交易信号
- 参数验证完善

---

#### Task 2.3.3: 内置函数库
- **优先级：** P1
- **预计时间：** 1.5天
- **依赖：** Task 2.3.1

**子任务：**
- [ ] 实现`input()`函数（参数定义）
- [ ] 实现`ta.*`命名空间函数（调用指标）
- [ ] 实现基本运算符（+, -, *, /, >, <, ==）
- [ ] 实现`close`, `open`, `high`, `low`变量（当前K线数据）

**验收标准：**
- 所有内置函数可用
- 与Pine Script行为一致

---

### 2.4 Freqtrade集成 (4天)

#### Task 2.4.1: Freqtrade环境搭建
- **优先级：** P0
- **预计时间：** 0.5天
- **依赖：** 无

**子任务：**
- [ ] 安装Freqtrade
  ```bash
  git clone https://github.com/freqtrade/freqtrade.git
  cd freqtrade
  ./setup.sh -i
  ```
- [ ] 配置币安Testnet连接
- [ ] 运行示例策略验证环境

**验收标准：**
- Freqtrade正常运行
- 能连接币安Testnet

---

#### Task 2.4.2: 策略转换器开发
- **优先级：** P0
- **预计时间：** 2天
- **依赏：** Task 2.4.1, Task 2.1.1

**子任务：**
- [ ] 创建Python转换器项目：
  ```python
  # freqtrade_converter/converter.py
  import ast
  import inspect
  from freqtrade.strategy import IStrategy

  class StrategyToPineConverter:
      def __init__(self, strategy_class: type[IStrategy]):
          self.strategy = strategy_class

      def convert(self) -> str:
          """转换策略为Pine Script"""
          indicators = self._extract_indicators()
          entry = self._extract_entry_logic()
          exit = self._extract_exit_logic()

          return self._generate_pine(indicators, entry, exit)

      def _extract_indicators(self) -> dict:
          """解析populate_indicators方法"""
          source = inspect.getsource(
              self.strategy.populate_indicators
          )
          tree = ast.parse(source)
          # 分析AST，提取指标调用
          return indicators

      def _generate_pine(self, indicators, entry, exit) -> str:
          template = """
          //@version=5
          strategy("{name}", overlay=true)

          {indicators}

          {entry_logic}

          {exit_logic}
          """
          return template.format(...)
  ```
- [ ] 实现指标提取逻辑
- [ ] 实现入场/出场逻辑提取
- [ ] 生成Pine Script代码
- [ ] 单元测试（转换示例策略）

**验收标准：**
- 能转换简单Freqtrade策略
- 生成的Pine Script语法正确

---

#### Task 2.4.3: 回测一致性验证
- **优先级：** P1
- **预计时间：** 1.5天
- **依赖：** Task 2.4.2, Task 2.3.2

**子任务：**
- [ ] 选择测试策略（如简单的RSI策略）
- [ ] 在Freqtrade中回测，记录结果
- [ ] 转换为Pine Script DSL
- [ ] 在Rust引擎中回测（使用相同历史数据）
- [ ] 对比两边的交易信号和收益
- [ ] 调试差异，确保一致性

**验收标准：**
- 两边生成的交易信号>95%一致
- 收益曲线基本相同

---

### 2.5 配置化策略加载 (2天)

#### Task 2.5.1: 策略热加载
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 2.3.3

**子任务：**
- [ ] 实现策略文件监控（notify库）
- [ ] 策略变更时重新加载
- [ ] 添加语法校验（加载前）
- [ ] 平滑切换（不中断交易）

**验收标准：**
- 修改策略文件后自动生效
- 无需重启程序

---

#### Task 2.5.2: 多策略管理
- **优先级：** P2
- **预计时间：** 1天
- **依赖：** Task 2.5.1

**子任务：**
- [ ] 支持同时运行多个策略
- [ ] 配置每个策略的资金分配
- [ ] 独立统计每个策略的收益
- [ ] 添加策略启用/禁用开关

**验收标准：**
- 能同时运行2-3个策略
- 互不干扰

---

### Phase 2 交付清单

- [ ] ✅ Pine Script DSL解析器完成
- [ ] ✅ 10+技术指标可用
- [ ] ✅ 能从Freqtrade转换策略
- [ ] ✅ DSL策略可执行且信号正确
- [ ] ✅ 回测一致性验证通过
- [ ] ✅ 策略文档完善

---

## 📊 Phase 3: 监控系统 (2-3周)

### 3.1 Redis集成 (3天)

#### Task 3.1.1: AWS ElastiCache设置
- **优先级：** P0
- **预计时间：** 0.5天
- **依赖：** Phase 2完成

**子任务：**
- [ ] 使用Terraform创建ElastiCache集群
- [ ] 配置安全组（仅EC2可访问）
- [ ] 获取连接端点
- [ ] 测试连接

**验收标准：**
- Redis集群运行正常
- 从EC2能访问

---

#### Task 3.1.2: Redis客户端集成
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 3.1.1

**子任务：**
- [ ] 集成redis-rs库
- [ ] 实现连接池
- [ ] 封装常用操作：
  ```rust
  pub struct RedisStore {
      client: redis::aio::ConnectionManager,
  }

  impl RedisStore {
      pub async fn record_metric(&self, key: &str, value: f64)
          -> Result<()> {
          let timestamp = Utc::now().timestamp();
          self.client
              .zadd(key, value, timestamp)
              .await?;
          Ok(())
      }

      pub async fn get_latest_metrics(&self, key: &str, count: usize)
          -> Result<Vec<(f64, f64)>> {
          self.client
              .zrevrange_withscores(key, 0, count as isize - 1)
              .await
      }
  }
  ```
- [ ] 添加错误重试逻辑

**验收标准：**
- Redis操作稳定
- 连接断开能自动恢复

---

#### Task 3.1.3: 指标写入集成
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 3.1.2

**子任务：**
- [ ] 在关键点插入指标记录：
  - 订单延迟（下单时）
  - 持仓变化（成交时）
  - PnL更新（每分钟）
  - 策略信号（生成时）
- [ ] 实现批量写入（减少网络往返）
- [ ] 添加TTL（1小时后自动删除）
- [ ] 性能测试（不影响交易延迟）

**验收标准：**
- 指标记录完整
- 写入延迟<1ms

---

### 3.2 AWS Timestream集成 (3天)

#### Task 3.2.1: Timestream数据库设置
- **优先级：** P1
- **预计时间：** 0.5天
- **依赖：** Task 3.1.1

**子任务：**
- [ ] 创建Timestream数据库和表（Terraform）
- [ ] 配置数据保留策略（热数据30天，冷数据1年）
- [ ] 设置IAM权限

**验收标准：**
- Timestream数据库可用
- 权限配置正确

---

#### Task 3.2.2: Timestream写入客户端
- **优先级：** P1
- **预计时间：** 1.5天
- **依赖：** Task 3.2.1

**子任务：**
- [ ] 集成aws-sdk-timestreamwrite
- [ ] 实现批量写入逻辑：
  ```rust
  pub struct TimestreamWriter {
      client: aws_sdk_timestreamwrite::Client,
      buffer: Vec<Record>,
      flush_interval: Duration,
  }

  impl TimestreamWriter {
      pub async fn write_trade(&mut self, trade: &Trade) {
          let record = Record::builder()
              .time(trade.timestamp.to_string())
              .dimension("symbol", &trade.symbol)
              .measure_name("price")
              .measure_value(&trade.price.to_string())
              .measure_value_type(MeasureValueType::Double)
              .build();

          self.buffer.push(record);

          if self.buffer.len() >= 100 {
              self.flush().await;
          }
      }

      async fn flush(&mut self) {
          self.client
              .write_records()
              .database_name("trading_metrics")
              .table_name("trades")
              .records(std::mem::take(&mut self.buffer))
              .send()
              .await
              .ok();
      }
  }
  ```
- [ ] 实现定时刷新（每秒）
- [ ] 错误处理（写入失败不影响交易）

**验收标准：**
- 数据成功写入Timestream
- 批量写入性能良好

---

#### Task 3.2.3: 历史数据查询
- **优先级：** P2
- **预计时间：** 1天
- **依赖：** Task 3.2.2

**子任务：**
- [ ] 实现常用查询封装：
  - 按时间范围查询交易记录
  - 计算历史收益率
  - 统计策略表现
- [ ] 添加查询缓存
- [ ] 创建查询API端点

**验收标准：**
- 查询功能正常
- 响应速度<1秒

---

### 3.3 Web Dashboard (7天)

#### Task 3.3.1: Axum后端框架
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 3.1.3

**子任务：**
- [ ] 创建Axum服务器：
  ```rust
  use axum::{Router, routing::get};

  pub async fn start_web_server(state: AppState) {
      let app = Router::new()
          .route("/", get(serve_index))
          .route("/ws", get(websocket_handler))
          .route("/api/metrics", get(get_metrics))
          .with_state(state);

      axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
          .serve(app.into_make_service())
          .await
          .unwrap();
  }
  ```
- [ ] 实现静态文件服务
- [ ] 添加CORS配置
- [ ] 健康检查端点

**验收标准：**
- 服务器正常启动
- 静态文件可访问

---

#### Task 3.3.2: WebSocket实时推送
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 3.3.1

**子任务：**
- [ ] 实现WebSocket连接管理
- [ ] 定时从Redis读取最新指标（1秒）
- [ ] 序列化为JSON推送到前端
- [ ] 处理客户端断连

**验收标准：**
- 前端能接收实时数据
- 多客户端并发支持

---

#### Task 3.3.3: REST API端点
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 3.3.1

**子任务：**
- [ ] 实现以下API：
  - GET /api/positions - 当前持仓
  - GET /api/orders - 订单历史
  - GET /api/pnl - 盈亏统计
  - GET /api/system - 系统状态
- [ ] 添加认证（Bearer Token）
- [ ] API文档（OpenAPI）

**验收标准：**
- 所有端点功能正常
- 返回数据准确

---

#### Task 3.3.4: Svelte前端基础
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 3.3.2

**子任务：**
- [ ] 初始化Svelte项目
  ```bash
  npm create vite@latest dashboard -- --template svelte
  ```
- [ ] 设计页面布局：
  - 顶部：系统状态栏
  - 左侧：持仓列表
  - 中间：PnL曲线图
  - 右侧：订单日志
- [ ] 实现WebSocket连接逻辑
- [ ] 响应式设计（支持移动端）

**验收标准：**
- 页面布局美观
- 数据流连通

---

#### Task 3.3.5: 图表组件实现
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 3.3.4

**子任务：**
- [ ] 集成Chart.js或ECharts
- [ ] 实现PnL曲线图（实时更新）
- [ ] 实现持仓饼图
- [ ] 实现延迟柱状图
- [ ] 添加时间范围选择（1h, 1d, 1w）

**验收标准：**
- 图表流畅更新
- 交互体验良好

---

### 3.4 性能优化 (4天)

#### Task 3.4.1: 网络延迟优化
- **优先级：** P0
- **预计时间：** 2天
- **依赖：** Phase 2完成

**子任务：**
- [ ] 部署到AWS东京（ap-northeast-1）
- [ ] 测试到币安的延迟（ping, traceroute）
- [ ] 优化HTTP/2设置
- [ ] 启用TCP_NODELAY
- [ ] 调整系统参数（sysctl）
- [ ] 压测订单延迟

**验收标准：**
- 订单延迟降至<50ms（P95）
- WebSocket延迟<10ms

---

#### Task 3.4.2: 内存优化
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** 无

**子任务：**
- [ ] 使用Valgrind检测内存泄漏
- [ ] 优化订单簿数据结构（减少复制）
- [ ] 实现对象池（Order, Trade等）
- [ ] 长时间运行测试（监控内存使用）

**验收标准：**
- 无内存泄漏
- 7天运行内存增长<100MB

---

#### Task 3.4.3: CPU优化
- **优先级：** P2
- **预计时间：** 1天
- **依赖：** Task 2.2.1

**子任务：**
- [ ] 使用flamegraph分析热点
- [ ] 优化指标计算（SIMD）
- [ ] 减少锁竞争（无锁队列）
- [ ] 基准测试对比

**验收标准：**
- CPU使用率降低20%
- 吞吐量提升

---

### Phase 3 交付清单

- [ ] ✅ Redis和Timestream集成完成
- [ ] ✅ Web Dashboard正常运行
- [ ] ✅ 实时监控数据准确
- [ ] ✅ 订单延迟<50ms（P95）
- [ ] ✅ 系统稳定性测试通过（7天无故障）

---

## 🚢 Phase 4: 生产就绪 (2周)

### 4.1 错误处理和容错 (3天)

#### Task 4.1.1: 全局错误处理
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Phase 3完成

**子任务：**
- [ ] 统一错误类型定义：
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum TradingError {
      #[error("网络错误: {0}")]
      Network(#[from] reqwest::Error),

      #[error("币安API错误: {code} - {msg}")]
      BinanceAPI { code: i32, msg: String },

      #[error("风控拒绝: {0}")]
      RiskRejected(String),

      #[error("策略错误: {0}")]
      Strategy(String),
  }
  ```
- [ ] 实现错误恢复策略
- [ ] 添加错误计数和限流（防止错误风暴）
- [ ] 关键错误触发告警

**验收标准：**
- 所有错误有明确分类
- 关键错误不会导致崩溃

---

#### Task 4.1.2: 状态持久化和恢复
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 4.1.1

**子任务：**
- [ ] 实现状态快照（每30秒）
- [ ] 保存到Redis（带过期时间）
- [ ] 启动时恢复状态逻辑
- [ ] 测试崩溃恢复场景

**验收标准：**
- 程序重启后能恢复持仓状态
- 未完成订单能正确处理

---

#### Task 4.1.3: 优雅关闭
- **优先级：** P0
- **预计时间：** 0.5天
- **依赖：** Task 4.1.2

**子任务：**
- [ ] 捕获SIGTERM/SIGINT信号
- [ ] 关闭流程：
  1. 停止接受新信号
  2. 等待当前订单完成
  3. 保存状态快照
  4. 关闭WebSocket
  5. 退出
- [ ] 超时强制退出（30秒）

**验收标准：**
- kill命令能优雅关闭
- 状态正确保存

---

### 4.2 安全加固 (2天)

#### Task 4.2.1: API密钥安全
- **优先级：** P0
- **预计时间：** 0.5天
- **依赖：** 无

**子任务：**
- [ ] 集成AWS Secrets Manager
- [ ] 从环境变量读取密钥（不写入配置文件）
- [ ] 内存中密钥加密存储
- [ ] 定期轮换密钥（手动）

**验收标准：**
- 密钥不出现在代码和日志中
- 通过安全审计

---

#### Task 4.2.2: 访问控制
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 4.2.1

**子任务：**
- [ ] Web Dashboard添加认证
- [ ] 实现JWT令牌
- [ ] IP白名单限制
- [ ] 审计日志（所有API调用）

**验收标准：**
- 未授权无法访问Dashboard
- 所有操作有日志

---

#### Task 4.2.3: 安全测试
- **优先级：** P1
- **预计时间：** 0.5天
- **依赖：** Task 4.2.2

**子任务：**
- [ ] SQL注入测试（如适用）
- [ ] XSS测试（前端）
- [ ] API限流测试
- [ ] 依赖库漏洞扫描（cargo audit）

**验收标准：**
- 无已知高危漏洞
- 通过基础安全测试

---

### 4.3 部署自动化 (3天)

#### Task 4.3.1: Docker镜像构建
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 4.1.3

**子任务：**
- [ ] 优化Dockerfile（多阶段构建）
- [ ] 添加健康检查
- [ ] 配置日志输出到stdout
- [ ] 推送到ECR（AWS容器仓库）

**验收标准：**
- 镜像大小<100MB
- 启动时间<10秒

---

#### Task 4.3.2: Terraform基础设施代码
- **优先级：** P0
- **预计时间：** 1.5天
- **依赖：** Task 4.3.1

**子任务：**
- [ ] 完善Terraform脚本（参考设计文档9.1节）
- [ ] 定义所有AWS资源：
  - EC2实例
  - ElastiCache Redis
  - Timestream数据库
  - VPC和安全组
  - IAM角色
- [ ] 添加变量和输出
- [ ] 测试部署流程

**验收标准：**
- `terraform apply`能一键部署
- 资源配置正确

---

#### Task 4.3.3: CI/CD流水线
- **优先级：** P1
- **预计时间：** 0.5天
- **依赖：** Task 4.3.2

**子任务：**
- [ ] 创建GitHub Actions工作流：
  ```yaml
  name: Deploy
  on:
    push:
      branches: [main]

  jobs:
    deploy:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v3
        - name: Build Docker image
          run: docker build -t trading-engine .
        - name: Push to ECR
          run: |
            aws ecr get-login-password | docker login ...
            docker push ...
        - name: Deploy to EC2
          run: ssh ec2-user@$HOST 'docker pull && docker restart'
  ```
- [ ] 添加测试阶段
- [ ] 自动部署到测试环境

**验收标准：**
- 代码推送后自动部署
- 部署失败有通知

---

### 4.4 文档和培训 (2天)

#### Task 4.4.1: 运维文档
- **优先级：** P0
- **预计时间：** 1天
- **依赖：** Task 4.3.3

**子任务：**
- [ ] 编写部署手册
- [ ] 编写故障排查指南
- [ ] 编写配置参考
- [ ] 编写监控指标说明
- [ ] 创建FAQ文档

**验收标准：**
- 文档清晰完整
- 他人能根据文档部署

---

#### Task 4.4.2: 用户手册
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Phase 3完成

**子任务：**
- [ ] 编写策略开发指南（Pine Script）
- [ ] 编写Freqtrade使用指南
- [ ] 编写Dashboard使用说明
- [ ] 录制演示视频（可选）

**验收标准：**
- 新用户能快速上手
- 示例完整

---

### 4.5 生产环境测试 (4天)

#### Task 4.5.1: 小资金实盘测试
- **优先级：** P0
- **预计时间：** 3天
- **依赖：** Task 4.3.2

**子任务：**
- [ ] 部署到生产环境
- [ ] 配置小额资金（如$100）
- [ ] 选择低波动币种（BTC/USDT）
- [ ] 运行保守策略（低频）
- [ ] 每日检查日志和监控
- [ ] 记录所有异常

**验收标准：**
- 3天无崩溃
- 交易逻辑正确
- 盈亏与预期一致

---

#### Task 4.5.2: 压力测试
- **优先级：** P1
- **预计时间：** 1天
- **依赖：** Task 4.5.1

**子任务：**
- [ ] 模拟高频交易场景（订单频率x10）
- [ ] 测试极端市场波动
- [ ] 测试同时多策略运行
- [ ] 监控系统资源使用

**验收标准：**
- 系统稳定
- 无性能瓶颈

---

### Phase 4 交付清单

- [ ] ✅ 完整错误处理和容错机制
- [ ] ✅ 安全加固完成
- [ ] ✅ 一键部署流程就绪
- [ ] ✅ 文档齐全
- [ ] ✅ 小资金实盘测试成功（3天）
- [ ] ✅ 生产环境稳定运行

---

## 📈 后续优化方向（Phase 5+）

### 5.1 高级功能
- [ ] 多交易所支持（Coinbase, Kraken）
- [ ] 跨交易所套利
- [ ] 机器学习预测模型集成
- [ ] 网格交易策略
- [ ] 动态止损（追踪止损）

### 5.2 性能提升
- [ ] 真正的高频交易（<10ms延迟）
- [ ] FPGA硬件加速（极客方向）
- [ ] 协议层优化（FIX协议）
- [ ] 专线连接到交易所

### 5.3 风控增强
- [ ] 基于VaR的风险管理
- [ ] 市场冲击成本估算
- [ ] 资金费率监控（期货）

### 5.4 运营工具
- [ ] 移动端App（查看监控）
- [ ] 多账户管理
- [ ] 策略市场（分享策略）

---

## 🎯 里程碑总结

| 阶段 | 时间 | 核心交付 | 关键指标 |
|-----|------|---------|---------|
| Phase 1 | 4-6周 | MVP核心引擎 | 订单延迟<100ms, 24h稳定 |
| Phase 2 | 3-4周 | 策略系统 | DSL可用，回测一致性>95% |
| Phase 3 | 2-3周 | 监控系统 | 订单延迟<50ms, Web UI |
| Phase 4 | 2周 | 生产就绪 | 安全审计通过，实盘测试 |
| **总计** | **11-15周** | **完整系统** | **可持续盈利** |

---

## 📝 任务管理建议

### 使用工具
- **代码管理：** Git + GitHub
- **任务跟踪：** GitHub Projects 或 Jira
- **文档：** Markdown + GitHub Wiki
- **沟通：** Slack/Discord

### 工作流程
1. 每个Task创建一个Git分支
2. 完成后提交PR，代码审查
3. 合并到main分支
4. 更新本TODO文档（标记完成）
5. 周报总结进度

### 质量保证
- 每个Task要求：
  - [ ] 单元测试
  - [ ] 代码注释
  - [ ] 更新文档
  - [ ] 性能测试（如适用）

---

**祝开发顺利！如有问题，参考技术设计文档或提Issue讨论。**
