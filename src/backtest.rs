//! 策略回测引擎：在历史 K 线上重放某条 `StrategyRule`，以「信号触发后持有 N 根」
//! 的前向收益衡量该信号的胜率与收益特征。与线上引擎一致采用「沿触发」去抖，
//! 避免一段持续信号被重复计数。
#![allow(dead_code)]

use std::collections::HashMap;

use chrono::NaiveDateTime;

use crate::config::AppConfig;
use crate::indicators::{Candle, IndicatorRegistry};
use crate::signals::double_cross::detect_double_golden_cross;
use crate::signals::eval::eval_node;
use crate::signals::{Scope, Side, SignalNode, StrategyRule};

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

// ---------------- 多标的聚合 + 报告生成 ----------------

/// 单只标的的回测结果。
#[derive(Debug, Clone)]
pub struct CodeResult {
    pub code: String,
    pub result: BacktestResult,
}

/// 一条策略在多只标的上的回测汇总（含分标的明细）。
#[derive(Debug, Clone)]
pub struct StrategyReport {
    pub rule: StrategyRule,
    /// 信号触发后持有的根数。
    pub hold: usize,
    /// 每只标的的结果（含未触发的标的）。
    pub per_code: Vec<CodeResult>,
    /// 跨标的聚合结果。
    pub agg: BacktestResult,
    /// 数据起始日期（最早一根可用 K 线）。
    pub date_start: Option<String>,
    /// 数据结束日期（最晚一根可用 K 线）。
    pub date_end: Option<String>,
    /// 实际参与回测的标的数量（有数据）。
    pub code_count: usize,
}

/// 在某条策略的「适用标的 -> K 线序列」映射上做回测，并跨标的聚合。
///
/// `code_series` 应由调用方按策略的 `scope`/`timeframe` 过滤并仅保留长度足够的序列。
pub fn backtest_strategy(
    rule: &StrategyRule,
    code_series: &HashMap<String, Vec<Candle>>,
    commission: f64,
    hold: usize,
) -> StrategyReport {
    let mut per_code = Vec::with_capacity(code_series.len());
    let mut agg_trades = 0usize;
    let mut agg_wins = 0usize;
    let mut sum_win = 0.0;
    let mut sum_loss = 0.0;
    let mut total_return = 0.0;
    let mut max_dd = 0.0;
    let mut date_start: Option<NaiveDateTime> = None;
    let mut date_end: Option<NaiveDateTime> = None;

    for (code, series) in code_series {
        let res = backtest_rule(rule, series, commission, hold);
        if res.trades > 0 {
            agg_trades += res.trades;
            agg_wins += res.wins;
            let losses = res.trades - res.wins;
            sum_win += res.avg_win * res.wins as f64;
            sum_loss += res.avg_loss * losses as f64;
            total_return += res.total_return;
            if res.max_drawdown > max_dd {
                max_dd = res.max_drawdown;
            }
            if let Some(first) = series.first() {
                if date_start.map_or(true, |s: NaiveDateTime| first.date < s) {
                    date_start = Some(first.date);
                }
            }
            if let Some(last) = series.last() {
                if date_end.map_or(true, |e: NaiveDateTime| last.date > e) {
                    date_end = Some(last.date);
                }
            }
        }
        per_code.push(CodeResult {
            code: code.clone(),
            result: res,
        });
    }

    let win_rate = if agg_trades > 0 {
        agg_wins as f64 / agg_trades as f64
    } else {
        0.0
    };
    let avg_win = if agg_wins > 0 {
        sum_win / agg_wins as f64
    } else {
        0.0
    };
    let losses = agg_trades - agg_wins;
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

    let agg = BacktestResult {
        bars: 0,
        trades: agg_trades,
        wins: agg_wins,
        win_rate,
        avg_win,
        avg_loss,
        profit_factor,
        total_return,
        max_drawdown: max_dd,
    };

    StrategyReport {
        rule: rule.clone(),
        hold,
        per_code,
        agg,
        date_start: date_start.map(fmt_date),
        date_end: date_end.map(fmt_date),
        code_count: code_series.len(),
    }
}

fn fmt_date(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d").to_string()
}

fn pct(x: f64) -> String {
    format!("{:.2}%", x * 100.0)
}

fn side_text(side: Side) -> &'static str {
    match side {
        Side::Buy => "买入（做多）",
        Side::Sell => "卖出（做空）",
    }
}

fn period_text(rule: &StrategyRule) -> String {
    match &rule.timeframe {
        Some(tf) => format!("{} 分钟", tf),
        None => "日线".to_string(),
    }
}

/// 把一条策略的回测结果渲染为 markdown 文本。
pub fn render_strategy_report_md(report: &StrategyReport, market: &str) -> String {
    let r = &report.rule;
    let a = &report.agg;
    let mut s = String::new();

    s.push_str(&format!("# {} 策略回测报告\n\n", r.label));
    s.push_str(&format!("> 策略 ID：`{}`\n\n", r.id));

    // 元信息表
    s.push_str("## 策略信息\n\n");
    s.push_str("| 项目 | 内容 |\n");
    s.push_str("| --- | --- |\n");
    s.push_str(&format!("| 市场 | {} |\n", market));
    s.push_str(&format!("| 名称 | {} |\n", r.label));
    s.push_str(&format!("| 方向 | {} |\n", side_text(r.side)));
    s.push_str(&format!("| 周期 | {} |\n", period_text(r)));
    s.push_str(&format!("| 信号 | `{}` |\n", r.signal_text));
    if !r.note.is_empty() {
        s.push_str(&format!("| 备注 | {} |\n", r.note));
    }
    let adjust_note = if market == "美股" {
        "前复权（分红/拆股调整）"
    } else {
        "复权 qfq"
    };
    s.push_str(&format!(
        "| 回测参数 | 信号触发后持有 {} 根；单边佣金 {:.4}（往返约 {:.4}）；{} |\n",
        report.hold, 0.0003, 0.0006, adjust_note
    ));
    s.push_str(&format!(
        "| 数据区间 | {} ~ {} |\n",
        report.date_start.as_deref().unwrap_or("N/A"),
        report.date_end.as_deref().unwrap_or("N/A")
    ));
    s.push_str(&format!("| 参与回测标的数 | {} |\n", report.code_count));
    s.push('\n');

    // 汇总表
    s.push_str("## 回测汇总（跨标的）\n\n");
    s.push_str("| 指标 | 数值 |\n");
    s.push_str("| --- | --- |\n");
    s.push_str(&format!("| 触发次数（交易笔数） | {} |\n", a.trades));
    s.push_str(&format!("| 盈利笔数 | {} |\n", a.wins));
    s.push_str(&format!("| 胜率 | {} |\n", pct(a.win_rate)));
    s.push_str(&format!("| 平均盈利 | {} |\n", pct(a.avg_win)));
    s.push_str(&format!("| 平均亏损 | {} |\n", pct(a.avg_loss)));
    let pf = if a.profit_factor.is_infinite() {
        "∞（无亏损）".to_string()
    } else {
        format!("{:.2}", a.profit_factor)
    };
    s.push_str(&format!("| 盈亏比 | {} |\n", pf));
    s.push_str(&format!("| 累计净收益（多标的合计） | {} |\n", pct(a.total_return)));
    s.push_str(&format!("| 最大回撤 | {} |\n", pct(a.max_drawdown)));
    s.push('\n');

    // 分标的明细
    s.push_str("## 分标的明细\n\n");
    s.push_str("| 代码 | 触发次数 | 盈利 | 胜率 | 累计净收益 | 最大回撤 |\n");
    s.push_str("| --- | ---: | ---: | ---: | ---: | ---: |\n");
    for cr in &report.per_code {
        let res = &cr.result;
        if res.trades == 0 {
            s.push_str(&format!(
                "| {} | 0 | - | - | - | - |\n",
                cr.code
            ));
        } else {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                cr.code,
                res.trades,
                res.wins,
                pct(res.win_rate),
                pct(res.total_return),
                pct(res.max_drawdown)
            ));
        }
    }
    s.push('\n');

    // 说明
    s.push_str("## 说明\n\n");
    s.push_str(
        "1. 本回测在所选个股的历史 K 线上重放策略信号，以「信号触发后持有固定根数」的前向收益衡量策略表现。\n",
    );
    s.push_str(
        "2. 采用沿触发（false→true 才计入），避免持续信号被重复计数；手续费按单边佣金 ×2 扣减，未计印花税与滑点。\n",
    );
    s.push_str(
        "3. 累计净收益为多标的各自权益曲线净收益的合计，仅供横向比较，不代表实盘收益。\n",
    );
    s.push_str(
        "4. 历史回测结果不代表未来收益，本报告仅供参考，不构成任何投资建议。\n",
    );

    s
}

/// 按策略周期选取某标的的回测序列。
///
/// 含 `timeframe` 的形态规则走分钟 K 线（复合键 `{code}@{tf}`），否则走日线。
/// 序列长度不足 3 根时返回 `None`（不足以产生可信前向收益）。
/// 该逻辑此前内联于 [`write_strategy_reports`]，抽离后成为可独立测试的纯函数。
pub(crate) fn select_series(
    timeframe: Option<&str>,
    code: &str,
    klines: &HashMap<String, Vec<Candle>>,
    intraday: &HashMap<String, Vec<Candle>>,
) -> Option<Vec<Candle>> {
    let series = if let Some(tf) = timeframe {
        intraday.get(&format!("{}@{}", code, tf)).cloned()
    } else {
        klines.get(code).cloned()
    };
    series.filter(|s| s.len() >= 3)
}

/// 对每条策略：解析其作用范围、匹配对应 K 线（日线或分钟），跑回测并写出
/// `<id> 策略回测报告.md` 文件。返回「策略 ID -> 报告路径」列表。
pub fn write_strategy_reports(
    out_dir: &str,
    rules: &[StrategyRule],
    klines: &HashMap<String, Vec<Candle>>,
    intraday: &HashMap<String, Vec<Candle>>,
    watchlist: &[String],
    config: &AppConfig,
    market: &str,
) -> Vec<(String, std::path::PathBuf)> {
    let _ = std::fs::create_dir_all(out_dir);
    let mut written = Vec::new();

    for rule in rules {
        // 解析作用范围 -> 适用代码列表。
        let codes: Vec<String> = match &rule.scope {
            Scope::Watchlist => watchlist.to_vec(),
            Scope::Codes(cs) => cs.clone(),
        };

        // 按周期匹配 K 线序列（仅保留长度足够者）。
        let mut series_map: HashMap<String, Vec<Candle>> = HashMap::new();
        for code in &codes {
            if let Some(s) = select_series(rule.timeframe.as_deref(), code, klines, intraday) {
                series_map.insert(code.clone(), s);
            }
        }

        let hold = if rule.timeframe.is_some() { 5 } else { 10 };
        let report = backtest_strategy(rule, &series_map, config.commission, hold);
        let md = render_strategy_report_md(&report, market);

        let fname = format!("{} 策略回测报告.md", rule.id);
        let path = std::path::Path::new(out_dir).join(&fname);
        if std::fs::write(&path, md).is_ok() {
            written.push((rule.id.clone(), path));
        }
    }

    written
}
