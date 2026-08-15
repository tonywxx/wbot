//! Data sources: A-share (`AkShareSource`) and US (`YfSource`) adapters over the
//! `MarketSource` trait, plus historical K-line fetch helpers.

use akshare::AkShareClient;
use yfinance_rs::{Decimal, Interval, Range, Ticker, YfClient};
use reqwest::Client;
use crate::indicators::Candle;
use chrono::{NaiveDate, NaiveDateTime};
use num_traits::cast::ToPrimitive;
use std::future::Future;
use std::pin::Pin;

use super::types::{Market, MarketData, Quote, SourceError, pct_change};
use super::realtime::{
    em_fetch_board, em_fetch_quotes_batch, em_fetch_quote, realtime_http_client,
    yahoo_fetch_quotes_batch,
};

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

/// A 股数据源：历史 K 线仍走 `AkShareClient`；实时报价 / 盘口快照改走东方财富
/// （`EmClient` 见文件末尾的实时源实现），与 `OkxSource` 同构。
pub struct AkShareSource {
    client: AkShareClient,
    http: Client,
}

impl AkShareSource {
    pub fn new() -> Self {
        Self {
            client: AkShareClient::new(),
            http: realtime_http_client(),
        }
    }
}

impl Default for AkShareSource {
    fn default() -> Self {
        Self::new()
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
        let http = self.http.clone();
        // 东方财富全市场盘口（个股 + 主要指数），best-effort：失败时返回空快照而非崩溃。
        Box::pin(async move { Some(em_fetch_board(&http).await) })
    }

    fn fetch_quotes<'a>(
        &'a self,
        codes: &'a [String],
    ) -> Pin<Box<dyn Future<Output = Vec<Quote>> + Send + 'a>> {
        let http = self.http.clone();
        Box::pin(async move {
            // A 股实时报价走东方财富：先按 secids 批量拉取（一次请求），
            // 若批量失败则回退到逐代码 `stock/get`，保证自选股表格能更新。
            let mut spots = em_fetch_quotes_batch(&http, codes).await;
            if spots.is_empty() && !codes.is_empty() {
                for code in codes {
                    if let Some(s) = em_fetch_quote(&http, code).await {
                        spots.push(s);
                    }
                }
            }
            spots
                .into_iter()
                .map(|s| Quote {
                    code: s.code,
                    name: s.name,
                    latest_price: s.latest_price,
                    change_pct: s.change_pct,
                    market: Market::A,
                })
                .collect()
        })
    }
}

/// 美股数据源：历史 K 线仍走 `YfClient`（Yahoo Finance via yfinance-rs）；
/// 实时报价改走 Yahoo `v8/finance/chart`（免 key，见文件末尾 `yahoo_fetch_quote`），
/// 与东方财富实时源同构，且不受 yfinance-rs `quote()` 接口变动影响。
pub struct YfSource {
    client: YfClient,
    http: Client,
}

impl YfSource {
    pub fn new() -> Self {
        Self {
            client: YfClient::default(),
            http: realtime_http_client(),
        }
    }
}

impl Default for YfSource {
    fn default() -> Self {
        Self::new()
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
        let http = self.http.clone();
        // 美股无 A 股式全市场盘口。改为走 Yahoo `v7/finance/quote` 批量接口
        // （一次请求最多约 50 只），避免逐代码串行请求带来的高延迟与速率限制；
        // `yahoo_fetch_quotes_batch` 内部在批量接口失效时自动逐代码回退到 `v8/chart`。
        let symbols: Vec<&str> = codes.iter().map(|s| s.as_str()).collect();
        Box::pin(async move {
            if codes.is_empty() {
                return Vec::new();
            }
            let quotes = yahoo_fetch_quotes_batch(&http, &symbols).await;
            let mut out = Vec::with_capacity(quotes.len());
            for (symbol, price, prev, name) in quotes {
                if price <= 0.0 {
                    continue;
                }
                let change_pct = pct_change(price, prev);
                let code = symbol.clone();
                out.push(Quote {
                    code,
                    name: if name.is_empty() { symbol } else { name },
                    latest_price: price,
                    change_pct,
                    market: Market::Us,
                });
            }
            out
        })
    }
}

/// 将 yfinance 的 `PriceAmount` / `QuantityAmount` 转为 `f64`（不足时回退 0.0）。
fn amount_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
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
    candles.sort_by_key(|a| a.date);
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
    candles.sort_by_key(|a| a.date);
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
    candles.sort_by_key(|a| a.date);
    if candles.len() > count {
        candles = candles.split_off(candles.len() - count);
    }
    Ok(candles)
}
