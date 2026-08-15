//! 信号求值引擎：对作用范围内每只标的，按 `IndicatorRegistry` 计算指标并递归
//! 求值 `SignalNode`；以「沿触发」去抖（上一根未触发、本根触发才发事件）。

use std::collections::HashMap;

use chrono::{DateTime, Local};

use crate::indicators::{Candle, IndicatorRegistry};
use crate::signals::{CmpOp, CrossDir, Operand, Scope, Side, SignalNode, StrategyRule};

/// 一个新触发的信号事件（用于信号视图展示与可选自动下单）。
#[derive(Debug, Clone)]
pub struct SignalEvent {
    pub ts: DateTime<Local>,
    pub code: String,
    pub side: Side,
    pub rule_id: String,
    pub label: String,
    /// 策略的信号表达式（DSL 原文或形态描述），用于通知说明「为什么」。
    pub signal_text: String,
    /// 策略备注 / 说明，用于通知补充说明。
    pub note: String,
    /// 判断所用周期（如 Some("15") 表示 15 分钟线，None 表示日线），
    /// 用于在策略日志中标明该信号依据什么周期触发。
    pub timeframe: Option<String>,
}

pub struct SignalEngine {
    rules: Vec<StrategyRule>,
    /// (code, rule_id) -> 上一周期是否触发。
    prev: HashMap<(String, String), bool>,
}

impl SignalEngine {
    pub fn new(rules: Vec<StrategyRule>) -> Self {
        SignalEngine {
            rules,
            prev: HashMap::new(),
        }
    }

    /// 按 id 启用/停用某条规则（供策略选择界面切换）。
    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(r) = self.rules.iter_mut().find(|r| r.id == id) {
            r.enabled = enabled;
        }
    }

    /// 对作用范围内每只标的求值，返回本周期**新触发**的信号。
    /// `intraday` 为分钟 K 线，键为 `"{code}@{timeframe}"`，供形态规则使用。
    pub fn evaluate(
        &mut self,
        reg: &IndicatorRegistry,
        klines: &HashMap<String, Vec<Candle>>,
        prices: &HashMap<String, f64>,
        intraday: &HashMap<String, Vec<Candle>>,
    ) -> Vec<SignalEvent> {
        let mut events = Vec::new();
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            let codes: Vec<String> = match &rule.scope {
                Scope::Watchlist => prices.keys().cloned().collect(),
                Scope::Codes(cs) => cs.clone(),
            };
            for code in codes {
                // 序列选取 / 最小长度 / 持仓门槛统一来自 series module（实盘、回测、UI 共用）。
                let plan = match crate::series::select_rule_series(rule, &code, klines, intraday) {
                    Some(p) => p,
                    None => {
                        self.prev.insert((code.clone(), rule.id.clone()), false);
                        continue;
                    }
                };
                let price = prices.get(&code).copied();
                // 盘中近似：仅日线路径用实时价覆盖末根收盘。克隆后再改，保证 `klines`
                // 不被污染——回测（直接借用 `klines`）读到的始终是纯历史。intraday 路径
                // 与无实时价时不覆盖，与修复前行为一致。
                let owned = if rule.timeframe.is_none() {
                    let mut v = plan.series.to_vec();
                    if let (Some(p), Some(last)) = (price, v.last_mut()) {
                        last.close = p;
                    }
                    Some(v)
                } else {
                    None
                };
                let series: &[Candle] = owned.as_deref().unwrap_or(plan.series);
                let cur = eval_node(&rule.signal, reg, series, price, rule.side);
                let key = (code.clone(), rule.id.clone());
                let prev_trig = self.prev.get(&key).copied().unwrap_or(false);
                if cur && !prev_trig {
                    events.push(SignalEvent {
                        ts: Local::now(),
                        code: code.clone(),
                        side: rule.side,
                        rule_id: rule.id.clone(),
                        label: rule.label.clone(),
                        signal_text: rule.signal_text.clone(),
                        note: rule.note.clone(),
                        timeframe: rule.timeframe.clone(),
                    });
                }
                self.prev.insert(key, cur);
            }
        }
        events
    }
}

pub(crate) fn eval_node(
    node: &SignalNode,
    reg: &IndicatorRegistry,
    series: &[Candle],
    price: Option<f64>,
    side: Side,
) -> bool {
    match node {
        SignalNode::And(children) => {
            children.iter().all(|c| eval_node(c, reg, series, price, side))
        }
        SignalNode::Or(children) => {
            children.iter().any(|c| eval_node(c, reg, series, price, side))
        }
        SignalNode::Not(b) => !eval_node(b, reg, series, price, side),
        SignalNode::Cmp { op, left, right } => {
            let l = operand_value(left, reg, series, price);
            let r = operand_value(right, reg, series, price);
            match (l, r) {
                (Some(lv), Some(rv)) => cmp_op(*op, lv, rv),
                _ => false,
            }
        }
        SignalNode::Cross { dir, left, right } => {
            let ls = operand_series(left, reg, series, price);
            let rs = operand_series(right, reg, series, price);
            cross(*dir, &ls, &rs)
        }
        SignalNode::Pattern(spec) => {
            let close: Vec<f64> = series.iter().map(|c| c.close).collect();
            let high: Vec<f64> = series.iter().map(|c| c.high).collect();
            let low: Vec<f64> = series.iter().map(|c| c.low).collect();
            let bullish = side == Side::Buy;
            crate::signals::double_cross::detect_double_golden_cross(
                &close,
                &high,
                &low,
                spec.fast,
                spec.slow,
                spec.higher_low,
                bullish,
            )
        }
    }
}

fn cmp_op(op: CmpOp, l: f64, r: f64) -> bool {
    match op {
        CmpOp::Gt => l > r,
        CmpOp::Lt => l < r,
        CmpOp::Gte => l >= r,
        CmpOp::Lte => l <= r,
        CmpOp::Eq => (l - r).abs() < 1e-9,
    }
}

/// 比较/取值取操作数末值（NaN 视为无值 -> false）。
fn operand_value(
    o: &Operand,
    reg: &IndicatorRegistry,
    series: &[Candle],
    price: Option<f64>,
) -> Option<f64> {
    match o {
        Operand::Number(v) => Some(*v),
        Operand::Price(src) => price.or_else(|| series.last().map(|c| src.value(c))),
        Operand::Indicator(id) => reg
            .eval(id, series)
            .and_then(|v| v.last().copied())
            .filter(|x| !x.is_nan()),
    }
}

/// 交叉检测需要末两根序列值。
fn operand_series(o: &Operand, reg: &IndicatorRegistry, series: &[Candle], _price: Option<f64>) -> Vec<f64> {
    match o {
        Operand::Number(v) => vec![*v; series.len()],
        Operand::Price(src) => series.iter().map(|c| src.value(c)).collect(),
        Operand::Indicator(id) => reg
            .eval(id, series)
            .unwrap_or_else(|| vec![f64::NAN; series.len()]),
    }
}

/// 末两根：l1<=r1 且 l0>r0 => 上穿；l1>=r1 且 l0<r0 => 下穿。
fn cross(dir: CrossDir, left: &[f64], right: &[f64]) -> bool {
    let n = left.len().min(right.len());
    if n < 2 {
        return false;
    }
    let l1 = left[n - 2];
    let l0 = left[n - 1];
    let r1 = right[n - 2];
    let r0 = right[n - 1];
    if l1.is_nan() || l0.is_nan() || r1.is_nan() || r0.is_nan() {
        return false;
    }
    match dir {
        CrossDir::Above => l1 <= r1 && l0 > r0,
        CrossDir::Below => l1 >= r1 && l0 < r0,
    }
}
