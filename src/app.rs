//! Application state shared between the event loop and the renderer.

use std::collections::HashMap;
use std::time::Instant;

use crate::config::AppConfig;
use crate::indicators::Candle;
use crate::market::MarketData;
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
        }
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
            let (series, hold) = if let Some(tf) = &rule.timeframe {
                match self
                    .intraday_klines
                    .get(&format!("{}@{}", code, tf))
                    .filter(|s| s.len() >= 3)
                {
                    Some(s) => (s, 5usize),
                    None => continue,
                }
            } else {
                match self.klines.get(&code).filter(|s| s.len() >= 3) {
                    Some(s) => (s, 10usize),
                    None => continue,
                }
            };
            let res = crate::backtest::backtest_rule(rule, series, self.config.commission, hold);
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
