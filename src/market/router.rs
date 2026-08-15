//! Dual-source router (`MarketRouter`) and watchlist loading.

use std::collections::HashMap;
use crate::indicators::Candle;

use super::types::{market_of, Market, MarketData, Quote, SourceError, IndexSpot, Breadth};
use super::source::{AkShareSource, MarketSource, YfSource};
use super::realtime::{fetch_us_breadth, fetch_us_indices, realtime_http_client};
use crate::crypto::AdaqCryptoSource;

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
            crypto: Box::new(AdaqCryptoSource::new()),
        }
    }

    /// 注入式构造（测试 / 替换数据源用）。加密货币默认用真实 `AdaqCryptoSource`。
    pub fn from_sources(a: Box<dyn MarketSource>, us: Box<dyn MarketSource>) -> Self {
        Self {
            a,
            us,
            crypto: Box::new(AdaqCryptoSource::new()),
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

    /// 美股指数栏（标普 / 纳指 / 道指 / 罗素 / VIX + 黄金 + WTI），每刷新周期拉取。
    pub async fn fetch_us_indices(&self) -> Vec<IndexSpot> {
        let http = realtime_http_client();
        fetch_us_indices(&http).await
    }

    /// 美股市场广度（样本篮子涨跌家数），每 60s 批量拉取一次。
    pub async fn fetch_us_breadth(&self) -> Breadth {
        let http = realtime_http_client();
        fetch_us_breadth(&http).await
    }
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

/// 解析自选股文件：按行取首个非注释 token。
fn parse_watchlist_file(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

/// 加载 A 股自选股：优先读取 cwd 下的 `watchlist_a.txt`（每行一个 6 位代码）。
/// 文件存在但没有任何标的时跳过该市场（返回空列表）；文件缺失才回退 [`DEFAULT_WATCHLIST`]。
pub fn load_watchlist() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist_a.txt") {
        return parse_watchlist_file(&text);
    }
    DEFAULT_WATCHLIST.iter().map(|s| s.to_string()).collect()
}

/// 加载美股自选股：优先读取 cwd 下的 `watchlist.txt`（每行一个 ticker）。
/// 文件存在但没有任何标的时跳过该市场（返回空列表）；文件缺失才回退 [`DEFAULT_WATCHLIST_US`]。
pub fn load_watchlist_us() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist.txt") {
        return parse_watchlist_file(&text);
    }
    DEFAULT_WATCHLIST_US.iter().map(|s| s.to_string()).collect()
}

/// 加载加密货币自选股：优先读取 cwd 下的 `watchlist_crypto.txt`（每行一个
/// `BASE-USDT` 交易对）。文件存在但没有任何标的时跳过该市场（返回空列表）；
/// 文件缺失才回退 [`DEFAULT_WATCHLIST_CRYPTO`]。
pub fn load_watchlist_crypto() -> Vec<String> {
    if let Ok(text) = std::fs::read_to_string("watchlist_crypto.txt") {
        return parse_watchlist_file(&text);
    }
    DEFAULT_WATCHLIST_CRYPTO.iter().map(|s| s.to_string()).collect()
}

/// 加载合并自选股：美股 → 加密货币（若 `crypto_enabled`）→ A 股。
/// 各市场优先读取其 `watchlist*.txt`（美股 `watchlist.txt`、加密货币
/// `watchlist_crypto.txt`、A 股 `watchlist_a.txt`）；文件存在但没有任何标的时
/// 该市场跳过，文件缺失则使用内置默认清单。
pub fn load_watchlist_combined(crypto_enabled: bool) -> Vec<String> {
    let mut v = Vec::new();
    v.extend(load_watchlist_us());
    if crypto_enabled {
        v.extend(load_watchlist_crypto());
    }
    v.extend(load_watchlist());
    v
}
