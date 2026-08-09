# PRD：A 股模拟交易系统扩展（基于 wbot TUI）

> 文档角色：产品经理 PRD（简洁版）。面向现有 Rust TUI 行情程序 `/Users/tony/github/wbot`，将其从「行情看板」扩展为「可自定义策略的模拟交易系统」。本文档仅定义需求与数据模型，不含代码实现。

---

## 1. 产品目标

**一句话目标**：在现有 A 股实时行情 TUI 中，接入历史 K 线、计算可扩展的技术指标、允许用户以结构化规则自定义买卖信号，并实时模拟交易与展示账户表现。

关键目标（3-5 条）：
1. **真实数据驱动**：复用 akshare-rs 实时快照，并补齐历史 K 线，使指标与信号全部基于真实行情。
2. **指标引擎可扩展**：内置 MA/SMA/EMA、MACD、RSI，并以统一 `Indicator` 抽象预留 KDJ、BOLL 等扩展点。
3. **策略用户可编辑**：提供 TOML/JSON 形式的结构化信号规则，支持 cross_above/cross_below/gt/lt/and/or 等算子自由组合，无需改代码即可新增策略。
4. **闭环模拟交易**：以最新价市价成交，完整记录现金、持仓、已实现/未实现盈亏与交易历史。
5. **一体化 TUI 看盘**：在终端内统一展示实时行情、指标状态、信号提示与模拟账户表现，支持视图/标签页切换。

---

## 2. 用户故事（5-8 条）

1. 作为一名交易者，我在 TUI 中**看到自选股与全市场实时行情**，以便**快速掌握盘面强弱**。
2. 作为一名交易者，我在 TUI 中**查看某只股票的 MA/MACD/RSI 指标状态与数值**，以便**判断其技术形态**。
3. 作为一名交易者，我在 TUI 中**编写并保存「MA5 上穿 MA20 且 RSI<30」这样的规则**，以便**让系统自动监控买入时机**。
4. 作为一名交易者，我在 TUI 中**收到实时信号高亮提示（买入/卖出）**，以便**及时决策而不漏看**。
5. 作为一名交易者，我在 TUI 中**触发模拟买入/卖出（以最新价市价成交）**，以便**验证策略而不承担真实风险**。
6. 作为一名交易者，我在 TUI 中**查看模拟账户的现金、持仓、浮动盈亏与总资产**，以便**评估策略收益**。
7. 作为一名交易者，我在 TUI 中**回看交易历史明细**，以便**复盘每笔成交的成本与盈亏**。
8. 作为一名交易者，我**关闭程序后重新打开，策略与账户状态仍在**，以便**连续进行多日模拟**。

---

## 3. 需求池（P0 / P1 / P2）

### 能力域 A：真实行情数据源（实时快照 + 历史 K 线）
- **P0**
  - 保留现有实时快照（`stock_zh_a_spot_em` 指数 + 个股），维持 5s 轮询刷新。
  - 新增历史 K 线接入：对每个被监控标的拉取历史 K 线（默认日线，复权=qfq，默认 250 根用于指标预热）。akshare-rs 已支持 `a_share_candles(code, adjust, count)` / `stock_zh_a_hist`，需确认其 feature 开关并在 `Cargo.toml` 启用。
  - K 线**用途明确**：仅作为指标计算的输入（不直接展示完整 K 线图，P2 再考虑图表）。
- **P1**：多周期 K 线（日/周/60 分钟）按需拉取；K 线缓存更新（收盘后增量刷新，盘中仅刷新当日）。
- **P2**：本地 K 线缓存落盘，减少重复网络请求；非交易时段用缓存回放。

### 能力域 B：技术指标计算（可扩展引擎）
- **P0**
  - 内置核心指标：`MA`/`SMA`/`EMA`（任意周期、任意价格源 close/open/high/low）、`MACD`（DIF/DEA/MACD 柱，默认 12/26/9）、`RSI`（默认 14）。
  - **可扩展设计**：定义统一 trait（如 `Indicator { fn eval(&self, klines) -> Series }`），指标以「名称 + 参数」注册到指标表（registry），新增指标只需实现 trait 并登记。
  - 输出为带时间戳的数值序列，供条件引擎按「当前值 / 上一根值」取用（cross 类算子需前后两根）。
- **P1**：预留并给出 `KDJ`、`BOLL` 的接口与示例实现（默认参数 KDJ 9/3/3、BOLL 20/2）。
- **P2**：指标参数在规则内就近声明（无需全局配置）；多数据源指标（如成交量均量）。

### 能力域 C：自定义指标条件组合触发买卖信号
- **P0**
  - 提供**结构化规则文件**（默认 `strategy.toml`，备选 JSON），用户可编辑、热加载。
  - 规则支持递归条件树，原子算子：`gt/lt/gte/lte/eq`、`cross_above/cross_below`；逻辑算子：`and/or/not`。
  - 每条规则含：`id`、`label`、`side`（buy/sell）、`scope`（watchlist 或指定代码列表）、`enabled`、`signal`（条件树）。
  - 引擎在每次行情刷新后对 `scope` 内标的逐条求值，命中即产生信号事件（带时间戳、标的、规则 id、触发原因）。
  - 支持同标的「买入信号」与「卖出信号」独立规则；避免同一规则在连续刷新中重复触发（去抖：信号沿触发或最小间隔）。
- **P1**：规则作用域支持通配/分组；提供规则启停开关与命中计数统计。
- **P2**：规则模板市场（内置金叉/死叉/背离等模板）；信号回测（对历史 K 线重放统计胜率）。

### 能力域 D：模拟交易执行
- **P0**
  - 账户状态：现金（`cash`）、持仓表（`positions`：代码、数量、持仓均价、方向默认多头）、总资产。
  - 成交方式：**市价单，以最新价成交**（买入按 `latest_price`，卖出同）；含基础手续费（默认 0.03%，可配置；印花税卖出 0.05% 可配置）。
  - 盈亏计算：未实现盈亏 = Σ(最新价 − 持仓均价) × 数量；已实现盈亏在平仓/减仓时按均价差结算并记录。
  - 交易历史：每笔成交记录（时间、代码、买卖、价格、数量、手续费、该笔已实现盈亏、现金变动）。
  - 基础风控：现金不足拒买、持仓不足拒卖；整手（100 股）约束可配置。
- **P1**：仓位比例下单（按现金百分比）；止损/止盈自动规则（信号或阈值触发）。
- **P2**：做空/融资模拟；多账户；滑点与成交概率模型。

### 能力域 E：TUI 展示
- **P0**
  - 保留并增强现有「实时行情」视图：指数条、市场广度、自选股（附涨跌幅）、涨跌榜。
  - 新增「信号」视图：列出当前/近期触发信号（时间、标的、方向、触发规则与原因），高亮未处理信号。
  - 新增「账户」视图：现金、总资产、持仓表（数量/均价/最新/浮动盈亏）、交易历史表（分页滚动）。
  - 新增「指标」视图（或行情视图内嵌面板）：展示选中标的的 MA/MACD/RSI 等数值与状态（如「RSI=28 超卖」「MA5↑MA20 金叉」）。
  - **视图/标签页切换**：新增快捷键（如 `1/2/3/4` 或方向键切 Tab），复用现有 `Focus`/键盘事件机制。
- **P1**：信号可直接在 TUI 内一键「模拟买入/卖出」确认成交。
- **P2**：简单 ASCII/区块 K 线或指标迷你走势图；信号声音/颜色脉冲提醒。

### 能力域 F：配置与持久化
- **P0**
  - 规则持久化：`strategy.toml`（信号条件，用户编辑）。
  - 账户状态持久化：`account.json`（现金 + 持仓 + 初始资金），启动时加载、交易后写回。
  - 交易历史持久化：`trades.csv` 或 `trades.json`（追加写，启动可载入复盘）。
  - 失败兜底：文件缺失/损坏时使用默认值（初始资金假设值、空策略）并提示，不崩溃。
- **P1**：配置文件路径可在启动参数/环境变量指定；账户可「重置」。
- **P2**：配置版本号与迁移；导出导入策略包。

---

## 4. 信号条件组合模型设计

### 4.1 数据模型（TOML 伪结构）

```toml
# strategy.toml —— 用户自定义信号规则集

# 全局默认（可被单条规则覆盖）
default_scope = "watchlist"   # watchlist | 指定代码列表
commission = 0.0003           # 单边手续费
stamp_tax  = 0.0005           # 卖出印花税

# 单条规则（可多条）
[[strategy]]
id      = "golden_cross_oversold"
label   = "MA金叉 + RSI超卖"
side    = "buy"               # buy | sell
scope   = "watchlist"         # 省略则用 default_scope；或 ["600519","000858"]
enabled = true

# 条件树：递归结构
#   逻辑节点:  { op = "and"|"or"|"not", args = [ <node>, ... ] }
#   比较原子:  { op = "gt"|"lt"|"gte"|"lte"|"eq", left = <ind>, right = <ind|number> }
#   交叉原子:  { op = "cross_above"|"cross_below", left = <ind>, right = <ind> }
#   指标表达式: "NAME(source, params)"，如 MA(close,5) / RSI(close,14) / MACD(close,12,26,9).dif
[strategy.signal]
op = "and"
args = [
  { op = "cross_above", left = "MA(close,5)",  right = "MA(close,20)" },
  { op = "lt",          left = "RSI(close,14)", right = 30 },
]
```

指标表达式语法（抽象）：
`<IND>(<source>, <params...>)` ，其中
- `IND` ∈ { MA, SMA, EMA, MACD, RSI, KDJ, BOLL, PRICE }（可扩展）
- `source` ∈ { close, open, high, low, volume }
- 复合输出以 `.字段` 取子值，如 `MACD(close,12,26,9).dif` / `MACD(...).dea` / `KDJ(close,9,3,3).k` / `BOLL(close,20).mid`
- `PRICE(close)` 表示最新收盘价，便于与指标比较

### 4.2 示例

**示例 1 — 金叉 + 超卖（买入）**
```toml
[[strategy]]
id = "golden_cross_oversold"
label = "金叉且超卖"
side = "buy"
scope = "watchlist"
enabled = true
[strategy.signal]
op = "and"
args = [
  { op = "cross_above", left = "MA(close,5)",  right = "MA(close,20)" },
  { op = "lt",          left = "RSI(close,14)", right = 30 },
]
```

**示例 2 — 死叉 + 超买（卖出）**
```toml
[[strategy]]
id = "death_cross_overbought"
label = "死叉且超买"
side = "sell"
scope = "watchlist"
enabled = true
[strategy.signal]
op = "and"
args = [
  { op = "cross_below", left = "MA(close,5)",  right = "MA(close,20)" },
  { op = "gt",          left = "RSI(close,14)", right = 70 },
]
```

**示例 3 — MACD 金叉 + 价在均线上 + 非超买（买入，多条件）**
```toml
[[strategy]]
id = "macd_momentum"
label = "MACD金叉且站上MA20且RSI适中"
side = "buy"
scope = ["600519", "000858", "300750"]
enabled = true
[strategy.signal]
op = "and"
args = [
  { op = "cross_above", left = "MACD(close,12,26,9).dif", right = "MACD(close,12,26,9).dea" },
  { op = "gt",          left = "PRICE(close)",            right = "MA(close,20)" },
  { op = "lt",          left = "RSI(close,14)",           right = 50 },
]
```

**示例 4 — 预留扩展：KDJ 超买回落（卖出）**
```toml
[[strategy]]
id = "kdj_overbought"
label = "KDJ-K 下穿 80"
side = "sell"
scope = "watchlist"
enabled = false   # KDJ 为 P1 扩展，默认关闭
[strategy.signal]
op = "cross_below"
left  = "KDJ(close,9,3,3).k"
right = 80
```

---

## 5. 假设与待确认问题

### 5.1 默认假设（已采用，可在评审中调整）
- **初始资金**：默认 ¥1,000,000，存于 `account.json`。
- **K 线周期**：默认**日 K 线**，复权方式 `qfq`，默认拉取 **250 根**用于指标预热。
- **配置文件格式与路径**：规则 `strategy.toml`、账户 `account.json`、交易历史 `trades.csv`，均位于程序运行目录（与现有 `watchlist.txt` 同级）。
- **默认观测标的**：来自 `watchlist.txt`（缺失则回退内置默认列表），策略 `scope` 默认 `watchlist`。
- **交易方向**：仅做多（买入持有后卖出），不含做空/融资。
- **成交模型**：信号出现后**不自动下单**，需用户在 TUI 确认（或 P1 的自动规则）；成交价取实时 `latest_price`，含单边 0.03% 手续费、卖出 0.05% 印花税。
- **刷新节奏**：实时快照维持 5s；指标/信号在每次快照刷新后重算（日 K 场景下信号主要在收盘后变化，盘中信号以当日最新价近似）。
- **数据源**：继续复用 akshare-rs；需确认历史 K 线接口所属 feature 并在 `Cargo.toml` 启用（当前仅开 `equity`）。

### 5.2 待确认问题（需用户/团队拍板）
1. **是否允许信号自动成交**（无需人工确认）？还是仅提示、由用户手动模拟下单？这影响风控与账户视图设计。
2. **历史 K 线的 feature 与接口**：`a_share_candles` 是否属于 `equity` feature，还是需要新增 feature？是否需要升级 akshare-rs 版本（当前 `0.1`，crates.io 已有 `0.1.4` 含 `ta` 模块，是否直接复用其内置指标以加快进度）？
3. **小数点后/整手规则**：A 股 100 股整手，是否强制整手买入？碎股如何处理？
4. **指标引擎自研还是复用**：akshare-rs 自带 `ta` 模块（SMA/EMA/RSI/MACD，泛型于 OHLCV）。是自研可扩展引擎，还是封装其 `ta` 模块作为底层实现？（建议：自研薄封装层，底层可选复用 `ta`，保证可扩展与可控。）
5. **多周期/盘中信号语义**：日 K 下，"cross_above" 在盘中以最新价计算是否可接受（可能出现盘中假突破）？是否需要引入 60 分钟线降低噪音？
6. **是否需要 K 线图/指标走势可视化**（ASCII 迷你图），还是仅数值与状态文本？
7. **账户重置与多策略并行**：是否支持同时运行多条买卖规则作用于同一标的（可能产生冲突信号），以及是否需要「重置账户」入口？
8. **部署/运行环境**：是否在交易时段内常驻运行（影响刷新与网络稳定性要求），还是盘后复盘为主？

---

> 附：与现有架构的集成建议（供架构师参考，非需求）
> - 新增模块：`indicators.rs`（指标 trait+registry）、`signals.rs`（规则解析与求值）、`trading.rs`（账户/成交/盈亏）、`persist.rs`（配置与历史读写）。
> - `MarketData` 扩展：增加 `klines: HashMap<code, Vec<Candle>>` 缓存；新增 `Msg::Kline` / `Msg::Signal(Vec<SignalEvent>)` 通道消息。
> - `App` 状态扩展：`account`、`signals`、`indicator_cache`、`active_view`（标签页）。
> - TUI：在 `ui.rs` 增加 `render_signals / render_account / render_indicators` 与顶部 Tab 栏；键盘映射新增视图切换键。
