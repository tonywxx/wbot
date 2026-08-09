//! 策略回测引擎：在历史 K 线上重放某条 `StrategyRule`，以「信号触发后持有 N 根」
//! 的前向收益衡量该信号的胜率与收益特征。与线上引擎一致采用「沿触发」去抖，
//! 避免一段持续信号被重复计数。
#![allow(dead_code)]

use crate::indicators::{Candle, IndicatorRegistry};
use crate::signals::double_cross::detect_double_golden_cross;
use crate::signals::eval::eval_node;
use crate::signals::{Side, SignalNode, StrategyRule};

/// 回测结果汇总。
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct BacktestResult {
    pub bars: usize,
    /// 触发的信号次数（= 回测中的交易笔数）。
    pub trades: usize,
    pub wins: usize,
    /// 胜率（盈利信号占比）。
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    /// 盈亏比（总盈利 / 总亏损）；无亏损时为 +inf。
    pub profit_factor: f64,
    /// 累计净收益（按每次信号满仓计，仅作参考）。
    pub total_return: f64,
    /// 权益曲线最大回撤（比例，正值）。
    pub max_drawdown: f64,
}

impl Default for BacktestResult {
    fn default() -> Self {
        BacktestResult {
            bars: 0,
            trades: 0,
            wins: 0,
            win_rate: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,
            total_return: 0.0,
            max_drawdown: 0.0,
        }
    }
}

/// 在 `prefix`（升序）上求该规则在末根是否触发。
fn signal_on_prefix(rule: &StrategyRule, reg: &IndicatorRegistry, prefix: &[Candle]) -> bool {
    match &rule.signal {
        SignalNode::Pattern(spec) => {
            let n = prefix.len();
            if n < spec.slow + 3 {
                return false;
            }
            let close: Vec<f64> = prefix.iter().map(|c| c.close).collect();
            let high: Vec<f64> = prefix.iter().map(|c| c.high).collect();
            let low: Vec<f64> = prefix.iter().map(|c| c.low).collect();
            detect_double_golden_cross(
                &close,
                &high,
                &low,
                spec.fast,
                spec.slow,
                spec.higher_low,
                rule.side == Side::Buy,
            )
        }
        node => eval_node(node, reg, prefix, None, rule.side),
    }
}

/// 回测单条规则。
///
/// `commission` 为单边费率（每笔往返约 2×commission）；`hold` 为信号触发后持有的根数。
/// 采用沿触发：仅统计 false→true 的「新触发」信号，避免持续信号重复计数。
pub fn backtest_rule(
    rule: &StrategyRule,
    series: &[Candle],
    commission: f64,
    hold: usize,
) -> BacktestResult {
    let n = series.len();
    if n < 3 || hold == 0 {
        return BacktestResult::default();
    }
    let reg = IndicatorRegistry::new();
    let long = rule.side == Side::Buy;
    let min_len = match &rule.signal {
        SignalNode::Pattern(s) => s.slow + 3,
        _ => 2,
    };

    let mut trades = 0usize;
    let mut wins = 0usize;
    let mut sum_win = 0.0;
    let mut sum_loss = 0.0;
    let mut equity = 1.0;
    let mut peak = 1.0;
    let mut max_dd = 0.0;
    let mut prev_sig = false;

    for i in (min_len - 1)..n {
        let prefix = &series[0..=i];
        let sig = signal_on_prefix(rule, &reg, prefix);
        // 沿触发：仅在新触发（false->true）时计入一次。
        let fresh = sig && !prev_sig;
        prev_sig = sig;

        if fresh && i + hold < n {
            let entry = series[i].close;
            let exit = series[i + hold].close;
            let gross = if long {
                (exit - entry) / entry
            } else {
                (entry - exit) / entry
            };
            let net = gross - 2.0 * commission;
            trades += 1;
            if net > 0.0 {
                wins += 1;
                sum_win += net;
            } else {
                sum_loss += -net;
            }
            equity += net;
            if equity > peak {
                peak = equity;
            }
            let dd = if peak > 0.0 { (peak - equity) / peak } else { 0.0 };
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    let win_rate = if trades > 0 {
        wins as f64 / trades as f64
    } else {
        0.0
    };
    let avg_win = if wins > 0 {
        sum_win / wins as f64
    } else {
        0.0
    };
    let losses = trades - wins;
    let avg_loss = if losses > 0 {
        sum_loss / losses as f64
    } else {
        0.0
    };
    let profit_factor = if sum_loss > 0.0 {
        sum_win / sum_loss
    } else if sum_win > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    BacktestResult {
        bars: n,
        trades,
        wins,
        win_rate,
        avg_win,
        avg_loss,
        profit_factor,
        total_return: equity - 1.0,
        max_drawdown: max_dd,
    }
}
