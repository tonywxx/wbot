# Replace the realtime quote path with self-rolled East Money (A) + Yahoo v8 (US)

`akshare-rs` (A-shares) and `yfinance-rs` (US) both fail to return **realtime** prices, while their historical K-line methods still work. We replace only the realtime branch — `MarketSource::fetch_quotes` / `fetch_snapshot` — with hand-rolled `reqwest` clients that hit the providers' public HTTP endpoints directly, mirroring the existing `OkxSource` (which already hand-rolls OKX with no third-party crate). Historical K-lines (`fetch_klines` / `fetch_intraday`) and the backtest reports stay on `akshare-rs` / `yfinance-rs`.

**Considered Options — US realtime source**
- (a) Yahoo `v8/finance/chart` (no key) — **chosen**: zero-config, verified working through the same upstream `yfinance` used before, returns `regularMarketPrice`; ~15 min delay unauthenticated. Batch alternative `v7/finance/quote` (verify during implementation) returns `regularMarketChangePercent` per symbol in one call.
- (b) Finnhub free token — rejected for now: true realtime (60/min) but needs a free API key in config; more setup than the no-key replacement we need.
- (c) Polygon free tier — rejected: 15 min delay + 5/min + token; no advantage over Yahoo here.
- (d) Stooq — rejected: returned an error page in probe; unreliable as a primary.

**Considered Options — replacement scope**
- (a) Realtime-only (keep crates for history) — **chosen**: smallest blast radius, backtest untouched, akshare/yfinance historical still functional.
- (b) Full hand-roll (drop both crates) — rejected this pass: removes `polars`/akshare weight but re-implements A-share + US historical K-lines and re-tests all backtests; revisit if akshare/yfinance history also breaks.
- (c) Dual-track transition — rejected: unnecessary complexity while the crates' history path is still healthy.

**Considered Options — A-share fallback**
- (a) East Money single source — simpler, but no resilience if EM is unreachable.
- (b) East Money primary + Tencent `gtimg.cn` / Sina `sinajs.cn` GBK fallback — **chosen**: both verified returning realtime data; the fallback needs only the numeric price/change fields (name falls back to the code), so **no GBK decoder crate is required**.

**Consequences**
- New `EmSource` takes over A-share `fetch_quotes` / `fetch_snapshot` (East Money `push2`/`push2his` `stock/get` for targeted quotes, the EM spot-board `clist` for the full board / snapshot / breadth; Tencent/Sina as fallback); `AkShareSource` retains only `fetch_klines` / `fetch_intraday`.
- New `YahooSource` takes over US `fetch_quotes` (Yahoo `v8` chart / `v7` quote); `YfSource` retains US K-line methods.
- `MarketSource` trait and `MarketRouter` wiring unchanged — this is an adapter-swap, not a reshaping.
- "Realtime" is now **asymmetric**: A-share is true sub-second realtime; US is ~15 min delayed (unauthenticated Yahoo). The dashboard footer must state this so users don't mistake US quotes for live ticks.
- No new heavy dependencies; `akshare-rs` / `yfinance-rs` stay (history). The GBK fallback avoids adding an encoding crate.
