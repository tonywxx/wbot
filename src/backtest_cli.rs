//! 回测报告编排：拉取历史行情 -> 逐策略回测 -> 写出 markdown 报告。
//!
//! 同时被二进制 `backtest` 子命令与 `examples/backtest_all.rs` 复用。
//! - [`generate_reports`]：A 股回测（akshare），默认写到 `reports/`。
//! - [`generate_reports_us`]：美股回测（yfinance-rs / Yahoo Finance），写到 `reports_us/`。
//!   数据获取失败时（如本沙箱环境 Yahoo 返回 429）相关标的无数据，对应报告仍会
//!   生成（明细为 N/A），不会崩溃。

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::backtest::write_strategy_reports;
use crate::config::load_config;
use crate::i18n::Lang;
use crate::market::{MarketRouter, load_watchlist, load_watchlist_crypto, load_watchlist_us};
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

/// 在 `watchlist` 上跑完整回测流程：拉取日线 / 分钟线 -> 逐策略回测 -> 写报告。
///
/// `market_label` 仅用于报告标题与日志（如 `"A股"` / `"美股"`）；
/// `warn_on_empty` 控制当两类数据均为空时是否打印限流告警（美股在沙箱环境常因
/// Yahoo 429 返回空，需要提示；A 股一般不需要）。
///
/// 返回「策略 ID -> 报告文件绝对路径」列表。网络不可用时相关标的无数据，
/// 对应报告仍会生成（明细为 N/A），不会崩溃。
async fn generate_reports_for(
    out_dir: &str,
    watchlist: Vec<String>,
    market_label: &str,
    warn_on_empty: bool,
) -> Result<Vec<(String, PathBuf)>> {
    let config = load_config();
    let lang = Lang::from_config(&config.language);
    let strategies = parse_strategy_file("strategy.toml");
    if strategies.is_empty() {
        anyhow::bail!("未解析到任何策略（strategy.toml 缺失或为空）。");
    }

    let router = MarketRouter::new();

    // 日线（DSL 规则 + 形态规则的日线分支）。
    println!(
        "正在拉取 {} 只标的的日线历史（{} 根，复权 {}）…",
        watchlist.len(),
        config.kline_count,
        config.kline_adjust
    );
    let (klines, kerrs) =
        router.fetch_all_klines(&watchlist, &config.kline_adjust, config.kline_count).await;
    if !kerrs.is_empty() {
        eprintln!(
            "⚠️ 以下标的日线行情获取失败: {}",
            kerrs.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(", ")
        );
    }

    // 分钟 K 线（含 timeframe 的规则）。
    let tf_bars = collect_tf_bars(&strategies);
    let intraday = if tf_bars.is_empty() {
        HashMap::new()
    } else {
        println!("正在拉取分钟 K 线（{:?}）…", tf_bars);
        let (intraday, ierrs) = router.fetch_all_intraday(&watchlist, &tf_bars).await;
        if !ierrs.is_empty() {
            eprintln!(
                "⚠️ 以下标的分钟行情获取失败: {}",
                ierrs.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(", ")
            );
        }
        intraday
    };

    let have_daily = klines.values().filter(|s| !s.is_empty()).count();
    let have_intraday = intraday.values().filter(|s| !s.is_empty()).count();
    println!(
        "数据就绪：日线 {} 只、分钟 {} 组。开始生成报告…",
        have_daily, have_intraday
    );
    if warn_on_empty && have_daily == 0 && have_intraday == 0 {
        eprintln!(
            "⚠️  未获取到任何{market_label}历史行情：数据源在本环境可能不可达。\n\
             \t报告仍会生成，但明细为 N/A。请在可访问对应数据源的环境运行本命令以获取真实回测结果。"
        );
    }

    let paths = write_strategy_reports(
        out_dir,
        &strategies,
        &klines,
        &intraday,
        &watchlist,
        &config,
        market_label,
        lang,
    );

    Ok(paths)
}

/// A 股回测（akshare），默认写到 `reports/`。详见 [`generate_reports_for`]。
pub async fn generate_reports(out_dir: &str) -> Result<Vec<(String, PathBuf)>> {
    generate_reports_for(out_dir, load_watchlist(), "A股", false).await
}

/// 美股回测（yfinance-rs / Yahoo Finance），写到 `reports_us/`。详见 [`generate_reports_for`]。
///
/// 数据获取失败时（如本沙箱环境 Yahoo 返回 429）相关标的无数据，对应报告仍会
/// 生成（明细为 N/A），不会崩溃 —— 在可访问 Yahoo 的环境中运行即可获得真实回测结果。
pub async fn generate_reports_us(out_dir: &str) -> Result<Vec<(String, PathBuf)>> {
    generate_reports_for(out_dir, load_watchlist_us(), "美股", true).await
}

/// 加密货币回测（OKX 现货，经 okx-rs 拉取历史 K 线），写到 `reports_crypto/`。
///
/// 对 `watchlist_crypto.txt`（或内置默认加密货币清单）中的交易对，复用全部已配置
/// 策略做回测。OKX 公开接口可达，故加密货币回测在本环境即可获得真实结果。
pub async fn generate_reports_crypto(out_dir: &str) -> Result<Vec<(String, PathBuf)>> {
    let crypto_label_en = "Crypto";
    let crypto_label_zh = "加密货币";
    let label = if Lang::from_config(&load_config().language) == Lang::Zh {
        crypto_label_zh
    } else {
        crypto_label_en
    };
    generate_reports_for(out_dir, load_watchlist_crypto(), label, true).await
}
