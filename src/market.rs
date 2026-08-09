//! Market data fetching & derived metrics, built on akshare-rs.

use akshare::stock::feature::SpotQuote;
use akshare::stock::zh_index::IndexSpotEm;
use akshare::AkShareClient;

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
