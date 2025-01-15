use std::{collections::HashMap, fs, str::FromStr};

use anyhow::Result;
use solana_sdk::{
    message::v0::MessageAddressTableLookup, pubkey::Pubkey, signer::Signer,
    transaction::VersionedTransaction,
};
use solana_transaction_status_client_types::{
    option_serializer::OptionSerializer, EncodedTransactionWithStatusMeta, UiTransactionStatusMeta,
    UiTransactionTokenBalance,
};

use crate::constant::{PUMP_PROGRAM_ID, RAYDIUM_PROGRAM_ID};

// 分析交易，找到聪明钱包
pub struct WalletAnalyzer {
    // 盈利计算
    profit_map: HashMap<String, u64>,
    // token地址
    token_address: Vec<String>,
    // 聪明钱包地址
    smart_wallets: Vec<String>,
}

impl WalletAnalyzer {
    pub fn new(token_address: Vec<String>) -> Self {
        Self {
            token_address,
            profit_map: HashMap::new(),
            smart_wallets: Vec::new(),
        }
    }

    // 累积盈利
    pub fn cum_profit(&mut self, meta: &UiTransactionStatusMeta) -> Result<()> {
        let balance_change = get_balance_change(&self.token_address, meta)?;
        println!("balance_change {:?}", balance_change);
        Ok(())
    }
}

// 判断是否为target交易
pub fn is_target(account_keys: &[Pubkey]) -> bool {
    account_keys.contains(&PUMP_PROGRAM_ID) || account_keys.contains(&RAYDIUM_PROGRAM_ID)
}

pub fn string_to_pub_key(origin: &str) -> Pubkey {
    Pubkey::from_str_const(origin)
}

// 计算一笔交易的sol的数值改变
fn cacl_sol_amount_change(sol_pre_balance: u64, sol_post_balance: u64) -> i64 {
    return sol_post_balance as i64 - sol_pre_balance as i64;
}

// 计算相关token的数值改变
fn cacl_token_amount_change(
    token_address: &Vec<String>,
    pre_token_balances: &Vec<UiTransactionTokenBalance>,
    post_token_balances: &Vec<UiTransactionTokenBalance>,
) -> HashMap<String, i64> {
    // token_pub_key => amount_change
    let mut change_map = HashMap::<String, i64>::new();

    // 计算每一种代币的余额变化
    for i in 0..post_token_balances.len() {
        if pre_token_balances.len() <= i {
            println!("pre_token_balances {:?}", pre_token_balances);
            println!("post_token_balances {:?}", post_token_balances);
        }
        // 当前代币的地址
        let pubkey = &pre_token_balances[i].mint;
        // 判断是否为需要判断的代币
        if token_address.contains(pubkey) {
            // 账户对应的余额变化
            let pre_balance = &pre_token_balances[i]
                .ui_token_amount
                .amount
                .parse::<i64>()
                .unwrap();
            let post_balance = &post_token_balances[i]
                .ui_token_amount
                .amount
                .parse::<i64>()
                .unwrap();
            let profit = post_balance - pre_balance;
            // 已有更新，未有创建
            *change_map.entry(pubkey.clone()).or_insert(0) += profit;
        }
    }
    change_map
}

// 获取地址的相关token的余额变化
pub fn get_balance_change(
    token_addresses: &Vec<String>,
    meta: &UiTransactionStatusMeta,
) -> Result<(i64, HashMap<String, i64>)> {
    let sol_amount = cacl_sol_amount_change(meta.pre_balances[0], meta.post_balances[0]);
    // sol余额检查
    let token_info = if meta.pre_token_balances.is_some() {
        let pre_token_balances = meta.pre_token_balances.as_ref().unwrap();
        let post_token_balances = meta.post_token_balances.as_ref().unwrap();
        if pre_token_balances.len() == post_token_balances.len() {
            cacl_token_amount_change(token_addresses, pre_token_balances, post_token_balances)
        } else {
            HashMap::new()
        }
        // 计算代币的变化
    } else {
        HashMap::new()
    };
    Ok((sol_amount, token_info))
}
