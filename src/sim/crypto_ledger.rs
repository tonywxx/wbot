//! 模拟加密货币账户（OKX 现货，USDT 计价）。
//!
//! 与股票 [`crate::sim::account::Account`] 共享 [`crate::ledger_core`] 的成交核心
//! （均价滚动 / 盈亏 / 手续费），此处仅保留加密货币独有的状态形状（f64 数量、
//! 无整手取整、无印花税）与下单签名。本模块从原 `crypto.rs` 拆出（架构评审候选 5），
//! 与 `account.rs` 为邻，便于两类账本一并审阅与测试。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ledger_core::{buy_calc, rolled_avg_cost, sell_calc, FeePolicy};

/// 加密费率策略：买卖同费率（单 `fee_rate`）。
struct CryptoFee(f64);
impl FeePolicy for CryptoFee {
    fn buy_fee(&self, notional: f64) -> f64 {
        notional * self.0
    }
    fn sell_fee(&self, notional: f64) -> f64 {
        notional * self.0
    }
}

/// 模拟加密货币账户：USDT 现金 + 基础币持仓（含均价成本）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CryptoLedger {
    /// 可用 USDT。
    pub usdt: f64,
    /// 各合约（如 BTC-USDT）的基础币持仓数量。
    pub positions: HashMap<String, f64>,
    /// 各合约的加权平均成本价（USDT），用于实现盈亏计算。
    pub avg_cost: HashMap<String, f64>,
}

/// 模拟加密货币成交结果。
#[derive(Debug, Clone)]
pub struct CryptoFill {
    pub fee: f64,
    pub cash_delta: f64,
    pub realized_pnl: f64,
}

impl CryptoLedger {
    /// 新建模拟账户，初始 `usdt` 现金。
    pub fn new(usdt: f64) -> Self {
        CryptoLedger {
            usdt,
            positions: HashMap::new(),
            avg_cost: HashMap::new(),
        }
    }

    /// 账户总权益（USDT）= 现金 + Σ(持仓数量 × 最新价)。
    pub fn total_value(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut total = self.usdt;
        for (inst, qty) in &self.positions {
            let p = prices.get(inst).copied().unwrap_or(0.0);
            total += qty * p;
        }
        total
    }

    /// 模拟买入 / 卖出。`base_qty` 为基础币数量；`price` 为成交价（USDT）。
    /// `fee_rate` 为单边费率。返回成交明细；现金 / 持仓不足则拒绝。
    pub fn place_order(
        &mut self,
        inst_id: &str,
        buy: bool,
        base_qty: f64,
        price: f64,
        fee_rate: f64,
    ) -> anyhow::Result<CryptoFill> {
        if base_qty <= 0.0 {
            anyhow::bail!("数量必须为正");
        }
        let fee_rate = fee_rate.max(0.0);
        let policy = CryptoFee(fee_rate);
        if buy {
            let c = buy_calc(base_qty, price, &policy);
            if c.total > self.usdt + 1e-9 {
                anyhow::bail!(
                    "USDT 不足：需要 {:.2}，可用 {:.2}",
                    c.total,
                    self.usdt
                );
            }
            self.usdt -= c.total;
            let pos = self.positions.entry(inst_id.to_string()).or_insert(0.0);
            let prev_qty = *pos;
            let prev_avg = self.avg_cost.get(inst_id).copied().unwrap_or(price);
            let new_avg = rolled_avg_cost(prev_qty, prev_avg, base_qty, price);
            *pos = prev_qty + base_qty;
            self.avg_cost.insert(inst_id.to_string(), new_avg);
            Ok(CryptoFill {
                fee: c.fee,
                cash_delta: -c.total,
                realized_pnl: 0.0,
            })
        } else {
            let pos = self
                .positions
                .get_mut(inst_id)
                .ok_or_else(|| anyhow::anyhow!("无持仓：{}", inst_id))?;
            if *pos < base_qty - 1e-12 {
                anyhow::bail!("持仓不足：持有 {:.6}，欲卖 {:.6}", pos, base_qty);
            }
            let avg = self.avg_cost.get(inst_id).copied().unwrap_or(price);
            let c = sell_calc(base_qty, price, avg, &policy);
            *pos -= base_qty;
            if *pos <= 1e-12 {
                self.positions.remove(inst_id);
                self.avg_cost.remove(inst_id);
            }
            self.usdt += c.cash_delta;
            Ok(CryptoFill {
                fee: c.fee,
                cash_delta: c.cash_delta,
                realized_pnl: c.realized,
            })
        }
    }
}
