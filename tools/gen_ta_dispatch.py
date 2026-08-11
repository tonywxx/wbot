#!/usr/bin/env python3
"""Generate src/indicators/ta_dispatch.rs: an adaq-talib-backed dispatch for all TA_* indicators.

Sources of truth:
  - adaq-talib 0.1.1 signatures + `_default` variant bodies  -> opt-input order / type / defaults (authoritative)
  - docs/ta-lib-indicators.bilingual.md                       -> the 161 canonical TA-Lib names + group + human meaning

Robustness notes:
  - Only the 161 TA-Lib functions listed in the doc are emitted. adaq-talib also exposes helper
    fns (approx_eq, rolling_*, candle_*, new, advance, ...) that are NOT TA-Lib functions and are
    excluded so the generated code always compiles.
  - Multi-input functions (candlesticks take 4 OHLC inputs, BOP takes 4, AD/ADOSC/MFI take
    H/L/C/V, etc.) are wired by *parameter name* (open/high/low/close/volume) read straight from
    the real adaq-talib signature, so every arity is correct without a hand-maintained table.
"""
import re, os


def rust_str(s: str) -> str:
    """Emit a valid Rust double-quoted string literal (escaping \\ and ")."""
    esc = s.replace("\\", "\\\\").replace('"', '\\"')
    return '"' + esc + '"'


ADAQ = os.environ.get(
    "ADAQ_SRC",
    os.path.expanduser("~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/adaq-talib-0.1.1/src"),
)
DOC = "/Users/tony/github/wbot/docs/ta-lib-indicators.bilingual.md"
OUT = "/Users/tony/github/wbot/src/indicators/ta_dispatch.rs"

# ---------------------------------------------------------------- collect signatures
sigs = {}  # fn -> dict(module, inputs[name list], opts[type list], ret)
for root, _, files in os.walk(ADAQ):
    for f in files:
        if not f.endswith(".rs"):
            continue
        path = os.path.join(root, f)
        rel = os.path.relpath(path, ADAQ).replace(os.sep, "::")[:-3]  # momentum / pattern::batch_1
        # skip the private internal `core` module and crate-root files (not public TA API)
        if rel in ("lib", "main", "error") or rel.startswith("core"):
            continue
        src = open(path, encoding="utf-8").read()
        for m in re.finditer(r"pub fn (\w+)\s*\(", src):
            fn = m.group(1)
            if fn.endswith("_default") or fn.endswith("_with_output"):
                continue
            start = m.end() - 1
            depth = 0
            i = start
            while i < len(src):
                c = src[i]
                if c == "(":
                    depth += 1
                elif c == ")":
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            params = src[start + 1:i]
            rm = re.match(r"\s*->\s*([^\{;]+)", src[i + 1:])
            ret = rm.group(1).strip() if rm else ""
            plist = [p.strip() for p in params.split(",") if p.strip()]
            inputs = []   # names of the &[f64] params, in order
            opts = []
            for p in plist:
                if "&[f64]" in p:
                    inputs.append(p.split(":")[0].strip())
                    continue
                if p.endswith("MaType"):
                    opts.append("matype")
                elif p.endswith("usize"):
                    opts.append("usize")
                elif p.endswith("f64"):
                    opts.append("f64")
                else:
                    opts.append("other:" + p)
            if ret.startswith("Result<Vec<f64>"):
                rtype = "vec"
            else:
                mm = re.match(r"Result<(\w+),", ret)
                rtype = mm.group(1) if mm else "vec"
            sigs[fn] = dict(module=rel, inputs=inputs, opts=opts, ret=rtype)

# ---------------------------------------------------------------- constants from defaults.rs
CONST = {}
dfile = os.path.join(ADAQ, "core", "defaults.rs")
if os.path.exists(dfile):
    for line in open(dfile, encoding="utf-8"):
        m = re.match(r"pub const (\w+):\s*(?:usize|f64)\s*=\s*([0-9.eE+-]+);", line)
        if m:
            CONST[m.group(1)] = float(m.group(2))

# ---------------------------------------------------------------- defaults from _default bodies
def_body = {}
for root, _, files in os.walk(ADAQ):
    for f in files:
        if not f.endswith(".rs"):
            continue
        path = os.path.join(root, f)
        rel = os.path.relpath(path, ADAQ).replace(os.sep, "::")[:-3]
        if rel in ("lib", "main", "error") or rel.startswith("core"):
            continue
        src = open(os.path.join(root, f), encoding="utf-8").read()
        for m in re.finditer(r"pub fn (\w+_default)\s*\(", src):
            name = m.group(1)            # e.g. bbands_default
            base = name[:-len("_default")]  # bbands
            start = m.end() - 1
            depth = 0
            i = start
            while i < len(src):
                c = src[i]
                if c == "{":
                    depth += 1
                elif c == "}":
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            body = src[m.end():i]
            # Find the call that supplies the defaults. Prefer the same-named fn; some
            # `_default` variants delegate to a *different* adaq fn (e.g. macd_ext_default ->
            # macd(values, MACD_FAST, MACD_SLOW, MACD_SIGNAL)). Only fall back to a delegated
            # call whose input arity matches `base`, so we never misalign opt-input positions.
            calls = re.findall(r"\b([a-z_]\w*)\s*\(", body)
            cands = [c for c in calls if c in sigs]
            target = base if base in cands else None
            if target is None:
                nin_base = len(sigs.get(base, {}).get("inputs", []))
                for c in cands:
                    if len(sigs[c]["inputs"]) == nin_base:
                        target = c
                        break
            if target is None:
                continue
            cm = re.search(re.escape(target) + r"\s*\(", body)
            if not cm:
                continue
            cs = cm.end() - 1
            d2 = 0
            j = cs
            while j < len(body):
                c = body[j]
                if c == "(":
                    d2 += 1
                elif c == ")":
                    d2 -= 1
                    if d2 == 0:
                        break
                j += 1
            argstr = body[cs + 1:j]
            args = []
            buf = ""
            dd = 0
            for ch in argstr:
                if ch == "(":
                    dd += 1
                    buf += ch
                elif ch == ")":
                    dd -= 1
                    buf += ch
                elif ch == "," and dd == 0:
                    args.append(buf.strip())
                    buf = ""
                else:
                    buf += ch
            if buf.strip():
                args.append(buf.strip())
            def_body[base] = args

def default_of(arg, opt_type):
    a = arg.strip()
    if opt_type == "matype":
        m = re.match(r"MaType::(\w+)", a)
        if m:
            names = ["Sma", "Ema", "Wma", "Dema", "Tema", "Trima", "Kama", "Mama"]
            return float(names.index(m.group(1)) if m.group(1) in names else 0)
        return 0.0
    if re.fullmatch(r"[0-9.eE+-]+", a):
        return float(a)
    if a in CONST:
        return CONST[a]
    return 30.0 if opt_type == "usize" else 0.0

# ---------------------------------------------------------------- input binding by param name
def input_token(name: str) -> str:
    n = name.lower()
    if "open" in n:
        return "open"
    if "high" in n:
        return "high"
    if "low" in n:
        return "low"
    if "close" in n:
        return "close"
    if "volume" in n or n == "vol":
        return "volume"
    if "period" in n:
        return "periods"
    return "src"  # real0/real1/values/prices/data/real -> the selected price series

TOKEN = {"open": "open", "high": "high", "low": "low", "close": "close",
         "volume": "volume", "src": "src", "periods": "periods"}
STRUCT_BRIDGE = {
    "TA_MACD": ("Macd", ["macd", "signal", "hist"]),
    "TA_MACDEXT": ("Macd", ["macd", "signal", "hist"]),
    "TA_MACDFIX": ("Macd", ["macd", "signal", "hist"]),
    "TA_AROON": ("Aroon", ["up", "down"]),
    "TA_STOCH": ("Stoch", ["slow_k", "slow_d"]),
    "TA_STOCHF": ("StochF", ["fast_k", "fast_d"]),
    "TA_MAMA": ("Mama", ["mama", "fama"]),
    "TA_HT_PHASOR": ("HtPhasor", ["in_phase", "quadrature"]),
    "TA_HT_SINE": ("HtSine", ["sine", "lead_sine"]),
    "TA_MINMAX": ("MinMax", ["min", "max"]),
    "TA_MINMAXINDEX": ("MinMaxIndex", ["min_idx", "max_idx"]),
    "TA_BBANDS": ("Bbands", ["upper", "middle", "lower"]),
    "TA_ACCBANDS": ("AccBands", ["upper", "middle", "lower"]),
}

# ---------------------------------------------------------------- parse doc for meta + canonical set
DOC_DATA = {}  # ta_name -> dict(group, params[(name,type,default,range)], outputs[(name,type)])
if os.path.exists(DOC):
    text = open(DOC, encoding="utf-8").read()
    blocks = re.split(r"^### ", text, flags=re.M)
    for blk in blocks[1:]:
        lines = blk.splitlines()
        name = lines[0].strip()
        ta_name = "TA_" + name.upper()
        group = ""
        params = []
        outputs = []
        for ln in lines:
            mg = re.match(r"- \*\*分组 / Group\*\*:\s*(.*)", ln)
            if mg:
                group = mg.group(1).strip()
            if ln.strip().startswith("|") and "名称 Name" not in ln and "---" not in ln:
                cells = [c.strip() for c in ln.strip().strip("|").split("|")]
                if len(cells) >= 4 and cells[0].startswith("optIn"):
                    params.append((cells[0], cells[1], cells[2], cells[3]))
            mo = re.match(r"- \*\*输出 / Outputs\*\*:\s*(.*)", ln)
            if mo:
                for ent in mo.group(1).split(","):
                    em = re.match(r"`?([\w.]+)`?\s*\((\w+)\)", ent.strip())
                    if em:
                        outputs.append((em.group(1), em.group(2)))
        DOC_DATA[ta_name] = dict(group=group, params=params, outputs=outputs)

# only the 161 TA-Lib names are real; map each adaq fn -> canonical TA name via normalized form
doc_by_norm = {}
for ta_name in DOC_DATA:
    norm = ta_name.replace("TA_", "").replace("_", "").upper()
    doc_by_norm[norm] = ta_name

def ta_name_of(fn):
    norm = fn.upper().replace("_", "")
    return doc_by_norm.get(norm, "TA_" + norm)

# ---------------------------------------------------------------- emit
L = []
L.append("//! 自动生成 —— TA-Lib 指标分发层（adaq-talib 后端）。")
L.append("//!")
L.append("//! 由 `tools/gen_ta_dispatch.py` 依据 adaq-talib 0.1.1 的签名与 `_default` 变体生成。")
L.append("//! 所有 `TA_*` 指标经由 adaq-talib（纯 Rust、零 FFI）计算，覆盖 TA-Lib 0.7.1 全部 161 个函数。")
L.append("")
L.append("use crate::indicators::{Candle, PriceSource};")
L.append("")
L.append("#[inline]")
L.append("fn pu(p: &[f64], i: usize, d: f64) -> usize { p.get(i).copied().unwrap_or(d).round().max(0.0) as usize }")
L.append("#[inline]")
L.append("fn pr(p: &[f64], i: usize, d: f64) -> f64 { p.get(i).copied().unwrap_or(d) }")
L.append("#[inline]")
L.append("fn mat(p: &[f64], i: usize, d: f64) -> adaq_talib::overlap::MaType {")
L.append("    ma_type_from(p.get(i).copied().unwrap_or(d).round() as i32)")
L.append("}")
L.append("")
L.append("fn ma_type_from(v: i32) -> adaq_talib::overlap::MaType {")
L.append("    use adaq_talib::overlap::MaType::*;")
L.append("    match v {")
L.append("        0 => Sma, 1 => Ema, 2 => Wma, 3 => Dema, 4 => Tema, 5 => Trima, 6 => Kama, 7 => Mama, _ => Sma,")
L.append("    }")
L.append("}")
L.append("")
L.append("#[allow(unused_variables)]")
L.append("pub fn call_adaq(")
L.append("    name: &str,")
L.append("    candles: &[Candle],")
L.append("    source: PriceSource,")
L.append("    params: &[f64],")
L.append("    field: Option<&str>,")
L.append(") -> Option<Vec<f64>> {")
L.append("    let n = candles.len();")
L.append("    if n == 0 {")
L.append("        return Some(Vec::new());")
L.append("    }")
L.append("    let open: Vec<f64> = candles.iter().map(|c| c.open).collect();")
L.append("    let high: Vec<f64> = candles.iter().map(|c| c.high).collect();")
L.append("    let low: Vec<f64> = candles.iter().map(|c| c.low).collect();")
L.append("    let close: Vec<f64> = candles.iter().map(|c| c.close).collect();")
L.append("    let volume: Vec<f64> = candles.iter().map(|c| c.volume).collect();")
L.append("    let src: Vec<f64> = candles.iter().map(|c| source.value(c)).collect();")
L.append("    match name {")

emitted = set()
for fn, s in sorted(sigs.items()):
    ta_name = ta_name_of(fn)
    if ta_name not in DOC_DATA:
        # adaq-only helper (approx_eq, rolling_*, candle_*, new, advance, ...) — skip.
        continue
    emitted.add(ta_name)
    module = s["module"]
    opts = s["opts"]
    ret = s["ret"]

    if fn == "mavp":
        # values -> &src, synthetic periods array -> &periods
        dargs = def_body.get(fn)
        d0 = default_of(dargs[2], "usize") if (dargs and len(dargs) > 2) else 30.0
        opt_exprs = []
        for idx, ot in enumerate(opts):
            default = 30.0
            if dargs is not None and len(dargs) > 2 + idx:
                default = default_of(dargs[2 + idx], ot)
            if ot == "usize":
                opt_exprs.append("pu(params, %d, %s)" % (idx, repr(float(default))))
            elif ot == "f64":
                opt_exprs.append("pr(params, %d, %s)" % (idx, repr(float(default))))
            elif ot == "matype":
                opt_exprs.append("mat(params, %d, %s)" % (idx, repr(float(default))))
            else:
                opt_exprs.append("pr(params, %d, 0.0)" % idx)
        L.append('        "%s" => {' % ta_name)
        L.append("            let periods = vec![%s; n];" % repr(float(d0)))
        L.append("            let __r = adaq_talib::%s::%s(&src, &periods, %s).ok()?;" % (module, fn, ", ".join(opt_exprs)))
        L.append("            Some(__r)")
        L.append("        }")
        continue

    toks = [input_token(nm) for nm in s["inputs"]]
    in_vars = ", ".join("&" + TOKEN[t] for t in toks)
    dargs = def_body.get(fn)
    opt_exprs = []
    nin = len(s["inputs"])
    for idx, ot in enumerate(opts):
        default = 30.0
        if dargs is not None and len(dargs) > nin + idx:
            default = default_of(dargs[nin + idx], ot)
        if ot == "usize":
            opt_exprs.append("pu(params, %d, %s)" % (idx, repr(float(default))))
        elif ot == "f64":
            opt_exprs.append("pr(params, %d, %s)" % (idx, repr(float(default))))
        elif ot == "matype":
            opt_exprs.append("mat(params, %d, %s)" % (idx, repr(float(default))))
        else:
            opt_exprs.append("pr(params, %d, 0.0)" % idx)
    call_args = ", ".join([in_vars] + opt_exprs) if opt_exprs else in_vars
    L.append('        "%s" => {' % ta_name)
    if ret == "vec":
        L.append("            let __v = adaq_talib::%s::%s(%s).ok()?;" % (module, fn, call_args))
        L.append("            Some(__v)")
    else:
        sb = STRUCT_BRIDGE.get(ta_name)
        if sb is None:
            L.append("            let __r = adaq_talib::%s::%s(%s).ok()?;" % (module, fn, call_args))
            L.append("            Some(__r)")
            L.append("        }")
            continue
        struct_fields = sb[1]
        first = struct_fields[0]
        L.append("            let __r = adaq_talib::%s::%s(%s).ok()?;" % (module, fn, call_args))
        L.append("            Some(match field {")
        for fld in struct_fields:
            L.append('                Some(s) if s.eq_ignore_ascii_case("%s") => __r.%s,' % (fld, fld))
        L.append("                _ => __r.%s," % first)
        L.append("            })")
    L.append("        }")

L.append("        _ => None,")
L.append("    }")
L.append("}")
L.append("")

# ---- TaOptInput / TaFuncMeta structs
L.append("/// TA_* 函数元信息（用于文档生成）。")
L.append("pub struct TaOptInput {")
L.append("    pub name: String,")
L.append("    pub display: String,")
L.append("    pub kind: String,")
L.append("    pub default: f64,")
L.append("    pub min: Option<f64>,")
L.append("    pub max: Option<f64>,")
L.append("}")
L.append("")
L.append("/// 函数元信息（用于文档生成）。")
L.append("pub struct TaFuncMeta {")
L.append("    pub name: String,")
L.append("    pub group: String,")
L.append("    pub hint: String,")
L.append("    pub opt_inputs: Vec<TaOptInput>,")
L.append("    pub outputs: Vec<(String, String)>,")
L.append("}")
L.append("")

# ta_function_exists / _known (only the 161 emitted TA names)
L.append("/// 判断某 TA_* 函数是否被 adaq-talib 支持。")
L.append("pub fn ta_function_exists(name: &str) -> bool {")
L.append("    _known(name)")
L.append("}")
L.append("")
L.append("fn _known(name: &str) -> bool {")
L.append("    matches!(")
L.append("        name,")
for i, ta_name in enumerate(sorted(emitted)):
    pat = rust_str(ta_name)
    if i < len(emitted) - 1:
        L.append("        %s |" % pat)
    else:
        L.append("        %s" % pat)
L.append("    )")
L.append("}")
L.append("")

# list_all_functions from DOC (161 TA-Lib names)
L.append("/// 列出 TA-Lib 0.7.1 提供的全部 161 个函数（名称, 分组），用于文档生成与自检。")
L.append("pub fn list_all_functions() -> Vec<(String, String)> {")
L.append("    vec![")
for ta_name, d in sorted(DOC_DATA.items()):
    g = d["group"]
    L.append('        (%s.to_string(), %s.to_string()),' % (rust_str(ta_name), rust_str(g)))
L.append("    ]")
L.append("}")
L.append("")

# ta_meta from DOC
L.append("/// 取得某函数的完整元信息（可选参数含默认值/范围、输出字段名与类型）。")
L.append("pub fn ta_meta(name: &str) -> Option<TaFuncMeta> {")
L.append("    match name {")
for ta_name, d in sorted(DOC_DATA.items()):
    params = d["params"]
    outputs = d["outputs"]
    L.append('        %s => Some(TaFuncMeta {' % rust_str(ta_name))
    L.append('            name: %s.to_string(),' % rust_str(ta_name))
    L.append('            group: %s.to_string(),' % rust_str(d["group"]))
    L.append('            hint: String::new(),')
    L.append("            opt_inputs: vec![")
    for (pn, pt, pd, prng) in params:
        lo = "None"
        hi = "None"
        rm = re.match(r"([0-9.eE+-]+)\.\.([0-9.eE+-]+)", prng)
        if rm:
            lo = "Some(%s)" % repr(float(rm.group(1)))
            hi = "Some(%s)" % repr(float(rm.group(2)))
        kind = "int" if pt == "int" else ("real" if pt == "real" else pt)
        defv = pd
        try:
            defv = repr(float(pd))
        except ValueError:
            defv = "0.0"
        L.append('                TaOptInput { name: %s.to_string(), display: String::new(), kind: %s.to_string(), default: %s, min: %s, max: %s },' % (rust_str(pn), rust_str(kind), defv, lo, hi))
    L.append("            ],")
    L.append("            outputs: vec![")
    for (on, ot) in outputs:
        L.append('                (%s.to_string(), %s.to_string()),' % (rust_str(on), rust_str(ot)))
    L.append("            ],")
    L.append("        }),")
L.append("        _ => None,")
L.append("    }")
L.append("}")
L.append("")

# ta_output_names
L.append("/// 取得某函数输出字段名（用于 `.field` 选择）。")
L.append("pub fn ta_output_names(name: &str) -> Option<Vec<String>> {")
L.append("    match name {")
for ta_name, sb in STRUCT_BRIDGE.items():
    fields = sb[1]
    L.append('        %s => Some(vec![%s]),' % (rust_str(ta_name), ", ".join(rust_str(f) + ".to_string()" for f in fields)))
L.append('        _ => Some(vec!["outReal".to_string()]),')
L.append("    }")
L.append("}")
L.append("")

open(OUT, "w", encoding="utf-8").write("\n".join(L))
print("wrote", OUT, "with", len(emitted), "TA functions (filtered from", len(sigs), "adaq pub fns)")
print("doc entries:", len(DOC_DATA))
miss = [t for t in DOC_DATA if t not in emitted]
if miss:
    print("WARN doc TA names with NO adaq impl:", miss)
else:
    print("OK: all 161 doc TA names have an adaq-talib implementation")
