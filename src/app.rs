//! Application state shared between the event loop and the renderer.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use crate::config::AppConfig;
use crate::sim::crypto_ledger::CryptoLedger;
use crate::indicators::Candle;
use crate::market::{Breadth, IndexSpot, Quote};
use crate::notify::Notifier;
use crate::backtest::BacktestResult;
use crate::signals::{Side, SignalEngine, SignalEvent, StrategyRule};
use crate::sim::account::Account;
use crate::sim::history::Trade;

/// 一条策略通知日志（noti 提示）：含触发时间、标的代码/名称、原因与买卖方向（用于着色）。
#[derive(Debug, Clone)]
pub struct StrategyLogEntry {
    pub ts: String,
    /// 标的代码（如 BTC-USDT）。
    pub code: String,
    /// 标的显示名（优先实时报价名，缺失回退代码）。
    pub name: String,
    /// 触发原因（备注或信号表达式）。
    pub reason: String,
    /// 判断所用周期（日线 / 15m 等），标明该信号依据什么周期触发。
    pub period: String,
    pub side: Side,
}

/// 价格变动标记：记录某标的「最近一次价格变动的方向与价差」，
/// 供行情面板持续显示（不淡出），让用户一眼看出该标的上一笔是涨还是跌、变动多少。
#[derive(Debug, Clone, Copy)]
pub struct PriceFlash {
    /// 方向：+1 涨、-1 跌（仅在发生变动时写入，故恒非 0）。
    pub dir: i8,
    /// 最近一次变动的价差（新价 − 旧价）；方向由 `dir` 决定符号。
    pub delta: f64,
}

/// 当前展示的视图。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Market,
    Indicators,
    Signals,
    Account,
    Strategies,
}

/// In-memory UI state.
pub struct App {
    // --- 行情看板（美股化） ---
    /// 指数栏：美股指数 + 黄金 + 原油（由 `fetch_us_indices` 填充）。
    pub indices: Vec<IndexSpot>,
    /// 美股市场广度（样本篮子涨跌家数），可选。
    pub breadth: Option<Breadth>,
    /// 统一实时报价表（A 股 / 美股 / 加密货币），键为代码。watchlist 表格与
    /// 最新价/涨跌幅均以它为权威来源，确保三类资产在刷新周期内都能更新。
    pub quotes: HashMap<String, Quote>,
    /// 各标的最新一次价格变动的闪烁标记（涨/跌 + 价差），键为代码；用于面板高亮。
    pub price_flash: HashMap<String, PriceFlash>,
    /// 各标的的近期价格序列（滚动窗口），键为代码；用于 watchlist 的迷你走势图。
    pub price_history: HashMap<String, VecDeque<f64>>,
    /// 自选股表格是否显示名称列（默认隐藏，按 `n` 切换）。
    pub show_name: bool,
    pub status: String,
    pub last_update: Option<Instant>,
    pub watchlist: Vec<String>,
    /// 策略通知日志（noti 提示），最新在前，可滚动。
    pub strategy_log: Vec<StrategyLogEntry>,
    /// 策略日志滚动偏移（已跳过的条目数）。
    pub log_scroll: u16,
    pub refresh: u64,

    // --- 模拟交易 ---
    pub active_view: View,
    pub selected_code: Option<String>,
    pub klines: HashMap<String, Vec<Candle>>,
    pub prices: HashMap<String, f64>,
    /// 分钟 K 线，键为 `"{code}@{timeframe}"`（如 "600519@15"）。
    pub intraday_klines: HashMap<String, Vec<Candle>>,
    pub signals: Vec<SignalEvent>,
    pub strategies: Vec<StrategyRule>,
    pub engine: SignalEngine,
    pub account: Account,
    pub trades: Vec<Trade>,
    pub config: AppConfig,
    /// 桌面通知器（带冷却去重）。
    pub notifier: Notifier,
    pub signal_cursor: usize,
    pub trade_cursor: usize,
    pub indicator_cursor: usize,
    /// 策略选择视图光标。
    pub strategy_cursor: usize,
    /// 当前选中个股的回测结果缓存（rule_id -> 结果）。
    pub backtests: HashMap<String, BacktestResult>,
    /// 模拟加密货币账户（USDT 现金 + 基础币持仓）。
    pub crypto: CryptoLedger,
    /// 帮助弹窗是否显示（按 `h` 切换）。默认 false。
    pub show_help: bool,
}

impl App {
    pub fn new(
        watchlist: Vec<String>,
        refresh: u64,
        klines: HashMap<String, Vec<Candle>>,
        account: Account,
        strategies: Vec<StrategyRule>,
        config: AppConfig,
    ) -> App {
        let engine = SignalEngine::new(strategies.clone());
        let notifier = Notifier::new(config.notify_enabled, config.notify_cooldown);
        App {
            indices: Vec::new(),
            breadth: None,
            quotes: HashMap::new(),
            price_flash: HashMap::new(),
            price_history: HashMap::new(),
            show_name: false,
            status: "加载中…".to_string(),
            last_update: None,
            watchlist: watchlist.clone(),
            strategy_log: Vec::new(),
            log_scroll: 0,
            refresh,
            active_view: View::Market,
            selected_code: watchlist.first().cloned(),
            klines,
            prices: HashMap::new(),
            intraday_klines: HashMap::new(),
            signals: Vec::new(),
            strategies,
            engine,
            account,
            trades: Vec::new(),
            config,
            notifier,
            signal_cursor: 0,
            trade_cursor: 0,
            indicator_cursor: 0,
            strategy_cursor: 0,
            backtests: HashMap::new(),
            crypto: CryptoLedger::new(100_000.0),
            show_help: false,
        }
    }

    /// 以最新价写入 `prices`（三市场实时价唯一权威源）。
    ///
    /// 不再「覆盖末根收盘」——那是候选 2 修复的污染源：`select_rule_series` 直接借用
    /// `klines`，覆盖末根会让实时价渗进回测末根。盘中近似改由 `SignalEngine::evaluate`
    /// 在求值入口对日线序列克隆后覆盖，回测读到的始终是纯历史。
    /// 快照（A 股盘口）与统一报价两条路径共用同一注入逻辑，避免重复。
    pub fn apply_last_price(&mut self, code: &str, price: f64) {
        self.prices.insert(code.to_string(), price);
    }

    /// 将最新价追加到该标的的滚动价格序列（用于迷你走势图），超出窗口上限则
    /// 丢弃最旧的一笔，始终保持固定长度的近期窗口。
    pub fn push_price_history(&mut self, code: &str, price: f64) {
        const WINDOW: usize = 48;
        let hist = self
            .price_history
            .entry(code.to_string())
            .or_insert_with(|| VecDeque::with_capacity(WINDOW));
        hist.push_back(price);
        if hist.len() > WINDOW {
            hist.pop_front();
        }
    }

    /// 在覆盖旧报价「前」调用：比对旧最新价与新价，若发生变化则记录闪烁标记
    /// （涨/跌 + 触发时刻），供行情面板短暂高亮。无变化（首笔或价格持平）不打标。
    pub fn record_price_change(&mut self, code: &str, new_price: f64) {
        let prev = self.quotes.get(code).map(|q| q.latest_price);
        let (dir, delta) = match prev {
            Some(p) if (new_price - p).abs() > f64::EPSILON => {
                if new_price > p {
                    (1, new_price - p)
                } else {
                    (-1, new_price - p)
                }
            }
            _ => (0, 0.0),
        };
        if dir != 0 {
            self.price_flash
                .insert(code.to_string(), PriceFlash { dir, delta });
        }
    }

    /// 当前视图内向下滚动 / 移动光标。
    pub fn scroll_down(&mut self) {
        match self.active_view {
            View::Market => {
                if !self.strategy_log.is_empty() {
                    let max = (self.strategy_log.len().saturating_sub(1)) as u16;
                    self.log_scroll = (self.log_scroll + 1).min(max);
                }
            }
            View::Signals => {
                if !self.signals.is_empty() {
                    self.signal_cursor = (self.signal_cursor + 1).min(self.signals.len() - 1);
                    self.selected_code = Some(self.signals[self.signal_cursor].code.clone());
                }
            }
            View::Account => {
                if !self.trades.is_empty() {
                    self.trade_cursor = (self.trade_cursor + 1).min(self.trades.len() - 1);
                }
            }
            View::Indicators => self.cycle_indicator(1),
            View::Strategies => {
                if !self.strategies.is_empty() {
                    self.strategy_cursor = (self.strategy_cursor + 1).min(self.strategies.len() - 1);
                }
            }
        }
    }

    /// 当前视图内向上滚动 / 移动光标。
    pub fn scroll_up(&mut self) {
        match self.active_view {
            View::Market => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            View::Signals => {
                if !self.signals.is_empty() {
                    self.signal_cursor = self.signal_cursor.saturating_sub(1);
                    self.selected_code = Some(self.signals[self.signal_cursor].code.clone());
                }
            }
            View::Account => {
                self.trade_cursor = self.trade_cursor.saturating_sub(1);
            }
            View::Indicators => self.cycle_indicator(-1),
            View::Strategies => {
                self.strategy_cursor = self.strategy_cursor.saturating_sub(1);
            }
        }
    }

    /// 切换当前光标处策略的启用/停用。
    pub fn toggle_strategy(&mut self) {
        if self.strategies.is_empty() {
            return;
        }
        let idx = self.strategy_cursor.min(self.strategies.len() - 1);
        let id = self.strategies[idx].id.clone();
        let next = !self.strategies[idx].enabled;
        self.strategies[idx].enabled = next;
        self.engine.set_enabled(&id, next);
    }

    /// 对当前选中个股重算所有策略的回测结果（写入 `backtests` 缓存）。
    pub fn recompute_backtests(&mut self) {
        let code = match &self.selected_code {
            Some(c) => c.clone(),
            None => self.watchlist.first().cloned().unwrap_or_default(),
        };
        if code.is_empty() {
            self.backtests.clear();
            return;
        }
        self.backtests.clear();
        for rule in &self.strategies {
            // 序列选取 / 最小长度 / 持仓门槛统一来自 series module（与实盘、回测报告一致）。
            let plan = match crate::series::select_rule_series(rule, &code, &self.klines, &self.intraday_klines) {
                Some(p) => p,
                None => continue,
            };
            let res = crate::backtest::backtest_rule(rule, plan.series, self.config.commission, plan.hold);
            self.backtests.insert(rule.id.clone(), res);
        }
    }

    fn cycle_indicator(&mut self, dir: isize) {
        if self.watchlist.is_empty() {
            return;
        }
        let n = self.watchlist.len() as isize;
        let cur = self.indicator_cursor as isize;
        let next = ((cur + dir) % n + n) % n;
        self.indicator_cursor = next as usize;
        self.selected_code = Some(self.watchlist[next as usize].clone());
    }
}
