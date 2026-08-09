//! 回测报告编排：拉取历史行情 -> 逐策略回测 -> 写出 markdown 报告。
//!
//! 同时被二进制 `backtest` 子命令与 `examples/backtest_all.rs` 复用。

use std::collections::HashMap;
use std::path::PathBuf;

use akshare::AkShareClient;
use anyhow::Result;

use crate::backtest::write_strategy_reports;
use crate::config::load_config;
use crate::market::{fetch_all_intraday, fetch_all_klines, load_watchlist};
use crate::signals::parse_strategy_file;

/// 对 `strategy.toml` 中的每条策略，在自选股历史数据上跑回测，
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

    // 分钟 K 线（含 timeframe 的规则）：收集去重后的 (timeframe, bars) 组合。
    let mut tf_bars: Vec<(String, usize)> = Vec::new();
    for r in &strategies {
        if let (Some(tf), Some(bars)) = (r.timeframe.clone(), r.bars) {
            if !tf_bars.iter().any(|(t, b)| t == &tf && *b == bars) {
                tf_bars.push((tf, bars));
            }
        }
    }
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
    );

    Ok(paths)
}
