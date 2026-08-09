//! 信号条件模型 + 递归 DSL。
//!
//! 用户用 `strategy.toml` 描述「条件组合」触发买卖信号；`dsl` 模块把 TOML 文本
//! 解析为递归的 `SignalNode` 表达式树；`eval` 模块对每只标的求值并做「沿触发」
//! 去抖（false->true 才发事件），避免连续重复触发。

pub mod dsl;
pub mod eval;
pub mod double_cross;

pub use eval::{SignalEngine, SignalEvent};

use crate::indicators::{IndicatorId, PriceSource};
use serde::{Deserialize, Serialize};

/// 买卖方向。序列化用小写（"buy"/"sell"），便于 TOML/JSON 读写。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

/// 作用范围。
#[derive(Debug, Clone)]
pub enum Scope {
    Watchlist,
    Codes(Vec<String>),
}

/// 操作数：指标序列 / 数字 / 价格序列。
#[derive(Debug, Clone)]
pub enum Operand {
    Indicator(IndicatorId),
    Number(f64),
    Price(PriceSource),
}

/// 比较算子。
#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
}

/// 交叉方向。
#[derive(Debug, Clone, Copy)]
pub enum CrossDir {
    Above,
    Below,
}

/// 递归信号表达式树。
#[derive(Debug, Clone)]
pub enum SignalNode {
    And(Vec<SignalNode>),
    Or(Vec<SignalNode>),
    Not(Box<SignalNode>),
    Cmp { op: CmpOp, left: Operand, right: Operand },
    Cross { dir: CrossDir, left: Operand, right: Operand },
    /// 顺序状态机形态（双金叉/双死叉回踩等），由 `double_cross` 模块独立检测，
    /// 无法用点状 DSL 表达。
    Pattern(PatternSpec),
}

/// 形态检测参数（与 DSL 节点并列，由 `kind="pattern"` 规则驱动）。
#[derive(Debug, Clone)]
pub struct PatternSpec {
    /// 形态名（目前支持 "double_golden"）。
    #[allow(dead_code)]
    pub name: String,
    /// 快线周期（如 5）。
    pub fast: usize,
    /// 慢线周期（如 10）。
    pub slow: usize,
    /// 是否要求「回踩不破前低 / 反抽不过前高」。
    pub higher_low: bool,
}

/// TOML 中的原始规则（`signal` 为 DSL 文本字符串；`pattern` 形态规则不使用 `signal`）。
#[derive(Debug, Clone, Deserialize)]
pub struct RawRule {
    pub id: String,
    pub label: String,
    pub side: Side,
    pub scope: String,
    pub enabled: bool,
    #[serde(default)]
    pub signal: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    #[serde(default)]
    pub fast: Option<usize>,
    #[serde(default)]
    pub slow: Option<usize>,
    #[serde(default)]
    pub bars: Option<usize>,
    #[serde(default)]
    pub higher_low: Option<bool>,
    /// 策略备注 / 说明（用于选择界面展示，可选）。
    #[serde(default)]
    pub note: Option<String>,
}

/// 编译后的规则（`signal` 已解析为 `SignalNode`）。
#[derive(Debug, Clone)]
pub struct StrategyRule {
    pub id: String,
    pub label: String,
    pub side: Side,
    pub scope: Scope,
    pub enabled: bool,
    pub signal: SignalNode,
    /// 形态规则的周期（如 "15" / "60"），DSL 规则为 None。
    pub timeframe: Option<String>,
    /// 形态规则的回看根数，DSL 规则为 None。
    pub bars: Option<usize>,
    /// 策略备注 / 说明（展示用；选择界面读取）。
    #[allow(dead_code)]
    pub note: String,
}

/// 解析 `strategy.toml` 为编译后的规则列表。文件缺失/损坏返回空列表（不崩溃）。
pub fn parse_strategy_file(path: &str) -> Vec<StrategyRule> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return Vec::new(),
    };
    let doc: toml::Value = match toml::from_str(&text) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("策略文件解析失败 ({}): {}", path, e);
            return Vec::new();
        }
    };
    let raw: Vec<RawRule> = match doc.get("rules") {
        Some(v) => match v.clone().try_into() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("策略规则解析失败: {}", e);
                return Vec::new();
            }
        },
        None => return Vec::new(),
    };

    let mut out = Vec::with_capacity(raw.len());
    for r in raw {
        // 形态规则（kind="pattern" 或显式 pattern=）走独立检测器，不解析 DSL。
        let (node, timeframe, bars) = if r.pattern.is_some() || r.kind.as_deref() == Some("pattern") {
            let spec = PatternSpec {
                name: r.pattern.clone().unwrap_or_else(|| "double_golden".to_string()),
                fast: r.fast.unwrap_or(5),
                slow: r.slow.unwrap_or(10),
                higher_low: r.higher_low.unwrap_or(true),
            };
            (SignalNode::Pattern(spec), r.timeframe.clone(), r.bars)
        } else {
            let n = match dsl::parse_signal(&r.signal) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("规则 {} 信号解析失败: {}", r.id, e);
                    continue;
                }
            };
            (n, None, None)
        };
        let scope = dsl::parse_scope(&r.scope);
        out.push(StrategyRule {
            id: r.id,
            label: r.label,
            side: r.side,
            scope,
            enabled: r.enabled,
            signal: node,
            timeframe,
            bars,
            note: r.note.clone().unwrap_or_default(),
        });
    }
    out
}
