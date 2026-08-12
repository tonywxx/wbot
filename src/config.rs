//! 全局配置（费率、整手、K 线参数等）。缺失时回退默认值，绝不 panic。
//!
//! 配置来源：优先读取工作目录下的 `config.toml`，未知 / 缺失字段回退 [`AppConfig::default`]。
//! 新增字段：`language`（界面语言 en/zh，默认 en）、`crypto_enabled`（是否启用加密货币，
//! 默认 true）、`live_trading`（是否启用 OKX 真实下单，默认 false，需环境变量凭证）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 默认作用范围（字符串形式；"watchlist" 或逗号分隔代码）。
    pub default_scope: String,
    /// 手续费率（双边）。
    pub commission: f64,
    /// 印花税率（仅卖出）。
    pub stamp_tax: f64,
    /// 每手股数。
    pub lot_size: i64,
    /// 自动下单开关（P1，默认关闭；P0 仅手动）。
    pub auto_trade: bool,
    /// K 线复权方式。
    pub kline_adjust: String,
    /// K 线保留根数。
    pub kline_count: usize,
    /// 分钟 K 线刷新间隔（秒）。
    pub intraday_refresh: u64,
    /// 是否发送桌面通知。
    pub notify_enabled: bool,
    /// 同一 (标的, 规则) 通知冷却时间（秒），避免重复弹窗。
    pub notify_cooldown: u64,
    /// 界面语言："en"（默认，英文）或 "zh"（简体中文）。
    pub language: String,
    /// 是否启用加密货币（OKX）数据源与交易支持。
    pub crypto_enabled: bool,
    /// 是否启用 OKX 真实下单（需 OKX_API_KEY / OKX_API_SECRET / OKX_PASSPHRASE 环境变量）。
    /// 默认 false：加密货币下单仅走模拟账户（CryptoLedger）。
    pub live_trading: bool,
    /// 加密货币单次买入预算（USDT 计价），用于把预算换算为基础币数量。默认 1000.0。
    pub crypto_lot_usdt: f64,
    /// 加密货币单边手续费率。默认 0.001（0.1%）。
    pub crypto_fee_rate: f64,
    /// 涨（up）配色：命名色或 #rrggbb，默认 green（覆盖原硬编码的中国习惯 涨=红）。
    pub up_color: String,
    /// 跌（down）配色：命名色或 #rrggbb，默认 red。
    pub down_color: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_scope: "watchlist".to_string(),
            commission: 0.0003,
            stamp_tax: 0.0005,
            lot_size: 100,
            auto_trade: false,
            kline_adjust: "qfq".to_string(),
            kline_count: 250,
            intraday_refresh: 120,
            notify_enabled: true,
            notify_cooldown: 300,
            // 界面默认英文显示；用户可在 config.toml 将 language 设为 "zh"。
            language: "en".to_string(),
            crypto_enabled: true,
            live_trading: false,
            crypto_lot_usdt: 1000.0,
            crypto_fee_rate: 0.001,
            up_color: "green".to_string(),
            down_color: "red".to_string(),
        }
    }
}

/// `config.toml` 的原始形态：所有字段可选，缺失即回退默认值。
#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    default_scope: Option<String>,
    commission: Option<f64>,
    stamp_tax: Option<f64>,
    lot_size: Option<i64>,
    auto_trade: Option<bool>,
    kline_adjust: Option<String>,
    kline_count: Option<usize>,
    intraday_refresh: Option<u64>,
    notify_enabled: Option<bool>,
    notify_cooldown: Option<u64>,
    language: Option<String>,
    crypto_enabled: Option<bool>,
    live_trading: Option<bool>,
    crypto_lot_usdt: Option<f64>,
    crypto_fee_rate: Option<f64>,
    up_color: Option<String>,
    down_color: Option<String>,
}

/// 读取 `config.toml`（若存在）并合并到默认值之上。
/// 文件缺失 / 损坏时安全回退到 [`AppConfig::default`]，绝不 panic。
pub fn load_config() -> AppConfig {
    let mut cfg = AppConfig::default();
    let text = match std::fs::read_to_string("config.toml") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return cfg,
    };
    let raw: RawConfig = match toml::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("config.toml 解析失败，使用默认配置: {}", e);
            return cfg;
        }
    };
    if let Some(v) = raw.default_scope {
        cfg.default_scope = v;
    }
    if let Some(v) = raw.commission {
        cfg.commission = v;
    }
    if let Some(v) = raw.stamp_tax {
        cfg.stamp_tax = v;
    }
    if let Some(v) = raw.lot_size {
        cfg.lot_size = v;
    }
    if let Some(v) = raw.auto_trade {
        cfg.auto_trade = v;
    }
    if let Some(v) = raw.kline_adjust {
        cfg.kline_adjust = v;
    }
    if let Some(v) = raw.kline_count {
        cfg.kline_count = v;
    }
    if let Some(v) = raw.intraday_refresh {
        cfg.intraday_refresh = v;
    }
    if let Some(v) = raw.notify_enabled {
        cfg.notify_enabled = v;
    }
    if let Some(v) = raw.notify_cooldown {
        cfg.notify_cooldown = v;
    }
    if let Some(v) = raw.language {
        // 归一化为小写后再存储，便于后续比对。
        cfg.language = v.trim().to_ascii_lowercase();
    }
    if let Some(v) = raw.crypto_enabled {
        cfg.crypto_enabled = v;
    }
    if let Some(v) = raw.live_trading {
        cfg.live_trading = v;
    }
    if let Some(v) = raw.crypto_lot_usdt {
        cfg.crypto_lot_usdt = v;
    }
    if let Some(v) = raw.crypto_fee_rate {
        cfg.crypto_fee_rate = v;
    }
    if let Some(v) = raw.up_color {
        cfg.up_color = v.trim().to_string();
    }
    if let Some(v) = raw.down_color {
        cfg.down_color = v.trim().to_string();
    }
    cfg
}
