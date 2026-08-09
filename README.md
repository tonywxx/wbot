# wbot

> [简体中文](README.zh-CN.md)

> A terminal (TUI) simulated-trading assistant for **A-shares, US stocks, and crypto (OKX)**, built on **real market data**, with built-in indicators, a signal engine, pattern recognition, simulated order execution, and **strategy backtest report generation**.

`wbot` is a command-line trading assistant written in Rust. It uses [`akshare-rs`](https://github.com/Cricle/akshare-rs) for A-share & index quotes, [`yfinance-rs`](https://github.com/gramistella/yfinance-rs) (Yahoo Finance) for US equities, and [`okx-rs`](https://github.com/roytang121/okx-rs) for OKX crypto spot — all unified behind a single `Candle` pipeline. From the keyboard you can browse the market, inspect technical indicators, track strategy signals, and place **risk-free simulated trades**. It can also batch-backtest every strategy defined in `strategy.toml` and automatically generate readable Markdown backtest reports.

> **Convention:** the terminal uses the Chinese color scheme — **red = up, green = down**.

---

## Table of Contents

- [Feature Overview](#feature-overview)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Installation & Build](#installation--build)
- [Usage](#usage)
  - [1. Launch the TUI](#1-launch-the-tui)
  - [2. TUI Views & Hotkeys](#2-tui-views--hotkeys)
  - [3. Generate Strategy Backtest Reports](#3-generate-strategy-backtest-reports)
  - [4. Common Operation Examples](#4-common-operation-examples)
- [Configuration](#configuration)
  - [Watchlists](#watchlists)
  - [Strategy File strategy.toml](#strategy-file-strategytoml)
  - [config.toml & Parameters](#configtoml--parameters)
- [Backtest Report Notes](#backtest-report-notes)
- [Simulated Trading Notes](#simulated-trading-notes)
- [US Stock Support](#us-stock-support)
- [Cryptocurrency (OKX) Support](#cryptocurrency-okx-support)
- [Internationalization (i18n)](#internationalization-i18n)
- [Risk Disclaimer](#risk-disclaimer)
- [Development Guide](#development-guide)

---

## Feature Overview

All engines (indicators, signals, backtest, simulated trading) operate on a **market-agnostic `Candle` stream**, so **every feature below works for A-shares, US equities, and OKX crypto** — only the data source differs. Symbol routing is automatic by shape:

- **6 ASCII digits** (`600519`) → A-share (akshare)
- **Contains a hyphen** (`BTC-USDT`) → crypto (OKX)
- **Anything else** (`AAPL`, `BRK-B`) → US (Yahoo Finance)

### 1. Real-time Market Terminal (`Market` view)

- Live **index strip** (e.g. 上证指数, 深证成指, 创业板指) with price and change %.
- **Market breadth panel**: count of advancing / declining / flat / limit-up / limit-down stocks across the whole A-share board, plus the total.
- **Watchlist table**: code, name, latest price, change % for every symbol you track (A-share, US, or crypto).
- **Gainers / Losers boards**: top-30 rankings, switchable with `Tab`.
- Snapshot refreshed every **5 seconds**; K-line increments pushed every ~60 s; intraday (minute) K-lines refreshed every `intraday_refresh` seconds (default 120 s).

### 2. Technical Indicators (`Indicators` view)

Computes and displays live indicator values for the selected symbol:

- **Moving averages**: MA5 / MA10 / MA20.
- **RSI(14)**.
- **MACD**: DIF, DEA, HIST (red/green colored).
- **BOLL**: mid / upper / lower bands.
- **KDJ**: K / D / J lines.
- A **bull/bear arrangement** hint (`MA5 > MA10` → short-term bullish).
- Use `↑` / `↓` to cycle through the watchlist and inspect each symbol.

### 3. Signal Engine (DSL)

- Evaluates `strategy.toml` DSL expressions per symbol.
- **Edge-triggering** (fires only on a false→true transition) to avoid double counting.
- Newly triggered signals raise a **desktop notification** (with per-(symbol, rule) cooldown).

### 4. Pattern Recognition

- Supports `double_golden` — a sequential **state machine** for the double-golden / double-dead-cross pullback pattern (needs consecutive-bar logic that point-in-time DSL cannot express), usable on 15-min, 60-min, etc.

### 5. Simulated Trading (`Account` view)

- **Virtual account** with an initial capital of **¥1,000,000** (A-share) / equivalent USD (US) / **100,000 USDT** (crypto).
- **Market orders**: buy/sell at the latest price. A-share/US orders round down to whole lots (`lot_size`, default 100); crypto buys size by a USDT budget (`crypto_lot_usdt`, default 1000) converted to base-coin quantity.
- **Cost model**: two-sided commission + sell-side stamp tax (A-share), or single-sided `crypto_fee_rate` (crypto); deducted live.
- **Account dashboard**: total assets, total P&L, unrealized P&L, realized P&L.
- **Positions table**: qty, avg cost, latest price, market value, P&L.
- **Trade blotter**: every fill with direction, price, qty, realized P&L.
- Persistence: fills appended to `trades.json`, account snapshot saved to `account.json`; both reload automatically on restart.

### 6. Strategy Management (`Strategies` view)

- Browse **all strategies** with the **live backtest win rate** for the currently selected symbol.
- Each strategy shows status (enabled/disabled), side (buy/sell), label, win rate, and trigger count.
- A detail panel shows the strategy note and full backtest stats (trades, win rate, avg win/loss, profit factor, max drawdown, cumulative return).
- Toggle enable/disable instantly with `Space`.

### 7. Backtest Engine & Reports

- Replays each strategy's signal on historical K-lines: for every trigger it takes a "buy/sell then **hold forward N bars**" approach (daily holds 10 bars, minute holds 5 bars) and aggregates:
  - **Win rate**, **average win / average loss**, **profit factor**, **total return**, **max drawdown**.
- Emits one independent `<id> 策略回测报告.md` per strategy: strategy metadata + cross-symbol summary table + per-symbol detail table + disclaimer.
- Runs for **A-share, US, and crypto** independently (see [backtest subcommands](#3-generate-strategy-backtest-reports)).

### 8. Multi-Timeframe

Daily DSL strategies and minute (T+0) strategies carrying `timeframe` are evaluated on the **correct period's** K-lines — never mixed up.

### 9. Notifications & Persistence

- Desktop notifications via `osascript` on macOS, falling back to stderr on other platforms.
- Safe persistence with missing/corrupt-file fallbacks — the program never panics on bad input.

### 10. Cryptocurrency (OKX) Support

- A **third market** behind the same `Candle` engine: OKX spot pairs (e.g. `BTC-USDT`, `ETH-USDT`).
- Public market data via OKX `/market/candles` (mapped to `Candle`); daily `1D` and intraday `1m`/`5m`/`15m`/`30m`/`1H` bars.
- **Simulated crypto account** (`CryptoLedger`): USDT cash + base-coin positions with average-cost tracking — works with no credentials.
- **Optional real trading**: set `live_trading = true` in `config.toml` and export `OKX_API_KEY` / `OKX_API_SECRET` / `OKX_PASSPHRASE`; then `Enter` in the TUI also sends a real OKX market order (failure only warns, the local ledger is still updated).
- The same 39 strategies backtest on crypto pairs via `backtest crypto` → `reports_crypto/`.

### 11. Internationalization (i18n)

- The entire UI and all backtest reports are localized. Default is **English**; set `language = "zh-CN"` (or `zh` / `chinese`) in `config.toml` to switch everything to **Simplified Chinese**.
- All strings go through a `tr()` lookup that falls back to English for unknown keys and never panics.

---

## Architecture

```
                ┌───────────────── data_loop (tokio async) ─────────────────┐
   akshare ─────►  fetch_market()  ──► Snapshot (indices + spots, every 5s)  │
   yfinance ───►  fetch_klines()  ──► daily K-lines (every ~60s)             │
   okx ────────►  fetch_candles()──► crypto K-lines (1D / intraday)         │
                 fetch_intraday()──► minute K-lines (every intraday_refresh) │
                └───────────────────────┬───────────────────────────────────┘
                                         │ Msg<Snapshot/Klines/Intraday>
                ┌────────────────────────▼──────────────────────────────────┐
   run_app() ──► apply_snapshot → eval_signals(engine) → notify + recompute │
                 handle_enter → account.place_order / crypto ledger → persist│
                └────────────────────────────────────────────────────────────┘

   Backtest path (separate CLI):  backtest_cli → fetch K-lines → backtest::
   write_strategy_reports → "<market>/<id> 策略回测报告.md"
```

The signal/indicator/backtest/sim engines are **market-agnostic**: they only ever consume `Candle` sequences. Market routing is handled by a `MarketRouter` over a `MarketSource` trait — A-share via `AkShareSource`, US via `YfSource`, crypto via `OkxSource` — dispatched by symbol shape (`market_of`). Because every engine sees only `Candle`s, the same strategies run on any market.

---

## Tech Stack

- **Language**: Rust (Edition 2024)
- **TUI**: [`ratatui`](https://ratatui.rs) 0.30 + `crossterm` 0.29 (cross-platform terminal control)
- **Async**: `tokio` 1.48 (multi-thread runtime)
- **Market Data**: `akshare-rs` (`equity` feature, A-shares & indices); `yfinance-rs` 0.9 (Yahoo Finance, US stocks); `okx-rs` (OKX V5, crypto) + `reqwest` 0.11 for public candles
- **Config / Serialization**: `serde` + `toml` + `serde_json`
- **Time**: `chrono`; **Errors**: `anyhow`; **Numeric**: `num-traits`

---

## Project Structure

```
wbot/
├── Cargo.toml                 # library + binary + examples definitions
├── config.toml                # optional global config (language, fees, crypto, …); read at startup
├── src/
│   ├── lib.rs                 # exposes all modules as `wbot::`, shared by binary & examples
│   ├── main.rs                # binary entry: TUI main loop + `backtest` subcommand (a-share/us/crypto/all)
│   ├── app.rs                 # app state (views, focus, account, crypto ledger, signal evaluation, backtests)
│   ├── market.rs              # MarketRouter/MarketSource trait; A-share (akshare) + US (yfinance) + OKX sources; breadth; watchlists
│   ├── crypto.rs              # OKX integration: public candles, real orders, CryptoLedger (sim account)
│   ├── i18n.rs                # internationalization: Lang + tr() lookup (en/zh), backtest report strings
│   ├── indicators.rs          # Candle, PriceSource, Indicator trait, IndicatorRegistry, build_indicator
│   ├── indicators/            # ma, macd, rsi, kdj, boll implementations
│   ├── signals.rs             # StrategyRule, RawRule, parse_strategy_file, Scope/Side enums
│   ├── signals/               # dsl (recursive parser), eval (signal engine), double_cross (pattern state machine)
│   ├── sim.rs                 # simulated-trading module root
│   ├── sim/                   # account.rs (Account/Position/Order), history.rs (Trade persistence)
│   ├── config.rs              # AppConfig defaults + load_config() (reads config.toml)
│   ├── persist.rs             # account.json / trades.json load & save
│   ├── notify.rs              # desktop notifier (cooldown + dedup)
│   ├── backtest.rs            # backtest engine + Markdown report rendering (i18n-aware)
│   ├── backtest_cli.rs        # async report generation (A-share / US / crypto), shared by binary and example
│   ├── ui.rs                  # ratatui render dispatch + tabs/header/indices/footer
│   ├── ui/                    # market_view, indicator_view, signal_view, account_view, strategy_view
│   └── tests.rs               # unit tests
├── examples/
│   └── backtest_all.rs        # backtest example reusing wbot::backtest_cli
├── strategy.toml              # strategy definitions (user-editable; 39 rules shipped)
├── watchlist.txt              # A-share watchlist
├── watchlist_us.txt           # US watchlist (optional; enables US tickers in the TUI)
├── watchlist_crypto.txt       # OKX crypto watchlist (optional; enables crypto in the TUI)
├── reports/                   # A-share backtest report output directory (auto-generated)
├── reports_us/                # US backtest report output directory (auto-generated)
└── reports_crypto/            # Crypto backtest report output directory (auto-generated)
```

---

## Installation & Build

### Requirements

- A working [Rust toolchain](https://rustup.rs/) (a recent version is recommended to support Edition 2024).
- Network access to the data sources you intend to use: `akshare` (A-share quotes), Yahoo Finance (US quotes), and/or OKX (crypto quotes). The TUI only contacts a source when its watchlist file is present, so you can run A-share-only without US/crypto connectivity.

### Build

```bash
# after cloning or entering the project directory
cargo build --release      # compile the release build (faster to run, slower to build)
cargo build                # compile the debug build
```

> The first build downloads and compiles `ratatui`, `tokio`, `akshare`/`reqwest`, `yfinance-rs`/`polars`, `okx-rs`, and other dependencies — this takes a while; please be patient.

---

## Usage

### 1. Launch the TUI

```bash
cargo run                 # launch in debug mode
# or
cargo run --release       # launch in release mode
```

On startup it will automatically:

1. Load `watchlist.txt` (and `watchlist_us.txt` / `watchlist_crypto.txt` if present) and `strategy.toml`;
2. Load `config.toml` if present (language, fees, crypto settings) — otherwise use defaults;
3. Fetch historical daily / minute K-lines to initialize;
4. Enter the full-screen TUI and start streaming quotes and evaluating signals.

Exit: press `q` or `Esc`.

### 2. TUI Views & Hotkeys

Switch views (number keys `1`–`5`):

| Key | View | Content |
| --- | --- | --- |
| `1` | Market | Index strip + market breadth + watchlist + gainers/losers boards (Tab toggles focus) |
| `2` | Indicators | MA5/MA10/MA20, RSI14, MACD, BOLL, KDJ for the selected symbol (`↑`/`↓` switch symbol) |
| `3` | Signals | Currently triggered buy/sell signals (`Enter` to trade on the highlighted one) |
| `4` | Account | Virtual funds, positions, and fills (`Enter` to buy the selected symbol at market) |
| `5` | Strategies | All strategies with their live backtest win rate (`Space` to enable / disable) |

Global hotkeys:

| Key | Action |
| --- | --- |
| `↑` / `k` | Move cursor up (or switch symbol in Indicators view) |
| `↓` / `j` | Move cursor down (or switch symbol in Indicators view) |
| `Tab` | In the Market view, toggle focus between "gainers / losers" boards |
| `Space` | In the Strategies view, enable / disable the current strategy |
| `r` | Force a refresh of real-time quotes |
| `Enter` | In Signals / Account views, place a market order at the latest price (A-share / US / crypto) |
| `q` / `Esc` | Quit the program |

> Triggered signals raise a system desktop notification (can be disabled in config). Orders are **simulated fills** written only to local `trades.json` / `account.json` — no real money is involved, unless `live_trading = true` is set for crypto (see [Crypto support](#cryptocurrency-okx-support)).

### 3. Generate Strategy Backtest Reports

Four equivalent ways to backtest **every** strategy in `strategy.toml` and emit an independent Markdown report each. The market is selected by the subcommand:

```bash
cargo run -- backtest reports            # A-shares       -> ./reports
cargo run -- backtest us reports_us      # US equities    -> ./reports_us
cargo run -- backtest crypto reports_crypto   # OKX crypto -> ./reports_crypto
cargo run -- backtest all                # all three markets (reports/ + reports_us/ + reports_crypto/)
```

The `examples` path mirrors the binary:

```bash
cargo run --example backtest_all -- reports   # A-share -> ./reports
cargo run --example backtest_all us           # US        -> ./reports_us
cargo run --example backtest_all crypto       # crypto    -> ./reports_crypto
cargo run --example backtest_all all          # all three markets
```

You'll see output like:

```
== A-share：已生成 39 份策略回测报告 ==
  - ma_golden : reports/ma_golden 策略回测报告.md
  - s01_ma_bull_arr : reports/s01_ma_bull_arr 策略回测报告.md
  ...
```

One `<id> 策略回测报告.md` is produced per strategy, covering daily DSL, classic, T+0 intraday, and pattern strategies — for whichever market you selected.

### 4. Common Operation Examples

**a) Add your first custom strategy.** Edit `strategy.toml`, append a rule, and reload the TUI — no code change needed:

```toml
[[rules]]
id = "my_ma20_cross"
label = "收盘价上穿 MA20"
side = "buy"
scope = "watchlist"
enabled = true
signal = "cross_above(PRICE(close), MA(close,20))"
note = "价格站上 20 日均线，趋势转强"
```

**b) Backtest a single market and inspect the report:**

```bash
cargo run -- backtest reports            # A-shares -> ./reports
cargo run -- backtest us reports_us      # US       -> ./reports_us
cargo run -- backtest crypto reports_crypto   # crypto -> ./reports_crypto
# then open any report, e.g.:
open "reports/ma_golden 策略回测报告.md"   # macOS
```

**c) Simulate a trade in the TUI:**

1. Press `1` → Market; pick a ticker.
2. Press `3` → Signals; if a buy signal is highlighted, press `Enter` to buy one lot at the latest price.
3. Press `4` → Account; review total assets, P&L, positions, and the trade blotter.

**d) Enable US stocks:** create `watchlist_us.txt` (one ticker per line); the TUI then merges both lists and evaluates US signals just like A-shares.

**e) Enable crypto:** create `watchlist_crypto.txt` (one `BASE-USDT` pair per line, e.g. `BTC-USDT`); the TUI merges it and you can simulate crypto trades in the Account view with `Enter`.

**f) Switch the UI to Chinese:** set `language = "zh-CN"` in `config.toml`, then restart — every panel and every backtest report renders in Simplified Chinese.

**g) Quick-check a strategy's live win rate:** press `5` → Strategies, move the cursor to any strategy, and read its win rate / trigger count for the currently selected symbol in the detail panel.

---

## Configuration

### Watchlists

Three optional watchlist files live in the working directory. Each is one symbol per line; `#` starts a comment and may carry a note. A file is only loaded when it exists, so the default TUI stays A-share-only until you opt in.

| File | Market | Format | Fallback |
| --- | --- | --- | --- |
| `watchlist.txt` | A-share | 6-digit code (e.g. `600519`) | built-in 10 liquid A-shares |
| `watchlist_us.txt` | US | ticker (e.g. `AAPL`, `BRK-B`) | built-in 26 liquid US names |
| `watchlist_crypto.txt` | OKX crypto | `BASE-USDT` pair (e.g. `BTC-USDT`) | built-in 10 liquid pairs |

Example `watchlist_crypto.txt`:

```
# 加密货币自选清单（OKX 现货，BASE-USDT 交易对）
BTC-USDT
ETH-USDT
SOL-USDT
```

- If a watchlist file is absent/empty, the corresponding built-in default list is used.
- Both backtesting and signal evaluation operate on the symbols in the loaded lists.

### Strategy File strategy.toml

Strategies come in two kinds: **DSL condition strategies** and **pattern strategies**.

#### DSL Condition Strategy

```toml
[[rules]]
id = "ma_golden"                       # unique id (used as the report filename)
label = "MA5 上穿 MA10 (金叉)"          # display name
side = "buy"                           # "buy" / "sell"
scope = "watchlist"                    # "watchlist" or comma-separated codes, e.g. "600519,000858"
enabled = true                         # enabled or not
signal = "cross_above(MA(close,5), MA(close,10))"   # condition expression
note = "短期均线金叉，经典趋势启动信号"  # note (shown in the strategy view)
```

The DSL supports (function names are case-insensitive):

- **Logic**: `and(...)` `or(...)` `not(...)`
- **Comparison**: `gt(a,b)` `lt(a,b)` `gte(a,b)` `lte(a,b)` `eq(a,b)`
- **Cross**: `cross_above(a,b)` (crosses above) `cross_below(a,b)` (crosses below)
- **Indicators** (source defaults to `close`): `MA(src,p)` `SMA(src,p)` `EMA(src,p)` `RSI(src,p)`
  - `MACD(src,fast,slow,sig).dif / .dea / .hist`
  - `KDJ(n,k,d).k / .d / .j`
  - `BOLL(p,k).mid / .upper / .lower`
- **Price**: `PRICE(close)` (also `open` / `high` / `low` / `volume`)

#### Minute (T+0) DSL Strategy

A DSL rule carrying `timeframe` and `bars` is evaluated on the corresponding minute K-lines (not daily):

```toml
[[rules]]
id = "t0_buy_rsi"
label = "T+0 日内超卖低吸"
side = "buy"
scope = "watchlist"
enabled = true
timeframe = "5"        # minute period: 1/5/15/30/60 (crypto uses 1m/5m/15m/30m/1H)
bars = 60              # lookback bars
signal = "lt(RSI(close,14), 25)"
note = "5 分钟 RSI<25 日内超卖，适合低吸做 T 的买点"
```

#### Pattern Strategy

Pattern rules use a **sequential state machine** (not point-in-time DSL), suited to patterns like the double-golden-cross pullback that need consecutive判断:

```toml
[[rules]]
id = "dg_buy_15"
label = "15m 双金叉回踩买入"
side = "buy"
scope = "watchlist"
enabled = true
kind = "pattern"               # declares a pattern rule
pattern = "double_golden"      # pattern name
timeframe = "15"               # minute period
fast = 5                       # EMA fast period
slow = 10                      # EMA slow period
bars = 100                     # lookback bars (Sina 1H ~36 bars, auto-truncated)
higher_low = true              # pullback doesn't break prior low / rebound doesn't exceed prior high
note = "15 分钟双金叉回踩不破前低，末根成本高于慢线，多头成本基础确立后买入"
```

### config.toml & Parameters

`wbot` reads an optional `config.toml` from the working directory at startup via `load_config()`. Every field is optional; missing/unknown fields and a missing/corrupt file all fall back to `AppConfig::default()` — the program never panics. A ready-to-edit `config.toml` ships with the repo.

| Parameter | Default | Description |
| --- | --- | --- |
| `language` | `"en"` | UI & report language: `en` (default) or `zh` / `zh-CN` / `chinese` (Simplified Chinese) |
| `commission` | `0.0003` | Two-sided commission rate (0.03%, A-share/US) |
| `stamp_tax` | `0.0005` | Stamp tax (sell side only, 0.05%, A-share concept) |
| `lot_size` | `100` | Shares per lot (A-share/US) |
| `auto_trade` | `false` | Auto-order switch (manual only by default) |
| `kline_adjust` | `"qfq"` | K-line adjustment (forward-adjusted; ignored for crypto spot) |
| `kline_count` | `250` | Daily K-line bars retained |
| `intraday_refresh` | `120` | Minute K-line refresh interval (seconds) |
| `notify_enabled` | `true` | Whether to send desktop notifications |
| `notify_cooldown` | `300` | Notification cooldown per (symbol, rule) (seconds) |
| `crypto_enabled` | `true` | Whether to enable the OKX crypto data source & trading support |
| `live_trading` | `false` | Send real OKX market orders (requires `OKX_API_KEY`/`OKX_API_SECRET`/`OKX_PASSPHRASE` env vars) |
| `crypto_lot_usdt` | `1000.0` | Per-buy budget in USDT (converted to base-coin quantity at fill price) |
| `crypto_fee_rate` | `0.001` | One-sided crypto fee rate (0.1%) |
| `default_scope` | `"watchlist"` | Default strategy scope |

> The simulated A-share/US account starts with **¥1,000,000** initial capital; the simulated crypto account starts with **100,000 USDT**.

---

## Backtest Report Notes

The backtest engine replays a strategy's signals on historical K-lines. For **each signal trigger** it takes a "buy/sell then hold forward N bars" approach to compute returns (daily holds 10 bars, minute holds 5 bars), and aggregates:

- **Win rate**: winning trades / total triggers
- **Average win / average loss**
- **Profit factor**
- **Total return**
- **Max drawdown**

Each `<id> 策略回测报告.md` contains:

1. **Strategy info**: id, name, side, scope, signal expression / pattern params, data range, number of covered symbols, hold period;
2. **Cross-symbol summary table**: horizontal comparison of win rate, trade count, cumulative return, etc.;
3. **Per-symbol detail table**: backtest statistics for each symbol;
4. **Disclaimer**: backtests are historical simulations and do not represent future returns.

> The "win rate" in reports is measured by the backtest engine on the selected symbols — not a subjective claim; the TUI Strategies view also shows the live backtest win rate for the current symbol. Report text follows the `language` setting in `config.toml`.

---

## Simulated Trading Notes

- In the Account view, press `Enter` to buy the currently selected symbol at the **latest price**; in the Signals view, highlight a signal and press `Enter` to order in its direction (buy / sell). This works for A-share, US, and crypto symbols.
- **A-share / US**: each order fills in whole lots of `lot_size` (default 100 shares); commission and stamp tax are deducted live per `config`. Buys reject if cash is insufficient; sells reject if the position is too small.
- **Crypto**: a buy sizes `crypto_lot_usdt` (default 1000 USDT) of base coin at the latest price; a sell liquidates the whole position. Fee is `crypto_fee_rate` (default 0.1%) per side; rejects if USDT/position is insufficient. By default this updates only the local `CryptoLedger` (no real funds move).
- Fills are appended to `trades.json` and the account snapshot is written to `account.json`; both reload automatically on restart, so you can track simulated positions continuously.

---

## US Stock Support

`wbot` is not A-share-only: every engine (indicators, signal evaluation, backtest, simulated trading) operates on a market-agnostic `Candle` stream, so the **same** strategies and reports work for US equities. The only difference is the data source, selected automatically by symbol shape:

- **A-share code** — 6 ASCII digits (e.g. `600519`) → fetched via `akshare`.
- **US ticker** — anything else (e.g. `AAPL`, `BRK-B`) → fetched via `yfinance-rs` (Yahoo Finance). Note: Berkshire Hathaway is `BRK-B` (hyphen), not `BRK.B`.

### US watchlist

Create a `watchlist_us.txt` next to `watchlist.txt` (one ticker per line, `#` comments allowed, same format as the A-share list). If the file is absent, a built-in default list of liquid US names is used. The TUI merges both lists only when `watchlist_us.txt` exists, so the default TUI stays A-share-only until you opt in.

### Backtest US stocks

```bash
cargo run -- backtest us                # US backtest, writes to ./reports_us
cargo run -- backtest us my_us_reports  # custom output directory
cargo run --example backtest_all us     # writes to ./reports_us
```

Reports land in `reports_us/` (or your chosen dir), each named `<id> 策略回测报告.md`, with the same structure as the A-share reports (strategy info + cross-symbol summary + per-symbol detail) and a `市场：美股` row.

### Practical notes

- **Currency**: US quotes are in **USD**. Position values, P&L and returns shown in the TUI and reports are denominated in USD, not CNY.
- **Fees**: the backtest / simulated-trading engine reads fees from `AppConfig` (`src/config.rs`) — a two-sided `commission` plus a sell-side `stamp_tax`. The `stamp_tax` is an A-share concept that does **not** apply to US trades; for a US-only run you may set `stamp_tax = 0` to avoid overstating sell-side costs. The `lot_size` (round-lot) default of 100 is also reasonable for US equities.
- **Timeframes & history**: US daily bars use Yahoo `range=1y` (~252 bars); intraday uses `range=1mo` at `1` / `5` / `15` / `30` / `60` minutes. As with A-shares, a strategy's `bars` lookback truncates the series to the most recent N bars, so a 5-minute strategy with `bars=60` only spans ~1 trading session.
- **TUI**: with `watchlist_us.txt` present, the TUI merges both lists and evaluates signals / runs simulated trades for US tickers exactly like A-shares; US prices seed the board in USD.
- **Symbol format**: pass tickers exactly as Yahoo uses them — `BRK-B` (hyphen) for Berkshire Hathaway, `BF-B` for Brown-Forman, etc. Share-class suffixes use a hyphen, never a dot.

> **Network note:** US data comes from Yahoo Finance. In some sandboxed / datacenter networks Yahoo returns HTTP `429` (rate-limited / bot-blocked), in which case no US bars are fetched and the reports show `N/A` for every symbol. Run the command on a machine with normal Yahoo access to obtain real backtest results — the code path is identical.

---

## Cryptocurrency (OKX) Support

`wbot` treats crypto as a **first-class third market**: the same `Candle` engine, indicator math, signal DSL, and backtest all apply to OKX spot pairs. Symbol routing sends any symbol containing a hyphen (e.g. `BTC-USDT`, `ETH-USDT`) to the OKX source.

### Data source

- Public market data uses OKX `/market/candles` (via `reqwest`, kept `Send + Sync`) mapped to `Candle`: daily `1D` and intraday `1m`/`5m`/`15m`/`30m`/`1H`.
- There is **no** A-share-style full-board snapshot for crypto, so the Market view's breadth/indices panels stay A-share-only; crypto symbols appear in the watchlist and gainers/losers boards using their latest price.

### Crypto watchlist

Create `watchlist_crypto.txt` (one `BASE-USDT` pair per line). If absent, a built-in default of 10 liquid pairs (`BTC-USDT`, `ETH-USDT`, `SOL-USDT`, …) is used. The TUI merges it only when the file exists.

### Simulated crypto trading

- Enter on a crypto symbol in the Account/Signals view updates the local `CryptoLedger` — a simulated account with **100,000 USDT** cash plus base-coin positions and average-cost tracking. No credentials required.
- A buy converts `crypto_lot_usdt` (default 1000 USDT) into base-coin quantity at the latest price; a sell liquidates the whole position. Fee is `crypto_fee_rate` (default 0.1%) per side.
- Rejects if USDT or the position is insufficient.

### Optional real trading

To actually send orders to OKX:

1. Set `live_trading = true` in `config.toml`;
2. Export credentials in the environment:

   ```bash
   export OKX_API_KEY=...
   export OKX_API_SECRET=...
   export OKX_PASSPHRASE=...
   ```

Then `Enter` in the TUI also places a real OKX **market** order (spot, `cash` mode). If the live order fails, only a warning is printed — the local simulated ledger is still updated. With `live_trading = false` (default) no network order is ever sent.

### Backtest crypto

```bash
cargo run -- backtest crypto                # crypto backtest, writes to ./reports_crypto
cargo run -- backtest crypto my_crypto_reports   # custom output directory
cargo run --example backtest_all crypto     # writes to ./reports_crypto
```

Reports land in `reports_crypto/`, each named `<id> 策略回测报告.md`, with the same structure as the other markets and a `市场：加密货币` row. OKX's public candle API is generally reachable, so crypto backtests produce real results even in sandboxed environments.

### Practical notes

- **Currency**: crypto quotes are in **USDT**; P&L and returns are denominated in USDT.
- **Fees**: use `crypto_fee_rate` (default 0.1%), a one-sided rate — there is no A-share-style stamp tax.
- **Symbol format**: use OKX spot instruments exactly — `BTC-USDT`, `ETH-USDT` (hyphen). The hyphen is what triggers crypto routing, so do **not** write `BTC.USDT`.

> **Risk note:** `live_trading = true` places **real orders with real funds** on your OKX account. The simulated ledger and real account are separate; only the simulated ledger is persisted locally. Double-check `crypto_lot_usdt` and your environment variables before enabling.

---

## Internationalization (i18n)

The interface and all backtest reports are fully localized.

- **Default**: English.
- **Switch to Simplified Chinese**: set `language = "zh-CN"` in `config.toml` (also accepted: `zh`, `zh_cn`, `chinese`, `中文`, `中文版`), then restart.

Every UI string and report label is resolved through `tr(key, lang)`; unknown keys fall back to English and the lookup never panics. The `config.toml` that ships with the repo documents the accepted values.

---

## Risk Disclaimer

`wbot` is for **learning and technical research only**. All trades are **simulated** and involve no real funds or broker interfaces — except when crypto `live_trading = true` is explicitly enabled (see [Crypto support](#cryptocurrency-okx-support)), which connects to your own OKX account at your own risk. Strategy signals and backtest results are based on historical data and are subject to model bias, overfitting, and future uncertainty — they **do not constitute any investment advice**. Users assume all risks of any action taken accordingly.

---

## Development Guide

- **Shared code**: `src/lib.rs` exposes all modules as `wbot::`, so both the binary (`main.rs`) and `examples/` can reuse them — no duplicate backtest / market logic.
- **Add a strategy**: just edit `strategy.toml`; no code change needed. The DSL evaluator and pattern state machine already support common indicators and the double-golden-cross pattern.
- **Extend backtesting**: core logic lives in `src/backtest.rs` (engine + Markdown rendering, i18n-aware) and `src/backtest_cli.rs` (async data fetching & orchestration for A-share / US / crypto), both shared by the binary subcommand and `examples/backtest_all.rs`.
- **Add a market**: implement the `MarketSource` trait (`src/market.rs`) and register it in `MarketRouter::new()`; the rest of the engine consumes only `Candle`s.
- **Extend indicators**: implement the `Indicator` trait in `src/indicators/` and register a new `kind` in `build_indicator` (`src/indicators.rs`); it then becomes usable from any DSL expression.
- **Run tests**:

  ```bash
  cargo test
  ```

- **Build the example**:

  ```bash
  cargo build --example backtest_all
  ```

---

> Language: English ｜ Data source: `akshare` (A-share quotes) · Yahoo Finance (US quotes) · OKX (crypto spot) ｜ See `LICENSE` for the license.
