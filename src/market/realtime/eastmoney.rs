//! East Money (A-share) realtime provider: `secid` mapping, single-quote /
//! batch / board fetches, and the `clist`/`stock/get` JSON parsers.
//!
//! Prices arrive as "fens" (×100) and are divided down via [`fen`]/[`fen_opt`].

use reqwest::Client;
use serde_json::Value;
use super::http::fetch_json_fallback;
use crate::market::types::{Spot, IndexSpot, MarketData};

/// Hosts tried in order: `push2` (realtime) then `push2his` (history host,
/// which also serves realtime and is what the sandbox proxy reliably reaches).
const EM_HOSTS: [&str; 2] = ["push2.eastmoney.com", "push2his.eastmoney.com"];

/// East Money expects this `Referer`; set on every East Money request.
const EM_REFERER: &str = "https://quote.eastmoney.com/";

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

/// 东财数值字段以「分」(×100) 返回，统一在此除 100；缺失 / 非数字回退 0.0。
fn fen(v: Option<&Value>) -> f64 {
    v.and_then(|x| x.as_f64()).map(|x| x / 100.0).unwrap_or(0.0)
}

/// 同 [`fen`]，但保留 `Option`：缺失 / 非数字时返回 `None`（用于指数等允许缺值的字段）。
fn fen_opt(v: Option<&Value>) -> Option<f64> {
    v.and_then(|x| x.as_f64()).map(|x| x / 100.0)
}

/// Fetch a single A-share realtime quote (East Money `stock/get`).
/// Returns `None` if both hosts fail or the payload is empty.
pub(crate) async fn em_fetch_quote(http: &Client, code: &str) -> Option<Spot> {
    let secid = em_secid(code);
    let fields = "f43,f57,f58,f59,f60,f169,f170,f46,f47,f48,f49,f171,f15,f16";
    fetch_json_fallback(
        http,
        &EM_HOSTS,
        Some(EM_REFERER),
        |host| {
            format!(
                "https://{}/api/qt/stock/get?secid={}&fields={}",
                host, secid, fields
            )
        },
        parse_em_quote_json,
    )
    .await
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
    fetch_json_fallback(
        http,
        &EM_HOSTS,
        Some(EM_REFERER),
        |host| {
            format!(
                "https://{}/api/qt/clist/get?pn=1&pz={}&po=1&np=1&invt=2&fid=f3\
                 &fs=&secids={}&fields={}",
                host,
                codes.len().max(1),
                secids_str,
                fields
            )
        },
        |v| {
            let s = parse_em_spots(v);
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        },
    )
    .await
    .unwrap_or_default()
}

/// Fetch the full A-share board (all listings) + major indices for the breadth /
/// movers UI. Best-effort: returns whatever each endpoint yields; empty on total
/// failure so the UI degrades instead of crashing.
pub(crate) async fn em_fetch_board(http: &Client) -> MarketData {
    // 1) Full A-share spot board (clist). 全部A股 filter.
    let board_fs = "m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23";
    let fields = "f12,f13,f14,f2,f3,f4,f15,f16,f17,f18,f5,f6";
    let spots = fetch_json_fallback(
        http,
        &EM_HOSTS,
        Some(EM_REFERER),
        |host| {
            format!(
                "https://{}/api/qt/clist/get?pn=1&pz=8000&po=1&np=1&invt=2&fid=f3\
                 &fs={}&fields={}",
                host, board_fs, fields
            )
        },
        |v| {
            let s = parse_em_spots(v);
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        },
    )
    .await
    .unwrap_or_default();

    // 2) Major indices via clist `secids` (one request): 上证指数 / 深证成指 /
    //    创业板指 / 沪深300 / 科创50 / 中小100.
    let index_secids = "1.000001,0.399001,0.399006,1.000300,1.000688,0.399005";
    let idx_fields = "f12,f14,f2,f3";
    let indices = fetch_json_fallback(
        http,
        &EM_HOSTS,
        Some(EM_REFERER),
        |host| {
            format!(
                "https://{}/api/qt/clist/get?pn=1&pz=20&po=1&np=1&invt=2&fid=f3\
                 &fs=&secids={}&fields={}",
                host, index_secids, idx_fields
            )
        },
        |v| {
            let ix = parse_em_indices(v);
            if ix.is_empty() {
                None
            } else {
                Some(ix)
            }
        },
    )
    .await
    .unwrap_or_default();

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
    let latest_price = fen(d.get("f43"));
    let change_amount = fen(d.get("f169"));
    let change_pct = fen(d.get("f170"));
    let open = fen(d.get("f46"));
    let high = fen(d.get("f15"));
    let low = fen(d.get("f16"));
    let prev_close = fen(d.get("f60"));
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
        let latest_price = fen(item.get("f2"));
        let change_pct = fen(item.get("f3"));
        let change_amount = fen(item.get("f4"));
        let high = fen(item.get("f15"));
        let low = fen(item.get("f16"));
        let open = fen(item.get("f17"));
        let prev_close = fen(item.get("f18"));
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
        let latest_price = fen_opt(item.get("f2"));
        let change_pct = fen_opt(item.get("f3"));
        out.push(IndexSpot {
            code,
            name,
            latest_price,
            change_pct,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

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

    // ---- Offline: secid mapping ----
    #[test]
    fn em_secid_mapping() {
        assert_eq!(em_secid("600519"), "1.600519"); // Shanghai
        assert_eq!(em_secid("688167"), "1.688167"); // 科创板 (SH)
        assert_eq!(em_secid("000858"), "0.000858"); // Shenzhen
        assert_eq!(em_secid("300750"), "0.300750"); // 创业板 (SZ)
        assert_eq!(em_secid("830799"), "0.830799"); // 北交所 (BJ→0)
    }
}
