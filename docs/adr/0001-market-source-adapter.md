# Introduce a MarketSource adapter for the dual data provider

We have two market-data providers — `akshare` for A-shares and `yfinance-rs` for US stocks — that were threaded as a *pair* through `main`, `data_loop`, `fetch_all_*_market`, and `backtest_cli`, with symbol-shape routing (`market_of`) switched inline at every call site. We introduce a `MarketSource` trait (async, object-safe) with two implementations — `AkShareSource` (A-share) and `YfSource` (US) — and a `MarketRouter` that owns both and dispatches `fetch_klines` / `fetch_intraday` by `market_of`. This turns one *hypothetical* seam (`Candle`) into two *real* adapters behind a single interface, removes the inline `market_of` switching (including the leaked re-check in `main.rs` and the dead `AkShareClient` built in `generate_reports_us`), and makes each source unit-testable behind a fake.

**Considered Options**

- (a) `MarketSource` trait + `MarketRouter` owning both sources — **chosen**.
- (b) Keep inline switching on `market_of` at each call site — rejected: leaks the provider pair into every caller and produced a dead client.
- (c) Caller holds `Vec<Box<dyn MarketSource>>` and iterates until one returns data — rejected: pushes routing cost to every call and loses the single-owner clarity.

**Consequences**

- Callers take `&MarketRouter` (or `&dyn MarketSource`) instead of two concrete clients; `MarketRouter::from_sources` injects fakes for tests.
- The A-share-only board snapshot is represented as `fetch_snapshot(&self) -> Option<MarketData>` (US returns `None`); the asymmetry is hidden behind the seam, not duplicated at call sites.
- Error handling stays `-> Vec<Candle>` for this pass (errors logged, empty on failure). Promotion to `Result<Vec<Candle>, _>` is deferred to a later failure-surfacing pass (the silent rule-drop candidate from the architecture review).
