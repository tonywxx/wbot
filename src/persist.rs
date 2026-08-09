//! 持久化：账户与成交记录的读写，统一缺失/损坏兜底。

use crate::sim::account::Account;

/// 保存账户到 JSON 文件。
pub fn save_account(path: &str, acc: &Account) -> anyhow::Result<()> {
    let s = serde_json::to_string_pretty(acc)?;
    std::fs::write(path, s)?;
    Ok(())
}

/// 读取账户；文件缺失/损坏回退新账户（初始资金 100 万）。
pub fn load_account(path: &str, lot_size: i64, commission: f64, stamp_tax: f64) -> Account {
    match std::fs::read_to_string(path) {
        Ok(s) if !s.trim().is_empty() => serde_json::from_str(&s)
            .unwrap_or_else(|_| Account::new(lot_size, commission, stamp_tax)),
        _ => Account::new(lot_size, commission, stamp_tax),
    }
}
