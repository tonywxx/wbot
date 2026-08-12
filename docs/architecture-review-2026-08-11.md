# wbot 架构评审报告

> 生成日期：2026-08-11
> 范围：`src/`（37 个 Rust 模块，约 5350 行）
> 目标：在不改变外部行为的前提下，梳理可改进的结构性机会，按收益/风险排序。

---

## 1. 总体结论

wbot 的架构**整体扎实**，远超一般 CLI 工具的水平：

- **清晰的 seams**：`MarketSource` trait + `MarketRouter`（`src/market.rs:100`、`src/market.rs:389`）把 A 股 / 美股 / 加密货币三个数据源收敛到统一 `Candle` 管道，引擎层完全 provider-free。
- **deep module 实践到位**：`src/series.rs` 把原本散落在 `eval` / `backtest` / `app` 三处的「最小长度 / 持仓根数」门槛收口为单一真相源，是教科书式的接口收窄。
- **决策有迹可循**：`docs/adr/` 记录了市场源适配（0001）、加密货币默认优先（0002）、涨跌颜色可配置（0003）三个关键决策。
- **编译零警告**：`cargo build` 干净。

主要短板集中在 **「运行时状态的正确性边界」** 与 **「少量重复逻辑」** 上，而非结构性缺陷。下面按优先级列出。

---

## 2. 问题清单（按优先级）

### P0 — 需要尽快处理的正确性边界

#### P0-1 历史 K 线与实时价的「权威分裂」与回测污染风险
`App::apply_last_price`（`src/app.rs:121`）**原地改写** `app.klines` 最后一根收盘价为实时价：

```rust
pub fn apply_last_price(&mut self, code: &str, price: f64) {
    if let Some(k) = self.klines.get_mut(code) {
        if let Some(last) = k.last_mut() { last.close = price; }
    }
}
```

但 `app.klines` 同时被两处消费：

- `eval_signals`（`src/main.rs:272`）—— 实时信号求值，用改写后的末根，这是**有意**的盘中近似。
- `App::recompute_backtests`（`src/app.rs:186`）—— 它直接读 `&self.klines` 跑回测，于是**实时价被当作历史收盘**参与回测计算。

`apply_snapshot` 与 `apply_quotes` 两条路径都会调用 `apply_last_price`（`src/main.rs:233`、`src/main.rs:261`），且 `merge_klines`（`src/main.rs:314`）每 60s 用整段重拉覆盖，于是「历史序列」在刷新周期内被实时价反复污染。

**风险**：回测报告（UI 内与 `reports/` 生成的）混入了盘中实时价，读者无法 Distinguish 真实历史回测与含尾根实时价的近似回测；且 `app.klines` 不再等于「从数据源拉到的历史」，违反单一真相源原则。

**建议**：
- 把实时价从 `klines` 剥离，改为独立的 `live_overlay: HashMap<String, f64>`（`app.prices` 已近此意，但当前又回写进 klines）。
- 信号求值与回测求值都接受「历史序列 + 可选末根实时覆盖」两个入参，而非改写共享可变状态。
- 若保留「改写末根」作为盘中近似，至少把 klines 拆成 `history: HashMap<…>`（不可变）与 `live: HashMap<…>`，渲染时合并。

---

### P1 — 显著的可维护性问题

#### P1-1 回测聚合逻辑重复实现
`backtest_rule`（`src/backtest.rs:81`）与 `backtest_strategy`（`src/backtest.rs:205`）各自独立实现了 `win_rate / avg_win / avg_loss / profit_factor` 的聚合（两段几乎相同的 ~25 行）。一旦费率口径或盈亏比定义（`+inf` 语义）需要调整，两处极易漂移。

**建议**：抽出一个 `BacktestAccumulator`（`pub(crate)` 纯结构），`backtest_rule` 累积单标的、外层再跨标的累积，单测只覆盖一处。这正是 `src/series.rs` 已经在做的「收口」模式的延伸。

#### P1-2 `MarketRouter` 测试接缝不自洽
`MarketRouter::from_sources`（`src/market.rs:406`）签名只接受 `a` 与 `us`，**硬编码** `crypto: Box::new(OkxSource::new())`。注释声称「测试 / 替换数据源用」，但加密货币源无法注入，测试仍需真实 OKX 网络。只有 `from_sources_full`（`src/market.rs:415`）支持三源全注入。

**建议**：统一为单一 `from_sources_full`，删除 `from_sources`；或让 `from_sources` 也接收 crypto。减少接口歧义。

#### P1-3 数据源错误传播口径不一致
`MarketSource` 的三个方法返回类型各不同：`fetch_klines` → `Result<_, SourceError>`、`fetch_snapshot` → `Option`、`fetch_quotes` → `Vec<Quote>`（失败仅在内部 `eprintln!` 丢弃，见 `src/market.rs:286`、`src/market.rs:126`）。

后果：watchlist 中某只美股 / 加密货币报价失败，UI 完全无感知（静默），与 `fetch_klines` 已经做好的「失败浮出」 seam（`src/market.rs:442`）自相矛盾。

**建议**：`fetch_quotes` 也返回 `Result<Vec<Quote>, SourceError>`（或部分成功 + 失败清单，与 `fetch_all_klines` 一致），由 `fetch_all_quotes` 聚合带回失败清单，UI 状态栏提示。

#### P1-4 初始化阶段的失败被丢弃
`main`（`src/main.rs:472`、`src/main.rs:480`）用 `_init_kerrs` / `_init_ierrs` 显式丢弃初始拉取的错误。若启动时全部标的失败（如断网），程序仍以空数据进入 TUI，用户只见空白。

**建议**：统计 `init_kerrs`，若失败占比超过阈值（或全部失败），打印一次性告警并/或延缓进入 UI，至少把失败数计入启动日志。

---

### P2 — 次要 / 风格层面

#### P2-1 `App` 巨型结构体混合多关注点
`App`（`src/app.rs:34`）持有 ~30 个字段，横跨：行情看板（`data`/`quotes`/`prices`）、模拟交易（`account`/`trades`/`crypto`）、信号引擎（`engine`/`strategies`/`signals`）、UI 游标状态（`signal_cursor`/`trade_cursor`/…）、回测缓存（`backtests`）。`run_app` 的命令式循环（`src/main.rs:136`）把这些关注点交织在键盘/消息分发中。

这不是致命问题（TUI 应用常见），但若后续要加视图/功能，建议把 **UI 状态**（游标、焦点、视图）与 **领域状态**（账户、行情、信号）分离，或全部收到一个 `Reducer`/状态机里，降低 `main.rs` 的耦合。

#### P2-2 同步 / 异步通道混用跨边界
`data_loop`（异步）通过 `std::sync::mpsc::Sender<Msg>` 推送（`src/main.rs:56`），而 `run_app`（同步）用 `try_recv` 消费；反向请求却用 `tokio::mpsc`（`src/main.rs:57`）。两种 mpsc 跨同一条 async/sync 边界，虽功能正确但易误用（同步 mpsc 在 async 任务中 send 会阻塞 worker）。

**建议**：要么全异步（`ui_tx` 也用 `tokio::mpsc`，在 `run_app` 内用 `block_on(try_recv)` 或由 async 渲染任务驱动），要么明确注释该边界的阻塞语义。

#### P2-3 每次真实下单新建运行时
`place_crypto_live`（`src/crypto_gateway.rs:72`）每次下单都 `Builder::new_current_thread().build()` + `block_on`。运行时构建有开销；且该路径在 `data_loop` 之外的同步调用处触发，易在多线程运行时下踩坑。

**建议**：复用一个惰性 `static` 或注入的 `Handle`（如 `tokio::runtime::Handle::try_current()` 失败再建），避免重复构建。

#### P2-4 三个 watchlist 加载函数近乎重复
`load_watchlist` / `load_watchlist_us` / `load_watchlist_crypto`（`src/market.rs:574`–`624`）逻辑完全相同，仅靠文件名常量与默认数组区分。

**建议**：抽为一个 `load_watchlist_named(file: &str, fallback: &[&str]) -> Vec<String>`，三处调用即可。

#### P2-5 `market.rs` 体量偏大（864 行）但内聚
trait 定义、三个适配器、`MarketRouter`、watchlist 加载、涨跌家数计算、解析辅助全在一文件。内容内聚，不强求拆分；若想减负，可把三个 `*Source` 移到 `market::sources` 子模块。

#### P2-6 `backtest.rs` 顶部 `#![allow(dead_code)]`
`src/backtest.rs:4` 整体 `allow(dead_code)` 掩盖了潜在未使用项（如 `BacktestResult` 的部分字段、`bars` 等）。建议改为逐项 `#[allow(dead_code)]`，并清理真正无用的导出，避免掩盖未来误删。

---

## 3. 改进优先级一览

| 编号 | 问题 | 优先级 | 影响 | 工作量 |
|------|------|--------|------|--------|
| P0-1 | 历史 K 线被实时价原地改写、污染回测 | P0 | 回测结果可信度 | 中 |
| P1-1 | 回测聚合逻辑重复 | P1 | 维护 / 口径漂移 | 小 |
| P1-2 | `MarketRouter` 测试接缝不自洽 | P1 | 可测性 | 小 |
| P1-3 | `fetch_quotes` 静默吞错 | P1 | 可观测性 | 小 |
| P1-4 | 初始拉取失败被丢弃 | P1 | 启动健壮性 | 小 |
| P2-1 | `App` 巨型结构体 | P2 | 可扩展性 | 中 |
| P2-2 | sync/async 通道混用 | P2 | 稳健性 | 中 |
| P2-3 | 每次下单新建运行时 | P2 | 性能 | 小 |
| P2-4 | watchlist 加载三连重复 | P2 | 整洁度 | 小 |
| P2-5 | `market.rs` 偏大 | P2 | 可读性 | 小 |
| P2-6 | 整体 `allow(dead_code)` | P2 | 整洁度 | 小 |

---

## 4. 推荐落地顺序

1. **先解决 P0-1**：引入 `live_overlay` 或将 `klines` 拆为 `history` + `live`，让回测只消费不可变历史。这是唯一直接影响「报告/信号可信度」的项。
2. **P1-1 + P1-2 + P1-4**：三处都是小改动、低风险，可一个 PR 完成，立刻提升可测性与启动健壮性。
3. **P1-3**：对齐错误传播口径，让 UI 能提示报价失败。
4. **P2 各项**：按需，不阻塞主线。

---

## 5. 已验证的事实

- `cargo build`：0 warning / 0 error（当前工作区）。
- `docs/adr/` 含 3 篇决策记录，覆盖了本次评审涉及的两处关键 seam（市场源适配、加密货币优先级）。
- 模块边界（`market` / `signals` / `sim` / `indicators` / `backtest` / `ui`）划分清晰，未发现循环依赖或跨层直接复用 provider 原生类型的情况（符合 CONTEXT.md 的 `Candle` 通用单元约定）。
