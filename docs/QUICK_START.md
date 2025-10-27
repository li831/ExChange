# 交易引擎快速启动指南

## 🚀 5 分钟快速测试

### 步骤 1: 编译项目

```bash
cd trading-engine
cargo build --release
```

### 步骤 2: 测试 Telegram 连接

```bash
cargo run --example test_telegram
```

**预期**: 您的 Telegram 应该收到 2 条消息

### 步骤 3: 运行交易引擎

```bash
RUST_LOG=info cargo run
```

**预期输出**:
```
INFO trading_engine: 🚀 Trading Engine Starting
INFO trading_engine: Environment: testnet
INFO trading_engine::websocket: WebSocket connected successfully
INFO trading_engine::engine: 📡 Subscribed to market data streams
INFO trading_engine::engine: 📊 BTCUSDT - Price: 95234.56 | Buffer: 10/100 | Total received: 100
INFO trading_engine::engine: 🔍 Checking strategy signals...
INFO trading_engine::engine: ⏳ BTCUSDT: Accumulating data... (10/20 required)
```

### 步骤 4: 观察运行

让程序运行至少 **30 分钟**, 观察:

1. **价格数据**: 每隔几秒看到采样的价格更新 (格式: `📊 SYMBOL - Price: xxx | Buffer: x/100 | Total received: xxx`)
2. **数据积累**: 前 20 分钟会看到 `⏳ Accumulating data...` 进度提示
3. **策略检查**: 每 60 秒会检查一次策略，显示 `🔍 Checking strategy signals...`
4. **MA 分析**: 数据积累足够后，会显示 `📐 MA Analysis` 详细计算过程
5. **Telegram 通知**: 如果生成交易信号，会收到通知

### 步骤 5: 停止程序

按 `Ctrl+C` 停止

---

## 📋 关键命令速查

### 运行测试
```bash
cargo test
```

### 查看详细日志 (仅用于深度调试)
```bash
RUST_LOG=debug cargo run
```

**注意**: info 级别已足够查看价格数据、策略分析等信息，debug 会产生大量日志

### 后台运行
```bash
nohup cargo run --release > trading.log 2>&1 &
```

### 查看日志
```bash
tail -f trading.log
```

### 停止后台进程
```bash
# 查找进程 ID
ps aux | grep trading-engine

# 停止进程
kill <PID>
```

---

## 🔍 日志级别说明

- `RUST_LOG=error` - 只显示错误
- `RUST_LOG=warn` - 显示警告和错误
- `RUST_LOG=info` - 显示信息、警告、错误 **(推荐用于生产和测试)**
  - 包含采样的价格数据 (1:10 采样，避免日志过多)
  - 包含数据积累进度
  - 包含完整的 MA 计算分析
  - 包含交易信号和执行信息
- `RUST_LOG=debug` - 显示所有调试信息 (数据量极大，仅用于深度调试)
- `RUST_LOG=trace` - 显示最详细的信息 (通常不需要)

---

## 📊 检查系统状态

### 查看进程
```bash
ps aux | grep trading
```

### 查看内存使用
```bash
top -p $(pgrep trading-engine)
```

### 查看网络连接
```bash
netstat -an | grep ESTABLISHED | grep 9443
```

---

## ⚠️ 注意事项

1. **确保配置文件存在**: `config.local.toml` 必须包含正确的 API 密钥
2. **网络连接**: 确保能访问 binance.com 和 api.telegram.org
3. **首次运行**: 策略需要积累至少 20 个价格数据点才会开始工作 (约 20 分钟)
4. **Testnet 密钥**: 请使用 Binance Testnet 的密钥，不要使用真实账户密钥

---

详细测试步骤请参考: [TESTING_GUIDE.md](./TESTING_GUIDE.md)
