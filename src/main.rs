//! A股模拟交易系统 TUI — 基于真实行情数据。
//!
//! 启动流程：加载配置/账户/策略 -> `block_on` 拉取历史 K 线初始化 -> 进入 UI 循环。
//! `data_loop` 每 5s 推送实时快照、每 ~60s 推送 K 线增量；`run_app` 收到快照后
//! 用最新价覆盖末根收盘、重算指标并求值信号，用户可在信号/账户视图按 Enter 下单。

// 模块由 src/lib.rs 声明并共享；二进制通过 `wbot::` 引用本库。
use std::collections::HashMap;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::interval;

use wbot::app::{App, Focus, View};
use wbot::config::load_config;
use wbot::crypto_gateway::trade_crypto;
use wbot::i18n::{order_failed, record_failed, traded_fee, tr, Lang};
use wbot::indicators::{Candle, IndicatorRegistry};
use wbot::market::{self, MarketRouter, load_watchlist_combined, market_of, Market};
use wbot::persist::{load_account, save_account};
use wbot::signals::{parse_strategy_file, Side, StrategyRule};
use wbot::sim::account::{fill_to_trade, Order};
use wbot::sim::history::{append_trade, load_trades};
use wbot::ui;

/// 从异步数据任务推送到 UI 循环的消息。
enum Msg {
    Snapshot(market::MarketData),
    /// 统一实时报价（A 股 / 美股 / 加密货币），刷新周期驱动 watchlist 的 name/price/change。
    Quotes(Vec<market::Quote>),
    Klines(HashMap<String, Vec<Candle>>),
    /// 分钟 K 线增量（键 `"{code}@{tf}"`）。
    Intraday(HashMap<String, Vec<Candle>>),
    Error(String),
}

/// UI 循环向数据任务发起的请求（强制刷新）。
enum Request {
    Refresh,
}

/// 异步数据任务：定时推送快照与 K 线增量。
async fn data_loop(
    router: MarketRouter,
    codes: Vec<String>,
    ui_tx: std::sync::mpsc::Sender<Msg>,
    mut req_rx: mpsc::Receiver<Request>,
    refresh: u64,
    kline_adjust: String,
    kline_count: usize,
    tf_bars: Vec<(String, usize)>,
    intraday_refresh: u64,
    lang: Lang,
) {
    // 首屏快照（A 股板块；美股 / 加密货币无对应全市场板块，单独走行情接口）
    if let Some(d) = router.fetch_snapshot().await {
        if !(d.indices.is_empty() && d.spots.is_empty()) {
            let _ = ui_tx.send(Msg::Snapshot(d));
        }
    }

    let mut tick = interval(Duration::from_secs(refresh.max(1)));
    let mut ktick = interval(Duration::from_secs(60));
    let mut itick = interval(Duration::from_secs(intraday_refresh.max(10)));
    loop {
        tokio::select! {
            _ = tick.tick() => {
                // 统一实时报价：覆盖 A 股 / 美股 / 加密货币三类资产的自选股。
                let quotes = router.fetch_all_quotes(&codes).await;
                let _ = ui_tx.send(Msg::Quotes(quotes));

                // A 股全市场盘口（指数 + 涨跌家数 + 涨幅榜），仅 A 股需要。
                match router.fetch_snapshot().await {
                    Some(d) if !(d.indices.is_empty() && d.spots.is_empty()) => {
                        let _ = ui_tx.send(Msg::Snapshot(d));
                    }
                    _ => {
                        // 盘口快照失败不应影响 watchlist 报价刷新；仅更新状态提示。
                        let _ = ui_tx.send(Msg::Error(
                            format!("{}", tr("net_request_failed", lang))
                        ));
                    }
                }
            }
            _ = ktick.tick() => {
                let (k, kerrs) = router.fetch_all_klines(&codes, &kline_adjust, kline_count).await;
                if !kerrs.is_empty() {
                    let _ = ui_tx.send(Msg::Error(format!(
                        "{} ({} 只: {})",
                        tr("net_request_failed", lang),
                        kerrs.len(),
                        kerrs.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(", ")
                    )));
                }
                let _ = ui_tx.send(Msg::Klines(k));
            }
            _ = itick.tick() => {
                if !tf_bars.is_empty() {
                    let (map, ierrs) = router.fetch_all_intraday(&codes, &tf_bars).await;
                    if !ierrs.is_empty() {
                        let _ = ui_tx.send(Msg::Error(format!(
                            "{} ({} 组: {})",
                            tr("net_request_failed", lang),
                            ierrs.len(),
                            ierrs.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>().join(", ")
                        )));
                    }
                    let _ = ui_tx.send(Msg::Intraday(map));
                }
            }
            _ = req_rx.recv() => {
                // 强制刷新：同时拉取统一报价与 A 股盘口。
                let quotes = router.fetch_all_quotes(&codes).await;
                let _ = ui_tx.send(Msg::Quotes(quotes));
                if let Some(d) = router.fetch_snapshot().await {
                    if !(d.indices.is_empty() && d.spots.is_empty()) {
                        let _ = ui_tx.send(Msg::Snapshot(d));
                    }
                }
            }
        }
    }
}

/// 同步 UI 循环：渲染、键盘、消息分发。
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ui_rx: std::sync::mpsc::Receiver<Msg>,
    req_tx: mpsc::Sender<Request>,
    app: &mut App,
) -> Result<()> {
    let lang = Lang::from_config(&app.config.language);
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            // 帮助打开时仅关闭帮助，不直接退出。
                            if app.show_help {
                                app.show_help = false;
                            } else {
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            // 帮助打开时先关闭帮助；否则退出。
                            if app.show_help {
                                app.show_help = false;
                            } else {
                                break;
                            }
                        }
                        KeyCode::Char('h') => {
                            // 切换帮助弹窗显示。
                            app.show_help = !app.show_help;
                        }
                        KeyCode::Char('l') => {
                            // 运行时切换界面语言（en <-> zh），帮助与界面文案即时跟随。
                            toggle_language(app);
                        }
                        KeyCode::Char('1') => app.active_view = View::Market,
                        KeyCode::Char('2') => app.active_view = View::Indicators,
                        KeyCode::Char('3') => app.active_view = View::Signals,
                        KeyCode::Char('4') => app.active_view = View::Account,
                        KeyCode::Char('5') => app.active_view = View::Strategies,
                        KeyCode::Char(' ') => {
                            if app.active_view == View::Strategies {
                                app.toggle_strategy();
                            }
                        }
                        KeyCode::Char('r') => {
                            let _ = req_tx.try_send(Request::Refresh);
                        }
                        KeyCode::Tab => {
                            if app.active_view == View::Market {
                                app.focus = match app.focus {
                                    Focus::Gainers => Focus::Losers,
                                    Focus::Losers => Focus::Gainers,
                                };
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                        KeyCode::Enter => handle_enter(app),
                        _ => {}
                    }
                }
            }
        }

        while let Ok(msg) = ui_rx.try_recv() {
            match msg {
                Msg::Snapshot(d) => apply_snapshot(app, &d),
                Msg::Quotes(q) => apply_quotes(app, &q),
                Msg::Klines(k) => merge_klines(app, k),
                Msg::Intraday(k) => merge_intraday(app, k),
                Msg::Error(e) => app.status = format!("{}: {}", tr("error", lang), e),
            }
        }
    }
    Ok(())
}

/// 运行时切换界面语言：en <-> zh。仅改内存中的 `app.config.language`，
/// 下次渲染（界面与帮助弹窗）即按新语言取词，无需重启。
fn toggle_language(app: &mut App) {
    let lang = Lang::from_config(&app.config.language);
    app.config.language = (if lang == Lang::Zh { "en" } else { "zh" }).to_string();
}

/// 收到实时快照：更新 A 股盘口（指数 + 涨跌家数 + 涨幅榜）、用最新价覆盖末根收盘、
/// 重算并求值信号。watchlist 的 name/price 来自更通用的 [`apply_quotes`]。
fn apply_snapshot(app: &mut App, d: &market::MarketData) {
    app.data = Some(d.clone());
    let lang = Lang::from_config(&app.config.language);
    app.status = tr("ok", lang).into();
    app.last_update = Some(Instant::now());

    // 以 A 股盘口的最新价覆盖末根日线收盘（盘中近似，接受假突破）。
    for s in &d.spots {
        app.apply_last_price(&s.code, s.latest_price);
    }

    if app.selected_code.is_none() {
        app.selected_code = d.spots.first().map(|s| s.code.clone());
    }

    eval_signals(app);
}

/// 收到统一实时报价：合并到 `app.quotes`，并以它为权威来源维护 `app.prices`
/// （覆盖 A 股 / 美股 / 加密货币三类资产），保证 watchlist 表格能实时更新
/// 名称与最新价。采用「增量合并」：仅更新本次返回的标的，保留此前已拉到的价格，
/// 避免某次刷新网络抖动导致表格整体回落到 `—`。若当前未选中任何标的，
/// 则回退到 watchlist 首个代码。
fn apply_quotes(app: &mut App, quotes: &[market::Quote]) {
    if quotes.is_empty() {
        return;
    }
    let lang = Lang::from_config(&app.config.language);
    app.status = tr("ok", lang).into();
    app.last_update = Some(Instant::now());

    for q in quotes {
        app.quotes.insert(q.code.clone(), q.clone());
        app.prices.insert(q.code.clone(), q.latest_price);
        // 以实时报价覆盖末根日线收盘（与 A 股盘口共用同一注入逻辑）。
        app.apply_last_price(&q.code, q.latest_price);
    }

    if app.selected_code.is_none() {
        app.selected_code = app.watchlist.first().cloned();
    }

    eval_signals(app);
}

/// 用最新价/盘口 + 日内 K 线对所有规则求值；新触发的信号发桌面通知。
fn eval_signals(app: &mut App) {
    let lang = Lang::from_config(&app.config.language);
    let reg = IndicatorRegistry::new();
    let events = app
        .engine
        .evaluate(&reg, &app.klines, &app.prices, &app.intraday_klines);
    app.signals = events.clone();
    for ev in &events {
        let title = if ev.side == Side::Buy {
            tr("buy_signal", lang)
        } else {
            tr("sell_signal", lang)
        };
        let msg = format!(
            "{} {} [{}]",
            ev.label,
            ev.code,
            if ev.side == Side::Buy { "BUY" } else { "SELL" }
        );
        app.notifier.notify(&ev.rule_id, &ev.code, title, &msg);
    }
    // 重算当前选中个股的回测胜率（供策略选择界面展示）。
    app.recompute_backtests();
}

/// 收到 K 线增量：整段替换对应标的序列（数据源每次返回完整历史）。
/// 最新价由 [`apply_quotes`]（统一实时报价）维护，这里不再写入 `app.prices`，
/// 以免与每 5s 刷新一次的报价源相互覆盖产生抖动。
fn merge_klines(app: &mut App, k: HashMap<String, Vec<Candle>>) {
    for (code, series) in k {
        app.klines.insert(code, series);
    }
}

/// 收到分钟 K 线增量：整段替换并重新求值（分钟形态规则依赖它）。
fn merge_intraday(app: &mut App, k: HashMap<String, Vec<Candle>>) {
    for (key, series) in k {
        app.intraday_klines.insert(key, series);
    }
    eval_signals(app);
}

/// 在信号/账户视图按 Enter：对选中标的以最新价市价下单。
fn handle_enter(app: &mut App) {
    let code = match &app.selected_code {
        Some(c) => c.clone(),
        None => return,
    };
    let price = match app.prices.get(&code) {
        Some(p) => *p,
        None => return,
    };
    let lang = Lang::from_config(&app.config.language);

    // 加密货币走模拟账本（可选真实下单）；A 股 / 美股走原模拟账户。
    if market_of(&code) == Market::Crypto {
        match trade_crypto(app, &code, price) {
            Ok(msg) => app.status = msg,
            Err(e) => app.status = order_failed(&e.to_string(), lang),
        }
        return;
    }

    let side = if app.active_view == View::Signals {
        app.signals
            .get(app.signal_cursor)
            .map(|s| s.side)
            .unwrap_or(Side::Buy)
    } else {
        Side::Buy
    };
    let qty = app.config.lot_size;
    let order = Order {
        code: code.clone(),
        side,
        qty,
        price,
    };
    match app.account.place_order(&order) {
        Ok(fill) => {
            let trade = fill_to_trade(&order, &fill, chrono::Local::now());
            if let Err(e) = append_trade("trades.json", &trade) {
                app.status = record_failed(&e.to_string(), lang);
            } else {
                app.trades.push(trade);
            }
            let _ = save_account("account.json", &app.account);
            app.status = format!(
                "{} {} @ {:.2}{}",
                if side == Side::Buy {
                    tr("traded_buy", lang)
                } else {
                    tr("traded_sell", lang)
                },
                code,
                price,
                traded_fee(fill.fee, lang)
            );
        }
        Err(e) => {
            app.status = order_failed(&e.to_string(), lang);
        }
    }
}

fn main() -> Result<()> {
    // 子命令：`wbot backtest [输出目录]`（A 股）/ `wbot backtest us [输出目录]`（美股）
    // —— 对全部策略跑回测并生成 markdown 报告。
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("backtest") {
        let sub = args.get(2).map(|s| s.as_str());
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        // 收集 (市场标签, 报告路径列表)，统一打印。
        let mut all: Vec<(String, Vec<(String, PathBuf)>)> = Vec::new();
        match sub {
            Some("us") => {
                let out = args.get(3).cloned().unwrap_or_else(|| "reports_us".to_string());
                let paths = rt.block_on(wbot::backtest_cli::generate_reports_us(&out))?;
                all.push(("US".into(), paths));
            }
            Some("crypto") => {
                let out = args
                    .get(3)
                    .cloned()
                    .unwrap_or_else(|| "reports_crypto".to_string());
                let paths = rt.block_on(wbot::backtest_cli::generate_reports_crypto(&out))?;
                all.push(("Crypto".into(), paths));
            }
            Some("all") => {
                let a = rt.block_on(wbot::backtest_cli::generate_reports("reports"))?;
                let u = rt.block_on(wbot::backtest_cli::generate_reports_us("reports_us"))?;
                let c = rt.block_on(wbot::backtest_cli::generate_reports_crypto("reports_crypto"))?;
                all.push(("A-share".into(), a));
                all.push(("US".into(), u));
                all.push(("Crypto".into(), c));
            }
            // 默认（无第二参数或传入目录名）跑 A 股回测。
            _ => {
                let out = sub
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "reports".to_string());
                let paths = rt.block_on(wbot::backtest_cli::generate_reports(&out))?;
                all.push(("A-share".into(), paths));
            }
        }
        let total: usize = all.iter().map(|(_, p)| p.len()).sum();
        for (label, paths) in &all {
            println!("== {}：已生成 {} 份策略回测报告 ==", label, paths.len());
            for (id, p) in paths {
                println!("  - {} : {}", id, p.display());
            }
        }
        println!("合计 {} 份报告。", total);
        return Ok(());
    }

    let refresh: u64 = 5;
    let watchlist = load_watchlist_combined();
    let config = load_config();
    let lang = Lang::from_config(&config.language);
    let account = load_account(
        "account.json",
        config.lot_size,
        config.commission,
        config.stamp_tax,
    );
    let strategies: Vec<StrategyRule> = parse_strategy_file("strategy.toml");
    let trades = load_trades("trades.json");

    // 从形态规则中提取需要拉取的 (timeframe, bars) 组合，去重。
    let mut tf_bars: Vec<(String, usize)> = Vec::new();
    for r in &strategies {
        if let (Some(tf), Some(bars)) = (r.timeframe.clone(), r.bars) {
            if !tf_bars.iter().any(|(t, b)| t == &tf && *b == bars) {
                tf_bars.push((tf, bars));
            }
        }
    }

    // Tokio 运行时；启动时拉取历史 K 线初始化账户与指标。
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let router = MarketRouter::new();
    let (initial_klines, _init_kerrs) = rt.block_on(router.fetch_all_klines(
        &watchlist,
        &config.kline_adjust,
        config.kline_count,
    ));
    let initial_intraday = if tf_bars.is_empty() {
        HashMap::new()
    } else {
        let (m, _init_ierrs) = rt.block_on(router.fetch_all_intraday(&watchlist, &tf_bars));
        m
    };

    let (ui_tx, ui_rx) = std::sync::mpsc::channel::<Msg>();
    let (req_tx, req_rx) = mpsc::channel::<Request>(4);
    rt.spawn(data_loop(
        router,
        watchlist.clone(),
        ui_tx,
        req_rx,
        refresh,
        config.kline_adjust.clone(),
        config.kline_count,
        tf_bars,
        config.intraday_refresh,
        lang,
    ));

    // 进入 raw 模式 + 备用屏幕。
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(watchlist, refresh, initial_klines, account, strategies, config);
    app.trades = trades;
    app.intraday_klines = initial_intraday;
    app.recompute_backtests();
    let result = run_app(&mut terminal, ui_rx, req_tx, &mut app);

    // 始终恢复终端。
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
