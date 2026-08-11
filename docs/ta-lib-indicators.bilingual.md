# TA-Lib 指标参考手册 / TA-Lib Indicators Reference

> 本程序通过 **adaq-talib**（纯 Rust、零 FFI 的 TA-Lib 0.7.1 重实现）对接其**全部**函数（共 161 个），可在策略 DSL 中以 `TA_<FUNC>(...)` 形式直接引用。原 C 版 TA-Lib 已移除，无需本机安装任何 C 库。
> This program exposes **all** TA-Lib functions (161 total) via **adaq-talib** — a pure-Rust, zero-FFI reimplementation of TA-Lib 0.7.1 — and references them in the strategy DSL as `TA_<FUNC>(...)`. The old C TA-Lib has been removed; no native C library needs to be installed.

## 通用约定 / Conventions
- **参数写法 / Parameters**: `TA_RSI(close, 14)` —— 第一个参数为价格来源（adaq-talib 按函数自身价格掩码取用，多数用收盘价），其余为可选参数；缺省时取 TA-Lib 默认值。
- **多输出选择 / Multi-output**: 用 `.0 / .1 / .2` 或输出名选择，如 `TA_MACD(close,12,26,9).hist`、`TA_BBANDS(close,20,2).upper`。默认取首个输出。
- **前导值 / Lookback**: 序列前若干根不足计算长度，输出为 `NaN`（不参与信号比较）。
- **后端兼容性 / Backend note**: `TA_MACDEXT` 与 `TA_MACDFIX` 在 adaq-talib 中固定 MAType 为 EMA，故仅周期类参数生效（快/慢/信号周期）；其余 159 个函数参数与 TA-Lib 完全对应。
/ `TA_MACDEXT` and `TA_MACDFIX` fix the moving-average type (MAType) to EMA in adaq-talib, so only the period parameters take effect; the other 159 functions match TA-Lib exactly.

MAType 取值（整数，用于 MA / MACD / BBANDS 等的可选均线类型）：
- 0 = SMA（简单）  1 = EMA（指数）  2 = WMA（加权）  3 = DEMA（双指数）
- 4 = TEMA（三重指数）  5 = TRIMA（三角）  6 = KAMA（自适应）  7 = MAMA（MESA）  8 = T3


---

## Cycle Indicators / 周期指标 / Cycle Indicators / 周期指标

### TA_HT_DCPERIOD

- **分组 / Group**: Cycle Indicators / 周期指标 / Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换主导周期：以根数表示的主导周期。
- **Meaning (EN)**: Hilbert Transform Dominant Cycle Period — dominant cycle in bars.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_HT_DCPERIOD(close)`

### TA_HT_DCPHASE

- **分组 / Group**: Cycle Indicators / 周期指标 / Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换主导周期相位。
- **Meaning (EN)**: Hilbert Transform Dominant Cycle Phase — phase of dominant cycle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_HT_DCPHASE(close)`

### TA_HT_PHASOR

- **分组 / Group**: Cycle Indicators / 周期指标 / Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换相量：同相与正交分量。
- **Meaning (EN)**: Hilbert Transform Phasor — in-phase & quadrature components.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInPhase.0.0` (real), `outQuadrature.1.1` (real)

- **策略示例 / DSL**: `TA_TA_HT_PHASOR(close)`

### TA_HT_SINE

- **分组 / Group**: Cycle Indicators / 周期指标 / Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换正弦：主导周期的正弦与超前正弦。
- **Meaning (EN)**: Hilbert Transform Sine — sine & lead-sine of dominant cycle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outSine.0.0` (real), `outLeadSine.1.1` (real)

- **策略示例 / DSL**: `TA_TA_HT_SINE(close)`

### TA_HT_TRENDMODE

- **分组 / Group**: Cycle Indicators / 周期指标 / Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换趋势/周期模式：1 为趋势，0 为周期。
- **Meaning (EN)**: Hilbert Transform Trend vs Cycle Mode — 1=trend, 0=cycle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_HT_TRENDMODE(close)`


---

## Math Operators / 数学运算 / Math Operators / 数学运算

### TA_ADD

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 相加：两序列逐元素相加（本封装两者均取收盘价）。
- **Meaning (EN)**: Add — inReal + second price series (both = close here).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ADD(close)`

### TA_DIV

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 相除（逐元素）。
- **Meaning (EN)**: Divide — inReal / second price series.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_DIV(close)`

### TA_MAX

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内最大值。
- **Meaning (EN)**: Max over period — highest value in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MAX(close, 30)`

### TA_MAXINDEX

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内最大值所在位置（整数输出）。
- **Meaning (EN)**: Max Index — bar index of the max within the window (integer).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_MAXINDEX(close, 30)`

### TA_MIN

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内最小值。
- **Meaning (EN)**: Min over period — lowest value in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MIN(close, 30)`

### TA_MININDEX

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内最小值所在位置（整数输出）。
- **Meaning (EN)**: Min Index — bar index of the min within the window (integer).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_MININDEX(close, 30)`

### TA_MINMAX

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内最小与最大（两输出）。
- **Meaning (EN)**: Min & Max over period — two outputs: min then max.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outMin.0.0` (real), `outMax.1.1` (real)

- **策略示例 / DSL**: `TA_TA_MINMAX(close, 30)`

### TA_MINMAXINDEX

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内最小/最大值位置（两整数输出）。
- **Meaning (EN)**: Min & Max Index — indices of min and max (two integer outputs).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outMinIdx.0.0` (integer), `outMaxIdx.1.1` (integer)

- **策略示例 / DSL**: `TA_TA_MINMAXINDEX(close, 30)`

### TA_MULT

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 相乘（逐元素）。
- **Meaning (EN)**: Multiply — inReal × second price series.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MULT(close)`

### TA_SUB

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 相减（逐元素）。
- **Meaning (EN)**: Subtract — inReal − second price series.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SUB(close)`

### TA_SUM

- **分组 / Group**: Math Operators / 数学运算 / Math Operators / 数学运算
- **含义（中文）**: 窗口内求和。
- **Meaning (EN)**: Sum over period — total of values in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SUM(close, 30)`


---

## Math Transform / 数学变换 / Math Transform / 数学变换

### TA_ACOS

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 反余弦（逐元素）。
- **Meaning (EN)**: Arc Cosine — acos(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ACOS(close)`

### TA_ASIN

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 反正弦（逐元素）。
- **Meaning (EN)**: Arc Sine — asin(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ASIN(close)`

### TA_ATAN

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 反正切（逐元素）。
- **Meaning (EN)**: Arc Tangent — atan(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ATAN(close)`

### TA_CEIL

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 向上取整。
- **Meaning (EN)**: Ceiling — smallest integer ≥ x.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_CEIL(close)`

### TA_COS

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 余弦（逐元素）。
- **Meaning (EN)**: Cosine — cos(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_COS(close)`

### TA_COSH

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 双曲余弦（逐元素）。
- **Meaning (EN)**: Hyperbolic Cosine — cosh(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_COSH(close)`

### TA_EXP

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 指数（逐元素）。
- **Meaning (EN)**: Exponential — e^x element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_EXP(close)`

### TA_FLOOR

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 向下取整。
- **Meaning (EN)**: Floor — largest integer ≤ x.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_FLOOR(close)`

### TA_LN

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 自然对数（逐元素）。
- **Meaning (EN)**: Natural Log — ln(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_LN(close)`

### TA_LOG10

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 常用对数（逐元素）。
- **Meaning (EN)**: Base-10 Log — log10(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_LOG10(close)`

### TA_SIN

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 正弦（逐元素）。
- **Meaning (EN)**: Sine — sin(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SIN(close)`

### TA_SINH

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 双曲正弦（逐元素）。
- **Meaning (EN)**: Hyperbolic Sine — sinh(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SINH(close)`

### TA_SQRT

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 平方根（逐元素）。
- **Meaning (EN)**: Square Root — √x element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SQRT(close)`

### TA_TAN

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 正切（逐元素）。
- **Meaning (EN)**: Tangent — tan(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TAN(close)`

### TA_TANH

- **分组 / Group**: Math Transform / 数学变换 / Math Transform / 数学变换
- **含义（中文）**: 双曲正切（逐元素）。
- **Meaning (EN)**: Hyperbolic Tangent — tanh(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TANH(close)`


---

## Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标

### TA_ADX

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 平均趋向指数：衡量趋势强度（非方向），0–100。
- **Meaning (EN)**: Average Directional Movement Index — trend strength (not direction), 0–100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ADX(close, 14)`

### TA_ADXR

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: ADX 评级：将 ADX 与 `period` 根前的自身值归一化比较。
- **Meaning (EN)**: ADX Rating — ADX normalized against its value `period` bars ago.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ADXR(close, 14)`

### TA_APO

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 绝对价格振荡器：收盘价的快慢 EMA 之差。
- **Meaning (EN)**: Absolute Price Oscillator — EMA(fast)-EMA(slow) of close.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_APO(close, 12, 26, 0)`

### TA_AROON

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 阿隆指标：输出阿隆上/下，衡量距近期极值的时间。
- **Meaning (EN)**: Aroon — outputs Aroon-Up & Aroon-Down measuring time since recent extrema.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outAroonDown.0.0` (real), `outAroonUp.1.1` (real)

- **策略示例 / DSL**: `TA_TA_AROON(close, 14)`

### TA_AROONOSC

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 阿隆振荡器：阿隆上减阿隆下。
- **Meaning (EN)**: Aroon Oscillator — Aroon-Up minus Aroon-Down.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_AROONOSC(close, 14)`

### TA_BOP

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 力量平衡：以 (收-开)/(高-低) 衡量多空主导。
- **Meaning (EN)**: Balance Of Power — close vs open dominance: (close-open)/(high-low).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_BOP(close)`

### TA_CCI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 顺势指标：价格偏离其移动平均的标准差倍数。
- **Meaning (EN)**: Commodity Channel Index — deviation of price from its moving average in σ.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_CCI(close, 14)`

### TA_CMO

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 钱德动量振荡器：类 RSI，范围 -100..100。
- **Meaning (EN)**: Chande Momentum Oscillator — RSI-like, range -100..100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_CMO(close, 14)`

### TA_DX

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 动向指数：ADX 的前置量（方向运动差的绝对值）。
- **Meaning (EN)**: Directional Movement Index — precursor of ADX (abs DM differential).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_DX(close, 14)`

### TA_IMI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: TA-Lib 函数（详见 TA-Lib 官方文档）。
- **Meaning (EN)**: TA-Lib function (see TA-Lib documentation for details).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_IMI(close, 14)`

### TA_MACD

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 指数平滑异同移动平均：含 DIF、DEA(信号线)、HIST 三输出。
- **Meaning (EN)**: Moving Average Convergence/Divergence — DIF, DEA(signal), HIST.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInSignalPeriod | int | 9 | 1..100000 |

- **输出 / Outputs**: `outMACD.0.0` (real), `outMACDSignal.1.1` (real), `outMACDHist.2.2` (real)

- **策略示例 / DSL**: `TA_TA_MACD(close, 12, 26, 9)`

### TA_MACDEXT

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 可配置快/慢/信号周期的 MACD（adaq-talib 中 MAType 固定为 EMA）。
- **Meaning (EN)**: MACD with configurable fast/slow/signal periods (MAType is fixed to EMA in adaq-talib).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInFastMAType | int-list | 0 | 离散整数列表 / discrete integer list |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInSlowMAType | int-list | 0 | 离散整数列表 / discrete integer list |
| optInSignalPeriod | int | 9 | 1..100000 |
| optInSignalMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outMACD.0.0` (real), `outMACDSignal.1.1` (real), `outMACDHist.2.2` (real)

- **策略示例 / DSL**: `TA_TA_MACDEXT(close, 12, 0, 26, 0, 9, 0)`

### TA_MACDFIX

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 固定信号周期的 MACD：信号线固定为 9 周期，MAType 固定为 EMA。
- **Meaning (EN)**: MACD Fix — MACD with a fixed 9-period signal (MAType fixed to EMA in adaq-talib).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInSignalPeriod | int | 9 | 1..100000 |

- **输出 / Outputs**: `outMACD.0.0` (real), `outMACDSignal.1.1` (real), `outMACDHist.2.2` (real)

- **策略示例 / DSL**: `TA_TA_MACDFIX(close, 9)`

### TA_MFI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 资金流量指数：以成交量加权的 RSI（0–100）。
- **Meaning (EN)**: Money Flow Index — RSI weighted by volume (0–100).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MFI(close, 14)`

### TA_MINUS_DI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 负向方向指标：向下方向运动。
- **Meaning (EN)**: Minus Directional Indicator — downward directional movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MINUS_DI(close, 14)`

### TA_MINUS_DM

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 负向方向运动（原始值）。
- **Meaning (EN)**: Minus Directional Movement — raw downward movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MINUS_DM(close, 14)`

### TA_MOM

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 动量：当前收盘价减去 period 根前收盘价。
- **Meaning (EN)**: Momentum — close(t) - close(t-period).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MOM(close, 10)`

### TA_PLUS_DI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 正向方向指标：向上方向运动。
- **Meaning (EN)**: Plus Directional Indicator — upward directional movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_PLUS_DI(close, 14)`

### TA_PLUS_DM

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 正向方向运动（原始值）。
- **Meaning (EN)**: Plus Directional Movement — raw upward movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_PLUS_DM(close, 14)`

### TA_PPO

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 百分比价格振荡器：快慢 EMA 之差占慢线百分比。
- **Meaning (EN)**: Percentage Price Oscillator — (EMAfast-EMAslow)/EMAslow·100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_PPO(close, 12, 26, 0)`

### TA_ROC

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 变动率：价格相对 period 根前的百分比变化。
- **Meaning (EN)**: Rate Of Change — (close(t)-close(t-period))/close(t-period)·100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ROC(close, 10)`

### TA_ROCP

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 变动率（比例）：当前价/前期价 - 1。
- **Meaning (EN)**: Rate Of Change Percentage — (price/prev)-1.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ROCP(close, 10)`

### TA_ROCR

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 变动率比值：当前价 / period 根前价。
- **Meaning (EN)**: Rate Of Change Ratio — close(t)/close(t-period).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ROCR(close, 10)`

### TA_ROCR100

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 变动率比值×100：当前价 / 前期价 × 100。
- **Meaning (EN)**: Rate Of Change Ratio ×100 — close(t)/close(t-period)·100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ROCR100(close, 10)`

### TA_RSI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 相对强弱指数：Wilder 动量振荡器，0–100。
- **Meaning (EN)**: Relative Strength Index — Wilder momentum oscillator, 0–100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_RSI(close, 14)`

### TA_STOCH

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 随机指标：由高/低/收派生的慢速 %K 与 %D。
- **Meaning (EN)**: Stochastic — %K and %D slow stochastic from high/low/close.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastK_Period | int | 5 | 1..100000 |
| optInSlowK_Period | int | 3 | 1..100000 |
| optInSlowK_MAType | int-list | 0 | 离散整数列表 / discrete integer list |
| optInSlowD_Period | int | 3 | 1..100000 |
| optInSlowD_MAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outSlowK.0.0` (real), `outSlowD.1.1` (real)

- **策略示例 / DSL**: `TA_TA_STOCH(close, 5, 3, 0, 3, 0)`

### TA_STOCHF

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 快速随机指标：快速 %K 与 %D。
- **Meaning (EN)**: Stochastic Fast — fast %K and %D stochastic.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastK_Period | int | 5 | 1..100000 |
| optInFastD_Period | int | 3 | 1..100000 |
| optInFastD_MAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outFastK.0.0` (real), `outFastD.1.1` (real)

- **策略示例 / DSL**: `TA_TA_STOCHF(close, 5, 3, 0)`

### TA_STOCHRSI

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 随机 RSI：把随机指标公式作用于 RSI。
- **Meaning (EN)**: Stochastic RSI — applies stochastic formula to RSI itself.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |
| optInFastK_Period | int | 5 | 1..100000 |
| optInFastD_Period | int | 3 | 1..100000 |
| optInFastD_MAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outFastK.0.0` (real), `outFastD.1.1` (real)

- **策略示例 / DSL**: `TA_TA_STOCHRSI(close, 14, 5, 3, 0)`

### TA_TRIX

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 三重平滑 EMA 的变化率，用于趋势与过滤。
- **Meaning (EN)**: Trix — triple-smoothed EMA rate of change, trend/filter.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TRIX(close, 30)`

### TA_ULTOSC

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 终极振荡器：三个不同周期 ROI 的加权平均。
- **Meaning (EN)**: Ultimate Oscillator — weighted average of 3 different-period ROIs.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod1 | int | 7 | 1..100000 |
| optInTimePeriod2 | int | 14 | 1..100000 |
| optInTimePeriod3 | int | 28 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ULTOSC(close, 7, 14, 28)`

### TA_WILLR

- **分组 / Group**: Momentum Indicators / 动量指标 / Momentum Indicators / 动量指标
- **含义（中文）**: 威廉指标：随机指标的逆，范围 -100..0。
- **Meaning (EN)**: Williams' %R — inverse of Stochastic, range -100..0.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_WILLR(close, 14)`


---

## Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究

### TA_ACCBANDS

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: TA-Lib 函数（详见 TA-Lib 官方文档）。
- **Meaning (EN)**: TA-Lib function (see TA-Lib documentation for details).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 20 | 2..100000 |

- **输出 / Outputs**: `outRealUpperBand.0.0` (real), `outRealMiddleBand.1.1` (real), `outRealLowerBand.2.2` (real)

- **策略示例 / DSL**: `TA_TA_ACCBANDS(close, 20)`

### TA_BBANDS

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 布林带：中轨=收盘价 SMA，上/下轨=中轨±nbDev·标准差。用于均值回归与波动率通道。
- **Meaning (EN)**: Bollinger Bands: middle = SMA(close,period); upper/lower = middle ± nbDev·σ. Mean-reversion & volatility envelope.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 2..100000 |
| optInNbDevUp | real | 2 | — |
| optInNbDevDn | real | 2 | — |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outRealUpperBand.0.0` (real), `outRealMiddleBand.1.1` (real), `outRealLowerBand.2.2` (real)

- **策略示例 / DSL**: `TA_TA_BBANDS(close, 5, 2, 2, 0)`

### TA_DEMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 双指数移动平均：对 EMA 做二次平滑，进一步降低滞后。
- **Meaning (EN)**: Double Exponential Moving Average — smoother/faster EMA variant reducing lag.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_DEMA(close, 30)`

### TA_EMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 指数移动平均：近期价格权重更高。
- **Meaning (EN)**: Exponential Moving Average — weights recent prices more heavily.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_EMA(close, 30)`

### TA_HT_TRENDLINE

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 希尔伯特变换瞬时趋势线：基于数字信号处理的自适应趋势线。
- **Meaning (EN)**: Hilbert Transform Instantaneous Trendline — adaptive trend line via DSP.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_HT_TRENDLINE(close)`

### TA_KAMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 卡夫曼自适应均线：依据效率比在噪声中自适应调整快慢。
- **Meaning (EN)**: Kaufman Adaptive Moving Average — adapts speed to noise (efficiency ratio).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_KAMA(close, 30)`

### TA_MA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 移动平均：可通过 MAType 选择多种均线算法。
- **Meaning (EN)**: Moving Average with selectable MA type (SMA/EMA/WMA/DEMA/TEMA/TRIMA/KAMA/MAMA/T3).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MA(close, 30, 0)`

### TA_MAMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: MESA 自适应均线：源自希尔伯特变换周期的自适应均线。
- **Meaning (EN)**: MESA Adaptive Moving Average — adaptive MA from Hilbert transform cycle.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastLimit | real | 0.5 | 0.01..0.99 |
| optInSlowLimit | real | 0.05 | 0.01..0.99 |

- **输出 / Outputs**: `outMAMA.0.0` (real), `outFAMA.1.1` (real)

- **策略示例 / DSL**: `TA_TA_MAMA(close, 0.5, 0.05)`

### TA_MAVP

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 变周期移动平均：周期随外部周期数组变化。
- **Meaning (EN)**: Moving Average with Variable Period — period varies by an external cycle array.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInMinPeriod | int | 2 | 1..100000 |
| optInMaxPeriod | int | 30 | 1..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MAVP(close, 2, 30, 0)`

### TA_MIDPOINT

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 中值：窗口内最高与最低的中间值。
- **Meaning (EN)**: MidPoint over period — average of highest and lowest in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MIDPOINT(close, 14)`

### TA_MIDPRICE

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 中价：窗口内最高价与最低价的平均值。
- **Meaning (EN)**: MidPrice over period — (highest high + lowest low)/2 over the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MIDPRICE(close, 14)`

### TA_SAR

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 抛物线 SAR：趋势跟踪的止损反转指标。
- **Meaning (EN)**: Parabolic SAR — trailing stop-and-reverse indicator for trend following.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInAcceleration | real | 0.02 | — |
| optInMaximum | real | 0.2 | — |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SAR(close, 0.02, 0.2)`

### TA_SAREXT

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 扩展抛物线 SAR：可配置加速极值参数的 SAR。
- **Meaning (EN)**: Parabolic SAR Extended — SAR with configurable acceleration extremes.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInStartValue | real | 0 | — |
| optInOffsetOnReverse | real | 0 | — |
| optInAccelerationInitLong | real | 0.02 | — |
| optInAccelerationLong | real | 0.02 | — |
| optInAccelerationMaxLong | real | 0.2 | — |
| optInAccelerationInitShort | real | 0.02 | — |
| optInAccelerationShort | real | 0.02 | — |
| optInAccelerationMaxShort | real | 0.2 | — |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SAREXT(close, 0, 0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2)`

### TA_SMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 简单移动平均：窗口内收盘价的算术平均。
- **Meaning (EN)**: Simple Moving Average — arithmetic mean of close over period.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_SMA(close, 30)`

### TA_T3

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: T3 均线：带量价因子的三重平滑 EMA。
- **Meaning (EN)**: T3 Moving Average — triple-smoothed EMA with volume factor.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 1..100000 |
| optInVFactor | real | 0.7 | 0..1 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_T3(close, 5, 0.7)`

### TA_TEMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 三重指数移动平均：相较 DEMA 进一步降低滞后。
- **Meaning (EN)**: Triple Exponential Moving Average — further lag reduction vs DEMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TEMA(close, 30)`

### TA_TRIMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 三角移动平均：对称双重平滑的 SMA。
- **Meaning (EN)**: Triangular Moving Average — double-smoothed (symmetrical) SMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TRIMA(close, 30)`

### TA_WMA

- **分组 / Group**: Overlap Studies / 重叠研究 / Overlap Studies / 重叠研究
- **含义（中文）**: 加权移动平均：线性加权，近期权重更大。
- **Meaning (EN)**: Weighted Moving Average — linearly weighted, recent prices heavier.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_WMA(close, 30)`


---

## Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别

### TA_CDL2CROWS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 两只乌鸦：看跌反转。
- **Meaning (EN)**: Two Crows bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL2CROWS(close)`

### TA_CDL3BLACKCROWS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 三只乌鸦：看跌反转。
- **Meaning (EN)**: Three Black Crows bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL3BLACKCROWS(close)`

### TA_CDL3INSIDE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 内困三日：反转形态。
- **Meaning (EN)**: Three Inside Up/Down reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL3INSIDE(close)`

### TA_CDL3LINESTRIKE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 三线打击。
- **Meaning (EN)**: Three-Line Strike (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL3LINESTRIKE(close)`

### TA_CDL3OUTSIDE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 外困三日：反转形态。
- **Meaning (EN)**: Three Outside Up/Down reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL3OUTSIDE(close)`

### TA_CDL3STARSINSOUTH

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 南方三星：看涨。
- **Meaning (EN)**: Three Stars In The South bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL3STARSINSOUTH(close)`

### TA_CDL3WHITESOLDIERS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 白色三兵：看涨。
- **Meaning (EN)**: Three Advancing White Soldiers bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDL3WHITESOLDIERS(close)`

### TA_CDLABANDONEDBABY

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 弃婴：反转形态。
- **Meaning (EN)**: Abandoned Baby reversal (gap doji).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLABANDONEDBABY(close, 0.3)`

### TA_CDLADVANCEBLOCK

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 推进块：看跌。
- **Meaning (EN)**: Advance Block bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLADVANCEBLOCK(close)`

### TA_CDLBELTHOLD

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 腰带线。
- **Meaning (EN)**: Belt-hold line (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLBELTHOLD(close)`

### TA_CDLBREAKAWAY

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 脱离形态：反转。
- **Meaning (EN)**: Breakaway reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLBREAKAWAY(close)`

### TA_CDLCLOSINGMARUBOZU

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 收盘光头线。
- **Meaning (EN)**: Closing Marubozu (no upper shadow).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLCLOSINGMARUBOZU(close)`

### TA_CDLCONCEALBABYSWALL

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 藏婴吞没：看涨。
- **Meaning (EN)**: Concealing Baby Swallow bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLCONCEALBABYSWALL(close)`

### TA_CDLCOUNTERATTACK

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 反击线：反转。
- **Meaning (EN)**: Counterattack lines reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLCOUNTERATTACK(close)`

### TA_CDLDARKCLOUDCOVER

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 乌云盖顶：看跌反转。
- **Meaning (EN)**: Dark Cloud Cover bearish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.5 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLDARKCLOUDCOVER(close, 0.5)`

### TA_CDLDOJI

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 十字星：犹豫。
- **Meaning (EN)**: Doji indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLDOJI(close)`

### TA_CDLDOJISTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 十字星：反转。
- **Meaning (EN)**: Doji Star reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLDOJISTAR(close)`

### TA_CDLDRAGONFLYDOJI

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 蜻蜓十字：看涨反转。
- **Meaning (EN)**: Dragonfly Doji bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLDRAGONFLYDOJI(close)`

### TA_CDLENGULFING

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 吞没形态。
- **Meaning (EN)**: Engulfing Pattern (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLENGULFING(close)`

### TA_CDLEVENINGDOJISTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 黄昏十字星：看跌反转。
- **Meaning (EN)**: Evening Doji Star bearish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLEVENINGDOJISTAR(close, 0.3)`

### TA_CDLEVENINGSTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 黄昏之星：看跌反转。
- **Meaning (EN)**: Evening Star bearish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLEVENINGSTAR(close, 0.3)`

### TA_CDLGAPSIDESIDEWHITE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 并列阳线缺口。
- **Meaning (EN)**: Up/Down-gap Side-by-Side White lines.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLGAPSIDESIDEWHITE(close)`

### TA_CDLGRAVESTONEDOJI

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 墓碑十字：看跌反转。
- **Meaning (EN)**: Gravestone Doji bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLGRAVESTONEDOJI(close)`

### TA_CDLHAMMER

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 锤头：看涨反转。
- **Meaning (EN)**: Hammer bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHAMMER(close)`

### TA_CDLHANGINGMAN

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 上吊线：看跌反转。
- **Meaning (EN)**: Hanging Man bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHANGINGMAN(close)`

### TA_CDLHARAMI

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 孕线：反转。
- **Meaning (EN)**: Harami reversal (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHARAMI(close)`

### TA_CDLHARAMICROSS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 十字孕线：反转。
- **Meaning (EN)**: Harami Cross reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHARAMICROSS(close)`

### TA_CDLHIGHWAVE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 高价波动：犹豫。
- **Meaning (EN)**: High-Wave indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHIGHWAVE(close)`

### TA_CDLHIKKAKE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 陷阱形态。
- **Meaning (EN)**: Hikkake Pattern (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHIKKAKE(close)`

### TA_CDLHIKKAKEMOD

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 改良陷阱形态。
- **Meaning (EN)**: Modified Hikkake Pattern.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHIKKAKEMOD(close)`

### TA_CDLHOMINGPIGEON

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 家鸽：看涨。
- **Meaning (EN)**: Homing Pigeon bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLHOMINGPIGEON(close)`

### TA_CDLIDENTICAL3CROWS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 等同三乌鸦：看跌。
- **Meaning (EN)**: Identical Three Crows bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLIDENTICAL3CROWS(close)`

### TA_CDLINNECK

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 颈内线：反转。
- **Meaning (EN)**: In-Neck reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLINNECK(close)`

### TA_CDLINVERTEDHAMMER

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 倒锤头：看涨反转。
- **Meaning (EN)**: Inverted Hammer bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLINVERTEDHAMMER(close)`

### TA_CDLKICKING

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 反冲形态。
- **Meaning (EN)**: Kicking (bullish/bearish) gap lines.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLKICKING(close)`

### TA_CDLKICKINGBYLENGTH

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 按长度反冲。
- **Meaning (EN)**: Kicking by Length.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLKICKINGBYLENGTH(close)`

### TA_CDLLADDERBOTTOM

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 梯底：看涨。
- **Meaning (EN)**: Ladder Bottom bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLLADDERBOTTOM(close)`

### TA_CDLLONGLEGGEDDOJI

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 长腿十字：犹豫。
- **Meaning (EN)**: Long-Legged Doji indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLLONGLEGGEDDOJI(close)`

### TA_CDLLONGLINE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 长实体线。
- **Meaning (EN)**: Long Line candle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLLONGLINE(close)`

### TA_CDLMARUBOZU

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 光头光脚线。
- **Meaning (EN)**: Marubozu (no shadows).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLMARUBOZU(close)`

### TA_CDLMATCHINGLOW

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 呼应低：看涨。
- **Meaning (EN)**: Matching Low bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLMATCHINGLOW(close)`

### TA_CDLMATHOLD

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 铺垫：看涨持续。
- **Meaning (EN)**: Mat Hold continuation (bullish).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.5 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLMATHOLD(close, 0.5)`

### TA_CDLMORNINGDOJISTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 早晨十字星：看涨反转。
- **Meaning (EN)**: Morning Doji Star bullish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLMORNINGDOJISTAR(close, 0.3)`

### TA_CDLMORNINGSTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 早晨之星：看涨反转。
- **Meaning (EN)**: Morning Star bullish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | — |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLMORNINGSTAR(close, 0.3)`

### TA_CDLONNECK

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 颈上线：反转。
- **Meaning (EN)**: On-Neck reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLONNECK(close)`

### TA_CDLPIERCING

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 刺透形态：看涨反转。
- **Meaning (EN)**: Piercing Pattern bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLPIERCING(close)`

### TA_CDLRICKSHAWMAN

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: TA-Lib 函数（详见 TA-Lib 官方文档）。
- **Meaning (EN)**: TA-Lib function (see TA-Lib documentation for details).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLRICKSHAWMAN(close)`

### TA_CDLRISEFALL3METHODS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 三法形态：持续。
- **Meaning (EN)**: Rising/Falling Three Methods continuation.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLRISEFALL3METHODS(close)`

### TA_CDLSEPARATINGLINES

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 分离线：持续。
- **Meaning (EN)**: Separating Lines continuation.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLSEPARATINGLINES(close)`

### TA_CDLSHOOTINGSTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 射击之星：看跌反转。
- **Meaning (EN)**: Shooting Star bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLSHOOTINGSTAR(close)`

### TA_CDLSHORTLINE

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 短实体线。
- **Meaning (EN)**: Short Line candle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLSHORTLINE(close)`

### TA_CDLSPINNINGTOP

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 纺锤线：犹豫。
- **Meaning (EN)**: Spinning Top indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLSPINNINGTOP(close)`

### TA_CDLSTALLEDPATTERN

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 停滞形态：看跌。
- **Meaning (EN)**: Stalled Pattern bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLSTALLEDPATTERN(close)`

### TA_CDLSTICKSANDWICH

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**:  Stick 夹心：看涨。
- **Meaning (EN)**: Stick Sandwich bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLSTICKSANDWICH(close)`

### TA_CDLTAKURI

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 垂柳线：看涨。
- **Meaning (EN)**: Takuri (dragonfly doji variant) bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLTAKURI(close)`

### TA_CDLTASUKIGAP

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 跳空并列：持续。
- **Meaning (EN)**: Tasuki Gap continuation.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLTASUKIGAP(close)`

### TA_CDLTHRUSTING

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 插入线：看跌持续。
- **Meaning (EN)**: Thrusting continuation (bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLTHRUSTING(close)`

### TA_CDLTRISTAR

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 三星：反转。
- **Meaning (EN)**: Tristar (three doji) reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLTRISTAR(close)`

### TA_CDLUNIQUE3RIVER

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 独特三河底：看涨。
- **Meaning (EN)**: Unique 3 River bottom bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLUNIQUE3RIVER(close)`

### TA_CDLUPSIDEGAP2CROWS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 上跳双鸦：看跌。
- **Meaning (EN)**: Upside Gap Two Crows bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLUPSIDEGAP2CROWS(close)`

### TA_CDLXSIDEGAP3METHODS

- **分组 / Group**: Pattern Recognition / 形态识别 / Pattern Recognition / 形态识别
- **含义（中文）**: 跳空三法。
- **Meaning (EN)**: Upside/Downside Gap Three Methods.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_TA_CDLXSIDEGAP3METHODS(close)`


---

## Price Transform / 价格变换 / Price Transform / 价格变换

### TA_AVGDEV

- **分组 / Group**: Price Transform / 价格变换 / Price Transform / 价格变换
- **含义（中文）**: TA-Lib 函数（详见 TA-Lib 官方文档）。
- **Meaning (EN)**: TA-Lib function (see TA-Lib documentation for details).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_AVGDEV(close, 14)`

### TA_AVGPRICE

- **分组 / Group**: Price Transform / 价格变换 / Price Transform / 价格变换
- **含义（中文）**: 平均价格：(高+低+开+收)/4。
- **Meaning (EN)**: Average Price — (high+low+open+close)/4.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_AVGPRICE(close)`

### TA_MEDPRICE

- **分组 / Group**: Price Transform / 价格变换 / Price Transform / 价格变换
- **含义（中文）**: 中价：(高+低)/2。
- **Meaning (EN)**: Median Price — (high+low)/2.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_MEDPRICE(close)`

### TA_TYPPRICE

- **分组 / Group**: Price Transform / 价格变换 / Price Transform / 价格变换
- **含义（中文）**: 典型价格：(高+低+收)/3。
- **Meaning (EN)**: Typical Price — (high+low+close)/3.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TYPPRICE(close)`

### TA_WCLPRICE

- **分组 / Group**: Price Transform / 价格变换 / Price Transform / 价格变换
- **含义（中文）**: 加权收盘价：(高+低+收·2)/4。
- **Meaning (EN)**: Weighted Close Price — (high+low+close·2)/4.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_WCLPRICE(close)`


---

## Statistic Functions / 统计函数 / Statistic Functions / 统计函数

### TA_BETA

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 贝塔：资产相对市场的线性回归斜率（本封装两者均取收盘价）。
- **Meaning (EN)**: Beta — slope of linear regression of asset vs market (both = close here).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_BETA(close, 5)`

### TA_CORREL

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 皮尔逊相关系数：两序列在窗口内的相关性。
- **Meaning (EN)**: Pearson Correlation — correlation of two series over period.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_CORREL(close, 30)`

### TA_LINEARREG

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 线性回归：最小二乘拟合线末端值。
- **Meaning (EN)**: Linear Regression — endpoint value of least-squares fit line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_LINEARREG(close, 14)`

### TA_LINEARREG_ANGLE

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 线性回归角度：拟合线斜率（度）。
- **Meaning (EN)**: Linear Regression Angle — slope angle (degrees) of fitted line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_LINEARREG_ANGLE(close, 14)`

### TA_LINEARREG_INTERCEPT

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 线性回归截距。
- **Meaning (EN)**: Linear Regression Intercept — y-intercept of fitted line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_LINEARREG_INTERCEPT(close, 14)`

### TA_LINEARREG_SLOPE

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 线性回归斜率：每根变化量。
- **Meaning (EN)**: Linear Regression Slope — per-bar slope of fitted line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_LINEARREG_SLOPE(close, 14)`

### TA_STDDEV

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 标准差：收盘价相对 SMA 的离散度（×nbDev）。
- **Meaning (EN)**: Standard Deviation — σ of close around its SMA (×nbDev).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 2..100000 |
| optInNbDev | real | 1 | — |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_STDDEV(close, 5, 1)`

### TA_TSF

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 时间序列预测：回归线向前投影一根。
- **Meaning (EN)**: Time Series Forecast — regression line projected one bar ahead.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TSF(close, 14)`

### TA_VAR

- **分组 / Group**: Statistic Functions / 统计函数 / Statistic Functions / 统计函数
- **含义（中文）**: 方差：收盘价相对 SMA 的离散平方。
- **Meaning (EN)**: Variance — σ² of close around its SMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 1..100000 |
| optInNbDev | real | 1 | — |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_VAR(close, 5, 1)`


---

## Volatility Indicators / 波动率指标 / Volatility Indicators / 波动率指标

### TA_ATR

- **分组 / Group**: Volatility Indicators / 波动率指标 / Volatility Indicators / 波动率指标
- **含义（中文）**: 平均真实波幅：真实波幅的 Wilder 均值，衡量波动率。
- **Meaning (EN)**: Average True Range — Wilder average of True Range, measures volatility.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ATR(close, 14)`

### TA_NATR

- **分组 / Group**: Volatility Indicators / 波动率指标 / Volatility Indicators / 波动率指标
- **含义（中文）**: 归一化 ATR：ATR 除以收盘价，无量纲。
- **Meaning (EN)**: Normalized ATR — ATR divided by close, scale-independent.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_NATR(close, 14)`

### TA_TRANGE

- **分组 / Group**: Volatility Indicators / 波动率指标 / Volatility Indicators / 波动率指标
- **含义（中文）**: 真实波幅：三者最大值。
- **Meaning (EN)**: True Range — high-low, |high-prevClose|, |low-prevClose| (max).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_TRANGE(close)`


---

## Volume Indicators / 成交量指标 / Volume Indicators / 成交量指标

### TA_AD

- **分组 / Group**: Volume Indicators / 成交量指标 / Volume Indicators / 成交量指标
- **含义（中文）**: 累积/派发线：以收盘价在高低区间的位置加权的累计成交量。
- **Meaning (EN)**: Chaikin A/D Line — cumulative volume weighted by close location in range.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_AD(close)`

### TA_ADOSC

- **分组 / Group**: Volume Indicators / 成交量指标 / Volume Indicators / 成交量指标
- **含义（中文）**: A/D 振荡器：快慢 A/D 均线之差。
- **Meaning (EN)**: Chaikin A/D Oscillator — fast A/D MA minus slow A/D MA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 3 | 2..100000 |
| optInSlowPeriod | int | 10 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_ADOSC(close, 3, 10)`

### TA_OBV

- **分组 / Group**: Volume Indicators / 成交量指标 / Volume Indicators / 成交量指标
- **含义（中文）**: 能量潮：按价格方向带符号累计成交量。
- **Meaning (EN)**: On Balance Volume — cumulative volume signed by price direction.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TA_OBV(close)`

