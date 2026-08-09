//! 策略 DSL 解析：TOML `signal` 字符串 -> `SignalNode` 表达式树。
//!
//! 语法（函数式，无中缀）：
//!   逻辑:  and(a, b, ...)  or(a, b, ...)  not(a)
//!   比较:  gt(a,b) lt(a,b) gte(a,b) lte(a,b) eq(a,b)
//!   交叉:  cross_above(a,b)  cross_below(a,b)
//!   操作数: 数字 | PRICE(src) | NAME(src, p1, p2, ...)[.field]
//!   指标名: MA|SMA|EMA|RSI|MACD|KDJ|BOLL（大小写不敏感）
//!   来源:   close|open|high|low|volume

use crate::indicators::{IndicatorId, PriceSource};
use crate::signals::{CmpOp, CrossDir, Operand, Scope, SignalNode};

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(f64),
    LP,
    RP,
    Comma,
    Dot,
    End,
}

fn tokenize(s: &str) -> Vec<Tok> {
    let chars: Vec<char> = s.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        match ch {
            '(' => {
                toks.push(Tok::LP);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RP);
                i += 1;
            }
            ',' => {
                toks.push(Tok::Comma);
                i += 1;
            }
            '.' => {
                toks.push(Tok::Dot);
                i += 1;
            }
            _ if ch.is_ascii_digit() || ch == '-' || ch == '+' => {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num: f64 = s[start..i].parse().unwrap_or(f64::NAN);
                toks.push(Tok::Num(num));
            }
            _ if ch.is_ascii_alphabetic() => {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                toks.push(Tok::Ident(s[start..i].to_string()));
            }
            _ => {
                i += 1;
            }
        }
    }
    toks.push(Tok::End);
    toks
}

enum Arg {
    Node(SignalNode),
    Operand(Operand),
}

/// 顶层解析入口。
pub fn parse_signal(s: &str) -> anyhow::Result<SignalNode> {
    let toks = tokenize(s);
    let mut p = Parser { toks, pos: 0 };
    let node = p.parse_node()?;
    if !matches!(p.peek(), Tok::End) {
        anyhow::bail!("表达式存在多余 token: {:?}", p.peek());
    }
    Ok(node)
}

/// 解析作用范围字符串。默认 watchlist。
pub fn parse_scope(s: &str) -> Scope {
    let t = s.trim();
    if t.eq_ignore_ascii_case("watchlist") {
        return Scope::Watchlist;
    }
    let codes: Vec<String> = t
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|c| !c.is_empty())
        .map(|c| c.to_string())
        .collect();
    if codes.is_empty() {
        Scope::Watchlist
    } else {
        Scope::Codes(codes)
    }
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        self.toks.get(self.pos).unwrap_or(&Tok::End)
    }

    fn next_tok(&mut self) -> Tok {
        let t = self.peek().clone();
        self.pos += 1;
        t
    }

    fn expect(&mut self, want: &Tok) -> anyhow::Result<()> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(want) {
            self.pos += 1;
            Ok(())
        } else {
            anyhow::bail!("期望 {:?}, 实际 {:?}", want, self.peek())
        }
    }

    fn parse_node(&mut self) -> anyhow::Result<SignalNode> {
        let name = match self.next_tok() {
            Tok::Ident(n) => n,
            other => anyhow::bail!("期望函数名, 实际 {:?}", other),
        };
        self.expect(&Tok::LP)?;
        let args = self.parse_args()?;
        self.expect(&Tok::RP)?;

        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "and" | "or" => {
                let nodes = args
                    .into_iter()
                    .map(|a| match a {
                        Arg::Node(n) => Ok(n),
                        Arg::Operand(_) => anyhow::bail!("{} 的参数须为条件表达式", lower),
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(if lower == "and" {
                    SignalNode::And(nodes)
                } else {
                    SignalNode::Or(nodes)
                })
            }
            "not" => {
                let n = match args.into_iter().next() {
                    Some(Arg::Node(n)) => n,
                    _ => anyhow::bail!("not 的参数须为条件表达式"),
                };
                Ok(SignalNode::Not(Box::new(n)))
            }
            "gt" | "lt" | "gte" | "lte" | "eq" => {
                let (l, r) = two_operands(args)?;
                let op = match lower.as_str() {
                    "gt" => CmpOp::Gt,
                    "lt" => CmpOp::Lt,
                    "gte" => CmpOp::Gte,
                    "lte" => CmpOp::Lte,
                    _ => CmpOp::Eq,
                };
                Ok(SignalNode::Cmp { op, left: l, right: r })
            }
            "cross_above" | "cross_below" => {
                let (l, r) = two_operands(args)?;
                let dir = if lower == "cross_above" {
                    CrossDir::Above
                } else {
                    CrossDir::Below
                };
                Ok(SignalNode::Cross {
                    dir,
                    left: l,
                    right: r,
                })
            }
            other => anyhow::bail!("未知函数: {}", other),
        }
    }

    fn parse_args(&mut self) -> anyhow::Result<Vec<Arg>> {
        let mut args = Vec::new();
        if matches!(self.peek(), Tok::RP) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_arg()?);
            match self.peek() {
                Tok::Comma => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Ok(args)
    }

    fn parse_arg(&mut self) -> anyhow::Result<Arg> {
        if let Tok::Num(_) = self.peek() {
            if let Tok::Num(n) = self.next_tok() {
                return Ok(Arg::Operand(Operand::Number(n)));
            }
        }
        if let Tok::Ident(_) = self.peek() {
            let name = match self.peek().clone() {
                Tok::Ident(n) => n,
                _ => unreachable!(),
            };
            let lower = name.to_ascii_lowercase();
            if matches!(
                lower.as_str(),
                "and" | "or" | "not" | "gt" | "lt" | "gte" | "lte" | "eq" | "cross_above"
                    | "cross_below"
            ) {
                let node = self.parse_node()?;
                return Ok(Arg::Node(node));
            }
            let operand = self.parse_operand()?;
            return Ok(Arg::Operand(operand));
        }
        anyhow::bail!("无法解析的参数: {:?}", self.peek())
    }

    fn parse_operand(&mut self) -> anyhow::Result<Operand> {
        let name = match self.next_tok() {
            Tok::Ident(n) => n,
            other => anyhow::bail!("期望指标名, 实际 {:?}", other),
        };
        self.expect(&Tok::LP)?;
        let src = self.parse_source()?;
        let mut params = Vec::new();
        while matches!(self.peek(), Tok::Comma) {
            self.pos += 1;
            params.push(self.parse_number()?);
        }
        self.expect(&Tok::RP)?;

        let mut field = None;
        if matches!(self.peek(), Tok::Dot) {
            self.pos += 1;
            if let Tok::Ident(f) = self.next_tok() {
                field = Some(f);
            }
        }

        let kind = name.to_ascii_uppercase();
        if kind == "PRICE" {
            return Ok(Operand::Price(src));
        }
        Ok(Operand::Indicator(IndicatorId {
            kind,
            source: src,
            params,
            field,
        }))
    }

    fn parse_source(&mut self) -> anyhow::Result<PriceSource> {
        let s = match self.next_tok() {
            Tok::Ident(n) => n,
            other => anyhow::bail!("期望来源(close/open/...), 实际 {:?}", other),
        };
        PriceSource::from_str(&s).ok_or_else(|| anyhow::anyhow!("未知来源: {}", s))
    }

    fn parse_number(&mut self) -> anyhow::Result<f64> {
        match self.next_tok() {
            Tok::Num(n) => {
                if n.is_nan() {
                    anyhow::bail!("非法数字");
                }
                Ok(n)
            }
            other => anyhow::bail!("期望数字, 实际 {:?}", other),
        }
    }
}

fn two_operands(args: Vec<Arg>) -> anyhow::Result<(Operand, Operand)> {
    if args.len() != 2 {
        anyhow::bail!("比较/交叉需要恰好 2 个操作数, 实际 {}", args.len());
    }
    let mut it = args.into_iter();
    let l = match it.next().unwrap() {
        Arg::Operand(o) => o,
        _ => anyhow::bail!("期望操作数"),
    };
    let r = match it.next().unwrap() {
        Arg::Operand(o) => o,
        _ => anyhow::bail!("期望操作数"),
    };
    Ok((l, r))
}
