//! Market data fetching & derived metrics.
//!
//! Two data sources, unified behind a `Candle` pipeline:
//! - **A-shares / indices** via `akshare` (`AkShareClient`).
//! - **US stocks** via `yfinance-rs` (`YfClient`, Yahoo Finance).
//!
//! Symbol routing is by shape: a 6-digit numeric code is treated as an A-share;
//! anything else (e.g. `AAPL`, `BRK.B`) is treated as a US ticker. The indicator,
//! signal, backtest and simulated-trading engines are all market-agnostic — they
//! only ever see `Candle` sequences — so "US support" is purely a data-source switch.

use akshare::stock::feature::SpotQuote;
use akshare::stock::zh_index::IndexSpotEm;
use akshare::AkShareClient;
use yfinance_rs::{Decimal, Interval, Range, Ticker, YfClient};

use crate::indicators::Candle;
use chrono::{NaiveDate, NaiveDateTime};
use num_traits::cast::ToPrimitive;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

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
    /// 涨跌幅百分比（与 A 股 `SpotQuote.change_pct` 同口径；缺失时为 0.0）。
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

/// 数据源抽象：把 A 股 / 美股两个 provider 收敛到同一异步接口之后。
///
/// 引擎层（指标 / 信号 / 回测 / 模拟交易）只消费 `Candle`，从不直接接触
/// `AkShareClient` / `YfClient`。两个真实适配器（`AkShareSource` / `YfSource`）
/// 各自封装自己的解析；`MarketRouter` 按代码形态（`market_of`）把请求派发给对应适配器，
/// 于是「两个真实适配器 + 一个 seam」，而非在每个调用点内联切换。
pub trait MarketSource: Send + Sync {
    /// 该数据源服务的市场。
    fn market(&self) -> Market;

    /// 单标的日线（或单周期）历史，映射为升序 `Candle`。
    ///
    /// 失败时返回 [`SourceError`]（`Network` / `Parse`），不再坍缩成空序列。
    fn fetch_klines<'a>(
        &'a self,
        code: &'a str,
        adjust: &'a str,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>;

    /// 单标的分钟 K 线（周期 `tf`，保留末 `bars` 根），映射为升序 `Candle`。
    ///
    /// 失败时返回 [`SourceError`]（`Network` / `Parse`），不再坍缩成空序列。
    fn fetch_intraday<'a>(
        &'a self,
        code: &'a str,
        tf: &'a str,
        bars: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>;

    /// 全市场盘口快照（指数 + 个股）。仅 A 股有对应数据；美股返回 `None`。
    fn fetch_snapshot<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Option<MarketData>> + Send + 'a>>;

    /// 拉取给定代码列表的实时报价（统一 [`Quote`]）。用于让 watchlist 表格在刷新
    /// 周期内能更新三类资产（A 股 / 美股 / 加密货币）的名称与最新价。
    /// 默认实现返回空向量（不提供实时报价的源可不加实现）。
    fn fetch_quotes<'a>(
        &'a self,
        _codes: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Vec<Quote>> + Send + 'a>> {
        Box::pin(async move { Vec::new() })
    }
}

/// A 股数据源：封装 `AkShareClient`。
pub struct AkShareSource {
    client: AkShareClient,
}

impl AkShareSource {
    pub fn new() -> Self {
        Self { client: AkShareClient::new() }
    }
}

impl MarketSource for AkShareSource {
    fn market(&self) -> Market {
        Market::A
    }

    fn fetch_klines<'a>(
        &'a self,
        code: &'a str,
        adjust: &'a str,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move { fetch_klines(client, code, adjust, count).await })
    }

    fn fetch_intraday<'a>(
        &'a self,
        code: &'a str,
        tf: &'a str,
        bars: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move { fetch_minute_klines(client, code, tf, bars).await })
    }

    fn fetch_snapshot<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Option<MarketData>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move { Some(fetch_market(client).await) })
    }

    fn fetch_quotes<'a>(
        &'a self,
        codes: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Vec<Quote>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move {
            // A 股实时报价来自东方财富全市场盘口；从快照中筛出自选股代码。
            let data = fetch_market(client).await;
            codes
                .iter()
                .filter_map(|code| {
                    data.spots.iter().find(|s| &s.code == code).map(|s| Quote {
                        code: s.code.clone(),
                        name: s.name.clone(),
                        latest_price: s.latest_price,
                        change_pct: s.change_pct,
                        market: Market::A,
                    })
                })
                .collect()
        })
    }
}

/// 美股数据源：封装 `YfClient`（Yahoo Finance）。
pub struct YfSource {
    client: YfClient,
}

impl YfSource {
    pub fn new() -> Self {
        Self { client: YfClient::default() }
    }
}

impl MarketSource for YfSource {
    fn market(&self) -> Market {
        Market::Us
    }

    fn fetch_klines<'a>(
        &'a self,
        code: &'a str,
        _adjust: &'a str,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move { fetch_klines_us(client, code, Range::Y1, Interval::D1, count).await })
    }

    fn fetch_intraday<'a>(
        &'a self,
        code: &'a str,
        tf: &'a str,
        bars: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>> {
        let client = &self.client;
        let interval = interval_from_tf(tf);
        Box::pin(async move { fetch_klines_us(client, code, Range::M1, interval, bars).await })
    }

    fn fetch_snapshot<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Option<MarketData>> + Send + 'a>> {
        Box::pin(async move { Option::<MarketData>::None })
    }

    fn fetch_quotes<'a>(
        &'a self,
        codes: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Vec<Quote>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move {
            if codes.is_empty() {
                return Vec::new();
            }
            // 美股无 A 股式全市场盘口，需逐个（或批量）向 Yahoo 拉取实时报价。
            // `Ticker::quote()` 返回 `Quote { price, name, previous_close, ... }`；
            // 涨跌幅由 (price - previous_close) / previous_close 反推（Yahoo 不直接给百分比）。
            let mut out = Vec::with_capacity(codes.len());
            for code in codes {
                let ticker = Ticker::new(client, code.clone());
                match ticker.quote().await {
                    Ok(q) => {
                        let price = q.price.map(|p| amount_to_f64(p.into_inner())).unwrap_or(0.0);
                        if price <= 0.0 {
                            continue;
                        }
                        let prev = q.previous_close.map(|p| amount_to_f64(p.into_inner())).unwrap_or(0.0);
                        let change_pct = if prev > 0.0 {
                            (price - prev) / prev * 100.0
                        } else {
                            0.0
                        };
                        out.push(Quote {
                            code: code.clone(),
                            name: q.name.clone().unwrap_or_else(|| code.clone()),
                            latest_price: price,
                            change_pct,
                            market: Market::Us,
                        });
                    }
                    Err(e) => {
                        eprintln!("美股报价获取失败 {}: {}", code, e);
                    }
                }
            }
            out
        })
    }
}

/// 加密货币数据源：封装 `OkxClient`（OKX V5，经 okx-rs）。
pub struct OkxSource {
    client: crate::crypto::OkxClient,
}

impl OkxSource {
    pub fn new() -> Self {
        Self {
            client: crate::crypto::OkxClient::new(),
        }
    }

    /// 分钟周期（akshare 形式 "1"/"5"/"15"/"30"/"60"）映射到 OKX `bar`。
    fn bar_from_tf(tf: &str) -> &'static str {
        match tf {
            "1" => "1m",
            "5" => "5m",
            "15" => "15m",
            "30" => "30m",
            "60" => "1H",
            _ => "1H",
        }
    }
}

impl MarketSource for OkxSource {
    fn market(&self) -> Market {
        Market::Crypto
    }

    fn fetch_klines<'a>(
        &'a self,
        code: &'a str,
        _adjust: &'a str,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>> {
        let client = &self.client;
        // 加密货币现货日线用 OKX "1D" 周期；复权对现货无意义，忽略 `adjust`。
        Box::pin(async move { client.fetch_candles(code, "1D", count).await })
    }

    fn fetch_intraday<'a>(
        &'a self,
        code: &'a str,
        tf: &'a str,
        bars: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>> {
        let client = &self.client;
        let bar = OkxSource::bar_from_tf(tf);
        Box::pin(async move { client.fetch_candles(code, bar, bars).await })
    }

    fn fetch_snapshot<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Option<MarketData>> + Send + 'a>> {
        // 加密货币无 A 股式全市场盘口快照，返回 `None`。
        Box::pin(async move { Option::<MarketData>::None })
    }

    fn fetch_quotes<'a>(
        &'a self,
        codes: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Vec<Quote>> + Send + 'a>> {
        let client = &self.client;
        Box::pin(async move {
            let mut out = Vec::with_capacity(codes.len());
            for code in codes {
                // 名称直接取交易对本身（如 `BTC-USDT`），UI 即可识别。
                if let Some((price, open24h)) = client.fetch_ticker_price(code).await {
                    if price > 0.0 {
                        let change_pct = if open24h > 0.0 {
                            (price - open24h) / open24h * 100.0
                        } else {
                            0.0
                        };
                        out.push(Quote {
                            code: code.clone(),
                            name: code.clone(),
                            latest_price: price,
                            change_pct,
                            market: Market::Crypto,
                        });
                    }
                }
            }
            out
        })
    }
}

/// 双数据源路由器：持有 A / 美股两个适配器，按代码形态派发。
///
/// 调用方只持有 `MarketRouter`（或 `&dyn MarketSource`），不再内联 `market_of` 切换，
/// 也不再同时持有两个具体 client。测试可用 [`MarketRouter::from_sources`] 注入假适配器。
pub struct MarketRouter {
    a: Box<dyn MarketSource>,
    us: Box<dyn MarketSource>,
    crypto: Box<dyn MarketSource>,
}

impl MarketRouter {
    /// 构造真实数据源（A 股走 akshare，美股走 yfinance，加密货币走 OKX）。
    pub fn new() -> Self {
        Self {
            a: Box::new(AkShareSource::new()),
            us: Box::new(YfSource::new()),
            crypto: Box::new(OkxSource::new()),
        }
    }

    /// 注入式构造（测试 / 替换数据源用）。加密货币默认用真实 `OkxSource`。
    pub fn from_sources(a: Box<dyn MarketSource>, us: Box<dyn MarketSource>) -> Self {
        Self {
            a,
            us,
            crypto: Box::new(OkxSource::new()),
        }
    }

    /// 全量注入式构造（含加密货币数据源）。
    pub fn from_sources_full(
        a: Box<dyn MarketSource>,
        us: Box<dyn MarketSource>,
        crypto: Box<dyn MarketSource>,
    ) -> Self {
        Self { a, us, crypto }
    }

    /// 按代码形态返回对应适配器。
    pub fn source_for(&self, code: &str) -> &dyn MarketSource {
        match market_of(code) {
            Market::A => self.a.as_ref(),
            Market::Us => self.us.as_ref(),
            Market::Crypto => self.crypto.as_ref(),
        }
    }

    /// 代码所属市场（符号形态路由，封装在此）。
    pub fn market_of_code(&self, code: &str) -> Market {
        market_of(code)
    }

    /// 批量日线路由：逐代码派发给对应适配器。
    ///
    /// 返回 `(成功序列表, 失败清单)`。best-effort 语义保留——成功部分照常入表，
    /// 失败代码与 [`SourceError`] 一并带回，调用方可据此归因 / 告警，而不再被
    /// `if !s.is_empty()` 静默丢弃（候选 3：失败浮出 interface）。
    pub async fn fetch_all_klines(
        &self,
        codes: &[String],
        adjust: &str,
        count: usize,
    ) -> (HashMap<String, Vec<Candle>>, Vec<(String, SourceError)>) {
        let mut map = HashMap::with_capacity(codes.len());
        let mut errs: Vec<(String, SourceError)> = Vec::new();
        for code in codes {
            match self.source_for(code).fetch_klines(code, adjust, count).await {
                Ok(s) => {
                    if !s.is_empty() {
                        map.insert(code.clone(), s);
                    }
                }
                Err(e) => errs.push((code.clone(), e)),
            }
        }
        (map, errs)
    }

    /// 批量分钟线路由：逐 (代码, 周期) 派发给对应适配器。
    ///
    /// 返回 `(成功序列表, 失败清单)`，失败键为复合键 `{code}@{tf}`。语义同 [`fetch_all_klines`]。
    pub async fn fetch_all_intraday(
        &self,
        codes: &[String],
        tf_bars: &[(String, usize)],
    ) -> (HashMap<String, Vec<Candle>>, Vec<(String, SourceError)>) {
        if tf_bars.is_empty() {
            return (HashMap::new(), Vec::new());
        }
        let mut map = HashMap::new();
        let mut errs: Vec<(String, SourceError)> = Vec::new();
        for (tf, bars) in tf_bars {
            for code in codes {
                let key = format!("{}@{}", code, tf);
                match self.source_for(code).fetch_intraday(code, tf, *bars).await {
                    Ok(s) => {
                        if !s.is_empty() {
                            map.insert(key, s);
                        }
                    }
                    Err(e) => errs.push((key, e)),
                }
            }
        }
        (map, errs)
    }

    /// A 股盘口快照（美股无对应数据，返回 `None`）。
    pub async fn fetch_snapshot(&self) -> Option<MarketData> {
        self.a.fetch_snapshot().await
    }

    /// 批量拉取 watchlist 中所有代码的实时报价（统一 [`Quote`]）。
    ///
    /// 按代码形态（`market_of`）把代码分流到 A / 美股 / 加密货币三个适配器，
    /// 各自并发/顺序拉取后合并返回。三类资产因此都能在刷新周期内更新
    /// 名称与最新价，而不再依赖 A 股专属的全市场盘口快照。
    pub async fn fetch_all_quotes(&self, codes: &[String]) -> Vec<Quote> {
        let mut a_codes: Vec<String> = Vec::new();
        let mut us_codes: Vec<String> = Vec::new();
        let mut crypto_codes: Vec<String> = Vec::new();
        for code in codes {
            match market_of(code) {
                Market::A => a_codes.push(code.clone()),
                Market::Us => us_codes.push(code.clone()),
                Market::Crypto => crypto_codes.push(code.clone()),
            }
        }
        let mut all = Vec::new();
        all.extend(self.a.fetch_quotes(&a_codes).await);
        all.extend(self.us.fetch_quotes(&us_codes).await);
        all.extend(self.crypto.fetch_quotes(&crypto_codes).await);
        all
    }
}

/// 将 yfinance 的 `PriceAmount` / `QuantityAmount` 转为 `f64`（不足时回退 0.0）。
fn amount_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

/// Default自选股 (watchlist) — bare 6-digit A-share codes, no market prefix.
pub const DEFAULT_WATCHLIST: &[&str] = &[
    "600519", // 贵州茅台
    "601318", // 中国平安
    "600036", // 招商银行
    "000858", // 五粮液
    "300750", // 宁德时代
    "601899", // 紫金矿业
    "600900", // 长江电力
    "000001", // 平安银行
    "002594", // 比亚迪
    "600276", // 恒瑞医药
];

/// Default US watchlist — liquid US tickers (Yahoo Finance symbols).
pub const DEFAULT_WATCHLIST_US: &[&str] = &[
    "AAPL", "MSFT", "NVDA", "GOOGL", "AMZN", "META", "TSLA", "BRK.B", "JPM", "V", "JNJ", "WMT",
    "MA", "PG", "HD", "XOM", "BAC", "KO", "PEP", "COST", "NFLX", "AMD", "ORCL", "CRM", "ADBE", "BRK-B",
];

/// Default crypto watchlist — liquid OKX spot pairs (USDT-quoted).
pub const DEFAULT_WATCHLIST_CRYPTO: &[&str] = &[
    "BTC-USDT", "ETH-USDT", "SOL-USDT", "BNB-USDT", "XRP-USDT", "DOGE-USDT",
    "ADA-USDT", "AVAX-USDT", "LINK-USDT", "MATIC-USDT",
];

/// A full market snapshot returned by one refresh cycle.
#[derive(Clone)]
pub struct MarketData {
    pub indices: Vec<IndexSpotEm>,
    pub spots: Vec<SpotQuote>,
}

/// Fetch indices + the full A-share spot board in parallel (best-effort:
/// a failure of one endpoint does not block the other).
pub async fn fetch_market(client: &AkShareClient) -> MarketData {
    let (indices, spots) = tokio::join!(
        client.stock_zh_index_spot_em(),
        client.stock_zh_a_spot_em(),
    );
    MarketData {
        indices: indices.unwrap_or_default(),
        spots: spots.unwrap_or_default(),
    }
}

/// Load the watchlist: prefer `watchlist.txt` in the cwd (one bare code per line),
/// otherwise fall back to [`DEFAULT_WATCHLIST`].
pub fn load_watchlist() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist.txt") {
        let parsed: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    DEFAULT_WATCHLIST.iter().map(|s| s.to_string()).collect()
}

/// 加载美股自选股：优先读取 cwd 下的 `watchlist_us.txt`（每行一个 ticker），
/// 否则回退到 [`DEFAULT_WATCHLIST_US`]。
pub fn load_watchlist_us() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist_us.txt") {
        let parsed: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    DEFAULT_WATCHLIST_US.iter().map(|s| s.to_string()).collect()
}

/// 加载加密货币自选股：优先读取 cwd 下的 `watchlist_crypto.txt`（每行一个
/// `BASE-USDT` 交易对），否则回退到 [`DEFAULT_WATCHLIST_CRYPTO`]。
pub fn load_watchlist_crypto() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist_crypto.txt") {
        let parsed: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    DEFAULT_WATCHLIST_CRYPTO.iter().map(|s| s.to_string()).collect()
}

/// 加载合并自选股：加密货币（若 `crypto_enabled`）→ 美股 → A 股。
/// 三类均默认载入内置清单（加密货币需 `crypto_enabled` 为真）；若对应
/// `watchlist_*.txt` 文件存在，则文件内容覆盖内置清单。如此加密货币成为
/// 默认优先加载的投资标的，其后依次为美股与 A 股（见 ADR 0002）。
pub fn load_watchlist_combined(crypto_enabled: bool) -> Vec<String> {
    let mut v = Vec::new();
    if crypto_enabled {
        v.extend(load_watchlist_crypto());
    }
    v.extend(load_watchlist_us());
    v.extend(load_watchlist());
    v
}

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
    pub fn compute(spots: &[SpotQuote]) -> Breadth {
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
pub fn top_gainers(spots: &[SpotQuote], n: usize) -> Vec<SpotQuote> {
    sorted(spots, true).into_iter().take(n).collect()
}

/// Top `n` losers (ascending change_pct).
pub fn top_losers(spots: &[SpotQuote], n: usize) -> Vec<SpotQuote> {
    sorted(spots, false).into_iter().take(n).collect()
}

fn sorted(spots: &[SpotQuote], desc: bool) -> Vec<SpotQuote> {
    let mut v: Vec<SpotQuote> = spots
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
pub fn find_spot<'a>(spots: &'a [SpotQuote], code: &str) -> Option<&'a SpotQuote> {
    spots.iter().find(|s| s.code == code)
}

/// 解析 K 线日期：兼容 "2024-01-02" 与 "20240102" 两种格式（日线，时间归零）。
fn parse_kline_date(s: &str) -> Option<NaiveDateTime> {
    let d = if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        d
    } else if let Ok(d) = NaiveDate::parse_from_str(s, "%Y%m%d") {
        d
    } else {
        return None;
    };
    d.and_hms_opt(0, 0, 0)
}

/// 解析分钟 K 线时间（Sina 格式 "2024-01-02 09:30:00"）。
fn parse_minute_datetime(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

/// 拉取单只标的的历史 K 线并映射为 `Candle`（按日期升序）。
pub async fn fetch_klines(
    client: &AkShareClient,
    code: &str,
    adjust: &str,
    count: usize,
) -> Result<Vec<Candle>, SourceError> {
    let points = match client.a_share_candles(code, adjust, count).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("K线获取失败 {}: {}", code, e);
            return Err(SourceError::Network(e.to_string()));
        }
    };
    let mut candles: Vec<Candle> = points
        .into_iter()
        .filter_map(|p| {
            let date = parse_kline_date(&p.trade_date)?;
            Some(Candle {
                date,
                open: p.open,
                high: p.high,
                low: p.low,
                close: p.close,
                volume: p.volume as f64,
            })
        })
        .collect();
    // 升序排序，确保指标计算顺序正确
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(candles)
}

/// 拉取单只标的的分钟 K 线（akshare `stock_zh_a_minute`），映射为 `Candle`（升序）。
/// `period` ∈ {"1","5","15","30","60"}。`count` 为期望保留根数；Sina 历史有限，
/// 实际返回以可用数据为准（1H 约 36 根，不足 `count` 不报错）。
pub async fn fetch_minute_klines(
    client: &AkShareClient,
    code: &str,
    period: &str,
    count: usize,
) -> Result<Vec<Candle>, SourceError> {
    let bars = match client.stock_zh_a_minute(code, period).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("分钟K线失败 {}@{}: {}", code, period, e);
            return Err(SourceError::Network(e.to_string()));
        }
    };
    let mut candles: Vec<Candle> = bars
        .into_iter()
        .filter_map(|b| {
            let dt = parse_minute_datetime(&b.datetime)?;
            Some(Candle {
                date: dt,
                open: b.open,
                high: b.high,
                low: b.low,
                close: b.close,
                volume: b.volume,
            })
        })
        .collect();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    if candles.len() > count {
        candles = candles.split_off(candles.len() - count);
    }
    Ok(candles)
}

/// 将 yfinance-rs 的 `paft` OHLCV 柱映射为自有 `Candle`（按时间升序）。
fn map_yf_candle(c: yfinance_rs::core::models::Candle) -> Candle {
    let date: NaiveDateTime = c.ts.naive_utc();
    let open = amount_to_f64(c.ohlc.open.into_inner());
    let high = amount_to_f64(c.ohlc.high.into_inner());
    let low = amount_to_f64(c.ohlc.low.into_inner());
    let close = amount_to_f64(c.ohlc.close.into_inner());
    let volume = c
        .volume
        .map(|q| amount_to_f64(Decimal::from(q.into_inner())))
        .unwrap_or(0.0);
    Candle {
        date,
        open,
        high,
        low,
        close,
        volume,
    }
}

/// 分钟周期字符串（akshare 形式 "1"/"5"/"15"/"30"/"60"）映射到 yfinance `Interval`。
fn interval_from_tf(tf: &str) -> Interval {
    match tf {
        "1" => Interval::I1m,
        "5" => Interval::I5m,
        "15" => Interval::I15m,
        "30" => Interval::I30m,
        "60" => Interval::I1h,
        _ => Interval::I1h,
    }
}

/// 拉取单只美股的历史 K 线（日线或分钟），映射为 `Candle`（升序）。
///
/// `range` 控制回看区间（日线用 `Range::Y1` 约 252 根；分钟用 `Range::M1` 约 1 个月），
/// `interval` 为目标周期。`count` 为期望保留根数（超出尾部截断，不足不报错）。
pub async fn fetch_klines_us(
    client: &YfClient,
    symbol: &str,
    range: Range,
    interval: Interval,
    count: usize,
) -> Result<Vec<Candle>, SourceError> {
    let ticker = Ticker::new(client, symbol);
    let bars = match ticker.history(Some(range), Some(interval), false).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("美股K线获取失败 {}: {}", symbol, e);
            return Err(SourceError::Network(e.to_string()));
        }
    };
    let mut candles: Vec<Candle> = bars.into_iter().map(map_yf_candle).collect();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    if candles.len() > count {
        candles = candles.split_off(candles.len() - count);
    }
    Ok(candles)
}

