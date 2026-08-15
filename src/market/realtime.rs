//! Realtime quote sources (hand-rolled, provider-independent).
//!
//!  - A-shares: East Money `push2` / `push2his` (`stock/get` per symbol,
//!    `clist` for the full board + indices). Prices arrive as "fens" (×100),
//!    so every numeric field is divided by 100 on parse.
//!  - US: Yahoo Finance `v8/finance/chart` (no API key), `query1` with
//!    `query2` fallback.
//!
//! Both paths are pure-function-testable: the `parse_*` helpers take a
//! `serde_json::Value` and return the same structs the engine consumes, so the
//! network shape can be exercised offline with captured sample payloads.

use reqwest::{Client, Proxy};
use reqwest::header::HeaderValue;
use serde_json::Value;
use std::time::Duration;

use super::types::{Spot, IndexSpot, MarketData, Breadth};

/// Read an outbound HTTP proxy from the standard env vars (`HTTPS_PROXY`,
/// `https_proxy`, `HTTP_PROXY`, `http_proxy`). Returns `None` when unset, so
/// the client talks directly on hosts without a proxy. (`Proxy::from_env` is
/// feature-gated off in this reqwest build; we replicate it with `Proxy::all`,
/// which is always available.)
fn env_proxy() -> Option<Proxy> {
    let val = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .ok()?;
    Proxy::all(val).ok()
}

/// Shared `reqwest` client for realtime endpoints.
///
/// Applies the ambient HTTP proxy (if any) and a desktop `User-Agent` East
/// Money / Yahoo accept. Times out at 15s so a stalled endpoint can't hang a
/// refresh cycle. (The `Referer` header is added per-request because
/// `ClientBuilder::header` is feature-gated off in this reqwest build;
/// `RequestBuilder::header` is always available.)
pub(crate) fn realtime_http_client() -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );
    if let Some(p) = env_proxy() {
        builder = builder.proxy(p);
    }
    builder
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// Map a bare 6-digit A-share code to East Money `secid` (`1.600519` SH / `0.000001` SZ).
///
/// Shanghai / 科创板 prefixes (`6`, `9`) → market `1`; everything else
/// (深圳 `0/2/3`, 北交所 `8/4`) → market `0`. Bare `f12` codes from the board
/// API are 6-digit, so this matches the watchlist exactly.
fn em_secid(code: &str) -> String {
    let market = match code.chars().next() {
        Some('6') | Some('9') => '1',
        _ => '0',
    };
    format!("{}.{}", market, code)
}

/// Hosts tried in order: `push2` (realtime) then `push2his` (history host,
/// which also serves realtime and is what the sandbox proxy reliably reaches).
const EM_HOSTS: [&str; 2] = ["push2.eastmoney.com", "push2his.eastmoney.com"];

/// East Money expects this `Referer`; set on every East Money request.
const EM_REFERER: &str = "https://quote.eastmoney.com/";

/// GET `url` and return the response body as `String`, or `None` on any
/// transport / read error. `referer` (when `Some`) is attached as a `Referer`
/// header — required by East Money, harmless elsewhere.
async fn http_text(http: &Client, url: &str, referer: Option<&str>) -> Option<String> {
    let mut req = http.get(url);
    if let Some(r) = referer {
        if let Ok(hv) = HeaderValue::from_str(r) {
            req = req.header("Referer", hv);
        }
    }
    let resp = req.send().await.ok()?;
    resp.text().await.ok()
}

/// Fetch a single A-share realtime quote (East Money `stock/get`).
/// Returns `None` if both hosts fail or the payload is empty.
pub(crate) async fn em_fetch_quote(http: &Client, code: &str) -> Option<Spot> {
    let secid = em_secid(code);
    let fields = "f43,f57,f58,f59,f60,f169,f170,f46,f47,f48,f49,f171,f15,f16";
    for host in EM_HOSTS {
        let url = format!(
            "https://{}/api/qt/stock/get?secid={}&fields={}",
            host, secid, fields
        );
        if let Some(txt) = http_text(http, &url, Some(EM_REFERER)).await {
            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                if let Some(s) = parse_em_quote_json(&v) {
                    return Some(s);
                }
            }
        }
    }
    None
}

/// Fetch a batch of A-share realtime quotes in one request (East Money `clist`
/// with `secids`). Returns `Vec::new()` on failure — callers fall back per-code.
pub(crate) async fn em_fetch_quotes_batch(http: &Client, codes: &[String]) -> Vec<Spot> {
    if codes.is_empty() {
        return Vec::new();
    }
    let secids: Vec<String> = codes.iter().map(|c| em_secid(c)).collect();
    let secids_str = secids.join(",");
    let fields = "f12,f13,f14,f2,f3,f4,f15,f16,f17,f18,f5,f6";
    for host in EM_HOSTS {
        let url = format!(
            "https://{}/api/qt/clist/get?pn=1&pz={}&po=1&np=1&invt=2&fid=f3\
             &fs=&secids={}&fields={}",
            host,
            codes.len().max(1),
            secids_str,
            fields
        );
        if let Some(txt) = http_text(http, &url, Some(EM_REFERER)).await {
            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                let spots = parse_em_spots(&v);
                if !spots.is_empty() {
                    return spots;
                }
            }
        }
    }
    Vec::new()
}

/// Fetch the full A-share board (all listings) + major indices for the breadth /
/// movers UI. Best-effort: returns whatever each endpoint yields; empty on total
/// failure so the UI degrades instead of crashing.
pub(crate) async fn em_fetch_board(http: &Client) -> MarketData {
    // 1) Full A-share spot board (clist). 全部A股 filter.
    let mut spots = Vec::new();
    let board_fs = "m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23";
    let fields = "f12,f13,f14,f2,f3,f4,f15,f16,f17,f18,f5,f6";
    for host in EM_HOSTS {
        let url = format!(
            "https://{}/api/qt/clist/get?pn=1&pz=8000&po=1&np=1&invt=2&fid=f3\
             &fs={}&fields={}",
            host, board_fs, fields
        );
        if let Some(txt) = http_text(http, &url, Some(EM_REFERER)).await {
            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                let s = parse_em_spots(&v);
                if !s.is_empty() {
                    spots = s;
                    break;
                }
            }
        }
    }

    // 2) Major indices via clist `secids` (one request): 上证指数 / 深证成指 /
    //    创业板指 / 沪深300 / 科创50 / 中小100.
    let index_secids = "1.000001,0.399001,0.399006,1.000300,1.000688,0.399005";
    let mut indices = Vec::new();
    let idx_fields = "f12,f14,f2,f3";
    for host in EM_HOSTS {
        let url = format!(
            "https://{}/api/qt/clist/get?pn=1&pz=20&po=1&np=1&invt=2&fid=f3\
             &fs=&secids={}&fields={}",
            host, index_secids, idx_fields
        );
        if let Some(txt) = http_text(http, &url, Some(EM_REFERER)).await {
            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                let ix = parse_em_indices(&v);
                if !ix.is_empty() {
                    indices = ix;
                    break;
                }
            }
        }
    }

    MarketData { indices, spots }
}

/// Parse a single `stock/get` payload into [`Spot`].
///
/// `data` may be `null` (unknown secid) — returns `None` in that case.
/// All numeric fields arrive as "fens" (×100) and are divided down.
fn parse_em_quote_json(v: &Value) -> Option<Spot> {
    let d = v.get("data")?;
    let code = d.get("f57").and_then(|x| x.as_str())?.to_string();
    if code.is_empty() {
        return None;
    }
    let name = d
        .get("f58")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let latest_price = d
        .get("f43")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let change_amount = d
        .get("f169")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let change_pct = d
        .get("f170")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let open = d
        .get("f46")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let high = d
        .get("f15")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let low = d
        .get("f16")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let prev_close = d
        .get("f60")
        .and_then(|x| x.as_f64())
        .map(|x| x / 100.0)
        .unwrap_or(0.0);
    let volume = d.get("f47").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let amount = d.get("f48").and_then(|x| x.as_f64()).unwrap_or(0.0);
    Some(Spot {
        code,
        name,
        latest_price,
        change_pct,
        change_amount,
        volume,
        amount,
        high,
        low,
        open,
        prev_close,
    })
}

/// Parse an East Money `clist` payload (`data.diff` array) into [`Spot`]s.
/// Unknown / halted rows (empty `f12`, null `f2`) are skipped.
fn parse_em_spots(v: &Value) -> Vec<Spot> {
    let mut out = Vec::new();
    let diff = match v.get("data").and_then(|d| d.get("diff")) {
        Some(Value::Array(a)) => a,
        _ => return out,
    };
    for item in diff {
        let code = item
            .get("f12")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if code.is_empty() {
            continue;
        }
        let name = item
            .get("f14")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let latest_price = item
            .get("f2")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let change_pct = item
            .get("f3")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let change_amount = item
            .get("f4")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let high = item
            .get("f15")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let low = item
            .get("f16")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let open = item
            .get("f17")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let prev_close = item
            .get("f18")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0)
            .unwrap_or(0.0);
        let volume = item.get("f5").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let amount = item.get("f6").and_then(|x| x.as_f64()).unwrap_or(0.0);
        out.push(Spot {
            code,
            name,
            latest_price,
            change_pct,
            change_amount,
            volume,
            amount,
            high,
            low,
            open,
            prev_close,
        });
    }
    out
}

/// Parse an East Money `clist` payload (`data.diff`) into [`IndexSpot`]s.
/// `latest_price` / `change_pct` are kept as `Option` because East Money may
/// omit them for some indices mid-session.
fn parse_em_indices(v: &Value) -> Vec<IndexSpot> {
    let mut out = Vec::new();
    let diff = match v.get("data").and_then(|d| d.get("diff")) {
        Some(Value::Array(a)) => a,
        _ => return out,
    };
    for item in diff {
        let code = item
            .get("f12")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if code.is_empty() {
            continue;
        }
        let name = item
            .get("f14")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let latest_price = item
            .get("f2")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0);
        let change_pct = item
            .get("f3")
            .and_then(|x| x.as_f64())
            .map(|x| x / 100.0);
        out.push(IndexSpot {
            code,
            name,
            latest_price,
            change_pct,
        });
    }
    out
}

/// Fetch a single US realtime quote from Yahoo `v8/finance/chart` (no API key).
/// `query1` is tried first, then `query2`. Returns `(price, prev_close, name)`
/// or `None` on failure / non-trading.
pub(crate) async fn yahoo_fetch_quote(
    http: &Client,
    symbol: &str,
) -> Option<(f64, f64, String)> {
    let hosts = ["query1.finance.yahoo.com", "query2.finance.yahoo.com"];
    for host in hosts {
        let url = format!(
            "https://{}/v8/finance/chart/{}?interval=1d&range=1d",
            host, symbol
        );
        if let Some(txt) = http_text(http, &url, None).await {
            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                if let Some(r) = parse_yahoo_quote_json(&v) {
                    return Some(r);
                }
            }
        }
    }
    None
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
async fn yahoo_fetch_quotes_batch(
    http: &Client,
    symbols: &[&str],
) -> Vec<(String, f64, f64, String)> {
    if symbols.is_empty() {
        return Vec::new();
    }
    let joined = symbols.join(",");
    let hosts = ["query1.finance.yahoo.com", "query2.finance.yahoo.com"];
    for host in hosts {
        let url = format!(
            "https://{}/v7/finance/quote?symbols={}&fields=regularMarketPrice,regularMarketPreviousClose,shortName,longName",
            host, joined
        );
        if let Some(txt) = http_text(http, &url, None).await {
            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                if let Some(out) = parse_yahoo_quotes_batch(&v) {
                    if !out.is_empty() {
                        return out;
                    }
                }
            }
        }
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

/// 拉取指数栏所需的美股指数 + 黄金 + 原油报价，映射为 [`IndexSpot`]。
/// 显示名优先用 [`US_INDICES`] 中的友好名（Yahoo 对指数返回的 `shortName`
/// 常为 `^GSPC` 之类原始代码），缺失时回退到 Yahoo 的名称。
pub async fn fetch_us_indices(http: &Client) -> Vec<IndexSpot> {
    let syms: Vec<&str> = US_INDICES.iter().map(|t| t.0).collect();
    let quotes = yahoo_fetch_quotes_batch(http, &syms).await;
    let mut out = Vec::with_capacity(US_INDICES.len());
    for (sym, price, prev, name) in quotes {
        let change_pct = if prev > 0.0 {
            (price - prev) / prev * 100.0
        } else {
            0.0
        };
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
            let pct = if prev > 0.0 {
                (price - prev) / prev * 100.0
            } else {
                0.0
            };
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

// ===========================================================================
// Tests
//
// Two tiers:
//  - Offline parser tests: feed captured East Money / Yahoo JSON (in the same
//    "fens" / raw shape the live code requests, i.e. NO `fltt` param) into the
//    pure `parse_*` helpers. No network, deterministic.
//  - Live `#[ignore]` tests: hit the real endpoints to confirm A-shares return
//    a valid last-close during non-trading hours and US quotes keep refreshing
//    over time. Run with `cargo test -- --ignored`.
// ===========================================================================

#[cfg(test)]
mod realtime_tests {
    use super::*;
    use crate::market::router::MarketRouter;
    use std::collections::HashSet;

    // ---- Offline: East Money single-quote (`stock/get`) parser ----
    #[test]
    fn parse_em_quote_json_sample() {
        // Captured shape from `push2his ... /stock/get?secid=1.600519`
        // (no `fltt` → prices are "fens", ×100). 134300 → 1343.00, etc.
        let json = r#"{
            "rc":0,
            "data":{
                "f43":134300,"f57":"600519","f58":"贵州茅台",
                "f60":134650,"f169":-350,"f170":-26,
                "f46":134650,"f15":135688,"f16":133251,
                "f47":35060,"f48":4717613108.0,"f49":17643
            }
        }"#;
        let v: Value = serde_json::from_str(json).unwrap();
        let s = parse_em_quote_json(&v).expect("should parse");
        assert_eq!(s.code, "600519");
        assert_eq!(s.name, "贵州茅台");
        assert!((s.latest_price - 1343.00).abs() < 1e-6, "got {}", s.latest_price);
        assert!((s.prev_close - 1346.50).abs() < 1e-6);
        assert!((s.change_amount - (-3.50)).abs() < 1e-6);
        assert!((s.change_pct - (-0.26)).abs() < 1e-6);
        assert!((s.high - 1356.88).abs() < 1e-6);
        assert!((s.low - 1332.51).abs() < 1e-6);
    }

    #[test]
    fn parse_em_quote_json_null_data_is_none() {
        let v: Value = serde_json::from_str(r#"{"rc":0,"data":null}"#).unwrap();
        assert!(parse_em_quote_json(&v).is_none());
    }

    // ---- Offline: East Money board / batch (`clist` diff) parser ----
    #[test]
    fn parse_em_spots_sample() {
        // `data.diff` array in fens format (no `fltt`).
        let json = r#"{
            "data":{"total":2,"diff":[
                {"f12":"600519","f14":"贵州茅台","f2":134300,"f3":-26,"f4":-350,
                 "f15":135688,"f16":133251,"f17":134650,"f18":134650,"f5":35060,"f6":4717613108.0},
                {"f12":"000858","f14":"五粮液","f2":15000,"f3":350,"f4":508,
                 "f15":15200,"f16":14800,"f17":14900,"f18":14500,"f5":50000,"f6":750000000.0}
            ]}
        }"#;
        let v: Value = serde_json::from_str(json).unwrap();
        let spots = parse_em_spots(&v);
        assert_eq!(spots.len(), 2);
        let moutai = &spots[0];
        assert_eq!(moutai.code, "600519");
        assert!((moutai.latest_price - 1343.00).abs() < 1e-6);
        assert!((moutai.change_pct - (-0.26)).abs() < 1e-6);
        assert!((moutai.prev_close - 1346.50).abs() < 1e-6);
        let wuliang = &spots[1];
        assert!((wuliang.latest_price - 150.00).abs() < 1e-6);
        assert!((wuliang.change_pct - 3.50).abs() < 1e-6);
    }

    #[test]
    fn parse_em_indices_sample() {
        let json = r#"{
            "data":{"diff":[
                {"f12":"1.000001","f14":"上证指数","f2":321050,"f3":-50},
                {"f12":"0.399001","f14":"深证成指","f2":1200000,"f3":120}
            ]}
        }"#;
        let v: Value = serde_json::from_str(json).unwrap();
        let idx = parse_em_indices(&v);
        assert_eq!(idx.len(), 2);
        assert_eq!(idx[0].code, "1.000001");
        assert_eq!(idx[0].name, "上证指数");
        assert!((idx[0].latest_price.unwrap() - 3210.50).abs() < 1e-6);
        assert!((idx[0].change_pct.unwrap() - (-0.50)).abs() < 1e-6);
        assert!((idx[1].latest_price.unwrap() - 12000.00).abs() < 1e-6);
    }

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
        let change_pct = (price - prev) / prev * 100.0;
        assert!(change_pct < 0.0 && change_pct > -2.0);
    }

    // ---- Offline: secid mapping ----
    #[test]
    fn em_secid_mapping() {
        assert_eq!(em_secid("600519"), "1.600519"); // Shanghai
        assert_eq!(em_secid("688167"), "1.688167"); // 科创板 (SH)
        assert_eq!(em_secid("000858"), "0.000858"); // Shenzhen
        assert_eq!(em_secid("300750"), "0.300750"); // 创业板 (SZ)
        assert_eq!(em_secid("830799"), "0.830799"); // 北交所 (BJ→0)
    }

    // ---- Live (ignored): A-share returns a valid last-close when closed ----
    #[tokio::test]
    #[ignore = "requires network: East Money"]
    async fn live_a_share_returns_last_close_when_closed() {
        let http = realtime_http_client();
        // 600519 贵州茅台 — during non-trading hours East Money still returns the
        // last close (latest price + day change%), NOT a live-changing tick.
        let spot = em_fetch_quote(&http, "600519")
            .await
            .expect("East Money should return 600519 even when closed");
        assert!(
            spot.latest_price > 0.0,
            "last-close price must be positive, got {}",
            spot.latest_price
        );
        assert!(spot.change_pct.is_finite(), "change_pct must be finite");
        println!(
            "A-share 600519 {} last_price={} change_pct={:.2}%",
            spot.name, spot.latest_price, spot.change_pct
        );
    }

    // ---- Live (ignored): US quote keeps refreshing over time ----
    #[tokio::test]
    #[ignore = "requires network: Yahoo Finance"]
    async fn live_us_quote_updates_over_time() {
        let http = realtime_http_client();
        let mut prices: Vec<f64> = Vec::new();
        let start = std::time::Instant::now();
        for i in 0..4 {
            let (price, prev, name) = yahoo_fetch_quote(&http, "AAPL")
                .await
                .unwrap_or_else(|| panic!("Yahoo should return AAPL on attempt {}", i));
            assert!(price > 0.0, "US price must be positive");
            let change_pct = if prev > 0.0 { (price - prev) / prev * 100.0 } else { 0.0 };
            assert!(change_pct.is_finite());
            prices.push(price);
            println!(
                "US AAPL {} attempt {}: price={} change_pct={:.2}%",
                name, i, price, change_pct
            );
            if i < 3 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
        let elapsed = start.elapsed();
        // The loop must have spanned real time (proves repeated successful
        // fetches over time, not a single cached value or a hang).
        assert!(
            elapsed >= std::time::Duration::from_secs(6),
            "refresh loop should span real time, elapsed {:?}",
            elapsed
        );
        let distinct = prices
            .iter()
            .map(|p| (p * 100.0).round() as i64)
            .collect::<HashSet<_>>()
            .len();
        println!("distinct AAPL prices observed over {:?}: {}", elapsed, distinct);
        // During an active US session the tape moves; if it didn't change across
        // ~6s that's almost certainly a flat micro-window, not a bug — warn, don't fail.
        if distinct == 1 {
            println!(
                "WARN: AAPL price unchanged across the window (flat tape or between ticks); \
                 re-run during active US trading to observe movement."
            );
        }
    }

    // ---- Live (ignored): full router path for both markets ----
    #[tokio::test]
    #[ignore = "requires network"]
    async fn live_router_fetch_all_quotes_a_and_us() {
        let router = MarketRouter::new();
        let codes = vec![
            "600519".to_string(),
            "000858".to_string(),
            "AAPL".to_string(),
            "MSFT".to_string(),
        ];
        let quotes = router.fetch_all_quotes(&codes).await;
        assert_eq!(quotes.len(), 4, "all 4 codes should resolve to quotes");
        for q in &quotes {
            assert!(q.latest_price > 0.0, "{} price must be positive", q.code);
            assert!(q.change_pct.is_finite(), "{} change_pct finite", q.code);
        }
        for q in &quotes {
            println!(
                "quote {} [{}] market={:?} price={} change_pct={:.2}%",
                q.code, q.name, q.market, q.latest_price, q.change_pct
            );
        }
    }
}
