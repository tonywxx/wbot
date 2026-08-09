//! 单元/集成测试：指标数学、DSL 解析与求值、模拟交易。
//! 仅依赖合成数据，无需联网。

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDateTime;

    use crate::indicators::{Candle, IndicatorId, IndicatorRegistry, PriceSource};
    use crate::market::{Market, MarketData, MarketRouter, MarketSource};
    use crate::signals::dsl::{parse_scope, parse_signal};
    use crate::signals::{PatternSpec, Scope, Side, SignalEngine, SignalNode, StrategyRule};
    use crate::sim::account::{Account, Order};

    use crate::backtest::select_series;

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
        assert!(a.positions.get("600000").is_none());
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

    #[test]
    fn shipped_strategy_toml_parses() {
        // 校验随包发布的 strategy.toml 可被正确解析且每条 rule 的信号可构造。
        // 总数 = 5 基础 DSL + 4 形态 + 20 经典 + 10 T+0 = 39；若某条 DSL 解析失败
        // 会被静默跳过导致数量不足，这里用精确计数兜底。
        let rules = crate::signals::parse_strategy_file("strategy.toml");
        assert_eq!(rules.len(), 39, "strategy.toml 规则数量应为 39");
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
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Candle>> + Send + 'a>> {
            let out = self.out.clone();
            Box::pin(async move { out })
        }
        fn fetch_intraday<'a>(
            &'a self,
            _code: &'a str,
            _tf: &'a str,
            _bars: usize,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Candle>> + Send + 'a>> {
            let out = self.out.clone();
            Box::pin(async move { out })
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
        let map = router
            .fetch_all_klines(
                &["600519".to_string(), "AAPL".to_string()],
                "qfq",
                10,
            )
            .await;
        // A 股适配器返回了数据 -> 入表；美股适配器返回空 -> 不入表（非空过滤生效）。
        assert_eq!(map.get("600519").map(|v| v.len()), Some(1));
        assert!(!map.contains_key("AAPL"));
    }
}
