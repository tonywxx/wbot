# wbot

> Other languages: [简体中文](README.zh-CN.md)

> A terminal (TUI) simulated-trading assistant for A-shares built on real market data, with built-in indicators, a signal engine, pattern recognition, simulated order execution, and **strategy backtest report generation**.

`wbot` is a command-line trading assistant written in Rust. It uses [`akshare`](https://github.com/Cricle/akshare-rs) to fetch real-time and historical A-share and index quotes, and lets you browse the market, inspect technical indicators, track strategy signals, and place **risk-free simulated trades** entirely from the keyboard. It can also batch-backtest every strategy defined in `strategy.toml` and automatically generate readable Markdown backtest reports.

---

## Table of Contents

- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Installation & Build](#installation--build)
- [Usage](#usage)
  - [1. Launch the TUI](#1-launch-the-tui)
  - [2. TUI Views & Hotkeys](#2-tui-views--hotkeys)
  - [3. Generate Strategy Backtest Reports](#3-generate-strategy-backtest-reports)
- [Configuration](#configuration)
  - [Watchlist watchlist.txt](#watchlist-watchlisttxt)
  - [Strategy File strategy.toml](#strategy-file-strategytoml)
  - [Fees & Parameters](#fees--parameters)
- [Backtest Report Notes](#backtest-report-notes)
- [Simulated Trading Notes](#simulated-trading-notes)
- [Risk Disclaimer](#risk-disclaimer)
- [Development Guide](#development-guide)

---

## Features

| Module | Description |
| --- | --- |
| **Market** | Real-time index and watchlist gainers/losers boards (toggleable), refreshed every 5 seconds. |
| **Indicators** | Computes MA / SMA / EMA / RSI / MACD / KDJ / BOLL for the selected stock, refreshed live with K-lines. |
| **Signals** | Evaluates `strategy.toml` DSL expressions per symbol with **edge-triggering** (fires only when a condition turns false→true) to avoid double counting; new signals raise a desktop notification. |
| **Pattern Recognition** | Supports `double_golden` (double golden/dead cross pullback) minute-level state machine for 15-minute, 60-minute, etc. periods. |
| **Simulated Trading** | Virtual account, market orders, commission & stamp-tax calculation; fills persisted to `trades.json`, account state saved to `account.json`. |
| **Strategy Management** | Browse all strategies with live backtest win rate; toggle enable/disable with Space. |
| **Backtest Engine** | Replays each strategy's signal on historical K-lines, holding forward N bars to compute win rate, average win/loss, profit factor, total return, and max drawdown. |
| **Report Generation** | Emits one independent `<id> 策略回测报告.md` per strategy: strategy metadata + cross-symbol summary table + per-symbol detail table. |
| **Multi-Timeframe** | Daily DSL strategies and minute (T+0) strategies carrying `timeframe` are evaluated on the correct period's K-lines — never mixed up. |

---

## Tech Stack

- **Language**: Rust (Edition 2024)
- **TUI**: [`ratatui`](https://ratatui.rs) + `crossterm` (cross-platform terminal control)
- **Async**: `tokio` (multi-thread runtime)
- **Market Data**: `akshare` (`equity` feature, covering A-shares and indices)
- **Config / Serialization**: `serde` + `toml` + `serde_json`
- **Time**: `chrono`

---

## Project Structure

```
wbot/
├── Cargo.toml            # library + binary + examples definitions
├── src/
│   ├── lib.rs            # exposes all modules, shared by the binary and examples
│   ├── main.rs           # binary entry: TUI main loop + `backtest` subcommand
│   ├── app.rs            # application state machine (views, focus, account, signal evaluation)
│   ├── market.rs         # quote fetching (indices / stocks / daily / minute K-lines)
│   ├── ui.rs             # ratatui rendering
│   ├── indicators.rs     # indicator computation and registry
│   ├── signals.rs        # DSL parsing & evaluation engine, StrategyRule
│   ├── sim/              # simulated trading (account.rs / history.rs)
│   ├── config.rs         # fees, lot size, K-line parameters
│   ├── persist.rs        # account / trade persistence
│   ├── notify.rs         # desktop notifications
│   ├── backtest.rs       # backtest engine + Markdown report rendering
│   ├── backtest_cli.rs   # async report-generation entry (shared by binary and example)
│   └── tests.rs          # unit tests
├── examples/
│   └── backtest_all.rs   # backtest example reusing wbot::backtest_cli
├── strategy.toml         # strategy definitions (user-editable)
├── watchlist.txt         # watchlist
└── reports/              # backtest report output directory (auto-generated)
```

---

## Installation & Build

### Requirements

- A working [Rust toolchain](https://rustup.rs/) (a recent version is recommended to support Edition 2024).
- Network access to the `akshare` data source (to fetch A-share quotes).

### Build

```bash
# after cloning or entering the project directory
cargo build --release      # compile the release build (faster to run, slower to build)
cargo build                # compile the debug build
```

> The first build downloads and compiles `ratatui`, `tokio`, `akshare`/`reqwest`, and other dependencies — this takes a while; please be patient.

---

## Usage

### 1. Launch the TUI

```bash
cargo run                 # launch in debug mode
# or
cargo run --release       # launch in release mode
```

On startup it will automatically:

1. Load `watchlist.txt` and `strategy.toml`;
2. Fetch historical daily / minute K-lines to initialize;
3. Enter the full-screen TUI and start streaming quotes and evaluating signals.

Exit: press `q` or `Esc`.

### 2. TUI Views & Hotkeys

Switch views (number keys `1`–`5`):

| Key | View | Content |
| --- | --- | --- |
| `1` | Market | Index + watchlist gainers/losers (Tab toggles gainers / losers) |
| `2` | Indicators | Technical indicator values for the selected stock |
| `3` | Signals | List of currently triggered buy/sell signals (Enter to trade on the highlighted one) |
| `4` | Account | Virtual account funds, positions, and fills (Enter to buy the selected symbol at market) |
| `5` | Strategies | All strategies with their live backtest win rate (Space to enable / disable) |

Global hotkeys:

| Key | Action |
| --- | --- |
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |
| `Tab` | In the Market view, toggle "gainers / losers" |
| `Space` | In the Strategies view, enable / disable the current strategy |
| `r` | Force a refresh of real-time quotes |
| `Enter` | In Signals / Account views, place a market order at the latest price |
| `q` / `Esc` | Quit the program |

> Triggered signals raise a system desktop notification (can be disabled in config). Orders are **simulated fills** written only to local `trades.json` / `account.json` — no real money is involved.

### 3. Generate Strategy Backtest Reports

Two equivalent ways to backtest **every** strategy in `strategy.toml` and emit an independent Markdown report each:

**Option A — binary subcommand** (recommended):

```bash
cargo run -- backtest reports
# equivalent to: wbot backtest <output-dir>, default output dir is "reports"
```

**Option B — examples**:

```bash
cargo run --example backtest_all -- reports
# the directory argument is optional; default is "reports"
```

You'll see output like:

```
已生成 39 份策略回测报告 -> reports
  - ma_golden : reports/ma_golden 策略回测报告.md
  - s01_ma_bull_arr : reports/s01_ma_bull_arr 策略回测报告.md
  ...
```

One `<id> 策略回测报告.md` is produced per strategy, covering daily DSL, classic, T+0 intraday, and pattern strategies.

---

## Configuration

### Watchlist watchlist.txt

One 6-digit A-share code per line; `#` starts a comment and may carry a Chinese note:

```
# A股自选股列表 (watchlist)
600519   # 贵州茅台
601318   # 中国平安
600036   # 招商银行
...
```

- Leaving this file empty or deleting it falls back to a built-in default list.
- Both backtesting and signal evaluation operate on the symbols in this list.

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

The DSL supports:

- **Logic**: `and(...)` `or(...)` `not(...)`
- **Comparison**: `gt(a,b)` `lt(a,b)` `gte(a,b)` `lte(a,b)` `eq(a,b)`
- **Cross**: `cross_above(a,b)` (crosses above) `cross_below(a,b)` (crosses below)
- **Indicators**: `MA(src,p)` `SMA(src,p)` `EMA(src,p)` `RSI(src,p)`
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
timeframe = "5"        # minute period: 1/5/15/30/60
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

### Fees & Parameters

Fees and K-line parameters are provided by `AppConfig::default()` in `src/config.rs` (can later be switched to reading `config.toml`):

| Parameter | Default | Description |
| --- | --- | --- |
| `commission` | `0.0003` | Two-sided commission rate (0.03%) |
| `stamp_tax` | `0.0005` | Stamp tax (sell side only, 0.05%) |
| `lot_size` | `100` | Shares per lot |
| `auto_trade` | `false` | Auto-order switch (manual only by default) |
| `kline_adjust` | `"qfq"` | K-line adjustment (forward-adjusted) |
| `kline_count` | `250` | Daily K-line bars retained |
| `intraday_refresh` | `120` | Minute K-line refresh interval (seconds) |
| `notify_enabled` | `true` | Whether to send desktop notifications |
| `notify_cooldown` | `300` | Notification cooldown per (symbol, rule) (seconds) |

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

> The "win rate" in reports is measured by the backtest engine on the selected symbols — not a subjective claim; the TUI Strategies view also shows the live backtest win rate for the current symbol.

---

## Simulated Trading Notes

- In the Account view, press `Enter` to buy the currently selected symbol at the **latest price**; in the Signals view, highlight a signal and press `Enter` to order in its direction (buy / sell).
- Each order fills in whole lots of `lot_size` (default 100 shares); commission and stamp tax are deducted live per `config`.
- Fills are appended to `trades.json` and the account snapshot is written to `account.json`; both reload automatically on restart, so you can track simulated positions continuously.

---

## Risk Disclaimer

`wbot` is for **learning and technical research only**. All trades are **simulated** and involve no real funds or broker interfaces. Strategy signals and backtest results are based on historical data and are subject to model bias, overfitting, and future uncertainty — they **do not constitute any investment advice**. Users assume all risks of any action taken accordingly.

---

## Development Guide

- **Shared code**: `src/lib.rs` exposes all modules as `wbot::`, so both the binary (`main.rs`) and `examples/` can reuse them — no duplicate backtest / market logic.
- **Add a strategy**: just edit `strategy.toml`; no code change needed. The DSL evaluator and pattern state machine already support common indicators and the double-golden-cross pattern.
- **Extend backtesting**: core logic lives in `src/backtest.rs` (engine + Markdown rendering) and `src/backtest_cli.rs` (async data fetching & orchestration), both shared by the binary subcommand and `examples/backtest_all.rs`.
- **Run tests**:

  ```bash
  cargo test
  ```

- **Build the example**:

  ```bash
  cargo build --example backtest_all
  ```

---

> Language: English ｜ Data source: `akshare` (A-share quotes) ｜ See `LICENSE` for the license.
