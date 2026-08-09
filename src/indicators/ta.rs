//! TA-Lib 集成层（FFI + 安全封装）。
//!
//! 本模块通过 TA-Lib 的 **abstract API** 对接其全部函数（重叠研究、动量、成交量、
//! 波动率、价格变换、周期、形态识别、统计、数学变换/运算等约 150+ 个），而非
//! 逐个手写绑定。借助抽象 API，我们可以用一个通用入口按函数名调用任意指标。
//!
//! 设计要点：
//! - `call_ta_func` 是核心：给定 TA-Lib 函数名、K 线序列、可选参数与输出序号，
//!   返回与输入等长（前导为 `NAN`）的 `f64` 序列。
//! - `TaIndicator` 实现 `Indicator` trait，使任意 TA-Lib 函数可直接在已有策略
//!   DSL 中以 `TA_<FUNC>(src, p1, p2, ...)[.field]` 形式引用。
//! - 价格类输入统一传入完整的 OHLCV 结构，由 TA-Lib 按自身价格掩码选取所需分量
//!   （多数函数使用收盘价；如函数需要 high/low/close，也会自动取用）。
//!
//! TA-Lib C 库需在本机安装（本项目通过 `build.rs` 经 pkg-config 链接）。
//!
//! TA-Lib integration (FFI + safe wrapper).
//! Talks to the real TA-Lib through its abstract API so that *every* TA-Lib
//! function is reachable from a single generic entry point.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::sync::Once;

use crate::indicators::{Candle, Indicator, IndicatorId, PriceSource};

// ---- 基础类型 / 常量 (mirror of ta_defs.h) ----
#[allow(non_camel_case_types)]
type TA_RetCode = c_int;
#[allow(non_camel_case_types)]
type TA_Integer = c_int;
#[allow(non_camel_case_types)]
type TA_Real = f64;

const TA_SUCCESS: TA_RetCode = 0;

// TA_InputParameterType
#[allow(dead_code)]
const TA_INPUT_PRICE: c_int = 0;
#[allow(dead_code)]
const TA_INPUT_REAL: c_int = 1;
#[allow(dead_code)]
const TA_INPUT_INTEGER: c_int = 2;

// TA_OptInputParameterType
#[allow(dead_code)]
const TA_OPTIN_REALRANGE: c_int = 0;
#[allow(dead_code)]
const TA_OPTIN_REALLIST: c_int = 1;
#[allow(dead_code)]
const TA_OPTIN_INTEGERRANGE: c_int = 2;
#[allow(dead_code)]
const TA_OPTIN_INTEGERLIST: c_int = 3;

// TA_OutputParameterType
#[allow(dead_code)]
const TA_OUTPUT_REAL: c_int = 0;
#[allow(dead_code)]
const TA_OUTPUT_INTEGER: c_int = 1;

// ---- Opaque handles ----
#[repr(C)]
struct TA_FuncHandle {
    _private: [u8; 0],
}
#[repr(C)]
struct TA_ParamHolder {
    _private: [u8; 0],
}

// TA_FuncInfo layout must match ta_abstract.h exactly.
#[repr(C)]
struct TA_FuncInfo {
    name: *const c_char,
    group: *const c_char,
    hint: *const c_char,
    camel_case_name: *const c_char,
    flags: u32,
    nb_input: u32,
    nb_opt_input: u32,
    nb_output: u32,
    handle: *const TA_FuncHandle,
}

#[repr(C)]
struct TA_InputParameterInfo {
    type_: c_int,
    param_name: *const c_char,
    flags: c_int,
}

#[repr(C)]
struct TA_OptInputParameterInfo {
    type_: c_int,
    param_name: *const c_char,
    flags: c_int,
    display_name: *const c_char,
    data_set: *const c_void,
    default_value: TA_Real,
    help_file: *const c_char,
}

#[repr(C)]
struct TA_OutputParameterInfo {
    type_: c_int,
    param_name: *const c_char,
    flags: c_int,
}

// ---- FFI declarations (TA-Lib abstract API) ----
#[link(name = "ta-lib")]
#[allow(dead_code)]
unsafe extern "C" {
    fn TA_Initialize() -> TA_RetCode;
    fn TA_Shutdown() -> TA_RetCode;

    fn TA_GetFuncHandle(name: *const c_char, handle: *mut *const TA_FuncHandle) -> TA_RetCode;
    fn TA_GetFuncInfo(
        handle: *const TA_FuncHandle,
        info: *mut *const TA_FuncInfo,
    ) -> TA_RetCode;

    fn TA_GetInputParameterInfo(
        handle: *const TA_FuncHandle,
        param_idx: u32,
        info: *mut *const TA_InputParameterInfo,
    ) -> TA_RetCode;
    fn TA_GetOptInputParameterInfo(
        handle: *const TA_FuncHandle,
        param_idx: u32,
        info: *mut *const TA_OptInputParameterInfo,
    ) -> TA_RetCode;
    fn TA_GetOutputParameterInfo(
        handle: *const TA_FuncHandle,
        param_idx: u32,
        info: *mut *const TA_OutputParameterInfo,
    ) -> TA_RetCode;

    fn TA_ParamHolderAlloc(
        handle: *const TA_FuncHandle,
        allocated: *mut *mut TA_ParamHolder,
    ) -> TA_RetCode;
    fn TA_ParamHolderFree(params: *mut TA_ParamHolder);

    fn TA_SetInputParamPricePtr(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        open: *const TA_Real,
        high: *const TA_Real,
        low: *const TA_Real,
        close: *const TA_Real,
        volume: *const TA_Real,
        open_interest: *const TA_Real,
    ) -> TA_RetCode;
    fn TA_SetInputParamRealPtr(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        value: *const TA_Real,
    ) -> TA_RetCode;
    fn TA_SetInputParamIntegerPtr(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        value: *const TA_Integer,
    ) -> TA_RetCode;

    fn TA_SetOptInputParamInteger(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        value: TA_Integer,
    ) -> TA_RetCode;
    fn TA_SetOptInputParamReal(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        value: TA_Real,
    ) -> TA_RetCode;

    fn TA_SetOutputParamRealPtr(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        out: *mut TA_Real,
    ) -> TA_RetCode;
    fn TA_SetOutputParamIntegerPtr(
        params: *mut TA_ParamHolder,
        param_idx: u32,
        out: *mut TA_Integer,
    ) -> TA_RetCode;

    fn TA_GetLookback(params: *const TA_ParamHolder, lookback: *mut TA_Integer) -> TA_RetCode;
    fn TA_CallFunc(
        params: *const TA_ParamHolder,
        start_idx: TA_Integer,
        end_idx: TA_Integer,
        out_beg_idx: *mut TA_Integer,
        out_nb_element: *mut TA_Integer,
    ) -> TA_RetCode;
}

static INIT: Once = Once::new();

/// 确保 TA-Lib 全局状态已初始化（幂等）。
fn ensure_initialized() {
    INIT.call_once(|| {
        unsafe {
            TA_Initialize();
        }
    });
}

/// 判断某 TA-Lib 函数名是否存在。
pub fn ta_function_exists(name: &str) -> bool {
    ensure_initialized();
    let cname = match CString::new(name) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut handle: *const TA_FuncHandle = ptr::null();
    unsafe { TA_GetFuncHandle(cname.as_ptr(), &mut handle) == TA_SUCCESS && !handle.is_null() }
}

/// 取得某 TA-Lib 函数的全部输出字段名（用于 `.field` 选择）。
pub fn ta_output_names(name: &str) -> Option<Vec<String>> {
    ensure_initialized();
    let cname = CString::new(name).ok()?;
    let mut handle: *const TA_FuncHandle = ptr::null();
    unsafe {
        if TA_GetFuncHandle(cname.as_ptr(), &mut handle) != TA_SUCCESS || handle.is_null() {
            return None;
        }
        let mut info: *const TA_FuncInfo = ptr::null();
        if TA_GetFuncInfo(handle, &mut info) != TA_SUCCESS || info.is_null() {
            return None;
        }
        let nb = (*info).nb_output as usize;
        let mut names = Vec::with_capacity(nb);
        for i in 0..nb {
            let mut oinfo: *const TA_OutputParameterInfo = ptr::null();
            if TA_GetOutputParameterInfo(handle, i as u32, &mut oinfo) != TA_SUCCESS
                || oinfo.is_null()
            {
                names.push(format!("out{i}"));
                continue;
            }
            let pname = (*oinfo).param_name;
            if pname.is_null() {
                names.push(format!("out{i}"));
            } else {
                let s = CStr::from_ptr(pname).to_string_lossy().into_owned();
                names.push(s);
            }
        }
        Some(names)
    }
}

/// 将 `field`（None / 整数字符串 / 输出名）解析为输出序号。
fn resolve_output_index(name: &str, field: Option<&str>) -> usize {
    match field {
        None => 0,
        Some(f) => {
            // 直接数字：0,1,2 ...
            if let Ok(n) = f.parse::<usize>() {
                return n;
            }
            // 否则按输出名（大小写不敏感）匹配
            if let Some(names) = ta_output_names(name) {
                if let Some(pos) = names
                    .iter()
                    .position(|n| n.eq_ignore_ascii_case(f))
                {
                    return pos;
                }
            }
            0
        }
    }
}

/// 通用调用入口：按函数名计算某输出序列。
///
/// - `name`    : TA-Lib 函数名（如 "RSI"、"BBANDS"、"MACD"）。
/// - `series`  : 输入 K 线（OHLCV）。
/// - `params`  : 可选参数（时间周期 / MAType / 偏差等），不足时取 TA-Lib 默认值。
/// - `out_idx` : 输出序号（多输出函数如 MACD=3、BBANDS=3；默认 0）。
///
/// 返回与 `series` 等长的 `Vec<f64>`，前导无效区为 `NAN`。
pub fn call_ta_func(
    name: &str,
    series: &[Candle],
    params: &[f64],
    out_idx: usize,
) -> Option<Vec<f64>> {
    ensure_initialized();
    let n = series.len();
    if n == 0 {
        return Some(Vec::new());
    }

    let cname = CString::new(name).ok()?;
    let mut handle: *const TA_FuncHandle = ptr::null();
    unsafe {
        if TA_GetFuncHandle(cname.as_ptr(), &mut handle) != TA_SUCCESS || handle.is_null() {
            return None;
        }
    }

    // 函数元信息
    let mut info: *const TA_FuncInfo = ptr::null();
    let (nb_input, nb_opt, nb_output) = unsafe {
        if TA_GetFuncInfo(handle, &mut info) != TA_SUCCESS || info.is_null() {
            return None;
        }
        (
            (*info).nb_input as usize,
            (*info).nb_opt_input as usize,
            (*info).nb_output as usize,
        )
    };
    if nb_output == 0 {
        return None;
    }
    let out_idx = out_idx.min(nb_output - 1);

    // 价格数组（保持存活直到调用结束）
    let open: Vec<TA_Real> = series.iter().map(|c| c.open).collect();
    let high: Vec<TA_Real> = series.iter().map(|c| c.high).collect();
    let low: Vec<TA_Real> = series.iter().map(|c| c.low).collect();
    let close: Vec<TA_Real> = series.iter().map(|c| c.close).collect();
    let volume: Vec<TA_Real> = series.iter().map(|c| c.volume).collect();
    let oi: Vec<TA_Real> = vec![0.0; n];

    // 分配参数持有者
    let mut holder: *mut TA_ParamHolder = ptr::null_mut();
    unsafe {
        if TA_ParamHolderAlloc(handle, &mut holder) != TA_SUCCESS || holder.is_null() {
            return None;
        }
    }

    // 设置输入：价格类传完整 OHLCV；其余（极少）传收盘价数组
    for i in 0..nb_input {
        let mut iinfo: *const TA_InputParameterInfo = ptr::null();
        unsafe {
            if TA_GetInputParameterInfo(handle, i as u32, &mut iinfo) != TA_SUCCESS
                || iinfo.is_null()
            {
                continue;
            }
            if (*iinfo).type_ == TA_INPUT_PRICE {
                TA_SetInputParamPricePtr(
                    holder,
                    i as u32,
                    open.as_ptr(),
                    high.as_ptr(),
                    low.as_ptr(),
                    close.as_ptr(),
                    volume.as_ptr(),
                    oi.as_ptr(),
                );
            } else {
                TA_SetInputParamRealPtr(holder, i as u32, close.as_ptr());
            }
        }
    }

    // 设置可选参数：用户提供则采用，否则取默认值；整数型四舍五入
    for i in 0..nb_opt {
        let mut oinfo: *const TA_OptInputParameterInfo = ptr::null();
        unsafe {
            if TA_GetOptInputParameterInfo(handle, i as u32, &mut oinfo) != TA_SUCCESS
                || oinfo.is_null()
            {
                continue;
            }
            let t = (*oinfo).type_;
            let def = (*oinfo).default_value;
            let val = params.get(i).copied().unwrap_or(def);
            if t == TA_OPTIN_INTEGERRANGE || t == TA_OPTIN_INTEGERLIST {
                TA_SetOptInputParamInteger(holder, i as u32, val.round() as TA_Integer);
            } else {
                TA_SetOptInputParamReal(holder, i as u32, val);
            }
        }
    }

    // 为全部输出预分配缓冲（real / integer 各一个，简单且无映射错误）
    let mut out_real: Vec<Vec<TA_Real>> = (0..nb_output).map(|_| vec![0.0; n]).collect();
    let mut out_int: Vec<Vec<TA_Integer>> = (0..nb_output).map(|_| vec![0; n]).collect();
    for i in 0..nb_output {
        let mut oinfo: *const TA_OutputParameterInfo = ptr::null();
        unsafe {
            if TA_GetOutputParameterInfo(handle, i as u32, &mut oinfo) != TA_SUCCESS
                || oinfo.is_null()
            {
                continue;
            }
            if (*oinfo).type_ == TA_OUTPUT_INTEGER {
                TA_SetOutputParamIntegerPtr(holder, i as u32, out_int[i].as_mut_ptr());
            } else {
                TA_SetOutputParamRealPtr(holder, i as u32, out_real[i].as_mut_ptr());
            }
        }
    }

    // 计算
    let mut out_beg: TA_Integer = 0;
    let mut out_nb: TA_Integer = 0;
    let end = (n as TA_Integer) - 1;
    let rc = unsafe {
        let r = TA_CallFunc(holder, 0, end, &mut out_beg, &mut out_nb);
        TA_ParamHolderFree(holder);
        r
    };
    if rc != TA_SUCCESS {
        return None;
    }

    // 对齐输出：TA-Lib 从 out_beg 开始填充 out_nb 个元素，其余为 NAN
    let mut result = vec![f64::NAN; n];
    let ob = out_beg as usize;
    let on = out_nb as usize;
    let is_int = unsafe {
        let mut oinfo: *const TA_OutputParameterInfo = ptr::null();
        TA_GetOutputParameterInfo(handle, out_idx as u32, &mut oinfo);
        oinfo.is_null() || (*oinfo).type_ == TA_OUTPUT_INTEGER
    };
    if is_int {
        for k in 0..on {
            result[ob + k] = out_int[out_idx][k] as f64;
        }
    } else {
        for k in 0..on {
            result[ob + k] = out_real[out_idx][k];
        }
    }
    Some(result)
}

/// 可作为策略 DSL 指标的 TA-Lib 封装。
pub struct TaIndicator {
    name: String,
    params: Vec<f64>,
    field: Option<String>,
}

impl TaIndicator {
    /// 构造；若函数名不存在返回 `None`（DSL 求值将得到 NAN -> 不触发）。
    pub fn try_new(name: &str, params: Vec<f64>, field: Option<String>) -> Option<Self> {
        if ta_function_exists(name) {
            Some(TaIndicator {
                name: name.to_string(),
                params,
                field,
            })
        } else {
            None
        }
    }
}

impl Indicator for TaIndicator {
    fn id(&self) -> IndicatorId {
        IndicatorId {
            kind: format!("TA_{}", self.name),
            source: PriceSource::Close,
            params: self.params.clone(),
            field: self.field.clone(),
        }
    }

    fn eval(&self, series: &[Candle]) -> Vec<f64> {
        let out_idx = resolve_output_index(&self.name, self.field.as_deref());
        call_ta_func(&self.name, series, &self.params, out_idx)
            .unwrap_or_else(|| vec![f64::NAN; series.len()])
    }
}

/// 列出所有可用 TA-Lib 函数（名称, 分组）。用于文档生成与自检。
pub fn list_all_functions() -> Vec<(String, String)> {
    use std::cell::RefCell;

    ensure_initialized();

    extern "C" fn cb(info: *const TA_FuncInfo, opaque: *mut c_void) {
        if info.is_null() {
            return;
        }
        unsafe {
            let name = if (*info).name.is_null() {
                String::new()
            } else {
                CStr::from_ptr((*info).name).to_string_lossy().into_owned()
            };
            let group = if (*info).group.is_null() {
                String::new()
            } else {
                CStr::from_ptr((*info).group).to_string_lossy().into_owned()
            };
            let vec = &*(opaque as *const RefCell<Vec<(String, String)>>);
            vec.borrow_mut().push((name, group));
        }
    }

    let buf = RefCell::new(Vec::new());
    let opaque = &buf as *const RefCell<Vec<(String, String)>> as *mut c_void;
    unsafe {
        unsafe extern "C" {
            fn TA_ForEachFunc(
                func: Option<extern "C" fn(*const TA_FuncInfo, *mut c_void)>,
                opaque: *mut c_void,
            );
        }
        TA_ForEachFunc(Some(cb), opaque);
    }
    buf.into_inner()
}

/// 可选参数元信息（用于文档生成）。
pub struct TaOptInput {
    pub name: String,
    pub display: String,
    pub kind: String, // "int" | "real" | "int-list" | "real-list"
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
    pub outputs: Vec<(String, String)>, // (name, "real" | "integer")
}

#[repr(C)]
struct TA_RealRange {
    min: f64,
    max: f64,
    precision: i32,
    suggested_start: f64,
    suggested_end: f64,
    suggested_increment: f64,
}

#[repr(C)]
struct TA_IntegerRange {
    min: i32,
    max: i32,
    suggested_start: i32,
    suggested_end: i32,
    suggested_increment: i32,
}

/// 取得某函数的完整元信息（可选参数含默认值/范围、输出字段名与类型）。
pub fn ta_meta(name: &str) -> Option<TaFuncMeta> {
    ensure_initialized();
    let cname = CString::new(name).ok()?;
    let mut handle: *const TA_FuncHandle = ptr::null();
    unsafe {
        if TA_GetFuncHandle(cname.as_ptr(), &mut handle) != TA_SUCCESS || handle.is_null() {
            return None;
        }
    }
    let mut info: *const TA_FuncInfo = ptr::null();
    unsafe {
        if TA_GetFuncInfo(handle, &mut info) != TA_SUCCESS || info.is_null() {
            return None;
        }
    }
    let (group, hint) = unsafe {
        let group = if (*info).group.is_null() {
            String::new()
        } else {
            CStr::from_ptr((*info).group).to_string_lossy().into_owned()
        };
        let hint = if (*info).hint.is_null() {
            String::new()
        } else {
            CStr::from_ptr((*info).hint).to_string_lossy().into_owned()
        };
        (group, hint)
    };

    // 可选参数
    let nb_opt = unsafe { (*info).nb_opt_input as usize };
    let mut opt_inputs = Vec::with_capacity(nb_opt);
    for i in 0..nb_opt {
        let mut oinfo: *const TA_OptInputParameterInfo = ptr::null();
        unsafe {
            if TA_GetOptInputParameterInfo(handle, i as u32, &mut oinfo) != TA_SUCCESS
                || oinfo.is_null()
            {
                continue;
            }
            let t = (*oinfo).type_;
            let pname = if (*oinfo).param_name.is_null() {
                format!("opt{i}")
            } else {
                CStr::from_ptr((*oinfo).param_name).to_string_lossy().into_owned()
            };
            let disp = if (*oinfo).display_name.is_null() {
                String::new()
            } else {
                CStr::from_ptr((*oinfo).display_name).to_string_lossy().into_owned()
            };
            let def = (*oinfo).default_value;
            let (kind, min, max) = if t == TA_OPTIN_INTEGERRANGE {
                let ds = (*oinfo).data_set as *const TA_IntegerRange;
                (
                    "int".to_string(),
                    Some((*ds).min as f64),
                    Some((*ds).max as f64),
                )
            } else if t == TA_OPTIN_REALRANGE {
                let ds = (*oinfo).data_set as *const TA_RealRange;
                (
                    "real".to_string(),
                    Some((*ds).min),
                    Some((*ds).max),
                )
            } else if t == TA_OPTIN_INTEGERLIST {
                ("int-list".to_string(), None, None)
            } else {
                ("real-list".to_string(), None, None)
            };
            opt_inputs.push(TaOptInput {
                name: pname,
                display: disp,
                kind,
                default: def,
                min,
                max,
            });
        }
    }

    // 输出
    let nb_out = unsafe { (*info).nb_output as usize };
    let mut outputs = Vec::with_capacity(nb_out);
    for i in 0..nb_out {
        let mut oinfo: *const TA_OutputParameterInfo = ptr::null();
        unsafe {
            if TA_GetOutputParameterInfo(handle, i as u32, &mut oinfo) != TA_SUCCESS
                || oinfo.is_null()
            {
                outputs.push((format!("out{i}"), "real".to_string()));
                continue;
            }
            let pname = if (*oinfo).param_name.is_null() {
                format!("out{i}")
            } else {
                CStr::from_ptr((*oinfo).param_name).to_string_lossy().into_owned()
            };
            let ty = if (*oinfo).type_ == TA_OUTPUT_INTEGER {
                "integer".to_string()
            } else {
                "real".to_string()
            };
            outputs.push((pname, ty));
        }
    }

    Some(TaFuncMeta {
        name: name.to_string(),
        group,
        hint,
        opt_inputs,
        outputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ta_exists_check() {
        assert!(ta_function_exists("RSI"));
        assert!(ta_function_exists("BBANDS"));
        assert!(!ta_function_exists("NOT_A_REAL_FUNC"));
    }

    #[test]
    fn ta_list_nonempty() {
        let all = list_all_functions();
        assert!(all.len() > 100, "TA-Lib should expose >100 functions");
    }
}
