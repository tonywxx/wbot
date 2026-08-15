//! Realtime HTTP infrastructure shared by the East Money / Yahoo providers:
//! ambient proxy detection, the shared `reqwest` client, the low-level
//! `http_text` GET, and the generic `fetch_json_fallback` host-rotation helper.

use reqwest::{Client, Proxy};
use reqwest::header::HeaderValue;
use serde_json::Value;
use std::time::Duration;

/// 单次请求失败后的重试次数（仍由上层 `fetch_json_fallback` 的 host 轮询兜底）。
const HTTP_MAX_ATTEMPTS: u32 = 2;
/// 重试之间的退避（毫秒）。
const HTTP_RETRY_BACKOFF_MS: u64 = 250;

/// Read an outbound HTTP proxy from the standard env vars (`HTTPS_PROXY`,
/// `https_proxy`, `HTTP_PROXY`, `http_proxy`). Returns `None` when unset, so
/// the client talks directly on hosts without a proxy. (`Proxy::from_env` is
/// feature-gated off in this reqwest build; we replicate it with `Proxy::all`,
/// which is always available.)
fn env_proxy() -> Option<Proxy> {
    let val = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .ok()?;
    Proxy::all(val).ok()
}

/// Shared `reqwest` client for realtime endpoints.
///
/// Applies the ambient HTTP proxy (if any) and a desktop `User-Agent` East
/// Money / Yahoo accept. Splits timeouts: a 5s `connect_timeout` fails fast on
/// an unreachable host, while the 15s overall `timeout` bounds a stalled read
/// so a refresh cycle can't hang. (The `Referer` header is added per-request
/// because `ClientBuilder::header` is feature-gated off in this reqwest build;
/// `RequestBuilder::header` is always available.)
pub(crate) fn realtime_http_client() -> Client {
    let mut builder = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );
    if let Some(p) = env_proxy() {
        builder = builder.proxy(p);
    }
    builder
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// GET `url` and return the response body as `String`, or `None` on any
/// transport / read error. `referer` (when `Some`) is attached as a `Referer`
/// header — required by East Money, harmless elsewhere.
///
/// 对瞬时网络抖动做 `HTTP_MAX_ATTEMPTS` 次重试（指数退避由固定 `HTTP_RETRY_BACKOFF_MS`
/// 近似）；上层 `fetch_json_fallback` 仍会按 host 轮询做最终兜底，二者不冲突。
async fn http_text(http: &Client, url: &str, referer: Option<&str>) -> Option<String> {
    for attempt in 0..HTTP_MAX_ATTEMPTS {
        let mut req = http.get(url);
        if let Some(r) = referer
            && let Ok(hv) = HeaderValue::from_str(r)
        {
            req = req.header("Referer", hv);
        }
        match req.send().await {
            Ok(resp) => return resp.text().await.ok(),
            Err(_) => {
                if attempt + 1 < HTTP_MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(HTTP_RETRY_BACKOFF_MS)).await;
                }
            }
        }
    }
    None
}

/// 按 `hosts` 顺序逐个尝试：首个返回可用 body、且 `parse` 给出 `Some(T)` 的主机
/// 即作为结果返回；若所有主机都失败或 `parse` 全部拒绝（如空数组）则返回 `None`。
///
/// 把所有 provider 复用的「遍历 host → `http_text` → `from_str` → `parse`」循环
/// 收敛到一处（`em_fetch_quote` / `em_fetch_quotes_batch` / `em_fetch_board` /
/// `yahoo_fetch_quote` / `yahoo_fetch_quotes_batch` 此前各复制了一遍）。
pub(crate) async fn fetch_json_fallback<T>(
    http: &Client,
    hosts: &[&str],
    referer: Option<&str>,
    build_url: impl Fn(&str) -> String,
    parse: impl Fn(&Value) -> Option<T>,
) -> Option<T> {
    for host in hosts {
        let url = build_url(host);
        if let Some(txt) = http_text(http, &url, referer).await
            && let Ok(v) = serde_json::from_str::<Value>(&txt)
            && let Some(x) = parse(&v)
        {
            return Some(x);
        }
    }
    None
}
