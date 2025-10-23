# 币安量化交易引擎

**当前版本**: Phase 1 MVP (92% 完成)
**状态**: ✅ 核心功能已实现，准备测试

## 🎯 项目概述

高性能的加密货币量化交易系统，采用 Rust + Tokio 异步架构，专注于币安交易所的现货市场。

### 核心特性

- ⚡ **低延迟**: WebSocket 实时数据流
- 🛡️ **风险控制**: 日亏损 <3%, 单笔亏损 <1%
- 📊 **双均线策略**: 金叉死叉自动检测
- 📱 **Telegram 告警**: 实时交易通知
- ✅ **测试驱动**: 41 个单元测试，100% 通过

### 已实现功能

| 模块 | 状态 | 测试覆盖 |
|------|------|----------|
| 配置系统 | ✅ | 3 tests |
| WebSocket 客户端 | ✅ | 3 tests |
| OrderBook 数据结构 | ✅ | 9 tests |
| Binance REST API | ✅ | 8 tests |
| 技术指标 (SMA, RSI) | ✅ | 12 tests |
| 双均线策略 | ✅ | 3 tests |
| 风控管理器 | ✅ | 3 tests |
| Telegram 告警 | ✅ | 1 test |
| 主交易循环 | ✅ | - |

---

## 🚀 快速开始

### 1. 环境要求

- Rust 1.75+
- 币安 Testnet API 密钥
- Telegram Bot Token (可选，用于告警)

### 2. 配置

配置文件已自动创建并填充密钥 ✅

查看配置:
```bash
cat config.local.toml
```

### 3. 运行测试

```bash
# 运行所有单元测试
cargo test

# 测试 Telegram 连接
cargo run --example test_telegram
```

### 4. 启动交易引擎

```bash
# 开发模式 (带详细日志)
RUST_LOG=info cargo run

# 生产模式 (优化编译)
cargo run --release
```

### 5. 观察运行

程序启动后会:
1. 连接币安 WebSocket
2. 订阅 BTCUSDT 和 ETHUSDT 实时数据
3. 每 60 秒检查策略信号
4. 发送 Telegram 通知

**停止**: 按 `Ctrl+C`

---

## 📖 文档

### 必读文档

- 📘 [快速启动指南](../docs/QUICK_START.md) - 5 分钟上手
- 📗 [详细测试指南](../docs/TESTING_GUIDE.md) - 完整测试流程
- 📋 [测试结果模板](../docs/TEST_RESULTS.md) - 数据收集表格

### 技术文档

- 📄 [技术设计文档](../TECHNICAL_DESIGN.md)
- 📝 [开发计划](../docs/plans/2025-01-22-phase1-mvp-core-engine.md)

---

## 🧪 测试指南

### 快速测试 (5 分钟)

```bash
# 1. 单元测试
cargo test

# 2. Telegram 连接
cargo run --example test_telegram

# 3. 运行引擎 (观察 5 分钟)
RUST_LOG=info cargo run
```

### 完整测试 (30 分钟)

参考: [TESTING_GUIDE.md](../docs/TESTING_GUIDE.md)

1. ✅ 单元测试验证
2. ✅ Telegram 连接测试
3. ✅ WebSocket 连接测试 (5 分钟)
4. ✅ 策略信号生成测试 (30 分钟)
5. ✅ 风控功能测试

### 稳定性测试 (24 小时，可选)

```bash
# 后台运行
nohup cargo run --release > trading.log 2>&1 &

# 监控日志
tail -f trading.log

# 停止
kill $(pgrep trading-engine)
```

---

## 🔍 常用命令

### 开发

```bash
# 编译
cargo build

# 运行
cargo run

# 测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

### 日志级别

```bash
# 只看重要信息 (推荐)
RUST_LOG=info cargo run

# 查看调试信息 (数据量大)
RUST_LOG=debug cargo run

# 只看错误
RUST_LOG=error cargo run
```

### 监控

```bash
# 查看进程
ps aux | grep trading

# 查看内存
top -p $(pgrep trading-engine)

# 查看网络连接
netstat -an | grep 9443
```

---

## 📊 系统架构

```
用户
 │
 └─> TradingEngine (主引擎)
      │
      ├─> WebSocket ─────> 币安实时数据
      ├─> Strategy ──────> 双均线策略
      ├─> RiskManager ───> 风控检查
      ├─> BinanceClient ─> REST API (下单)
      └─> TelegramAlerter > Telegram 通知
```

---

## 🛠️ 开发状态

### Phase 1 MVP (当前)

**完成度**: 11/12 任务 (92%)

**已完成**:
- ✅ 项目基础设施
- ✅ 配置系统
- ✅ 日志系统
- ✅ WebSocket 连接
- ✅ OrderBook 数据结构
- ✅ Binance REST API
- ✅ 技术指标库
- ✅ 双均线策略
- ✅ 风控管理器
- ✅ Telegram 告警
- ✅ 主交易循环集成

**待完成**:
- ⏳ Task 12: 端到端测试

### Phase 2 (计划中)

- Pine Script DSL 解析器
- 更多技术指标
- Freqtrade 策略转换器
- 策略热加载

### Phase 3 (计划中)

- Redis 集成
- AWS Timestream
- Web Dashboard

---

## ⚠️ 注意事项

1. **仅用于测试**: 当前使用 Binance Testnet，无真实资金风险
2. **配置文件安全**: `config.local.toml` 包含密钥，已加入 `.gitignore`
3. **首次运行**: 策略需要积累 20+ 价格数据点 (约 20 分钟)
4. **网络要求**: 需要稳定访问 binance.com 和 api.telegram.org

---

## 📞 问题反馈

测试过程中如有问题，请填写 [TEST_RESULTS.md](../docs/TEST_RESULTS.md)

---

## 📜 许可证

待定

---

**最后更新**: 2025-01-23
**版本**: v0.1.0-mvp
