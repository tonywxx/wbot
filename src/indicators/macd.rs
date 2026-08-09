//! MACD：DIF = EMA(fast) - EMA(slow)，DEA = EMA(DIF, signal)，HIST = 2*(DIF-DEA)。
//! 通过 `IndicatorId.field` 取 `dif` / `dea` / `hist`。

use crate::indicators::{ema, Candle, Indicator, IndicatorId, PriceSource};

pub struct Macd {
    id: IndicatorId,
}

impl Macd {
    pub fn new(source: PriceSource, fast: usize, slow: usize, signal: usize, field: Option<String>) -> Self {
        Macd {
            id: IndicatorId {
                kind: "MACD".to_string(),
                source,
                params: vec![fast as f64, slow as f64, signal as f64],
                field,
            },
        }
    }
}

impl Indicator for Macd {
    fn id(&self) -> IndicatorId {
        self.id.clone()
    }

    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let p = &self.id.params;
        let fast = p[0].max(1.0) as usize;
        let slow = p[1].max(1.0) as usize;
        let signal = p[2].max(1.0) as usize;
        let vals: Vec<f64> = series.iter().map(|c| self.id.source.value(c)).collect();
        let n = vals.len();
        let ema_fast = ema(&vals, fast);
        let ema_slow = ema(&vals, slow);

        let mut dif = vec![f64::NAN; n];
        for i in 0..n {
            if !ema_fast[i].is_nan() && !ema_slow[i].is_nan() {
                dif[i] = ema_fast[i] - ema_slow[i];
            }
        }

        // DEA = EMA of DIF（跳过 DIF 的 NaN 前导）
        let mut dea = vec![f64::NAN; n];
        let k = 2.0 / (signal as f64 + 1.0);
        let mut prev = f64::NAN;
        let mut seen: i64 = 0;
        let mut sum = 0.0;
        for i in 0..n {
            let v = dif[i];
            if v.is_nan() {
                continue;
            }
            seen += 1;
            sum += v;
            if seen < signal as i64 {
                continue;
            }
            if prev.is_nan() {
                prev = sum / signal as f64;
            } else {
                prev = v * k + prev * (1.0 - k);
            }
            dea[i] = prev;
        }

        let field = self.id.field.as_deref().unwrap_or("dif");
        let mut out = vec![f64::NAN; n];
        for i in 0..n {
            if dif[i].is_nan() || dea[i].is_nan() {
                continue;
            }
            out[i] = match field {
                "dea" => dea[i],
                "hist" => 2.0 * (dif[i] - dea[i]),
                _ => dif[i],
            };
        }
        out
    }
}
