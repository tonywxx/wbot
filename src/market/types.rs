//! Market types and derived metrics (market classification, quotes, spot / index
//! snapshots, breadth, movers).

// ===========================================================================
// Market classification
// ===========================================================================

/// 市场分类：A 股 / 美股 / 加密货币（OKX）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    A,
    Us,
    Crypto,
}

/// 根据代码形态判断市场：
/// - 6 位纯数字 → A 股；
/// - 含连字符（如 `BTC-USDT`、`ETH-USDT`）→ 加密货币（OKX 现货）；
/// - 其余（如 `AAPL`、`BRK.B`）→ 美股。
pub fn market_of(symbol: &str) -> Market {
    if symbol.chars().count() == 6 && symbol.chars().all(|c| c.is_ascii_digit()) {
        Market::A
    } else if symbol.contains('-') {
        Market::Crypto
    } else {
        Market::Us
    }
}

/// 统一行情报价：覆盖 A 股 / 美股 / 加密货币三种市场。
///
/// 旧实现里「美股 / 加密货币」没有 A 股那种全市场盘口快照（`MarketData.spots`），
/// 导致 watchlist 表格（`find_spot(&d.spots, code)`）对这两类标的永远匹配不到，
/// 名称与最新价始终显示 `—`，定时刷新也无法更新它们。
///
/// 这里引入一个轻量、与 provider 无关的统一报价结构，由 `MarketRouter::fetch_all_quotes`
/// 按代码形态（`market_of`）逐市场拉取后合并返回；UI 表格与 `app.prices` 均以它为权威来源，
/// 从而让三类资产在刷新周期内都能正确更新 name / price / change%。
#[derive(Debug, Clone)]
pub struct Quote {
    pub code: String,
    /// 显示名（A 股为中文名；美股为 `shortName`/代码；加密货币为 `instId`，如 `BTC-USDT`）。
    pub name: String,
    /// 最新价（加密货币/美股为实时报价；A 股为盘口最新价）。
    pub latest_price: f64,
    /// 涨跌幅百分比（与 A 股 `Spot.change_pct` 同口径；缺失时为 0.0）。
    pub change_pct: f64,
    /// 所属市场（仅供 UI 着色 / 调试，不影响路由）。
    pub market: Market,
}

/// 行情获取失败的种类。
///
/// 此前 `fetch_klines` / `fetch_intraday` 把一切失败坍缩成空 `Vec<Candle>`，
/// 调用方（`fetch_all_klines` 用 `if !s.is_empty()` 直接丢弃）无法区分
/// 「网络抖动」与「该标的本就无数据」，错误被静默吞掉（ADR-0001 预留的
/// failure-surfacing seam）。升级为 `Result<_, SourceError>` 后，seam 处即可
/// 表达失败「种类」，让路由聚合层把失败代码与原因带回调用方。
#[derive(Debug, Clone, PartialEq)]
pub enum SourceError {
    /// 网络 / 请求层失败（reqwest 传输错误等）。携带可读消息。
    Network(String),
    /// 响应解析失败（结构不符、字段解析失败等）。
    Parse(String),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceError::Network(m) => write!(f, "网络错误: {}", m),
            SourceError::Parse(m) => write!(f, "解析错误: {}", m),
        }
    }
}

impl std::error::Error for SourceError {}

// ===========================================================================
// Spot / Index / Market snapshot
// ===========================================================================

/// 单只 A 股盘口（取自东方财富 `clist`，已与 akshare 类型解耦）。
///
/// 仅保留引擎真正消费的字段；字段名与旧 `akshare::SpotQuote` 一致，
/// 因此 `market_view.rs` / `Breadth` / `top_gainers` 等消费方无需改动。
#[derive(Clone, Debug)]
pub struct Spot {
    pub code: String,
    pub name: String,
    pub latest_price: f64,
    pub change_pct: f64,
    pub change_amount: f64,
    pub volume: f64,
    pub amount: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub prev_close: f64,
}

/// 指数盘口。`latest_price` / `change_pct` 为 `Option`：东方财富对部分指数不返回
/// 实时价时缺省，UI 处跳过该指数而非崩溃。
#[derive(Clone, Debug)]
pub struct IndexSpot {
    pub code: String,
    pub name: String,
    pub latest_price: Option<f64>,
    pub change_pct: Option<f64>,
}

/// A full market snapshot returned by one refresh cycle.
#[derive(Clone)]
pub struct MarketData {
    pub indices: Vec<IndexSpot>,
    pub spots: Vec<Spot>,
}

// ===========================================================================
// Breadth + derived helpers
// ===========================================================================

/// Market breadth derived from the full spot board.
#[derive(Debug, Clone, Copy)]
pub struct Breadth {
    pub up: usize,
    pub down: usize,
    pub flat: usize,
    pub limit_up: usize,
    pub limit_down: usize,
    pub total: usize,
}

impl Breadth {
    pub fn compute(spots: &[Spot]) -> Breadth {
        let mut b = Breadth {
            up: 0,
            down: 0,
            flat: 0,
            limit_up: 0,
            limit_down: 0,
            total: spots.len(),
        };
        for s in spots {
            if s.latest_price <= 0.0 {
                continue;
            }
            if s.change_pct > 0.0 {
                b.up += 1;
            } else if s.change_pct < 0.0 {
                b.down += 1;
            } else {
                b.flat += 1;
            }
            // 涨停/跌停: 主板 ~10%, 创业板/科创板 ~20% — count >= 9.8% as a close approximation.
            if s.change_pct >= 9.8 {
                b.limit_up += 1;
            } else if s.change_pct <= -9.8 {
                b.limit_down += 1;
            }
        }
        b
    }
}

/// Top `n` gainers (descending change_pct).
pub fn top_gainers(spots: &[Spot], n: usize) -> Vec<Spot> {
    sorted(spots, true).into_iter().take(n).collect()
}

/// Top `n` losers (ascending change_pct).
pub fn top_losers(spots: &[Spot], n: usize) -> Vec<Spot> {
    sorted(spots, false).into_iter().take(n).collect()
}

fn sorted(spots: &[Spot], desc: bool) -> Vec<Spot> {
    let mut v: Vec<Spot> = spots
        .iter()
        .filter(|s| s.latest_price > 0.0)
        .cloned()
        .collect();
    v.sort_by(|a, b| {
        let ord = a
            .change_pct
            .partial_cmp(&b.change_pct)
            .unwrap_or(std::cmp::Ordering::Equal);
        if desc {
            ord.reverse()
        } else {
            ord
        }
    });
    v
}

/// Resolve a watchlist entry to its live spot quote (exact code match).
pub fn find_spot<'a>(spots: &'a [Spot], code: &str) -> Option<&'a Spot> {
    spots.iter().find(|s| s.code == code)
}
