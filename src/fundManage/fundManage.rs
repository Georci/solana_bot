use crate::error::Error;
use crate::fundManage::strategy::CopyTradeStrategy;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fmt::Display;
use crate::states::states::User;

/**
 *  资金管理模块
 *  当前模块负责：
 *  1.管理当前配置账户的资金
 *  2.设置跟单策略(比如说分给聪明钱包账户多少可跟单资金，每一个仓位跟单跟多少资金)
 *  3.监控聪明钱包的行为(当监控到买入行为，则立刻买入，监控到卖出行为，则立刻卖出)
 *  3.在跟单的时候需要告诉调用交易的api，具体要买什么代币，买多少
 */


/// todo!: 现在没做的点：1.实时监控的结构还没有设计 2.事件触发的结构也没设计 3.对每个仓位应该做风险控制

/// 跟单事件
#[derive(Debug, Clone)]
pub enum FollowEvent {
    Buy {
        token: Pubkey,
        amount: f64,
        price: f64,
    },
    Sell {
        token: Pubkey,
        amount: f64,
        price: f64,
    },
}

/// 资金管理模块的配置
//账户的配置
#[derive(Debug, Clone)]
pub struct UserConfig {
    pub address: Pubkey,     // 需要进行跟单的账户地址
    pub private_key: String, // 进行跟单的账户私钥
    pub owner: String,       // 跟单人姓名

    pub smart_wallets: Vec<SmartWallet>, // 当前跟单的聪明钱包
    pub copy_wallets_amount: u8,         // 当前跟单的聪明钱包数量

    pub total_copy_funds: f64, // 已经跟单的总金额账户的总资金(成本)
    pub total_profit: f64,     // 当前跟单总的盈利
    pub balance_change: f64,   // 当前已经跟单的利润变化(总的盈利/总的成本)

    pub cost_limit: f64, // 最大跟单金额
}

impl UserConfig {
    pub fn new(address: Pubkey, pk: String, owner: String, smart_wallets: Vec<SmartWallet>, cost_limit: f64) -> Self {
        let mut copy_wallets_amount:u8 = 0;
        let mut total_copy_funds:f64 = 0.0;

        for smart_wallet in smart_wallets.iter() {
            copy_wallets_amount += 1;
            total_copy_funds += smart_wallet.allocate_funds;
        }
        Self {
            address,
            private_key: pk,
            owner,
            smart_wallets,
            copy_wallets_amount,
            total_copy_funds,
            total_profit: 0.0,
            balance_change: 0.0,
            cost_limit,
        }
    }

    pub fn copy_trade(&mut self) -> Result<(), Error> {
        // 这里应该是监控这些钱包

        // 监控到这些钱包的行为之后执行具体的操作(包括构造交易什么的)


        Ok(())
    }


}

impl Display for UserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
               "{}, your wallet address:{}, \nyou have already copy {} smart wallet, \ntotal copy funds: {}, \n total profit: {}, \nyour wallet balance change: {}, \nyour cost limit: {}",
               self.owner,
               self.address,
            self.copy_wallets_amount,
            self.total_copy_funds,
            self.total_profit,
            self.balance_change,
            self.cost_limit
        )
    }
}

/// 聪明钱包(我现在想的是只有在states的User结构体中在打分评选之后的高分钱包，才能被认定成聪明钱包)
/// 所以User本质上没啥作用，他的字段都应该只是为了选举出聪明钱包罢了，真正有用的是SmartWallet
#[derive(Debug, Clone)]
pub struct SmartWallet {
    pub address: Pubkey, // 钱包地址
    pub score: f64,      // 钱包得分

    pub allocate_funds: f64, // 分配给当前聪明钱包最大可跟单金额
    pub positions: HashMap<Pubkey, Position>, // 该聪明钱包当前持有的仓位
    pub history_position: HashMap<Pubkey, HistoryPosition>, // 该聪明钱包自跟单以来已经平仓的仓位
    pub position_amount: u8, // 当前跟单该聪明钱包的仓位数量
    pub history_position_amount: u8, // 已经平仓的仓位数量

    pub strategy: CopyTradeStrategy, // 跟单当前聪明钱包的策略
}

impl SmartWallet {
    pub fn new(smart_wallet: User, allocate_funds: f64, strategy: CopyTradeStrategy) -> Self {
        let address = smart_wallet.address;
        let score = smart_wallet.score;
        
        Self{
            address,
            score,
            allocate_funds,
            positions: Default::default(),
            history_position: Default::default(),
            position_amount: 0,
            history_position_amount: 0,
            strategy,
        }
    }
}

impl Display for SmartWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\nsmart wallet: {}, \nsmart wallet score:{}, \nthe max fund you have allocated this smart wallet: {}, \ncopy this wallet:{} position, \nhistory position:{}, copy strategy:{}",
            self.address,
            self.score,
            self.allocate_funds,
            self.position_amount,
            self.history_position_amount,
            self.strategy
        )
    }
}

/// 仓位信息
#[derive(Debug, Clone)]
pub struct Position {
    pub token: Pubkey,        // 当前仓位持有的代币地址
    pub symbol: String,       // 当前仓位持有的代币名称
    pub amount: f64,          // 持有的代币数量
    pub price_per_token: f64, // 当前时间每个token价格(usd)

    pub profit: f64,         // 当前仓位的盈利
    pub cost: f64,           // 当前仓位的成本
    pub balance_change: f64, // 当前仓位的盈利百分比(总盈利/总成本)

    pub is_active: bool, // 当前仓位是否已经平仓
}

/// 历史仓位信息
#[derive(Debug, Clone)]
pub struct HistoryPosition {
    pub token: Pubkey,        // 仓位持有的代币地址
    pub symbol: String,       // 仓位持有的代币名称
    pub amount: f64,          // 平仓的代币数量
    pub price_per_token: f64, // 平仓时每个token价格(usd)

    pub profit: f64,         // 仓位的盈利
    pub cost: f64,           // 仓位的成本
    pub balance_change: f64, // 仓位的盈利百分比(总盈利/总成本)
}



#[cfg(test)]
mod tests {
    use super::*;

}
