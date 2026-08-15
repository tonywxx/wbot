# wbot

> [English](README.md)

> 一个基于真实行情数据（**A 股 + 美股 + 加密货币 OKX**）的**模拟交易终端（TUI）**，内置指标计算、完整的 **TA-Lib** 指标库（161 个函数）、信号引擎、形态识别、模拟下单与**策略回测报告生成**。

`wbot` 是一个 Rust 编写的命令行交易助手，使用 [`akshare`](https://crates.io/crates/akshare) 拉取 A 股与指数行情，使用 [`yfinance-rs`](https://github.com/gramistella/yfinance-rs)（Yahoo Finance）接入美股，并使用 [`adaq-trading-crypto`](https://crates.io/crates/adaq-trading-crypto)（ccxt 兼容，OKX 现货 + WebSocket 实时）接入 OKX 加密货币——三者统一在单一的 `Candle` 数据流之上。指标计算由 [`adaq-talib`](https://crates.io/crates/adaq-talib)（纯 Rust、零 FFI 的 TA-Lib 0.7.1 重实现）提供，可在策略 DSL 中直接以 `TA_<FUNC>(...)` 形式引用**全部 161 个 TA-Lib 函数**。在终端里你可以全键盘浏览行情、查看技术指标、跟踪策略信号、进行无风险的模拟交易，并能对 `strategy.toml` 中的每一条策略批量回测、自动生成可读性强的 Markdown 回测报告。

> **配色约定**：终端采用中国习惯 —— **红涨绿跌**。

---

## 目录

- [功能特性总览](#功能特性总览)
- [架构设计](#架构设计)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [安装与构建](#安装与构建)
- [使用方法](#使用方法)
  - [1. 启动 TUI 终端](#1-启动-tui-终端)
  - [2. TUI 视图与快捷键](#2-tui-视图与快捷键)
  - [3. 生成策略回测报告](#3-生成策略回测报告)
  - [4. 常见操作示例](#4-常见操作示例)
- [配置说明](#配置说明)
  - [自选股列表](#自选股列表)
  - [策略文件 strategy.toml](#策略文件-strategytoml)
  - [config.toml 与参数](#configtoml-与参数)
- [回测报告说明](#回测报告说明)
- [模拟交易说明](#模拟交易说明)
- [美股支持](#美股支持)
- [加密货币（OKX）支持](#加密货币okx-支持)
- [国际化（i18n）](#国际化i18n)
- [风险提示与免责声明](#风险提示与免责声明)
- [开发指南](#开发指南)

---

## 功能特性总览

所有引擎（指标、信号、回测、模拟交易）都运行在「与市场无关」的 `Candle` 数据流之上，因此**下列每一项功能对 A 股、美股与 OKX 加密货币同时生效** —— 区别仅在于数据源。标的按代码形态自动分流：

- **6 位纯数字**（`600519`）→ A 股（akshare）
- **含连字符**（`BTC-USDT`）→ 加密货币（OKX）
- **其余形式**（`AAPL`、`BRK-B`）→ 美股（Yahoo Finance）

### 1. 实时行情终端（`行情` 视图）

- 实时**指数条**（如上证指数、深证成指、创业板指）含最新价与涨跌幅。
- **市场广度面板**：全市场上涨 / 下跌 / 平盘 / 涨停 / 跌停家数统计，以及总数。
- **自选股表**：所跟踪标的的代码、名称、最新价、涨跌幅（含 A 股、美股、加密货币）。
- **涨幅榜 / 跌幅榜**：各取前 30 名，`Tab` 切换焦点。
- 快照每 **5 秒**刷新；日线增量约每 60 秒推送一次；分钟 K 线每 `intraday_refresh` 秒（默认 120 秒）刷新。

### 2. 技术指标（`指标` 视图）

对选中个股实时计算并展示指标数值：

- **均线**：MA5 / MA10 / MA20。
- **RSI(14)**。
- **MACD**：DIF、DEA、HIST（红绿着色）。
- **BOLL**：中轨 / 上轨 / 下轨。
- **KDJ**：K / D / J 线。
- **多空排列**提示（`MA5 > MA10` → 短期多头）。
- 按 `↑` / `↓` 在自选股间切换，逐一查看各标的指标。
- 除以上内置指标外，策略规则中还可通过 `TA_<FUNC>(...)` 的 DSL 语法使用**全部 161 个 TA-Lib 函数**（见[功能 12](#12-ta-lib-指标库)）。

### 3. 信号引擎（DSL）

- 基于 `strategy.toml` 的 DSL 表达式逐标的求值。
- **沿触发**（仅当条件由假变真时触发），避免重复计数。
- 新信号触发**桌面通知**（同一「标的, 规则」带冷却去重）。

### 4. 形态识别

- 支持 `double_golden` —— 双金叉 / 双死叉回踩的**顺序状态机**，用于 15 分钟、60 分钟等周期（需连续判断，点状 DSL 无法表达）。

### 5. 模拟交易（`账户` 视图）

- **虚拟账户**，初始资金 **¥1,000,000**（A 股）/ 等值美元（美股）/ **100,000 USDT**（加密货币）。
- **市价单**：以最新价买入/卖出。A 股/美股按整手（`lot_size`，默认 100）成交；加密货币按 USDT 预算（`crypto_lot_usdt`，默认 1000）换算为基础币数量。
- **费用模型**：双边佣金 + 单边印花税（A 股），或单边 `crypto_fee_rate`（加密货币），实时扣减。
- **账户看板**：总资产、总盈亏、浮动盈亏、已实现盈亏。
- **持仓表**：数量、成本、现价、市值、盈亏。
- **成交记录**：每笔成交的方向、价格、数量、已实现盈亏。
- 持久化：成交追加写入 `trades.json`，账户快照写入 `account.json`；重启自动加载，可连续跟踪模拟持仓。

### 6. 策略管理（`策略` 视图）

- 浏览**全部策略**，并展示当前选中个股的**实时回测胜率**。
- 每条策略显示状态（启用/停用）、方向（买/卖）、名称、胜率、触发次数。
- 下方明细面板展示策略备注与完整回测统计（交易次数、胜率、均盈、均亏、盈亏比、最大回撤、累计收益）。
- 按 `空格` 即时启用 / 停用。

### 7. 回测引擎与报告

- 在历史 K 线上重放每条策略信号：对每次触发采取「买入 / 卖出后**前向持有 N 根**」的方式（日线持有 10 根、分钟持有 5 根）计算收益，并汇总：
  - **胜率**、**平均盈利 / 平均亏损**、**盈亏比**、**总收益率**、**最大回撤**。
- 对每条策略输出独立的 `<id> 策略回测报告.md`：策略元数据 + 跨标的汇总表 + 分标的明细表 + 免责声明。
- 支持**A 股、美股、加密货币**三套独立回测（见[回测子命令](#3-生成策略回测报告)）。

### 8. 多周期支持

日线 DSL 策略与带 `timeframe` 的分钟（T+0）策略分别使用对应周期 K 线求值，互不混淆。

### 9. 通知与持久化

- 桌面通知：macOS 通过 `osascript`，其它平台降级为 stderr 输出。
- 安全持久化：文件缺失/损坏均有兜底，程序不会 panic。

### 10. 加密货币（OKX）支持

- 作为**第三个市场**接入同一套 `Candle` 引擎：OKX 现货交易对（如 `BTC-USDT`、`ETH-USDT`）。
- 历史 K 线经 [`adaq-trading-crypto`](https://crates.io/crates/adaq-trading-crypto) 的 `fetch_ohlcv`（REST）拉取并映射为 `Candle`；日线 `1D` 与分钟 `1m`/`5m`/`15m`/`30m`/`1H`。
- **模拟加密货币账户**（`CryptoLedger`）：USDT 现金 + 基础币持仓，含均价成本跟踪——无需任何凭证即可使用。
- **可选真实下单**：在 `config.toml` 设置 `live_trading = true` 并导出 `OKX_API_KEY` / `OKX_API_SECRET` / `OKX_PASSPHRASE` 后，TUI 中回车会额外向 OKX 发送真实市价单（失败仅告警，本地账本仍更新）。
- 同一套 43 条策略可通过 `backtest crypto` 在加密货币对上回测，输出到 `reports_crypto/`。

### 11. 国际化（i18n）

- 整套界面与所有回测报告均本地化。默认**英文**；在 `config.toml` 设置 `language = "zh-CN"`（或 `zh` / `chinese`）即可将界面与报告整体切换为**简体中文**。
- 所有文案均经过 `tr()` 取词，未知 key 回退英文，绝不 panic。

### 12. TA-Lib 指标库

- 通过 [`adaq-talib`](https://crates.io/crates/adaq-talib)（纯 Rust、零 FFI 的 TA-Lib 0.7.1 重实现）在策略 DSL 中开放**全部 161 个 TA-Lib 函数**，因此**无需在本机安装任何 C 库**。
- 以 `TA_<FUNC>(...)` 形式引用任意函数，例如 `TA_RSI(close,14)`、`TA_MACD(close,12,26,9).hist`、`TA_BBANDS(close,20,2).upper`、`TA_ADX(close,14)`，以及蜡烛图形态如 `TA_CDLHAMMER(close)`。
- 覆盖 TA-Lib 全部分组：重叠研究（Overlap Studies）、动量指标（Momentum）、成交量指标（Volume）、波动率指标（Volatility）、价格变换（Price Transform）、周期指标（Cycle）、形态识别（Pattern Recognition，共 61 种蜡烛图形态）、统计函数（Statistic Functions）、数学变换（Math Transform）、数学运算（Math Operators）。
- 多输出函数用 `.0` / `.1` / `.2` 或输出名选择一个序列；序列前若干根不足计算长度时输出 `NaN`，不参与信号比较。
- 每个函数的中英文含义、参数表与 DSL 示例由 `cargo run --example ta_indicators_list` 生成，输出到 `docs/ta-lib-indicators.bilingual.md`。

---

## 架构设计

```
                ┌─────────── data_loop (tokio 异步) ───────────┐
   akshare ────►  fetch_market()  ──► 快照(指数+个股, 每 5s)     │
   yfinance ──►  fetch_klines()  ──► 日线 K 线(约每 60s)         │
   okx (adaq-trading-crypto) ─► fetch_ohlcv()+WS ─► 加密货币 K 线(1D / 分钟) │
                 fetch_intraday()──► 分钟 K 线(每 intraday_refresh)│
                └─────────────────────┬─────────────────────────┘
                                       │ Msg<快照/K线/分钟K线>
                ┌──────────────────────▼────────────────────────┐
   run_app() ──► apply_snapshot → eval_signals(引擎) → 通知+重算 │
                 handle_enter → account.place_order / 加密账本 → 持久化│
                └────────────────────────────────────────────────┘

   回测路径(独立 CLI):  backtest_cli → 拉取 K 线 → backtest::
   write_strategy_reports → "<市场>/<id> 策略回测报告.md"
```

信号 / 指标 / 回测 / 模拟交易引擎均「与市场无关」，只消费 `Candle` 序列。行情路由由 `MarketRouter` 基于 `MarketSource` trait 派发——A 股走 `AkShareSource`、美股走 `YfSource`、加密货币走 `OkxSource`——按代码形态（`market_of`）选择。因为各级引擎只看到 `Candle`，同一套策略可在任意市场运行。

---

## 技术栈

- **语言**：Rust（Edition 2024）
- **TUI**：[`ratatui`](https://ratatui.rs) 0.30 + `crossterm` 0.29（交叉平台终端控制）
- **异步**：`tokio` 1.53（多线程运行时）
- **行情数据**：
  - A 股与指数：[`akshare`](https://crates.io/crates/akshare)（`equity` feature）
  - 美股：[`yfinance-rs`](https://github.com/gramistella/yfinance-rs) 0.9（Yahoo Finance）
  - 加密货币（OKX）：[`adaq-trading-crypto`](https://crates.io/crates/adaq-trading-crypto)（`okx` + `realtime` feature —— 历史 K 线走 REST，实时价走 WebSocket）；`rust_decimal` + `rustls`（ring provider）提供 TLS
  - A 股 / 美股实时报价：自定义直连 HTTP 客户端（`src/market/realtime/`，`reqwest` 0.13）
- **指标计算**：[`adaq-talib`](https://crates.io/crates/adaq-talib) 0.1.5 —— 纯 Rust、零 FFI 的 TA-Lib 0.7.1 重实现（共 161 个函数，在 DSL 中以 `TA_<FUNC>(...)` 引用）；另含内置 MA / RSI / MACD / KDJ / BOLL
- **配置 / 序列化**：`serde` + `toml` + `serde_json`
- **时间**：`chrono`；**错误**：`anyhow`；**数值**：`num-traits`

---

## 项目结构

```
wbot/
├── Cargo.toml                 # 库 + 二进制 + examples 定义
├── config.toml                # 可选全局配置（语言、费率、加密货币等），启动时读取
├── src/
│   ├── lib.rs                 # 暴露所有模块为 `wbot::`，供二进制与 examples 共享
│   ├── main.rs                # 二进制入口：TUI 主循环 + `backtest` / `probe` 子命令
│   ├── app.rs                 # 应用状态（视图、焦点、账户、加密账本、信号求值、回测）
│   ├── market/                # 行情数据模块（拆分为子模块）
│   │   ├── mod.rs             # 模块根
│   │   ├── types.rs           # 共享行情类型（Quote、Snapshot、涨跌家数等）
│   │   ├── source.rs          # MarketSource trait + A 股(akshare) / 美股(yfinance) 数据源
│   │   ├── router.rs          # MarketRouter：按代码形态（market_of）派发
│   │   └── realtime/          # 实时报价源（直连 HTTP）：eastmoney.rs、yahoo.rs、http.rs、mod.rs
│   ├── crypto.rs              # OKX 集成（adaq-trading-crypto）：历史 K 线 + WebSocket 实时 + CryptoLedger
│   ├── crypto_gateway.rs      # OKX 真实下单网关（发送真实市价单）
│   ├── ledger_core.rs         # 账本通用原语（现金、持仓、均价成本）
│   ├── i18n.rs                # 国际化：Lang + tr() 取词（en/zh），回测报告文案
│   ├── indicators.rs          # Candle、PriceSource、Indicator trait、IndicatorRegistry、build_indicator
│   ├── indicators/            # ma、macd、rsi、kdj、boll、ta（adaq-talib 派发）、ta_dispatch
│   ├── signals.rs             # StrategyRule、RawRule、parse_strategy_file、Scope/Side 枚举
│   ├── signals/               # dsl（递归解析器）、eval（信号引擎）、double_cross（形态状态机）
│   ├── sim.rs                 # 模拟交易模块根
│   ├── sim/                   # account.rs（Account/Position/Order）、crypto_ledger.rs、history.rs（Trade 持久化）
│   ├── series.rs              # 序列通用工具（最小长度 / 持仓根数门槛——单一真相源）
│   ├── config.rs              # AppConfig 默认值 + load_config()（读取 config.toml）
│   ├── persist.rs             # account.json / trades.json 读写
│   ├── notify.rs              # 桌面通知器（冷却 + 去重）
│   ├── backtest.rs            # 回测引擎 + Markdown 报告渲染（支持 i18n）
│   ├── backtest_cli.rs        # 异步报告生成（A 股 / 美股 / 加密货币），二进制与 example 共用
│   ├── ui.rs                  # ratatui 渲染分派 + 标签/头部/指数/页脚
│   ├── ui/                    # market_view、indicator_view、signal_view、account_view、strategy_view
│   └── tests.rs               # 单元测试
├── examples/
│   ├── backtest_all.rs        # 复用 wbot::backtest_cli 的回测示例
│   └── ta_indicators_list.rs  # 依据 TA-Lib 元信息生成 docs/ta-lib-indicators.bilingual.md
├── strategy.toml              # 策略定义（用户可自由编辑；内置 43 条）
├── watchlist.txt              # 美股自选股列表（默认读取）
├── watchlist_a.txt            # A 股自选股列表
├── watchlist_crypto.txt       # OKX 加密货币自选列表（可选；存在时 TUI 启用加密货币）
├── reports/                   # A 股回测报告输出目录（自动生成）
├── reports_us/                # 美股回测报告输出目录（自动生成）
└── reports_crypto/            # 加密货币回测报告输出目录（自动生成）
```

---

## 安装与构建

### 环境要求

- 已安装 [Rust 工具链](https://rustup.rs/)（建议使用较新版本以支持 Edition 2024）。
- 可访问所使用数据源的网络：`akshare`（A 股行情）、Yahoo Finance（美股行情）和/或 OKX（加密货币行情）。TUI 仅在对应自选股文件存在时才会访问该数据源，因此可仅用 A 股而无需美股/加密货币连通性。

### 构建

```bash
# 克隆或进入项目目录后
cargo build --release      # 编译发布版本（较慢但运行更快）
cargo build                # 编译调试版本
```

> 首次构建会拉取并编译 `ratatui`、`tokio`、`akshare`/`reqwest`、`yfinance-rs`/`polars`、`adaq-talib`、`adaq-trading-crypto` 等依赖，耗时较长，请耐心等待。

---

## 使用方法

### 1. 启动 TUI 终端

```bash
cargo run                 # 调试模式启动
# 或
cargo run --release       # 发布模式启动
```

启动后会自动：

1. 按顺序加载 `watchlist.txt`（美股）/ `watchlist_crypto.txt`（加密货币）/ `watchlist_a.txt`（A 股）与 `strategy.toml`；
2. 加载 `config.toml`（语言、费率、加密货币等参数），缺失则使用默认值；
3. 拉取历史日线 / 分钟 K 线完成初始化；
4. 进入全屏 TUI，开始实时推送行情并求值信号。

退出：按 `q` 或 `Esc`。

### 2. TUI 视图与快捷键

切换视图（数字键 `1`–`5`）：

| 按键 | 视图 | 内容 |
| --- | --- | --- |
| `1` | 行情 Market | 指数条 + 市场广度 + 自选股 + 涨/跌幅榜（Tab 切换焦点） |
| `2` | 指标 Indicators | 选中标的的 MA5/MA10/MA20、RSI14、MACD、BOLL、KDJ（`↑`/`↓` 切换标的） |
| `3` | 信号 Signals | 当前触发的买卖信号列表（光标选中后按 Enter 下单） |
| `4` | 账户 Account | 虚拟账户资金、持仓、成交（按 Enter 对选中标的买入） |
| `5` | 策略 Strategies | 全部策略及其实时回测胜率（空格启用 / 停用） |

全局快捷键：

| 按键 | 作用 |
| --- | --- |
| `↑` / `k` | 上移光标（指标视图内切换标的） |
| `↓` / `j` | 下移光标（指标视图内切换标的） |
| `Tab` | 行情视图内切换「涨幅榜 / 跌幅榜」焦点 |
| `空格` | 策略视图内启用 / 停用当前策略 |
| `r` | 强制刷新实时行情 |
| `Enter` | 信号 / 账户视图内按最新价市价下单（A 股 / 美股 / 加密货币） |
| `q` / `Esc` | 退出程序 |

> 信号触发后会通过系统桌面通知提示（可在配置中关闭）。下单为**模拟成交**，仅写入本地 `trades.json` / `account.json`，不涉及真实资金；仅当为加密货币设置 `live_trading = true` 时才会真实下单（见[加密货币支持](#加密货币okx-支持)）。

### 3. 生成策略回测报告

提供四种方式，对 `strategy.toml` 中的**每一条策略**跑回测并生成独立 Markdown 报告，由子命令选择市场：

```bash
cargo run -- backtest reports                  # A 股     -> ./reports
cargo run -- backtest us reports_us            # 美股     -> ./reports_us
cargo run -- backtest crypto reports_crypto    # OKX 加密 -> ./reports_crypto
cargo run -- backtest all                       # 三个市场全跑（reports/ + reports_us/ + reports_crypto/）
```

`examples` 路径与二进制对应：

```bash
cargo run --example backtest_all -- reports     # A 股 -> ./reports
cargo run --example backtest_all us             # 美股 -> ./reports_us
cargo run --example backtest_all crypto         # 加密 -> ./reports_crypto
cargo run --example backtest_all all            # 三个市场全跑
```

运行后会输出类似：

```
== A-share：已生成 43 份策略回测报告 ==
  - ma_golden : reports/ma_golden 策略回测报告.md
  - s01_ma_bull_arr : reports/s01_ma_bull_arr 策略回测报告.md
  ...
```

每一条策略生成一份 `<id> 策略回测报告.md`，覆盖日线 DSL、经典策略、T+0 日内策略与形态策略——对应你选择的市场。

### 4. 常见操作示例

**a) 添加你的第一条自定义策略。** 编辑 `strategy.toml`，追加一条规则，重载 TUI 即可 —— 无需改代码：

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

**b) 回测单个市场并查看报告：**

```bash
cargo run -- backtest reports                  # A 股 -> ./reports
cargo run -- backtest us reports_us            # 美股 -> ./reports_us
cargo run -- backtest crypto reports_crypto    # 加密 -> ./reports_crypto
# 然后打开任意报告，例如：
open "reports/ma_golden 策略回测报告.md"   # macOS
```

**c) 在 TUI 中模拟一笔交易：**

1. 按 `1` → 行情，挑选一只标的；
2. 按 `3` → 信号，若高亮某条买入信号，按 `Enter` 以最新价买入一手；
3. 按 `4` → 账户，查看总资产、盈亏、持仓与成交记录。

**d) 美股标的：** 编辑 `watchlist.txt`（每行一个 ticker）；美股自选在列表中默认最先加载，对美股标的的信号求值与 A 股完全一致。

**e) 在 TUI 中启用加密货币：** 创建 `watchlist_crypto.txt`（每行一个 `BASE-USDT` 交易对，如 `BTC-USDT`）；TUI 合并后可在账户视图按 `Enter` 模拟加密货币交易。

**f) 切换界面为中文：** 在 `config.toml` 设置 `language = "zh-CN"` 后重启，所有面板与回测报告均以简体中文呈现。

**g) 快速查看某策略的实时胜率：** 按 `5` → 策略，移动光标到任意策略，在明细面板即可读到当前选中个股的胜率与触发次数。

---

## 配置说明

### 自选股列表

工作目录下有三个可选自选股文件，每行一个标的，`#` 开头为注释可附备注。文件**仅在存在时**被加载，因此默认 TUI 仅含 A 股，待你显式开启。

| 文件 | 市场 | 格式 | 缺失回退 |
| --- | --- | --- | --- |
| `watchlist.txt` | 美股 | ticker（如 `AAPL`、`BRK-B`） | 内置 26 只流动性美股 |
| `watchlist_a.txt` | A 股 | 6 位代码（如 `600519`） | 内置 10 只流动性 A 股 |
| `watchlist_crypto.txt` | OKX 加密 | `BASE-USDT` 交易对（如 `BTC-USDT`） | 内置 10 只流动性交易对 |

加载顺序为美股 → 加密货币 → A 股。某个市场的自选文件存在但没有任何标的（空文件或仅注释）时，该市场跳过、不加载；文件缺失才回退内置默认列表。

示例 `watchlist_crypto.txt`：

```
# 加密货币自选清单（OKX 现货，BASE-USDT 交易对）
BTC-USDT
ETH-USDT
SOL-USDT
```

- 若文件缺失或为空，则使用对应的内置默认列表。
- 回测与信号求值均基于已加载列表内的标的。

### 策略文件 strategy.toml

策略分为两类：**DSL 条件策略** 与 **形态（pattern）策略**。

#### DSL 条件策略

```toml
[[rules]]
id = "ma_golden"                       # 唯一标识（回测报告文件名用）
label = "MA5 上穿 MA10 (金叉)"          # 展示名
side = "buy"                           # "buy" / "sell"
scope = "watchlist"                    # "watchlist" 或逗号分隔代码，如 "600519,000858"
enabled = true                         # 是否启用
signal = "cross_above(MA(close,5), MA(close,10))"   # 条件表达式
note = "短期均线金叉，经典趋势启动信号"  # 备注（策略视图展示）
```

DSL 支持（函数名大小写不敏感）：

- **逻辑**：`and(...)` `or(...)` `not(...)`
- **比较**：`gt(a,b)` `lt(a,b)` `gte(a,b)` `lte(a,b)` `eq(a,b)`
- **交叉**：`cross_above(a,b)`（上穿）`cross_below(a,b)`（下穿）
- **指标**（来源默认 `close`）：`MA(src,p)` `SMA(src,p)` `EMA(src,p)` `RSI(src,p)`
  - `MACD(src,fast,slow,sig).dif / .dea / .hist`
  - `KDJ(n,k,d).k / .d / .j`
  - `BOLL(p,k).mid / .upper / .lower`
- **TA-Lib**（经 `adaq-talib` 提供的全部 161 个 TA-Lib 0.7.1 函数）：`TA_<FUNC>(...)`。第一个参数为价格来源（`close`、`open`、`high`、`low`、`volume`），其余为可选的 TA-Lib 参数。多输出函数用 `.0` / `.1` / `.2` 或输出名选择序列，例如 `TA_MACD(close,12,26,9).hist`、`TA_BBANDS(close,20,2).upper`。蜡烛图形态函数返回 `0` / `100` / `-100`，例如 `TA_CDLHAMMER(close)`。完整函数列表、参数表与 DSL 示例见 `docs/ta-lib-indicators.bilingual.md`。
- **价格**：`PRICE(close)`（也支持 `open` / `high` / `low` / `volume`）

#### 分钟（T+0）DSL 策略

带 `timeframe` 与 `bars` 的 DSL 规则会改用对应分钟 K 线求值（非日线）：

```toml
[[rules]]
id = "t0_buy_rsi"
label = "T+0 日内超卖低吸"
side = "buy"
scope = "watchlist"
enabled = true
timeframe = "5"        # 分钟周期：1/5/15/30/60（加密货币用 1m/5m/15m/30m/1H）
bars = 60              # 回看根数
signal = "lt(RSI(close,14), 25)"
note = "5 分钟 RSI<25 日内超卖，适合低吸做 T 的买点"
```

#### 形态（pattern）策略

形态规则使用**顺序状态机**（非点状 DSL），适合双金叉回踩这类需要连续判断的模式：

```toml
[[rules]]
id = "dg_buy_15"
label = "15m 双金叉回踩买入"
side = "buy"
scope = "watchlist"
enabled = true
kind = "pattern"               # 声明为形态规则
pattern = "double_golden"      # 形态名
timeframe = "15"               # 分钟周期
fast = 5                       # EMA 快线周期
slow = 10                      # EMA 慢线周期
bars = 100                     # 回看根数（Sina 1H 约 36 根，自动截断）
higher_low = true              # 回踩不破前低 / 反抽不过前高
note = "15 分钟双金叉回踩不破前低，末根成本高于慢线，多头成本基础确立后买入"
```

### config.toml 与参数

`wbot` 在启动时通过 `load_config()` 读取工作目录下可选的 `config.toml`。所有字段均可选；缺失/未知字段以及文件缺失/解析失败都会安全回退到 `AppConfig::default()`，绝不 panic。仓库已附一份可直接编辑的 `config.toml`。

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `language` | `"en"` | 界面与报告语言：`en`（默认）或 `zh` / `zh-CN` / `chinese`（简体中文） |
| `commission` | `0.0003` | 双边手续费率（万三，A 股/美股） |
| `stamp_tax` | `0.0005` | 印花税（仅卖出，万五，A 股概念） |
| `lot_size` | `100` | 每手股数（A 股/美股） |
| `auto_trade` | `false` | 自动下单开关（默认仅手动） |
| `kline_adjust` | `"qfq"` | K 线复权方式（前复权；加密货币现货忽略此选项） |
| `kline_count` | `250` | 日线保留根数 |
| `intraday_refresh` | `120` | 分钟 K 线刷新间隔（秒） |
| `notify_enabled` | `true` | 是否发送桌面通知 |
| `notify_cooldown` | `300` | 同一（标的, 规则）通知冷却（秒） |
| `crypto_enabled` | `true` | 是否启用 OKX 加密货币数据源与交易支持 |
| `live_trading` | `false` | 是否发送 OKX 真实下单（需 `OKX_API_KEY`/`OKX_API_SECRET`/`OKX_PASSPHRASE` 环境变量） |
| `crypto_lot_usdt` | `1000.0` | 单次买入预算（USDT），按成交价换算为基础币数量 |
| `crypto_fee_rate` | `0.001` | 加密货币单边手续费率（0.1%） |
| `default_scope` | `"watchlist"` | 策略默认作用范围 |

> 模拟 A 股/美股账户初始资金为 **¥1,000,000**；模拟加密货币账户初始资金为 **100,000 USDT**。

---

## 回测报告说明

回测引擎在历史 K 线上重放策略信号，对**每一次信号触发**采取「买入 / 卖出后前向持有 N 根」的方式计算收益（日线持有 10 根、分钟线持有 5 根），并汇总：

- **胜率**：盈利次数 / 总触发次数
- **平均盈利 / 平均亏损**
- **盈亏比（Profit Factor）**
- **总收益率**
- **最大回撤**

每份 `<id> 策略回测报告.md` 包含：

1. **策略信息**：id、名称、方向、作用范围、信号表达式 / 形态参数、数据区间、覆盖标的数量、持有周期；
2. **跨标的汇总表**：各标的胜率、交易次数、累计收益等横向对比；
3. **分标的明细表**：逐标的的回测统计；
4. **免责声明**：回测为历史模拟，不代表未来收益。

> 报告中的「胜率」由回测引擎在所选个股上实测，并非主观断言；TUI 策略视图也会实时展示当前标的的回测胜率。报告文案遵循 `config.toml` 的 `language` 设置。

---

## 模拟交易说明

- 启动后在「账户 Account」视图按 `Enter` 可对当前选中标的以**最新价市价买入**；在「信号 Signals」视图选中某条信号后按 `Enter` 按其方向（买 / 卖）下单。对 A 股、美股、加密货币标的均适用。
- **A 股 / 美股**：每笔下单按 `lot_size`（默认 100 股）整手成交，手续费与印花税按 `config` 参数实时扣减；现金不足拒买、持仓不足拒卖。
- **加密货币**：买入将 `crypto_lot_usdt`（默认 1000 USDT）换算为对应基础币数量；卖出清掉全部持仓。单边费率为 `crypto_fee_rate`（默认 0.1%）；USDT 或持仓不足则拒单。默认仅更新本地 `CryptoLedger`，不动用真实资金。
- 成交记录追加写入 `trades.json`，账户快照写入 `account.json`；重新启动会自动加载，便于连续跟踪模拟持仓。

---

## 美股支持

`wbot` 并非只服务于 A 股：所有引擎（指标、信号求值、回测、模拟交易）都运行在「与市场无关」的 `Candle` 数据流之上，因此**同一套**策略与报告可直接用于美股，差别仅在于数据源 —— 按代码形态自动选择：

- **A 股代码**——6 位纯数字（如 `600519`）→ 经 `akshare` 拉取。
- **美股代码**——其余形式（如 `AAPL`、`BRK-B`）→ 经 `yfinance-rs`（Yahoo Finance）拉取。注意伯克希尔哈撒韦在 Yahoo 中写作 `BRK-B`（连字符），而非 `BRK.B`。

### 美股自选股

编辑 `watchlist.txt`（每行一个 ticker，`#` 注释，格式同 A 股列表）即可调整美股自选股。文件缺失时使用内置的默认美股名单；文件存在但没有任何标的时，美股市场跳过。TUI 始终合并美股 / 加密货币 / A 股三个市场。

### 回测美股

```bash
cargo run -- backtest us                # 美股回测，写入 ./reports_us
cargo run -- backtest us my_us_reports  # 自定义输出目录
cargo run --example backtest_all us     # 写入 ./reports_us
```

报告落在 `reports_us/`（或你指定的目录），文件名 `<id> 策略回测报告.md`，结构与 A 股报告一致（策略信息 + 跨标的汇总 + 分标的明细），并带 `市场：美股` 行。

### 注意事项

- **币种**：美股报价以 **USD（美元）** 计。TUI 与回测报告中的持仓市值、盈亏、收益率均以美元计价，而非人民币。
- **费用**：回测 / 模拟交易引擎的费用来自 `AppConfig`（`src/config.rs`）——双边 `commission` + 单边 `stamp_tax`（印花税）。印花税是 A 股特有概念，美股并不征收；纯美股回测时可将 `stamp_tax` 设为 `0`，以免高估卖出成本。`lot_size`（整手）默认 100 股对美股同样适用。
- **周期与历史**：美股日线取 Yahoo `range=1y`（约 252 根）；分钟线取 `range=1mo`，支持 `1 / 5 / 15 / 30 / 60` 分钟。与 A 股一致，策略的 `bars` 回看会截断为最近 N 根，因此 `bars=60` 的 5 分钟策略仅覆盖约 1 个交易日。
- **TUI 行为**：美股标的（来自 `watchlist.txt`）的信号求值与模拟交易与 A 股完全一致，美股行情以美元进入看板。
- **代码格式**：请按 Yahoo 的写法传入代码——伯克希尔哈撒韦写作 `BRK-B`（连字符）、`BF-B` 等；股份类别后缀用连字符，切勿用点号。

> **网络说明：** 美股数据来自 Yahoo Finance。在部分沙箱 / 机房网络中，Yahoo 会返回 HTTP `429`（限流 / 反爬），此时取不到美股 K 线，报告各标的为 `N/A`。请在可正常访问 Yahoo 的机器上运行以获得真实回测结果——代码路径完全一致。

---

## 加密货币（OKX）支持

`wbot` 把加密货币当作**一等公民第三市场**对待：同一套 `Candle` 引擎、指标数学、信号 DSL 与回测全部适用于 OKX 现货交易对。任何含连字符的代码（如 `BTC-USDT`、`ETH-USDT`）会被自动路由到 OKX 数据源。

### 数据源

- 历史 K 线经 [`adaq-trading-crypto`](https://crates.io/crates/adaq-trading-crypto) 的 `fetch_ohlcv`（REST）拉取并映射为 `Candle`：日线 `1D` 与分钟 `1m`/`5m`/`15m`/`30m`/`1H`。
- **实时价格**通过双连接 WebSocket（`OkxWs`，主备两连接 + 多路复用订阅）推送——见 `src/crypto.rs::spawn_realtime_feed`。原先基于 `reqwest` 的 `/market/candles` 轮询路径已被替换。
- 加密货币没有 A 股式全市场盘口快照，因此行情视图的广度/指数面板仍为 A 股专属；加密货币标的会以最新价出现在自选股表与涨跌幅榜中。

### 加密货币自选股

创建 `watchlist_crypto.txt`（每行一个 `BASE-USDT` 交易对）。文件缺失时回退到内置 10 只流动性交易对（`BTC-USDT`、`ETH-USDT`、`SOL-USDT` …）。TUI 仅在文件存在时合并它。

### 模拟加密货币交易

- 在账户/信号视图对加密货币标的按 `Enter`，更新本地 `CryptoLedger`——一个含 **100,000 USDT** 现金与基础币持仓、含均价成本跟踪的模拟账户，无需任何凭证。
- 买入将 `crypto_lot_usdt`（默认 1000 USDT）按最新价换算为基础币数量；卖出清掉全部持仓。单边费率为 `crypto_fee_rate`（默认 0.1%）。
- USDT 或持仓不足则拒单。

### 可选真实下单

若需真正向 OKX 发单：

1. 在 `config.toml` 设置 `live_trading = true`；
2. 在环境中导出凭证：

   ```bash
   export OKX_API_KEY=...
   export OKX_API_SECRET=...
   export OKX_PASSPHRASE=...
   ```

此时 TUI 中回车还会向 OKX 下**真实市价单**（现货，`cash` 模式）。若真实下单失败，仅打印告警——本地模拟账本仍会更新。在 `live_trading = false`（默认）时绝不发送任何网络订单。

### 探测 OKX 连通性

`wbot` 提供 `probe` 子命令，可**同时**验证加密货币（OKX）的 REST 历史 K 线路径（`fetch_ohlcv`）与 WebSocket 实时 ticker，便于在依赖加密货币数据前确认连通性：

```bash
cargo run -- probe
```

### 回测加密货币

```bash
cargo run -- backtest crypto                      # 加密回测，写入 ./reports_crypto
cargo run -- backtest crypto my_crypto_reports    # 自定义输出目录
cargo run --example backtest_all crypto           # 写入 ./reports_crypto
```

报告落在 `reports_crypto/`，文件名 `<id> 策略回测报告.md`，结构与其它市场一致并带 `市场：加密货币` 行。OKX 公开 K 线接口通常可达，因此即便在沙箱环境中加密货币回测也能得到真实结果。

### 注意事项

- **币种**：加密货币报价以 **USDT** 计；盈亏与收益率均以 USDT 计价。
- **费用**：使用单边 `crypto_fee_rate`（默认 0.1%），无 A 股式印花税。
- **代码格式**：请按 OKX 现货合约精确书写——`BTC-USDT`、`ETH-USDT`（连字符）。正是连字符触发了加密货币路由，请勿写作 `BTC.USDT`。

> **风险提示：** `live_trading = true` 会向你自己的 OKX 账户下**真实订单、动用真实资金**。模拟账本与真实账户相互独立，仅模拟账本会被本地持久化。启用前请务必核对 `crypto_lot_usdt` 与环境变量。

---

## 国际化（i18n）

界面与所有回测报告均已完整本地化。

- **默认**：英文。
- **切换简体中文**：在 `config.toml` 设置 `language = "zh-CN"`（也可接受 `zh`、`zh_cn`、`chinese`、`中文`、`中文版`），重启即可。

每条界面文案与报告标签均经 `tr(key, lang)` 取词；未知 key 回退英文，取词过程绝不 panic。仓库附带的 `config.toml` 中已注明可接受的取值。

---

## 风险提示与免责声明

`wbot` 仅用于**学习与技术研究**，所有交易均为**模拟**，不涉及任何真实资金与券商接口——除非你显式开启加密货币 `live_trading = true`（见[加密货币支持](#加密货币okx-支持)），该模式会在你自担风险的前提下连接你自己的 OKX 账户。策略信号与回测结果基于历史数据，存在模型偏差、过拟合与未来不确定性，**不构成任何投资建议**。使用者须自行承担据此操作的一切风险。

---

## 开发指南

- **共享代码**：`src/lib.rs` 将全部模块暴露为 `wbot::`，二进制（`main.rs`）与 `examples/` 均可复用，避免重复实现回测 / 行情逻辑。
- **新增策略**：直接编辑 `strategy.toml` 即可，无需改代码；DSL 求值与形态状态机已支持常见指标与双金叉形态。
- **扩展回测**：核心在 `src/backtest.rs`（引擎 + Markdown 渲染，支持 i18n）与 `src/backtest_cli.rs`（异步数据拉取与编排，覆盖 A 股 / 美股 / 加密货币），二者均被二进制子命令与 `examples/backtest_all.rs` 共用。
- **新增市场**：实现 `MarketSource` trait（`src/market/source.rs`）并在 `MarketRouter`（`src/market/router.rs`）中登记；引擎其余部分只消费 `Candle`，无需改动。
- **扩展指标**：在 `src/indicators/` 下实现 `Indicator` trait，并在 `build_indicator`（`src/indicators.rs`）中登记新的 `kind`，即可被任意 DSL 表达式引用。
- **运行测试**：

  ```bash
  cargo test
  ```

- **构建示例**：

  ```bash
  cargo build --example backtest_all
  ```

- **重新生成 TA-Lib 参考文档**：`cargo run --example ta_indicators_list` 会依据 TA-Lib 运行时的元信息（函数列表、参数表、中英文含义、DSL 示例）写出 `docs/ta-lib-indicators.bilingual.md`。

---

> 文档语言：简体中文 ｜ 数据来源：`akshare`（A 股行情）· Yahoo Finance（美股行情）· `adaq-trading-crypto`（OKX 加密货币现货）｜ 指标：`adaq-talib`（TA-Lib 0.7.1，161 个函数）｜ 许可证见 `LICENSE`。
