# 交付总结 · A 股模拟交易系统（wbot）

> 负责人：主理人（WorkBuddy）｜日期：2026-08-09
> 状态：**已完成并通过编译 + 16 项单元测试**（IS_PASS: YES）

---

## TL;DR

把原有的 Rust TUI 行情看板扩展为一个**基于真实行情的模拟交易系统**。新增自研、可扩展的指标引擎（MA/EMA/RSI/MACD/KDJ/BOLL）、递归策略 DSL（`strategy.toml` 驱动）、信号引擎（沿触发去抖）、模拟账户（市价单 / 盈亏 / 持仓 / 成交记录），以及 4 视图 TUI（行情 / 指标 / 信号 / 账户，按 `1/2/3/4` 切换，信号与账户视图 `Enter` 下单）。

> ⚠️ 过程说明：原派发的「工程师」Agent 上报「实现完成」但**零代码落地**（仅对 4 个原始文件原地未改、未新增任何模块）。主理人已接管，依据架构设计文档与已核实的 `akshare 0.1.14` API 直接实现并验证。

---

## 已实现需求（对照 PRD P0/P1）

| 域 | 内容 | 状态 |
|----|------|------|
| A 真实数据 | `a_share_candles` 拉取历史 K 线（qfq，250 根）；`stock_zh_a_spot_em` / `stock_zh_index_spot_em` 维持实时快照 | ✅ |
| B 技术指标 | MA/SMA/EMA、MACD(dif/dea/hist)、RSI(14, Wilder)、KDJ(9,3,3)、BOLL(20,2) | ✅（KDJ/BOLL 为 P1，DSL 可引用，默认策略未启用） |
| C 信号 DSL | 递归 `and/or/not` + `gt/lt/gte/lte/eq` + `cross_above/cross_below`；指标表达式 `MA(close,5)`、`MACD(close,12,26,9).dif`、`RSI(close,14)`、`PRICE(close)` 等 | ✅ |
| D 模拟交易 | 市价单、100 股整手向下取整、现金/持仓校验、手续费 0.03% + 印花税 0.05%、已实现/浮动盈亏、成交落盘 `trades.json` | ✅ |
| E TUI | 4 视图 + 顶部 Tab 栏；红涨绿跌（中国习惯）；信号列表光标、`Enter` 下单；账户概览/持仓/成交表 | ✅ |
| F 持久化 | `account.json` / `trades.json` / `strategy.toml`，缺失或损坏安全兜底默认 | ✅ |

---

## 文件清单（全部位于仓库，未提交 git）

新增模块：
- `src/indicators.rs` — `Candle` / `PriceSource` / `IndicatorId` / `Indicator` trait / `IndicatorRegistry` 工厂
- `src/indicators/{ma,macd,rsi,kdj,boll}.rs` — 各指标实现
- `src/signals.rs` — `Side`/`Scope`/`Operand`/`CmpOp`/`CrossDir`/`SignalNode`/`StrategyRule`
- `src/signals/{dsl,eval}.rs` — DSL 解析器 + 信号引擎（沿触发去抖）
- `src/sim.rs` + `src/sim/{account,history}.rs` — 账户 / 成交
- `src/config.rs` — `AppConfig`
- `src/persist.rs` — 账户 / 成交读写
- `src/tests.rs` — 16 项单元测试（`#[cfg(test)]`）
- `strategy.toml` — 示例策略（5 条规则）

改造文件：
- `src/main.rs` — `Msg::Klines`、`data_loop` K 线慢刷新分支、`block_on` 初始化 K 线、`run_app` 信号求值 + 手动下单 + 视图切换键
- `src/market.rs` — `fetch_klines` / `fetch_all_klines`（`CandlePoint` → `Candle`）
- `src/app.rs` — `View` 枚举 + 模拟交易状态字段 + `engine`
- `src/ui.rs` — 改为模块根，分派到 `ui/{market_view,indicator_view,signal_view,account_view}.rs`
- `Cargo.toml` — 新增 `serde` / `serde_json` / `toml 0.8` / `chrono 0.4.20`

设计/需求文档（既有）：`docs/prd-sim-trading.md`、`docs/design-sim-trading.md`。

---

## 运行方式

```bash
cd /Users/tony/github/wbot
cargo run            # 实时行情须联网（akshare 经东方财富/腾讯/Tushare 兜底）
```

- 键位：`1/2/3/4` 切视图；`↑/↓` 或 `j/k` 滚动/移动光标；`Tab` 仅行情视图切涨跌榜；`Enter` 在信号/账户视图对选中标的以最新价下单；`r` 刷新；`q`/`Esc` 退出。
- 策略编辑：`strategy.toml`（热改后重启生效）。账户状态存于 `account.json`、成交存于 `trades.json`。

---

## 验证结果

- `cargo build`：**0 errors / 0 warnings**。
- `cargo test`：**16 passed / 0 failed**（指标数学、DSL 解析与求值、信号沿触发去抖、模拟交易下单/拒绝/盈亏、随包 `strategy.toml` 解析）。
- 修复的关键缺陷：指标窗口切片 `i - period + 1` 在 `usize` 下下溢（已改为 `i + 1 - period`）。

---

## 已知限制 / 后续建议

1. **盘中近似**：日 K 用最新价覆盖末根 `close` 后重算指标与交叉，存在盘中假突破（设计决策 #5）。
2. **K 线刷新**：~60s 整段替换（P0 简化）；P1 可改为收盘后整根追加。
3. **自动下单**（`auto_trade`）：参数与字段已预留，默认关闭；P1 可启用命中信号自动下单。
4. **账户重置**：P1 入口（避免与 `r` 冲突，如 `Ctrl+R`），待实现。
5. 未提交 git —— 如需纳入版本控制请告知，我再帮你 commit。
