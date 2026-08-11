# Graph Report - .  (2026-08-10)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 528 nodes · 1323 edges · 23 communities (18 shown, 5 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 68 edges (avg confidence: 0.8)
- Token cost: 845 input · 49 output

## Graph Freshness
- Built from commit: `2dabd498`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Market Data Integration
- Technical Indicators
- App Main Loop
- Backtest Test Suite
- UI Application Logic
- ta.rs
- Candle
- signals.rs
- Lang
- OkxClient
- generate_reports_for
- ta_dispatch.rs
- Notifier
- gen_ta_dispatch.py
- adaq-talib
- akshare-rs
- Candle Concept
- okx-rs

## God Nodes (most connected - your core abstractions)
1. `App` - 58 edges
2. `Candle` - 44 edges
3. `Lang` - 31 edges
4. `tr()` - 22 edges
5. `IndicatorId` - 22 edges
6. `series()` - 20 edges
7. `StrategyRule` - 19 edges
8. `run_app()` - 16 edges
9. `PriceSource` - 14 edges
10. `MarketData` - 14 edges

## Surprising Connections (you probably didn't know these)
- `dsl_parse_ok()` --calls--> `parse_signal()`  [INFERRED]
  src/tests.rs → src/signals/dsl.rs
- `ta_dsl_parse_ta_indicator()` --calls--> `parse_signal()`  [INFERRED]
  src/tests.rs → src/signals/dsl.rs
- `signal_on_prefix()` --calls--> `detect_double_golden_cross()`  [INFERRED]
  src/backtest.rs → src/signals/double_cross.rs
- `side_text()` --calls--> `tr()`  [INFERRED]
  src/backtest.rs → src/i18n.rs
- `period_text()` --calls--> `period_min()`  [INFERRED]
  src/backtest.rs → src/i18n.rs

## Import Cycles
- 2-file cycle: `src/crypto.rs -> src/main.rs -> src/crypto.rs`
- 3-file cycle: `src/app.rs -> src/crypto.rs -> src/main.rs -> src/app.rs`
- 4-file cycle: `src/app.rs -> src/crypto.rs -> src/main.rs -> src/ui.rs -> src/app.rs`

## Communities (23 total, 5 thin omitted)

### Community 0 - "Market Data Integration"
Cohesion: 0.08
Nodes (46): AkShareClient, Decimal, IndexSpotEm, Interval, Range, SpotQuote, AkShareSource, amount_to_f64() (+38 more)

### Community 1 - "Technical Indicators"
Cohesion: 0.06
Nodes (36): Boll, Option, Self, String, Vec, build_indicator(), ema(), Indicator (+28 more)

### Community 2 - "App Main Loop"
Cohesion: 0.08
Nodes (47): CrosstermBackend, Receiver, Sender, side_text(), apply_snapshot(), data_loop(), eval_signals(), handle_enter() (+39 more)

### Community 3 - "Backtest Test Suite"
Cohesion: 0.07
Nodes (40): parse_scope(), backtest_double_golden_wins_on_synthetic(), backtest_never_fires_has_no_trades(), boll_width_positive(), candle(), double_dead_fires_on_lower_high(), double_golden_fires_on_higher_low(), double_golden_rejects_broken_base() (+32 more)

### Community 4 - "UI Application Logic"
Cohesion: 0.10
Nodes (39): Color, App, Focus, HashMap, Instant, Option, String, Vec (+31 more)

### Community 5 - "ta.rs"
Cohesion: 0.08
Nodes (31): bilingual_group(), desc(), fmt_num(), group_fallback(), main(), String, Self, Option (+23 more)

### Community 6 - "Candle"
Cohesion: 0.12
Nodes (36): backtest_rule(), backtest_strategy(), BacktestResult, CodeResult, fmt_date(), pct(), period_text(), render_strategy_report_md() (+28 more)

### Community 7 - "signals.rs"
Cohesion: 0.16
Nodes (22): CmpOp, CrossDir, Arg, parse_signal(), Parser, Result, String, Vec (+14 more)

### Community 8 - "Lang"
Cohesion: 0.16
Nodes (26): backtest_line(), cash(), crypto_usdt(), help_items(), hold_n(), initial(), Lang, last_close() (+18 more)

### Community 9 - "OkxClient"
Cohesion: 0.14
Nodes (14): Client, Rest, CryptoFill, CryptoLedger, OkxClient, OrderData, PlaceOrderReq, Default (+6 more)

### Community 10 - "generate_reports_for"
Cohesion: 0.21
Nodes (16): collect_tf_bars(), generate_reports(), generate_reports_crypto(), generate_reports_for(), generate_reports_us(), PathBuf, Result, String (+8 more)

### Community 11 - "ta_dispatch.rs"
Cohesion: 0.24
Nodes (16): MaType, call_adaq(), _known(), list_all_functions(), ma_type_from(), mat(), pr(), pu() (+8 more)

### Community 12 - "Notifier"
Cohesion: 0.25
Nodes (8): Duration, esc(), Notifier, HashMap, Instant, Self, String, send_notification()

## Knowledge Gaps
- **5 isolated node(s):** `Candle Concept`, `adaq-talib`, `akshare-rs`, `yfinance-rs`, `okx-rs`
  These have ≤1 connection - possible missing edges or undocumented components.
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Candle` connect `Candle` to `Market Data Integration`, `Technical Indicators`, `App Main Loop`, `UI Application Logic`, `ta.rs`, `OkxClient`, `ta_dispatch.rs`?**
  _High betweenness centrality (0.270) - this node is a cross-community bridge._
- **Why does `App` connect `UI Application Logic` to `Market Data Integration`, `App Main Loop`, `Candle`, `Lang`, `OkxClient`, `generate_reports_for`, `Notifier`?**
  _High betweenness centrality (0.217) - this node is a cross-community bridge._
- **Why does `Market` connect `Market Data Integration` to `Backtest Test Suite`, `generate_reports_for`, `App Main Loop`, `UI Application Logic`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Are the 20 inferred relationships involving `tr()` (e.g. with `period_text()` and `render_strategy_report_md()`) actually correct?**
  _`tr()` has 20 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Candle Concept`, `adaq-talib`, `akshare-rs` to the rest of the system?**
  _5 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Market Data Integration` be split into smaller, more focused modules?**
  _Cohesion score 0.0806697108066971 - nodes in this community are weakly interconnected._
- **Should `Technical Indicators` be split into smaller, more focused modules?**
  _Cohesion score 0.05868118572292801 - nodes in this community are weakly interconnected._