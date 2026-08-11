//! 自动生成 —— TA-Lib 指标分发层（adaq-talib 后端）。
//!
//! 由 `tools/gen_ta_dispatch.py` 依据 adaq-talib 0.1.1 的签名与 `_default` 变体生成。
//! 所有 `TA_*` 指标经由 adaq-talib（纯 Rust、零 FFI）计算，覆盖 TA-Lib 0.7.1 全部 161 个函数。

use crate::indicators::{Candle, PriceSource};

#[inline]
fn pu(p: &[f64], i: usize, d: f64) -> usize { p.get(i).copied().unwrap_or(d).round().max(0.0) as usize }
#[inline]
fn pr(p: &[f64], i: usize, d: f64) -> f64 { p.get(i).copied().unwrap_or(d) }
#[inline]
fn mat(p: &[f64], i: usize, d: f64) -> adaq_talib::overlap::MaType {
    ma_type_from(p.get(i).copied().unwrap_or(d).round() as i32)
}

fn ma_type_from(v: i32) -> adaq_talib::overlap::MaType {
    use adaq_talib::overlap::MaType::*;
    match v {
        0 => Sma, 1 => Ema, 2 => Wma, 3 => Dema, 4 => Tema, 5 => Trima, 6 => Kama, 7 => Mama, _ => Sma,
    }
}

#[allow(unused_variables)]
pub fn call_adaq(
    name: &str,
    candles: &[Candle],
    source: PriceSource,
    params: &[f64],
    field: Option<&str>,
) -> Option<Vec<f64>> {
    let n = candles.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let open: Vec<f64> = candles.iter().map(|c| c.open).collect();
    let high: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let low: Vec<f64> = candles.iter().map(|c| c.low).collect();
    let close: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let volume: Vec<f64> = candles.iter().map(|c| c.volume).collect();
    let src: Vec<f64> = candles.iter().map(|c| source.value(c)).collect();
    match name {
        "TA_ACCBANDS" => {
            let __r = adaq_talib::overlap::accbands(&high, &low, &close, pu(params, 0, 20.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("upper") => __r.upper,
                Some(s) if s.eq_ignore_ascii_case("middle") => __r.middle,
                Some(s) if s.eq_ignore_ascii_case("lower") => __r.lower,
                _ => __r.upper,
            })
        }
        "TA_ACOS" => {
            let __v = adaq_talib::math_trans::acos(&src).ok()?;
            Some(__v)
        }
        "TA_AD" => {
            let __v = adaq_talib::volume::ad(&high, &low, &close, &volume).ok()?;
            Some(__v)
        }
        "TA_ADD" => {
            let __v = adaq_talib::math_ops::add(&src, &src).ok()?;
            Some(__v)
        }
        "TA_ADOSC" => {
            let __v = adaq_talib::volume::adosc(&high, &low, &close, &volume, pu(params, 0, 3.0), pu(params, 1, 10.0)).ok()?;
            Some(__v)
        }
        "TA_ADX" => {
            let __v = adaq_talib::momentum::adx(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_ADXR" => {
            let __v = adaq_talib::momentum::adxr(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_APO" => {
            let __v = adaq_talib::momentum::apo(&src, pu(params, 0, 12.0), pu(params, 1, 26.0)).ok()?;
            Some(__v)
        }
        "TA_AROON" => {
            let __r = adaq_talib::momentum::aroon(&high, &low, pu(params, 0, 14.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("up") => __r.up,
                Some(s) if s.eq_ignore_ascii_case("down") => __r.down,
                _ => __r.up,
            })
        }
        "TA_AROONOSC" => {
            let __v = adaq_talib::momentum::aroon_osc(&high, &low, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_ASIN" => {
            let __v = adaq_talib::math_trans::asin(&src).ok()?;
            Some(__v)
        }
        "TA_ATAN" => {
            let __v = adaq_talib::math_trans::atan(&src).ok()?;
            Some(__v)
        }
        "TA_ATR" => {
            let __v = adaq_talib::volatility::atr(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_AVGDEV" => {
            let __v = adaq_talib::price_transform::avgdev(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_AVGPRICE" => {
            let __v = adaq_talib::price_transform::avgprice(&high, &low, &close, &open).ok()?;
            Some(__v)
        }
        "TA_BBANDS" => {
            let __r = adaq_talib::overlap::bbands(&src, pu(params, 0, 20.0), pr(params, 1, 2.0), pr(params, 2, 2.0), mat(params, 3, 0.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("upper") => __r.upper,
                Some(s) if s.eq_ignore_ascii_case("middle") => __r.middle,
                Some(s) if s.eq_ignore_ascii_case("lower") => __r.lower,
                _ => __r.upper,
            })
        }
        "TA_BETA" => {
            let __v = adaq_talib::stat::beta(&src, &src, pu(params, 0, 5.0)).ok()?;
            Some(__v)
        }
        "TA_BOP" => {
            let __v = adaq_talib::momentum::bop(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CCI" => {
            let __v = adaq_talib::momentum::cci(&high, &low, &close, pu(params, 0, 20.0)).ok()?;
            Some(__v)
        }
        "TA_CDL2CROWS" => {
            let __v = adaq_talib::pattern::batch_1::cdl_2crows(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDL3BLACKCROWS" => {
            let __v = adaq_talib::pattern::batch_2::cdl_3blackcrows(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDL3INSIDE" => {
            let __v = adaq_talib::pattern::batch_2::cdl_3inside(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDL3LINESTRIKE" => {
            let __v = adaq_talib::pattern::batch_2::cdl_3linestrike(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDL3OUTSIDE" => {
            let __v = adaq_talib::pattern::batch_2::cdl_3outside(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDL3STARSINSOUTH" => {
            let __v = adaq_talib::pattern::batch_2::cdl_3starsinsouth(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDL3WHITESOLDIERS" => {
            let __v = adaq_talib::pattern::batch_2::cdl_3whitesoldiers(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLABANDONEDBABY" => {
            let __v = adaq_talib::pattern::batch_2::cdl_abandonedbaby(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLADVANCEBLOCK" => {
            let __v = adaq_talib::pattern::batch_2::cdl_advanceblock(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLBELTHOLD" => {
            let __v = adaq_talib::pattern::batch_3::cdl_belthold(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLBREAKAWAY" => {
            let __v = adaq_talib::pattern::batch_3::cdl_breakaway(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLCLOSINGMARUBOZU" => {
            let __v = adaq_talib::pattern::batch_3::cdl_closingmarubozu(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLCONCEALBABYSWALL" => {
            let __v = adaq_talib::pattern::batch_3::cdl_concealbabyswall(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLCOUNTERATTACK" => {
            let __v = adaq_talib::pattern::batch_3::cdl_counterattack(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLDARKCLOUDCOVER" => {
            let __v = adaq_talib::pattern::batch_3::cdl_darkcloudcover(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLDOJI" => {
            let __v = adaq_talib::pattern::batch_1::cdl_doji(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLDOJISTAR" => {
            let __v = adaq_talib::pattern::batch_3::cdl_dojistar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLDRAGONFLYDOJI" => {
            let __v = adaq_talib::pattern::batch_3::cdl_dragonflydoji(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLENGULFING" => {
            let __v = adaq_talib::pattern::batch_1::cdl_engulfing(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLEVENINGDOJISTAR" => {
            let __v = adaq_talib::pattern::batch_4::cdl_eveningdojistar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLEVENINGSTAR" => {
            let __v = adaq_talib::pattern::batch_4::cdl_eveningstar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLGAPSIDESIDEWHITE" => {
            let __v = adaq_talib::pattern::batch_4::cdl_gapsidesidewhite(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLGRAVESTONEDOJI" => {
            let __v = adaq_talib::pattern::batch_4::cdl_gravestonedoji(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHAMMER" => {
            let __v = adaq_talib::pattern::batch_1::cdl_hammer(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHANGINGMAN" => {
            let __v = adaq_talib::pattern::batch_4::cdl_hangingman(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHARAMI" => {
            let __v = adaq_talib::pattern::batch_1::cdl_harami(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHARAMICROSS" => {
            let __v = adaq_talib::pattern::batch_4::cdl_haramicross(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHIGHWAVE" => {
            let __v = adaq_talib::pattern::batch_1::cdl_highwave(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHIKKAKE" => {
            let __v = adaq_talib::pattern::batch_4::cdl_hikkake(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHIKKAKEMOD" => {
            let __v = adaq_talib::pattern::batch_4::cdl_hikkakemod(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLHOMINGPIGEON" => {
            let __v = adaq_talib::pattern::batch_5::cdl_homingpigeon(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLIDENTICAL3CROWS" => {
            let __v = adaq_talib::pattern::batch_5::cdl_identical3crows(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLINNECK" => {
            let __v = adaq_talib::pattern::batch_5::cdl_inneck(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLINVERTEDHAMMER" => {
            let __v = adaq_talib::pattern::batch_5::cdl_invertedhammer(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLKICKING" => {
            let __v = adaq_talib::pattern::batch_5::cdl_kicking(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLKICKINGBYLENGTH" => {
            let __v = adaq_talib::pattern::batch_5::cdl_kickingbylength(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLLADDERBOTTOM" => {
            let __v = adaq_talib::pattern::batch_5::cdl_ladderbottom(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLLONGLEGGEDDOJI" => {
            let __v = adaq_talib::pattern::batch_6::cdl_longleggeddoji(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLLONGLINE" => {
            let __v = adaq_talib::pattern::batch_6::cdl_longline(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLMARUBOZU" => {
            let __v = adaq_talib::pattern::batch_1::cdl_marubozu(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLMATCHINGLOW" => {
            let __v = adaq_talib::pattern::batch_6::cdl_matchinglow(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLMATHOLD" => {
            let __v = adaq_talib::pattern::batch_6::cdl_mathold(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLMORNINGDOJISTAR" => {
            let __v = adaq_talib::pattern::batch_6::cdl_morningdojistar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLMORNINGSTAR" => {
            let __v = adaq_talib::pattern::batch_6::cdl_morningstar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLONNECK" => {
            let __v = adaq_talib::pattern::batch_6::cdl_onneck(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLPIERCING" => {
            let __v = adaq_talib::pattern::batch_7::cdl_piercing(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLRICKSHAWMAN" => {
            let __v = adaq_talib::pattern::batch_7::cdl_rickshawman(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLRISEFALL3METHODS" => {
            let __v = adaq_talib::pattern::batch_7::cdl_risefall3methods(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLSEPARATINGLINES" => {
            let __v = adaq_talib::pattern::batch_7::cdl_separatinglines(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLSHOOTINGSTAR" => {
            let __v = adaq_talib::pattern::batch_1::cdl_shootingstar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLSHORTLINE" => {
            let __v = adaq_talib::pattern::batch_7::cdl_shortline(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLSPINNINGTOP" => {
            let __v = adaq_talib::pattern::batch_7::cdl_spinningtop(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLSTALLEDPATTERN" => {
            let __v = adaq_talib::pattern::batch_7::cdl_stalledpattern(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLSTICKSANDWICH" => {
            let __v = adaq_talib::pattern::batch_8::cdl_sticksandwich(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLTAKURI" => {
            let __v = adaq_talib::pattern::batch_8::cdl_takuri(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLTASUKIGAP" => {
            let __v = adaq_talib::pattern::batch_8::cdl_tasukigap(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLTHRUSTING" => {
            let __v = adaq_talib::pattern::batch_8::cdl_thrusting(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLTRISTAR" => {
            let __v = adaq_talib::pattern::batch_8::cdl_tristar(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLUNIQUE3RIVER" => {
            let __v = adaq_talib::pattern::batch_8::cdl_unique3river(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLUPSIDEGAP2CROWS" => {
            let __v = adaq_talib::pattern::batch_8::cdl_upsidegap2crows(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CDLXSIDEGAP3METHODS" => {
            let __v = adaq_talib::pattern::batch_8::cdl_xsidegap3methods(&open, &high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_CEIL" => {
            let __v = adaq_talib::math_trans::ceil(&src).ok()?;
            Some(__v)
        }
        "TA_CMO" => {
            let __v = adaq_talib::momentum::cmo(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_CORREL" => {
            let __v = adaq_talib::stat::correl(&src, &src, pu(params, 0, 5.0)).ok()?;
            Some(__v)
        }
        "TA_COS" => {
            let __v = adaq_talib::math_trans::cos(&src).ok()?;
            Some(__v)
        }
        "TA_COSH" => {
            let __v = adaq_talib::math_trans::cosh(&src).ok()?;
            Some(__v)
        }
        "TA_DEMA" => {
            let __v = adaq_talib::overlap::dema(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_DIV" => {
            let __v = adaq_talib::math_ops::div(&src, &src).ok()?;
            Some(__v)
        }
        "TA_DX" => {
            let __v = adaq_talib::momentum::dx(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_EMA" => {
            let __v = adaq_talib::overlap::ema(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_EXP" => {
            let __v = adaq_talib::math_trans::exp(&src).ok()?;
            Some(__v)
        }
        "TA_FLOOR" => {
            let __v = adaq_talib::math_trans::floor(&src).ok()?;
            Some(__v)
        }
        "TA_HT_DCPERIOD" => {
            let __v = adaq_talib::cycle::ht_dcperiod(&src).ok()?;
            Some(__v)
        }
        "TA_HT_DCPHASE" => {
            let __v = adaq_talib::cycle::ht_dcphase(&src).ok()?;
            Some(__v)
        }
        "TA_HT_PHASOR" => {
            let __r = adaq_talib::cycle::ht_phasor(&src).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("in_phase") => __r.in_phase,
                Some(s) if s.eq_ignore_ascii_case("quadrature") => __r.quadrature,
                _ => __r.in_phase,
            })
        }
        "TA_HT_SINE" => {
            let __r = adaq_talib::cycle::ht_sine(&src).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("sine") => __r.sine,
                Some(s) if s.eq_ignore_ascii_case("lead_sine") => __r.lead_sine,
                _ => __r.sine,
            })
        }
        "TA_HT_TRENDLINE" => {
            let __v = adaq_talib::cycle::ht_trendline(&src).ok()?;
            Some(__v)
        }
        "TA_HT_TRENDMODE" => {
            let __v = adaq_talib::cycle::ht_trendmode(&src).ok()?;
            Some(__v)
        }
        "TA_IMI" => {
            let __v = adaq_talib::momentum::imi(&open, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_KAMA" => {
            let __v = adaq_talib::overlap::kama(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_LINEARREG" => {
            let __v = adaq_talib::stat::linear_reg(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_LINEARREG_ANGLE" => {
            let __v = adaq_talib::stat::linear_reg_angle(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_LINEARREG_INTERCEPT" => {
            let __v = adaq_talib::stat::linear_reg_intercept(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_LINEARREG_SLOPE" => {
            let __v = adaq_talib::stat::linear_reg_slope(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_LN" => {
            let __v = adaq_talib::math_trans::ln(&src).ok()?;
            Some(__v)
        }
        "TA_LOG10" => {
            let __v = adaq_talib::math_trans::log10(&src).ok()?;
            Some(__v)
        }
        "TA_MA" => {
            let __v = adaq_talib::overlap::ma(&src, pu(params, 0, 30.0), mat(params, 1, 0.0)).ok()?;
            Some(__v)
        }
        "TA_MACD" => {
            let __r = adaq_talib::momentum::macd(&src, pu(params, 0, 12.0), pu(params, 1, 26.0), pu(params, 2, 9.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("macd") => __r.macd,
                Some(s) if s.eq_ignore_ascii_case("signal") => __r.signal,
                Some(s) if s.eq_ignore_ascii_case("hist") => __r.hist,
                _ => __r.macd,
            })
        }
        "TA_MACDEXT" => {
            let __r = adaq_talib::momentum::macd_ext(&src, pu(params, 0, 12.0), pu(params, 1, 26.0), pu(params, 2, 9.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("macd") => __r.macd,
                Some(s) if s.eq_ignore_ascii_case("signal") => __r.signal,
                Some(s) if s.eq_ignore_ascii_case("hist") => __r.hist,
                _ => __r.macd,
            })
        }
        "TA_MACDFIX" => {
            let __r = adaq_talib::momentum::macd_fix(&src, pu(params, 0, 12.0), pu(params, 1, 26.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("macd") => __r.macd,
                Some(s) if s.eq_ignore_ascii_case("signal") => __r.signal,
                Some(s) if s.eq_ignore_ascii_case("hist") => __r.hist,
                _ => __r.macd,
            })
        }
        "TA_MAMA" => {
            let __r = adaq_talib::cycle::mama(&src, pr(params, 0, 0.5), pr(params, 1, 0.05)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("mama") => __r.mama,
                Some(s) if s.eq_ignore_ascii_case("fama") => __r.fama,
                _ => __r.mama,
            })
        }
        "TA_MAVP" => {
            let periods = vec![2.0; n];
            let __r = adaq_talib::overlap::mavp(&src, &periods, pu(params, 0, 2.0), pu(params, 1, 30.0), mat(params, 2, 0.0)).ok()?;
            Some(__r)
        }
        "TA_MAX" => {
            let __v = adaq_talib::math_ops::max(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_MAXINDEX" => {
            let __v = adaq_talib::math_ops::max_index(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_MEDPRICE" => {
            let __v = adaq_talib::price_transform::medprice(&high, &low).ok()?;
            Some(__v)
        }
        "TA_MFI" => {
            let __v = adaq_talib::momentum::mfi(&high, &low, &close, &volume, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_MIDPOINT" => {
            let __v = adaq_talib::overlap::midpoint(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_MIDPRICE" => {
            let __v = adaq_talib::overlap::midprice(&high, &low, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_MIN" => {
            let __v = adaq_talib::math_ops::min(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_MININDEX" => {
            let __v = adaq_talib::math_ops::min_index(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_MINMAX" => {
            let __r = adaq_talib::math_ops::minmax(&src, pu(params, 0, 30.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("min") => __r.min,
                Some(s) if s.eq_ignore_ascii_case("max") => __r.max,
                _ => __r.min,
            })
        }
        "TA_MINMAXINDEX" => {
            let __r = adaq_talib::math_ops::minmax_index(&src, pu(params, 0, 30.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("min_idx") => __r.min_idx,
                Some(s) if s.eq_ignore_ascii_case("max_idx") => __r.max_idx,
                _ => __r.min_idx,
            })
        }
        "TA_MINUS_DI" => {
            let __v = adaq_talib::momentum::minus_di(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_MINUS_DM" => {
            let __v = adaq_talib::momentum::minus_dm(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_MOM" => {
            let __v = adaq_talib::momentum::mom(&src, pu(params, 0, 10.0)).ok()?;
            Some(__v)
        }
        "TA_MULT" => {
            let __v = adaq_talib::math_ops::mult(&src, &src).ok()?;
            Some(__v)
        }
        "TA_NATR" => {
            let __v = adaq_talib::volatility::natr(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_OBV" => {
            let __v = adaq_talib::volume::obv(&close, &volume).ok()?;
            Some(__v)
        }
        "TA_PLUS_DI" => {
            let __v = adaq_talib::momentum::plus_di(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_PLUS_DM" => {
            let __v = adaq_talib::momentum::plus_dm(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_PPO" => {
            let __v = adaq_talib::momentum::ppo(&src, pu(params, 0, 12.0), pu(params, 1, 26.0)).ok()?;
            Some(__v)
        }
        "TA_ROC" => {
            let __v = adaq_talib::momentum::roc(&src, pu(params, 0, 10.0)).ok()?;
            Some(__v)
        }
        "TA_ROCP" => {
            let __v = adaq_talib::momentum::rocp(&src, pu(params, 0, 10.0)).ok()?;
            Some(__v)
        }
        "TA_ROCR" => {
            let __v = adaq_talib::momentum::rocr(&src, pu(params, 0, 10.0)).ok()?;
            Some(__v)
        }
        "TA_ROCR100" => {
            let __v = adaq_talib::momentum::rocr100(&src, pu(params, 0, 10.0)).ok()?;
            Some(__v)
        }
        "TA_RSI" => {
            let __v = adaq_talib::momentum::rsi(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_SAR" => {
            let __v = adaq_talib::overlap::sar(&high, &low, pr(params, 0, 0.02), pr(params, 1, 0.2)).ok()?;
            Some(__v)
        }
        "TA_SAREXT" => {
            let __v = adaq_talib::overlap::sarext(&high, &low, pr(params, 0, 0.0), pr(params, 1, 0.0), pr(params, 2, 0.02), pr(params, 3, 0.02), pr(params, 4, 0.2), pr(params, 5, 0.02), pr(params, 6, 0.02), pr(params, 7, 0.2)).ok()?;
            Some(__v)
        }
        "TA_SIN" => {
            let __v = adaq_talib::math_trans::sin(&src).ok()?;
            Some(__v)
        }
        "TA_SINH" => {
            let __v = adaq_talib::math_trans::sinh(&src).ok()?;
            Some(__v)
        }
        "TA_SMA" => {
            let __v = adaq_talib::overlap::sma(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_SQRT" => {
            let __v = adaq_talib::math_trans::sqrt(&src).ok()?;
            Some(__v)
        }
        "TA_STDDEV" => {
            let __v = adaq_talib::stat::stddev(&src, pu(params, 0, 5.0), pr(params, 1, 1.0)).ok()?;
            Some(__v)
        }
        "TA_STOCH" => {
            let __r = adaq_talib::momentum::stoch(&high, &low, &close, pu(params, 0, 5.0), pu(params, 1, 3.0), pu(params, 2, 3.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("slow_k") => __r.slow_k,
                Some(s) if s.eq_ignore_ascii_case("slow_d") => __r.slow_d,
                _ => __r.slow_k,
            })
        }
        "TA_STOCHF" => {
            let __r = adaq_talib::momentum::stoch_f(&high, &low, &close, pu(params, 0, 5.0), pu(params, 1, 3.0)).ok()?;
            Some(match field {
                Some(s) if s.eq_ignore_ascii_case("fast_k") => __r.fast_k,
                Some(s) if s.eq_ignore_ascii_case("fast_d") => __r.fast_d,
                _ => __r.fast_k,
            })
        }
        "TA_STOCHRSI" => {
            let __v = adaq_talib::momentum::stoch_rsi(&close, pu(params, 0, 14.0), pu(params, 1, 14.0)).ok()?;
            Some(__v)
        }
        "TA_SUB" => {
            let __v = adaq_talib::math_ops::sub(&src, &src).ok()?;
            Some(__v)
        }
        "TA_SUM" => {
            let __v = adaq_talib::math_ops::sum(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_T3" => {
            let __v = adaq_talib::overlap::t3(&src, pu(params, 0, 5.0), pr(params, 1, 0.7)).ok()?;
            Some(__v)
        }
        "TA_TAN" => {
            let __v = adaq_talib::math_trans::tan(&src).ok()?;
            Some(__v)
        }
        "TA_TANH" => {
            let __v = adaq_talib::math_trans::tanh(&src).ok()?;
            Some(__v)
        }
        "TA_TEMA" => {
            let __v = adaq_talib::overlap::tema(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_TRANGE" => {
            let __v = adaq_talib::volatility::trange(&high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_TRIMA" => {
            let __v = adaq_talib::overlap::trima(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_TRIX" => {
            let __v = adaq_talib::momentum::trix(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        "TA_TSF" => {
            let __v = adaq_talib::stat::tsf(&src, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_TYPPRICE" => {
            let __v = adaq_talib::price_transform::typprice(&high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_ULTOSC" => {
            let __v = adaq_talib::momentum::ultosc(&high, &low, &close, pu(params, 0, 7.0), pu(params, 1, 14.0), pu(params, 2, 28.0)).ok()?;
            Some(__v)
        }
        "TA_VAR" => {
            let __v = adaq_talib::stat::var(&src, pu(params, 0, 5.0), pr(params, 1, 1.0)).ok()?;
            Some(__v)
        }
        "TA_WCLPRICE" => {
            let __v = adaq_talib::price_transform::wclprice(&high, &low, &close).ok()?;
            Some(__v)
        }
        "TA_WILLR" => {
            let __v = adaq_talib::momentum::willr(&high, &low, &close, pu(params, 0, 14.0)).ok()?;
            Some(__v)
        }
        "TA_WMA" => {
            let __v = adaq_talib::overlap::wma(&src, pu(params, 0, 30.0)).ok()?;
            Some(__v)
        }
        _ => None,
    }
}

/// TA_* 函数元信息（用于文档生成）。
pub struct TaOptInput {
    pub name: String,
    pub display: String,
    pub kind: String,
    pub default: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// 函数元信息（用于文档生成）。
pub struct TaFuncMeta {
    pub name: String,
    pub group: String,
    pub hint: String,
    pub opt_inputs: Vec<TaOptInput>,
    pub outputs: Vec<(String, String)>,
}

/// 判断某 TA_* 函数是否被 adaq-talib 支持。
pub fn ta_function_exists(name: &str) -> bool {
    _known(name)
}

fn _known(name: &str) -> bool {
    matches!(
        name,
        "TA_ACCBANDS" |
        "TA_ACOS" |
        "TA_AD" |
        "TA_ADD" |
        "TA_ADOSC" |
        "TA_ADX" |
        "TA_ADXR" |
        "TA_APO" |
        "TA_AROON" |
        "TA_AROONOSC" |
        "TA_ASIN" |
        "TA_ATAN" |
        "TA_ATR" |
        "TA_AVGDEV" |
        "TA_AVGPRICE" |
        "TA_BBANDS" |
        "TA_BETA" |
        "TA_BOP" |
        "TA_CCI" |
        "TA_CDL2CROWS" |
        "TA_CDL3BLACKCROWS" |
        "TA_CDL3INSIDE" |
        "TA_CDL3LINESTRIKE" |
        "TA_CDL3OUTSIDE" |
        "TA_CDL3STARSINSOUTH" |
        "TA_CDL3WHITESOLDIERS" |
        "TA_CDLABANDONEDBABY" |
        "TA_CDLADVANCEBLOCK" |
        "TA_CDLBELTHOLD" |
        "TA_CDLBREAKAWAY" |
        "TA_CDLCLOSINGMARUBOZU" |
        "TA_CDLCONCEALBABYSWALL" |
        "TA_CDLCOUNTERATTACK" |
        "TA_CDLDARKCLOUDCOVER" |
        "TA_CDLDOJI" |
        "TA_CDLDOJISTAR" |
        "TA_CDLDRAGONFLYDOJI" |
        "TA_CDLENGULFING" |
        "TA_CDLEVENINGDOJISTAR" |
        "TA_CDLEVENINGSTAR" |
        "TA_CDLGAPSIDESIDEWHITE" |
        "TA_CDLGRAVESTONEDOJI" |
        "TA_CDLHAMMER" |
        "TA_CDLHANGINGMAN" |
        "TA_CDLHARAMI" |
        "TA_CDLHARAMICROSS" |
        "TA_CDLHIGHWAVE" |
        "TA_CDLHIKKAKE" |
        "TA_CDLHIKKAKEMOD" |
        "TA_CDLHOMINGPIGEON" |
        "TA_CDLIDENTICAL3CROWS" |
        "TA_CDLINNECK" |
        "TA_CDLINVERTEDHAMMER" |
        "TA_CDLKICKING" |
        "TA_CDLKICKINGBYLENGTH" |
        "TA_CDLLADDERBOTTOM" |
        "TA_CDLLONGLEGGEDDOJI" |
        "TA_CDLLONGLINE" |
        "TA_CDLMARUBOZU" |
        "TA_CDLMATCHINGLOW" |
        "TA_CDLMATHOLD" |
        "TA_CDLMORNINGDOJISTAR" |
        "TA_CDLMORNINGSTAR" |
        "TA_CDLONNECK" |
        "TA_CDLPIERCING" |
        "TA_CDLRICKSHAWMAN" |
        "TA_CDLRISEFALL3METHODS" |
        "TA_CDLSEPARATINGLINES" |
        "TA_CDLSHOOTINGSTAR" |
        "TA_CDLSHORTLINE" |
        "TA_CDLSPINNINGTOP" |
        "TA_CDLSTALLEDPATTERN" |
        "TA_CDLSTICKSANDWICH" |
        "TA_CDLTAKURI" |
        "TA_CDLTASUKIGAP" |
        "TA_CDLTHRUSTING" |
        "TA_CDLTRISTAR" |
        "TA_CDLUNIQUE3RIVER" |
        "TA_CDLUPSIDEGAP2CROWS" |
        "TA_CDLXSIDEGAP3METHODS" |
        "TA_CEIL" |
        "TA_CMO" |
        "TA_CORREL" |
        "TA_COS" |
        "TA_COSH" |
        "TA_DEMA" |
        "TA_DIV" |
        "TA_DX" |
        "TA_EMA" |
        "TA_EXP" |
        "TA_FLOOR" |
        "TA_HT_DCPERIOD" |
        "TA_HT_DCPHASE" |
        "TA_HT_PHASOR" |
        "TA_HT_SINE" |
        "TA_HT_TRENDLINE" |
        "TA_HT_TRENDMODE" |
        "TA_IMI" |
        "TA_KAMA" |
        "TA_LINEARREG" |
        "TA_LINEARREG_ANGLE" |
        "TA_LINEARREG_INTERCEPT" |
        "TA_LINEARREG_SLOPE" |
        "TA_LN" |
        "TA_LOG10" |
        "TA_MA" |
        "TA_MACD" |
        "TA_MACDEXT" |
        "TA_MACDFIX" |
        "TA_MAMA" |
        "TA_MAVP" |
        "TA_MAX" |
        "TA_MAXINDEX" |
        "TA_MEDPRICE" |
        "TA_MFI" |
        "TA_MIDPOINT" |
        "TA_MIDPRICE" |
        "TA_MIN" |
        "TA_MININDEX" |
        "TA_MINMAX" |
        "TA_MINMAXINDEX" |
        "TA_MINUS_DI" |
        "TA_MINUS_DM" |
        "TA_MOM" |
        "TA_MULT" |
        "TA_NATR" |
        "TA_OBV" |
        "TA_PLUS_DI" |
        "TA_PLUS_DM" |
        "TA_PPO" |
        "TA_ROC" |
        "TA_ROCP" |
        "TA_ROCR" |
        "TA_ROCR100" |
        "TA_RSI" |
        "TA_SAR" |
        "TA_SAREXT" |
        "TA_SIN" |
        "TA_SINH" |
        "TA_SMA" |
        "TA_SQRT" |
        "TA_STDDEV" |
        "TA_STOCH" |
        "TA_STOCHF" |
        "TA_STOCHRSI" |
        "TA_SUB" |
        "TA_SUM" |
        "TA_T3" |
        "TA_TAN" |
        "TA_TANH" |
        "TA_TEMA" |
        "TA_TRANGE" |
        "TA_TRIMA" |
        "TA_TRIX" |
        "TA_TSF" |
        "TA_TYPPRICE" |
        "TA_ULTOSC" |
        "TA_VAR" |
        "TA_WCLPRICE" |
        "TA_WILLR" |
        "TA_WMA"
    )
}

/// 列出 TA-Lib 0.7.1 提供的全部 161 个函数（名称, 分组），用于文档生成与自检。
pub fn list_all_functions() -> Vec<(String, String)> {
    vec![
        ("TA_ACCBANDS".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_ACOS".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_AD".to_string(), "Volume Indicators / 成交量指标".to_string()),
        ("TA_ADD".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_ADOSC".to_string(), "Volume Indicators / 成交量指标".to_string()),
        ("TA_ADX".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_ADXR".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_APO".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_AROON".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_AROONOSC".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_ASIN".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_ATAN".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_ATR".to_string(), "Volatility Indicators / 波动率指标".to_string()),
        ("TA_AVGDEV".to_string(), "Price Transform / 价格变换".to_string()),
        ("TA_AVGPRICE".to_string(), "Price Transform / 价格变换".to_string()),
        ("TA_BBANDS".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_BETA".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_BOP".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_CCI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_CDL2CROWS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDL3BLACKCROWS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDL3INSIDE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDL3LINESTRIKE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDL3OUTSIDE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDL3STARSINSOUTH".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDL3WHITESOLDIERS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLABANDONEDBABY".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLADVANCEBLOCK".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLBELTHOLD".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLBREAKAWAY".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLCLOSINGMARUBOZU".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLCONCEALBABYSWALL".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLCOUNTERATTACK".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLDARKCLOUDCOVER".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLDOJI".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLDOJISTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLDRAGONFLYDOJI".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLENGULFING".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLEVENINGDOJISTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLEVENINGSTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLGAPSIDESIDEWHITE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLGRAVESTONEDOJI".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHAMMER".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHANGINGMAN".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHARAMI".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHARAMICROSS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHIGHWAVE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHIKKAKE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHIKKAKEMOD".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLHOMINGPIGEON".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLIDENTICAL3CROWS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLINNECK".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLINVERTEDHAMMER".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLKICKING".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLKICKINGBYLENGTH".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLLADDERBOTTOM".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLLONGLEGGEDDOJI".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLLONGLINE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLMARUBOZU".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLMATCHINGLOW".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLMATHOLD".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLMORNINGDOJISTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLMORNINGSTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLONNECK".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLPIERCING".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLRICKSHAWMAN".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLRISEFALL3METHODS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLSEPARATINGLINES".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLSHOOTINGSTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLSHORTLINE".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLSPINNINGTOP".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLSTALLEDPATTERN".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLSTICKSANDWICH".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLTAKURI".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLTASUKIGAP".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLTHRUSTING".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLTRISTAR".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLUNIQUE3RIVER".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLUPSIDEGAP2CROWS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CDLXSIDEGAP3METHODS".to_string(), "Pattern Recognition / 形态识别".to_string()),
        ("TA_CEIL".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_CMO".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_CORREL".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_COS".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_COSH".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_DEMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_DIV".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_DX".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_EMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_EXP".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_FLOOR".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_HT_DCPERIOD".to_string(), "Cycle Indicators / 周期指标".to_string()),
        ("TA_HT_DCPHASE".to_string(), "Cycle Indicators / 周期指标".to_string()),
        ("TA_HT_PHASOR".to_string(), "Cycle Indicators / 周期指标".to_string()),
        ("TA_HT_SINE".to_string(), "Cycle Indicators / 周期指标".to_string()),
        ("TA_HT_TRENDLINE".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_HT_TRENDMODE".to_string(), "Cycle Indicators / 周期指标".to_string()),
        ("TA_IMI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_KAMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_LINEARREG".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_LINEARREG_ANGLE".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_LINEARREG_INTERCEPT".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_LINEARREG_SLOPE".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_LN".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_LOG10".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_MA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_MACD".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MACDEXT".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MACDFIX".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MAMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_MAVP".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_MAX".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_MAXINDEX".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_MEDPRICE".to_string(), "Price Transform / 价格变换".to_string()),
        ("TA_MFI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MIDPOINT".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_MIDPRICE".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_MIN".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_MININDEX".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_MINMAX".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_MINMAXINDEX".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_MINUS_DI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MINUS_DM".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MOM".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_MULT".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_NATR".to_string(), "Volatility Indicators / 波动率指标".to_string()),
        ("TA_OBV".to_string(), "Volume Indicators / 成交量指标".to_string()),
        ("TA_PLUS_DI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_PLUS_DM".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_PPO".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_ROC".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_ROCP".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_ROCR".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_ROCR100".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_RSI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_SAR".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_SAREXT".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_SIN".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_SINH".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_SMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_SQRT".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_STDDEV".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_STOCH".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_STOCHF".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_STOCHRSI".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_SUB".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_SUM".to_string(), "Math Operators / 数学运算".to_string()),
        ("TA_T3".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_TAN".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_TANH".to_string(), "Math Transform / 数学变换".to_string()),
        ("TA_TEMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_TRANGE".to_string(), "Volatility Indicators / 波动率指标".to_string()),
        ("TA_TRIMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
        ("TA_TRIX".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_TSF".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_TYPPRICE".to_string(), "Price Transform / 价格变换".to_string()),
        ("TA_ULTOSC".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_VAR".to_string(), "Statistic Functions / 统计函数".to_string()),
        ("TA_WCLPRICE".to_string(), "Price Transform / 价格变换".to_string()),
        ("TA_WILLR".to_string(), "Momentum Indicators / 动量指标".to_string()),
        ("TA_WMA".to_string(), "Overlap Studies / 重叠研究".to_string()),
    ]
}

/// 取得某函数的完整元信息（可选参数含默认值/范围、输出字段名与类型）。
pub fn ta_meta(name: &str) -> Option<TaFuncMeta> {
    match name {
        "TA_ACCBANDS" => Some(TaFuncMeta {
            name: "TA_ACCBANDS".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 20.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outRealUpperBand.0".to_string(), "real".to_string()),
                ("outRealMiddleBand.1".to_string(), "real".to_string()),
                ("outRealLowerBand.2".to_string(), "real".to_string()),
            ],
        }),
        "TA_ACOS" => Some(TaFuncMeta {
            name: "TA_ACOS".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_AD" => Some(TaFuncMeta {
            name: "TA_AD".to_string(),
            group: "Volume Indicators / 成交量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ADD" => Some(TaFuncMeta {
            name: "TA_ADD".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ADOSC" => Some(TaFuncMeta {
            name: "TA_ADOSC".to_string(),
            group: "Volume Indicators / 成交量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 3.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 10.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ADX" => Some(TaFuncMeta {
            name: "TA_ADX".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ADXR" => Some(TaFuncMeta {
            name: "TA_ADXR".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_APO" => Some(TaFuncMeta {
            name: "TA_APO".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 12.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 26.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_AROON" => Some(TaFuncMeta {
            name: "TA_AROON".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outAroonDown.0".to_string(), "real".to_string()),
                ("outAroonUp.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_AROONOSC" => Some(TaFuncMeta {
            name: "TA_AROONOSC".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ASIN" => Some(TaFuncMeta {
            name: "TA_ASIN".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ATAN" => Some(TaFuncMeta {
            name: "TA_ATAN".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ATR" => Some(TaFuncMeta {
            name: "TA_ATR".to_string(),
            group: "Volatility Indicators / 波动率指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_AVGDEV" => Some(TaFuncMeta {
            name: "TA_AVGDEV".to_string(),
            group: "Price Transform / 价格变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_AVGPRICE" => Some(TaFuncMeta {
            name: "TA_AVGPRICE".to_string(),
            group: "Price Transform / 价格变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_BBANDS" => Some(TaFuncMeta {
            name: "TA_BBANDS".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInNbDevUp".to_string(), display: String::new(), kind: "real".to_string(), default: 2.0, min: None, max: None },
                TaOptInput { name: "optInNbDevDn".to_string(), display: String::new(), kind: "real".to_string(), default: 2.0, min: None, max: None },
                TaOptInput { name: "optInMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outRealUpperBand.0".to_string(), "real".to_string()),
                ("outRealMiddleBand.1".to_string(), "real".to_string()),
                ("outRealLowerBand.2".to_string(), "real".to_string()),
            ],
        }),
        "TA_BETA" => Some(TaFuncMeta {
            name: "TA_BETA".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_BOP" => Some(TaFuncMeta {
            name: "TA_BOP".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_CCI" => Some(TaFuncMeta {
            name: "TA_CCI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_CDL2CROWS" => Some(TaFuncMeta {
            name: "TA_CDL2CROWS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDL3BLACKCROWS" => Some(TaFuncMeta {
            name: "TA_CDL3BLACKCROWS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDL3INSIDE" => Some(TaFuncMeta {
            name: "TA_CDL3INSIDE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDL3LINESTRIKE" => Some(TaFuncMeta {
            name: "TA_CDL3LINESTRIKE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDL3OUTSIDE" => Some(TaFuncMeta {
            name: "TA_CDL3OUTSIDE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDL3STARSINSOUTH" => Some(TaFuncMeta {
            name: "TA_CDL3STARSINSOUTH".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDL3WHITESOLDIERS" => Some(TaFuncMeta {
            name: "TA_CDL3WHITESOLDIERS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLABANDONEDBABY" => Some(TaFuncMeta {
            name: "TA_CDLABANDONEDBABY".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.3, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLADVANCEBLOCK" => Some(TaFuncMeta {
            name: "TA_CDLADVANCEBLOCK".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLBELTHOLD" => Some(TaFuncMeta {
            name: "TA_CDLBELTHOLD".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLBREAKAWAY" => Some(TaFuncMeta {
            name: "TA_CDLBREAKAWAY".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLCLOSINGMARUBOZU" => Some(TaFuncMeta {
            name: "TA_CDLCLOSINGMARUBOZU".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLCONCEALBABYSWALL" => Some(TaFuncMeta {
            name: "TA_CDLCONCEALBABYSWALL".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLCOUNTERATTACK" => Some(TaFuncMeta {
            name: "TA_CDLCOUNTERATTACK".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLDARKCLOUDCOVER" => Some(TaFuncMeta {
            name: "TA_CDLDARKCLOUDCOVER".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.5, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLDOJI" => Some(TaFuncMeta {
            name: "TA_CDLDOJI".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLDOJISTAR" => Some(TaFuncMeta {
            name: "TA_CDLDOJISTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLDRAGONFLYDOJI" => Some(TaFuncMeta {
            name: "TA_CDLDRAGONFLYDOJI".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLENGULFING" => Some(TaFuncMeta {
            name: "TA_CDLENGULFING".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLEVENINGDOJISTAR" => Some(TaFuncMeta {
            name: "TA_CDLEVENINGDOJISTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.3, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLEVENINGSTAR" => Some(TaFuncMeta {
            name: "TA_CDLEVENINGSTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.3, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLGAPSIDESIDEWHITE" => Some(TaFuncMeta {
            name: "TA_CDLGAPSIDESIDEWHITE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLGRAVESTONEDOJI" => Some(TaFuncMeta {
            name: "TA_CDLGRAVESTONEDOJI".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHAMMER" => Some(TaFuncMeta {
            name: "TA_CDLHAMMER".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHANGINGMAN" => Some(TaFuncMeta {
            name: "TA_CDLHANGINGMAN".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHARAMI" => Some(TaFuncMeta {
            name: "TA_CDLHARAMI".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHARAMICROSS" => Some(TaFuncMeta {
            name: "TA_CDLHARAMICROSS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHIGHWAVE" => Some(TaFuncMeta {
            name: "TA_CDLHIGHWAVE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHIKKAKE" => Some(TaFuncMeta {
            name: "TA_CDLHIKKAKE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHIKKAKEMOD" => Some(TaFuncMeta {
            name: "TA_CDLHIKKAKEMOD".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLHOMINGPIGEON" => Some(TaFuncMeta {
            name: "TA_CDLHOMINGPIGEON".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLIDENTICAL3CROWS" => Some(TaFuncMeta {
            name: "TA_CDLIDENTICAL3CROWS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLINNECK" => Some(TaFuncMeta {
            name: "TA_CDLINNECK".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLINVERTEDHAMMER" => Some(TaFuncMeta {
            name: "TA_CDLINVERTEDHAMMER".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLKICKING" => Some(TaFuncMeta {
            name: "TA_CDLKICKING".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLKICKINGBYLENGTH" => Some(TaFuncMeta {
            name: "TA_CDLKICKINGBYLENGTH".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLLADDERBOTTOM" => Some(TaFuncMeta {
            name: "TA_CDLLADDERBOTTOM".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLLONGLEGGEDDOJI" => Some(TaFuncMeta {
            name: "TA_CDLLONGLEGGEDDOJI".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLLONGLINE" => Some(TaFuncMeta {
            name: "TA_CDLLONGLINE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLMARUBOZU" => Some(TaFuncMeta {
            name: "TA_CDLMARUBOZU".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLMATCHINGLOW" => Some(TaFuncMeta {
            name: "TA_CDLMATCHINGLOW".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLMATHOLD" => Some(TaFuncMeta {
            name: "TA_CDLMATHOLD".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.5, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLMORNINGDOJISTAR" => Some(TaFuncMeta {
            name: "TA_CDLMORNINGDOJISTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.3, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLMORNINGSTAR" => Some(TaFuncMeta {
            name: "TA_CDLMORNINGSTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInPenetration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.3, min: None, max: None },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLONNECK" => Some(TaFuncMeta {
            name: "TA_CDLONNECK".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLPIERCING" => Some(TaFuncMeta {
            name: "TA_CDLPIERCING".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLRICKSHAWMAN" => Some(TaFuncMeta {
            name: "TA_CDLRICKSHAWMAN".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLRISEFALL3METHODS" => Some(TaFuncMeta {
            name: "TA_CDLRISEFALL3METHODS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLSEPARATINGLINES" => Some(TaFuncMeta {
            name: "TA_CDLSEPARATINGLINES".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLSHOOTINGSTAR" => Some(TaFuncMeta {
            name: "TA_CDLSHOOTINGSTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLSHORTLINE" => Some(TaFuncMeta {
            name: "TA_CDLSHORTLINE".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLSPINNINGTOP" => Some(TaFuncMeta {
            name: "TA_CDLSPINNINGTOP".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLSTALLEDPATTERN" => Some(TaFuncMeta {
            name: "TA_CDLSTALLEDPATTERN".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLSTICKSANDWICH" => Some(TaFuncMeta {
            name: "TA_CDLSTICKSANDWICH".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLTAKURI" => Some(TaFuncMeta {
            name: "TA_CDLTAKURI".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLTASUKIGAP" => Some(TaFuncMeta {
            name: "TA_CDLTASUKIGAP".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLTHRUSTING" => Some(TaFuncMeta {
            name: "TA_CDLTHRUSTING".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLTRISTAR" => Some(TaFuncMeta {
            name: "TA_CDLTRISTAR".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLUNIQUE3RIVER" => Some(TaFuncMeta {
            name: "TA_CDLUNIQUE3RIVER".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLUPSIDEGAP2CROWS" => Some(TaFuncMeta {
            name: "TA_CDLUPSIDEGAP2CROWS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CDLXSIDEGAP3METHODS" => Some(TaFuncMeta {
            name: "TA_CDLXSIDEGAP3METHODS".to_string(),
            group: "Pattern Recognition / 形态识别".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_CEIL" => Some(TaFuncMeta {
            name: "TA_CEIL".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_CMO" => Some(TaFuncMeta {
            name: "TA_CMO".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_CORREL" => Some(TaFuncMeta {
            name: "TA_CORREL".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_COS" => Some(TaFuncMeta {
            name: "TA_COS".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_COSH" => Some(TaFuncMeta {
            name: "TA_COSH".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_DEMA" => Some(TaFuncMeta {
            name: "TA_DEMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_DIV" => Some(TaFuncMeta {
            name: "TA_DIV".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_DX" => Some(TaFuncMeta {
            name: "TA_DX".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_EMA" => Some(TaFuncMeta {
            name: "TA_EMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_EXP" => Some(TaFuncMeta {
            name: "TA_EXP".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_FLOOR" => Some(TaFuncMeta {
            name: "TA_FLOOR".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_HT_DCPERIOD" => Some(TaFuncMeta {
            name: "TA_HT_DCPERIOD".to_string(),
            group: "Cycle Indicators / 周期指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_HT_DCPHASE" => Some(TaFuncMeta {
            name: "TA_HT_DCPHASE".to_string(),
            group: "Cycle Indicators / 周期指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_HT_PHASOR" => Some(TaFuncMeta {
            name: "TA_HT_PHASOR".to_string(),
            group: "Cycle Indicators / 周期指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInPhase.0".to_string(), "real".to_string()),
                ("outQuadrature.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_HT_SINE" => Some(TaFuncMeta {
            name: "TA_HT_SINE".to_string(),
            group: "Cycle Indicators / 周期指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outSine.0".to_string(), "real".to_string()),
                ("outLeadSine.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_HT_TRENDLINE" => Some(TaFuncMeta {
            name: "TA_HT_TRENDLINE".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_HT_TRENDMODE" => Some(TaFuncMeta {
            name: "TA_HT_TRENDMODE".to_string(),
            group: "Cycle Indicators / 周期指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_IMI" => Some(TaFuncMeta {
            name: "TA_IMI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_KAMA" => Some(TaFuncMeta {
            name: "TA_KAMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_LINEARREG" => Some(TaFuncMeta {
            name: "TA_LINEARREG".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_LINEARREG_ANGLE" => Some(TaFuncMeta {
            name: "TA_LINEARREG_ANGLE".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_LINEARREG_INTERCEPT" => Some(TaFuncMeta {
            name: "TA_LINEARREG_INTERCEPT".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_LINEARREG_SLOPE" => Some(TaFuncMeta {
            name: "TA_LINEARREG_SLOPE".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_LN" => Some(TaFuncMeta {
            name: "TA_LN".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_LOG10" => Some(TaFuncMeta {
            name: "TA_LOG10".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MA" => Some(TaFuncMeta {
            name: "TA_MA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MACD" => Some(TaFuncMeta {
            name: "TA_MACD".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 12.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 26.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInSignalPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 9.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outMACD.0".to_string(), "real".to_string()),
                ("outMACDSignal.1".to_string(), "real".to_string()),
                ("outMACDHist.2".to_string(), "real".to_string()),
            ],
        }),
        "TA_MACDEXT" => Some(TaFuncMeta {
            name: "TA_MACDEXT".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 12.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInFastMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
                TaOptInput { name: "optInSlowPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 26.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
                TaOptInput { name: "optInSignalPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 9.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInSignalMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outMACD.0".to_string(), "real".to_string()),
                ("outMACDSignal.1".to_string(), "real".to_string()),
                ("outMACDHist.2".to_string(), "real".to_string()),
            ],
        }),
        "TA_MACDFIX" => Some(TaFuncMeta {
            name: "TA_MACDFIX".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInSignalPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 9.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outMACD.0".to_string(), "real".to_string()),
                ("outMACDSignal.1".to_string(), "real".to_string()),
                ("outMACDHist.2".to_string(), "real".to_string()),
            ],
        }),
        "TA_MAMA" => Some(TaFuncMeta {
            name: "TA_MAMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastLimit".to_string(), display: String::new(), kind: "real".to_string(), default: 0.5, min: Some(0.01), max: Some(0.99) },
                TaOptInput { name: "optInSlowLimit".to_string(), display: String::new(), kind: "real".to_string(), default: 0.05, min: Some(0.01), max: Some(0.99) },
            ],
            outputs: vec![
                ("outMAMA.0".to_string(), "real".to_string()),
                ("outFAMA.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_MAVP" => Some(TaFuncMeta {
            name: "TA_MAVP".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInMinPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 2.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInMaxPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MAX" => Some(TaFuncMeta {
            name: "TA_MAX".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MAXINDEX" => Some(TaFuncMeta {
            name: "TA_MAXINDEX".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_MEDPRICE" => Some(TaFuncMeta {
            name: "TA_MEDPRICE".to_string(),
            group: "Price Transform / 价格变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MFI" => Some(TaFuncMeta {
            name: "TA_MFI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MIDPOINT" => Some(TaFuncMeta {
            name: "TA_MIDPOINT".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MIDPRICE" => Some(TaFuncMeta {
            name: "TA_MIDPRICE".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MIN" => Some(TaFuncMeta {
            name: "TA_MIN".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MININDEX" => Some(TaFuncMeta {
            name: "TA_MININDEX".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outInteger".to_string(), "integer".to_string()),
            ],
        }),
        "TA_MINMAX" => Some(TaFuncMeta {
            name: "TA_MINMAX".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outMin.0".to_string(), "real".to_string()),
                ("outMax.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_MINMAXINDEX" => Some(TaFuncMeta {
            name: "TA_MINMAXINDEX".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outMinIdx.0".to_string(), "integer".to_string()),
                ("outMaxIdx.1".to_string(), "integer".to_string()),
            ],
        }),
        "TA_MINUS_DI" => Some(TaFuncMeta {
            name: "TA_MINUS_DI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MINUS_DM" => Some(TaFuncMeta {
            name: "TA_MINUS_DM".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MOM" => Some(TaFuncMeta {
            name: "TA_MOM".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 10.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_MULT" => Some(TaFuncMeta {
            name: "TA_MULT".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_NATR" => Some(TaFuncMeta {
            name: "TA_NATR".to_string(),
            group: "Volatility Indicators / 波动率指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_OBV" => Some(TaFuncMeta {
            name: "TA_OBV".to_string(),
            group: "Volume Indicators / 成交量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_PLUS_DI" => Some(TaFuncMeta {
            name: "TA_PLUS_DI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_PLUS_DM" => Some(TaFuncMeta {
            name: "TA_PLUS_DM".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_PPO" => Some(TaFuncMeta {
            name: "TA_PPO".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 12.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowPeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 26.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInMAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ROC" => Some(TaFuncMeta {
            name: "TA_ROC".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 10.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ROCP" => Some(TaFuncMeta {
            name: "TA_ROCP".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 10.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ROCR" => Some(TaFuncMeta {
            name: "TA_ROCR".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 10.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ROCR100" => Some(TaFuncMeta {
            name: "TA_ROCR100".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 10.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_RSI" => Some(TaFuncMeta {
            name: "TA_RSI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SAR" => Some(TaFuncMeta {
            name: "TA_SAR".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInAcceleration".to_string(), display: String::new(), kind: "real".to_string(), default: 0.02, min: None, max: None },
                TaOptInput { name: "optInMaximum".to_string(), display: String::new(), kind: "real".to_string(), default: 0.2, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SAREXT" => Some(TaFuncMeta {
            name: "TA_SAREXT".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInStartValue".to_string(), display: String::new(), kind: "real".to_string(), default: 0.0, min: None, max: None },
                TaOptInput { name: "optInOffsetOnReverse".to_string(), display: String::new(), kind: "real".to_string(), default: 0.0, min: None, max: None },
                TaOptInput { name: "optInAccelerationInitLong".to_string(), display: String::new(), kind: "real".to_string(), default: 0.02, min: None, max: None },
                TaOptInput { name: "optInAccelerationLong".to_string(), display: String::new(), kind: "real".to_string(), default: 0.02, min: None, max: None },
                TaOptInput { name: "optInAccelerationMaxLong".to_string(), display: String::new(), kind: "real".to_string(), default: 0.2, min: None, max: None },
                TaOptInput { name: "optInAccelerationInitShort".to_string(), display: String::new(), kind: "real".to_string(), default: 0.02, min: None, max: None },
                TaOptInput { name: "optInAccelerationShort".to_string(), display: String::new(), kind: "real".to_string(), default: 0.02, min: None, max: None },
                TaOptInput { name: "optInAccelerationMaxShort".to_string(), display: String::new(), kind: "real".to_string(), default: 0.2, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SIN" => Some(TaFuncMeta {
            name: "TA_SIN".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SINH" => Some(TaFuncMeta {
            name: "TA_SINH".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SMA" => Some(TaFuncMeta {
            name: "TA_SMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SQRT" => Some(TaFuncMeta {
            name: "TA_SQRT".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_STDDEV" => Some(TaFuncMeta {
            name: "TA_STDDEV".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInNbDev".to_string(), display: String::new(), kind: "real".to_string(), default: 1.0, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_STOCH" => Some(TaFuncMeta {
            name: "TA_STOCH".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastK_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowK_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 3.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowK_MAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
                TaOptInput { name: "optInSlowD_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 3.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInSlowD_MAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outSlowK.0".to_string(), "real".to_string()),
                ("outSlowD.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_STOCHF" => Some(TaFuncMeta {
            name: "TA_STOCHF".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInFastK_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInFastD_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 3.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInFastD_MAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outFastK.0".to_string(), "real".to_string()),
                ("outFastD.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_STOCHRSI" => Some(TaFuncMeta {
            name: "TA_STOCHRSI".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
                TaOptInput { name: "optInFastK_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInFastD_Period".to_string(), display: String::new(), kind: "int".to_string(), default: 3.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInFastD_MAType".to_string(), display: String::new(), kind: "int-list".to_string(), default: 0.0, min: None, max: None },
            ],
            outputs: vec![
                ("outFastK.0".to_string(), "real".to_string()),
                ("outFastD.1".to_string(), "real".to_string()),
            ],
        }),
        "TA_SUB" => Some(TaFuncMeta {
            name: "TA_SUB".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_SUM" => Some(TaFuncMeta {
            name: "TA_SUM".to_string(),
            group: "Math Operators / 数学运算".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_T3" => Some(TaFuncMeta {
            name: "TA_T3".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInVFactor".to_string(), display: String::new(), kind: "real".to_string(), default: 0.7, min: Some(0.0), max: Some(1.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TAN" => Some(TaFuncMeta {
            name: "TA_TAN".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TANH" => Some(TaFuncMeta {
            name: "TA_TANH".to_string(),
            group: "Math Transform / 数学变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TEMA" => Some(TaFuncMeta {
            name: "TA_TEMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TRANGE" => Some(TaFuncMeta {
            name: "TA_TRANGE".to_string(),
            group: "Volatility Indicators / 波动率指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TRIMA" => Some(TaFuncMeta {
            name: "TA_TRIMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TRIX" => Some(TaFuncMeta {
            name: "TA_TRIX".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TSF" => Some(TaFuncMeta {
            name: "TA_TSF".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_TYPPRICE" => Some(TaFuncMeta {
            name: "TA_TYPPRICE".to_string(),
            group: "Price Transform / 价格变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_ULTOSC" => Some(TaFuncMeta {
            name: "TA_ULTOSC".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod1".to_string(), display: String::new(), kind: "int".to_string(), default: 7.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInTimePeriod2".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInTimePeriod3".to_string(), display: String::new(), kind: "int".to_string(), default: 28.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_VAR" => Some(TaFuncMeta {
            name: "TA_VAR".to_string(),
            group: "Statistic Functions / 统计函数".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 5.0, min: Some(1.0), max: Some(100000.0) },
                TaOptInput { name: "optInNbDev".to_string(), display: String::new(), kind: "real".to_string(), default: 1.0, min: None, max: None },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_WCLPRICE" => Some(TaFuncMeta {
            name: "TA_WCLPRICE".to_string(),
            group: "Price Transform / 价格变换".to_string(),
            hint: String::new(),
            opt_inputs: vec![
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_WILLR" => Some(TaFuncMeta {
            name: "TA_WILLR".to_string(),
            group: "Momentum Indicators / 动量指标".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 14.0, min: Some(2.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        "TA_WMA" => Some(TaFuncMeta {
            name: "TA_WMA".to_string(),
            group: "Overlap Studies / 重叠研究".to_string(),
            hint: String::new(),
            opt_inputs: vec![
                TaOptInput { name: "optInTimePeriod".to_string(), display: String::new(), kind: "int".to_string(), default: 30.0, min: Some(1.0), max: Some(100000.0) },
            ],
            outputs: vec![
                ("outReal".to_string(), "real".to_string()),
            ],
        }),
        _ => None,
    }
}

/// 取得某函数输出字段名（用于 `.field` 选择）。
pub fn ta_output_names(name: &str) -> Option<Vec<String>> {
    match name {
        "TA_MACD" => Some(vec!["macd".to_string(), "signal".to_string(), "hist".to_string()]),
        "TA_MACDEXT" => Some(vec!["macd".to_string(), "signal".to_string(), "hist".to_string()]),
        "TA_MACDFIX" => Some(vec!["macd".to_string(), "signal".to_string(), "hist".to_string()]),
        "TA_AROON" => Some(vec!["up".to_string(), "down".to_string()]),
        "TA_STOCH" => Some(vec!["slow_k".to_string(), "slow_d".to_string()]),
        "TA_STOCHF" => Some(vec!["fast_k".to_string(), "fast_d".to_string()]),
        "TA_MAMA" => Some(vec!["mama".to_string(), "fama".to_string()]),
        "TA_HT_PHASOR" => Some(vec!["in_phase".to_string(), "quadrature".to_string()]),
        "TA_HT_SINE" => Some(vec!["sine".to_string(), "lead_sine".to_string()]),
        "TA_MINMAX" => Some(vec!["min".to_string(), "max".to_string()]),
        "TA_MINMAXINDEX" => Some(vec!["min_idx".to_string(), "max_idx".to_string()]),
        "TA_BBANDS" => Some(vec!["upper".to_string(), "middle".to_string(), "lower".to_string()]),
        "TA_ACCBANDS" => Some(vec!["upper".to_string(), "middle".to_string(), "lower".to_string()]),
        _ => Some(vec!["outReal".to_string()]),
    }
}
