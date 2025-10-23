# 交易引擎端到端测试指南

> **测试目标**: 验证 Phase 1 MVP 核心交易引擎在真实币安 Testnet 环境下的完整功能

**测试日期**: _请填写测试日期_
**测试人员**: _请填写_
**测试环境**: 币安 Testnet + Telegram Bot

---

## 📋 测试前准备检查清单

### 1. 环境检查
- [ ] Rust 版本 >= 1.75
  ```bash
  rustc --version
  ```
- [ ] 网络连接正常 (能访问 binance.com)
- [ ] Git 仓库状态正常
  ```bash
  git status
  ```

### 2. 配置文件检查
- [ ] `config.local.toml` 文件已创建 ✅ (已自动填充)
- [ ] Binance API Key 已配置 ✅
- [ ] Binance Secret Key 已配置 ✅
- [ ] Telegram Bot Token 已配置 ✅
- [ ] Telegram Chat ID 已配置 ✅

**验证命令**:
```bash
cd trading-engine
cat config.local.toml
```

**预期输出**: 应该看到包含 API 密钥和 Telegram 配置的完整配置文件

---

## 🧪 测试阶段 1: 单元测试验证

**目的**: 确保所有模块的单元测试通过

### 执行步骤:

```bash
cd trading-engine
cargo test
```

### 预期结果:
- ✅ 所有测试通过 (应该是 41 个测试)
- ✅ 无编译错误
- ✅ 无运行时错误

### 数据收集:

请记录以下信息:

```
测试总数: ___________
通过数量: ___________
失败数量: ___________
忽略数量: ___________
执行时长: ___________
```

**如果测试失败**, 请记录:
- 失败的测试名称
- 错误信息
- 完整的 cargo test 输出

---

## 🔌 测试阶段 2: Telegram 连接测试

**目的**: 验证 Telegram Bot 能够正常发送消息

### 执行步骤:

```bash
cd trading-engine
cargo run --example test_telegram
```

### 预期结果:
1. 程序输出:
   ```
   Sending test alert...
   Sending trade alert...
   Done! Check your Telegram
   ```

2. **Telegram 接收验证**:
   - [ ] 收到第一条消息: "Trading engine test alert"
   - [ ] 收到第二条消息: 包含 BTC 交易信息 (买入 0.001 BTC @ 50000.0)
   - [ ] 消息格式正确，包含 emoji 和时间戳

### 数据收集:

**成功情况**:
```
是否收到消息: 是/否
消息延迟: ________ 秒
消息格式: 正常/异常
```

**失败情况** (如果有):
```
错误信息:
____________________________
____________________________
```

**截图**: 请保存 Telegram 收到的消息截图

---

## 🌐 测试阶段 3: WebSocket 连接测试

**目的**: 验证能够连接到币安 WebSocket 并接收实时数据

### 执行步骤:

```bash
cd trading-engine
RUST_LOG=info cargo run
```

**注意**: 程序会一直运行，按 `Ctrl+C` 停止

### 预期结果:

**初始启动日志** (前 30 秒):
```
INFO trading_engine: 🚀 Trading Engine Starting
INFO trading_engine: Environment: testnet
INFO trading_engine: Trading symbols: ["BTCUSDT", "ETHUSDT"]
INFO trading_engine::engine: 🚀 Starting trading engine...
INFO trading_engine::websocket: Connecting to WebSocket: wss://stream.binance.com:9443/ws
INFO trading_engine::websocket: WebSocket connected successfully
INFO trading_engine::websocket: Subscribing to streams: ["btcusdt@trade", "ethusdt@trade"]
INFO trading_engine::engine: 📡 Subscribed to market data streams
```

**实时数据日志** (运行中):
```
DEBUG trading_engine::engine: BTCUSDT - Price: 95234.56
DEBUG trading_engine::engine: ETHUSDT - Price: 3421.78
DEBUG trading_engine::engine: BTCUSDT - Price: 95235.12
...
```

### 数据收集:

**连接状态**:
```
WebSocket 连接成功: 是/否
订阅成功: 是/否
开始接收数据时间: ________ 秒后
数据更新频率: 每秒约 ________ 条
```

**观察时长**: 至少运行 **5 分钟**

**运行期间记录**:
```
总运行时长: ________ 分钟
收到的价格更新数量 (大约): ________
是否有断连: 是/否
是否有错误日志: 是/否
```

**Telegram 通知**:
- [ ] 收到 "Trading engine started successfully" 启动通知

**如果有错误**, 请完整复制错误日志。

---

## 📈 测试阶段 4: 策略信号生成测试

**目的**: 验证双均线策略能够生成交易信号

### 执行步骤:

继续上一步的程序运行，或重新启动:

```bash
cd trading-engine
RUST_LOG=debug cargo run
```

**测试时长**: 至少运行 **30 分钟** (策略每 60 秒检查一次)

### 预期结果:

**价格数据积累** (前 10 分钟):
```
DEBUG trading_engine::engine: BTCUSDT: Insufficient price data (15)
DEBUG trading_engine::engine: ETHUSDT: Insufficient price data (12)
...
```

**策略开始工作** (约 20 分钟后，当积累足够数据):
```
DEBUG trading_engine::strategy::dual_ma: MA values - Fast: 95234.50 -> 95236.20, Slow: 95220.10 -> 95225.30
```

**可能生成信号** (取决于市场走势):
```
INFO trading_engine::strategy::dual_ma: 🔼 Bullish crossover detected! Fast MA crossed above Slow MA
INFO trading_engine::engine: 📊 BTCUSDT - LONG signal generated
INFO trading_engine::engine: ✅ Risk check passed, executing BUY order for BTCUSDT
```

或者:
```
INFO trading_engine::strategy::dual_ma: 🔽 Bearish crossover detected! Fast MA crossed below Slow MA
INFO trading_engine::engine: 📊 BTCUSDT - SHORT signal generated
INFO trading_engine::engine: ✅ Risk check passed, executing SELL order for BTCUSDT
```

### 数据收集:

**价格数据积累**:
```
BTCUSDT 价格数据积累到 20+ 条用时: ________ 分钟
ETHUSDT 价格数据积累到 20+ 条用时: ________ 分钟
```

**策略信号**:
```
观察时长: ________ 分钟
生成的 LONG 信号数量: ________
生成的 SHORT 信号数量: ________
无信号 (None) 的次数: ________
```

**Telegram 通知** (如果生成了交易信号):
- [ ] 收到交易执行通知
- [ ] 通知包含交易对、方向、价格、数量

**信号详情** (如有):
```
信号 1:
  时间: ________
  交易对: ________
  方向: ________
  触发原因: ________

信号 2:
  时间: ________
  交易对: ________
  方向: ________
  触发原因: ________
```

---

## 🛡️ 测试阶段 5: 风控功能测试

**目的**: 验证风控管理器能够正确拦截违规交易

### 测试场景 5.1: 正常交易 (应该通过)

这在阶段 4 中已经测试。

### 测试场景 5.2: 模拟日亏损超限 (需要修改代码临时测试)

**方法**: 暂时跳过此测试，因为需要修改代码。在生产环境中会自动触发。

### 数据收集:

```
风控检查次数: ________ (每次生成信号都会检查)
通过检查次数: ________
拒绝交易次数: ________
```

**如果有拒绝**, 记录:
```
拒绝原因: ____________________________
Telegram 是否收到风控告警: 是/否
```

---

## 🕐 测试阶段 6: 稳定性测试 (可选但推荐)

**目的**: 验证系统长时间运行的稳定性

### 执行步骤:

```bash
cd trading-engine
nohup cargo run --release > trading.log 2>&1 &
```

**测试时长**: **24 小时**

### 监控方法:

**实时查看日志**:
```bash
tail -f trading.log
```

**每小时检查一次**:
```bash
# 检查进程是否还在运行
ps aux | grep trading-engine

# 查看最新日志
tail -20 trading.log

# 检查内存使用
top -p $(pgrep trading-engine)
```

### 数据收集:

**每 6 小时记录一次**:

| 时间 | 进程状态 | 内存占用 | 收到消息数 | 备注 |
|------|---------|---------|-----------|------|
| 0h   |         |         |           |      |
| 6h   |         |         |           |      |
| 12h  |         |         |           |      |
| 18h  |         |         |           |      |
| 24h  |         |         |           |      |

**最终统计**:
```
总运行时长: ________ 小时
崩溃次数: ________
内存泄漏: 有/无
平均内存占用: ________ MB
峰值内存占用: ________ MB
```

---

## 📊 测试阶段 7: 性能指标测试

**目的**: 收集系统性能数据

### 数据收集:

运行程序 10 分钟，然后停止，统计:

**WebSocket 性能**:
```bash
# 统计接收到的消息数量
grep "Price:" trading.log | wc -l
```

```
10 分钟内收到的价格更新总数: ________
平均每秒: ________ 条
```

**策略性能**:
```bash
# 统计策略检查次数
grep "MA values" trading.log | wc -l
```

```
策略检查总次数: ________
平均检查间隔: ________ 秒
```

**日志文件大小**:
```bash
ls -lh trading.log
```

```
10 分钟日志文件大小: ________ KB/MB
预计 24 小时日志大小: ________ MB
```

---

## 🐛 常见问题排查

### 问题 1: WebSocket 连接失败

**错误信息**:
```
Failed to connect to WebSocket
```

**解决方案**:
1. 检查网络连接
2. 检查防火墙设置
3. 尝试使用 VPN
4. 验证 ws_endpoint 配置

### 问题 2: Telegram 消息发送失败

**错误信息**:
```
Telegram API error: ...
```

**解决方案**:
1. 验证 bot token 是否正确
2. 验证 chat_id 是否正确
3. 检查是否先与 bot 对话过 (发送 /start)
4. 检查网络是否能访问 api.telegram.org

### 问题 3: 编译错误

**解决方案**:
```bash
# 清理并重新编译
cargo clean
cargo build
```

### 问题 4: API 密钥错误

**错误信息**:
```
API error (401): Unauthorized
```

**解决方案**:
1. 检查 config.local.toml 中的密钥
2. 确认使用的是 Testnet 密钥
3. 重新生成密钥

---

## ✅ 测试完成检查清单

### 必须完成的测试:
- [ ] 阶段 1: 单元测试 ✅
- [ ] 阶段 2: Telegram 连接 ✅
- [ ] 阶段 3: WebSocket 连接 ✅ (至少 5 分钟)
- [ ] 阶段 4: 策略信号生成 ✅ (至少 30 分钟)
- [ ] 阶段 5: 风控功能 ✅

### 推荐完成的测试:
- [ ] 阶段 6: 24 小时稳定性测试
- [ ] 阶段 7: 性能指标收集

---

## 📝 测试报告模板

### 测试概要

**测试日期**: _______________
**测试时长**: _______________
**测试环境**: Binance Testnet

### 测试结果汇总

| 测试阶段 | 状态 | 备注 |
|---------|------|------|
| 单元测试 | ✅/❌ |      |
| Telegram 连接 | ✅/❌ |      |
| WebSocket 连接 | ✅/❌ |      |
| 策略信号生成 | ✅/❌ |      |
| 风控功能 | ✅/❌ |      |
| 稳定性测试 | ✅/❌/跳过 |      |
| 性能指标 | ✅/❌/跳过 |      |

### 发现的问题

**问题 1**:
- 描述: ____________________________
- 严重程度: 严重/中等/轻微
- 重现步骤: ____________________________
- 错误日志: ____________________________

**问题 2**:
...

### 性能数据

- WebSocket 消息接收频率: ________ 条/秒
- 策略执行频率: ________ 次/分钟
- 平均内存占用: ________ MB
- 日志文件增长速度: ________ MB/小时

### 建议

1. ____________________________
2. ____________________________
3. ____________________________

### 总体评价

系统整体表现: 优秀/良好/一般/差

是否建议进入下一阶段开发: 是/否

---

## 🚀 下一步建议

根据测试结果:

### 如果测试全部通过 ✅
1. 提交测试报告
2. 开始 Phase 2 开发 (Pine Script DSL 和策略系统)
3. 考虑使用小额资金进行真实环境测试

### 如果有问题需要修复 ❌
1. 记录所有问题详情
2. 按优先级修复 bug
3. 修复后重新运行测试

---

**测试人员签名**: _______________
**日期**: _______________
