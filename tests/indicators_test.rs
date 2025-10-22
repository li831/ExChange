use trading_engine::indicators::{SMA, RSI, Indicator};

#[test]
fn test_sma_creation() {
    let sma = SMA::new(5);
    assert_eq!(sma.period(), 5);
    assert!(!sma.is_ready());
}

#[test]
fn test_sma_calculation() {
    let mut sma = SMA::new(3);

    // Not ready until we have enough data points
    sma.update(10.0);
    assert!(!sma.is_ready());
    assert!(sma.value().is_none());

    sma.update(20.0);
    assert!(!sma.is_ready());

    sma.update(30.0);
    assert!(sma.is_ready());

    // SMA(3) of [10, 20, 30] = (10 + 20 + 30) / 3 = 20
    assert_eq!(sma.value(), Some(20.0));

    // Add another value: [20, 30, 40]
    sma.update(40.0);
    // SMA(3) of [20, 30, 40] = (20 + 30 + 40) / 3 = 30
    assert_eq!(sma.value(), Some(30.0));
}

#[test]
fn test_sma_reset() {
    let mut sma = SMA::new(3);
    sma.update(10.0);
    sma.update(20.0);
    sma.update(30.0);
    assert!(sma.is_ready());

    sma.reset();
    assert!(!sma.is_ready());
    assert!(sma.value().is_none());
}

#[test]
fn test_rsi_creation() {
    let rsi = RSI::new(14);
    assert_eq!(rsi.period(), 14);
    assert!(!rsi.is_ready());
}

#[test]
fn test_rsi_calculation_uptrend() {
    let mut rsi = RSI::new(3);

    // Simulate an uptrend
    let prices = vec![100.0, 105.0, 110.0, 115.0];

    for price in prices {
        rsi.update(price);
    }

    assert!(rsi.is_ready());
    let value = rsi.value().unwrap();

    // RSI should be high (above 50) in an uptrend
    assert!(value > 50.0, "RSI should be > 50 in uptrend, got {}", value);
    assert!(value <= 100.0, "RSI should be <= 100, got {}", value);
}

#[test]
fn test_rsi_calculation_downtrend() {
    let mut rsi = RSI::new(3);

    // Simulate a downtrend
    let prices = vec![115.0, 110.0, 105.0, 100.0];

    for price in prices {
        rsi.update(price);
    }

    assert!(rsi.is_ready());
    let value = rsi.value().unwrap();

    // RSI should be low (below 50) in a downtrend
    assert!(value < 50.0, "RSI should be < 50 in downtrend, got {}", value);
    assert!(value >= 0.0, "RSI should be >= 0, got {}", value);
}

#[test]
fn test_rsi_bounds() {
    let mut rsi = RSI::new(3);

    // Fill with initial prices
    for i in 0..10 {
        rsi.update(100.0 + i as f64);
    }

    assert!(rsi.is_ready());
    let value = rsi.value().unwrap();

    // RSI should always be between 0 and 100
    assert!(value >= 0.0 && value <= 100.0, "RSI out of bounds: {}", value);
}

#[test]
fn test_rsi_reset() {
    let mut rsi = RSI::new(3);

    for i in 0..5 {
        rsi.update(100.0 + i as f64);
    }

    assert!(rsi.is_ready());

    rsi.reset();
    assert!(!rsi.is_ready());
    assert!(rsi.value().is_none());
}
