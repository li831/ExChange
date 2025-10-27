#!/usr/bin/env python3
"""
技术指标验证脚本 - 增强版
- 支持8位小数精度（加密货币标准）
- 生成大量测试数据（100-10000点）
- 模拟真实市场场景
"""

import json
import sys
from pathlib import Path
from decimal import Decimal, ROUND_HALF_UP

try:
    import numpy as np
    import pandas as pd
except ImportError:
    print("错误: 需要安装 numpy 和 pandas")
    print("运行: pip install numpy pandas")
    sys.exit(1)

# 检查是否安装了 pandas-ta
try:
    import pandas_ta as ta
    HAS_PANDAS_TA = True
except ImportError:
    print("警告: pandas-ta 未安装，将使用内置实现")
    print("推荐安装: pip install pandas-ta")
    HAS_PANDAS_TA = False

# 加密货币价格精度：8位小数
PRICE_PRECISION = 8

def round_price(price):
    """将价格四舍五入到8位小数"""
    if isinstance(price, (list, np.ndarray)):
        return [round_price(p) for p in price]
    d = Decimal(str(price))
    return float(d.quantize(Decimal('0.00000001'), rounding=ROUND_HALF_UP))


def calculate_sma_builtin(prices, period):
    """内置 SMA 实现（无需 pandas-ta）"""
    if len(prices) < period:
        return []

    result = []
    for i in range(period - 1, len(prices)):
        window = prices[i - period + 1:i + 1]
        result.append(sum(window) / period)

    return result


def calculate_rsi_builtin(prices, period=14):
    """内置 RSI 实现（无需 pandas-ta）"""
    if len(prices) < period + 1:
        return []

    deltas = np.diff(prices)
    gains = np.where(deltas > 0, deltas, 0)
    losses = np.where(deltas < 0, -deltas, 0)

    result = []
    for i in range(period - 1, len(deltas)):
        avg_gain = np.mean(gains[i - period + 1:i + 1])
        avg_loss = np.mean(losses[i - period + 1:i + 1])

        if avg_loss == 0:
            rsi = 100.0
        else:
            rs = avg_gain / avg_loss
            rsi = 100.0 - (100.0 / (1.0 + rs))

        result.append(rsi)

    return result


def generate_random_walk(start_price, num_points, volatility=0.02, trend=0.0, seed=None):
    """
    生成随机游走价格序列（模拟真实市场）

    Args:
        start_price: 起始价格
        num_points: 数据点数量
        volatility: 波动率（标准差）
        trend: 趋势（正数=上涨，负数=下跌）
        seed: 随机种子（用于可重现测试）
    """
    if seed is not None:
        np.random.seed(seed)

    prices = [start_price]
    for _ in range(num_points - 1):
        change = np.random.normal(trend, volatility)
        new_price = prices[-1] * (1 + change)
        prices.append(round_price(new_price))

    return prices


def generate_sine_wave(start_price, num_points, amplitude=0.1, frequency=0.05):
    """
    生成正弦波价格序列（规律震荡）

    Args:
        start_price: 起始价格
        num_points: 数据点数量
        amplitude: 振幅（相对于起始价格的百分比）
        frequency: 频率
    """
    prices = []
    for i in range(num_points):
        price = start_price * (1 + amplitude * np.sin(2 * np.pi * frequency * i))
        prices.append(round_price(price))

    return prices


def generate_flash_crash(start_price, num_points, crash_point=0.5, crash_magnitude=0.3):
    """
    生成闪崩场景（突然暴跌后恢复）

    Args:
        start_price: 起始价格
        num_points: 数据点数量
        crash_point: 闪崩发生位置（0-1之间）
        crash_magnitude: 闪崩幅度（百分比）
    """
    crash_index = int(num_points * crash_point)
    recovery_index = crash_index + int(num_points * 0.1)

    prices = []
    for i in range(num_points):
        if i < crash_index:
            # 崩盘前：缓慢上涨
            price = start_price * (1 + 0.001 * i)
        elif i < recovery_index:
            # 闪崩阶段：快速下跌
            progress = (i - crash_index) / (recovery_index - crash_index)
            price = start_price * (1 - crash_magnitude * progress)
        else:
            # 恢复阶段：逐步回升
            progress = (i - recovery_index) / (num_points - recovery_index)
            price = start_price * (1 - crash_magnitude * (1 - progress))

        prices.append(round_price(price))

    return prices


def generate_test_data():
    """生成测试数据集 - 增强版"""

    # 基准价格（模拟 BTC）
    BTC_BASE = 43521.78654321

    test_cases = {
        # ========== 小数据集（用于手工验证）==========
        "manual_verification": {
            "description": "手工验证数据（简单计算）",
            "size": 5,
            "prices": [10.00000000, 20.00000000, 30.00000000, 40.00000000, 50.00000000]
        },

        # ========== 中等数据集（100点）==========
        "btc_uptrend_100": {
            "description": "BTC 上涨趋势（100点，8位精度）",
            "size": 100,
            "prices": generate_random_walk(BTC_BASE, 100, volatility=0.015, trend=0.002, seed=42)
        },
        "btc_downtrend_100": {
            "description": "BTC 下跌趋势（100点，8位精度）",
            "size": 100,
            "prices": generate_random_walk(BTC_BASE, 100, volatility=0.015, trend=-0.002, seed=43)
        },
        "btc_sideways_100": {
            "description": "BTC 横盘震荡（100点，8位精度）",
            "size": 100,
            "prices": generate_sine_wave(BTC_BASE, 100, amplitude=0.02, frequency=0.05)
        },
        "btc_volatile_100": {
            "description": "BTC 高波动（100点，8位精度）",
            "size": 100,
            "prices": generate_random_walk(BTC_BASE, 100, volatility=0.05, trend=0.0, seed=44)
        },

        # ========== 大数据集（1000点）==========
        "btc_trend_1000": {
            "description": "BTC 长期趋势（1000点，8位精度）",
            "size": 1000,
            "prices": generate_random_walk(BTC_BASE, 1000, volatility=0.02, trend=0.001, seed=45)
        },
        "btc_ranging_1000": {
            "description": "BTC 长期震荡（1000点，8位精度）",
            "size": 1000,
            "prices": generate_sine_wave(BTC_BASE, 1000, amplitude=0.05, frequency=0.01)
        },

        # ========== 超大数据集（5000点）- 性能测试 ==========
        "btc_longterm_5000": {
            "description": "BTC 超长期数据（5000点，性能测试）",
            "size": 5000,
            "prices": generate_random_walk(BTC_BASE, 5000, volatility=0.02, trend=0.0005, seed=46)
        },

        # ========== 极端场景 ==========
        "btc_flash_crash": {
            "description": "BTC 闪崩场景（500点）",
            "size": 500,
            "prices": generate_flash_crash(BTC_BASE, 500, crash_point=0.5, crash_magnitude=0.3)
        },
        "btc_parabolic": {
            "description": "BTC 抛物线上涨（200点）",
            "size": 200,
            "prices": [round_price(BTC_BASE * (1 + 0.01 * i * i / 100)) for i in range(200)]
        },

        # ========== 边界条件 ==========
        "tiny_prices": {
            "description": "极小价格（Altcoin，8位精度）",
            "size": 50,
            "prices": generate_random_walk(0.00012345, 50, volatility=0.03, seed=47)
        },
        "large_prices": {
            "description": "极大价格（8位精度）",
            "size": 50,
            "prices": generate_random_walk(123456.78901234, 50, volatility=0.02, seed=48)
        },

        # ========== StockCharts 参考数据（保留用于对比）==========
        "stockcharts_rsi": {
            "description": "StockCharts RSI 标准示例",
            "size": 20,
            "prices": [
                44.00000000, 44.34000000, 44.09000000, 43.61000000, 44.33000000,
                44.83000000, 45.10000000, 45.42000000, 45.84000000, 46.08000000,
                45.89000000, 46.03000000, 45.61000000, 46.28000000, 46.28000000,
                46.00000000, 46.03000000, 46.41000000, 46.22000000, 45.64000000
            ]
        }
    }

    return test_cases


def calculate_indicators(prices, use_pandas_ta=False):
    """计算所有指标"""
    results = {}

    if use_pandas_ta and HAS_PANDAS_TA:
        # 使用 pandas-ta
        df = pd.DataFrame({'close': prices})

        # SMA
        results['sma_5'] = df.ta.sma(length=5, close='close').dropna().tolist()
        results['sma_10'] = df.ta.sma(length=10, close='close').dropna().tolist()
        results['sma_20'] = df.ta.sma(length=20, close='close').dropna().tolist()

        # RSI
        results['rsi_5'] = df.ta.rsi(length=5, close='close').dropna().tolist()
        results['rsi_14'] = df.ta.rsi(length=14, close='close').dropna().tolist()

        results['source'] = 'pandas-ta'
    else:
        # 使用内置实现
        results['sma_5'] = calculate_sma_builtin(prices, 5)
        results['sma_10'] = calculate_sma_builtin(prices, 10)
        results['sma_20'] = calculate_sma_builtin(prices, 20)

        results['rsi_5'] = calculate_rsi_builtin(prices, 5)
        results['rsi_14'] = calculate_rsi_builtin(prices, 14)

        results['source'] = 'builtin'

    return results


def main():
    """主函数"""
    print("=" * 80)
    print(" " * 20 + "技术指标验证脚本 - 增强版")
    print("=" * 80)
    print(f"精度设置: {PRICE_PRECISION} 位小数（加密货币标准）")
    print(f"数据源: {'pandas-ta' if HAS_PANDAS_TA else '内置实现'}")

    # 生成测试数据
    print("\n" + "=" * 80)
    print("步骤 1/3: 生成测试数据")
    print("=" * 80)

    test_cases = generate_test_data()
    total_cases = len(test_cases)
    total_points = sum(tc.get('size', len(tc['prices'])) for tc in test_cases.values())

    print(f"\n测试用例统计:")
    print(f"  - 总用例数: {total_cases}")
    print(f"  - 总数据点: {total_points:,}")
    print(f"  - 平均每用例: {total_points // total_cases:,} 点")

    print(f"\n数据集分布:")
    small = sum(1 for tc in test_cases.values() if tc.get('size', len(tc['prices'])) < 100)
    medium = sum(1 for tc in test_cases.values() if 100 <= tc.get('size', len(tc['prices'])) < 1000)
    large = sum(1 for tc in test_cases.values() if tc.get('size', len(tc['prices'])) >= 1000)
    print(f"  - 小数据集 (<100点):    {small} 个")
    print(f"  - 中数据集 (100-999点):  {medium} 个")
    print(f"  - 大数据集 (>=1000点):   {large} 个")

    # 计算指标
    print("\n" + "=" * 80)
    print("步骤 2/3: 计算参考指标值")
    print("=" * 80)

    reference_values = {}
    processed = 0

    for name, test_case in test_cases.items():
        processed += 1
        size = test_case.get('size', len(test_case['prices']))
        print(f"\n[{processed}/{total_cases}] {test_case['description']}")
        print(f"      数据点: {size:,} | ", end='')

        prices = test_case['prices']

        # 显示价格范围
        min_price = min(prices)
        max_price = max(prices)
        print(f"价格范围: {min_price:.8f} - {max_price:.8f}")

        # 计算指标
        indicators = calculate_indicators(prices, use_pandas_ta=HAS_PANDAS_TA)

        reference_values[name] = {
            'description': test_case['description'],
            'size': size,
            'prices': prices,
            'price_stats': {
                'min': round_price(min_price),
                'max': round_price(max_price),
                'mean': round_price(np.mean(prices)),
                'std': round_price(np.std(prices))
            },
            'indicators': indicators
        }

    # 保存到文件
    print("\n" + "=" * 80)
    print("步骤 3/3: 保存参考值")
    print("=" * 80)

    output_dir = Path(__file__).parent.parent / "tests" / "fixtures"
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / "indicator_reference.json"

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(reference_values, f, indent=2, ensure_ascii=False)

    file_size = output_file.stat().st_size / 1024  # KB
    print(f"\n保存位置: {output_file}")
    print(f"文件大小: {file_size:.2f} KB")

    # 验证示例
    print("\n" + "=" * 80)
    print("验证示例：手工计算对比")
    print("=" * 80)

    manual = reference_values['manual_verification']
    print(f"\n价格序列: {manual['prices'][:5]}")
    print(f"\nSMA(3) 手工计算:")
    print(f"  - 索引 2: (10+20+30)/3 = 20.00000000")
    print(f"  - 索引 3: (20+30+40)/3 = 30.00000000")
    print(f"  - 索引 4: (30+40+50)/3 = 40.00000000")

    print(f"\nSMA(5) 程序计算:")
    if manual['indicators']['sma_5']:
        print(f"  - 结果: {manual['indicators']['sma_5']}")
    else:
        print(f"  - （数据不足，无结果）")

    # 总结
    print("\n" + "=" * 80)
    print("生成完成")
    print("=" * 80)
    print(f"\n测试总结:")
    print(f"  [OK] {total_cases} 个测试用例")
    print(f"  [OK] {total_points:,} 个数据点")
    print(f"  [OK] 8位小数精度")
    print(f"  [OK] 包含极端场景测试")
    print(f"\n下一步:")
    print(f"  运行: cd trading-engine && cargo test --test indicator_validation_test")
    print("=" * 80)


if __name__ == "__main__":
    main()
