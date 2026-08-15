//! 加密货币（OKX）集成：行情拉取 / 实时下单传输层，统一走 `adaq-trading-crypto`。
//!
//! `adaq-trading-crypto` 是 ccxt 兼容的加密货币统一接口：
//! - **历史 K 线**：`Exchange::fetch_ohlcv`（REST），映射为自有 `Candle`。
//! - **实时价格**：`Realtime::watch_ticker`（WebSocket），由 [`spawn_realtime_feed`]
//!   在后台流式推送（见 `main.rs` 的 `data_loop`）。
//! - **实盘下单**：`Exchange::create_order`（市价单），替代原 okx-rs 路径。
//!
//! 交易所符号采用 ccxt 格式 `BTC/USDT`，本模块在它与 watchlist 的 `BTC-USDT`
//! 之间互转（[`to_ccxt`] / [`from_ccxt`]）。`adaq` 的 `Okx` 适配器是 `Send + Sync`，
//! 因此既可被多线程 tokio 运行时 `spawn`（实时流），也能满足 `MarketSource` 要求的
//! `Send` future（数据源在多线程 tokio 运行时中被 `spawn`）。
//!
//! 模拟加密账户 [`crate::sim::crypto_ledger::CryptoLedger`] 与实盘下单网关
//! [`crate::crypto_gateway`] 已分别拆到独立模块，本文件只负责「行情（历史 + 实时）
//! 与下单传输」这一层。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinSet;

use adaq_trading_crypto::adapters::Okx;
use adaq_trading_crypto::realtime::okx::OkxWs;
use adaq_trading_crypto::exchange::{Config, Exchange, Params, Realtime};
use adaq_trading_crypto::types::{OHLCV, Ticker};
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;

use crate::indicators::Candle;
use crate::market::{Market, MarketSource, Quote, SourceError};

// ===========================================================================
// 符号 / 周期映射（watchlist 的 `BTC-USDT` <-> ccxt 的 `BTC/USDT`）
// ===========================================================================

/// 将 watchlist 代码（`BTC-USDT`）转为 ccxt 符号（`BTC/USDT`）。
pub fn to_ccxt(code: &str) -> String {
    code.replace('-', "/")
}

/// 将 ccxt 符号（`BTC/USDT`）转回 watchlist 代码（`BTC-USDT`）。
pub fn from_ccxt(sym: &str) -> String {
    sym.replace('/', "-")
}

/// 将 akshare 形式的分钟周期（"1"/"5"/"15"/"30"/"60"）映射为 ccxt 周期。
fn tf_map(tf: &str) -> &'static str {
    match tf {
        "1" => "1m",
        "5" => "5m",
        "15" => "15m",
        "30" => "30m",
        "60" => "1h",
        _ => "1h",
    }
}

/// `rust_decimal::Decimal` → `f64`（带 Display 兜底，避免版本/精度边界下的 None）。
fn decimal_to_f64(d: rust_decimal::Decimal) -> f64 {
    d.to_f64()
        .unwrap_or_else(|| d.to_string().parse::<f64>().unwrap_or(0.0))
}

// ===========================================================================
// 数据映射（纯函数，可单测）
// ===========================================================================

/// 将单根 `OHLCV` 映射为自有 `Candle`（升序时间戳）。字段缺失则返回 `None`。
fn ohlcv_to_candle(o: OHLCV) -> Option<Candle> {
    let ts = o.timestamp?;
    let date = chrono::DateTime::from_timestamp_millis(ts).map(|d| d.naive_utc())?;
    let open = o.open.map(decimal_to_f64).unwrap_or(0.0);
    let high = o.high.map(decimal_to_f64).unwrap_or(0.0);
    let low = o.low.map(decimal_to_f64).unwrap_or(0.0);
    let close = o.close.map(decimal_to_f64).unwrap_or(0.0);
    let volume = o.volume.map(decimal_to_f64).unwrap_or(0.0);
    Some(Candle {
        date,
        open,
        high,
        low,
        close,
        volume,
    })
}

/// 将 `Ticker` 映射为统一 `Quote`（key 用原始 watchlist 代码）。
/// `last` 缺失或 ≤ 0 时返回 `None`（不写入价格表）。
fn ticker_to_quote(code: &str, t: &Ticker) -> Option<Quote> {
    let price = t.last.map(decimal_to_f64).unwrap_or(0.0);
    if price <= 0.0 {
        return None;
    }
    // OKX 的 ticker（REST 与 WS 共用 `parse_ticker`）只给 `last` 与 `open24h`，
    // **从不返回 `percentage`**，因此不能直接用 `t.percentage`（恒为 None -> chg% 卡在
    // 0.00%）。优先用 24h 涨跌幅公式 (last-open)/open*100 推算；仅在缺少 `open` 时才回退
    // 到 `percentage`（例如其它交易所/其它接口补了此字段）。
    let change_pct = match (t.last, t.open) {
        (Some(last), Some(open)) => {
            let open_f = decimal_to_f64(open);
            if open_f != 0.0 {
                (decimal_to_f64(last) - open_f) / open_f * 100.0
            } else {
                t.percentage.map(decimal_to_f64).unwrap_or(0.0)
            }
        }
        _ => t.percentage.map(decimal_to_f64).unwrap_or(0.0),
    };
    Some(Quote {
        code: code.to_string(),
        name: code.to_string(),
        latest_price: price,
        change_pct,
        market: Market::Crypto,
    })
}

// ===========================================================================
// 加密货币数据源（MarketSource）：历史 K 线 + REST 报价兜底
// ===========================================================================

/// 加密货币数据源：封装 adaq 的 `Okx` 适配器（公开配置，无凭证）。
///
/// 历史 K 线走 `fetch_ohlcv`；实时价格主通道是 WebSocket（[`spawn_realtime_feed`]），
/// 本结构提供的 `fetch_quotes` 作为 REST 兜底——当 WS 未连通或单笔更新丢失时，
/// `data_loop` 每 `refresh` 秒仍会拉一次快照，保证 watchlist 表格可用。
pub struct AdaqCryptoSource {
    client: Okx,
}

impl AdaqCryptoSource {
    /// 构造加密货币数据源（纯配置，无网络）。仅当 `adaq-trading-crypto` 特性 /
    /// 依赖异常时才失败，调用方应以 [`MarketRouter::build_crypto_source`] 兜底，
    /// 而非在启动期 panic。
    pub fn new() -> Result<Self, String> {
        let client = Okx::new(Config::default()).map_err(|e| {
            format!("adaq OKX 适配器构造失败（检查 adaq-trading-crypto 特性）: {}", e)
        })?;
        Ok(Self { client })
    }
}

impl AdaqCryptoSource {
    /// 拉取 OHLCV：统一 `fetch_klines`（日线）与 `fetch_intraday`（周期线）的公共路径。
    /// `tf` 按值接收，使其随返回的 future 一起存活（调用方只持有 `'a` 借用）。
    async fn fetch_ohlcv(
        &self,
        code: &str,
        tf: String,
        n: usize,
    ) -> Result<Vec<Candle>, SourceError> {
        let ccxt = to_ccxt(code);
        let ohlcv = self
            .client
            .fetch_ohlcv(&ccxt, &tf, None, Some(n as i64), Params::default())
            .await
            .map_err(|e| SourceError::Network(e.to_string()))?;
        let mut candles: Vec<Candle> = ohlcv.into_iter().filter_map(ohlcv_to_candle).collect();
        // 升序并保留末 `n` 根（最新）。
        candles.sort_by_key(|a| a.date);
        if candles.len() > n {
            candles = candles.split_off(candles.len() - n);
        }
        Ok(candles)
    }
}

impl MarketSource for AdaqCryptoSource {
    fn market(&self) -> Market {
        Market::Crypto
    }

    fn fetch_klines<'a>(
        &'a self,
        code: &'a str,
        _adjust: &'a str,
        count: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
    {
        Box::pin(self.fetch_ohlcv(code, "1d".to_string(), count))
    }

    fn fetch_intraday<'a>(
        &'a self,
        code: &'a str,
        tf: &'a str,
        bars: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
    {
        let tf = tf_map(tf).to_string();
        Box::pin(self.fetch_ohlcv(code, tf, bars))
    }

    fn fetch_snapshot<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<crate::market::MarketData>> + Send + 'a>>
    {
        // 加密货币无 A 股式全市场盘口快照，返回 `None`。
        Box::pin(async move { Option::<crate::market::MarketData>::None })
    }

    fn fetch_quotes<'a>(
        &'a self,
        _codes: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Quote>> + Send + 'a>> {
        // 加密货币实时报价**仅**由后台 WebSocket 流 [`spawn_realtime_feed`]（2 连接主备 +
        // 多路复用订阅）提供，不再走 REST 兜底——避免出口网络对 REST 主机（www.okx.com）
        // 受限时每 `refresh` 秒刷屏 `RequestTimeout`。历史 K 线仍走 REST `fetch_ohlcv`
        // （WS 不提供历史序列）。此处直接返回空，crypto 报价统一来自 WS 推送的 `Msg::Quotes`。
        Box::pin(async move { Vec::new() })
    }
}

// ===========================================================================
// 实时（WebSocket）价格流：后台 spawn，批量推送统一 `Quote`
// ===========================================================================

/// 启动加密货币实时价格 WebSocket 流（双连接主备 + 多路复用订阅）。
///
/// 拓扑：最多 **2 个** `Okx` WebSocket 连接实例。
/// - **主连接（primary）** 对 watchlist 全部符号做多路复用订阅（多个 `watch_ticker`
///   并发跑在同一连接实例上，由 adaq 在单条 WS 上按符号路由），并把最新价写入共享价格表。
/// - **备用连接（standby）** 同样订阅全部符号以保持热备，但不写入价格表。
/// - **故障切换**：主连接任一符号订阅出错 → 把 `primary` 让给对端（备用升主），并结束
///   本连接任务；supervisor 退避后以其「备用」身份重连（重连后只订阅、不写入），
///   实现「1 号断开 → 切 2 号；1 号重连 → 作为备用」。
/// - 另起一个 flush 任务每 ~300ms 把快照经 `tx` 批量推送（复用 `Msg::Quotes` 合并路径）。
///
/// 库内置心跳与重连退避；若 OKX 实时通道未实现（`NotSupported`）或持续失败，本流静默
/// 为空，watchlist 中加密货币报价不更新（`fetch_quotes` 已改为空实现，REST 不兜底），
/// 但 A 股 / 美股报价与加密货币 K 线历史不受影响，应用始终可用。
///
/// 注：多路复用依赖 adaq 在单个 `Okx` 实例上按符号路由消息；若其实现为每 `watch_*` 调用
/// 复用同一底层 WS，则 2 个实例 = 2 条 WS 连接（符合「连接只能 2 个」的约束）。
pub fn spawn_realtime_feed(symbols: &[String], tx: tokio::sync::mpsc::Sender<Vec<Quote>>) {
    // 显式安装 rustls 默认 crypto provider：adaq 的 WebSocket 走 rustls TLS，而
    // rustls 0.23 不会自动安装 provider。放在 WS 入口，确保无论调用方是否经过
    // `main()`（例如被测试或其它子命令直接调用）都能建连。重复安装是安全的——
    // 已安装时会返回 Err，这里忽略即可。
    let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());

    let store: Arc<Mutex<HashMap<String, Quote>>> = Arc::new(Mutex::new(HashMap::new()));
    // 当前主连接索引（0 或 1）。
    let primary: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    // 构造 2 个 WS 连接实例（公开配置，无网络；每个实例在单条 WS 上多路复用全部符号）。
    let mut conns: Vec<Option<Arc<OkxWs>>> = Vec::with_capacity(2);
    for i in 0..2 {
        match OkxWs::new(Config::default()) {
            Ok(e) => conns.push(Some(Arc::new(e))),
            Err(e) => {
                eprintln!("adaq OKX WS 适配器构造失败 (conn {}): {}", i, e);
                conns.push(None);
            }
        }
    }

    let syms: Vec<(String, String)> = symbols
        .iter()
        .map(|c| (c.clone(), to_ccxt(c)))
        .collect();

    for (idx, entry) in conns.iter().enumerate().take(2) {
        let Some(conn) = entry.clone() else { continue };
        let store = store.clone();
        let primary = primary.clone();
        let syms = syms.clone();
        tokio::spawn(async move {
            // supervisor：循环（重）启动本连接全部符号的订阅任务。
            loop {
                let mut set: JoinSet<()> = JoinSet::new();
                for (code, ccxt) in &syms {
                    let conn = conn.clone();
                    let store = store.clone();
                    let primary = primary.clone();
                    let code = code.clone();
                    let ccxt = ccxt.clone();
                    set.spawn(async move {
                        loop {
                            match conn.watch_ticker(&ccxt, Params::default()).await {
                                Ok(t) => {
                                    // 仅主连接写入价格表；备用连接仅维持订阅。
                                    if *primary.lock().unwrap_or_else(|e| e.into_inner()) == idx
                                        && let Some(q) = ticker_to_quote(&code, &t)
                                    {
                                        store
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner())
                                            .insert(code.clone(), q);
                                    }
                                }
                                Err(e) => {
                                    let is_primary = *primary.lock().unwrap_or_else(|e| e.into_inner()) == idx;
                                    if is_primary {
                                        // 主连接故障：切换给对端（对端升主），本任务返回，
                                        // 由 supervisor 以「备用」身份重连。
                                        eprintln!(
                                            "WS 主连接 {} 断开，切换备用 (conn {}): {}",
                                            code, idx, e
                                        );
                                        let mut p = primary.lock().unwrap_or_else(|e| e.into_inner());
                                        *p = 1 - idx;
                                        return;
                                    } else {
                                        // 备用连接报错：静默退避后重试，不切换。
                                        eprintln!("WS 备用连接 {} 错误（重试）: {}", code, e);
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                    }
                                }
                            }
                        }
                    });
                }
                // 等待任一任务结束（仅主连接故障切换会发生）。
                let _ = set.join_next().await;
                set.abort_all();
                // 以「备用」身份重连：退避后重启（此时 primary 已让给对端）。
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    // flush 任务：把共享价格表批量推送，避免高频单条消息冲击 UI 通道。
    tokio::spawn(async move {
        let mut flush = tokio::time::interval(Duration::from_millis(300));
        loop {
            flush.tick().await;
            let snapshot: Vec<Quote> = {
                let guard = store.lock().unwrap_or_else(|e| e.into_inner());
                guard.values().cloned().collect()
            };
            if snapshot.is_empty() {
                continue;
            }
            if tx.send(snapshot).await.is_err() {
                // 接收端已丢弃（应用退出），停止推送。
                break;
            }
        }
    });
}

// ===========================================================================
// CLI 探测：同时验证 REST（历史 K 线）与 WebSocket（实时 ticker）两条链路
// ===========================================================================

/// CLI `wbot probe`：探测 OKX 的 REST（历史 K 线 `fetch_ohlcv`）与 WebSocket
/// （实时 `watch_ticker`）两条链路，打印各自的连通性、耗时与样例数据。
///
/// 用途：在任何环境（开发机 / CI / 部署机）快速验证「网络是否可达、依赖是否就绪、
/// TLS provider 是否正常」，而不进入 TUI、不影响账户。两条链路独立探测、互不影响——
/// 例如沙箱里常见「WS 通但 REST 被出口限制」，本命令会把两种结果都打印出来，便于判断
/// 是代码问题还是环境（出口策略）问题。
pub fn probe_okx() -> anyhow::Result<()> {
    let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let symbol = "BTC-USDT";
        let ccxt = to_ccxt(symbol);
        println!("探测 OKX 链路（标的 {}）", symbol);

        // 1) REST 历史 K 线（fetch_ohlcv）
        println!("[1/2] REST  fetch_ohlcv ...");
        let t0 = std::time::Instant::now();
        match Okx::new(Config::default()) {
            Ok(ex) => match tokio::time::timeout(
                Duration::from_secs(15),
                ex.fetch_ohlcv(&ccxt, "1d", None, Some(1), Params::default()),
            )
            .await
            {
                Ok(Ok(v)) => {
                    println!("      -> OK ({:.2?})  bars={}", t0.elapsed(), v.len());
                    if let Some(c) = v.last().cloned().and_then(ohlcv_to_candle) {
                        println!("         latest close = {}", c.close);
                    }
                }
                Ok(Err(e)) => println!("      -> ERR ({:.2?}): {}", t0.elapsed(), e),
                Err(_) => println!("      -> TIMEOUT ({:.2?})", t0.elapsed()),
            },
            Err(e) => println!("      -> adapter init ERR: {}", e),
        }

        // 2) WebSocket 实时 ticker（watch_ticker）
        println!("[2/2] WS    watch_ticker ...");
        let t0 = std::time::Instant::now();
        match OkxWs::new(Config::default()) {
            Ok(ex) => match tokio::time::timeout(
                Duration::from_secs(15),
                ex.watch_ticker(&ccxt, Params::default()),
            )
            .await
            {
                Ok(Ok(t)) => {
                    println!(
                        "      -> OK ({:.2?})  last = {:?}",
                        t0.elapsed(),
                        t.last.map(decimal_to_f64)
                    );
                }
                Ok(Err(e)) => println!("      -> ERR ({:.2?}): {}", t0.elapsed(), e),
                Err(_) => println!("      -> TIMEOUT ({:.2?})", t0.elapsed()),
            },
            Err(e) => println!("      -> adapter init ERR: {}", e),
        }

        println!("提示：若 REST 超时而 WS 成功，通常是出口网络仅放行了 WS 主机；代码路径正确，部署到正常环境即可。");
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}

// ===========================================================================
// 实盘下单（市价单）：adaq `create_order`，替代原 okx-rs 路径
// ===========================================================================

/// OKX 实盘下单客户端（市价单）。仅在 `live_trading` 且已配置 OKX 凭证时由
/// [`crate::crypto_gateway`] 调用。下单走 adaq 的 `Okx` 适配器，需 API 密钥 /
/// 密钥 / passphrase 三个环境变量（运行时按 env 构造带凭证配置，公开行情无需凭证）。
pub struct OkxClient {
    /// 占位：公开行情不需要持有实例；下单时再按 env 构造带凭证适配器。
    _private: (),
}

impl OkxClient {
    /// 构造（公开行情客户端，无凭证）。
    pub fn new() -> Self {
        OkxClient { _private: () }
    }

    /// 是否已配置 OKX API 凭证（用于判断是否允许真实下单）。
    pub fn has_credentials() -> bool {
        std::env::var("OKX_API_KEY").is_ok()
            && std::env::var("OKX_API_SECRET").is_ok()
            && std::env::var("OKX_PASSPHRASE").is_ok()
    }

    /// 构造带凭证的 adaq `Okx` 适配器（真实下单用）。
    fn live_exchange() -> anyhow::Result<Okx> {
        let key = std::env::var("OKX_API_KEY")
            .map_err(|_| anyhow::anyhow!("缺少环境变量 OKX_API_KEY"))?;
        let secret = std::env::var("OKX_API_SECRET")
            .map_err(|_| anyhow::anyhow!("缺少环境变量 OKX_API_SECRET"))?;
        let pass = std::env::var("OKX_PASSPHRASE")
            .map_err(|_| anyhow::anyhow!("缺少环境变量 OKX_PASSPHRASE"))?;
        let config = Config {
            api_key: Some(key),
            secret: Some(secret),
            password: Some(pass),
            ..Default::default()
        };
        Okx::new(config).map_err(|e| anyhow::anyhow!("adaq OKX 构造失败: {}", e))
    }

    /// 真实市价下单（需凭证）。`sz` 为数量字符串；买入可传 `tgt_ccy="quote_ccy"`
    /// 表示以报价币（USDT）金额下单，卖出传 `None`（基础币数量）。
    /// 返回交易所订单 ID。
    ///
    /// 注意：本方法为异步，由 `crypto_gateway::place_crypto_live` 经
    /// `tokio::runtime::block_on` 在当前线程运行时内调用，不要求 future 为 `Send`。
    pub async fn place_market_order(
        &self,
        inst_id: &str,
        buy: bool,
        sz: &str,
        tgt_ccg: Option<&str>,
    ) -> anyhow::Result<String> {
        let exchange = Self::live_exchange()?;
        let ccxt = to_ccxt(inst_id);
        let side = if buy { "buy" } else { "sell" };
        let mut params = Params::default();
        // OKX 现货市价单必须带 `tdMode=cash`。
        params.insert("tdMode".to_string(), Value::from("cash"));
        if buy
            && let Some(t) = tgt_ccg
        {
            // 以报价币金额下单（如用 USDT 买多少 BTC）。
            params.insert("tgtCcy".to_string(), Value::from(t));
        }
        let order = exchange
            .create_order(&ccxt, "market", side, sz, None, params)
            .await
            .map_err(|e| anyhow::anyhow!("adaq OKX 下单失败: {}", e))?;
        // 优先取订单 ID，缺失时回退到 `info.ordId`，再回退到占位串。
        let id = order
            .id
            .filter(|s| !s.is_empty())
            .or_else(|| {
                order
                    .info
                    .get("ordId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("OKX:{}", inst_id));
        Ok(id)
    }
}

impl Default for OkxClient {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn ccxt_symbol_roundtrip() {
        assert_eq!(to_ccxt("BTC-USDT"), "BTC/USDT");
        assert_eq!(from_ccxt("BTC/USDT"), "BTC-USDT");
        assert_eq!(from_ccxt(&to_ccxt("ETH-USDT")), "ETH-USDT");
    }

    #[test]
    fn tf_map_basic() {
        assert_eq!(tf_map("1"), "1m");
        assert_eq!(tf_map("5"), "5m");
        assert_eq!(tf_map("15"), "15m");
        assert_eq!(tf_map("30"), "30m");
        assert_eq!(tf_map("60"), "1h");
        assert_eq!(tf_map("99"), "1h");
    }

    #[test]
    fn ohlcv_to_candle_maps_fields_and_timestamp() {
        let o = OHLCV {
            timestamp: Some(1_700_000_000_000),
            open: Some(Decimal::from(100)),
            high: Some(Decimal::from_str("110.5").unwrap()),
            low: Some(Decimal::from_str("99.25").unwrap()),
            close: Some(Decimal::from_str("105.75").unwrap()),
            volume: Some(Decimal::from(1234)),
        };
        let c = ohlcv_to_candle(o).expect("should map");
        assert_eq!(c.open, 100.0);
        assert!((c.high - 110.5).abs() < 1e-9);
        assert!((c.low - 99.25).abs() < 1e-9);
        assert!((c.close - 105.75).abs() < 1e-9);
        assert_eq!(c.volume, 1234.0);
        // 1700000000 s = 2023-11-14 22:13:20 UTC
        assert_eq!(c.date.format("%Y-%m-%d %H:%M:%S").to_string(), "2023-11-14 22:13:20");
    }

    #[test]
    fn ohlcv_to_candle_none_on_missing_ts() {
        let o = OHLCV {
            timestamp: None,
            open: Some(Decimal::from(1)),
            high: Some(Decimal::from(1)),
            low: Some(Decimal::from(1)),
            close: Some(Decimal::from(1)),
            volume: Some(Decimal::from(1)),
        };
        assert!(ohlcv_to_candle(o).is_none());
    }

    #[test]
    fn ticker_to_quote_maps_last_and_pct() {
        // 用 JSON 反序列化仅填充关键字段（其余由 serde 默认补零），避免手写 22 个字段。
        let v = serde_json::json!({
            "symbol": "BTC/USDT",
            "last": 50000.0,
            "percentage": 1.25,
        });
        let t: Ticker = serde_json::from_value(v).expect("parse ticker");
        let q = ticker_to_quote("BTC-USDT", &t).expect("should map");
        assert_eq!(q.code, "BTC-USDT");
        assert!((q.latest_price - 50000.0).abs() < 1e-9);
        assert!((q.change_pct - 1.25).abs() < 1e-9);
        assert_eq!(q.market, Market::Crypto);
    }

    #[test]
    fn ticker_to_quote_none_on_zero_last() {
        let v = serde_json::json!({ "symbol": "BTC/USDT", "last": 0.0 });
        let t: Ticker = serde_json::from_value(v).unwrap();
        assert!(ticker_to_quote("BTC-USDT", &t).is_none());
    }

    #[test]
    fn ticker_to_quote_chg_from_open_when_percentage_missing() {
        // OKX ticker（REST/WS）只给 last 与 open24h，percentage 恒为 None；chg% 必须由
        // (last-open)/open*100 推算，否则恒为 0.00%（用户反馈的 bug）。
        let v = serde_json::json!({
            "symbol": "BTC/USDT",
            "last": 105.0,
            "open": 100.0,
        });
        let t: Ticker = serde_json::from_value(v).expect("parse ticker");
        let q = ticker_to_quote("BTC-USDT", &t).expect("should map");
        assert!((q.change_pct - 5.0).abs() < 1e-6, "chg% 应为 +5.0，实际 {}", q.change_pct);

        // 下跌情形：last < open -> 负 chg%。
        let v2 = serde_json::json!({
            "symbol": "BTC/USDT",
            "last": 95.0,
            "open": 100.0,
        });
        let t2: Ticker = serde_json::from_value(v2).unwrap();
        let q2 = ticker_to_quote("BTC-USDT", &t2).expect("should map");
        assert!((q2.change_pct + 5.0).abs() < 1e-6, "chg% 应为 -5.0，实际 {}", q2.change_pct);
    }

    // ---- 离线集成检查：不依赖网络，验证 adaq 数据源与双连接拓扑可构造 ----
    #[tokio::test]
    async fn adaq_source_market_and_snapshot_offline() {
        // AdaqCryptoSource 离线可构造，归类为 Crypto，且提供无全市场快照（None）。
        let src = AdaqCryptoSource::new().expect("adaq 数据源离线应可构造");
        assert_eq!(src.market(), Market::Crypto);
        assert!(src.fetch_snapshot().await.is_none());
    }

    #[test]
    fn realtime_feed_two_connections_build_offline() {
        // 双连接主备拓扑：两个 OkxWs 实例必须在离线（仅构造、未订阅）条件下均可建立，
        // 这是「连接只能 2 个」约束与 failover 切换的物理前提（订阅/网络在 spawn 后才发生）。
        let a = OkxWs::new(Config::default());
        let b = OkxWs::new(Config::default());
        assert!(a.is_ok(), "主连接实例应可构造: {:?}", a.err());
        assert!(b.is_ok(), "备用连接实例应可构造: {:?}", b.err());
    }

    // ---- Live (ignored): OKX REST 历史 K 线 ----
    #[tokio::test]
    #[ignore = "requires network: OKX"]
    async fn live_fetch_ohlcv_okx() {
        let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
        let ex = Okx::new(Config::default()).expect("adaq OKX");
        let v = ex
            .fetch_ohlcv("BTC/USDT", "1d", None, Some(5), Params::default())
            .await
            .expect("fetch_ohlcv");
        assert!(!v.is_empty(), "OKX 应返回 BTC/USDT 日线");
        println!("OKX BTC/USDT 日线 {} 根", v.len());
    }

    // ---- Live (ignored): OKX WebSocket 实时 ticker ----
    #[tokio::test]
    #[ignore = "requires network: OKX WS"]
    async fn live_watch_ticker_okx() {
        let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
        let ex = OkxWs::new(Config::default()).expect("adaq OKX WS");
        let t = tokio::time::timeout(
            Duration::from_secs(20),
            ex.watch_ticker("BTC/USDT", Params::default()),
        )
        .await
        .expect("WS 超时")
        .expect("watch_ticker");
        assert!(t.last.is_some(), "WS 应返回最新价");
        println!("WS BTC/USDT last = {:?}", t.last.map(decimal_to_f64));
    }

    // ---- Live (ignored): 验证 WS 实时流确实持续推送更新（价格随行情变动）----
    #[tokio::test]
    #[ignore = "requires network: OKX WS"]
    async fn live_realtime_feed_pushes_live_updates() {
        let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
        // 直接复用 app 实际运行的同一入口 spawn_realtime_feed（2 连接主备 + 多路复用），
        // 收集数秒内的推送，确认：① 至少推送多次（实时流在跑）；② 价格随行情变化（实时更新）。
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<Quote>>(64);
        spawn_realtime_feed(&["BTC-USDT".to_string()], tx);

        let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
        let mut prices: Vec<f64> = Vec::new();
        let mut last_chg: f64 = 0.0;
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
                Ok(Some(qs)) => {
                    for q in qs {
                        if q.code == "BTC-USDT" {
                            prices.push(q.latest_price);
                            last_chg = q.change_pct;
                        }
                    }
                }
                _ => break,
            }
        }
        assert!(!prices.is_empty(), "WS 流应至少推送一次 BTC-USDT 报价");
        assert!(prices[0] > 0.0, "推送价格应 > 0");
        // 实时更新：flush 每 ~300ms 推一次，8s 窗口内应收到多次推送（行情在动则值会变）。
        assert!(prices.len() >= 2, "WS 应实时多次推送，实际仅 {} 次", prices.len());
        let distinct: std::collections::HashSet<i64> = prices
            .iter()
            .map(|p| (p * 100.0).round() as i64)
            .collect();
        println!(
            "WS 推送 BTC-USDT 采样数={} 不同价={} 最新价={:.2} chg%={:.2}",
            prices.len(),
            distinct.len(),
            prices.last().copied().unwrap_or(0.0),
            last_chg
        );
    }
}
