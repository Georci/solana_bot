use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    message::v0::MessageAddressTableLookup, signer::Signer, transaction::VersionedTransaction,
};
use std::collections::HashMap;
use std::fmt::Formatter;

// 某个钱包与持有代币的相关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTradeStats {
    pub symbol: String,     // 代币的名称
    pub token_mint: Pubkey, // 代币的 mint 地址

    pub total_bought: f64, // 总买入量
    pub total_sold: f64,   // 总卖出量
    pub net_position: f64, // 净持仓(= total_bought - total_sold)，也可用其他方式表示

    pub bought_time: Vec<u64>, // 买入时间
    pub sold_time: Vec<u64>,   // 卖出时间

    pub profit: f64,     // 盈亏（可能是价格差累积计算）
    pub win_count: u32,  // 盈利交易笔数
    pub lose_count: u32, // 亏损交易笔数
}

impl TokenTradeStats {
    /// 创建一个新的 TokenTradeStats，初始值全为 0 或空
    pub fn new(token_mint: Pubkey) -> Self {
        Self {
            symbol: "".to_string(),
            token_mint,
            total_bought: 0.0,
            total_sold: 0.0,
            bought_time: vec![],
            sold_time: vec![],
            net_position: 0.0,
            profit: 0.0,
            win_count: 0,
            lose_count: 0,
        }
    }

    /// 记录一次买入操作
    /// - `amount` 买入数量
    /// - `timestamp` 发生时间
    pub fn record_buy(&mut self, amount: f64, timestamp: u64) {
        self.total_bought += amount;
        self.net_position += amount;
        self.bought_time.push(timestamp);
    }

    /// 记录一次卖出操作
    /// - `amount` 卖出数量
    /// - `timestamp` 发生时间
    /// - `profit_change` 本次卖出带来的盈亏变化（可能是已实现盈亏）
    pub fn record_sell(&mut self, amount: f64, timestamp: u64, profit_change: f64) {
        self.total_sold += amount;
        self.net_position -= amount;
        self.sold_time.push(timestamp);

        // 更新当前总盈利
        self.profit += profit_change;
    }
}

impl std::fmt::Display for TokenTradeStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token: {}, Mint: {}\n  Total Bought: {:.2}, Total Sold: {:.2}, Net Position: {}\n  Profit: {:.2}, Win Count: {}, Lose Count: {}\n  Bought Times: {:?}\n  Sold Times: {:?}",
            self.symbol,
            self.token_mint,
            self.total_bought,
            self.total_sold,
            self.net_position,
            self.profit,
            self.win_count,
            self.lose_count,
            self.bought_time,
            self.sold_time,
        )
    }
}

// 与代币相关的相关交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRelatedTxs {
    pub signature: String,
}

impl TokenRelatedTxs {
    /// 创建一个新的 TokenRelatedTxs
    pub fn new(signature: &str) -> Self {
        Self {
            signature: signature.to_string(),
        }
    }
}

// UserInfo
#[derive(Debug, Clone)]
pub struct User {
    pub address: Pubkey,                               // 用户地址
    pub history_txs: Vec<String>,                      // 这段时间内用户所有的交易
    pub token_txs: Vec<String>,                        // 这段时间内与代币相关交易签名列表
    pub token_stats: HashMap<Pubkey, TokenTradeStats>, // 我们对代币的相关信息不停留在交易上，而关注这个钱包在一段时间内对某个代币的买卖
    pub distinct_token_count: u8,                      // 当前账户一段时间内的买卖代币总数
    pub time_day: u8,                                  // 时间期限, 以“天”为单位
    pub total_profit: f64,                             // 这段时间内总的盈利

    pub score: i64, // 最终评分
}

impl User {
    /// 创建一个新的 User
    pub fn new(address: Pubkey, time_day: u8) -> Self {
        Self {
            address,
            history_txs: vec![],
            token_txs: vec![],
            token_stats: HashMap::new(),
            distinct_token_count: 0,
            time_day,
            total_profit: 0.0,
            score: 0,
        }
    }

    /// 添加一条与代币相关的交易签名到历史列表
    pub fn add_history_tx(&mut self, signature: &str) {
        self.history_txs.push(signature.parse().unwrap());
    }

    /// 记录买入某种代币
    /// - `token_mint` 代币 mint
    /// - `amount` 买入数量
    /// - `timestamp` 买入发生时间（区块时间或自定义）
    pub fn buy_token(&mut self, token_mint: Pubkey, amount: f64, timestamp: u64) {
        // 如果没有对应的 TokenTradeStats，就创建一个
        let entry = self
            .token_stats
            .entry(token_mint)
            .or_insert_with(|| TokenTradeStats::new(token_mint));

        // 更新记录
        entry.record_buy(amount, timestamp);

        // 如果你想把这次买入也算在 total_profit 里，需要有个盈亏逻辑。
        // 这里只是演示，所以暂时不处理 profit 变动。

        // 更新 token_amount（假设它是记录所有买卖过的代币种类总数？还是数量？看你需求）
        // 如果 token_amount 指的是“代币种类数”，那么这里要判断 token_mint 是否是第一次出现
        // 如果指的是“买卖总笔数”或“买卖总量”，可做相应增量。
        self.distinct_token_count += 1;
    }

    /// 记录卖出某种代币
    /// - `profit_change` 本次卖出带来的盈亏变化（正数/负数都可能）
    pub fn sell_token(
        &mut self,
        token_mint: Pubkey,
        amount: f64,
        timestamp: u64,
        profit_change: f64,
    ) {
        let entry = self
            .token_stats
            .entry(token_mint)
            .or_insert_with(|| TokenTradeStats::new(token_mint));

        entry.record_sell(amount, timestamp, profit_change);

        // 更新本用户的 total_profit
        self.total_profit += profit_change;

        // 同理，如果 token_amount 代表“卖出次数”或者“参与交易次数”，也可在此更新
        self.distinct_token_count += 1;
    }

    // 给当前user进行打分，具体的打分策略我还没没想好
    pub fn score(&self) {}

    /// 输出当前用户的汇总信息
    pub fn display_with_filter(&self, filter_addresses: &[Pubkey]){
        println!("user address: {}", self.address);
        println!(
            "during the time: {}, \nuser's total transaction amount: {:?}, \ntx related to buying and selling tokens: {:?}",
            self.time_day, self.history_txs, self.token_txs
        );
        println!(
            "the number of tokens held by user: {}, \nuser's score: {}",
            self.distinct_token_count, self.score
        );

        println!("\nFiltered Token Stats:");
        for address in filter_addresses {
            if let Some(token_stat) = self.token_stats.get(address) {
                println!("Mint: {}\n{}", address, token_stat);
            }
        }
    }

}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "user address: {}", self.address).expect("TODO: panic message");
        write!(f,
               "\nduring the time: {}, \nuser's total transaction amount: {:?}, \ntx related to buying and selling tokens is:{:?}",
               self.time_day, self.history_txs, self.token_txs
        ).expect("TODO: panic message");
        write!(
            f,
            "\nthe number of tokens held by user:{}, \nthe user's score:{}",
             self.distinct_token_count, self.score
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display_user() {
        let address: Pubkey = Pubkey::new_from_array([1; 32]);
        let user = User::new(address, 120);

        println!("{}", user);
    }

    #[test]
    fn test_display_with_filter() {
        let address: Pubkey = Pubkey::new_from_array([1; 32]);
        let user = User::new(address, 120);
        let filter_addresses = vec![];

        user.display_with_filter(&filter_addresses);
    }
}
