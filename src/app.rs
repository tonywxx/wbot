//! Application state shared between the event loop and the renderer.

use std::collections::HashMap;
use std::time::Instant;

use crate::config::AppConfig;
use crate::sim::crypto_ledger::CryptoLedger;
use crate::indicators::Candle;
use crate::market::{MarketData, Quote};
use crate::notify::Notifier;
use crate::backtest::BacktestResult;
use crate::signals::{SignalEngine, SignalEvent, StrategyRule};
use crate::sim::account::Account;
use crate::sim::history::Trade;

/// Which mover panel (gainers / losers) is currently focused for scrolling (Market view).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Gainers,
    Losers,
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
    // --- 行情看板（保留） ---
    pub data: Option<MarketData>,
    /// 统一实时报价表（A 股 / 美股 / 加密货币），键为代码。watchlist 表格与
    /// 最新价/涨跌幅均以它为权威来源，确保三类资产在刷新周期内都能更新。
    pub quotes: HashMap<String, Quote>,
    pub status: String,
    pub last_update: Option<Instant>,
    pub watchlist: Vec<String>,
    pub scroll_gainers: u16,
    pub scroll_losers: u16,
    pub focus: Focus,
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
            data: None,
            quotes: HashMap::new(),
            status: "加载中…".to_string(),
            last_update: None,
            watchlist: watchlist.clone(),
            scroll_gainers: 0,
            scroll_losers: 0,
            focus: Focus::Gainers,
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

    /// 当前视图内向下滚动 / 移动光标。
    pub fn scroll_down(&mut self) {
        match self.active_view {
            View::Market => self.scroll_focused(1),
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
            View::Market => self.scroll_focused(-1),
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

    /// Scroll the focused mover panel by `delta` rows (clamped to >= 0).
    pub fn scroll_focused(&mut self, delta: i32) {
        let cur = match self.focus {
            Focus::Gainers => &mut self.scroll_gainers,
            Focus::Losers => &mut self.scroll_losers,
        };
        let next = (*cur as i32).saturating_add(delta);
        *cur = next.max(0) as u16;
    }
}
