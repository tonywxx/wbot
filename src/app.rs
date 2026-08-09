//! Application state shared between the event loop and the renderer.

use crate::market::MarketData;
use std::time::Instant;

/// Which mover panel (gainers / losers) is currently focused for scrolling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Gainers,
    Losers,
}

/// In-memory UI state.
pub struct App {
    pub data: Option<MarketData>,
    pub status: String,
    pub last_update: Option<Instant>,
    pub watchlist: Vec<String>,
    pub scroll_gainers: u16,
    pub scroll_losers: u16,
    pub focus: Focus,
    pub refresh: u64,
}

impl App {
    pub fn new(watchlist: Vec<String>, refresh: u64) -> App {
        App {
            data: None,
            status: "加载中…".to_string(),
            last_update: None,
            watchlist,
            scroll_gainers: 0,
            scroll_losers: 0,
            focus: Focus::Gainers,
            refresh,
        }
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
