//! KDJ（随机指标），P1 扩展（默认策略未启用，但可被 DSL 引用）。
//! RSV = (close - low_n)/(high_n - low_n) * 100；
//! K = (m-1)/m * prevK + 1/m * RSV；D = (m-1)/m * prevD + 1/m * K；J = 3K - 2D。

use crate::indicators::{Candle, Indicator, IndicatorId, PriceSource};

pub struct Kdj {
    id: IndicatorId,
}

impl Kdj {
    pub fn new(n: usize, k_smooth: usize, d_smooth: usize, field: Option<String>) -> Self {
        Kdj {
            id: IndicatorId {
                kind: "KDJ".to_string(),
                source: PriceSource::Close,
                params: vec![n as f64, k_smooth as f64, d_smooth as f64],
                field,
            },
        }
    }
}

impl Indicator for Kdj {
    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let p = &self.id.params;
        let n = p[0].max(1.0) as usize;
        let ks = p[1].max(2.0);
        let ds = p[2].max(2.0);
        let len = series.len();
        let mut k_out = vec![f64::NAN; len];
        let mut d_out = vec![f64::NAN; len];
        if len < n {
            return default_field(&self.id.field, &k_out, &d_out);
        }

        let mut prev_k = 50.0;
        let mut prev_d = 50.0;
        for i in (n - 1)..len {
            let mut hh = f64::NEG_INFINITY;
            let mut ll = f64::INFINITY;
            for c in &series[i + 1 - n..=i] {
                if c.high > hh {
                    hh = c.high;
                }
                if c.low < ll {
                    ll = c.low;
                }
            }
            let rsv = if (hh - ll).abs() < 1e-12 {
                50.0
            } else {
                (series[i].close - ll) / (hh - ll) * 100.0
            };
            let k = (ks - 1.0) / ks * prev_k + (1.0 / ks) * rsv;
            let d = (ds - 1.0) / ds * prev_d + (1.0 / ds) * k;
            k_out[i] = k;
            d_out[i] = d;
            prev_k = k;
            prev_d = d;
        }

        default_field(&self.id.field, &k_out, &d_out)
    }
}

fn default_field(field: &Option<String>, k: &[f64], d: &[f64]) -> Vec<f64> {
    match field.as_deref() {
        Some("d") => d.to_vec(),
        Some("j") => k
            .iter()
            .zip(d.iter())
            .map(|(kk, dd)| 3.0 * kk - 2.0 * dd)
            .collect(),
        _ => k.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn candle(close: f64, high: f64, low: f64) -> Candle {
        Candle {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            open: close,
            high,
            low,
            close,
            volume: 0.0,
        }
    }

    fn series(n: usize) -> Vec<Candle> {
        (0..n).map(|i| candle(100.0 + i as f64, 102.0 + i as f64, 98.0 + i as f64)).collect()
    }

    #[test]
    fn eval_does_not_underflow_at_window_boundary() {
        // n == len: the very first window start is (n-1) - (n-1) == 0.
        let k = Kdj::new(3, 3, 3, None);
        let out = k.eval(&series(3));
        assert_eq!(out.len(), 3);
        assert!(out[2].is_finite());

        // n + 1: boundary + one extra candle.
        let out2 = k.eval(&series(4));
        assert_eq!(out2.len(), 4);
        assert!(out2[3].is_finite());

        // n == 1 (single-candle windows) must not underflow either.
        let k1 = Kdj::new(1, 3, 3, None);
        let out1 = k1.eval(&series(5));
        assert_eq!(out1.len(), 5);
        assert!(out1[0].is_finite());

        // shorter than n returns NaNs without panicking.
        let out_short = k.eval(&series(2));
        assert!(out_short.iter().all(|v| v.is_nan()));
    }
}
