//! RSI（相对强弱指标），Wilder 平滑，默认周期 14。

use crate::indicators::{Candle, Indicator, IndicatorId, PriceSource};

pub struct Rsi {
    id: IndicatorId,
}

impl Rsi {
    pub fn new(source: PriceSource, period: usize) -> Self {
        Rsi {
            id: IndicatorId {
                kind: "RSI".to_string(),
                source,
                params: vec![period as f64],
                field: None,
            },
        }
    }
}

impl Indicator for Rsi {
    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let period = self.id.params[0].max(1.0) as usize;
        let vals: Vec<f64> = series.iter().map(|c| self.id.source.value(c)).collect();
        let n = vals.len();
        let mut out = vec![f64::NAN; n];
        if n < period + 1 {
            return out;
        }

        let mut gain = 0.0;
        let mut loss = 0.0;
        for i in 1..=period {
            let d = vals[i] - vals[i - 1];
            if d >= 0.0 {
                gain += d;
            } else {
                loss -= d;
            }
        }
        gain /= period as f64;
        loss /= period as f64;

        let rsi0 = if loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + gain / loss)
        };
        out[period] = rsi0;

        for i in (period + 1)..n {
            let d = vals[i] - vals[i - 1];
            let g = if d >= 0.0 { d } else { 0.0 };
            let l = if d < 0.0 { -d } else { 0.0 };
            gain = (gain * (period as f64 - 1.0) + g) / period as f64;
            loss = (loss * (period as f64 - 1.0) + l) / period as f64;
            out[i] = if loss == 0.0 {
                100.0
            } else {
                100.0 - 100.0 / (1.0 + gain / loss)
            };
        }
        out
    }
}
