use std::fmt::Display;
use std::ptr::write;

/// 跟单策略
#[derive(Debug, Clone)]
pub struct CopyTradeStrategy {
    pub allocate_funds: f64, // 分配给当前聪明钱包最大可跟单金额
    pub follow_ratio: f64,   // 跟单买入的比例（0.5 表示：聪明钱包买 100 token，我买 50）

    pub per_position_funds: f64, // 每一个仓位跟单多少(百分数，具体跟单金额：allocate_fund * per_position_funds)
    pub slippage: f64,           // 滑点
    pub fee_rate: f64,           // 手续费比例
}

impl CopyTradeStrategy {
    pub fn new(
        follow_ratio: f64,
        allocate_funds: f64,
        per_position_funds: f64,
        slippage: f64,
        fee_rate: f64,
    ) -> Self {
        Self {
            allocate_funds,
            follow_ratio,
            per_position_funds,
            slippage,
            fee_rate,
        }
    }

    pub fn get_default_strategy() -> Self {
        Self {
            allocate_funds: 0.1,      // 0.1 sol
            follow_ratio: 0.5,        // 按照50%的比例去跟单
            per_position_funds: 0.05, // 每个仓位跟单最大金额的5%
            slippage: 0.0,
            fee_rate: 0.0,
        }
    }
}

impl Display for CopyTradeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "this wallet has been allocate funds:{}, follow ratio: {}, per position funds: {}, silppage:{}, fee_rate:{}",
               self.allocate_funds,
            self.follow_ratio,
            self.per_position_funds,
            self.slippage,
            self.fee_rate
        )
    }
}
