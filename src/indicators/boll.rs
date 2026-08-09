//! BOLL（布林带），P1 扩展（默认策略未启用，但可被 DSL 引用）。
//! mid = SMA(period)；std = 总体标准差；upper = mid + k*std；lower = mid - k*std。

use crate::indicators::{Candle, Indicator, IndicatorId, PriceSource};

pub struct Boll {
    id: IndicatorId,
}

impl Boll {
    pub fn new(period: usize, k: f64, field: Option<String>) -> Self {
        Boll {
            id: IndicatorId {
                kind: "BOLL".to_string(),
                source: PriceSource::Close,
                params: vec![period as f64, k],
                field,
            },
        }
    }
}

impl Indicator for Boll {
    fn id(&self) -> IndicatorId {
        self.id.clone()
    }

    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let p = &self.id.params;
        let period = p[0].max(1.0) as usize;
        let k = p[1];
        let vals: Vec<f64> = series.iter().map(|c| self.id.source.value(c)).collect();
        let len = vals.len();
        let mut mid = vec![f64::NAN; len];
        let mut upper = vec![f64::NAN; len];
        let mut lower = vec![f64::NAN; len];

        for i in (period - 1)..len {
            let w = &vals[i + 1 - period..=i];
            let mean = w.iter().sum::<f64>() / period as f64;
            let var = w.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
            let sd = var.sqrt();
            mid[i] = mean;
            upper[i] = mean + k * sd;
            lower[i] = mean - k * sd;
        }

        match self.id.field.as_deref() {
            Some("upper") => upper,
            Some("lower") => lower,
            _ => mid,
        }
    }
}
