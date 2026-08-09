//! 回测报告编排：拉取历史行情 -> 逐策略回测 -> 写出 markdown 报告。
//!
//! 同时被二进制 `backtest` 子命令与 `examples/backtest_all.rs` 复用。
//! - [`generate_reports`]：A 股回测（akshare），默认写到 `reports/`。
//! - [`generate_reports_us`]：美股回测（yfinance-rs / Yahoo Finance），写到 `reports_us/`。
//!   数据获取失败时（如本沙箱环境 Yahoo 返回 429）相关标的无数据，对应报告仍会
//!   生成（明细为 N/A），不会崩溃。

use std::collections::HashMap;
use std::path::PathBuf;

use akshare::AkShareClient;
use anyhow::Result;
use yfinance_rs::YfClient;

use crate::backtest::write_strategy_reports;
use crate::config::load_config;
use crate::market::{
    fetch_all_intraday, fetch_all_intraday_market, fetch_all_klines, fetch_all_klines_market,
    load_watchlist, load_watchlist_us,
};
use crate::signals::parse_strategy_file;

/// 从策略集合中提取去重的 (timeframe, bars) 组合（分钟 / T+0 规则需要）。
fn collect_tf_bars(strategies: &[crate::signals::StrategyRule]) -> Vec<(String, usize)> {
    let mut tf_bars: Vec<(String, usize)> = Vec::new();
    for r in strategies {
        if let (Some(tf), Some(bars)) = (r.timeframe.clone(), r.bars) {
            if !tf_bars.iter().any(|(t, b)| t == &tf && *b == bars) {
                tf_bars.push((tf, bars));
            }
        }
    }
    tf_bars
}

/// 对 `strategy.toml` 中的每条策略，在 A 股自选股历史数据上跑回测，
/// 并将报告写入 `out_dir`（默认相对目录 `reports`）。
///
/// 返回「策略 ID -> 报告文件绝对路径」列表。网络不可用时相关标的无数据，
/// 对应报告仍会生成（明细为 N/A），不会崩溃。
pub async fn generate_reports(out_dir: &str) -> Result<Vec<(String, PathBuf)>> {
    let config = load_config();
    let watchlist = load_watchlist();
    let strategies = parse_strategy_file("strategy.toml");
    if strategies.is_empty() {
        anyhow::bail!("未解析到任何策略（strategy.toml 缺失或为空）。");
    }

    let client = AkShareClient::new();

    // 日线（DSL 规则 + 形态规则的日线分支）。
    println!(
        "正在拉取 {} 只标的的日线历史（{} 根，复权 {}）…",
        watchlist.len(),
        config.kline_count,
        config.kline_adjust
    );
    let klines =
        fetch_all_klines(&client, &watchlist, &config.kline_adjust, config.kline_count).await;

    // 分钟 K 线（含 timeframe 的规则）。
    let tf_bars = collect_tf_bars(&strategies);
    let intraday = if tf_bars.is_empty() {
        HashMap::new()
    } else {
        println!("正在拉取分钟 K 线（{:?}）…", tf_bars);
        fetch_all_intraday(&client, &watchlist, &tf_bars).await
    };

    let have_daily = klines.values().filter(|s| !s.is_empty()).count();
    let have_intraday = intraday.values().filter(|s| !s.is_empty()).count();
    println!(
        "数据就绪：日线 {} 只、分钟 {} 组。开始生成报告…",
        have_daily, have_intraday
    );

    let paths = write_strategy_reports(
        out_dir,
        &strategies,
        &klines,
        &intraday,
        &watchlist,
        &config,
        "A股",
    );

    Ok(paths)
}

/// 对 `strategy.toml` 中的每条策略，在美股自选股（Yahoo Finance）历史数据上跑回测，
/// 并将报告写入 `out_dir`（默认相对目录 `reports_us`）。
///
/// 美股数据通过 `yfinance-rs` 获取；与 A 股复用同一套指标 / 信号 / 回测引擎，
/// 仅数据源不同。若 Yahoo 在本环境不可达（如返回 429），报告明细为 N/A，
/// 不会崩溃 —— 在可访问 Yahoo 的环境中运行即可获得真实回测结果。
pub async fn generate_reports_us(out_dir: &str) -> Result<Vec<(String, PathBuf)>> {
    let config = load_config();
    let watchlist = load_watchlist_us();
    let strategies = parse_strategy_file("strategy.toml");
    if strategies.is_empty() {
        anyhow::bail!("未解析到任何策略（strategy.toml 缺失或为空）。");
    }

    let ak = AkShareClient::new();
    let yf = YfClient::default();

    // 日线（DSL 规则 + 形态规则的日线分支），美股按代码形态自动走 yfinance。
    println!(
        "正在拉取 {} 只美股的日线历史（{} 根）…",
        watchlist.len(),
        config.kline_count
    );
    let klines =
        fetch_all_klines_market(&ak, &yf, &watchlist, &config.kline_adjust, config.kline_count)
            .await;

    // 分钟 K 线（含 timeframe 的规则）。
    let tf_bars = collect_tf_bars(&strategies);
    let intraday = if tf_bars.is_empty() {
        HashMap::new()
    } else {
        println!("正在拉取美股分钟 K 线（{:?}）…", tf_bars);
        fetch_all_intraday_market(&ak, &yf, &watchlist, &tf_bars).await
    };

    let have_daily = klines.values().filter(|s| !s.is_empty()).count();
    let have_intraday = intraday.values().filter(|s| !s.is_empty()).count();
    println!(
        "美股数据就绪：日线 {} 只、分钟 {} 组。开始生成报告…",
        have_daily, have_intraday
    );
    if have_daily == 0 && have_intraday == 0 {
        eprintln!(
            "⚠️  未获取到任何美股历史行情：Yahoo Finance 在本环境可能不可达（常见为 429 限流）。\n\
             \t报告仍会生成，但明细为 N/A。请在可访问 Yahoo 的环境运行本命令以获取真实回测结果。"
        );
    }

    let paths = write_strategy_reports(
        out_dir,
        &strategies,
        &klines,
        &intraday,
        &watchlist,
        &config,
        "美股",
    );

    Ok(paths)
}
