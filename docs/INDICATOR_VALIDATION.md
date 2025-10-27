# 技术指标验证指南

## 📋 验证策略概览

技术指标验证需要确保实现的准确性和可靠性。本文档提供多层次的验证方法。

---

## 1. 数学公式验证 ✅

### 方法
对照标准公式定义，使用小数据集手工计算验证。

### SMA 验证示例

**公式**: SMA = (P₁ + P₂ + ... + Pₙ) / n

**测试数据**: [10, 20, 30, 40, 50]

**SMA(3) 手工计算**:
- 第3个值: (10 + 20 + 30) / 3 = 20.0 ✓
- 第4个值: (20 + 30 + 40) / 3 = 30.0 ✓
- 第5个值: (30 + 40 + 50) / 3 = 40.0 ✓

**Rust 测试代码**:
```rust
#[test]
fn test_sma_manual_verification() {
    let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let result = sma(&prices, 3);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], 20.0);  // (10+20+30)/3
    assert_eq!(result[1], 30.0);  // (20+30+40)/3
    assert_eq!(result[2], 40.0);  // (30+40+50)/3
}
```

### RSI 验证示例

**公式**: RSI = 100 - (100 / (1 + RS))
- RS = 平均涨幅 / 平均跌幅

**测试数据**: [44, 44.34, 44.09, 43.61, 44.33, 44.83]

**手工计算 RSI(5)**:
1. 价格变化: [+0.34, -0.25, -0.48, +0.72, +0.50]
2. 涨幅: [0.34, 0, 0, 0.72, 0.50]
3. 跌幅: [0, 0.25, 0.48, 0, 0]
4. 平均涨幅: (0.34 + 0.72 + 0.50) / 5 = 0.312
5. 平均跌幅: (0.25 + 0.48) / 5 = 0.146
6. RS = 0.312 / 0.146 = 2.137
7. RSI = 100 - (100 / (1 + 2.137)) = 68.1

---

## 2. 与权威库对比 🔬

### 推荐对比库

1. **TA-Lib** (Technical Analysis Library)
   - 金融领域事实标准
   - Python、C、Java 等多语言支持
   - 150+ 指标

2. **pandas-ta** (Pandas Technical Analysis)
   - 基于 Pandas 的现代实现
   - 易于使用，结果可靠

3. **TradingView Pine Script**
   - 业界广泛使用
   - 在线验证，可视化

### 验证流程

#### Step 1: 准备标准测试数据集

创建 `tests/fixtures/test_data.json`:
```json
{
  "btc_sample": {
    "symbol": "BTCUSDT",
    "interval": "1h",
    "data": [
      {"close": 44000.0, "timestamp": "2024-01-01T00:00:00Z"},
      {"close": 44340.0, "timestamp": "2024-01-01T01:00:00Z"},
      {"close": 44090.0, "timestamp": "2024-01-01T02:00:00Z"}
    ]
  }
}
```

#### Step 2: 使用 Python TA-Lib 生成参考值

```python
# scripts/generate_reference_values.py
import pandas as pd
import talib
import json

# 读取测试数据
with open('tests/fixtures/test_data.json') as f:
    data = json.load(f)

prices = [d['close'] for d in data['btc_sample']['data']]

# 计算指标
sma_5 = talib.SMA(prices, timeperiod=5)
rsi_14 = talib.RSI(prices, timeperiod=14)
macd, signal, hist = talib.MACD(prices)

# 保存参考值
reference = {
    'sma_5': sma_5.tolist(),
    'rsi_14': rsi_14.tolist(),
    'macd': {
        'macd': macd.tolist(),
        'signal': signal.tolist(),
        'histogram': hist.tolist()
    }
}

with open('tests/fixtures/reference_values.json', 'w') as f:
    json.dump(reference, f, indent=2)
```

#### Step 3: Rust 测试对比

```rust
#[test]
fn test_sma_against_talib() {
    // 加载测试数据和参考值
    let test_data = load_test_fixture("test_data.json");
    let reference = load_test_fixture("reference_values.json");

    let prices: Vec<f64> = test_data["btc_sample"]["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["close"].as_f64().unwrap())
        .collect();

    // 计算 SMA
    let result = sma(&prices, 5);

    // 对比参考值（允许浮点误差）
    let reference_sma = reference["sma_5"].as_array().unwrap();

    for (i, (our_val, ref_val)) in result.iter()
        .zip(reference_sma.iter())
        .enumerate()
    {
        let ref_f64 = ref_val.as_f64().unwrap();
        let diff = (our_val - ref_f64).abs();
        assert!(
            diff < 1e-6,
            "SMA mismatch at index {}: ours={}, reference={}, diff={}",
            i, our_val, ref_f64, diff
        );
    }
}
```

---

## 3. TradingView 在线验证 🌐

### 优势
- 可视化结果
- 业界标准
- 易于验证趋势和关键点

### 步骤

1. **准备数据**: 导出实际交易数据
2. **TradingView 验证**:
   - 访问 https://www.tradingview.com/chart/
   - 加载相同数据
   - 添加指标 (SMA, RSI, MACD, etc.)
   - 记录关键点位的指标值

3. **对比验证**:
   - 比对相同时间点的值
   - 验证交叉信号时机
   - 检查极值点

### 示例验证表

| 时间点 | 收盘价 | SMA(20) Rust | SMA(20) TV | 差异 | 状态 |
|--------|--------|--------------|------------|------|------|
| 2024-01-01 12:00 | 44000 | 43850.5 | 43850.48 | 0.02 | ✅ |
| 2024-01-01 13:00 | 44200 | 43920.3 | 43920.31 | 0.01 | ✅ |

---

## 4. 边界条件测试 🔍

### 必须测试的情况

```rust
#[test]
fn test_sma_edge_cases() {
    // 1. 空数据
    assert_eq!(sma(&[], 5), vec![]);

    // 2. 数据不足
    assert_eq!(sma(&[1.0, 2.0], 5), vec![]);

    // 3. 周期为1
    assert_eq!(sma(&[1.0, 2.0, 3.0], 1), vec![1.0, 2.0, 3.0]);

    // 4. 负数价格
    let result = sma(&[-1.0, -2.0, -3.0], 2);
    assert_eq!(result[0], -1.5);

    // 5. 极大值
    let large = vec![f64::MAX / 10.0; 5];
    let result = sma(&large, 3);
    assert!(result[0].is_finite());

    // 6. 极小值
    let small = vec![f64::MIN / 10.0; 5];
    let result = sma(&small, 3);
    assert!(result[0].is_finite());

    // 7. NaN 处理
    let with_nan = vec![1.0, f64::NAN, 3.0];
    let result = sma(&with_nan, 2);
    // 定义 NaN 处理策略
}

#[test]
fn test_rsi_edge_cases() {
    // 1. 所有价格相同（无波动）
    let mut rsi = RSI::new(14);
    for _ in 0..20 {
        rsi.update(100.0);
    }
    // RSI 应该是 NaN 或 50.0（取决于实现）

    // 2. 只涨不跌
    let mut rsi = RSI::new(5);
    for i in 0..10 {
        rsi.update(100.0 + i as f64);
    }
    assert_eq!(rsi.value().unwrap(), 100.0);

    // 3. 只跌不涨
    let mut rsi = RSI::new(5);
    for i in 0..10 {
        rsi.update(100.0 - i as f64);
    }
    assert_eq!(rsi.value().unwrap(), 0.0);
}
```

---

## 5. 性能基准测试 ⚡

### 目标
Phase 2 计划要求: **1000 数据点 < 1ms**

### 基准测试代码

```rust
// benches/indicator_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trading_engine::indicators::{sma, SMA, RSI, Indicator};

fn benchmark_sma_stateless(c: &mut Criterion) {
    let prices: Vec<f64> = (0..1000).map(|i| 100.0 + i as f64 * 0.1).collect();

    c.bench_function("sma_1000_points", |b| {
        b.iter(|| sma(black_box(&prices), black_box(20)))
    });
}

fn benchmark_sma_stateful(c: &mut Criterion) {
    c.bench_function("sma_update_1000_times", |b| {
        b.iter(|| {
            let mut indicator = SMA::new(20);
            for i in 0..1000 {
                indicator.update(black_box(100.0 + i as f64 * 0.1));
            }
            indicator.value()
        })
    });
}

criterion_group!(benches, benchmark_sma_stateless, benchmark_sma_stateful);
criterion_main!(benches);
```

**运行基准测试**:
```bash
cargo install criterion
cargo bench
```

---

## 6. 回归测试 🔄

### 目的
确保代码更改不会破坏已验证的指标。

### 实施方法

1. **创建黄金数据集**:
   - 一次性验证后，保存结果作为"黄金标准"
   - 存储在 `tests/fixtures/golden/`

2. **自动化回归测试**:
```rust
#[test]
fn test_sma_regression() {
    let golden = load_golden_data("sma_golden.json");

    for test_case in golden["test_cases"].as_array().unwrap() {
        let prices = extract_prices(test_case);
        let period = test_case["period"].as_u64().unwrap() as usize;
        let expected = extract_expected(test_case);

        let result = sma(&prices, period);

        assert_eq!(result.len(), expected.len());
        for (i, (r, e)) in result.iter().zip(expected.iter()).enumerate() {
            assert!(
                (r - e).abs() < 1e-10,
                "Regression failure at index {}", i
            );
        }
    }
}
```

---

## 7. 集成测试 🔗

### 多指标组合验证

```rust
#[test]
fn test_bollinger_bands_integration() {
    // 布林带 = SMA ± (StdDev * 倍数)
    let prices = vec![/* ... */];

    let middle = sma(&prices, 20);
    let stddev = calculate_stddev(&prices, 20);
    let upper = middle.iter().zip(stddev.iter())
        .map(|(m, s)| m + 2.0 * s)
        .collect::<Vec<_>>();
    let lower = middle.iter().zip(stddev.iter())
        .map(|(m, s)| m - 2.0 * s)
        .collect::<Vec<_>>();

    // 验证：价格应该大部分时间在带内
    let prices_in_band = prices.iter()
        .zip(upper.iter())
        .zip(lower.iter())
        .filter(|((p, u), l)| **p >= **l && **p <= **u)
        .count();

    let percentage = prices_in_band as f64 / prices.len() as f64;
    assert!(percentage > 0.95, "约95%的价格应在±2σ范围内");
}
```

---

## 8. 实战验证建议 💡

### 验证优先级

**必须做** (Phase 2 Task 4):
1. ✅ 数学公式手工验证（小数据集）
2. ✅ 与 TA-Lib 对比（标准数据集）
3. ✅ 边界条件测试
4. ✅ 性能基准测试

**推荐做** (Phase 2 后期):
5. ✅ TradingView 可视化验证（关键指标）
6. ✅ 回归测试套件
7. ✅ 集成测试（多指标组合）

**可选** (Phase 3):
8. ✅ 实盘数据验证
9. ✅ 压力测试（极端市场条件）
10. ✅ A/B 测试（不同实现对比）

---

## 9. 验证清单 ✓

### 每个指标必须通过

- [ ] 数学公式验证（手工计算3+个数据点）
- [ ] TA-Lib 对比（误差 < 1e-6）
- [ ] 边界条件测试（至少7种情况）
- [ ] 性能测试（1000点 < 1ms）
- [ ] TradingView 抽查（3+个关键时间点）
- [ ] 回归测试（黄金数据集）
- [ ] 文档完整（公式、参数、示例）

---

## 10. 常见问题

### Q: 浮点数比较如何处理？
A: 使用 epsilon (1e-6 ~ 1e-10) 进行近似比较:
```rust
assert!((a - b).abs() < 1e-6);
```

### Q: TA-Lib 和 TradingView 结果不一致怎么办？
A:
1. 检查数据对齐（时区、时间戳）
2. 确认参数一致（周期、平滑方法）
3. 查阅各自文档确认算法差异
4. 选择一个作为标准，记录差异原因

### Q: 性能测试不达标怎么办？
A:
1. 使用 `cargo flamegraph` 分析瓶颈
2. 考虑缓存中间结果
3. 使用滑动窗口而非重新计算
4. 必要时使用 SIMD 优化

---

## 参考资源

- [TA-Lib 文档](https://ta-lib.org/function.html)
- [TradingView Pine Script](https://www.tradingview.com/pine-script-docs/)
- [Investopedia 指标定义](https://www.investopedia.com/terms/t/technicalindicator.asp)
- [StockCharts 技术分析](https://school.stockcharts.com/doku.php?id=technical_indicators)

---

**最后更新**: 2025-01-27
**相关文档**: Phase 2 实施计划, TESTING_GUIDE.md
