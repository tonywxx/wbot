//! ratatui 渲染入口（模块根）。按 `app.active_view` 分派到各视图子模块。

pub mod market_view;
pub mod indicator_view;
pub mod signal_view;
pub mod account_view;
pub mod strategy_view;

use std::time::Instant;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, View};

/// 涨跌配色：涨=红、跌=绿、平=灰（中国习惯）。
pub fn pct_color(v: f64) -> Color {
    if v > 0.0 {
        Color::Red
    } else if v < 0.0 {
        Color::Green
    } else {
        Color::Gray
    }
}

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 视图 Tab 栏
            Constraint::Length(3), // 头部状态
            Constraint::Length(5), // 指数条
            Constraint::Min(8),    // 视图主体
            Constraint::Length(1), // 底部
        ])
        .split(size);

    render_tabs(frame, chunks[0], app);
    render_header(frame, chunks[1], app);
    render_indices(frame, chunks[2], app);
    match app.active_view {
        View::Market => market_view::render(frame, chunks[3], app),
        View::Indicators => indicator_view::render(frame, chunks[3], app),
        View::Signals => signal_view::render(frame, chunks[3], app),
        View::Account => account_view::render(frame, chunks[3], app),
        View::Strategies => strategy_view::render(frame, chunks[3], app),
    }
    render_footer(frame, chunks[4]);
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tabs = [
        ("1 行情", View::Market),
        ("2 指标", View::Indicators),
        ("3 信号", View::Signals),
        ("4 账户", View::Account),
        ("5 策略", View::Strategies),
    ];
    let mut spans = Vec::new();
    for (label, v) in tabs {
        let active = app.active_view == v;
        spans.push(Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(if active { Color::Yellow } else { Color::Gray })
                .add_modifier(if active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ));
        spans.push(Span::raw(" "));
    }
    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title("视图"));
    frame.render_widget(p, area);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let upd = match app.last_update {
        Some(t) => match Instant::now().checked_duration_since(t) {
            Some(d) => format!("{}s 前", d.as_secs()),
            None => "—".into(),
        },
        None => "—".into(),
    };
    let title = Line::from(vec![
        Span::styled(
            " A股模拟交易 ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("状态: {}", app.status),
            Style::default().fg(Color::Cyan),
        ),
    ]);
    let sub = Line::from(format!(
        "刷新: {}s  更新: {}   [1/2/3/4/5]视图 [↑/↓]滚动 [Space]启用/停用 [Enter]下单 [r]刷新 [q]退出",
        app.refresh, upd
    ));
    let p = Paragraph::new(vec![title, sub]).block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn render_indices(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mut spans = Vec::new();
    if let Some(d) = &app.data {
        for idx in &d.indices {
            if let (Some(p), Some(pc)) = (idx.latest_price, idx.change_pct) {
                spans.push(Span::styled(
                    format!("{} ", idx.name),
                    Style::default().fg(Color::Gray),
                ));
                spans.push(Span::styled(
                    format!("{:.2} ", p),
                    Style::default().fg(Color::White),
                ));
                spans.push(Span::styled(
                    format!("{:+.2}%  ", pc),
                    Style::default().fg(pct_color(pc)),
                ));
            }
        }
    } else {
        spans.push(Span::raw("加载行情中…"));
    }
    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title("指数"))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let p = Paragraph::new("akshare-rs · 真实行情 · 红涨绿跌 (中国习惯) · 模拟交易仅供学习")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}
