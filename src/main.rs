//! A股模拟交易系统 TUI — 基于真实行情数据。
//!
//! 启动流程：加载配置/账户/策略 -> `block_on` 拉取历史 K 线初始化 -> 进入 UI 循环。
//! `data_loop` 每 5s 推送实时快照、每 ~60s 推送 K 线增量；`run_app` 收到快照后
//! 用最新价覆盖末根收盘、重算指标并求值信号，用户可在信号/账户视图按 Enter 下单。

// 模块由 src/lib.rs 声明并共享；二进制通过 `wbot::` 引用本库。
use std::collections::HashMap;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use akshare::AkShareClient;
use yfinance_rs::YfClient;
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
use wbot::indicators::{Candle, IndicatorRegistry};
use wbot::market::{self, fetch_all_intraday_market, fetch_all_klines_market, load_watchlist_combined, market_of, Market};
use wbot::persist::{load_account, save_account};
use wbot::signals::{parse_strategy_file, Side, StrategyRule};
use wbot::sim::account::{fill_to_trade, Order};
use wbot::sim::history::{append_trade, load_trades};
use wbot::ui;

/// 从异步数据任务推送到 UI 循环的消息。
enum Msg {
    Snapshot(market::MarketData),
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
    client: AkShareClient,
    yf: YfClient,
    codes: Vec<String>,
    ui_tx: std::sync::mpsc::Sender<Msg>,
    mut req_rx: mpsc::Receiver<Request>,
    refresh: u64,
    kline_adjust: String,
    kline_count: usize,
    tf_bars: Vec<(String, usize)>,
    intraday_refresh: u64,
) {
    // 首屏快照（A 股板块；美股无对应全市场板块，单独走行情接口）
    let d = market::fetch_market(&client).await;
    if !(d.indices.is_empty() && d.spots.is_empty()) {
        let _ = ui_tx.send(Msg::Snapshot(d));
    }

    let mut tick = interval(Duration::from_secs(refresh.max(1)));
    let mut ktick = interval(Duration::from_secs(60));
    let mut itick = interval(Duration::from_secs(intraday_refresh.max(10)));
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let d = market::fetch_market(&client).await;
                if d.indices.is_empty() && d.spots.is_empty() {
                    let _ = ui_tx.send(Msg::Error("网络请求失败，请检查网络连接".into()));
                } else {
                    let _ = ui_tx.send(Msg::Snapshot(d));
                }
            }
            _ = ktick.tick() => {
                let k = fetch_all_klines_market(&client, &yf, &codes, &kline_adjust, kline_count).await;
                let _ = ui_tx.send(Msg::Klines(k));
            }
            _ = itick.tick() => {
                if !tf_bars.is_empty() {
                    let map = fetch_all_intraday_market(&client, &yf, &codes, &tf_bars).await;
                    let _ = ui_tx.send(Msg::Intraday(map));
                }
            }
            _ = req_rx.recv() => {
                let d = market::fetch_market(&client).await;
                if !(d.indices.is_empty() && d.spots.is_empty()) {
                    let _ = ui_tx.send(Msg::Snapshot(d));
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
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
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
                Msg::Klines(k) => merge_klines(app, k),
                Msg::Intraday(k) => merge_intraday(app, k),
                Msg::Error(e) => app.status = format!("错误: {}", e),
            }
        }
    }
    Ok(())
}

/// 收到实时快照：更新数据、用最新价覆盖末根收盘、重算并求值信号。
fn apply_snapshot(app: &mut App, d: &market::MarketData) {
    app.data = Some(d.clone());
    app.status = "OK".into();
    app.last_update = Some(Instant::now());

    let mut prices = HashMap::new();
    for s in &d.spots {
        prices.insert(s.code.clone(), s.latest_price);
        // 盘中近似：以最新价覆盖末根收盘（接受假突破）
        if let Some(k) = app.klines.get_mut(&s.code) {
            if let Some(last) = k.last_mut() {
                last.close = s.latest_price;
            }
        }
    }
    app.prices = prices;

    if app.selected_code.is_none() {
        app.selected_code = d.spots.first().map(|s| s.code.clone());
    }

    eval_signals(app);
}

/// 用最新价/盘口 + 日内 K 线对所有规则求值；新触发的信号发桌面通知。
fn eval_signals(app: &mut App) {
    let reg = IndicatorRegistry::new();
    let events = app
        .engine
        .evaluate(&reg, &app.klines, &app.prices, &app.intraday_klines);
    app.signals = events.clone();
    for ev in &events {
        let title = if ev.side == Side::Buy { "买入信号" } else { "卖出信号" };
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
/// 对美股标的，用末根收盘补全 `prices`（美股无 A 股式实时盘口，取最后收盘价）。
fn merge_klines(app: &mut App, k: HashMap<String, Vec<Candle>>) {
    for (code, series) in k {
        if market_of(&code) == Market::Us {
            if let Some(last) = series.last() {
                app.prices.insert(code.clone(), last.close);
            }
        }
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
                app.status = format!("成交但记录失败: {}", e);
            } else {
                app.trades.push(trade);
            }
            let _ = save_account("account.json", &app.account);
            app.status = format!(
                "已{} {} @ {:.2} (费用 {:.2})",
                if side == Side::Buy { "买入" } else { "卖出" },
                code,
                price,
                fill.fee
            );
        }
        Err(e) => {
            app.status = format!("下单失败: {}", e);
        }
    }
}

fn main() -> Result<()> {
    // 子命令：`wbot backtest [输出目录]`（A 股）/ `wbot backtest us [输出目录]`（美股）
    // —— 对全部策略跑回测并生成 markdown 报告。
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("backtest") {
        // 第二个参数可能是 "us" 或输出目录。
        let (us, out_dir) = match args.get(2).map(|s| s.as_str()) {
            Some("us") => (true, args.get(3).cloned().unwrap_or_else(|| "reports_us".to_string())),
            Some(dir) => (false, dir.to_string()),
            None => (false, "reports".to_string()),
        };
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let paths = if us {
            rt.block_on(wbot::backtest_cli::generate_reports_us(&out_dir))?
        } else {
            rt.block_on(wbot::backtest_cli::generate_reports(&out_dir))?
        };
        println!("已生成 {} 份策略回测报告 -> {}", paths.len(), out_dir);
        for (id, p) in &paths {
            println!("  - {} : {}", id, p.display());
        }
        return Ok(());
    }

    let refresh: u64 = 5;
    let watchlist = load_watchlist_combined();
    let config = load_config();
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
    let client = AkShareClient::new();
    let yf = YfClient::default();
    let initial_klines = rt.block_on(fetch_all_klines_market(
        &client,
        &yf,
        &watchlist,
        &config.kline_adjust,
        config.kline_count,
    ));
    let initial_intraday = if tf_bars.is_empty() {
        HashMap::new()
    } else {
        rt.block_on(fetch_all_intraday_market(&client, &yf, &watchlist, &tf_bars))
    };

    let (ui_tx, ui_rx) = std::sync::mpsc::channel::<Msg>();
    let (req_tx, req_rx) = mpsc::channel::<Request>(4);
    rt.spawn(data_loop(
        client,
        yf,
        watchlist.clone(),
        ui_tx,
        req_rx,
        refresh,
        config.kline_adjust.clone(),
        config.kline_count,
        tf_bars,
        config.intraday_refresh,
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
