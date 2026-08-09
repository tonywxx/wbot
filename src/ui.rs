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
use crate::i18n::{tr, updated_ago, Lang};

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
    let lang = Lang::from_config(&app.config.language);
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
    render_footer(frame, chunks[4], lang);
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let tabs = [
        (format!("1 {}", tr("v_market", lang)), View::Market),
        (format!("2 {}", tr("v_indicators", lang)), View::Indicators),
        (format!("3 {}", tr("v_signals", lang)), View::Signals),
        (format!("4 {}", tr("v_account", lang)), View::Account),
        (format!("5 {}", tr("v_strategies", lang)), View::Strategies),
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
        .block(Block::default().borders(Borders::ALL).title(tr("views", lang)));
    frame.render_widget(p, area);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let upd = match app.last_update {
        Some(t) => match Instant::now().checked_duration_since(t) {
            Some(d) => updated_ago(d.as_secs(), lang),
            None => "—".into(),
        },
        None => "—".into(),
    };
    let title = Line::from(vec![
        Span::styled(
            format!(" {} ", tr("title", lang)),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}: {}", tr("status", lang), app.status),
            Style::default().fg(Color::Cyan),
        ),
    ]);
    let sub = Line::from(format!(
        "{}: {}s   {}   {}",
        tr("refresh", lang),
        app.refresh,
        upd,
        tr("hint", lang)
    ));
    let p = Paragraph::new(vec![title, sub]).block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn render_indices(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
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
        spans.push(Span::raw(tr("loading", lang)));
    }
    let p = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title(tr("indices", lang)))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, lang: Lang) {
    let p = Paragraph::new(tr("footer", lang))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}
