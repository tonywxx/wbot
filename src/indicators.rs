//! 技术指标引擎（自研，可扩展）。
//!
//! 设计目标：统一 `Indicator` trait + 工厂式 `IndicatorRegistry`，纯 Rust 作用在
//! 自有 `Candle` 序列上，**不**依赖 `akshare::ta`。新增指标只需实现 `Indicator`
//! trait 并在 `build_indicator` 中登记一种 `kind`，即可被策略 DSL 表达式引用。

pub mod ma;
pub mod macd;
pub mod rsi;
pub mod kdj;
pub mod boll;
/// TA-Lib 分发层（adaq-talib 后端，纯 Rust、零 FFI）。由 `tools/gen_ta_dispatch.py` 生成。
pub mod ta_dispatch;
/// TA-Lib 集成层：以统一 DSL 入口对接 TA-Lib 全部函数（adaq-talib 后端），可作为策略 DSL 指标使用。
pub mod ta;

use chrono::NaiveDateTime;

/// 单根 K 线（仅保留指标计算所需字段）。
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub date: NaiveDateTime,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// 取值来源（对应 DSL 中的 `close/open/high/low/volume`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriceSource {
    Close,
    Open,
    High,
    Low,
    Volume,
}

impl PriceSource {
    pub fn value(self, c: &Candle) -> f64 {
        match self {
            PriceSource::Close => c.close,
            PriceSource::Open => c.open,
            PriceSource::High => c.high,
            PriceSource::Low => c.low,
            PriceSource::Volume => c.volume,
        }
    }

    /// DSL 关键词 -> 枚举。
    pub fn from_str(s: &str) -> Option<PriceSource> {
        match s.to_ascii_lowercase().as_str() {
            "close" => Some(PriceSource::Close),
            "open" => Some(PriceSource::Open),
            "high" => Some(PriceSource::High),
            "low" => Some(PriceSource::Low),
            "volume" | "vol" => Some(PriceSource::Volume),
            _ => None,
        }
    }
}

/// 指标唯一标识：决定如何构造计算实例。
#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorId {
    /// 指标种类：
    /// - 自研：`MA|SMA|EMA|MACD|RSI|KDJ|BOLL`
    /// - TA-Lib：以 `TA_` 前缀 + TA-Lib 函数名，如 `TA_RSI`、`TA_BBANDS`、`TA_MACD`
    pub kind: String,
    pub source: PriceSource,
    pub params: Vec<f64>,
    /// 输出字段选择：`MACD.dif/.dea/.hist`、`KDJ.k/.d/.j`、`BOLL.mid/.upper/.lower`，
    /// 以及 TA-Lib 多输出函数（如 `BBANDS` 的 `.0/.1/.2` 或输出名）。默认取首个输出。
    pub field: Option<String>,
}

/// 指标 trait：输入等长 `Candle` 序列，输出等长 `f64` 序列。
/// 前导数据不足的位置用 `f64::NAN` 表示（cross 比较时 NaN 视为未触发）。
pub trait Indicator: Send + Sync {
    fn eval(&self, series: &[Candle]) -> Vec<f64>;
    #[allow(dead_code)]
    fn id(&self) -> IndicatorId;
}

/// 指数移动平均（公共实现，供 MACD 复用）。
/// 前 `period-1` 个位置为 NaN，之后以首段 SMA 作为种子递推。
pub(crate) fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![f64::NAN; n];
    if period == 0 || n == 0 {
        return out;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev = f64::NAN;
    for i in 0..n {
        let v = values[i];
        if i + 1 < period {
            continue;
        }
        if prev.is_nan() {
            let s: f64 = values[i + 1 - period..=i].iter().sum();
            prev = s / period as f64;
        } else {
            prev = v * k + prev * (1.0 - k);
        }
        out[i] = prev;
    }
    out
}

/// 由 `IndicatorId` 即时构造对应指标实例（工厂）。
/// 新增指标类型：在此 `match` 中增加分支即可。
pub fn build_indicator(id: &IndicatorId) -> Option<Box<dyn Indicator>> {
    let src = id.source;
    let p = &id.params;

    // TA-Lib 函数：以 `TA_` 前缀路由到 TA 集成层（函数名 = 去掉前缀后的部分）。
    if let Some(ta_name) = id.kind.strip_prefix("TA_") {
        return ta::TaIndicator::try_new(
            ta_name,
            id.params.clone(),
            id.field.clone(),
            id.source,
        )
        .map(|t| Box::new(t) as Box<dyn Indicator>);
    }

    match id.kind.as_str() {
        "MA" | "SMA" => {
            let period = p.first().copied().unwrap_or(5.0) as usize;
            Some(Box::new(ma::Ma::new(src, period, false)))
        }
        "EMA" => {
            let period = p.first().copied().unwrap_or(5.0) as usize;
            Some(Box::new(ma::Ma::new(src, period, true)))
        }
        "MACD" => {
            let fast = p.first().copied().unwrap_or(12.0) as usize;
            let slow = p.get(1).copied().unwrap_or(26.0) as usize;
            let signal = p.get(2).copied().unwrap_or(9.0) as usize;
            Some(Box::new(macd::Macd::new(src, fast, slow, signal, id.field.clone())))
        }
        "RSI" => {
            let period = p.first().copied().unwrap_or(14.0) as usize;
            Some(Box::new(rsi::Rsi::new(src, period)))
        }
        "KDJ" => {
            let n = p.first().copied().unwrap_or(9.0) as usize;
            let ks = p.get(1).copied().unwrap_or(3.0) as usize;
            let ds = p.get(2).copied().unwrap_or(3.0) as usize;
            Some(Box::new(kdj::Kdj::new(n, ks, ds, id.field.clone())))
        }
        "BOLL" => {
            let period = p.first().copied().unwrap_or(20.0) as usize;
            let k = p.get(1).copied().unwrap_or(2.0);
            Some(Box::new(boll::Boll::new(period, k, id.field.clone())))
        }
        _ => None,
    }
}

/// 指标注册表（工厂式）。调用方无需预先注册，直接按 `IndicatorId` 求值。
pub struct IndicatorRegistry;

impl IndicatorRegistry {
    pub fn new() -> Self {
        IndicatorRegistry
    }

    /// 按 id 求指标序列；未知 kind 返回 `None`。
    pub fn eval(&self, id: &IndicatorId, series: &[Candle]) -> Option<Vec<f64>> {
        build_indicator(id).map(|ind| ind.eval(series))
    }
}

impl Default for IndicatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷：对一段序列求某指标的末值（用于比较/展示）。
pub fn last_value(reg: &IndicatorRegistry, id: &IndicatorId, series: &[Candle]) -> Option<f64> {
    reg.eval(id, series)
        .and_then(|v| v.last().copied())
        .filter(|x| !x.is_nan())
}
