//! 生成 TA-Lib 指标双语参考文档（中英双语）。
//!
//! 运行：`cargo run --example ta_indicators_list`
//! 输出：`docs/ta-lib-indicators.bilingual.md`
//!
//! 参数表（名称 / 类型 / 默认值 / 范围）完全来自本机 TA-Lib 运行时元信息，
//! 含义说明为中文 + 英文双语，由下方 `desc()` 提供（按函数名精确匹配，
//! 未收录者回退到「分组通用说明」）。

use std::fmt::Write as _;

use wbot::indicators::ta::{list_all_functions, ta_meta};

/// 取某函数的中/英含义说明；未精确收录时按分组回退到通用说明。
fn desc(name: &str, group: &str) -> (&'static str, &'static str) {
    let en = |s: &'static str| s;
    let zh = |s: &'static str| s;
    let out = match name {
        // ---------- Overlap Studies ----------
        "BBANDS" => ("Bollinger Bands: middle = SMA(close,period); upper/lower = middle ± nbDev·σ. Mean-reversion & volatility envelope.",
                     "布林带：中轨=收盘价 SMA，上/下轨=中轨±nbDev·标准差。用于均值回归与波动率通道。"),
        "DEMA" => ("Double Exponential Moving Average — smoother/faster EMA variant reducing lag.",
                   "双指数移动平均：对 EMA 做二次平滑，进一步降低滞后。"),
        "EMA" => ("Exponential Moving Average — weights recent prices more heavily.",
                  "指数移动平均：近期价格权重更高。"),
        "HT_TRENDLINE" => ("Hilbert Transform Instantaneous Trendline — adaptive trend line via DSP.",
                          "希尔伯特变换瞬时趋势线：基于数字信号处理的自适应趋势线。"),
        "KAMA" => ("Kaufman Adaptive Moving Average — adapts speed to noise (efficiency ratio).",
                   "卡夫曼自适应均线：依据效率比在噪声中自适应调整快慢。"),
        "MA" => ("Moving Average with selectable MA type (SMA/EMA/WMA/DEMA/TEMA/TRIMA/KAMA/MAMA/T3).",
                 "移动平均：可通过 MAType 选择多种均线算法。"),
        "MAMA" => ("MESA Adaptive Moving Average — adaptive MA from Hilbert transform cycle.",
                   "MESA 自适应均线：源自希尔伯特变换周期的自适应均线。"),
        "MAVP" => ("Moving Average with Variable Period — period varies by an external cycle array.",
                   "变周期移动平均：周期随外部周期数组变化。"),
        "MIDPOINT" => ("MidPoint over period — average of highest and lowest in the window.",
                       "中值：窗口内最高与最低的中间值。"),
        "MIDPRICE" => ("MidPrice over period — (highest high + lowest low)/2 over the window.",
                       "中价：窗口内最高价与最低价的平均值。"),
        "SAR" => ("Parabolic SAR — trailing stop-and-reverse indicator for trend following.",
                  "抛物线 SAR：趋势跟踪的止损反转指标。"),
        "SAREXT" => ("Parabolic SAR Extended — SAR with configurable acceleration extremes.",
                     "扩展抛物线 SAR：可配置加速极值参数的 SAR。"),
        "SMA" => ("Simple Moving Average — arithmetic mean of close over period.",
                  "简单移动平均：窗口内收盘价的算术平均。"),
        "T3" => ("T3 Moving Average — triple-smoothed EMA with volume factor.",
                 "T3 均线：带量价因子的三重平滑 EMA。"),
        "TEMA" => ("Triple Exponential Moving Average — further lag reduction vs DEMA.",
                   "三重指数移动平均：相较 DEMA 进一步降低滞后。"),
        "TRIMA" => ("Triangular Moving Average — double-smoothed (symmetrical) SMA.",
                    "三角移动平均：对称双重平滑的 SMA。"),
        "WMA" => ("Weighted Moving Average — linearly weighted, recent prices heavier.",
                  "加权移动平均：线性加权，近期权重更大。"),

        // ---------- Momentum Indicators ----------
        "ADX" => ("Average Directional Movement Index — trend strength (not direction), 0–100.",
                  "平均趋向指数：衡量趋势强度（非方向），0–100。"),
        "ADXR" => ("ADX Rating — ADX normalized against its value `period` bars ago.",
                   "ADX 评级：将 ADX 与 `period` 根前的自身值归一化比较。"),
        "APO" => ("Absolute Price Oscillator — EMA(fast)-EMA(slow) of close.",
                  "绝对价格振荡器：收盘价的快慢 EMA 之差。"),
        "AROON" => ("Aroon — outputs Aroon-Up & Aroon-Down measuring time since recent extrema.",
                    "阿隆指标：输出阿隆上/下，衡量距近期极值的时间。"),
        "AROONOSC" => ("Aroon Oscillator — Aroon-Up minus Aroon-Down.",
                       "阿隆振荡器：阿隆上减阿隆下。"),
        "BOP" => ("Balance Of Power — close vs open dominance: (close-open)/(high-low).",
                  "力量平衡：以 (收-开)/(高-低) 衡量多空主导。"),
        "CCI" => ("Commodity Channel Index — deviation of price from its moving average in σ.",
                 "顺势指标：价格偏离其移动平均的标准差倍数。"),
        "CMO" => ("Chande Momentum Oscillator — RSI-like, range -100..100.",
                  "钱德动量振荡器：类 RSI，范围 -100..100。"),
        "DX" => ("Directional Movement Index — precursor of ADX (abs DM differential).",
                "动向指数：ADX 的前置量（方向运动差的绝对值）。"),
        "MACD" => ("Moving Average Convergence/Divergence — DIF, DEA(signal), HIST.",
                  "指数平滑异同移动平均：含 DIF、DEA(信号线)、HIST 三输出。"),
        "MACDEXT" => ("MACD with configurable fast/slow/signal periods (MAType is fixed to EMA in adaq-talib).",
                      "可配置快/慢/信号周期的 MACD（adaq-talib 中 MAType 固定为 EMA）。"),
        "MACDFIX" => ("MACD Fix — MACD with a fixed 9-period signal (MAType fixed to EMA in adaq-talib).",
                      "固定信号周期的 MACD：信号线固定为 9 周期，MAType 固定为 EMA。"),
        "MFI" => ("Money Flow Index — RSI weighted by volume (0–100).",
                  "资金流量指数：以成交量加权的 RSI（0–100）。"),
        "MINUS_DI" => ("Minus Directional Indicator — downward directional movement.",
                       "负向方向指标：向下方向运动。"),
        "MINUS_DM" => ("Minus Directional Movement — raw downward movement.",
                       "负向方向运动（原始值）。"),
        "MOM" => ("Momentum — close(t) - close(t-period).",
                  "动量：当前收盘价减去 period 根前收盘价。"),
        "PLUS_DI" => ("Plus Directional Indicator — upward directional movement.",
                      "正向方向指标：向上方向运动。"),
        "PLUS_DM" => ("Plus Directional Movement — raw upward movement.",
                      "正向方向运动（原始值）。"),
        "PPO" => ("Percentage Price Oscillator — (EMAfast-EMAslow)/EMAslow·100.",
                 "百分比价格振荡器：快慢 EMA 之差占慢线百分比。"),
        "ROC" => ("Rate Of Change — (close(t)-close(t-period))/close(t-period)·100.",
                 "变动率：价格相对 period 根前的百分比变化。"),
        "ROCP" => ("Rate Of Change Percentage — (price/prev)-1.",
                   "变动率（比例）：当前价/前期价 - 1。"),
        "ROCR" => ("Rate Of Change Ratio — close(t)/close(t-period).",
                   "变动率比值：当前价 / period 根前价。"),
        "ROCR100" => ("Rate Of Change Ratio ×100 — close(t)/close(t-period)·100.",
                      "变动率比值×100：当前价 / 前期价 × 100。"),
        "RSI" => ("Relative Strength Index — Wilder momentum oscillator, 0–100.",
                 "相对强弱指数：Wilder 动量振荡器，0–100。"),
        "STOCH" => ("Stochastic — %K and %D slow stochastic from high/low/close.",
                    "随机指标：由高/低/收派生的慢速 %K 与 %D。"),
        "STOCHF" => ("Stochastic Fast — fast %K and %D stochastic.",
                     "快速随机指标：快速 %K 与 %D。"),
        "STOCHRSI" => ("Stochastic RSI — applies stochastic formula to RSI itself.",
                       "随机 RSI：把随机指标公式作用于 RSI。"),
        "TRIX" => ("Trix — triple-smoothed EMA rate of change, trend/filter.",
                   "三重平滑 EMA 的变化率，用于趋势与过滤。"),
        "ULTOSC" => ("Ultimate Oscillator — weighted average of 3 different-period ROIs.",
                     "终极振荡器：三个不同周期 ROI 的加权平均。"),
        "WILLR" => ("Williams' %R — inverse of Stochastic, range -100..0.",
                    "威廉指标：随机指标的逆，范围 -100..0。"),

        // ---------- Volume Indicators ----------
        "AD" => ("Chaikin A/D Line — cumulative volume weighted by close location in range.",
                "累积/派发线：以收盘价在高低区间的位置加权的累计成交量。"),
        "ADOSC" => ("Chaikin A/D Oscillator — fast A/D MA minus slow A/D MA.",
                   "A/D 振荡器：快慢 A/D 均线之差。"),
        "OBV" => ("On Balance Volume — cumulative volume signed by price direction.",
                 "能量潮：按价格方向带符号累计成交量。"),

        // ---------- Volatility Indicators ----------
        "ATR" => ("Average True Range — Wilder average of True Range, measures volatility.",
                 "平均真实波幅：真实波幅的 Wilder 均值，衡量波动率。"),
        "NATR" => ("Normalized ATR — ATR divided by close, scale-independent.",
                   "归一化 ATR：ATR 除以收盘价，无量纲。"),
        "TRANGE" => ("True Range — high-low, |high-prevClose|, |low-prevClose| (max).",
                     "真实波幅：三者最大值。"),

        // ---------- Price Transform ----------
        "AVGPRICE" => ("Average Price — (high+low+open+close)/4.",
                       "平均价格：(高+低+开+收)/4。"),
        "MEDPRICE" => ("Median Price — (high+low)/2.",
                       "中价：(高+低)/2。"),
        "TYPPRICE" => ("Typical Price — (high+low+close)/3.",
                      "典型价格：(高+低+收)/3。"),
        "WCLPRICE" => ("Weighted Close Price — (high+low+close·2)/4.",
                      "加权收盘价：(高+低+收·2)/4。"),

        // ---------- Cycle Indicators ----------
        "HT_DCPERIOD" => ("Hilbert Transform Dominant Cycle Period — dominant cycle in bars.",
                         "希尔伯特变换主导周期：以根数表示的主导周期。"),
        "HT_DCPHASE" => ("Hilbert Transform Dominant Cycle Phase — phase of dominant cycle.",
                        "希尔伯特变换主导周期相位。"),
        "HT_PHASOR" => ("Hilbert Transform Phasor — in-phase & quadrature components.",
                       "希尔伯特变换相量：同相与正交分量。"),
        "HT_SINE" => ("Hilbert Transform Sine — sine & lead-sine of dominant cycle.",
                     "希尔伯特变换正弦：主导周期的正弦与超前正弦。"),
        "HT_TRENDMODE" => ("Hilbert Transform Trend vs Cycle Mode — 1=trend, 0=cycle.",
                          "希尔伯特变换趋势/周期模式：1 为趋势，0 为周期。"),

        // ---------- Statistic Functions ----------
        "BETA" => ("Beta — slope of linear regression of asset vs market (both = close here).",
                  "贝塔：资产相对市场的线性回归斜率（本封装两者均取收盘价）。"),
        "CORREL" => ("Pearson Correlation — correlation of two series over period.",
                    "皮尔逊相关系数：两序列在窗口内的相关性。"),
        "LINEARREG" => ("Linear Regression — endpoint value of least-squares fit line.",
                       "线性回归：最小二乘拟合线末端值。"),
        "LINEARREG_ANGLE" => ("Linear Regression Angle — slope angle (degrees) of fitted line.",
                             "线性回归角度：拟合线斜率（度）。"),
        "LINEARREG_INTERCEPT" => ("Linear Regression Intercept — y-intercept of fitted line.",
                                 "线性回归截距。"),
        "LINEARREG_SLOPE" => ("Linear Regression Slope — per-bar slope of fitted line.",
                             "线性回归斜率：每根变化量。"),
        "STDDEV" => ("Standard Deviation — σ of close around its SMA (×nbDev).",
                    "标准差：收盘价相对 SMA 的离散度（×nbDev）。"),
        "TSF" => ("Time Series Forecast — regression line projected one bar ahead.",
                 "时间序列预测：回归线向前投影一根。"),
        "VAR" => ("Variance — σ² of close around its SMA.",
                 "方差：收盘价相对 SMA 的离散平方。"),

        // ---------- Math Transform ----------
        "ACOS" => ("Arc Cosine — acos(x) element-wise.", "反余弦（逐元素）。"),
        "ASIN" => ("Arc Sine — asin(x) element-wise.", "反正弦（逐元素）。"),
        "ATAN" => ("Arc Tangent — atan(x) element-wise.", "反正切（逐元素）。"),
        "CEIL" => ("Ceiling — smallest integer ≥ x.", "向上取整。"),
        "COS" => ("Cosine — cos(x) element-wise.", "余弦（逐元素）。"),
        "COSH" => ("Hyperbolic Cosine — cosh(x) element-wise.", "双曲余弦（逐元素）。"),
        "EXP" => ("Exponential — e^x element-wise.", "指数（逐元素）。"),
        "FLOOR" => ("Floor — largest integer ≤ x.", "向下取整。"),
        "LN" => ("Natural Log — ln(x) element-wise.", "自然对数（逐元素）。"),
        "LOG10" => ("Base-10 Log — log10(x) element-wise.", "常用对数（逐元素）。"),
        "SIN" => ("Sine — sin(x) element-wise.", "正弦（逐元素）。"),
        "SINH" => ("Hyperbolic Sine — sinh(x) element-wise.", "双曲正弦（逐元素）。"),
        "SQRT" => ("Square Root — √x element-wise.", "平方根（逐元素）。"),
        "TAN" => ("Tangent — tan(x) element-wise.", "正切（逐元素）。"),
        "TANH" => ("Hyperbolic Tangent — tanh(x) element-wise.", "双曲正切（逐元素）。"),

        // ---------- Math Operators ----------
        "ADD" => ("Add — inReal + second price series (both = close here).",
                 "相加：两序列逐元素相加（本封装两者均取收盘价）。"),
        "DIV" => ("Divide — inReal / second price series.", "相除（逐元素）。"),
        "MAX" => ("Max over period — highest value in the window.", "窗口内最大值。"),
        "MAXINDEX" => ("Max Index — bar index of the max within the window (integer).",
                      "窗口内最大值所在位置（整数输出）。"),
        "MIN" => ("Min over period — lowest value in the window.", "窗口内最小值。"),
        "MININDEX" => ("Min Index — bar index of the min within the window (integer).",
                      "窗口内最小值所在位置（整数输出）。"),
        "MINMAX" => ("Min & Max over period — two outputs: min then max.",
                     "窗口内最小与最大（两输出）。"),
        "MINMAXINDEX" => ("Min & Max Index — indices of min and max (two integer outputs).",
                         "窗口内最小/最大值位置（两整数输出）。"),
        "MULT" => ("Multiply — inReal × second price series.", "相乘（逐元素）。"),
        "SUB" => ("Subtract — inReal − second price series.", "相减（逐元素）。"),
        "SUM" => ("Sum over period — total of values in the window.", "窗口内求和。"),

        // ---------- Pattern Recognition (CDL*) ----------
        "CDL2CROWS" => ("Two Crows bearish reversal.", "两只乌鸦：看跌反转。"),
        "CDL3BLACKCROWS" => ("Three Black Crows bearish reversal.", "三只乌鸦：看跌反转。"),
        "CDL3INSIDE" => ("Three Inside Up/Down reversal.", "内困三日：反转形态。"),
        "CDL3LINESTRIKE" => ("Three-Line Strike (bullish/bearish).", "三线打击。"),
        "CDL3OUTSIDE" => ("Three Outside Up/Down reversal.", "外困三日：反转形态。"),
        "CDL3STARSINSOUTH" => ("Three Stars In The South bullish.", "南方三星：看涨。"),
        "CDL3WHITESOLDIERS" => ("Three Advancing White Soldiers bullish.", "白色三兵：看涨。"),
        "CDLABANDONEDBABY" => ("Abandoned Baby reversal (gap doji).", "弃婴：反转形态。"),
        "CDLADVANCEBLOCK" => ("Advance Block bearish.", "推进块：看跌。"),
        "CDLBELTHOLD" => ("Belt-hold line (bullish/bearish).", "腰带线。"),
        "CDLBREAKAWAY" => ("Breakaway reversal.", "脱离形态：反转。"),
        "CDLCLOSINGMARUBOZU" => ("Closing Marubozu (no upper shadow).", "收盘光头线。"),
        "CDLCONCEALBABYSWALL" => ("Concealing Baby Swallow bullish.", "藏婴吞没：看涨。"),
        "CDLCOUNTERATTACK" => ("Counterattack lines reversal.", "反击线：反转。"),
        "CDLDARKCLOUDCOVER" => ("Dark Cloud Cover bearish reversal.", "乌云盖顶：看跌反转。"),
        "CDLDOJI" => ("Doji indecision.", "十字星：犹豫。"),
        "CDLDOJISTAR" => ("Doji Star reversal.", "十字星：反转。"),
        "CDLDRAGONFLYDOJI" => ("Dragonfly Doji bullish reversal.", "蜻蜓十字：看涨反转。"),
        "CDLENGULFING" => ("Engulfing Pattern (bullish/bearish).", "吞没形态。"),
        "CDLEVENINGDOJISTAR" => ("Evening Doji Star bearish reversal.", "黄昏十字星：看跌反转。"),
        "CDLEVENINGSTAR" => ("Evening Star bearish reversal.", "黄昏之星：看跌反转。"),
        "CDLGAPSIDESIDEWHITE" => ("Up/Down-gap Side-by-Side White lines.", "并列阳线缺口。"),
        "CDLGRAVESTONEDOJI" => ("Gravestone Doji bearish reversal.", "墓碑十字：看跌反转。"),
        "CDLHAMMER" => ("Hammer bullish reversal.", "锤头：看涨反转。"),
        "CDLHANGINGMAN" => ("Hanging Man bearish reversal.", "上吊线：看跌反转。"),
        "CDLHARAMI" => ("Harami reversal (bullish/bearish).", "孕线：反转。"),
        "CDLHARAMICROSS" => ("Harami Cross reversal.", "十字孕线：反转。"),
        "CDLHIGHWAVE" => ("High-Wave indecision.", "高价波动：犹豫。"),
        "CDLHIKKAKE" => ("Hikkake Pattern (bullish/bearish).", "陷阱形态。"),
        "CDLHIKKAKEMOD" => ("Modified Hikkake Pattern.", "改良陷阱形态。"),
        "CDLHOMINGPIGEON" => ("Homing Pigeon bullish.", "家鸽：看涨。"),
        "CDLIDENTICAL3CROWS" => ("Identical Three Crows bearish.", "等同三乌鸦：看跌。"),
        "CDLINNECK" => ("In-Neck reversal.", "颈内线：反转。"),
        "CDLINVERTEDHAMMER" => ("Inverted Hammer bullish reversal.", "倒锤头：看涨反转。"),
        "CDLKICKING" => ("Kicking (bullish/bearish) gap lines.", "反冲形态。"),
        "CDLKICKINGBYLENGTH" => ("Kicking by Length.", "按长度反冲。"),
        "CDLLADDERBOTTOM" => ("Ladder Bottom bullish.", "梯底：看涨。"),
        "CDLLONGLEGGEDDOJI" => ("Long-Legged Doji indecision.", "长腿十字：犹豫。"),
        "CDLLONGLINE" => ("Long Line candle.", "长实体线。"),
        "CDLMARUBOZU" => ("Marubozu (no shadows).", "光头光脚线。"),
        "CDLMATCHINGLOW" => ("Matching Low bullish.", "呼应低：看涨。"),
        "CDLMATHOLD" => ("Mat Hold continuation (bullish).", "铺垫：看涨持续。"),
        "CDLMORNINGDOJISTAR" => ("Morning Doji Star bullish reversal.", "早晨十字星：看涨反转。"),
        "CDLMORNINGSTAR" => ("Morning Star bullish reversal.", "早晨之星：看涨反转。"),
        "CDLONNECK" => ("On-Neck reversal.", "颈上线：反转。"),
        "CDLPIERCING" => ("Piercing Pattern bullish reversal.", "刺透形态：看涨反转。"),
        "CDLRICKSHAWDOJI" => ("Rickshaw Doji indecision.", "黄包车夫十字：犹豫。"),
        "CDLRISEFALL3METHODS" => ("Rising/Falling Three Methods continuation.", "三法形态：持续。"),
        "CDLSEPARATINGLINES" => ("Separating Lines continuation.", "分离线：持续。"),
        "CDLSHOOTINGSTAR" => ("Shooting Star bearish reversal.", "射击之星：看跌反转。"),
        "CDLSHORTLINE" => ("Short Line candle.", "短实体线。"),
        "CDLSPINNINGTOP" => ("Spinning Top indecision.", "纺锤线：犹豫。"),
        "CDLSTALLEDPATTERN" => ("Stalled Pattern bearish.", "停滞形态：看跌。"),
        "CDLSTICKSANDWICH" => ("Stick Sandwich bullish.", " Stick 夹心：看涨。"),
        "CDLTAKURI" => ("Takuri (dragonfly doji variant) bullish.", "垂柳线：看涨。"),
        "CDLTASUKIGAP" => ("Tasuki Gap continuation.", "跳空并列：持续。"),
        "CDLTHRUSTING" => ("Thrusting continuation (bearish).", "插入线：看跌持续。"),
        "CDLTRISTAR" => ("Tristar (three doji) reversal.", "三星：反转。"),
        "CDLUNIQUE3RIVER" => ("Unique 3 River bottom bullish.", "独特三河底：看涨。"),
        "CDLUPSIDEGAP2CROWS" => ("Upside Gap Two Crows bearish.", "上跳双鸦：看跌。"),
        "CDLXSIDEGAP3METHODS" => ("Upside/Downside Gap Three Methods.", "跳空三法。"),

        // ---------- fallback ----------
        _ => {
            let (e, z) = group_fallback(group);
            (en(e), zh(z))
        }
    };
    out
}

fn group_fallback(group: &str) -> (&'static str, &'static str) {
    match group {
        "Overlap Studies" => (
            "Overlay/ moving-average study plotted on the price chart.",
            "在价格图上叠加的均线/重叠类指标。",
        ),
        "Momentum Indicators" => (
            "Oscillator measuring the speed or magnitude of price movements.",
            "衡量价格变动速度或幅度的振荡指标。",
        ),
        "Volume Indicators" => (
            "Indicator derived from traded volume / capital flow.",
            "由成交量或资金流向派生的指标。",
        ),
        "Volatility Indicators" => (
            "Estimator of market volatility from the trading range.",
            "基于波动区间的市场波动率估计指标。",
        ),
        "Price Transform" => (
            "Derives a representative price from the OHLC tuple.",
            "由 OHLC 派生代表性价格的指标。",
        ),
        "Cycle Indicators" => (
            "Hilbert-transform based dominant-cycle estimators.",
            "基于希尔伯特变换的主导周期估计指标。",
        ),
        "Pattern Recognition" => (
            "Candlestick pattern detector; output is 0 / 100 / -100 (integer).",
            "蜡烛图形态识别；输出为 0 / 100 / -100（整数）。",
        ),
        "Statistic Functions" => (
            "Statistical regression / dispersion function over a window.",
            "窗口内的统计回归 / 离散度函数。",
        ),
        "Math Transform" => (
            "Element-wise mathematical transform applied to the price series.",
            "对价格序列逐元素施加的数学变换。",
        ),
        "Math Operators" => (
            "Element-wise mathematical operator (add/sub/mul/div/min/max/sum).",
            "逐元素数学运算（加/减/乘/除/最值/求和）。",
        ),
        _ => (
            "TA-Lib function (see TA-Lib documentation for details).",
            "TA-Lib 函数（详见 TA-Lib 官方文档）。",
        ),
    }
}

fn matype_legend() -> &'static str {
    "MAType 取值（整数，用于 MA / MACD / BBANDS 等的可选均线类型）：\n\
    - 0 = SMA（简单）  1 = EMA（指数）  2 = WMA（加权）  3 = DEMA（双指数）\n\
    - 4 = TEMA（三重指数）  5 = TRIMA（三角）  6 = KAMA（自适应）  7 = MAMA（MESA）  8 = T3"
}

fn bilingual_group(group: &str) -> (&str, &str) {
    match group {
        "Overlap Studies" => ("Overlap Studies", "重叠研究"),
        "Momentum Indicators" => ("Momentum Indicators", "动量指标"),
        "Volume Indicators" => ("Volume Indicators", "成交量指标"),
        "Volatility Indicators" => ("Volatility Indicators", "波动率指标"),
        "Price Transform" => ("Price Transform", "价格变换"),
        "Cycle Indicators" => ("Cycle Indicators", "周期指标"),
        "Pattern Recognition" => ("Pattern Recognition", "形态识别"),
        "Statistic Functions" => ("Statistic Functions", "统计函数"),
        "Math Transform" => ("Math Transform", "数学变换"),
        "Math Operators" => ("Math Operators", "数学运算"),
        other => (other, other),
    }
}

fn main() {
    let all = list_all_functions();
    let mut grouped: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (name, group) in &all {
        grouped.entry(group.clone()).or_default().push(name.clone());
    }

    let mut md = String::new();
    let _ = writeln!(md, "# TA-Lib 指标参考手册 / TA-Lib Indicators Reference");
    let _ = writeln!(
        md,
        "\n> 本程序通过 **adaq-talib**（纯 Rust、零 FFI 的 TA-Lib 0.7.1 重实现）对接其**全部**函数（共 {} 个），可在策略 DSL 中以 `TA_<FUNC>(...)` 形式直接引用。原 C 版 TA-Lib 已移除，无需本机安装任何 C 库。\n\
         > This program exposes **all** TA-Lib functions ({} total) via **adaq-talib** — a pure-Rust, zero-FFI reimplementation of TA-Lib 0.7.1 — and references them in the strategy DSL as `TA_<FUNC>(...)`. The old C TA-Lib has been removed; no native C library needs to be installed.\n",
        all.len(),
        all.len()
    );
    let _ = writeln!(
        md,
        "## 通用约定 / Conventions\n\
        - **参数写法 / Parameters**: `TA_RSI(close, 14)` —— 第一个参数为价格来源（adaq-talib 按函数自身价格掩码取用，多数用收盘价），其余为可选参数；缺省时取 TA-Lib 默认值。\n\
        - **多输出选择 / Multi-output**: 用 `.0 / .1 / .2` 或输出名选择，如 `TA_MACD(close,12,26,9).hist`、`TA_BBANDS(close,20,2).upper`。默认取首个输出。\n\
        - **前导值 / Lookback**: 序列前若干根不足计算长度，输出为 `NaN`（不参与信号比较）。\n\
        - **后端兼容性 / Backend note**: `TA_MACDEXT` 与 `TA_MACDFIX` 在 adaq-talib 中固定 MAType 为 EMA，故仅周期类参数生效（快/慢/信号周期）；其余 159 个函数参数与 TA-Lib 完全对应。\n\
          / `TA_MACDEXT` and `TA_MACDFIX` fix the moving-average type (MAType) to EMA in adaq-talib, so only the period parameters take effect; the other 159 functions match TA-Lib exactly.\n"
    );
    let _ = writeln!(md, "{}\n", matype_legend());

    for (group, names) in &grouped {
        let (g_en, g_zh) = match group.as_str() {
            "Overlap Studies" => ("Overlap Studies", "重叠研究"),
            "Momentum Indicators" => ("Momentum Indicators", "动量指标"),
            "Volume Indicators" => ("Volume Indicators", "成交量指标"),
            "Volatility Indicators" => ("Volatility Indicators", "波动率指标"),
            "Price Transform" => ("Price Transform", "价格变换"),
            "Cycle Indicators" => ("Cycle Indicators", "周期指标"),
            "Pattern Recognition" => ("Pattern Recognition", "形态识别"),
            "Statistic Functions" => ("Statistic Functions", "统计函数"),
            "Math Transform" => ("Math Transform", "数学变换"),
            "Math Operators" => ("Math Operators", "数学运算"),
            other => (other, other),
        };
        let _ = writeln!(md, "\n---\n\n## {} / {}\n", g_en, g_zh);
        let mut sorted = names.clone();
        sorted.sort();
        for name in &sorted {
            let meta = match ta_meta(name) {
                Some(m) => m,
                None => continue,
            };
            let (en, zh) = desc(name, &meta.group);
            let (g_en, g_zh) = bilingual_group(&meta.group);
            let _ = writeln!(md, "### {}\n", meta.name);
            let _ = writeln!(
                md,
                "- **分组 / Group**: {} / {}\n\
                 - **含义（中文）**: {}\n\
                 - **Meaning (EN)**: {}\n",
                g_en, g_zh, zh, en
            );
            if !meta.opt_inputs.is_empty() {
                let _ = writeln!(
                    md,
                    "- **参数 / Parameters**:\n\
                     | 名称 Name | 类型 Type | 默认 Default | 范围 Range |\n\
                     |---|---|---|---|"
                );
                for o in &meta.opt_inputs {
                    let range = match (o.min, o.max) {
                        (Some(a), Some(b)) => {
                            let ca = if a.abs() > 1e20 {
                                "无限制".to_string()
                            } else {
                                fmt_num(a)
                            };
                            let cb = if b.abs() > 1e20 {
                                "无限制".to_string()
                            } else {
                                fmt_num(b)
                            };
                            format!("{}..{}", ca, cb)
                        }
                        _ => match o.kind.as_str() {
                            "int-list" => "离散整数列表 / discrete integer list".to_string(),
                            "real-list" => "离散实数列表 / discrete real list".to_string(),
                            _ => "—".to_string(),
                        },
                    };
                    let _ = writeln!(
                        md,
                        "| {} | {} | {} | {} |",
                        o.name,
                        o.kind,
                        fmt_num(o.default),
                        range
                    );
                }
                let _ = writeln!(md);
            } else {
                let _ = writeln!(md, "- **参数 / Parameters**: 无（无可选参数） / none\n");
            }
            let outs: Vec<String> = meta
                .outputs
                .iter()
                .enumerate()
                .map(|(i, (n, t))| format!("`{}{}` ({})", n, if meta.outputs.len() > 1 { format!(".{}", i) } else { String::new() }, t))
                .collect();
            let _ = writeln!(md, "- **输出 / Outputs**: {}\n", outs.join(", "));
            let dsl_params: String = meta
                .opt_inputs
                .iter()
                .map(|o| fmt_num(o.default))
                .collect::<Vec<_>>()
                .join(", ");
            let dsl = if dsl_params.is_empty() {
                format!("TA_{}(close)", meta.name)
            } else {
                format!("TA_{}(close, {})", meta.name, dsl_params)
            };
            let _ = writeln!(md, "- **策略示例 / DSL**: `{}`\n", dsl);
        }
    }

    let out_path = std::path::Path::new("docs/ta-lib-indicators.bilingual.md");
    std::fs::create_dir_all("docs").ok();
    std::fs::write(out_path, md).expect("写入文档失败");
    println!(
        "已生成 TA-Lib 双语参考文档 -> {} （覆盖 {} 个函数）",
        out_path.display(),
        all.len()
    );
}

fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}
