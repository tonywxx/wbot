//! 单元/集成测试：指标数学、DSL 解析与求值、模拟交易。
//! 仅依赖合成数据，无需联网。

#[cfg(test)]
mod suite {
    use std::collections::HashMap;

    use chrono::NaiveDateTime;

    use crate::indicators::{Candle, IndicatorId, IndicatorRegistry, PriceSource};
    use crate::market::{Market, MarketData, MarketRouter, MarketSource, Quote, SourceError};
    use crate::signals::dsl::{parse_scope, parse_signal};
    use crate::signals::{CmpOp, Operand, PatternSpec, Scope, Side, SignalEngine, SignalNode, StrategyRule};
    use crate::sim::account::{Account, Order};
    use crate::app::{App, View};
    use crate::config::AppConfig;
    use crate::sim::crypto_ledger::CryptoLedger;

    use crate::crypto_gateway::trade_crypto;
    use crate::series::{min_len_for, select_rule_series};
    use crate::signals::eval::SignalEvent;

    fn candle(c: f64) -> Candle {
        Candle {
            date: NaiveDateTime::parse_from_str("2024-01-02 09:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: c,
            high: c,
            low: c,
            close: c,
            volume: 0.0,
        }
    }
    fn series(v: &[f64]) -> Vec<Candle> {
        v.iter().map(|&c| candle(c)).collect()
    }
    fn id(kind: &str, params: &[f64], field: Option<&str>) -> IndicatorId {
        IndicatorId {
            kind: kind.to_string(),
            source: PriceSource::Close,
            params: params.to_vec(),
            field: field.map(|s| s.to_string()),
        }
    }

    // ---------------- 指标 ----------------

    #[test]
    fn sma_basic() {
        let s = series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let reg = IndicatorRegistry::new();
        let out = reg.eval(&id("MA", &[3.0], None), &s).unwrap();
        assert!(out[1].is_nan());
        assert!((out[2] - 2.0).abs() < 1e-9);
        assert!((out[4] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn ema_seeded() {
        let s = series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let reg = IndicatorRegistry::new();
        let out = reg.eval(&id("EMA", &[3.0], None), &s).unwrap();
        assert!(out[1].is_nan());
        assert!(out[2].is_finite());
    }

    #[test]
    fn rsi_bounds() {
        let v: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let s = series(&v);
        let reg = IndicatorRegistry::new();
        let out = reg.eval(&id("RSI", &[14.0], None), &s).unwrap();
        let last = *out.last().unwrap();
        assert!(last.is_finite());
        assert!(last > 0.0 && last <= 100.0);
    }

    #[test]
    fn macd_hist_positive_in_uptrend() {
        // 加速上涨（二次曲线）才能让 DIF 持续高于 DEA -> HIST>0
        let v: Vec<f64> = (0..40).map(|i| 100.0 + (i as f64) * (i as f64)).collect();
        let s = series(&v);
        let reg = IndicatorRegistry::new();
        let out = reg
            .eval(&id("MACD", &[12.0, 26.0, 9.0], Some("hist")), &s)
            .unwrap();
        let last = *out.last().unwrap();
        assert!(last.is_finite() && last > 0.0);
    }

    #[test]
    fn boll_width_positive() {
        let s = series(&[10.0, 11.0, 9.0, 12.0, 8.0, 13.0, 7.0, 14.0, 6.0, 15.0, 5.0, 16.0]);
        let reg = IndicatorRegistry::new();
        let mid = reg.eval(&id("BOLL", &[5.0, 2.0], Some("mid")), &s).unwrap();
        let up = reg.eval(&id("BOLL", &[5.0, 2.0], Some("upper")), &s).unwrap();
        let lo = reg.eval(&id("BOLL", &[5.0, 2.0], Some("lower")), &s).unwrap();
        let i = s.len() - 1;
        assert!(up[i] > mid[i] && mid[i] > lo[i]);
    }

    // ---------------- DSL ----------------

    #[test]
    fn dsl_parse_ok() {
        parse_signal("cross_above(MA(close,5), MA(close,10))").unwrap();
        parse_signal("lt(RSI(close,14), 30)").unwrap();
        parse_signal(
            "and(gt(RSI(close,14),30), cross_below(MA(close,5), MA(close,10)))",
        )
        .unwrap();
        parse_signal("not(eq(PRICE(close), 0))").unwrap();
        parse_signal("gt(MACD(close,12,26,9).dif, MACD(close,12,26,9).dea)").unwrap();
    }

    #[test]
    fn dsl_parse_err() {
        assert!(parse_signal("and(, )").is_err());
        assert!(parse_signal("gt(MA(close,5))").is_err());
        assert!(parse_signal("foobar(1,2)").is_err());
        assert!(parse_signal("lt(RSI(close),)").is_err());
    }

    #[test]
    fn dsl_scope() {
        assert!(matches!(parse_scope("watchlist"), Scope::Watchlist));
        if let Scope::Codes(cs) = parse_scope("600519, 000858") {
            assert_eq!(cs.len(), 2);
            assert_eq!(cs[0], "600519");
        } else {
            panic!("expected Codes");
        }
    }

    // ---------------- 信号引擎 ----------------

    fn rule(sig: &str) -> StrategyRule {
        StrategyRule {
            id: "t".into(),
            label: "t".into(),
            side: Side::Buy,
            scope: parse_scope("watchlist"),
            enabled: true,
            signal: parse_signal(sig).unwrap(),
            timeframe: None,
            bars: None,
            note: String::new(),
            signal_text: sig.to_string(),
        }
    }

    #[test]
    fn engine_edge_trigger_once() {
        let s = series(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let mut kl = HashMap::new();
        kl.insert("600000".to_string(), s);
        let mut pr = HashMap::new();
        pr.insert("600000".to_string(), 8.0);
        let mut eng = SignalEngine::new(vec![rule("gt(MA(close,5), 0)")]);
        let reg = IndicatorRegistry::new();
        assert_eq!(eng.evaluate(&reg, &kl, &pr, &HashMap::new()).len(), 1);
        // 沿触发：再次求值不应重复触发
        assert_eq!(eng.evaluate(&reg, &kl, &pr, &HashMap::new()).len(), 0);
    }

    #[test]
    fn engine_cross_at_last_bar_fires() {
        // MA3 恰好在最后一根上穿 MA5
        let s = series(&[10.0, 10.0, 10.0, 10.0, 10.0, 20.0]);
        let mut kl = HashMap::new();
        kl.insert("X".to_string(), s);
        let mut pr = HashMap::new();
        pr.insert("X".to_string(), 20.0);
        let mut eng = SignalEngine::new(vec![rule("cross_above(MA(close,3), MA(close,5))")]);
        let reg = IndicatorRegistry::new();
        let ev = eng.evaluate(&reg, &kl, &pr, &HashMap::new());
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].code, "X");
        assert_eq!(eng.evaluate(&reg, &kl, &pr, &HashMap::new()).len(), 0);
    }

    #[test]
    fn engine_no_cross_when_flat() {
        let s = series(&[10.0; 8]);
        let mut kl = HashMap::new();
        kl.insert("X".to_string(), s);
        let mut pr = HashMap::new();
        pr.insert("X".to_string(), 10.0);
        let mut eng = SignalEngine::new(vec![rule("cross_above(MA(close,3), MA(close,5))")]);
        let reg = IndicatorRegistry::new();
        assert_eq!(eng.evaluate(&reg, &kl, &pr, &HashMap::new()).len(), 0);
    }

    #[test]
    fn engine_respects_scope_and_enabled() {
        let s = series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let mut kl = HashMap::new();
        kl.insert("600000".to_string(), s);
        let mut pr = HashMap::new();
        pr.insert("600000".to_string(), 5.0);
        // 仅作用于 Codes(["999999"])，不覆盖 600000
        let mut r = rule("gt(MA(close,5), 0)");
        r.scope = parse_scope("999999");
        let mut eng = SignalEngine::new(vec![r]);
        let reg = IndicatorRegistry::new();
        assert_eq!(eng.evaluate(&reg, &kl, &pr, &HashMap::new()).len(), 0);

        // 禁用规则不触发
        let mut r2 = rule("gt(MA(close,5), 0)");
        r2.enabled = false;
        let mut eng2 = SignalEngine::new(vec![r2]);
        assert_eq!(eng2.evaluate(&reg, &kl, &pr, &HashMap::new()).len(), 0);
    }

    // ---------------- 模拟交易 ----------------

    #[test]
    fn account_buy_then_sell_profit() {
        let mut a = Account::new(100, 0.0003, 0.0005);
        let b = Order {
            code: "600000".into(),
            side: Side::Buy,
            qty: 100,
            price: 10.0,
        };
        let f = a.place_order(&b).unwrap();
        assert_eq!(a.positions.get("600000").unwrap().qty, 100);
        assert_eq!(f.realized_pnl, 0.0);
        assert!(a.cash < 1_000_000.0);

        let s = Order {
            code: "600000".into(),
            side: Side::Sell,
            qty: 100,
            price: 11.0,
        };
        let f2 = a.place_order(&s).unwrap();
        assert!(f2.realized_pnl > 0.0);
        assert!(!a.positions.contains_key("600000"));
    }

    #[test]
    fn account_rejects() {
        let mut a = Account::new(100, 0.0003, 0.0005);
        // 现金不足
        let big = Order {
            code: "X".into(),
            side: Side::Buy,
            qty: 100,
            price: 1_000_000.0,
        };
        assert!(a.place_order(&big).is_err());
        // 无持仓卖出
        let sell = Order {
            code: "Y".into(),
            side: Side::Sell,
            qty: 100,
            price: 10.0,
        };
        assert!(a.place_order(&sell).is_err());
        // 整手向下取整：150 -> 100
        let odd = Order {
            code: "Z".into(),
            side: Side::Buy,
            qty: 150,
            price: 10.0,
        };
        a.place_order(&odd).unwrap();
        assert_eq!(a.positions.get("Z").unwrap().qty, 100);
    }

    // ---------------- 加密模拟账本（此前为零覆盖）----------------

    #[test]
    fn crypto_ledger_buy_then_sell_realizes_pnl() {
        let mut l = CryptoLedger::new(1000.0);
        let buy = l.place_order("BTC-USDT", true, 1.0, 100.0, 0.001).unwrap();
        assert!((buy.fee - 0.1).abs() < 1e-9); // 100 * 0.001
        assert!((l.usdt - 899.9).abs() < 1e-9);
        assert!((l.avg_cost.get("BTC-USDT").copied().unwrap() - 100.0).abs() < 1e-9);

        let sell = l.place_order("BTC-USDT", false, 1.0, 120.0, 0.001).unwrap();
        // 成交额 120，手续费 0.12，盈亏 (120-100)*1 - 0.12 = 19.88
        assert!((sell.realized_pnl - 19.88).abs() < 1e-9);
        assert!(!l.positions.contains_key("BTC-USDT"));
        assert!((l.usdt - (899.9 + 120.0 - 0.12)).abs() < 1e-9);
    }

    #[test]
    fn crypto_ledger_rejects_insufficient_usdt() {
        let mut l = CryptoLedger::new(10.0);
        assert!(l.place_order("BTC-USDT", true, 1.0, 100.0, 0.001).is_err());
    }

    #[test]
    fn crypto_ledger_rejects_oversell() {
        let mut l = CryptoLedger::new(1000.0);
        l.place_order("BTC-USDT", true, 1.0, 100.0, 0.001).unwrap();
        assert!(l.place_order("BTC-USDT", false, 2.0, 100.0, 0.001).is_err());
    }

    // ---------------- App：价格注入 seam（快照 / 报价共用）----------------

    #[test]
    fn app_apply_last_price_writes_prices_not_klines() {
        // 候选 2 修复后：apply_last_price 只写 app.prices，绝不改写 klines 末根
        // （改写会让实时价经 select_rule_series 借用渗进回测）。
        let mut klines = HashMap::new();
        klines.insert("600519".to_string(), series(&[10.0, 11.0, 12.0]));
        let mut app = App::new(
            vec!["600519".to_string()],
            5,
            klines,
            Account::new(100, 0.0003, 0.0005),
            Vec::new(),
            AppConfig::default(),
        );

        // 快照路径（A 股盘口 latest_price）写入 prices。
        app.apply_last_price("600519", 13.5);
        assert!((app.prices.get("600519").copied().unwrap() - 13.5).abs() < 1e-9);
        // klines 末根收盘保持不变（仍是 12.0）。
        assert!((app.klines.get("600519").unwrap().last().unwrap().close - 12.0).abs() < 1e-9);

        // 报价路径（美股 / 加密 latest_price）走同一 seam，覆盖同一 prices 键。
        app.apply_last_price("600519", 99.0);
        assert!((app.prices.get("600519").copied().unwrap() - 99.0).abs() < 1e-9);
        assert!((app.klines.get("600519").unwrap().last().unwrap().close - 12.0).abs() < 1e-9);

        // 未知代码也写入 prices，不 panic；klines 不受影响。
        app.apply_last_price("NOPE", 1.0);
        assert!((app.prices.get("NOPE").copied().unwrap() - 1.0).abs() < 1e-9);
        assert!(!app.klines.contains_key("NOPE"));
    }

    #[test]
    fn recompute_backtests_ignores_live_price() {
        // 候选 2（P0）：实时价注入后，recompute_backtests 的结果必须与注入前完全一致——
        // 证明实时价没有渗进回测末根。
        let mut klines = HashMap::new();
        klines.insert("600519".to_string(), series(&[10.0, 11.0, 12.0]));
        // 规则 close > 50：末根真实收盘 12 不触发；若 klines 被改写到 999 则会触发。
        let rule = StrategyRule {
            id: "r1".to_string(),
            label: "close>50".to_string(),
            side: Side::Buy,
            scope: Scope::Watchlist,
            enabled: true,
            signal: SignalNode::Cmp {
                op: CmpOp::Gt,
                left: Operand::Price(PriceSource::Close),
                right: Operand::Number(50.0),
            },
            timeframe: None,
            bars: None,
            note: String::new(),
            signal_text: "close>50".to_string(),
        };
        let mut app = App::new(
            vec!["600519".to_string()],
            5,
            klines,
            Account::new(100, 0.0003, 0.0005),
            vec![rule],
            AppConfig::default(),
        );

        app.recompute_backtests();
        let before = app.backtests.clone();

        // 模拟实时价注入（999 远高于末根真实收盘 12）。
        app.apply_last_price("600519", 999.0);

        // 根因断言：klines 末根收盘未被改写。
        assert!((app.klines.get("600519").unwrap().last().unwrap().close - 12.0).abs() < 1e-9);

        app.recompute_backtests();
        let after = app.backtests.clone();

        // 行为断言：回测结果与注入前完全一致（实时价未泄漏）。
        assert_eq!(before.len(), after.len());
        for (id, r) in &before {
            let a = after.get(id).expect("rule missing after re-run");
            assert_eq!(r.trades, a.trades, "rule {id} trade count changed after live-price injection");
        }
    }

    #[test]
    fn crypto_gateway_buy_updates_sim_ledger_without_live_call() {
        // 默认 live_trading=false 且未配置凭证 -> OKX 不会被触碰，模拟账本始终更新。
        let mut app = App::new(
            vec!["BTC-USDT".to_string()],
            5,
            HashMap::new(),
            Account::new(100, 0.0003, 0.0005),
            Vec::new(),
            AppConfig::default(),
        );
        let usdt_before = app.crypto.usdt;
        let msg = trade_crypto(&mut app, "BTC-USDT", 50_000.0).unwrap();
        // 买入 notional(1000) / price(50000) = 0.02 基础币（数量数学此前未测）。
        let pos = app.crypto.positions.get("BTC-USDT").copied().unwrap();
        assert!((pos - 0.02).abs() < 1e-9, "base_qty 应为 notional/price");
        // USDT 被扣减（含手续费），且扣减仅来自模拟成交 -> 证明未走 OKX 分支。
        assert!(app.crypto.usdt < usdt_before);
        assert!(msg.contains("BTC-USDT"));
    }

    #[test]
    fn crypto_gateway_sell_without_position_rejected() {
        // 信号视图下卖出无持仓标的 -> 网关应在模拟账本更新前拒绝（此前躺在 bin 中未测）。
        let mut app = App::new(
            vec!["ETH-USDT".to_string()],
            5,
            HashMap::new(),
            Account::new(100, 0.0003, 0.0005),
            Vec::new(),
            AppConfig::default(),
        );
        app.active_view = View::Signals;
        app.signals.push(SignalEvent {
            ts: chrono::Local::now(),
            code: "ETH-USDT".to_string(),
            side: Side::Sell,
            rule_id: "r".to_string(),
            label: "sell".to_string(),
            signal_text: "cross_below(MA(close,5), MA(close,10))".to_string(),
            note: String::new(),
            timeframe: None,
        });
        assert!(trade_crypto(&mut app, "ETH-USDT", 100.0).is_err());
        assert!(!app.crypto.positions.contains_key("ETH-USDT"));
    }

    #[test]
    fn shipped_strategy_toml_parses() {
        // 校验随包发布的 strategy.toml 可被正确解析且每条 rule 的信号可构造。
        // 总数 = 5 基础 DSL + 4 形态 + 20 经典 + 10 T+0 + 4 TA-Lib 教学实例 = 43；
        // 若某条 DSL 解析失败会被静默跳过导致数量不足，这里用精确计数兜底。
        let rules = crate::signals::parse_strategy_file("strategy.toml");
        assert_eq!(rules.len(), 43, "strategy.toml 规则数量应为 43");
        assert!(rules.iter().any(|r| r.id == "dg_buy_15"), "缺少 dg_buy_15 形态规则");
        assert!(rules.iter().any(|r| r.id == "dg_sell_15"), "缺少 dg_sell_15 形态规则");
        assert!(rules.iter().any(|r| r.id == "s01_ma_bull_arr"), "缺少经典策略 s01");
        assert!(rules.iter().any(|r| r.id == "t0_buy_rsi"), "缺少 T+0 策略 t0_buy_rsi");
        for r in &rules {
            assert!(r.enabled, "规则 {} 应启用", r.id);
        }
        let _eng = SignalEngine::new(rules);
    }

    // ---------------- 双金叉/双死叉形态检测 ----------------

    /// 生成「双金叉回踩」合成序列（bullish 用）。
    /// ① 下跌筑底（前低≈17.6，末段 ef<es）
    /// ② 上涨（金叉 #1：ef 上穿 es）
    /// ③ 回踩下跌（死叉 mid；更高低 18.1 > 前低 17.6）
    /// ④ 缓跌平台（ef 仍 < es，无新交叉），末根单根急拉（金叉 #2 在末根，close>EMA10）
    /// `break_base=true` 时把回踩最低价砸到 15.0，破坏前低 -> 应判否。
    fn gen_double_golden(break_base: bool) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut close = Vec::new();
        let mut high = Vec::new();
        let mut low = Vec::new();
        let push = |c: f64, v: &mut Vec<f64>, h: &mut Vec<f64>, l: &mut Vec<f64>| {
            v.push(c);
            h.push(c + 0.1);
            l.push(c - 0.1);
        };
        // ① 下跌筑底 20 根 20.0 -> 17.7
        for i in 0..20 {
            let c = 20.0 - i as f64 * (2.3 / 19.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ② 上涨 14 根 17.7 -> 20.5（金叉 #1）
        for i in 1..=14 {
            let c = 17.7 + i as f64 * (2.8 / 14.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ③ 回踩 14 根 20.5 -> 18.2（死叉 mid）
        for i in 1..=14 {
            let c = 20.5 - i as f64 * (2.3 / 14.0);
            push(c, &mut close, &mut high, &mut low);
        }
        if break_base {
            let last = low.len() - 1;
            low[last] = 15.0;
            close[last] = 15.5;
            high[last] = 16.0;
        }
        // ④ 缓跌 10 根 18.2 -> 17.9（ef 仍 < es，无新交叉）
        for i in 1..=10 {
            let c = 18.2 - i as f64 * (0.3 / 10.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ④b 末根单根急拉 21.5（金叉 #2 在末根，close>EMA10）
        push(21.5, &mut close, &mut high, &mut low);
        (close, high, low)
    }

    /// 生成「双死叉反抽」合成序列（bearish 用）：镜像。
    /// ① 上涨筑顶（前高≈20.1，末段 ef>es）② 下跌（死叉 #1）③ 反抽（金叉 mid；不过前高）
    /// ④ 缓涨平台（ef 仍 > es），末根单根急跌（死叉 #2 在末根，close<EMA10）
    fn gen_double_dead() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut close = Vec::new();
        let mut high = Vec::new();
        let mut low = Vec::new();
        let push = |c: f64, v: &mut Vec<f64>, h: &mut Vec<f64>, l: &mut Vec<f64>| {
            v.push(c);
            h.push(c + 0.1);
            l.push(c - 0.1);
        };
        // ① 上涨筑顶 20 根 17.7 -> 20.0
        for i in 0..20 {
            let c = 17.7 + i as f64 * (2.3 / 19.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ② 下跌 14 根 20.0 -> 17.7（死叉 #1）
        for i in 1..=14 {
            let c = 20.0 - i as f64 * (2.3 / 14.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ③ 反抽 14 根 17.7 -> 19.6（金叉 mid；不过前高 20.1）
        for i in 1..=14 {
            let c = 17.7 + i as f64 * (1.9 / 14.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ④ 缓涨 10 根 19.6 -> 19.9（ef 仍 > es，无新交叉）
        for i in 1..=10 {
            let c = 19.6 + i as f64 * (0.3 / 10.0);
            push(c, &mut close, &mut high, &mut low);
        }
        // ④b 末根单根急跌 16.0（死叉 #2 在末根，close<EMA10）
        push(16.0, &mut close, &mut high, &mut low);
        (close, high, low)
    }

    #[test]
    fn double_golden_fires_on_higher_low() {
        let (c, h, l) = gen_double_golden(false);
        assert!(
            crate::signals::double_cross::detect_double_golden_cross(&c, &h, &l, 5, 10, true, true),
            "更高低双金叉应触发"
        );
    }

    #[test]
    fn double_golden_rejects_broken_base() {
        let (c, h, l) = gen_double_golden(true);
        assert!(
            !crate::signals::double_cross::detect_double_golden_cross(&c, &h, &l, 5, 10, true, true),
            "回踩破前低应判否"
        );
    }

    #[test]
    fn double_dead_fires_on_lower_high() {
        let (c, h, l) = gen_double_dead();
        assert!(
            crate::signals::double_cross::detect_double_golden_cross(&c, &h, &l, 5, 10, true, false),
            "更低高双死叉应触发"
        );
    }

    // ---------------- 回测引擎 ----------------

    fn to_candles(c: &[f64], h: &[f64], l: &[f64]) -> Vec<Candle> {
        let dt = NaiveDateTime::parse_from_str("2024-01-02 09:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        c.iter()
            .zip(h.iter())
            .zip(l.iter())
            .map(|((&cl, &hi), &lo)| Candle {
                date: dt,
                open: cl,
                high: hi,
                low: lo,
                close: cl,
                volume: 0.0,
            })
            .collect()
    }

    #[test]
    fn backtest_double_golden_wins_on_synthetic() {
        // 双金叉序列末根急拉触发，但需追加上涨尾部以留出前向窗口并保证盈利。
        let (mut c, mut h, mut l) = gen_double_golden(false);
        let last = *c.last().unwrap();
        for i in 1..=12 {
            let v = last + i as f64 * (1.5 / 12.0);
            c.push(v);
            h.push(v + 0.1);
            l.push(v - 0.1);
        }
        let series = to_candles(&c, &h, &l);
        let r = StrategyRule {
            id: "dg".into(),
            label: "dg".into(),
            side: Side::Buy,
            scope: parse_scope("watchlist"),
            enabled: true,
            signal: SignalNode::Pattern(PatternSpec {
                name: "double_golden".into(),
                fast: 5,
                slow: 10,
                higher_low: true,
            }),
            timeframe: Some("15".into()),
            bars: Some(100),
            note: String::new(),
            signal_text: String::new(),
        };
        let res = crate::backtest::backtest_rule(&r, &series, 0.0003, 5);
        assert!(res.trades >= 1, "应至少触发一次信号");
        assert!(res.win_rate > 0.0, "合成上涨序列应盈利，胜率>0");
    }

    #[test]
    fn backtest_never_fires_has_no_trades() {
        // 信号恒为假（MA 不可能 < 0），不应产生任何交易。
        let s = series(&[10.0; 60]);
        let r = rule("lt(MA(close,5), 0)");
        let res = crate::backtest::backtest_rule(&r, &s, 0.0003, 5);
        assert_eq!(res.trades, 0);
        assert_eq!(res.win_rate, 0.0);
    }

    // ---------------- select_rule_series（K 线周期选取，统一真相源）----------------

    #[test]
    fn select_series_picks_daily_when_no_timeframe() {
        // 非形态日线规则：无 timeframe -> 走日线表，门槛 3。
        let mut klines = HashMap::new();
        klines.insert("600519".to_string(), series(&[1.0, 2.0, 3.0]));
        let intraday = HashMap::new();
        let r = rule("lt(MA(close,5), 0)");
        let plan = select_rule_series(&r, "600519", &klines, &intraday).unwrap();
        assert_eq!(plan.series.len(), 3);
        assert_eq!(plan.hold, 10); // 日线持仓 10
    }

    #[test]
    fn select_series_picks_intraday_by_composite_key() {
        // 带 timeframe="15" 的非形态规则 -> 走分钟线，复合键 `{code}@15`，门槛 3。
        let mut intraday = HashMap::new();
        intraday.insert("AAPL@15".to_string(), series(&[1.0, 2.0, 3.0, 4.0]));
        let klines = HashMap::new();
        let mut r = rule("lt(MA(close,5), 0)");
        r.timeframe = Some("15".into());
        let plan = select_rule_series(&r, "AAPL", &klines, &intraday).unwrap();
        assert_eq!(plan.series.len(), 4);
        assert_eq!(plan.hold, 5); // 分钟持仓 5
    }

    #[test]
    fn select_series_rejects_too_short() {
        // 非形态门槛为 3 根；序列不足 3 根 -> None。
        let mut klines = HashMap::new();
        klines.insert("600519".to_string(), series(&[1.0, 2.0]));
        let intraday = HashMap::new();
        let r = rule("lt(MA(close,5), 0)");
        assert!(select_rule_series(&r, "600519", &klines, &intraday).is_none());
    }

    #[test]
    fn select_series_missing_code_is_none() {
        let klines = HashMap::new();
        let intraday = HashMap::new();
        let r = rule("lt(MA(close,5), 0)");
        assert!(select_rule_series(&r, "NOPE", &klines, &intraday).is_none());
        let mut r2 = rule("lt(MA(close,5), 0)");
        r2.timeframe = Some("15".into());
        assert!(select_rule_series(&r2, "NOPE", &klines, &intraday).is_none());
    }

    #[test]
    fn select_series_pattern_uses_slow_plus_three() {
        // 形态规则：最小长度 = slow + 3，随模块统一；短于它的序列被拒。
        let mut intraday = HashMap::new();
        intraday.insert("AAPL@15".to_string(), series(&[1.0; 13]));
        let mut short = HashMap::new();
        short.insert("AAPL@15".to_string(), series(&[1.0; 12]));
        let mut r = rule("lt(MA(close,5), 0)");
        r.signal = SignalNode::Pattern(PatternSpec {
            name: "double_golden".into(),
            fast: 5,
            slow: 10,
            higher_low: true,
        });
        r.timeframe = Some("15".into());
        let empty: HashMap<String, Vec<Candle>> = HashMap::new();
        assert_eq!(min_len_for(&r), 13); // slow(10) + 3
        let plan = select_rule_series(&r, "AAPL", &empty, &intraday).unwrap();
        assert_eq!(plan.series.len(), 13);
        assert!(select_rule_series(&r, "AAPL", &empty, &short).is_none());
    }

    #[test]
    fn rule_series_consistent_across_sites() {
        // 同一规则 + K 线，实盘/回测/UI 三处经 select_rule_series 得到一致的门槛与持仓。
        // 这是候选 1 的核心不变量：门槛只定义在一个地方。
        let mut klines = HashMap::new();
        klines.insert("600519".to_string(), series(&[1.0; 20]));
        let intraday = HashMap::new();
        let r = rule("lt(MA(close,5), 0)");

        let plan = select_rule_series(&r, "600519", &klines, &intraday).unwrap();
        assert_eq!(plan.series.len(), 20);
        assert_eq!(plan.hold, 10);
        assert_eq!(min_len_for(&r), 3); // 非形态门槛固定为 3

        // 三处一致拒绝太短序列（门槛 3）。
        let mut short = HashMap::new();
        short.insert("600519".to_string(), series(&[1.0, 2.0]));
        assert!(select_rule_series(&r, "600519", &short, &intraday).is_none());
    }


    #[test]
    fn account_assets_and_pnl() {
        let mut a = Account::new(100, 0.0003, 0.0005);
        let b = Order {
            code: "600000".into(),
            side: Side::Buy,
            qty: 100,
            price: 10.0,
        };
        a.place_order(&b).unwrap();
        let mut prices = HashMap::new();
        prices.insert("600000".to_string(), 12.0);
        let total = a.total_assets(&prices);
        let pnl = a.unrealized_pnl(&prices);
        assert!((total - (a.cash + 1200.0)).abs() < 1e-6);
        assert!((pnl - 200.0).abs() < 1e-6);
    }

    // ---- MarketSource adapter / MarketRouter routing ----

    /// 注入式假数据源：把单一市场吐出的合成 `Candle` 序列返回，用于在不联网的情况下
    /// 验证 `MarketRouter` 按代码形态把请求派发给对应适配器。
    struct FakeSource {
        market: Market,
        out: Vec<Candle>,
    }

    impl MarketSource for FakeSource {
        fn market(&self) -> Market {
            self.market
        }
        fn fetch_klines<'a>(
            &'a self,
            _code: &'a str,
            _adjust: &'a str,
            _count: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
        {
            let out = self.out.clone();
            Box::pin(async move { Ok(out) })
        }
        fn fetch_intraday<'a>(
            &'a self,
            _code: &'a str,
            _tf: &'a str,
            _bars: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
        {
            let out = self.out.clone();
            Box::pin(async move { Ok(out) })
        }
        fn fetch_snapshot<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<MarketData>> + Send + 'a>>
        {
            Box::pin(async move { None })
        }
    }

    #[tokio::test]
    async fn market_router_routes_by_symbol_shape() {
        let router = MarketRouter::from_sources(
            Box::new(FakeSource {
                market: Market::A,
                out: vec![candle(1.0)],
            }),
            Box::new(FakeSource {
                market: Market::Us,
                out: vec![],
            }),
        );
        // 6 位纯数字 -> A 股适配器；字母代码 / 带点的代码 -> 美股适配器。
        assert_eq!(router.source_for("600519").market(), Market::A);
        assert_eq!(router.source_for("000001").market(), Market::A);
        assert_eq!(router.source_for("AAPL").market(), Market::Us);
        assert_eq!(router.source_for("BRK.B").market(), Market::Us);
    }

    #[tokio::test]
    async fn market_router_fetch_all_klines_dispatches() {
        let router = MarketRouter::from_sources(
            Box::new(FakeSource {
                market: Market::A,
                out: vec![candle(1.0)],
            }),
            Box::new(FakeSource {
                market: Market::Us,
                out: vec![],
            }),
        );
        let (map, errs) = router
            .fetch_all_klines(
                &["600519".to_string(), "AAPL".to_string()],
                "qfq",
                10,
            )
            .await;
        // A 股适配器返回了数据 -> 入表；美股适配器返回空 -> 不入表（非空过滤生效）。
        assert_eq!(map.get("600519").map(|v| v.len()), Some(1));
        assert!(!map.contains_key("AAPL"));
        assert!(errs.is_empty(), "派发测试中不应出现失败");
    }

    /// 假「必定失败」数据源：用于验证 `fetch_all_klines` 把失败代码与原因带回，
    /// 而非被 `if !s.is_empty()` 静默丢弃（候选 3 的核心回归守卫）。
    struct FakeFailingSource {
        market: Market,
    }

    impl MarketSource for FakeFailingSource {
        fn market(&self) -> Market {
            self.market
        }
        fn fetch_klines<'a>(
            &'a self,
            _code: &'a str,
            _adjust: &'a str,
            _count: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
        {
            Box::pin(async move { Err(SourceError::Network("boom".into())) })
        }
        fn fetch_intraday<'a>(
            &'a self,
            _code: &'a str,
            _tf: &'a str,
            _bars: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
        {
            Box::pin(async move { Err(SourceError::Network("boom".into())) })
        }
        fn fetch_snapshot<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<MarketData>> + Send + 'a>>
        {
            Box::pin(async move { None })
        }
    }

    #[tokio::test]
    async fn market_router_fetch_all_klines_surfaces_failures() {
        // A 股吐数据、美股必定失败：失败代码应进清单而非被静默丢弃。
        let router = MarketRouter::from_sources(
            Box::new(FakeSource {
                market: Market::A,
                out: vec![candle(1.0)],
            }),
            Box::new(FakeFailingSource { market: Market::Us }),
        );
        let (map, errs) = router
            .fetch_all_klines(&["600519".to_string(), "AAPL".to_string()], "qfq", 10)
            .await;
        // 成功部分照常入表。
        assert_eq!(map.get("600519").map(|v| v.len()), Some(1));
        assert!(!map.contains_key("AAPL"));
        // 失败代码与原因被带回（此前会被 `if !s.is_empty()` 直接丢弃）。
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].0, "AAPL");
        assert_eq!(errs[0].1, SourceError::Network("boom".into()));
    }

    /// 假「报价源」：按代码形态返回一条合成报价，用于验证
    /// `MarketRouter::fetch_all_quotes` 会按 A / 美股 / 加密货币分流并合并。
    struct FakeQuoteSource {
        market: Market,
    }

    impl MarketSource for FakeQuoteSource {
        fn market(&self) -> Market {
            self.market
        }
        fn fetch_klines<'a>(
            &'a self,
            _code: &'a str,
            _adjust: &'a str,
            _count: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
        {
            Box::pin(async move { Ok(Vec::new()) })
        }
        fn fetch_intraday<'a>(
            &'a self,
            _code: &'a str,
            _tf: &'a str,
            _bars: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Candle>, SourceError>> + Send + 'a>>
        {
            Box::pin(async move { Ok(Vec::new()) })
        }
        fn fetch_snapshot<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<MarketData>> + Send + 'a>>
        {
            Box::pin(async move { None })
        }
        fn fetch_quotes<'a>(
            &'a self,
            codes: &'a [String],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Quote>> + Send + 'a>> {
            let out: Vec<Quote> = codes
                .iter()
                .map(|c| Quote {
                    code: c.clone(),
                    name: format!("name-{}", c),
                    latest_price: 9.0,
                    change_pct: 1.5,
                    market: self.market,
                })
                .collect();
            Box::pin(async move { out })
        }
    }

    #[tokio::test]
    async fn market_router_fetch_all_quotes_covers_all_markets() {
        // 注入式路由：A / 美股 / 加密货币各自吐出合成报价。
        let router = MarketRouter::from_sources_full(
            Box::new(FakeQuoteSource {
                market: Market::A,
            }),
            Box::new(FakeQuoteSource {
                market: Market::Us,
            }),
            Box::new(FakeQuoteSource {
                market: Market::Crypto,
            }),
        );
        let codes = [
            "600519".to_string(),
            "AAPL".to_string(),
            "BTC-USDT".to_string(),
        ];
        let quotes = router.fetch_all_quotes(&codes).await;
        assert_eq!(quotes.len(), 3, "三类资产的报价都应被拉取");
        // 验证分流正确：每个代码被对应市场适配器处理。
        let by_code: std::collections::HashMap<String, Market> =
            quotes.iter().map(|q| (q.code.clone(), q.market)).collect();
        assert_eq!(by_code.get("600519"), Some(&Market::A));
        assert_eq!(by_code.get("AAPL"), Some(&Market::Us));
        assert_eq!(by_code.get("BTC-USDT"), Some(&Market::Crypto));
    }

    #[tokio::test]
    async fn market_router_fetch_all_quotes_empty_when_no_codes() {
        let router = MarketRouter::from_sources(
            Box::new(FakeQuoteSource {
                market: Market::A,
            }),
            Box::new(FakeQuoteSource {
                market: Market::Us,
            }),
        );
        assert!(router.fetch_all_quotes(&[]).await.is_empty());
    }

    // ---------------- TA-Lib 集成 ----------------

    #[test]
    fn ta_rsi_lookback_and_finite() {
        // 单调递增序列 -> RSI(14) 接近 100；前 13 根为 NaN。
        let data: Vec<f64> = (1..=40).map(|x| x as f64).collect();
        let s = series(&data);
        let reg = IndicatorRegistry::new();
        let out = reg.eval(&id("TA_RSI", &[14.0], None), &s).unwrap();
        assert!(out[12].is_nan(), "RSI(14) 前 13 根应为 NaN");
        assert!(out[39].is_finite());
        assert!(out[39] > 90.0, "单调递增序列 RSI 应接近 100, got {}", out[39]);
    }

    #[test]
    fn ta_bbands_middle_equals_sma() {
        // TA-Lib BBANDS 中轨 = SMA(close, period)，与自研 MA 应完全一致。
        let mut rng = 0.0f64;
        let data: Vec<f64> = (0..60)
            .map(|_| {
                rng += 1.3;
                rng.sin() * 10.0 + 50.0
            })
            .collect();
        let s = series(&data);
        let reg = IndicatorRegistry::new();
        let bb = reg
            .eval(&id("TA_BBANDS", &[20.0, 2.0], Some("1")), &s)
            .unwrap();
        let sma = reg.eval(&id("MA", &[20.0], None), &s).unwrap();
        for i in 20..60 {
            assert!(
                (bb[i] - sma[i]).abs() < 1e-6,
                "BBANDS 中轨应与 SMA 相等 @{}: {} vs {}",
                i,
                bb[i],
                sma[i]
            );
        }
    }

    #[test]
    fn ta_invalid_name_returns_none() {
        let s = series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let reg = IndicatorRegistry::new();
        assert!(reg.eval(&id("TA_NOTAREALFUNC", &[], None), &s).is_none());
    }

    #[test]
    fn ta_field_selection_multi_output() {
        // MACD 三输出：0=macd,1=signal,2=hist。验证 field 选择生效且各不相同。
        let data: Vec<f64> = (0..60)
            .map(|x| 50.0 + (x as f64 * 0.7).sin() * 5.0)
            .collect();
        let s = series(&data);
        let reg = IndicatorRegistry::new();
        let macd = reg
            .eval(&id("TA_MACD", &[12.0, 26.0, 9.0], Some("0")), &s)
            .unwrap();
        let hist = reg
            .eval(&id("TA_MACD", &[12.0, 26.0, 9.0], Some("2")), &s)
            .unwrap();
        let n = s.len();
        let any_finite = (0..n).any(|i| macd[i].is_finite() && hist[i].is_finite());
        assert!(any_finite, "MACD 应输出有效值");
        let differ = (0..n).any(|i| (macd[i] - hist[i]).abs() > 1e-9);
        assert!(differ, "MACD 主值与柱状值应不同（field 选择生效）");
    }

    #[test]
    fn ta_adx_finite() {
        let s: Vec<Candle> = (0..45)
            .map(|i| {
                let c = 10.0 + (i as f64 * 0.3).sin() * 2.0 + i as f64 * 0.05;
                Candle {
                    date: NaiveDateTime::parse_from_str(
                        "2024-01-02 09:30:00",
                        "%Y-%m-%d %H:%M:%S",
                    )
                    .unwrap(),
                    open: c,
                    high: c + 0.6,
                    low: c - 0.6,
                    close: c,
                    volume: 1000.0,
                }
            })
            .collect();
        let reg = IndicatorRegistry::new();
        let out = reg.eval(&id("TA_ADX", &[14.0], None), &s).unwrap();
        assert!(out.iter().any(|v| v.is_finite()), "ADX 应有有限输出");
    }

    #[test]
    fn ta_dsl_parse_ta_indicator() {
        // 解析含 TA_ 指标的 DSL，确认 kind 与参数、field 正确解析。
        use crate::signals::dsl::parse_signal;
        use crate::signals::{Operand, SignalNode};

        let node = parse_signal("cross_above(TA_RSI(close,14), 30)").unwrap();
        match node {
            SignalNode::Cross { left, .. } => match left {
                Operand::Indicator(id) => {
                    assert_eq!(id.kind, "TA_RSI");
                    assert_eq!(id.params, vec![14.0]);
                }
                _ => panic!("cross 左操作数应为指标"),
            },
            _ => panic!("应为 Cross 节点"),
        }

        // 多输出 field 选择（MACD.hist）
        let node2 = parse_signal("cross_above(TA_MACD(close,12,26,9).hist, 0)").unwrap();
        if let SignalNode::Cross { left, .. } = node2 {
            if let Operand::Indicator(id) = left {
                assert_eq!(id.kind, "TA_MACD");
                assert_eq!(id.field.as_deref(), Some("hist"));
            } else {
                panic!("应为指标操作数");
            }
        } else {
            panic!("应为 Cross 节点");
        }
    }

    #[test]
    fn ta_dsl_strategy_file_parses() {
        // strategy.toml 中的 TA 教学规则应能被解析（解析失败会被跳过，这里确认存在）。
        use crate::signals::parse_strategy_file;
        let rules = parse_strategy_file("strategy.toml");
        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        for want in ["ta_ex1_buy", "ta_ex1_sell", "ta_ex2_buy", "ta_ex2_sell"] {
            assert!(ids.contains(&want), "strategy.toml 缺少 TA 示例规则: {want}");
        }
        // 确认它们通过 DSL 解析（signal_text 非空即代表 signal 已成功解析为节点）。
        for r in &rules {
            if r.id.starts_with("ta_ex") {
                assert!(!r.signal_text.is_empty(), "{} 的 signal 未解析", r.id);
            }
        }
    }
}
