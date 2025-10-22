pub mod dual_ma;

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Long,       // 买入
    Short,      // 卖出
    CloseLong,  // 平多仓
    CloseShort, // 平空仓
    None,       // 无操作
}

pub trait Strategy {
    fn generate_signal(&self, prices: &[f64]) -> Signal;
    fn name(&self) -> &str;
}
