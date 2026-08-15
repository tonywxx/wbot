//! TA-Lib 集成层（adaq-talib 纯 Rust 后端，无 FFI）。
//!
//! 本模块不再依赖 C TA-Lib / FFI，所有指标计算经由 `ta_dispatch`（adaq-talib 后端，
//! 纯 Rust、零 FFI、零依赖）完成，覆盖 TA-Lib 0.7.1 的全部 161 个函数。
//!
//! 设计要点：
//! - `call_ta_func` 是核心：给定 TA-Lib 函数名（不带 `TA_` 前缀）、K 线序列、价格来源、
//!   可选参数与输出字段，返回与输入等长（前导为 `NAN`）的 `f64` 序列。
//! - `TaIndicator` 实现 `Indicator` trait，使任意 TA-Lib 函数可直接在已有策略
//!   DSL 中以 `TA_<FUNC>(src, p1, p2, ...)[.field]` 形式引用。
//! - 价格类输入：单输入函数使用 `source` 指定的价格序列（close/open/high/low/volume）；
//!   多输入函数（如 `STOCH`、`BOP`、`CDL_*`）按各函数自身输入掩码取用 OHLCV，
//!   忽略 `source`。
//!
//! TA-Lib integration (adaq-talib backend, zero FFI).
//! Every TA-Lib function is computed by `ta_dispatch` (the adaq-talib backend) so that
//! *all* TA-Lib functions remain reachable from a single generic entry point.

use crate::indicators::{Candle, Indicator, PriceSource};
use super::ta_dispatch;

// 重新导出分发层提供的元信息与自检 API（外部以不带前缀的 TA-Lib 函数名调用）。
pub use super::ta_dispatch::{TaFuncMeta, TaOptInput};

/// 给函数名补上 `TA_` 前缀（幂等：已带前缀则不重复添加）。
fn prefixed(name: &str) -> String {
    if name.starts_with("TA_") {
        name.to_string()
    } else {
        format!("TA_{}", name)
    }
}

/// 判断某 TA-Lib 函数名是否被 adaq-talib 支持。
/// 接受带或不带 `TA_` 前缀的名称。
pub fn ta_function_exists(name: &str) -> bool {
    ta_dispatch::ta_function_exists(&prefixed(name))
}

/// 取得某 TA-Lib 函数的全部输出字段名（用于 `.field` 选择）。
/// 接受带或不带 `TA_` 前缀的名称。
pub fn ta_output_names(name: &str) -> Option<Vec<String>> {
    ta_dispatch::ta_output_names(&prefixed(name))
}

/// 列出随 TA-Lib 0.7.1 提供的全部 161 个函数（不带前缀的名称, 分组）。
/// 用于文档生成与自检。
pub fn list_all_functions() -> Vec<(String, String)> {
    ta_dispatch::list_all_functions()
        .into_iter()
        .map(|(n, g)| {
            let short = n.strip_prefix("TA_").unwrap_or(&n).to_string();
            (short, g)
        })
        .collect()
}

/// 取得某 TA-Lib 函数的完整元信息（可选参数含默认值/范围、输出字段名与类型）。
/// 接受带或不带 `TA_` 前缀的名称。
pub fn ta_meta(name: &str) -> Option<TaFuncMeta> {
    ta_dispatch::ta_meta(&prefixed(name))
}

/// 通用调用入口：按函数名计算某输出序列。
///
/// - `name`    : TA-Lib 函数名（如 "RSI"、"BBANDS"、"MACD"，可带或不带 `TA_` 前缀）。
/// - `series`  : 输入 K 线（OHLCV）。
/// - `source`  : 单输入函数使用的价格来源（close/open/high/low/volume）。
/// - `params`  : 可选参数（时间周期 / MAType / 偏差等），不足时取 adaq-talib 默认值。
/// - `field`   : 多输出函数的输出字段选择（如 "upper"/"hist"，或整数字符串取第 N 个输出）；
///   默认取首个输出。
///
/// 返回与 `series` 等长的 `Vec<f64>`，前导无效区为 `NAN`。
pub fn call_ta_func(
    name: &str,
    series: &[Candle],
    source: PriceSource,
    params: &[f64],
    field: Option<&str>,
) -> Option<Vec<f64>> {
    ta_dispatch::call_adaq(&prefixed(name), series, source, params, field)
}

/// 将 `field`（None / 整数字符串 / 输出名）解析为 `call_adaq` 接受的字段名。
/// 单输出函数忽略字段名，仍返回原 `field`（被分发层忽略）。
fn field_name_for(name: &str, field: Option<&str>) -> Option<String> {
    match field {
        None => None,
        Some(f) => {
            if let Ok(idx) = f.parse::<usize>() {
                // 数字 -> 对应输出名（缺省回退首个）
                ta_output_names(name).and_then(|v| v.into_iter().nth(idx))
            } else {
                Some(f.to_string())
            }
        }
    }
}

/// 可作为策略 DSL 指标的 TA-Lib 封装。
pub struct TaIndicator {
    name: String,
    source: PriceSource,
    params: Vec<f64>,
    field: Option<String>,
}

impl TaIndicator {
    /// 构造；若函数名不存在返回 `None`（DSL 求值将得到 NAN -> 不触发）。
    pub fn try_new(
        name: &str,
        params: Vec<f64>,
        field: Option<String>,
        source: PriceSource,
    ) -> Option<Self> {
        if ta_function_exists(name) {
            Some(TaIndicator {
                name: name.to_string(),
                source,
                params,
                field,
            })
        } else {
            None
        }
    }
}

impl Indicator for TaIndicator {
    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let field = field_name_for(&self.name, self.field.as_deref());
        call_ta_func(
            &self.name,
            series,
            self.source,
            &self.params,
            field.as_deref(),
        )
        .unwrap_or_else(|| vec![f64::NAN; series.len()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NaN-aware vector equality: equal where both are finite-and-equal, or both NaN at the
    /// same index. Needed because `assert_eq!` on `Vec<f64>` fails at leading-NaN positions
    /// (TA-Lib-style indicators emit `NaN` before their lookback window).
    fn vec_eq_nan(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|(x, y)| x == y || (x.is_nan() && y.is_nan()))
    }

    #[test]
    fn ta_exists_check() {
        assert!(ta_function_exists("RSI"));
        assert!(ta_function_exists("BBANDS"));
        assert!(!ta_function_exists("NOT_A_REAL_FUNC"));
    }

    #[test]
    fn ta_list_nonempty() {
        let all = list_all_functions();
        assert_eq!(all.len(), 161, "TA-Lib should expose exactly 161 functions");
    }

    /// 生成确定性的逼真 OHLCV 序列（high>=low、high>=open/close、low<=open/close），
    /// 以便对输入有约束的指标（SAR/ATR/ADX/CDL_* 等）也能正常计算。
    fn synthetic_candles(n: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(n);
        let mut price = 100.0f64;
        for i in 0..n {
            // 简单确定性游走
            let drift = ((i as f64 * 0.13).sin() * 1.5) + ((i as f64 * 0.037).cos() * 0.8);
            let open = price;
            let close = (price + drift).max(1.0);
            let hi_noise = 1.0 + (i % 3) as f64 * 0.4;
            let lo_noise = 1.0 + (i % 5) as f64 * 0.3;
            let high = open.max(close) + hi_noise;
            let low = open.min(close) - lo_noise;
            let volume = 1000.0 + (i as f64 * 7.0).abs();
            candles.push(Candle {
                date: chrono::NaiveDateTime::default(),
                open,
                high,
                low,
                close,
                volume,
            });
            price = close;
        }
        candles
    }

    /// 对每个 TA_* 函数调用，断言返回 `Some` 且与输入等长（前导为 NAN）。
    /// 这是“全部 161 个指标均经 adaq-talib 成功计算”的核心自检。
    #[test]
    fn every_ta_function_computes() {
        let candles = synthetic_candles(260);
        let mut failures = Vec::new();
        for (name, _group) in list_all_functions() {
            let out = call_ta_func(&name, &candles, PriceSource::Close, &[], None);
            match out {
                None => failures.push(format!("{name}: returned None")),
                Some(v) => {
                    if v.len() != candles.len() {
                        failures.push(format!("{name}: len {} != {}", v.len(), candles.len()));
                    }
                }
            }
        }
        assert!(
            failures.is_empty(),
            "TA function failures ({}):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// 参数与多输出字段选择的正确性抽查。
    #[test]
    fn ta_param_and_field_selection() {
        let candles = synthetic_candles(260);

        // 单输出 + 显式参数
        let rsi = call_ta_func("RSI", &candles, PriceSource::Close, &[14.0], None).unwrap();
        assert_eq!(rsi.len(), candles.len());
        assert!(rsi.iter().all(|x| x.is_nan() || (0.0..=100.0).contains(x)));

        // 多输出：默认（首个输出 = upper）vs 具名字段应相等
        let bbands_def = call_ta_func("BBANDS", &candles, PriceSource::Close, &[20.0, 2.0, 2.0], None).unwrap();
        let bbands_mid = call_ta_func("BBANDS", &candles, PriceSource::Close, &[20.0, 2.0, 2.0], Some("middle")).unwrap();
        let bbands_up = call_ta_func("BBANDS", &candles, PriceSource::Close, &[20.0, 2.0, 2.0], Some("upper")).unwrap();
        assert!(vec_eq_nan(&bbands_def, &bbands_up), "BBANDS default must equal upper band");
        assert!(bbands_up.iter().zip(&bbands_mid).all(|(u, m)| (u.is_nan() && m.is_nan()) || u >= m), "upper must be >= middle");

        // 数字字段等价于具名字段
        let bbands_0 = call_ta_func("BBANDS", &candles, PriceSource::Close, &[20.0, 2.0, 2.0], Some("0")).unwrap();
        assert!(vec_eq_nan(&bbands_0, &bbands_up), "BBANDS field '0' must equal upper band");

        // MACD 三输出均存在、等长
        for f in ["macd", "signal", "hist"] {
            let v = call_ta_func("MACD", &candles, PriceSource::Close, &[12.0, 26.0, 9.0], Some(f)).unwrap();
            assert_eq!(v.len(), candles.len());
        }
    }
}
