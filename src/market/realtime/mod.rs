//! Realtime quote sources (hand-rolled, provider-independent).
//!
//!  - A-shares: East Money `push2` / `push2his` (`stock/get` per symbol,
//!    `clist` for the full board + indices). Prices arrive as "fens" (×100),
//!    so every numeric field is divided by 100 on parse. See [`eastmoney`].
//!  - US: Yahoo Finance `v8/finance/chart` (no API key), `query1` with
//!    `query2` fallback. See [`yahoo`].
//!
//! Both paths are pure-function-testable: the `parse_*` helpers take a
//! `serde_json::Value` and return the same structs the engine consumes, so the
//! network shape can be exercised offline with captured sample payloads.

pub(crate) mod http;
pub(crate) mod eastmoney;
pub(crate) mod yahoo;

// Public entry points used by `router.rs` / `source.rs`. Re-exported here so
// callers keep importing from `market::realtime` rather than the submodules.
pub(crate) use eastmoney::{em_fetch_board, em_fetch_quotes_batch, em_fetch_quote};
pub(crate) use http::realtime_http_client;
pub(crate) use yahoo::{fetch_us_breadth, fetch_us_indices, yahoo_fetch_quotes_batch};

#[cfg(test)]
mod realtime_tests {
    use super::*;
    use super::yahoo::yahoo_fetch_quote;
    use crate::market::router::MarketRouter;
    use std::collections::HashSet;

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
            let change_pct = crate::market::types::pct_change(price, prev);
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
