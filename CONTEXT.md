# wbot

A terminal (TUI) stock-trading assistant: it streams market data, evaluates user-defined buy/sell signals, simulates trading, and backtests strategies against historical candles.

## Language

**Candle**: A single OHLCV bar (open/high/low/close/volume + timestamp). The universal, market-agnostic data unit — every engine (indicators, signals, backtest, simulated trading) consumes only Candle sequences and never sees a provider's native types.
_Avoid_: bar, kline (acceptable as synonyms in comments only)

**Market**: The classification of a tradable as an A-share or a US stock, decided by symbol shape (a 6-digit code is A-share; anything else is US).
_Avoid_: exchange, venue

**MarketSource**: The uniform async data-provider abstraction behind which a concrete provider (A-share or US) yields Candle sequences and an optional board snapshot. The seam that keeps the engine layer provider-free.
_Avoid_: client, provider, data feed

**MarketRouter**: The object that owns the A-share and US MarketSources and dispatches each request to the right one by symbol shape.
_Avoid_: dispatcher, loader

**Watchlist**: The set of symbols the user tracks in the dashboard, loaded from `watchlist.txt` / `watchlist_us.txt` (or built-in defaults).
_Avoid_: portfolio, symbols

**Indicator**: A derived price series (e.g. MA, RSI, MACD, KDJ, BOLL) computed over a Candle sequence by the indicator engine.
_Avoid_: metric

**StrategyRule**: A compiled buy/sell rule from `strategy.toml` — either a recursive DSL expression or a state-machine pattern (e.g. double-golden-cross) — carrying a scope (watchlist or explicit codes) and a side.
_Avoid_: strategy, signal rule

**Signal / SignalEvent**: A triggered buy/sell event produced by evaluating StrategyRules against live or historical candles, edge-triggered (only on false→true).
_Avoid_: alert, trigger

**Backtest**: Replaying a StrategyRule over historical candles, holding for a fixed number of bars after each trigger, to measure win rate, return, and drawdown.
_Avoid_: simulation, test

**Account / Trade**: The simulated trading position (cash, holdings, lot size) and a single executed buy/sell recorded against it.
_Avoid_: portfolio (for Account), order (for Trade)
