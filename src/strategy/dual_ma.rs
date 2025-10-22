use super::{Signal, Strategy};
use crate::indicators::{is_crossover, is_crossunder, sma};
use tracing::{debug, info};

pub struct DualMAStrategy {
    fast_period: usize,
    slow_period: usize,
}

impl DualMAStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(
            fast_period < slow_period,
            "Fast period must be less than slow period"
        );

        Self {
            fast_period,
            slow_period,
        }
    }

    pub fn fast_period(&self) -> usize {
        self.fast_period
    }

    pub fn slow_period(&self) -> usize {
        self.slow_period
    }
}

impl Strategy for DualMAStrategy {
    fn generate_signal(&self, prices: &[f64]) -> Signal {
        // 需要足够的数据计算慢线
        if prices.len() < self.slow_period {
            debug!(
                "Insufficient data: {} < {}",
                prices.len(),
                self.slow_period
            );
            return Signal::None;
        }

        // 计算均线
        let fast_ma = sma(prices, self.fast_period);
        let slow_ma = sma(prices, self.slow_period);

        // 需要至少2个数据点来检测交叉
        if fast_ma.len() < 2 || slow_ma.len() < 2 {
            return Signal::None;
        }

        // 由于快线和慢线的长度不同，我们需要对齐它们
        // slow_ma的第一个值对应prices的第slow_period-1个索引
        // fast_ma的第一个值对应prices的第fast_period-1个索引
        // 所以slow_ma比fast_ma晚开始 (slow_period - fast_period) 个值

        let offset = self.slow_period - self.fast_period;
        if fast_ma.len() < offset + 2 {
            return Signal::None;
        }

        // 获取对齐后的最新两个MA值
        let fast_prev = fast_ma[fast_ma.len() - 2];
        let fast_curr = fast_ma[fast_ma.len() - 1];
        let slow_prev = slow_ma[slow_ma.len() - 2];
        let slow_curr = slow_ma[slow_ma.len() - 1];

        debug!(
            "MA values - Fast: {:.2} -> {:.2}, Slow: {:.2} -> {:.2}",
            fast_prev, fast_curr, slow_prev, slow_curr
        );

        // 检测交叉
        if is_crossover(&fast_prev, &fast_curr, &slow_prev, &slow_curr) {
            info!("🔼 Bullish crossover detected! Fast MA crossed above Slow MA");
            Signal::Long
        } else if is_crossunder(&fast_prev, &fast_curr, &slow_prev, &slow_curr) {
            info!("🔽 Bearish crossover detected! Fast MA crossed below Slow MA");
            Signal::Short
        } else {
            Signal::None
        }
    }

    fn name(&self) -> &str {
        "Dual Moving Average"
    }
}
