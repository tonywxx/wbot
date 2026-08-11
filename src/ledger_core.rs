//! 账本核心纯数学：买卖交易计算 + 加权均价滚动。
//!
//! 股票 (`Account`) 与加密 (`CryptoLedger`) 的成交逻辑在数学上完全重复，
//! 抽离为单一可测核心以消除漂移；两账本作为 adapter 各自保留状态形状、
//! 数量精度（i64/lot vs f64）与手续费率结构（股票卖出含印花税）。
//! 状态读写与拒单判定仍留在各 adapter —— 它们依赖各自的形态，不宜下沉。

/// 手续费率策略：买入与卖出可分别计费。
/// 股票：买入=佣金，卖出=佣金+印花税；加密：买卖同费率。
pub trait FeePolicy {
    fn buy_fee(&self, notional: f64) -> f64;
    fn sell_fee(&self, notional: f64) -> f64;
}

/// 一次买入的纯计算结果（不含状态变更）。
#[derive(Debug, Clone, Copy)]
pub struct BuyCalc {
    #[allow(dead_code)]
    pub cost: f64,
    pub fee: f64,
    pub total: f64,
}

/// 买入交易计算：成交额 + 手续费 + 应付总额。
pub fn buy_calc(qty: f64, price: f64, policy: &impl FeePolicy) -> BuyCalc {
    let cost = qty * price;
    let fee = policy.buy_fee(cost);
    BuyCalc {
        cost,
        fee,
        total: cost + fee,
    }
}

/// 加权均价滚动：返回新持仓的加权平均成本价。
/// 新仓（`prev_qty == 0`）直接取本次成交价。
pub fn rolled_avg_cost(prev_qty: f64, prev_avg: f64, qty: f64, price: f64) -> f64 {
    let new_qty = prev_qty + qty;
    if new_qty == 0.0 {
        return price;
    }
    (prev_avg * prev_qty + price * qty) / new_qty
}

/// 一次卖出的纯计算结果（不含状态变更）。
#[derive(Debug, Clone, Copy)]
pub struct SellCalc {
    #[allow(dead_code)]
    pub proceeds: f64,
    pub fee: f64,
    pub realized: f64,
    pub cash_delta: f64,
}

/// 卖出交易计算：成交额 + 手续费 + 已实现盈亏 + 现金变动。
pub fn sell_calc(qty: f64, price: f64, avg: f64, policy: &impl FeePolicy) -> SellCalc {
    let proceeds = qty * price;
    let fee = policy.sell_fee(proceeds);
    let realized = (price - avg) * qty - fee;
    SellCalc {
        proceeds,
        fee,
        realized,
        cash_delta: proceeds - fee,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用对称费率策略（买卖同费率）。
    struct Flat(f64);
    impl FeePolicy for Flat {
        fn buy_fee(&self, n: f64) -> f64 {
            n * self.0
        }
        fn sell_fee(&self, n: f64) -> f64 {
            n * self.0
        }
    }

    #[test]
    fn rolled_avg_cost_new_position_is_price() {
        assert!((rolled_avg_cost(0.0, 0.0, 10.0, 20.0) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn rolled_avg_cost_blended() {
        // 100@10 + 50@20 => (1000 + 1000) / 150 = 2000/150
        let avg = rolled_avg_cost(100.0, 10.0, 50.0, 20.0);
        assert!((avg - 2000.0 / 150.0).abs() < 1e-9);
    }

    #[test]
    fn buy_calc_applies_fee() {
        let c = buy_calc(10.0, 100.0, &Flat(0.001));
        assert!((c.cost - 1000.0).abs() < 1e-9);
        assert!((c.fee - 1.0).abs() < 1e-9);
        assert!((c.total - 1001.0).abs() < 1e-9);
    }

    #[test]
    fn sell_calc_realized_after_fee() {
        // 卖 10@100，成本 80，费率 0.001 => 成交额 1000，手续费 1，盈亏 (100-80)*10 - 1 = 199
        let c = sell_calc(10.0, 100.0, 80.0, &Flat(0.001));
        assert!((c.proceeds - 1000.0).abs() < 1e-9);
        assert!((c.fee - 1.0).abs() < 1e-9);
        assert!((c.realized - 199.0).abs() < 1e-9);
        assert!((c.cash_delta - 999.0).abs() < 1e-9);
    }
}
