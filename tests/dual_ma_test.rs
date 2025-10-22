use trading_engine::strategy::{dual_ma::DualMAStrategy, Signal, Strategy};

#[test]
fn test_dual_ma_bullish_crossover() {
    let strategy = DualMAStrategy::new(2, 3);

    // 模拟快线向上穿越慢线的价格序列
    // 使用简单数字确保交叉
    let prices = vec![
        1.0, 1.0, 1.0,  // 慢线和快线都是1.0
        1.0, 2.0,        // 快线开始上升
    ];

    let signal = strategy.generate_signal(&prices);
    assert_eq!(signal, Signal::Long);
}

#[test]
fn test_dual_ma_bearish_crossover() {
    let strategy = DualMAStrategy::new(2, 3);

    // 模拟快线向下穿越慢线的价格序列
    // 使用简单数字确保交叉
    let prices = vec![
        3.0, 3.0, 3.0,  // 慢线和快线都是3.0
        3.0, 1.0,        // 快线开始下降
    ];

    let signal = strategy.generate_signal(&prices);
    assert_eq!(signal, Signal::Short);
}

#[test]
fn test_dual_ma_no_signal() {
    let strategy = DualMAStrategy::new(3, 5);

    // 横盘整理
    let prices = vec![5.0, 5.1, 4.9, 5.0, 5.1];

    let signal = strategy.generate_signal(&prices);
    assert_eq!(signal, Signal::None);
}
