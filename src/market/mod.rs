//! Market data fetching & derived metrics.
//!
//! **Realtime quotes** (the watchlist / board snapshot path) are hand-rolled
//! `reqwest` clients — A-shares via East Money (`push2`/`push2his` + Tencent/Sina
//! GBK fallback), US via Yahoo Finance `v8` chart — mirroring the `OkxSource`
//! pattern. **Historical K-lines / backtest** still go through `akshare`
//! (`AkShareClient`) and `yfinance-rs` (`YfClient`), which remain healthy.
//!
//! Symbol routing is by shape: a 6-digit numeric code is treated as an A-share;
//! anything else (e.g. `AAPL`, `BRK.B`) is treated as a US ticker. The indicator,
//! signal, backtest and simulated-trading engines are all market-agnostic — they
//! only ever see `Candle` sequences — so "US support" is purely a data-source switch.

pub mod types;
pub mod source;
pub mod router;
pub mod realtime;

pub use types::{Market, market_of, Quote, SourceError, Spot, IndexSpot, MarketData, Breadth};
pub use source::{MarketSource, AkShareSource, YfSource};
pub use router::{MarketRouter, load_watchlist, load_watchlist_us, load_watchlist_crypto, load_watchlist_combined};
pub use crate::crypto::AdaqCryptoSource;
