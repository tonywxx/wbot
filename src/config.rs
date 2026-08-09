//! 全局配置（费率、整手、K 线参数等）。缺失时回退默认值，绝不 panic。

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
        }
    }
}

/// 当前固定返回默认值（后续可改为读取 config.toml）。
pub fn load_config() -> AppConfig {
    AppConfig::default()
}
