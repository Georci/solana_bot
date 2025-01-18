use crate::constant::{PUMP_PROGRAM_ID, RAYDIUM_PROGRAM_ID};
use crate::error::{Error, TxParseError};
use crate::states::states::*;
use crate::utils::analyze_utils::*;
use crate::utils::analyze_utils::{get_balance_change, is_target};
use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_config::{RpcBlockConfig, RpcTransactionConfig};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiAccountsList,
    UiTransactionEncoding,
};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

/**
 *  当前模块负责：
 *  1.获取数据库中的地址信息
 *  2.获取当前地址历史交易(过去一月/半月)
 *  3.分析出每个地址在过去一段时间的买入的token数量
 *  4.分析出每个地址在这段时间内的收益(真想跟meme的话，感觉不需要看长线)
 */

pub struct Collector {
    pub client: RpcClient,
    pub retry_delay: Duration,
    pub interval: u64,
}

impl Collector {
    // 初始化方法
    pub fn new(interval: u64) -> anyhow::Result<Arc<Self>> {
        let rc_self = Arc::new(Self {
            client: RpcClient::new(env::var("RPC_URL").expect("RPC_URL is not set")),
            retry_delay: Duration::new(1, 0), //1秒重试
            interval,
        });
        Ok(rc_self)
    }

    pub async fn get_history_tx(&self, user: &mut User, limit: usize) -> Result<(), Error> {
        // 配置查询参数
        let config = GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: Some(limit),
            commitment: Some(CommitmentConfig::confirmed()),
        };
        // 调用 solana_client 提供的官方方法
        let statuses = self
            .client
            .get_signatures_for_address_with_config(&user.address, config)
            .await
            .map_err(|err| {
                Error::GetHistoryTxError(format!(
                    "Failed to fetch signatures for address: {:?}",
                    err
                ))
            })?;

        // 提取 signature 字段
        let signatures: Vec<String> = statuses.into_iter().map(|s| s.signature).collect();

        user.history_txs = signatures;
        Ok(())
    }

    // 分析当前传入的交易中，与代币相关的交易(pump.fun、raydium)
    pub async fn get_token_txs(&self, user: &mut User) -> Result<(), Error> {
        let txs = user.clone().history_txs;

        // 使用getTranscation的rpc方法获取交易的具体信息
        for tx in txs.iter() {
            let signature = Signature::from_str(tx).unwrap();
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base58),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };
            // 直接阻塞调用
            let transaction_result = self
                .client
                .get_transaction_with_config(&signature, config)
                .await
                .map_err(|err| Error::GetHistoryTxError(format!("Failed: {:?}", err)))?;

            // 当前交易的所有account
            let Vtx = transaction_result.transaction.transaction.decode().unwrap();
            let accounts = Vtx.message.static_account_keys();

            // 如果交易中的accounts中存在pump.fun和raydium我们就认为这个交易是买卖代币的交易
            let result = is_target(accounts);
            if result {
                // 如果是买卖代币的交易的话，我想想能不能将具体买卖代币的种类与时间给分析出来
                user.token_txs.push(tx.clone());
            } else {
                continue;
            }
        }
        Ok(())
    }
}

// 连接数据库，获取地址信息
pub fn get_address() -> String {
    "address".to_string()
}

pub fn get_default_address() -> Pubkey {
    Pubkey::from_str("H356FzDuxvVShAGWRqtjR5D5efWdYM2eoazydG21Mgrk").unwrap()
}

#[derive(Deserialize, Debug)]
pub struct TransactionSignature {
    pub blockTime: Option<u64>,             // 时间戳
    pub confirmationStatus: Option<String>, // 确认状态
    pub err: Option<String>,                // 错误
    pub memo: Option<String>,               // 备注
    pub signature: String,                  // 交易签名
    pub slot: u64,                          // Slot
}

#[derive(Deserialize, Debug)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Vec<TransactionSignature>,
}

pub async fn get_token_amounts() -> Result<(), ()> {
    Ok(())
}

pub fn get_default_user_activities() -> Vec<Value> {
    let mut activities = vec![];
    let a = json!({
        "chain": "sol",
        "tx_hash": "5FbvLzqZkZXV6JasopbhP3E4n7tQqJVECbaHDv6hPLDeUpjhb81ot7RyZzrq7EJkKBhrmv2Zn7cQ28n4KNCP8aHq",
        "timestamp": 1736812854,
        "event_type": "buy",
        "token": {
            "address": "HjQpoSuGewTosqSAhwGKKhm6g2UhPuhgKdBcsnBHpump",
            "symbol": "DOLL",
            "logo": "https://dd.dexscreener.com/ds-data/tokens/solana/HjQpoSuGewTosqSAhwGKKhm6g2UhPuhgKdBcsnBHpump.png?size=lg&key=2579c8"
        },
        "token_amount": "5871603.62937500000000000000",
        "quote_amount": "9.90099009900000000000",
        "cost_usd": "1809.40594059225",
        "buy_cost_usd": null,
        "price_usd": "0.00030816214016775",
        "is_open_or_close": 1,
        "quote_token": {
            "token_address": "So11111111111111111111111111111111111111112",
            "name": "Wrapped SOL",
            "symbol": "WSOL",
            "decimals": 9,
            "logo": "https://s2.coinmarketcap.com/static/img/coins/64x64/16116.png"
        },
        "from_address": "",
        "to_address": ""
    });
    let b = json!({
        "chain": "sol",
                "tx_hash": "25YknJLhKyy9eQn3RR4EX2UAArjHpSsQH1mu2bo1dttpKitRCoz1WEM2GLABTJmqjGCbJg3R73TvPHEFfGYSsA6d",
                "timestamp": 1736703887,
                "event_type": "buy",
                "token": {
                    "address": "3aQdQoY3v6PbovJ9jJjw5hn95U21qLGYp8mJmnwfpump",
                    "symbol": "GHN",
                    "logo": "https://pump.mypinata.cloud/ipfs/QmZYPdtuWBV4BfWJA19qH5QU7HJE5NAPMCfhd3mNZftxtD"
                },
                "token_amount": "18703645.22863800000000000000",
                "quote_amount": "1.27438486300000000000",
                "cost_usd": "242.73208485561",
                "buy_cost_usd": null,
                "price_usd": "0.000012977795598411",
                "is_open_or_close": 1,
                "quote_token": {
                    "token_address": "So11111111111111111111111111111111111111111",
                    "name": "SOL",
                    "symbol": "SOL",
                    "decimals": 9,
                    "logo": "https://www.dextools.io/resources/tokens/logos/3/solana/So11111111111111111111111111111111111111112.jpg"
                },
                "from_address": "",
                "to_address": ""
    });
    let c = json!({
                "chain": "sol",
                "tx_hash": "3Mma1rNFhjWk9R3V9sb1U31zFpaGX41QnuPgScXNssRBV6HZ2RXCYovr5uL3rXgZcg9mCciqMWAE7JHJDmkKotMg",
                "timestamp": 1736442395,
                "event_type": "sell",
                "token": {
                    "address": "6xmiC8Gsp6i8owu3JMDpt38vsCGznCmW5Fzjuomqpump",
                    "symbol": "CHEETAH",
                    "logo": "https://pump.mypinata.cloud/ipfs/QmVMQM3crR5y8o7eFyz8wDuTiiHVJ4M9KzehjH3nHkyb3v"
                },
                "token_amount": "21060060.86858700000000000000",
                "quote_amount": "4.32067168100000000000",
                "cost_usd": "829.74178961924", // cost_usd表示当前行为花的钱
                "buy_cost_usd": "375.59062821386", // buy_cost_usd如果当前行为是buy则值为null，如果当前行为是sell则表示buy是花的钱
                "price_usd": "0.000039398831513036",
                "is_open_or_close": 1,
                "quote_token": {
                    "token_address": "So11111111111111111111111111111111111111112",
                    "name": "Wrapped SOL",
                    "symbol": "WSOL",
                    "decimals": 9,
                    "logo": "https://s2.coinmarketcap.com/static/img/coins/64x64/16116.png"
                },
                "from_address": "",
                "to_address": ""
    });

    activities.push(a);
    activities.push(b);
    activities.push(c);
    activities
}

// 主函数：加载用户信息
pub fn load_user_info(user: &mut User) -> Result<(), TxParseError> {
    let activities = get_default_user_activities();
    let user_token_stats = &mut user.token_stats;

    for activity in activities.iter() {
        // 每条记录的利润先默认为0
        let mut tx_profit = 0.0;
        // 成本
        let mut cost = 0.0;

        // 1. 获取并处理 token 信息
        if let Some(token_info) = activity.get("token") {
            if let Some(address_str) = token_info.get("address").and_then(|v| v.as_str()) {
                let mint = string_to_pub_key(address_str);

                // 判断是否已存在
                if let Some(trade_stats) = user_token_stats.get_mut(&mint) {
                    // 已存在 => 更新
                    tx_profit = handle_event_type(token_info, trade_stats)?;
                } else {
                    // 不存在 => 初始化
                    let mut token_status = TokenTradeStats::new(mint);
                    // 读取 symbol
                    if let Some(symbol) = token_info.get("symbol").and_then(|v| v.as_str()) {
                        token_status.symbol = symbol.to_string();
                    }
                    user.distinct_token_count += 1;

                    // 处理 buy/sell
                    let event_type = activity.get("event_type").and_then(|v| v.as_str());
                    // 买入统计成本，卖出统计利润
                    match event_type {
                        Some("buy") => {
                            cost = handle_event_type(activity, &mut token_status)?;
                        }
                        Some("sell") => {
                            tx_profit = handle_event_type(activity, &mut token_status)?;
                        }
                        _ => {}
                    }
                    // 插入 map
                    user_token_stats.insert(mint, token_status);
                }
            }
        }

        // 2. 处理 tx_hash
        if let Some(tx_sig) = activity.get("tx_hash").and_then(|v| v.as_str()) {
            user.token_txs.push(tx_sig.to_string());
        }

        // 3. 更新用户总利润/成本
        user.total_profit += tx_profit;
        user.total_cost += cost;
    }
    Ok(())
}

/// 处理 "event_type" => buy/sell 并返回本次交易利润
fn handle_event_type(
    activity_info: &Value,
    stats: &mut TokenTradeStats,
) -> Result<f64, TxParseError> {
    let event_type = activity_info.get("event_type").and_then(|v| v.as_str());
    let timestamp = activity_info
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .unwrap_or(0); // 或者用ok_or(...)?

    // 读取 token_amount
    let token_amount = activity_info
        .get("token_amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    match event_type {
        // 买入统计成本，卖出统计利润
        Some("buy") => {
            let buy_cost = parse_f64_from_string_field(activity_info, "cost_usd")?;
            stats.record_buy(token_amount, timestamp);
            Ok(buy_cost) // 买入不产生利润
        }
        Some("sell") => {
            let buy_cost = parse_f64_from_string_field(activity_info, "buy_cost_usd")?;
            let sell_cost = parse_f64_from_string_field(activity_info, "cost_usd")?;

            let profit = sell_cost - buy_cost;
            stats.record_sell(token_amount, timestamp, profit);

            // 判断盈亏记录
            if profit > 0.0 {
                stats.win_count += 1;
            } else if profit < 0.0 {
                stats.lose_count += 1;
            }
            stats.profit += profit;

            Ok(profit)
        }
        _ => {
            // 未知的 or 无 event_type，不做处理
            Ok(0.0)
        }
    }
}

fn parse_f64_from_string_field(
    data: &serde_json::Value,
    field_name: &str,
) -> Result<f64, TxParseError> {
    data.get(field_name)
        .and_then(|v| v.as_str()) // 先尝试获取字符串
        .and_then(|s| s.parse::<f64>().ok()) // 再尝试解析为 f64
        .ok_or_else(|| TxParseError::InvalidField(field_name.to_string())) // 如果失败，返回自定义错误
}

//
pub fn analysis() -> Result<(), Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::result;

    #[tokio::test]
    async fn test_get_history_tx2() {
        dotenv().ok();
        let collector = Collector::new(11).unwrap();
        let address = get_default_address();
        let mut user = User::new(address, 15);
        let limit: usize = 5;
        let sigs = collector.get_history_tx(&mut user, limit).await.unwrap();
        println!("sigs: {:?}", sigs);
    }

    #[tokio::test]
    async fn test_get_token_txs() {
        dotenv().ok();
        let collector = Collector::new(11).unwrap();
        let address = get_default_address();
        let mut user = User::new(address, 15);
        let limit: usize = 5;
        let sigs = collector.get_history_tx(&mut user, limit).await.unwrap();
        println!("sigs: {:?}", sigs);
        let result = collector.get_token_txs(&mut user).await.unwrap();
        println!("user : {:?}", user);
    }

    #[test]
    pub fn test_load_user_info() {
        dotenv().ok();
        let address = get_default_address();
        let mut user = User::new(address, 15);
        let result = load_user_info(&mut user).unwrap();
        // println!("user : {}", user);
        let mut filter_addresses = vec![];
        filter_addresses.push(string_to_pub_key(
            "6xmiC8Gsp6i8owu3JMDpt38vsCGznCmW5Fzjuomqpump",
        ));
        user.display_with_filter(&filter_addresses);
    }
}
