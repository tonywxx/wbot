# wbot

> 其他语言：[English](README.md)

> 一个基于真实 A 股行情数据的**模拟交易终端（TUI）**，内置指标计算、信号引擎、形态识别、模拟下单与**策略回测报告生成**。

`wbot` 是一个 Rust 编写的命令行交易助手，使用 [`akshare`](https://github.com/Cricle/akshare-rs) 拉取 A 股与指数实时/历史行情，在终端里以全键盘方式浏览行情、查看技术指标、跟踪策略信号、进行无风险的模拟交易，并能对 `strategy.toml` 中的每一条策略批量回测、自动生成可读性强的 Markdown 回测报告。

---

## 目录

- [功能特性](#功能特性)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [安装与构建](#安装与构建)
- [使用方法](#使用方法)
  - [1. 启动 TUI 终端](#1-启动-tui-终端)
  - [2. TUI 视图与快捷键](#2-tui-视图与快捷键)
  - [3. 生成策略回测报告](#3-生成策略回测报告)
- [配置说明](#配置说明)
  - [自选股 watchlist.txt](#自选股-watchlisttxt)
  - [策略文件 strategy.toml](#策略文件-strategytoml)
  - [费率与参数](#费率与参数)
- [回测报告说明](#回测报告说明)
- [模拟交易说明](#模拟交易说明)
- [风险提示与免责声明](#风险提示与免责声明)
- [开发指南](#开发指南)

---

## 功能特性

| 模块 | 说明 |
| --- | --- |
| **行情面板（Market）** | 实时指数与自选股涨跌榜（涨幅榜 / 跌幅榜可切换），每 5 秒刷新。 |
| **技术指标（Indicators）** | 对选中个股计算 MA / SMA / EMA / RSI / MACD / KDJ / BOLL 并随 K 线实时刷新。 |
| **信号引擎（Signals）** | 基于 `strategy.toml` 的 DSL 表达式逐标的求值，**边沿触发**（仅当条件由假变真时触发），避免重复计数；新信号触发桌面通知。 |
| **形态识别** | 支持 `double_golden`（双金叉 / 双死叉回踩）分钟级形态状态机，用于 15 分钟、60 分钟等周期。 |
| **模拟交易（Account）** | 虚拟账户、市价下单、手续费与印花税计算，成交记录持久化到 `trades.json`，账户状态保存到 `account.json`。 |
| **策略管理（Strategies）** | 浏览全部策略，查看实时回测胜率，空格快捷启用 / 停用。 |
| **回测引擎** | 在历史 K 线上重放每条策略信号，前向持有 N 根计算胜率、平均盈亏、盈亏比、总收益率、最大回撤。 |
| **回测报告生成** | 对每条策略输出独立的 `<id> 策略回测报告.md`：策略元数据 + 跨标的汇总表 + 分标的明细表。 |
| **多周期支持** | 日线 DSL 策略与带 `timeframe` 的分钟（T+0）策略分别使用对应周期 K 线求值，互不混淆。 |

---

## 技术栈

- **语言**：Rust（Edition 2024）
- **TUI**：[`ratatui`](https://ratatui.rs) + `crossterm`（交叉平台终端控制）
- **异步**：`tokio`（多线程运行时）
- **行情数据**：`akshare`（`equity` feature，覆盖 A 股与指数）
- **配置 / 序列化**：`serde` + `toml` + `serde_json`
- **时间**：`chrono`

---

## 项目结构

```
wbot/
├── Cargo.toml            # 库 + 二进制 + examples 定义
├── src/
│   ├── lib.rs            # 暴露所有模块，供二进制与 examples 共享
│   ├── main.rs           # 二进制入口：TUI 主循环 + `backtest` 子命令
│   ├── app.rs            # 应用状态机（视图、焦点、账户、信号求值）
│   ├── market.rs         # 行情拉取（指数/个股/日线/分钟 K 线）
│   ├── ui.rs             # ratatui 渲染
│   ├── indicators.rs     # 指标计算与注册表
│   ├── signals.rs        # DSL 解析与求值引擎、StrategyRule
│   ├── sim/              # 模拟交易（account.rs / history.rs）
│   ├── config.rs         # 费率、整手、K 线参数
│   ├── persist.rs        # 账户 / 成交记录持久化
│   ├── notify.rs         # 桌面通知
│   ├── backtest.rs       # 回测引擎 + Markdown 报告渲染
│   ├── backtest_cli.rs   # 异步报告生成入口（二进制与 example 共用）
│   └── tests.rs          # 单元测试
├── examples/
│   └── backtest_all.rs   # 复用 wbot::backtest_cli 的回测示例
├── strategy.toml         # 策略定义（用户可自由编辑）
├── watchlist.txt         # 自选股列表
└── reports/              # 回测报告输出目录（自动生成）
```

---

## 安装与构建

### 环境要求

- 已安装 [Rust 工具链](https://rustup.rs/)（建议使用较新版本以支持 Edition 2024）。
- 需要可访问 `akshare` 数据源的网络环境（拉取 A 股行情）。

### 构建

```bash
# 克隆或进入项目目录后
cargo build --release      # 编译发布版本（较慢但运行更快）
cargo build                # 编译调试版本
```

> 首次构建会拉取并编译 `ratatui`、`tokio`、`akshare`/`reqwest` 等依赖，耗时较长，请耐心等待。

---

## 使用方法

### 1. 启动 TUI 终端

```bash
cargo run                 # 调试模式启动
# 或
cargo run --release       # 发布模式启动
```

启动后会自动：

1. 加载 `watchlist.txt` 自选股与 `strategy.toml` 策略；
2. 拉取历史日线 / 分钟 K 线完成初始化；
3. 进入全屏 TUI，开始实时推送行情并求值信号。

退出：按 `q` 或 `Esc`。

### 2. TUI 视图与快捷键

切换视图（数字键 `1`–`5`）：

| 按键 | 视图 | 内容 |
| --- | --- | --- |
| `1` | 行情 Market | 指数 + 自选股涨跌榜（Tab 切换涨幅 / 跌幅榜） |
| `2` | 指标 Indicators | 选中个股的技术指标数值 |
| `3` | 信号 Signals | 当前触发的买卖信号列表（光标选中后按 Enter 下单） |
| `4` | 账户 Account | 虚拟账户资金、持仓、成交（按 Enter 对选中标的买入） |
| `5` | 策略 Strategies | 全部策略及其实时回测胜率（空格启用 / 停用） |

全局快捷键：

| 按键 | 作用 |
| --- | --- |
| `↑` / `k` | 上移光标 |
| `↓` / `j` | 下移光标 |
| `Tab` | 行情视图内切换「涨幅榜 / 跌幅榜」 |
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

- 留空或删除本文件则使用内置默认列表。
- 回测与信号求值均基于该列表内的标的。

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

DSL 支持：

- **逻辑**：`and(...)` `or(...)` `not(...)`
- **比较**：`gt(a,b)` `lt(a,b)` `gte(a,b)` `lte(a,b)` `eq(a,b)`
- **交叉**：`cross_above(a,b)`（上穿）`cross_below(a,b)`（下穿）
- **指标**：`MA(src,p)` `SMA(src,p)` `EMA(src,p)` `RSI(src,p)`
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
- 每笔下单按 `lot_size`（默认 100 股）整手成交，手续费与印花税按 `config` 参数实时扣减。
- 成交记录追加写入 `trades.json`，账户快照写入 `account.json`；重新启动会自动加载，便于连续跟踪模拟持仓。

---

## 风险提示与免责声明

`wbot` 仅用于**学习与技术研究**，所有交易均为**模拟**，不涉及任何真实资金与券商接口。策略信号与回测结果基于历史数据，存在模型偏差、过拟合与未来不确定性，**不构成任何投资建议**。使用者须自行承担据此操作的一切风险。

---

## 开发指南

- **共享代码**：`src/lib.rs` 将全部模块暴露为 `wbot::`，二进制（`main.rs`）与 `examples/` 均可复用，避免重复实现回测 / 行情逻辑。
- **新增策略**：直接编辑 `strategy.toml` 即可，无需改代码；DSL 求值与形态状态机已支持常见指标与双金叉形态。
- **扩展回测**：核心在 `src/backtest.rs`（引擎 + Markdown 渲染）与 `src/backtest_cli.rs`（异步数据拉取与编排），二者均被二进制子命令与 `examples/backtest_all.rs` 共用。
- **运行测试**：

  ```bash
  cargo test
  ```

- **构建示例**：

  ```bash
  cargo build --example backtest_all
  ```

---

> 文档语言：简体中文 ｜ 数据来源：`akshare`（A 股行情） ｜ 许可证见 `LICENSE`。
