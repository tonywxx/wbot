//! 移动平均：SMA（简单）与 EMA（指数）。

use crate::indicators::{Candle, Indicator, IndicatorId, PriceSource};

pub struct Ma {
    id: IndicatorId,
    ema: bool,
}

impl Ma {
    pub fn new(source: PriceSource, period: usize, ema: bool) -> Self {
        Ma {
            id: IndicatorId {
                kind: if ema { "EMA" } else { "MA" }.to_string(),
                source,
                params: vec![period as f64],
                field: None,
            },
            ema,
        }
    }
}

impl Indicator for Ma {
    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let period = self.id.params[0].max(1.0) as usize;
        let vals: Vec<f64> = series.iter().map(|c| self.id.source.value(c)).collect();
        let n = vals.len();
        let mut out = vec![f64::NAN; n];

        if self.ema {
            out = crate::indicators::ema(&vals, period);
        } else {
            for i in (period - 1)..n {
                let s: f64 = vals[i + 1 - period..=i].iter().sum();
                out[i] = s / period as f64;
            }
        }
        out
    }
}
