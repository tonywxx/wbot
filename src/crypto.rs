//! 加密货币（OKX）集成层。
//!
//! - **公开行情**：`/market/candles` 历史 K 线拉取，直接用 `reqwest`（Send + Sync）
//!   实现，映射为自有 `Candle`。之所以不用 okx-rs 的 `request` 传输来做行情，是因为
//!   okx-rs 的 `Options` 持有 `Arc<dyn OKXEnv>`（`OKXEnv` 未标注 `Send + Sync`），
//!   导致其 `request` future 不是 `Send`，无法满足本项目 `MarketSource` trait 要求的
//!   `Send` future（数据源在多线程 tokio 运行时中被 `spawn`）。因此公开行情走 reqwest，
//!   既保证 `Send`，又避免引入系统 OpenSSL 依赖。
//! - **真实下单**：`/trade/order` 市价单，复用 okx-rs 的 `Rest` 传输与签名管线
//!   （需 `OKX_API_KEY` / `OKX_API_SECRET` / `OKX_PASSPHRASE` 环境变量；仅在
//!   `live_trading` 为真时启用）。下单路径通过 `tokio::runtime::block_on` 执行，
//!   不要求 future 为 `Send`，故可安全使用 okx-rs。
//! - **模拟加密账户**：`CryptoLedger` —— 以 USDT 计价的现金 + 基础币持仓，
//!   无需真实凭证即可在 TUI 中模拟加密货币买卖。

use std::collections::HashMap;

use okx_rs::api::{Options, Production, Rest};
use okx_rs::api::v5::Request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::indicators::Candle;

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
    /// 升序、截断到 `limit` 根。网络 / 解析失败时返回已获取的部分（不 panic）。
    pub async fn fetch_candles(&self, inst_id: &str, bar: &str, limit: usize) -> Vec<Candle> {
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
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("OKX 蜡烛请求失败 {} {}: {}", inst_id, bar, e);
                    break;
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
        all
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

/// 模拟加密货币账户：USDT 现金 + 基础币持仓（含均价成本）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CryptoLedger {
    /// 可用 USDT。
    pub usdt: f64,
    /// 各合约（如 BTC-USDT）的基础币持仓数量。
    pub positions: HashMap<String, f64>,
    /// 各合约的加权平均成本价（USDT），用于实现盈亏计算。
    pub avg_cost: HashMap<String, f64>,
}

/// 模拟加密货币成交结果。
#[derive(Debug, Clone)]
pub struct CryptoFill {
    pub fee: f64,
    pub cash_delta: f64,
    pub realized_pnl: f64,
}

impl CryptoLedger {
    /// 新建模拟账户，初始 `usdt` 现金。
    pub fn new(usdt: f64) -> Self {
        CryptoLedger {
            usdt,
            positions: HashMap::new(),
            avg_cost: HashMap::new(),
        }
    }

    /// 账户总权益（USDT）= 现金 + Σ(持仓数量 × 最新价)。
    pub fn total_value(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut total = self.usdt;
        for (inst, qty) in &self.positions {
            let p = prices.get(inst).copied().unwrap_or(0.0);
            total += qty * p;
        }
        total
    }

    /// 模拟买入 / 卖出。`base_qty` 为基础币数量；`price` 为成交价（USDT）。
    /// `fee_rate` 为单边费率。返回成交明细；现金 / 持仓不足则拒绝。
    pub fn place_order(
        &mut self,
        inst_id: &str,
        buy: bool,
        base_qty: f64,
        price: f64,
        fee_rate: f64,
    ) -> anyhow::Result<CryptoFill> {
        if base_qty <= 0.0 {
            anyhow::bail!("数量必须为正");
        }
        let fee_rate = fee_rate.max(0.0);
        if buy {
            let cost = base_qty * price;
            let fee = cost * fee_rate;
            let total = cost + fee;
            if total > self.usdt + 1e-9 {
                anyhow::bail!(
                    "USDT 不足：需要 {:.2}，可用 {:.2}",
                    total,
                    self.usdt
                );
            }
            self.usdt -= total;
            let pos = self.positions.entry(inst_id.to_string()).or_insert(0.0);
            let prev_qty = *pos;
            let prev_avg = self.avg_cost.get(inst_id).copied().unwrap_or(price);
            let new_qty = prev_qty + base_qty;
            let new_avg = (prev_avg * prev_qty + price * base_qty) / new_qty;
            *pos = new_qty;
            self.avg_cost.insert(inst_id.to_string(), new_avg);
            Ok(CryptoFill {
                fee,
                cash_delta: -total,
                realized_pnl: 0.0,
            })
        } else {
            let pos = self
                .positions
                .get_mut(inst_id)
                .ok_or_else(|| anyhow::anyhow!("无持仓：{}", inst_id))?;
            if *pos < base_qty - 1e-12 {
                anyhow::bail!("持仓不足：持有 {:.6}，欲卖 {:.6}", pos, base_qty);
            }
            let avg = self.avg_cost.get(inst_id).copied().unwrap_or(price);
            let proceeds = base_qty * price;
            let fee = proceeds * fee_rate;
            let realized = (price - avg) * base_qty - fee;
            *pos -= base_qty;
            if *pos <= 1e-12 {
                self.positions.remove(inst_id);
                self.avg_cost.remove(inst_id);
            }
            self.usdt += proceeds - fee;
            Ok(CryptoFill {
                fee,
                cash_delta: proceeds - fee,
                realized_pnl: realized,
            })
        }
    }
}
