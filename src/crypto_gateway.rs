//! 加密货币实时下单网关。
//!
//! 把原本散落在 `main.rs` 的实盘路由逻辑（`trade_crypto` / `place_crypto_live`）
//! 抽成独立 seam，使其可被单元测（此前躺在 19KB 的 bin 中、零覆盖）。
//!
//! 路由不变：模拟账本（[`crate::sim::crypto_ledger::CryptoLedger`]）**始终**更新；
//! 仅当 `live_trading` 且已配置 OKX 凭证时，额外发起真实市价单（失败仅告警，
//! 不影响模拟账本）。这是 `App::crypto` 与 `OkxClient` 之间的唯一接缝。

use anyhow::Result;
use tokio::runtime::Builder;

use crate::app::{App, View};
use crate::crypto::OkxClient;
use crate::i18n::{traded_fee, tr, Lang};
use crate::signals::Side;

/// 加密货币下单：始终更新模拟账本（CryptoLedger）；若 `live_trading` 且已配置
/// OKX 凭证，额外发起真实市价单（失败仅告警，不影响模拟账本）。
pub fn trade_crypto(app: &mut App, code: &str, price: f64) -> Result<String> {
    let lang = Lang::from_config(&app.config.language);
    let side = if app.active_view == View::Signals {
        app.signals
            .get(app.signal_cursor)
            .map(|s| s.side)
            .unwrap_or(Side::Buy)
    } else {
        Side::Buy
    };
    let buy = side == Side::Buy;
    let fee_rate = app.config.crypto_fee_rate;
    let notional = app.config.crypto_lot_usdt.max(0.0);

    // 卖出前记录持仓数量（模拟成交后持仓会被扣减）。
    let pre_pos = app.crypto.positions.get(code).copied().unwrap_or(0.0);

    let fill = if buy {
        let base_qty = notional / price.max(1e-9);
        app.crypto.place_order(code, true, base_qty, price, fee_rate)?
    } else {
        if pre_pos <= 1e-12 {
            anyhow::bail!("{}", tr("no_position", lang));
        }
        app.crypto.place_order(code, false, pre_pos, price, fee_rate)?
    };

    if app.config.live_trading && OkxClient::has_credentials() {
        let (sz, tgt) = if buy {
            (notional.to_string(), Some("quote_ccy"))
        } else {
            (pre_pos.to_string(), None)
        };
        if let Err(e) = place_crypto_live(code, buy, &sz, tgt) {
            eprintln!("实时下单失败（模拟账本已更新）: {}", e);
        }
    }

    Ok(format!(
        "{} {} @ {:.2}{}",
        if buy {
            tr("traded_buy", lang)
        } else {
            tr("traded_sell", lang)
        },
        code,
        price,
        traded_fee(fill.fee, lang)
    ))
}

/// 在当前线程运行时内发起 OKX 真实市价单（下单为异步，需临时 runtime）。
fn place_crypto_live(
    inst_id: &str,
    buy: bool,
    sz: &str,
    tgt_ccy: Option<&str>,
) -> Result<String> {
    let rt = Builder::new_current_thread().enable_all().build()?;
    let client = OkxClient::new();
    rt.block_on(client.place_market_order(inst_id, buy, sz, tgt_ccy))
}
