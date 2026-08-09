//! 模拟账户：现金、持仓、成交（市价单），含手续费/印花税与盈亏统计。

use std::collections::HashMap;

use crate::signals::Side;

use super::history::Trade;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub code: String,
    pub qty: i64,
    pub avg_cost: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Account {
    /// 初始资金（用于收益率展示）。
    pub initial: f64,
    pub cash: f64,
    pub positions: HashMap<String, Position>,
    pub lot_size: i64,
    pub commission: f64,
    pub stamp_tax: f64,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub code: String,
    pub side: Side,
    pub qty: i64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct FillResult {
    pub realized_pnl: f64,
    pub fee: f64,
    pub cash_delta: f64,
}

impl Account {
    pub fn new(lot_size: i64, commission: f64, stamp_tax: f64) -> Self {
        let initial = 1_000_000.0;
        Account {
            initial,
            cash: initial,
            positions: HashMap::new(),
            lot_size: lot_size.max(1),
            commission,
            stamp_tax,
        }
    }

    /// 总资产 = 现金 + 持仓市值（以最新价计，缺价用成本价）。
    pub fn total_assets(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut total = self.cash;
        for (code, pos) in &self.positions {
            let price = prices.get(code).copied().unwrap_or(pos.avg_cost);
            total += pos.qty as f64 * price;
        }
        total
    }

    /// 浮动盈亏 = Σ (最新价 - 持仓成本) * 数量。
    pub fn unrealized_pnl(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut pnl = 0.0;
        for (code, pos) in &self.positions {
            let price = prices.get(code).copied().unwrap_or(pos.avg_cost);
            pnl += (price - pos.avg_cost) * pos.qty as f64;
        }
        pnl
    }

    /// 下单成交。数量向下取整到整手倍数；现金/持仓不足则拒绝。
    pub fn place_order(&mut self, o: &Order) -> anyhow::Result<FillResult> {
        let lot = self.lot_size.max(1);
        let qty = (o.qty / lot) * lot;
        if qty <= 0 {
            anyhow::bail!("数量不足一手 ({})", lot);
        }
        let fee_rate = self.commission;

        match o.side {
            Side::Buy => {
                let cost = o.price * qty as f64;
                let fee = cost * fee_rate;
                let total = cost + fee;
                if total > self.cash + 1e-6 {
                    anyhow::bail!("现金不足：需要 {:.2}，可用 {:.2}", total, self.cash);
                }
                self.cash -= total;
                let pos = self
                    .positions
                    .entry(o.code.clone())
                    .or_insert(Position {
                        code: o.code.clone(),
                        qty: 0,
                        avg_cost: 0.0,
                    });
                let new_qty = pos.qty + qty;
                pos.avg_cost = (pos.avg_cost * pos.qty as f64 + cost) / new_qty as f64;
                pos.qty = new_qty;
                Ok(FillResult {
                    realized_pnl: 0.0,
                    fee,
                    cash_delta: -total,
                })
            }
            Side::Sell => {
                let pos = self
                    .positions
                    .get_mut(&o.code)
                    .ok_or_else(|| anyhow::anyhow!("无持仓：{}", o.code))?;
                if pos.qty < qty {
                    anyhow::bail!("持仓不足：持有 {}，欲卖 {}", pos.qty, qty);
                }
                let proceeds = o.price * qty as f64;
                let fee = proceeds * fee_rate + proceeds * self.stamp_tax;
                let realized = (o.price - pos.avg_cost) * qty as f64 - fee;
                pos.qty -= qty;
                if pos.qty == 0 {
                    self.positions.remove(&o.code);
                }
                self.cash += proceeds - fee;
                Ok(FillResult {
                    realized_pnl: realized,
                    fee,
                    cash_delta: proceeds - fee,
                })
            }
        }
    }
}

/// 由一次成交结果构造可持久化的 `Trade` 记录。
pub fn fill_to_trade(o: &Order, fill: &FillResult, ts: chrono::DateTime<chrono::Local>) -> Trade {
    Trade {
        ts,
        code: o.code.clone(),
        side: o.side,
        price: o.price,
        qty: (o.qty / 100) * 100, // 与账户取整保持一致（仅记录用）
        fee: fill.fee,
        realized_pnl: fill.realized_pnl,
        cash_delta: fill.cash_delta,
    }
}
