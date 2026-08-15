//! Yahoo Finance (US) realtime provider: single-quote / batch fetches, the
//! `v8/finance/chart` / `v7/finance/quote` JSON parsers, and the index-bar /
//! breadth baskets.

use reqwest::Client;
use serde_json::Value;
use super::http::fetch_json_fallback;
use crate::market::types::{IndexSpot, Breadth, pct_change};

/// 指数栏展示的美股指数 + 黄金 + 原油（Yahoo ticker, 友好显示名）。
pub const US_INDICES: &[(&str, &str)] = &[
    ("^GSPC", "S&P 500"),
    ("^IXIC", "Nasdaq"),
    ("^DJI", "Dow"),
    ("^RUT", "Russell 2K"),
    ("^VIX", "VIX"),
    ("GC=F", "Gold"),
    ("CL=F", "WTI Oil"),
];

/// 美股市场广度样本篮子：覆盖主要板块的大盘股，约 70 只，用于统计涨跌家数。
/// 广度按该篮子的「涨 / 跌 / 平」家数近似全市场情绪（每 60s 批量刷新一次）。
pub const US_BREADTH_BASKET: &[&str] = &[
    "AAPL", "MSFT", "AMZN", "GOOGL", "GOOG", "META", "NVDA", "TSLA", "BRK-B", "JPM",
    "V", "UNH", "XOM", "JNJ", "WMT", "MA", "PG", "HD", "CVX", "KO",
    "PEP", "ABBV", "PFE", "BAC", "COST", "DIS", "CSCO", "TMO", "MCD", "ABT",
    "WFC", "LIN", "AMD", "ORCL", "CRM", "ADBE", "NKE", "TXN", "AMGN", "INTC",
    "QCOM", "IBM", "CAT", "GE", "UPS", "BA", "GS", "MS", "C", "HON",
    "LOW", "SPGI", "BKNG", "T", "VZ", "PLD", "AVGO", "NFLX", "MRK", "LLY",
    "UNP", "RTX", "SCHW", "BLK", "DE", "MDT", "PM", "NEE", "DHR", "TMUS",
];

/// Fetch a single US realtime quote from Yahoo `v8/finance/chart` (no API key).
/// `query1` is tried first, then `query2`. Returns `(price, prev_close, name)`
/// or `None` on failure / non-trading.
pub(crate) async fn yahoo_fetch_quote(
    http: &Client,
    symbol: &str,
) -> Option<(f64, f64, String)> {
    let hosts = ["query1.finance.yahoo.com", "query2.finance.yahoo.com"];
    fetch_json_fallback(
        http,
        &hosts,
        None,
        |host| {
            format!(
                "https://{}/v8/finance/chart/{}?interval=1d&range=1d",
                host, symbol
            )
        },
        parse_yahoo_quote_json,
    )
    .await
}

/// Parse a Yahoo `v8/finance/chart` payload into `(price, prev_close, name)`.
/// `regularMarketPrice` is the latest; `chartPreviousClose` (falling back to
/// `previousClose`) anchors the change %. Returns `None` when price is missing
/// or non-positive.
fn parse_yahoo_quote_json(v: &Value) -> Option<(f64, f64, String)> {
    let result = v
        .get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())?;
    let first = result.first()?;
    let meta = first.get("meta")?;
    let price = meta.get("regularMarketPrice").and_then(|x| x.as_f64())?;
    if price <= 0.0 {
        return None;
    }
    let prev = meta
        .get("chartPreviousClose")
        .and_then(|x| x.as_f64())
        .or_else(|| meta.get("previousClose").and_then(|x| x.as_f64()))
        .unwrap_or(0.0);
    let name = meta
        .get("shortName")
        .and_then(|x| x.as_str())
        .or_else(|| meta.get("longName").and_then(|x| x.as_str()))
        .unwrap_or("")
        .to_string();
    Some((price, prev, name))
}

/// 批量拉取美股实时报价（Yahoo `v7/finance/quote`，单次最多约 50 只）。
/// 返回 `(symbol, price, prev_close, name)`；失败 / 价格无效的条目跳过。
/// 用于指数栏与美股市场广度篮子，避免逐代码请求带来的速率限制。
///
/// `v7/finance/quote` 经常因需要 crumb cookie 或被限流而返回空 `result`，
/// 此时逐代码回退到无需 crumb 的 `v8/finance/chart` 单点接口（与美股自选股
/// 复用同一路径），保证指数栏与市场广度在批量接口失效时仍能取到数据。
pub(crate) async fn yahoo_fetch_quotes_batch(
    http: &Client,
    symbols: &[&str],
) -> Vec<(String, f64, f64, String)> {
    if symbols.is_empty() {
        return Vec::new();
    }
    let joined = symbols.join(",");
    let hosts = ["query1.finance.yahoo.com", "query2.finance.yahoo.com"];
    let batch = fetch_json_fallback(
        http,
        &hosts,
        None,
        |host| {
            format!(
                "https://{}/v7/finance/quote?symbols={}&fields=regularMarketPrice,regularMarketPreviousClose,shortName,longName",
                host, joined
            )
        },
        |v| parse_yahoo_quotes_batch(v).filter(|x| !x.is_empty()),
    )
    .await;
    if let Some(out) = batch {
        return out;
    }
    // 批量接口无数据（被限流 / 需 crumb），逐代码回退到 v8/finance/chart。
    let mut out = Vec::with_capacity(symbols.len());
    for sym in symbols {
        if let Some((price, prev, name)) = yahoo_fetch_quote(http, sym).await {
            out.push(((*sym).to_string(), price, prev, name));
        }
    }
    out
}

/// 解析 Yahoo `v7/finance/quote` 的 `quoteResponse.result` 数组为
/// `(symbol, price, prev_close, name)`。价格缺失 / 非正时跳过该条目。
fn parse_yahoo_quotes_batch(v: &Value) -> Option<Vec<(String, f64, f64, String)>> {
    let arr = v
        .get("quoteResponse")
        .and_then(|q| q.get("result"))
        .and_then(|r| r.as_array())?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let symbol = item
            .get("symbol")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if symbol.is_empty() {
            continue;
        }
        let price = item
            .get("regularMarketPrice")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0);
        if price <= 0.0 {
            continue;
        }
        let prev = item
            .get("regularMarketPreviousClose")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0);
        let name = item
            .get("shortName")
            .and_then(|x| x.as_str())
            .or_else(|| item.get("longName").and_then(|x| x.as_str()))
            .unwrap_or("")
            .to_string();
        out.push((symbol, price, prev, name));
    }
    Some(out)
}

/// 拉取指数栏所需的美股指数 + 黄金 + 原油报价，映射为 [`IndexSpot`]。
/// 显示名优先用 [`US_INDICES`] 中的友好名（Yahoo 对指数返回的 `shortName`
/// 常为 `^GSPC` 之类原始代码），缺失时回退到 Yahoo 的名称。
pub async fn fetch_us_indices(http: &Client) -> Vec<IndexSpot> {
    let syms: Vec<&str> = US_INDICES.iter().map(|t| t.0).collect();
    let quotes = yahoo_fetch_quotes_batch(http, &syms).await;
    let mut out = Vec::with_capacity(US_INDICES.len());
    for (sym, price, prev, name) in quotes {
        let change_pct = pct_change(price, prev);
        let disp = US_INDICES
            .iter()
            .find(|t| t.0 == sym.as_str())
            .map(|t| t.1.to_string())
            .unwrap_or(name);
        out.push(IndexSpot {
            code: sym,
            name: disp,
            latest_price: Some(price),
            change_pct: Some(change_pct),
        });
    }
    out
}

/// 拉取美股市场广度：对 [`US_BREADTH_BASKET`] 样本分批批量报价，
/// 统计涨 / 跌 / 平家数，并把 `>= +1.9%` / `<= -1.9%` 记为「强涨 / 强跌」。
pub async fn fetch_us_breadth(http: &Client) -> Breadth {
    let mut b = Breadth {
        up: 0,
        down: 0,
        flat: 0,
        limit_up: 0,
        limit_down: 0,
        total: 0,
    };
    for chunk in US_BREADTH_BASKET.chunks(40) {
        let quotes = yahoo_fetch_quotes_batch(http, chunk).await;
        for (_sym, price, prev, _name) in quotes {
            let pct = pct_change(price, prev);
            b.total += 1;
            if pct > 0.0 {
                b.up += 1;
            } else if pct < 0.0 {
                b.down += 1;
            } else {
                b.flat += 1;
            }
            if pct >= 1.9 {
                b.limit_up += 1;
            } else if pct <= -1.9 {
                b.limit_down += 1;
            }
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // ---- Offline: Yahoo `v8/finance/chart` parser ----
    #[test]
    fn parse_yahoo_quote_json_sample() {
        let json = r#"{
            "chart":{"result":[{
                "meta":{
                    "regularMarketPrice":304.91,
                    "chartPreviousClose":308.26,
                    "regularMarketTime":1786478401,
                    "shortName":"Apple Inc.",
                    "longName":"Apple Inc."
                },
                "timestamp":[1786455000],
                "indicators":{"quote":[{"close":[304.9100036621094]}]}
            }],"error":null}
        }"#;
        let v: Value = serde_json::from_str(json).unwrap();
        let (price, prev, name) = parse_yahoo_quote_json(&v).expect("should parse");
        assert!((price - 304.91).abs() < 1e-6, "got {}", price);
        assert!((prev - 308.26).abs() < 1e-6);
        assert_eq!(name, "Apple Inc.");
        // change% derived by the caller: (304.91-308.26)/308.26*100 ≈ -1.087
        let change_pct = pct_change(price, prev);
        assert!(change_pct < 0.0 && change_pct > -2.0);
    }
}
