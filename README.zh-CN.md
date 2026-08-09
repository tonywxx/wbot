# wbot

> 其他语言：[English](README.md)

> 一个基于真实行情数据（A 股 + 美股）的**模拟交易终端（TUI）**，内置指标计算、信号引擎、形态识别、模拟下单与**策略回测报告生成**。

`wbot` 是一个 Rust 编写的命令行交易助手，使用 [`akshare-rs`](https://github.com/Cricle/akshare-rs) 拉取 A 股与指数实时/历史行情，并使用 [`yfinance-rs`](https://github.com/gramistella/yfinance-rs)（Yahoo Finance）接入美股。在终端里你可以全键盘浏览行情、查看技术指标、跟踪策略信号、进行无风险的模拟交易，并能对 `strategy.toml` 中的每一条策略批量回测、自动生成可读性强的 Markdown 回测报告。

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
  - [自选股 watchlist.txt](#自选股-watchlisttxt)
  - [策略文件 strategy.toml](#策略文件-strategytoml)
  - [费率与参数](#费率与参数)
- [回测报告说明](#回测报告说明)
- [模拟交易说明](#模拟交易说明)
- [美股支持](#美股支持)
- [风险提示与免责声明](#风险提示与免责声明)
- [开发指南](#开发指南)

---

## 功能特性总览

所有引擎（指标、信号、回测、模拟交易）都运行在「与市场无关」的 `Candle` 数据流之上，因此**下列每一项功能对 A 股与美股同时生效** —— 区别仅在于数据源。

### 1. 实时行情终端（`行情` 视图）
- 实时**指数条**（如上证指数、深证成指、创业板指）含最新价与涨跌幅。
- **市场广度面板**：全市场上涨 / 下跌 / 平盘 / 涨停 / 跌停家数统计，以及总数。
- **自选股表**：所跟踪标的的代码、名称、最新价、涨跌幅。
- **涨幅榜 / 跌幅榜**：各取前 30 名，`Tab` 切换焦点。
- 快照每 **5 秒**刷新；日线增量约每 60 秒推送一次；分钟 K 线每 `intraday_refresh` 秒（默认 120 秒）刷新。

### 2. 技术指标（`指标` 视图）
对选中个股实时计算并展示指标数值：
- **均线**：MA5 / MA10 / MA20。
- **RSI(14)**。
- **MACD**：DIF、DEA、HIST（红绿着色）。
- **多空排列**提示（`MA5 > MA10` → 短期多头）。
- 按 `↑` / `↓` 在自选股间切换，逐一查看各标的指标。

### 3. 信号引擎（DSL）
- 基于 `strategy.toml` 的 DSL 表达式逐标的求值。
- **沿触发**（仅当条件由假变真时触发），避免重复计数。
- 新信号触发**桌面通知**（同一「标的, 规则」带冷却去重）。

### 4. 形态识别
- 支持 `double_golden` —— 双金叉 / 双死叉回踩的**顺序状态机**，用于 15 分钟、60 分钟等周期（需连续判断，点状 DSL 无法表达）。

### 5. 模拟交易（`账户` 视图）
- **虚拟账户**，初始资金 **¥1,000,000**（A 股）/ 等值美元（美股）。
- **市价单**：以最新价买入/卖出，向下取整到整手（`lot_size`，默认 100）。
- **费用模型**：双边佣金 + 单边印花税，实时扣减。
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

### 8. 多周期支持
日线 DSL 策略与带 `timeframe` 的分钟（T+0）策略分别使用对应周期 K 线求值，互不混淆。

### 9. 通知与持久化
- 桌面通知：macOS 通过 `osascript`，其它平台降级为 stderr 输出。
- 安全持久化：文件缺失/损坏均有兜底，程序不会 panic。

---

## 架构设计

```
                ┌─────────── data_loop (tokio 异步) ───────────┐
   akshare ────►  fetch_market()  ──► 快照(指数+个股, 每 5s)     │
   yfinance ──►  fetch_klines()  ──► 日线 K 线(约每 60s)         │
                 fetch_intraday()──► 分钟 K 线(每 intraday_refresh)│
                └─────────────────────┬─────────────────────────┘
                                       │ Msg<快照/K线/分钟K线>
                ┌──────────────────────▼────────────────────────┐
   run_app() ──► apply_snapshot → eval_signals(引擎) → 通知+重算 │
                 handle_enter → account.place_order → 持久化(json)│
                └────────────────────────────────────────────────┘

   回测路径(独立 CLI):  backtest_cli → 拉取 K 线 → backtest::
   write_strategy_reports → "<id> 策略回测报告.md"
```

信号 / 指标 / 回测 / 模拟交易引擎均「与市场无关」，只消费 `Candle` 序列。标的按代码形态分流 —— 6 位数字视为 A 股，其余（如 `AAPL`、`BRK-B`）视为美股，因此同一套策略可在两个市场运行。

---

## 技术栈

- **语言**：Rust（Edition 2024）
- **TUI**：[`ratatui`](https://ratatui.rs) 0.30 + `crossterm` 0.29（交叉平台终端控制）
- **异步**：`tokio` 1.48（多线程运行时）
- **行情数据**：`akshare-rs`（`equity` feature，A 股与指数）；`yfinance-rs` 0.9（Yahoo Finance，美股）
- **配置 / 序列化**：`serde` + `toml` + `serde_json`
- **时间**：`chrono`；**错误**：`anyhow`；**数值**：`num-traits`

---

## 项目结构

```
wbot/
├── Cargo.toml                 # 库 + 二进制 + examples 定义
├── src/
│   ├── lib.rs                 # 暴露所有模块为 `wbot::`，供二进制与 examples 共享
│   ├── main.rs                # 二进制入口：TUI 主循环 + `backtest` 子命令
│   ├── app.rs                 # 应用状态（视图、焦点、账户、信号求值、回测）
│   ├── market.rs              # 行情/K 线拉取（akshare A 股 + yfinance 美股）、广度、自选股
│   ├── indicators.rs          # Candle、PriceSource、Indicator trait、IndicatorRegistry、build_indicator
│   ├── indicators/            # ma、macd、rsi、kdj、boll 实现
│   ├── signals.rs             # StrategyRule、RawRule、parse_strategy_file、Scope/Side 枚举
│   ├── signals/               # dsl（递归解析器）、eval（信号引擎）、double_cross（形态状态机）
│   ├── sim.rs                 # 模拟交易模块根
│   ├── sim/                   # account.rs（Account/Position/Order）、history.rs（Trade 持久化）
│   ├── config.rs              # AppConfig 默认值 + load_config()
│   ├── persist.rs             # account.json / trades.json 读写
│   ├── notify.rs              # 桌面通知器（冷却 + 去重）
│   ├── backtest.rs            # 回测引擎 + Markdown 报告渲染
│   ├── backtest_cli.rs        # 异步报告生成（A 股与美股），二进制与 example 共用
│   ├── ui.rs                  # ratatui 渲染分派 + 标签/头部/指数/页脚
│   ├── ui/                    # market_view、indicator_view、signal_view、account_view、strategy_view
│   └── tests.rs               # 单元测试
├── examples/
│   └── backtest_all.rs        # 复用 wbot::backtest_cli 的回测示例
├── strategy.toml              # 策略定义（用户可自由编辑；内置 39 条）
├── watchlist.txt              # A 股自选股列表
├── watchlist_us.txt           # 美股自选股列表（可选；存在时 TUI 启用美股标的）
├── reports/                   # A 股回测报告输出目录（自动生成）
└── reports_us/                # 美股回测报告输出目录（自动生成）
```

---

## 安装与构建

### 环境要求

- 已安装 [Rust 工具链](https://rustup.rs/)（建议使用较新版本以支持 Edition 2024）。
- 可访问 `akshare` 数据源（拉取 A 股行情）和/或 Yahoo Finance（拉取美股行情）的网络环境。

### 构建

```bash
# 克隆或进入项目目录后
cargo build --release      # 编译发布版本（较慢但运行更快）
cargo build                # 编译调试版本
```

> 首次构建会拉取并编译 `ratatui`、`tokio`、`akshare`/`reqwest`、`yfinance-rs`/`polars` 等依赖，耗时较长，请耐心等待。

---

## 使用方法

### 1. 启动 TUI 终端

```bash
cargo run                 # 调试模式启动
# 或
cargo run --release       # 发布模式启动
```

启动后会自动：

1. 加载 `watchlist.txt`（及若存在时的 `watchlist_us.txt`）与 `strategy.toml`；
2. 拉取历史日线 / 分钟 K 线完成初始化；
3. 进入全屏 TUI，开始实时推送行情并求值信号。

退出：按 `q` 或 `Esc`。

### 2. TUI 视图与快捷键

切换视图（数字键 `1`–`5`）：

| 按键 | 视图 | 内容 |
| --- | --- | --- |
| `1` | 行情 Market | 指数条 + 市场广度 + 自选股 + 涨/跌幅榜（Tab 切换焦点） |
| `2` | 指标 Indicators | 选中个股的 MA5/MA10/MA20、RSI14、MACD（`↑`/`↓` 切换标的） |
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
| `Enter` | 信号 / 账户视图内按最新价市价下单 |
| `q` / `Esc` | 退出程序 |

> 信号触发后会通过系统桌面通知提示（可在配置中关闭）。下单为**模拟成交**，仅写入本地 `trades.json` / `account.json`，不涉及真实资金。

### 3. 生成策略回测报告

提供两种等价方式，对 `strategy.toml` 中的**每一条策略**跑回测并生成独立 Markdown 报告：

**方式 A — 二进制子命令**（推荐）：

```bash
cargo run -- backtest reports
# 等价于：wbot backtest <输出目录>，目录默认 "reports"
```

**方式 B — examples 示例**：

```bash
cargo run --example backtest_all -- reports
# 目录参数可选，默认 "reports"
```

运行后会输出类似：

```
已生成 39 份策略回测报告 -> reports
  - ma_golden : reports/ma_golden 策略回测报告.md
  - s01_ma_bull_arr : reports/s01_ma_bull_arr 策略回测报告.md
  ...
```

每一条策略生成一份 `<id> 策略回测报告.md`，覆盖日线 DSL、经典策略、T+0 日内策略与形态策略。

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
cargo run -- backtest reports            # A 股 -> ./reports
cargo run -- backtest us reports_us      # 美股 -> ./reports_us
# 然后打开任意报告，例如：
open "reports/ma_golden 策略回测报告.md"   # macOS
```

**c) 在 TUI 中模拟一笔交易：**
1. 按 `1` → 行情，挑选一只标的；
2. 按 `3` → 信号，若高亮某条买入信号，按 `Enter` 以最新价买入一手；
3. 按 `4` → 账户，查看总资产、盈亏、持仓与成交记录。

**d) 在 TUI 中启用美股：** 创建 `watchlist_us.txt`（每行一个 ticker）；TUI 随即合并两份列表，对美股标的的信号求值与 A 股完全一致。

**e) 快速查看某策略的实时胜率：** 按 `5` → 策略，移动光标到任意策略，在明细面板即可读到当前选中个股的胜率与触发次数。

---

## 配置说明

### 自选股 watchlist.txt

每行一个 6 位 A 股代码，`#` 开头为注释，可附加中文备注：

```
# A股自选股列表 (watchlist)
600519   # 贵州茅台
601318   # 中国平安
600036   # 招商银行
...
```

- 留空或删除本文件则使用内置的 10 只流动性 A 股默认列表。
- 回测与信号求值均基于该列表内的标的（存在时再加上 `watchlist_us.txt`）。

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
timeframe = "5"        # 分钟周期：1/5/15/30/60
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

### 费率与参数

当前费率与 K 线参数通过 `src/config.rs` 的 `AppConfig::default()` 提供（后续可改为读取 `config.toml`）：

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `commission` | `0.0003` | 双边手续费率（万三） |
| `stamp_tax` | `0.0005` | 印花税（仅卖出，万五） |
| `lot_size` | `100` | 每手股数 |
| `auto_trade` | `false` | 自动下单开关（默认仅手动） |
| `kline_adjust` | `"qfq"` | K 线复权方式（前复权） |
| `kline_count` | `250` | 日线保留根数 |
| `intraday_refresh` | `120` | 分钟 K 线刷新间隔（秒） |
| `notify_enabled` | `true` | 是否发送桌面通知 |
| `notify_cooldown` | `300` | 同一（标的, 规则）通知冷却（秒） |

> 模拟账户初始资金为 **¥1,000,000**；美股回测/交易以美元计价。

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

> 报告中的「胜率」由回测引擎在所选个股上实测，并非主观断言；TUI 策略视图也会实时展示当前标的的回测胜率。

---

## 模拟交易说明

- 启动后在「账户 Account」视图按 `Enter` 可对当前选中标的以**最新价市价买入**；在「信号 Signals」视图选中某条信号后按 `Enter` 按其方向（买 / 卖）下单。
- 每笔下单按 `lot_size`（默认 100 股）整手成交，手续费与印花税按 `config` 参数实时扣减；现金不足拒买、持仓不足拒卖。
- 成交记录追加写入 `trades.json`，账户快照写入 `account.json`；重新启动会自动加载，便于连续跟踪模拟持仓。

---

## 美股支持

`wbot` 并非只服务于 A 股：所有引擎（指标、信号求值、回测、模拟交易）都运行在「与市场无关」的 `Candle` 数据流之上，因此**同一套**策略与报告可直接用于美股，差别仅在于数据源 —— 按代码形态自动选择：

- **A 股代码**——6 位纯数字（如 `600519`）→ 经 `akshare` 拉取。
- **美股代码**——其余形式（如 `AAPL`、`BRK-B`）→ 经 `yfinance-rs`（Yahoo Finance）拉取。注意伯克希尔哈撒韦在 Yahoo 中写作 `BRK-B`（连字符），而非 `BRK.B`。

### 美股自选股

在与 `watchlist.txt` 同级目录创建 `watchlist_us.txt`（每行一个 ticker，`#` 注释，格式同 A 股列表）。文件缺失时使用内置的默认美股名单。TUI 仅在 `watchlist_us.txt` 存在时才合并两份列表，因此默认 TUI 仍只含 A 股，待你显式开启美股。

### 回测美股

提供两种等价方式，对 `strategy.toml` 中的**每一条策略**跑美股回测，并逐策略输出独立 Markdown 报告：

**方式 A — 二进制子命令**（推荐）：

```bash
cargo run -- backtest us                # 美股回测，写入 ./reports_us
cargo run -- backtest us my_us_reports  # 自定义输出目录
```

**方式 B — examples 示例**：

```bash
cargo run --example backtest_all us     # 写入 ./reports_us
```

报告落在 `reports_us/`（或你指定的目录），文件名 `<id> 策略回测报告.md`，结构与 A 股报告一致（策略信息 + 跨标的汇总 + 分标的明细），并带 `市场：美股` 行。

### 注意事项

- **币种**：美股报价以 **USD（美元）** 计。TUI 与回测报告中的持仓市值、盈亏、收益率均以美元计价，而非人民币。
- **费用**：回测 / 模拟交易引擎的费用来自 `AppConfig`（`src/config.rs`）——双边 `commission` + 单边 `stamp_tax`（印花税）。印花税是 A 股特有概念，美股并不征收；纯美股回测时可将 `stamp_tax` 设为 `0`，以免高估卖出成本。`lot_size`（整手）默认 100 股对美股同样适用。
- **周期与历史**：美股日线取 Yahoo `range=1y`（约 252 根）；分钟线取 `range=1mo`，支持 `1 / 5 / 15 / 30 / 60` 分钟。与 A 股一致，策略的 `bars` 回看会截断为最近 N 根，因此 `bars=60` 的 5 分钟策略仅覆盖约 1 个交易日。
- **TUI 行为**：`watchlist_us.txt` 存在时，TUI 合并两份列表，对美股标的的信号求值与模拟交易与 A 股完全一致，美股行情以美元进入看板。
- **代码格式**：请按 Yahoo 的写法传入代码——伯克希尔哈撒韦写作 `BRK-B`（连字符）、`BF-B` 等；股份类别后缀用连字符，切勿用点号。

> **网络说明：** 美股数据来自 Yahoo Finance。在部分沙箱 / 机房网络中，Yahoo 会返回 HTTP `429`（限流 / 反爬），此时取不到美股 K 线，报告各标的为 `N/A`。请在可正常访问 Yahoo 的机器上运行以获得真实回测结果——代码路径完全一致。

---

## 风险提示与免责声明

`wbot` 仅用于**学习与技术研究**，所有交易均为**模拟**，不涉及任何真实资金与券商接口。策略信号与回测结果基于历史数据，存在模型偏差、过拟合与未来不确定性，**不构成任何投资建议**。使用者须自行承担据此操作的一切风险。

---

## 开发指南

- **共享代码**：`src/lib.rs` 将全部模块暴露为 `wbot::`，二进制（`main.rs`）与 `examples/` 均可复用，避免重复实现回测 / 行情逻辑。
- **新增策略**：直接编辑 `strategy.toml` 即可，无需改代码；DSL 求值与形态状态机已支持常见指标与双金叉形态。
- **扩展回测**：核心在 `src/backtest.rs`（引擎 + Markdown 渲染）与 `src/backtest_cli.rs`（异步数据拉取与编排），二者均被二进制子命令与 `examples/backtest_all.rs` 共用。
- **扩展指标**：在 `src/indicators/` 下实现 `Indicator` trait，并在 `build_indicator`（`src/indicators.rs`）中登记新的 `kind`，即可被任意 DSL 表达式引用。
- **运行测试**：

  ```bash
  cargo test
  ```

- **构建示例**：

  ```bash
  cargo build --example backtest_all
  ```

---

> 文档语言：简体中文 ｜ 数据来源：`akshare`（A 股行情）与 Yahoo Finance（美股行情）｜ 许可证见 `LICENSE`。
