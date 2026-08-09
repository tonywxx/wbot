//! 成交记录：JSON 持久化（追加/读取），缺失或损坏安全兜底。

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::signals::Side;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub ts: DateTime<Local>,
    pub code: String,
    pub side: Side,
    pub price: f64,
    pub qty: i64,
    pub fee: f64,
    pub realized_pnl: f64,
    pub cash_delta: f64,
}

/// 追加一条成交记录到 JSON 文件（整体重写，模拟交易量级可接受）。
pub fn append_trade(path: &str, t: &Trade) -> anyhow::Result<()> {
    let mut trades = load_trades(path);
    trades.push(t.clone());
    let s = serde_json::to_string_pretty(&trades)?;
    std::fs::write(path, s)?;
    Ok(())
}

/// 读取全部成交记录；文件缺失/损坏返回空 Vec（不崩溃）。
pub fn load_trades(path: &str) -> Vec<Trade> {
    match std::fs::read_to_string(path) {
        Ok(s) if !s.trim().is_empty() => serde_json::from_str(&s).unwrap_or_default(),
        _ => Vec::new(),
    }
}
