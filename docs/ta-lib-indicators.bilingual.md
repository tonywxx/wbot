# TA-Lib 指标参考手册 / TA-Lib Indicators Reference

> 本程序通过 TA-Lib 抽象 API 对接其**全部**函数（共 161 个），可在策略 DSL 中以 `TA_<FUNC>(...)` 形式直接引用。
> This program exposes **all** TA-Lib functions (161 total) via its abstract API; reference them in the strategy DSL as `TA_<FUNC>(...)`.

## 通用约定 / Conventions
- **参数写法 / Parameters**: `TA_RSI(close, 14)` —— 第一个参数为价格来源（TA-Lib 按函数自身价格掩码取用，多数用收盘价），其余为可选参数；缺省时取 TA-Lib 默认值。
- **多输出选择 / Multi-output**: 用 `.0 / .1 / .2` 或输出名选择，如 `TA_MACD(close,12,26,9).hist`、`TA_BBANDS(close,20,2).upper`。默认取首个输出。
- **前导值 / Lookback**: 序列前若干根不足计算长度，输出为 `NaN`（不参与信号比较）。

MAType 取值（整数，用于 MA / MACD / BBANDS 等的可选均线类型）：
- 0 = SMA（简单）  1 = EMA（指数）  2 = WMA（加权）  3 = DEMA（双指数）
- 4 = TEMA（三重指数）  5 = TRIMA（三角）  6 = KAMA（自适应）  7 = MAMA（MESA）  8 = T3


---

## Cycle Indicators / 周期指标

### HT_DCPERIOD

- **分组 / Group**: Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换主导周期：以根数表示的主导周期。
- **Meaning (EN)**: Hilbert Transform Dominant Cycle Period — dominant cycle in bars.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_HT_DCPERIOD(close)`

### HT_DCPHASE

- **分组 / Group**: Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换主导周期相位。
- **Meaning (EN)**: Hilbert Transform Dominant Cycle Phase — phase of dominant cycle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_HT_DCPHASE(close)`

### HT_PHASOR

- **分组 / Group**: Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换相量：同相与正交分量。
- **Meaning (EN)**: Hilbert Transform Phasor — in-phase & quadrature components.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInPhase.0` (real), `outQuadrature.1` (real)

- **策略示例 / DSL**: `TA_HT_PHASOR(close)`

### HT_SINE

- **分组 / Group**: Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换正弦：主导周期的正弦与超前正弦。
- **Meaning (EN)**: Hilbert Transform Sine — sine & lead-sine of dominant cycle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outSine.0` (real), `outLeadSine.1` (real)

- **策略示例 / DSL**: `TA_HT_SINE(close)`

### HT_TRENDMODE

- **分组 / Group**: Cycle Indicators / 周期指标
- **含义（中文）**: 希尔伯特变换趋势/周期模式：1 为趋势，0 为周期。
- **Meaning (EN)**: Hilbert Transform Trend vs Cycle Mode — 1=trend, 0=cycle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_HT_TRENDMODE(close)`


---

## Math Operators / 数学运算

### ADD

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 相加：两序列逐元素相加（本封装两者均取收盘价）。
- **Meaning (EN)**: Add — inReal + second price series (both = close here).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ADD(close)`

### DIV

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 相除（逐元素）。
- **Meaning (EN)**: Divide — inReal / second price series.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_DIV(close)`

### MAX

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内最大值。
- **Meaning (EN)**: Max over period — highest value in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MAX(close, 30)`

### MAXINDEX

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内最大值所在位置（整数输出）。
- **Meaning (EN)**: Max Index — bar index of the max within the window (integer).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_MAXINDEX(close, 30)`

### MIN

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内最小值。
- **Meaning (EN)**: Min over period — lowest value in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MIN(close, 30)`

### MININDEX

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内最小值所在位置（整数输出）。
- **Meaning (EN)**: Min Index — bar index of the min within the window (integer).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_MININDEX(close, 30)`

### MINMAX

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内最小与最大（两输出）。
- **Meaning (EN)**: Min & Max over period — two outputs: min then max.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outMin.0` (real), `outMax.1` (real)

- **策略示例 / DSL**: `TA_MINMAX(close, 30)`

### MINMAXINDEX

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内最小/最大值位置（两整数输出）。
- **Meaning (EN)**: Min & Max Index — indices of min and max (two integer outputs).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outMinIdx.0` (integer), `outMaxIdx.1` (integer)

- **策略示例 / DSL**: `TA_MINMAXINDEX(close, 30)`

### MULT

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 相乘（逐元素）。
- **Meaning (EN)**: Multiply — inReal × second price series.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MULT(close)`

### SUB

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 相减（逐元素）。
- **Meaning (EN)**: Subtract — inReal − second price series.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SUB(close)`

### SUM

- **分组 / Group**: Math Operators / 数学运算
- **含义（中文）**: 窗口内求和。
- **Meaning (EN)**: Sum over period — total of values in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SUM(close, 30)`


---

## Math Transform / 数学变换

### ACOS

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 反余弦（逐元素）。
- **Meaning (EN)**: Arc Cosine — acos(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ACOS(close)`

### ASIN

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 反正弦（逐元素）。
- **Meaning (EN)**: Arc Sine — asin(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ASIN(close)`

### ATAN

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 反正切（逐元素）。
- **Meaning (EN)**: Arc Tangent — atan(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ATAN(close)`

### CEIL

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 向上取整。
- **Meaning (EN)**: Ceiling — smallest integer ≥ x.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_CEIL(close)`

### COS

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 余弦（逐元素）。
- **Meaning (EN)**: Cosine — cos(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_COS(close)`

### COSH

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 双曲余弦（逐元素）。
- **Meaning (EN)**: Hyperbolic Cosine — cosh(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_COSH(close)`

### EXP

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 指数（逐元素）。
- **Meaning (EN)**: Exponential — e^x element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_EXP(close)`

### FLOOR

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 向下取整。
- **Meaning (EN)**: Floor — largest integer ≤ x.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_FLOOR(close)`

### LN

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 自然对数（逐元素）。
- **Meaning (EN)**: Natural Log — ln(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_LN(close)`

### LOG10

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 常用对数（逐元素）。
- **Meaning (EN)**: Base-10 Log — log10(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_LOG10(close)`

### SIN

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 正弦（逐元素）。
- **Meaning (EN)**: Sine — sin(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SIN(close)`

### SINH

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 双曲正弦（逐元素）。
- **Meaning (EN)**: Hyperbolic Sine — sinh(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SINH(close)`

### SQRT

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 平方根（逐元素）。
- **Meaning (EN)**: Square Root — √x element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SQRT(close)`

### TAN

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 正切（逐元素）。
- **Meaning (EN)**: Tangent — tan(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TAN(close)`

### TANH

- **分组 / Group**: Math Transform / 数学变换
- **含义（中文）**: 双曲正切（逐元素）。
- **Meaning (EN)**: Hyperbolic Tangent — tanh(x) element-wise.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TANH(close)`


---

## Momentum Indicators / 动量指标

### ADX

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 平均趋向指数：衡量趋势强度（非方向），0–100。
- **Meaning (EN)**: Average Directional Movement Index — trend strength (not direction), 0–100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ADX(close, 14)`

### ADXR

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: ADX 评级：将 ADX 与 `period` 根前的自身值归一化比较。
- **Meaning (EN)**: ADX Rating — ADX normalized against its value `period` bars ago.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ADXR(close, 14)`

### APO

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 绝对价格振荡器：收盘价的快慢 EMA 之差。
- **Meaning (EN)**: Absolute Price Oscillator — EMA(fast)-EMA(slow) of close.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_APO(close, 12, 26, 0)`

### AROON

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 阿隆指标：输出阿隆上/下，衡量距近期极值的时间。
- **Meaning (EN)**: Aroon — outputs Aroon-Up & Aroon-Down measuring time since recent extrema.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outAroonDown.0` (real), `outAroonUp.1` (real)

- **策略示例 / DSL**: `TA_AROON(close, 14)`

### AROONOSC

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 阿隆振荡器：阿隆上减阿隆下。
- **Meaning (EN)**: Aroon Oscillator — Aroon-Up minus Aroon-Down.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_AROONOSC(close, 14)`

### BOP

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 力量平衡：以 (收-开)/(高-低) 衡量多空主导。
- **Meaning (EN)**: Balance Of Power — close vs open dominance: (close-open)/(high-low).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_BOP(close)`

### CCI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 顺势指标：价格偏离其移动平均的标准差倍数。
- **Meaning (EN)**: Commodity Channel Index — deviation of price from its moving average in σ.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_CCI(close, 14)`

### CMO

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 钱德动量振荡器：类 RSI，范围 -100..100。
- **Meaning (EN)**: Chande Momentum Oscillator — RSI-like, range -100..100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_CMO(close, 14)`

### DX

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 动向指数：ADX 的前置量（方向运动差的绝对值）。
- **Meaning (EN)**: Directional Movement Index — precursor of ADX (abs DM differential).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_DX(close, 14)`

### IMI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 衡量价格变动速度或幅度的振荡指标。
- **Meaning (EN)**: Oscillator measuring the speed or magnitude of price movements.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_IMI(close, 14)`

### MACD

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 指数平滑异同移动平均：含 DIF、DEA(信号线)、HIST 三输出。
- **Meaning (EN)**: Moving Average Convergence/Divergence — DIF, DEA(signal), HIST.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInSignalPeriod | int | 9 | 1..100000 |

- **输出 / Outputs**: `outMACD.0` (real), `outMACDSignal.1` (real), `outMACDHist.2` (real)

- **策略示例 / DSL**: `TA_MACD(close, 12, 26, 9)`

### MACDEXT

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 可配置均线类型的 MACD（快/慢/信号线各自可选 MA 算法）。
- **Meaning (EN)**: MACD with configurable MAType for each of fast/slow/signal lines.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInFastMAType | int-list | 0 | 离散整数列表 / discrete integer list |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInSlowMAType | int-list | 0 | 离散整数列表 / discrete integer list |
| optInSignalPeriod | int | 9 | 1..100000 |
| optInSignalMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outMACD.0` (real), `outMACDSignal.1` (real), `outMACDHist.2` (real)

- **策略示例 / DSL**: `TA_MACDEXT(close, 12, 0, 26, 0, 9, 0)`

### MACDFIX

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 固定信号线的 MACD：信号线固定为 9 周期 SMA。
- **Meaning (EN)**: MACD Fix — MACD using a fixed 9-period signal SMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInSignalPeriod | int | 9 | 1..100000 |

- **输出 / Outputs**: `outMACD.0` (real), `outMACDSignal.1` (real), `outMACDHist.2` (real)

- **策略示例 / DSL**: `TA_MACDFIX(close, 9)`

### MFI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 资金流量指数：以成交量加权的 RSI（0–100）。
- **Meaning (EN)**: Money Flow Index — RSI weighted by volume (0–100).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MFI(close, 14)`

### MINUS_DI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 负向方向指标：向下方向运动。
- **Meaning (EN)**: Minus Directional Indicator — downward directional movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MINUS_DI(close, 14)`

### MINUS_DM

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 负向方向运动（原始值）。
- **Meaning (EN)**: Minus Directional Movement — raw downward movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MINUS_DM(close, 14)`

### MOM

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 动量：当前收盘价减去 period 根前收盘价。
- **Meaning (EN)**: Momentum — close(t) - close(t-period).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MOM(close, 10)`

### PLUS_DI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 正向方向指标：向上方向运动。
- **Meaning (EN)**: Plus Directional Indicator — upward directional movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_PLUS_DI(close, 14)`

### PLUS_DM

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 正向方向运动（原始值）。
- **Meaning (EN)**: Plus Directional Movement — raw upward movement.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_PLUS_DM(close, 14)`

### PPO

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 百分比价格振荡器：快慢 EMA 之差占慢线百分比。
- **Meaning (EN)**: Percentage Price Oscillator — (EMAfast-EMAslow)/EMAslow·100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 12 | 2..100000 |
| optInSlowPeriod | int | 26 | 2..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_PPO(close, 12, 26, 0)`

### ROC

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 变动率：价格相对 period 根前的百分比变化。
- **Meaning (EN)**: Rate Of Change — (close(t)-close(t-period))/close(t-period)·100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ROC(close, 10)`

### ROCP

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 变动率（比例）：当前价/前期价 - 1。
- **Meaning (EN)**: Rate Of Change Percentage — (price/prev)-1.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ROCP(close, 10)`

### ROCR

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 变动率比值：当前价 / period 根前价。
- **Meaning (EN)**: Rate Of Change Ratio — close(t)/close(t-period).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ROCR(close, 10)`

### ROCR100

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 变动率比值×100：当前价 / 前期价 × 100。
- **Meaning (EN)**: Rate Of Change Ratio ×100 — close(t)/close(t-period)·100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 10 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ROCR100(close, 10)`

### RSI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 相对强弱指数：Wilder 动量振荡器，0–100。
- **Meaning (EN)**: Relative Strength Index — Wilder momentum oscillator, 0–100.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_RSI(close, 14)`

### STOCH

- **分组 / Group**: Momentum Indicators / 动量指标
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

- **输出 / Outputs**: `outSlowK.0` (real), `outSlowD.1` (real)

- **策略示例 / DSL**: `TA_STOCH(close, 5, 3, 0, 3, 0)`

### STOCHF

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 快速随机指标：快速 %K 与 %D。
- **Meaning (EN)**: Stochastic Fast — fast %K and %D stochastic.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastK_Period | int | 5 | 1..100000 |
| optInFastD_Period | int | 3 | 1..100000 |
| optInFastD_MAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outFastK.0` (real), `outFastD.1` (real)

- **策略示例 / DSL**: `TA_STOCHF(close, 5, 3, 0)`

### STOCHRSI

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 随机 RSI：把随机指标公式作用于 RSI。
- **Meaning (EN)**: Stochastic RSI — applies stochastic formula to RSI itself.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |
| optInFastK_Period | int | 5 | 1..100000 |
| optInFastD_Period | int | 3 | 1..100000 |
| optInFastD_MAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outFastK.0` (real), `outFastD.1` (real)

- **策略示例 / DSL**: `TA_STOCHRSI(close, 14, 5, 3, 0)`

### TRIX

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 三重平滑 EMA 的变化率，用于趋势与过滤。
- **Meaning (EN)**: Trix — triple-smoothed EMA rate of change, trend/filter.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TRIX(close, 30)`

### ULTOSC

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 终极振荡器：三个不同周期 ROI 的加权平均。
- **Meaning (EN)**: Ultimate Oscillator — weighted average of 3 different-period ROIs.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod1 | int | 7 | 1..100000 |
| optInTimePeriod2 | int | 14 | 1..100000 |
| optInTimePeriod3 | int | 28 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ULTOSC(close, 7, 14, 28)`

### WILLR

- **分组 / Group**: Momentum Indicators / 动量指标
- **含义（中文）**: 威廉指标：随机指标的逆，范围 -100..0。
- **Meaning (EN)**: Williams' %R — inverse of Stochastic, range -100..0.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_WILLR(close, 14)`


---

## Overlap Studies / 重叠研究

### ACCBANDS

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 在价格图上叠加的均线/重叠类指标。
- **Meaning (EN)**: Overlay/ moving-average study plotted on the price chart.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 20 | 2..100000 |

- **输出 / Outputs**: `outRealUpperBand.0` (real), `outRealMiddleBand.1` (real), `outRealLowerBand.2` (real)

- **策略示例 / DSL**: `TA_ACCBANDS(close, 20)`

### BBANDS

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 布林带：中轨=收盘价 SMA，上/下轨=中轨±nbDev·标准差。用于均值回归与波动率通道。
- **Meaning (EN)**: Bollinger Bands: middle = SMA(close,period); upper/lower = middle ± nbDev·σ. Mean-reversion & volatility envelope.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 2..100000 |
| optInNbDevUp | real | 2 | 无限制..无限制 |
| optInNbDevDn | real | 2 | 无限制..无限制 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outRealUpperBand.0` (real), `outRealMiddleBand.1` (real), `outRealLowerBand.2` (real)

- **策略示例 / DSL**: `TA_BBANDS(close, 5, 2, 2, 0)`

### DEMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 双指数移动平均：对 EMA 做二次平滑，进一步降低滞后。
- **Meaning (EN)**: Double Exponential Moving Average — smoother/faster EMA variant reducing lag.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_DEMA(close, 30)`

### EMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 指数移动平均：近期价格权重更高。
- **Meaning (EN)**: Exponential Moving Average — weights recent prices more heavily.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_EMA(close, 30)`

### HT_TRENDLINE

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 希尔伯特变换瞬时趋势线：基于数字信号处理的自适应趋势线。
- **Meaning (EN)**: Hilbert Transform Instantaneous Trendline — adaptive trend line via DSP.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_HT_TRENDLINE(close)`

### KAMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 卡夫曼自适应均线：依据效率比在噪声中自适应调整快慢。
- **Meaning (EN)**: Kaufman Adaptive Moving Average — adapts speed to noise (efficiency ratio).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_KAMA(close, 30)`

### MA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 移动平均：可通过 MAType 选择多种均线算法。
- **Meaning (EN)**: Moving Average with selectable MA type (SMA/EMA/WMA/DEMA/TEMA/TRIMA/KAMA/MAMA/T3).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MA(close, 30, 0)`

### MAMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: MESA 自适应均线：源自希尔伯特变换周期的自适应均线。
- **Meaning (EN)**: MESA Adaptive Moving Average — adaptive MA from Hilbert transform cycle.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastLimit | real | 0.5 | 0.01..0.99 |
| optInSlowLimit | real | 0.05 | 0.01..0.99 |

- **输出 / Outputs**: `outMAMA.0` (real), `outFAMA.1` (real)

- **策略示例 / DSL**: `TA_MAMA(close, 0.5, 0.05)`

### MAVP

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 变周期移动平均：周期随外部周期数组变化。
- **Meaning (EN)**: Moving Average with Variable Period — period varies by an external cycle array.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInMinPeriod | int | 2 | 1..100000 |
| optInMaxPeriod | int | 30 | 1..100000 |
| optInMAType | int-list | 0 | 离散整数列表 / discrete integer list |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MAVP(close, 2, 30, 0)`

### MIDPOINT

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 中值：窗口内最高与最低的中间值。
- **Meaning (EN)**: MidPoint over period — average of highest and lowest in the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MIDPOINT(close, 14)`

### MIDPRICE

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 中价：窗口内最高价与最低价的平均值。
- **Meaning (EN)**: MidPrice over period — (highest high + lowest low)/2 over the window.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MIDPRICE(close, 14)`

### SAR

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 抛物线 SAR：趋势跟踪的止损反转指标。
- **Meaning (EN)**: Parabolic SAR — trailing stop-and-reverse indicator for trend following.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInAcceleration | real | 0.02 | 0..无限制 |
| optInMaximum | real | 0.2 | 0..无限制 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SAR(close, 0.02, 0.2)`

### SAREXT

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 扩展抛物线 SAR：可配置加速极值参数的 SAR。
- **Meaning (EN)**: Parabolic SAR Extended — SAR with configurable acceleration extremes.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInStartValue | real | 0 | 无限制..无限制 |
| optInOffsetOnReverse | real | 0 | 0..无限制 |
| optInAccelerationInitLong | real | 0.02 | 0..无限制 |
| optInAccelerationLong | real | 0.02 | 0..无限制 |
| optInAccelerationMaxLong | real | 0.2 | 0..无限制 |
| optInAccelerationInitShort | real | 0.02 | 0..无限制 |
| optInAccelerationShort | real | 0.02 | 0..无限制 |
| optInAccelerationMaxShort | real | 0.2 | 0..无限制 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SAREXT(close, 0, 0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2)`

### SMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 简单移动平均：窗口内收盘价的算术平均。
- **Meaning (EN)**: Simple Moving Average — arithmetic mean of close over period.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_SMA(close, 30)`

### T3

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: T3 均线：带量价因子的三重平滑 EMA。
- **Meaning (EN)**: T3 Moving Average — triple-smoothed EMA with volume factor.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 1..100000 |
| optInVFactor | real | 0.7 | 0..1 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_T3(close, 5, 0.7)`

### TEMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 三重指数移动平均：相较 DEMA 进一步降低滞后。
- **Meaning (EN)**: Triple Exponential Moving Average — further lag reduction vs DEMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TEMA(close, 30)`

### TRIMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 三角移动平均：对称双重平滑的 SMA。
- **Meaning (EN)**: Triangular Moving Average — double-smoothed (symmetrical) SMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TRIMA(close, 30)`

### WMA

- **分组 / Group**: Overlap Studies / 重叠研究
- **含义（中文）**: 加权移动平均：线性加权，近期权重更大。
- **Meaning (EN)**: Weighted Moving Average — linearly weighted, recent prices heavier.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_WMA(close, 30)`


---

## Pattern Recognition / 形态识别

### CDL2CROWS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 两只乌鸦：看跌反转。
- **Meaning (EN)**: Two Crows bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL2CROWS(close)`

### CDL3BLACKCROWS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 三只乌鸦：看跌反转。
- **Meaning (EN)**: Three Black Crows bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL3BLACKCROWS(close)`

### CDL3INSIDE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 内困三日：反转形态。
- **Meaning (EN)**: Three Inside Up/Down reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL3INSIDE(close)`

### CDL3LINESTRIKE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 三线打击。
- **Meaning (EN)**: Three-Line Strike (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL3LINESTRIKE(close)`

### CDL3OUTSIDE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 外困三日：反转形态。
- **Meaning (EN)**: Three Outside Up/Down reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL3OUTSIDE(close)`

### CDL3STARSINSOUTH

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 南方三星：看涨。
- **Meaning (EN)**: Three Stars In The South bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL3STARSINSOUTH(close)`

### CDL3WHITESOLDIERS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 白色三兵：看涨。
- **Meaning (EN)**: Three Advancing White Soldiers bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDL3WHITESOLDIERS(close)`

### CDLABANDONEDBABY

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 弃婴：反转形态。
- **Meaning (EN)**: Abandoned Baby reversal (gap doji).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLABANDONEDBABY(close, 0.3)`

### CDLADVANCEBLOCK

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 推进块：看跌。
- **Meaning (EN)**: Advance Block bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLADVANCEBLOCK(close)`

### CDLBELTHOLD

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 腰带线。
- **Meaning (EN)**: Belt-hold line (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLBELTHOLD(close)`

### CDLBREAKAWAY

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 脱离形态：反转。
- **Meaning (EN)**: Breakaway reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLBREAKAWAY(close)`

### CDLCLOSINGMARUBOZU

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 收盘光头线。
- **Meaning (EN)**: Closing Marubozu (no upper shadow).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLCLOSINGMARUBOZU(close)`

### CDLCONCEALBABYSWALL

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 藏婴吞没：看涨。
- **Meaning (EN)**: Concealing Baby Swallow bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLCONCEALBABYSWALL(close)`

### CDLCOUNTERATTACK

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 反击线：反转。
- **Meaning (EN)**: Counterattack lines reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLCOUNTERATTACK(close)`

### CDLDARKCLOUDCOVER

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 乌云盖顶：看跌反转。
- **Meaning (EN)**: Dark Cloud Cover bearish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.5 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLDARKCLOUDCOVER(close, 0.5)`

### CDLDOJI

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 十字星：犹豫。
- **Meaning (EN)**: Doji indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLDOJI(close)`

### CDLDOJISTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 十字星：反转。
- **Meaning (EN)**: Doji Star reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLDOJISTAR(close)`

### CDLDRAGONFLYDOJI

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 蜻蜓十字：看涨反转。
- **Meaning (EN)**: Dragonfly Doji bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLDRAGONFLYDOJI(close)`

### CDLENGULFING

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 吞没形态。
- **Meaning (EN)**: Engulfing Pattern (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLENGULFING(close)`

### CDLEVENINGDOJISTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 黄昏十字星：看跌反转。
- **Meaning (EN)**: Evening Doji Star bearish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLEVENINGDOJISTAR(close, 0.3)`

### CDLEVENINGSTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 黄昏之星：看跌反转。
- **Meaning (EN)**: Evening Star bearish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLEVENINGSTAR(close, 0.3)`

### CDLGAPSIDESIDEWHITE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 并列阳线缺口。
- **Meaning (EN)**: Up/Down-gap Side-by-Side White lines.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLGAPSIDESIDEWHITE(close)`

### CDLGRAVESTONEDOJI

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 墓碑十字：看跌反转。
- **Meaning (EN)**: Gravestone Doji bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLGRAVESTONEDOJI(close)`

### CDLHAMMER

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 锤头：看涨反转。
- **Meaning (EN)**: Hammer bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHAMMER(close)`

### CDLHANGINGMAN

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 上吊线：看跌反转。
- **Meaning (EN)**: Hanging Man bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHANGINGMAN(close)`

### CDLHARAMI

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 孕线：反转。
- **Meaning (EN)**: Harami reversal (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHARAMI(close)`

### CDLHARAMICROSS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 十字孕线：反转。
- **Meaning (EN)**: Harami Cross reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHARAMICROSS(close)`

### CDLHIGHWAVE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 高价波动：犹豫。
- **Meaning (EN)**: High-Wave indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHIGHWAVE(close)`

### CDLHIKKAKE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 陷阱形态。
- **Meaning (EN)**: Hikkake Pattern (bullish/bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHIKKAKE(close)`

### CDLHIKKAKEMOD

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 改良陷阱形态。
- **Meaning (EN)**: Modified Hikkake Pattern.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHIKKAKEMOD(close)`

### CDLHOMINGPIGEON

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 家鸽：看涨。
- **Meaning (EN)**: Homing Pigeon bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLHOMINGPIGEON(close)`

### CDLIDENTICAL3CROWS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 等同三乌鸦：看跌。
- **Meaning (EN)**: Identical Three Crows bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLIDENTICAL3CROWS(close)`

### CDLINNECK

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 颈内线：反转。
- **Meaning (EN)**: In-Neck reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLINNECK(close)`

### CDLINVERTEDHAMMER

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 倒锤头：看涨反转。
- **Meaning (EN)**: Inverted Hammer bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLINVERTEDHAMMER(close)`

### CDLKICKING

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 反冲形态。
- **Meaning (EN)**: Kicking (bullish/bearish) gap lines.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLKICKING(close)`

### CDLKICKINGBYLENGTH

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 按长度反冲。
- **Meaning (EN)**: Kicking by Length.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLKICKINGBYLENGTH(close)`

### CDLLADDERBOTTOM

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 梯底：看涨。
- **Meaning (EN)**: Ladder Bottom bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLLADDERBOTTOM(close)`

### CDLLONGLEGGEDDOJI

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 长腿十字：犹豫。
- **Meaning (EN)**: Long-Legged Doji indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLLONGLEGGEDDOJI(close)`

### CDLLONGLINE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 长实体线。
- **Meaning (EN)**: Long Line candle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLLONGLINE(close)`

### CDLMARUBOZU

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 光头光脚线。
- **Meaning (EN)**: Marubozu (no shadows).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLMARUBOZU(close)`

### CDLMATCHINGLOW

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 呼应低：看涨。
- **Meaning (EN)**: Matching Low bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLMATCHINGLOW(close)`

### CDLMATHOLD

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 铺垫：看涨持续。
- **Meaning (EN)**: Mat Hold continuation (bullish).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.5 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLMATHOLD(close, 0.5)`

### CDLMORNINGDOJISTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 早晨十字星：看涨反转。
- **Meaning (EN)**: Morning Doji Star bullish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLMORNINGDOJISTAR(close, 0.3)`

### CDLMORNINGSTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 早晨之星：看涨反转。
- **Meaning (EN)**: Morning Star bullish reversal.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInPenetration | real | 0.3 | 0..无限制 |

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLMORNINGSTAR(close, 0.3)`

### CDLONNECK

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 颈上线：反转。
- **Meaning (EN)**: On-Neck reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLONNECK(close)`

### CDLPIERCING

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 刺透形态：看涨反转。
- **Meaning (EN)**: Piercing Pattern bullish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLPIERCING(close)`

### CDLRICKSHAWMAN

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 蜡烛图形态识别；输出为 0 / 100 / -100（整数）。
- **Meaning (EN)**: Candlestick pattern detector; output is 0 / 100 / -100 (integer).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLRICKSHAWMAN(close)`

### CDLRISEFALL3METHODS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 三法形态：持续。
- **Meaning (EN)**: Rising/Falling Three Methods continuation.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLRISEFALL3METHODS(close)`

### CDLSEPARATINGLINES

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 分离线：持续。
- **Meaning (EN)**: Separating Lines continuation.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLSEPARATINGLINES(close)`

### CDLSHOOTINGSTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 射击之星：看跌反转。
- **Meaning (EN)**: Shooting Star bearish reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLSHOOTINGSTAR(close)`

### CDLSHORTLINE

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 短实体线。
- **Meaning (EN)**: Short Line candle.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLSHORTLINE(close)`

### CDLSPINNINGTOP

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 纺锤线：犹豫。
- **Meaning (EN)**: Spinning Top indecision.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLSPINNINGTOP(close)`

### CDLSTALLEDPATTERN

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 停滞形态：看跌。
- **Meaning (EN)**: Stalled Pattern bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLSTALLEDPATTERN(close)`

### CDLSTICKSANDWICH

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**:  Stick 夹心：看涨。
- **Meaning (EN)**: Stick Sandwich bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLSTICKSANDWICH(close)`

### CDLTAKURI

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 垂柳线：看涨。
- **Meaning (EN)**: Takuri (dragonfly doji variant) bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLTAKURI(close)`

### CDLTASUKIGAP

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 跳空并列：持续。
- **Meaning (EN)**: Tasuki Gap continuation.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLTASUKIGAP(close)`

### CDLTHRUSTING

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 插入线：看跌持续。
- **Meaning (EN)**: Thrusting continuation (bearish).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLTHRUSTING(close)`

### CDLTRISTAR

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 三星：反转。
- **Meaning (EN)**: Tristar (three doji) reversal.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLTRISTAR(close)`

### CDLUNIQUE3RIVER

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 独特三河底：看涨。
- **Meaning (EN)**: Unique 3 River bottom bullish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLUNIQUE3RIVER(close)`

### CDLUPSIDEGAP2CROWS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 上跳双鸦：看跌。
- **Meaning (EN)**: Upside Gap Two Crows bearish.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLUPSIDEGAP2CROWS(close)`

### CDLXSIDEGAP3METHODS

- **分组 / Group**: Pattern Recognition / 形态识别
- **含义（中文）**: 跳空三法。
- **Meaning (EN)**: Upside/Downside Gap Three Methods.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outInteger` (integer)

- **策略示例 / DSL**: `TA_CDLXSIDEGAP3METHODS(close)`


---

## Price Transform / 价格变换

### AVGDEV

- **分组 / Group**: Price Transform / 价格变换
- **含义（中文）**: 由 OHLC 派生代表性价格的指标。
- **Meaning (EN)**: Derives a representative price from the OHLC tuple.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_AVGDEV(close, 14)`

### AVGPRICE

- **分组 / Group**: Price Transform / 价格变换
- **含义（中文）**: 平均价格：(高+低+开+收)/4。
- **Meaning (EN)**: Average Price — (high+low+open+close)/4.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_AVGPRICE(close)`

### MEDPRICE

- **分组 / Group**: Price Transform / 价格变换
- **含义（中文）**: 中价：(高+低)/2。
- **Meaning (EN)**: Median Price — (high+low)/2.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_MEDPRICE(close)`

### TYPPRICE

- **分组 / Group**: Price Transform / 价格变换
- **含义（中文）**: 典型价格：(高+低+收)/3。
- **Meaning (EN)**: Typical Price — (high+low+close)/3.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TYPPRICE(close)`

### WCLPRICE

- **分组 / Group**: Price Transform / 价格变换
- **含义（中文）**: 加权收盘价：(高+低+收·2)/4。
- **Meaning (EN)**: Weighted Close Price — (high+low+close·2)/4.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_WCLPRICE(close)`


---

## Statistic Functions / 统计函数

### BETA

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 贝塔：资产相对市场的线性回归斜率（本封装两者均取收盘价）。
- **Meaning (EN)**: Beta — slope of linear regression of asset vs market (both = close here).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_BETA(close, 5)`

### CORREL

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 皮尔逊相关系数：两序列在窗口内的相关性。
- **Meaning (EN)**: Pearson Correlation — correlation of two series over period.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 30 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_CORREL(close, 30)`

### LINEARREG

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 线性回归：最小二乘拟合线末端值。
- **Meaning (EN)**: Linear Regression — endpoint value of least-squares fit line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_LINEARREG(close, 14)`

### LINEARREG_ANGLE

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 线性回归角度：拟合线斜率（度）。
- **Meaning (EN)**: Linear Regression Angle — slope angle (degrees) of fitted line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_LINEARREG_ANGLE(close, 14)`

### LINEARREG_INTERCEPT

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 线性回归截距。
- **Meaning (EN)**: Linear Regression Intercept — y-intercept of fitted line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_LINEARREG_INTERCEPT(close, 14)`

### LINEARREG_SLOPE

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 线性回归斜率：每根变化量。
- **Meaning (EN)**: Linear Regression Slope — per-bar slope of fitted line.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_LINEARREG_SLOPE(close, 14)`

### STDDEV

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 标准差：收盘价相对 SMA 的离散度（×nbDev）。
- **Meaning (EN)**: Standard Deviation — σ of close around its SMA (×nbDev).

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 2..100000 |
| optInNbDev | real | 1 | 无限制..无限制 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_STDDEV(close, 5, 1)`

### TSF

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 时间序列预测：回归线向前投影一根。
- **Meaning (EN)**: Time Series Forecast — regression line projected one bar ahead.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TSF(close, 14)`

### VAR

- **分组 / Group**: Statistic Functions / 统计函数
- **含义（中文）**: 方差：收盘价相对 SMA 的离散平方。
- **Meaning (EN)**: Variance — σ² of close around its SMA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 5 | 1..100000 |
| optInNbDev | real | 1 | 无限制..无限制 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_VAR(close, 5, 1)`


---

## Volatility Indicators / 波动率指标

### ATR

- **分组 / Group**: Volatility Indicators / 波动率指标
- **含义（中文）**: 平均真实波幅：真实波幅的 Wilder 均值，衡量波动率。
- **Meaning (EN)**: Average True Range — Wilder average of True Range, measures volatility.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ATR(close, 14)`

### NATR

- **分组 / Group**: Volatility Indicators / 波动率指标
- **含义（中文）**: 归一化 ATR：ATR 除以收盘价，无量纲。
- **Meaning (EN)**: Normalized ATR — ATR divided by close, scale-independent.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInTimePeriod | int | 14 | 1..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_NATR(close, 14)`

### TRANGE

- **分组 / Group**: Volatility Indicators / 波动率指标
- **含义（中文）**: 真实波幅：三者最大值。
- **Meaning (EN)**: True Range — high-low, |high-prevClose|, |low-prevClose| (max).

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_TRANGE(close)`


---

## Volume Indicators / 成交量指标

### AD

- **分组 / Group**: Volume Indicators / 成交量指标
- **含义（中文）**: 累积/派发线：以收盘价在高低区间的位置加权的累计成交量。
- **Meaning (EN)**: Chaikin A/D Line — cumulative volume weighted by close location in range.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_AD(close)`

### ADOSC

- **分组 / Group**: Volume Indicators / 成交量指标
- **含义（中文）**: A/D 振荡器：快慢 A/D 均线之差。
- **Meaning (EN)**: Chaikin A/D Oscillator — fast A/D MA minus slow A/D MA.

- **参数 / Parameters**:
| 名称 Name | 类型 Type | 默认 Default | 范围 Range |
|---|---|---|---|
| optInFastPeriod | int | 3 | 2..100000 |
| optInSlowPeriod | int | 10 | 2..100000 |

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_ADOSC(close, 3, 10)`

### OBV

- **分组 / Group**: Volume Indicators / 成交量指标
- **含义（中文）**: 能量潮：按价格方向带符号累计成交量。
- **Meaning (EN)**: On Balance Volume — cumulative volume signed by price direction.

- **参数 / Parameters**: 无（无可选参数） / none

- **输出 / Outputs**: `outReal` (real)

- **策略示例 / DSL**: `TA_OBV(close)`

