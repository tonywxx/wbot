//! 规则驱动的序列选取与持仓门槛（统一真相源）。
//!
//! 把「取哪条序列 + 最小长度 + 持仓根数」从 `eval` / `backtest` / `app` 三处
//! 收进一个 module，消除三套不一致的门槛（实盘日线 2、回测 3、形态 `slow+3`
//! 等）。interface 即测试面：三处都调 [`select_rule_series`]，不再各自重算。
//!
//! 设计为 deep module：interface 只有 [`select_rule_series`] 一个入口，
//! 实现吸收了原本散落在三处的门槛逻辑；[`min_len_for`] / [`hold_bars`]
//! 是供回测循环起点复用的内部纯函数，确保两端用同一门槛。

use std::collections::HashMap;

use crate::indicators::Candle;
use crate::signals::{SignalNode, StrategyRule};

/// 一条规则在某标的上对应的回测序列与持仓根数。
///
/// `series` 借用自调用方持有的 K 线表，避免热路径上的克隆。
#[derive(Debug, Clone)]
pub struct RuleSeries<'a> {
    pub series: &'a [Candle],
    /// 信号触发后持有的根数（分钟 5 / 日线 10）。
    pub hold: usize,
}

/// 给定规则 + 标的 + 日线/分钟 K 线，返回对应的序列与持仓根数。
///
/// 序列长度不足最小门槛时返回 `None`（调用点据此跳过该标的）。这是实盘求值、
/// 回测报告生成、UI 重算三处共用的唯一门槛来源。
pub fn select_rule_series<'a>(
    rule: &StrategyRule,
    code: &str,
    klines: &'a HashMap<String, Vec<Candle>>,
    intraday: &'a HashMap<String, Vec<Candle>>,
) -> Option<RuleSeries<'a>> {
    let series: &'a [Candle] = if let Some(tf) = &rule.timeframe {
        intraday.get(&format!("{code}@{tf}"))?
    } else {
        klines.get(code)?
    };
    if series.len() < min_len_for(rule) {
        return None;
    }
    Some(RuleSeries {
        series,
        hold: hold_bars(rule),
    })
}

/// 一条规则触发所需的最小序列长度。
///
/// - 形态（双交叉）规则：`slow + 3`（需慢线充分预热）。
/// - 其它（DSL / 阈值）规则：固定 `3` 根下限。
///
/// 供 [`select_rule_series`] 选序列，以及 `backtest_rule` 的循环起点复用，
/// 确保两端用同一门槛、不再漂移。
pub(crate) fn min_len_for(rule: &StrategyRule) -> usize {
    match &rule.signal {
        SignalNode::Pattern(spec) => spec.slow + 3,
        _ => 3,
    }
}

/// 信号触发后的持仓根数：分钟线 5、日线 10。
///
/// 与序列选取同源于本 module，避免 `backtest` / `app` 各写一份 `5 / 10`。
pub(crate) fn hold_bars(rule: &StrategyRule) -> usize {
    if rule.timeframe.is_some() { 5 } else { 10 }
}
