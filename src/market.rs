//! Market data fetching & derived metrics, built on akshare-rs.

use akshare::stock::feature::SpotQuote;
use akshare::stock::zh_index::IndexSpotEm;
use akshare::AkShareClient;

use crate::indicators::Candle;
use chrono::{NaiveDate, NaiveDateTime};
use std::collections::HashMap;

/// Default自选股 (watchlist) — bare 6-digit A-share codes, no market prefix.
pub const DEFAULT_WATCHLIST: &[&str] = &[
    "600519", // 贵州茅台
    "601318", // 中国平安
    "600036", // 招商银行
    "000858", // 五粮液
    "300750", // 宁德时代
    "601899", // 紫金矿业
    "600900", // 长江电力
    "000001", // 平安银行
    "002594", // 比亚迪
    "600276", // 恒瑞医药
];

/// A full market snapshot returned by one refresh cycle.
#[derive(Clone)]
pub struct MarketData {
    pub indices: Vec<IndexSpotEm>,
    pub spots: Vec<SpotQuote>,
}

/// Fetch indices + the full A-share spot board in parallel (best-effort:
/// a failure of one endpoint does not block the other).
pub async fn fetch_market(client: &AkShareClient) -> MarketData {
    let (indices, spots) = tokio::join!(
        client.stock_zh_index_spot_em(),
        client.stock_zh_a_spot_em(),
    );
    MarketData {
        indices: indices.unwrap_or_default(),
        spots: spots.unwrap_or_default(),
    }
}

/// Load the watchlist: prefer `watchlist.txt` in the cwd (one bare code per line),
/// otherwise fall back to [`DEFAULT_WATCHLIST`].
pub fn load_watchlist() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist.txt") {
        let parsed: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    DEFAULT_WATCHLIST.iter().map(|s| s.to_string()).collect()
}

/// Market breadth derived from the full spot board.
#[derive(Debug, Clone, Copy)]
pub struct Breadth {
    pub up: usize,
    pub down: usize,
    pub flat: usize,
    pub limit_up: usize,
    pub limit_down: usize,
    pub total: usize,
}

impl Breadth {
    pub fn compute(spots: &[SpotQuote]) -> Breadth {
        let mut b = Breadth {
            up: 0,
            down: 0,
            flat: 0,
            limit_up: 0,
            limit_down: 0,
            total: spots.len(),
        };
        for s in spots {
            if s.latest_price <= 0.0 {
                continue;
            }
            if s.change_pct > 0.0 {
                b.up += 1;
            } else if s.change_pct < 0.0 {
                b.down += 1;
            } else {
                b.flat += 1;
            }
            // 涨停/跌停: 主板 ~10%, 创业板/科创板 ~20% — count >= 9.8% as a close approximation.
            if s.change_pct >= 9.8 {
                b.limit_up += 1;
            } else if s.change_pct <= -9.8 {
                b.limit_down += 1;
            }
        }
        b
    }
}

/// Top `n` gainers (descending change_pct).
pub fn top_gainers(spots: &[SpotQuote], n: usize) -> Vec<SpotQuote> {
    sorted(spots, true).into_iter().take(n).collect()
}

/// Top `n` losers (ascending change_pct).
pub fn top_losers(spots: &[SpotQuote], n: usize) -> Vec<SpotQuote> {
    sorted(spots, false).into_iter().take(n).collect()
}

fn sorted(spots: &[SpotQuote], desc: bool) -> Vec<SpotQuote> {
    let mut v: Vec<SpotQuote> = spots
        .iter()
        .filter(|s| s.latest_price > 0.0)
        .cloned()
        .collect();
    v.sort_by(|a, b| {
        let ord = a
            .change_pct
            .partial_cmp(&b.change_pct)
            .unwrap_or(std::cmp::Ordering::Equal);
        if desc {
            ord.reverse()
        } else {
            ord
        }
    });
    v
}

/// Resolve a watchlist entry to its live spot quote (exact code match).
pub fn find_spot<'a>(spots: &'a [SpotQuote], code: &str) -> Option<&'a SpotQuote> {
    spots.iter().find(|s| s.code == code)
}

/// 解析 K 线日期：兼容 "2024-01-02" 与 "20240102" 两种格式（日线，时间归零）。
fn parse_kline_date(s: &str) -> Option<NaiveDateTime> {
    let d = if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        d
    } else if let Ok(d) = NaiveDate::parse_from_str(s, "%Y%m%d") {
        d
    } else {
        return None;
    };
    d.and_hms_opt(0, 0, 0)
}

/// 解析分钟 K 线时间（Sina 格式 "2024-01-02 09:30:00"）。
fn parse_minute_datetime(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

/// 拉取单只标的的历史 K 线并映射为 `Candle`（按日期升序）。
pub async fn fetch_klines(client: &AkShareClient, code: &str, adjust: &str, count: usize) -> Vec<Candle> {
    let points = match client.a_share_candles(code, adjust, count).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("K线获取失败 {}: {}", code, e);
            return Vec::new();
        }
    };
    let mut candles: Vec<Candle> = points
        .into_iter()
        .filter_map(|p| {
            let date = parse_kline_date(&p.trade_date)?;
            Some(Candle {
                date,
                open: p.open,
                high: p.high,
                low: p.low,
                close: p.close,
                volume: p.volume as f64,
            })
        })
        .collect();
    // 升序排序，确保指标计算顺序正确
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    candles
}

/// 拉取单只标的的分钟 K 线（akshare `stock_zh_a_minute`），映射为 `Candle`（升序）。
/// `period` ∈ {"1","5","15","30","60"}。`count` 为期望保留根数；Sina 历史有限，
/// 实际返回以可用数据为准（1H 约 36 根，不足 `count` 不报错）。
pub async fn fetch_minute_klines(
    client: &AkShareClient,
    code: &str,
    period: &str,
    count: usize,
) -> Vec<Candle> {
    let bars = match client.stock_zh_a_minute(code, period).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("分钟K线失败 {}@{}: {}", code, period, e);
            return Vec::new();
        }
    };
    let mut candles: Vec<Candle> = bars
        .into_iter()
        .filter_map(|b| {
            let dt = parse_minute_datetime(&b.datetime)?;
            Some(Candle {
                date: dt,
                open: b.open,
                high: b.high,
                low: b.low,
                close: b.close,
                volume: b.volume,
            })
        })
        .collect();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    if candles.len() > count {
        candles = candles.split_off(candles.len() - count);
    }
    candles
}

/// 批量拉取多只标的、多个 (timeframe, bars) 组合的分钟 K 线。
/// 结果以 `"{code}@{tf}"` 为键，匹配 `SignalEngine::evaluate` 的 intraday map。
pub async fn fetch_all_intraday(
    client: &AkShareClient,
    codes: &[String],
    tf_bars: &[(String, usize)],
) -> HashMap<String, Vec<Candle>> {
    let mut map = HashMap::new();
    for (tf, bars) in tf_bars {
        for code in codes {
            let series = fetch_minute_klines(client, code, tf, *bars).await;
            if !series.is_empty() {
                map.insert(format!("{}@{}", code, tf), series);
            }
        }
    }
    map
}

/// 批量拉取多只标的的 K 线。
pub async fn fetch_all_klines(
    client: &AkShareClient,
    codes: &[String],
    adjust: &str,
    count: usize,
) -> HashMap<String, Vec<Candle>> {
    let mut map = HashMap::with_capacity(codes.len());
    for code in codes {
        let k = fetch_klines(client, code, adjust, count).await;
        map.insert(code.clone(), k);
    }
    map
}
