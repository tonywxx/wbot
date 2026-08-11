//! OKX 集成传输层（行情 + 真实下单）。
//!
//! - **公开行情**：`/market/candles` 历史 K 线、`/market/tickers` 实时价，直接用
//!   `reqwest`（Send + Sync）实现，映射为自有 `Candle`。之所以不用 okx-rs 的
//!   `request` 传输来做行情，是因为 okx-rs 的 `Options` 持有 `Arc<dyn OKXEnv>`
//!   （`OKXEnv` 未标注 `Send + Sync`），导致其 `request` future 不是 `Send`，无法满足
//!   本项目 `MarketSource` trait 要求的 `Send` future（数据源在多线程 tokio 运行时中被
//!   `spawn`）。因此公开行情走 reqwest，既保证 `Send`，又避免引入系统 OpenSSL 依赖。
//! - **真实下单**：`/trade/order` 市价单，复用 okx-rs 的 `Rest` 传输与签名管线
//!   （需 `OKX_API_KEY` / `OKX_API_SECRET` / `OKX_PASSPHRASE` 环境变量；仅在
//!   `live_trading` 为真时由 [`crate::crypto_gateway`] 调用）。下单路径通过
//!   `tokio::runtime::block_on` 执行，不要求 future 为 `Send`，故可安全使用 okx-rs。
//!
//! 模拟加密账户 [`crate::sim::crypto_ledger::CryptoLedger`] 与实盘下单网关
//! [`crate::crypto_gateway`] 已分别拆到独立模块（架构评审候选 5），本文件仅保留
//! OKX 的 HTTP 传输适配。

use okx_rs::api::{Options, Production, Rest};
use okx_rs::api::v5::Request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::indicators::Candle;
use crate::market::SourceError;

/// OKX 下单接口（`/trade/order`，市价单）。仅需填充必要字段。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct PlaceOrderReq {
    inst_id: String,
    /// 交易模式：现货为 "cash"。
    td_mode: String,
    /// "buy" / "sell"。
    side: String,
    /// "market" 市价单。
    ord_type: String,
    /// 数量（字符串）：买入时若带 `tgt_ccy=quote_ccy` 则为报价币金额，否则为基础币数量。
    sz: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    px: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tgt_ccy: Option<String>,
}

impl Request for PlaceOrderReq {
    const METHOD: Method = Method::POST;
    const PATH: &'static str = "/trade/order";
    const AUTH: bool = true;
    type Response = Vec<OrderData>;
}

/// 下单接口的单条返回数据。
#[derive(Debug, Clone, Deserialize)]
struct OrderData {
    #[serde(default)]
    ord_id: String,
    #[serde(default)]
    s_code: String,
    #[serde(default)]
    s_msg: String,
}

/// OKX 公开行情客户端（reqwest，Send + Sync）。
pub struct OkxClient {
    client: reqwest::Client,
}

impl OkxClient {
    /// 构造公开（无凭证）行情客户端。
    pub fn new() -> Self {
        OkxClient {
            client: reqwest::Client::new(),
        }
    }

    /// 是否已配置 OKX API 凭证（用于判断是否允许真实下单）。
    pub fn has_credentials() -> bool {
        std::env::var("OKX_API_KEY").is_ok()
            && std::env::var("OKX_API_SECRET").is_ok()
            && std::env::var("OKX_PASSPHRASE").is_ok()
    }

    /// 构造带凭证的 okx-rs 客户端（真实下单用）。
    fn live_client() -> anyhow::Result<Rest> {
        let key = std::env::var("OKX_API_KEY")
            .map_err(|_| anyhow::anyhow!("缺少环境变量 OKX_API_KEY"))?;
        let secret = std::env::var("OKX_API_SECRET")
            .map_err(|_| anyhow::anyhow!("缺少环境变量 OKX_API_SECRET"))?;
        let pass = std::env::var("OKX_PASSPHRASE")
            .map_err(|_| anyhow::anyhow!("缺少环境变量 OKX_PASSPHRASE"))?;
        Ok(Rest::new(Options::new_with(Production, key, secret, pass)))
    }

    /// 拉取历史 K 线（升序 `Candle`，保留末 `limit` 根）。
    ///
    /// OKX 单页最多 100 根；如需更多，按 `before` 参数翻页取更早数据，最后合并、
    /// 升序、截断到 `limit` 根。网络 / 解析失败时返回 [`SourceError`]（不再静默
    /// 返回截断的部分序列——截断的 K 线会让信号计算基于缺失近期根，比明示失败更危险）。
    pub async fn fetch_candles(
        &self,
        inst_id: &str,
        bar: &str,
        limit: usize,
    ) -> Result<Vec<Candle>, SourceError> {
        let needed = limit.max(1);
        let page = 100usize;
        let mut all: Vec<Candle> = Vec::new();
        let mut before: Option<String> = None;

        for _ in 0..12 {
            let mut url = format!(
                "https://www.okx.com/api/v5/market/candles?instId={}&bar={}&limit={}",
                inst_id, bar, page
            );
            if let Some(b) = &before {
                url.push_str(&format!("&before={}", b));
            }
            let rows: Vec<Vec<String>> = match self.client.get(&url).send().await {
                Ok(resp) => match resp.json::<Value>().await {
                    Ok(v) => v
                        .get("data")
                        .and_then(|d| serde_json::from_value::<Vec<Vec<String>>>(d.clone()).ok())
                        .unwrap_or_default(),
                    Err(e) => {
                        eprintln!("OKX 蜡烛解析失败 {} {}: {}", inst_id, bar, e);
                        return Err(SourceError::Parse(e.to_string()));
                    }
                },
                Err(e) => {
                    eprintln!("OKX 蜡烛请求失败 {} {}: {}", inst_id, bar, e);
                    return Err(SourceError::Network(e.to_string()));
                }
            };
            if rows.is_empty() {
                break;
            }
            for row in &rows {
                if row.len() < 6 {
                    continue;
                }
                let ts = match row[0].parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let open = row[1].parse::<f64>().unwrap_or(0.0);
                let high = row[2].parse::<f64>().unwrap_or(0.0);
                let low = row[3].parse::<f64>().unwrap_or(0.0);
                let close = row[4].parse::<f64>().unwrap_or(0.0);
                let volume = row[5].parse::<f64>().unwrap_or(0.0);
                let date = match chrono::DateTime::from_timestamp_millis(ts) {
                    Some(d) => d.naive_utc(),
                    None => continue,
                };
                all.push(Candle {
                    date,
                    open,
                    high,
                    low,
                    close,
                    volume,
                });
            }
            if let Some(last) = rows.last() {
                before = Some(last[0].clone());
            }
            if all.len() >= needed || rows.len() < page {
                break;
            }
        }

        // 升序并保留末 `needed` 根（最新）。
        all.sort_by(|a, b| a.date.cmp(&b.date));
        if all.len() > needed {
            // `split_off` 返回尾部（最新 needed 根），留在本地的 `all` 中。
            all = all.split_off(all.len() - needed);
        }
        Ok(all)
    }

    /// 拉取单个 OKX 现货交易对的实时 ticker（最新价）。
    /// 成功返回最新价；网络 / 解析失败返回 `None`（不 panic）。
    pub async fn fetch_ticker_price(&self, inst_id: &str) -> Option<f64> {
        let url = format!(
            "https://www.okx.com/api/v5/market/tickers?instType=SPOT&instId={}",
            inst_id
        );
        let rows: Vec<Value> = match self.client.get(&url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(v) => v
                    .get("data")
                    .and_then(|d| serde_json::from_value::<Vec<Value>>(d.clone()).ok())
                    .unwrap_or_default(),
                Err(e) => {
                    eprintln!("OKX ticker 解析失败 {}: {}", inst_id, e);
                    return None;
                }
            },
            Err(e) => {
                eprintln!("OKX ticker 请求失败 {}: {}", inst_id, e);
                return None;
            }
        };
        // 返回匹配该 instId 的第一条记录的最后成交价（last）。
        for row in &rows {
            let id = row.get("instId").and_then(|v| v.as_str()).unwrap_or("");
            if id == inst_id {
                return row.get("last").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok());
            }
        }
        None
    }

    /// 真实市价下单（需凭证）。`sz` 为数量字符串；买入可传 `tgt_ccy="quote_ccy"`
    /// 表示以报价币（USDT）金额下单，卖出传 `None`（基础币数量）。
    /// 返回交易所订单 ID。
    ///
    /// 注意：本方法经由 `tokio::runtime::block_on` 调用（见 `main::place_crypto_live`），
    /// 不要求 future 为 `Send`，因此可以安全使用 okx-rs 的 `Rest` 传输。
    pub async fn place_market_order(
        &self,
        inst_id: &str,
        buy: bool,
        sz: &str,
        tgt_ccy: Option<&str>,
    ) -> anyhow::Result<String> {
        let client = Self::live_client()?;
        let req = PlaceOrderReq {
            inst_id: inst_id.to_string(),
            td_mode: "cash".into(),
            side: if buy { "buy" } else { "sell" }.into(),
            ord_type: "market".into(),
            sz: sz.to_string(),
            px: None,
            tgt_ccy: tgt_ccy.map(|s| s.to_string()),
        };
        let data = client.request(req).await?;
        match data.first() {
            Some(d) if d.s_code.is_empty() || d.s_code == "0" => Ok(d.ord_id.clone()),
            Some(d) => Err(anyhow::anyhow!("OKX 下单失败 {}: {}", d.s_code, d.s_msg)),
            None => Err(anyhow::anyhow!("OKX 下单未返回数据")),
        }
    }
}

impl Default for OkxClient {
    fn default() -> Self {
        Self::new()
    }
}
