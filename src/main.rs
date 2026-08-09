//! A股行情 TUI Dashboard — built on akshare-rs + ratatui.
//!
//! Layout: header (status) · index bar · [breadth + watchlist | gainers + losers].
//! Data is fetched by an async task on a tokio runtime and pushed to the sync
//! UI loop over an std mpsc channel; the UI loop handles keyboard input.

mod app;
mod market;
mod ui;

use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use app::{App, Focus};
use akshare::AkShareClient;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Message pushed from the async data task to the UI loop.
enum Msg {
    Snapshot(market::MarketData),
    Error(String),
}

/// Request pushed from the UI loop to the data task (force refresh).
enum Request {
    Refresh,
}

/// Async worker: refresh on an interval OR on a forced request, then push a snapshot.
async fn data_loop(
    client: AkShareClient,
    ui_tx: std::sync::mpsc::Sender<Msg>,
    mut req_rx: mpsc::Receiver<Request>,
    refresh: u64,
) {
    let mut tick = interval(Duration::from_secs(refresh.max(1)));
    loop {
        tokio::select! {
            _ = tick.tick() => {}
            _ = req_rx.recv() => {}
        }
        let d = market::fetch_market(&client).await;
        // Both empty => total network failure; surface an error instead of a blank board.
        if d.indices.is_empty() && d.spots.is_empty() {
            let _ = ui_tx.send(Msg::Error("网络请求失败，请检查网络连接".into()));
        } else {
            let _ = ui_tx.send(Msg::Snapshot(d));
        }
    }
}

/// Sync UI loop: render, read keys, drain data messages.
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
                        KeyCode::Char('r') => {
                            let _ = req_tx.try_send(Request::Refresh);
                        }
                        KeyCode::Tab => {
                            app.focus = match app.focus {
                                Focus::Gainers => Focus::Losers,
                                Focus::Losers => Focus::Gainers,
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_focused(1),
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_focused(-1),
                        _ => {}
                    }
                }
            }
        }

        while let Ok(msg) = ui_rx.try_recv() {
            match msg {
                Msg::Snapshot(d) => {
                    app.data = Some(d);
                    app.status = "OK".into();
                    app.last_update = Some(std::time::Instant::now());
                }
                Msg::Error(e) => {
                    app.status = format!("错误: {}", e);
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let refresh: u64 = 5;
    let watchlist = market::load_watchlist();

    // Tokio runtime for the async data worker.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let client = AkShareClient::new();
    let (ui_tx, ui_rx) = std::sync::mpsc::channel::<Msg>();
    let (req_tx, req_rx) = mpsc::channel::<Request>(4);
    rt.spawn(data_loop(client, ui_tx, req_rx, refresh));

    // Enter raw mode + alternate screen.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(watchlist, refresh);
    let result = run_app(&mut terminal, ui_rx, req_tx, &mut app);

    // Always restore the terminal.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
