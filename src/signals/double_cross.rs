//! 双金叉 / 双死叉回踩形态检测（顺序状态机，非点状 DSL）。
//!
//! 以 EMA(fast) 与 EMA(slow) 在收盘序列上的交叉为基础，识别「两次同向交叉之间
//! 出现一次反向交叉、且回踩（反抽）不破前低（前高）」的多头（空头）成本基础形态。
//!
//! - `bullish = true` 时检测**双金叉回踩买入**：
//!   ① 末根发生 EMA5 上穿 EMA10（第二次金叉）；
//!   ② 在此之前存在一次 EMA5 下穿 EMA10（中间的死叉回踩）；
//!   ③ 再之前存在一次 EMA5 上穿 EMA10（第一次金叉）；
//!   ④ 末根收盘价 > EMA10（成本已高于慢线）；
//!   ⑤ 回踩段最低价 > 第一次金叉之前的全部最低价（不破前低）；
//!   ⑥ 中间死叉处收盘价 < EMA10（回踩时成本低于慢线）。
//! - `bullish = false` 时镜像为**双死叉反抽卖出**。

use crate::indicators::ema;

/// 检测双金叉/双死叉回踩形态。
///
/// `close/high/low` 必须等长且按时间升序。`fast`/`slow` 为 EMA 周期。
/// `higher_low = true` 时才校验「不破前低 / 不过前高」。
/// `bullish = true` 检测买入形态，`false` 检测卖出形态。
pub fn detect_double_golden_cross(
    close: &[f64],
    high: &[f64],
    low: &[f64],
    fast: usize,
    slow: usize,
    higher_low: bool,
    bullish: bool,
) -> bool {
    let n = close.len();
    if n < slow + 3 {
        return false;
    }
    let ef = ema(close, fast);
    let es = ema(close, slow);

    // 同向交叉（bullish=金叉，bearish=死叉）与反向交叉，作为函数指针便于分支选择。
    fn is_golden(i: usize, ef: &[f64], es: &[f64]) -> bool {
        i >= 1
            && ef[i].is_finite()
            && es[i].is_finite()
            && ef[i - 1] <= es[i - 1]
            && ef[i] > es[i]
    }
    fn is_death(i: usize, ef: &[f64], es: &[f64]) -> bool {
        i >= 1
            && ef[i].is_finite()
            && es[i].is_finite()
            && ef[i - 1] >= es[i - 1]
            && ef[i] < es[i]
    }
    let same: fn(usize, &[f64], &[f64]) -> bool = if bullish { is_golden } else { is_death };
    let opp: fn(usize, &[f64], &[f64]) -> bool = if bullish { is_death } else { is_golden };

    let last = n - 1;
    // 条件①：末根必须为第二次同向交叉。
    if !same(last, &ef, &es) {
        return false;
    }

    // 找出中间的反向交叉（回踩/反抽）。
    let mut mid = None;
    for i in (1..last).rev() {
        if opp(i, &ef, &es) {
            mid = Some(i);
            break;
        }
    }
    let mid = match mid {
        Some(i) => i,
        None => return false,
    };

    // 找出第一次同向交叉（位于 mid 之前）。
    let mut first = None;
    for i in (1..mid).rev() {
        if same(i, &ef, &es) {
            first = Some(i);
            break;
        }
    }
    let first = match first {
        Some(i) => i,
        None => return false,
    };
    if first < 1 {
        return false;
    }

    // 条件④⑥：末根成本高于（低于）慢线；中间反向交叉处成本低于（高于）慢线。
    if bullish {
        if !(close[last] > es[last]) {
            return false;
        }
        if !(close[mid] < es[mid]) {
            return false;
        }
    } else {
        if !(close[last] < es[last]) {
            return false;
        }
        if !(close[mid] > es[mid]) {
            return false;
        }
    }

    // 条件⑤：回踩不破前低 / 反抽不过前高。
    if higher_low {
        if bullish {
            let prior_low = low[0..first].iter().cloned().fold(f64::INFINITY, f64::min);
            let pull_low = low[(first + 1)..=last]
                .iter()
                .cloned()
                .fold(f64::INFINITY, f64::min);
            if prior_low.is_infinite() || pull_low.is_infinite() {
                return false;
            }
            if !(pull_low > prior_low) {
                return false;
            }
        } else {
            let prior_high = high[0..first].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let pull_high = high[(first + 1)..=last]
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            if prior_high.is_infinite() || pull_high.is_infinite() {
                return false;
            }
            if !(pull_high < prior_high) {
                return false;
            }
        }
    }

    true
}
