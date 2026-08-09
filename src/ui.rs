//! ratatui rendering for the A-share dashboard.
//! Color convention follows Chinese market habit: 涨 (up) = Red, 跌 (down) = Green.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, Focus};
use crate::market::{find_spot, top_gainers, top_losers, Breadth};
use std::time::Instant;

/// How many movers to fetch/display per side (scroll reveals the rest).
const TOP_N: usize = 30;

/// Up = red, down = green, flat = gray (中国习惯).
fn pct_color(v: f64) -> Color {
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
            Constraint::Length(3), // header
            Constraint::Length(5), // indices bar
            Constraint::Min(8),    // body
            Constraint::Length(1), // footer
        ])
        .split(size);

    render_header(frame, chunks[0], app);
    render_indices(frame, chunks[1], app);
    render_body(frame, chunks[2], app);
    render_footer(frame, chunks[3]);
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
            " A股行情 Dashboard ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("状态: {}", app.status),
            Style::default().fg(Color::Cyan),
        ),
    ]);
    let sub = Line::from(format!(
        "刷新间隔: {}s   最近更新: {}   [q]退出 [r]刷新 [Tab]切换面板 [↑/↓]滚动",
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
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(6)])
        .split(body[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body[1]);

    render_breadth(frame, left[0], app);
    render_watchlist(frame, left[1], app);
    render_movers(frame, right[0], app, true);
    render_movers(frame, right[1], app, false);
}

fn render_breadth(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lines = match &app.data {
        Some(d) => {
            let b = Breadth::compute(&d.spots);
            vec![
                Line::from(vec![
                    Span::styled(
                        format!("上涨 {:>4}  ", b.up),
                        Style::default().fg(Color::Red),
                    ),
                    Span::styled(
                        format!("下跌 {:>4}  ", b.down),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        format!("平 {:>4}", b.flat),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("涨停 {:>4}  ", b.limit_up),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("跌停 {:>4}", b.limit_down),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(format!("总计 {:>4} 只", b.total)),
            ]
        }
        None => vec![Line::from("加载中…")],
    };
    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("市场广度"));
    frame.render_widget(p, area);
}

fn render_watchlist(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let widths = [
        Constraint::Length(9),
        Constraint::Min(8),
        Constraint::Length(11),
        Constraint::Length(10),
    ];
    let header = Row::new(vec![
        Cell::from("代码"),
        Cell::from("名称"),
        Cell::from("最新"),
        Cell::from("涨跌幅"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = match &app.data {
        Some(d) => app
            .watchlist
            .iter()
            .map(|code| match find_spot(&d.spots, code) {
                Some(s) => {
                    let c = pct_color(s.change_pct);
                    Row::new(vec![
                        Cell::from(code.clone()),
                        Cell::from(s.name.clone()),
                        Cell::from(format!("{:.2}", s.latest_price)),
                        Cell::from(format!("{:+.2}%", s.change_pct))
                            .style(Style::default().fg(c)),
                    ])
                }
                None => Row::new(vec![
                    Cell::from(code.clone()),
                    Cell::from("—"),
                    Cell::from("—"),
                    Cell::from("—").style(Style::default().fg(Color::DarkGray)),
                ]),
            })
            .collect(),
        None => vec![Row::new(vec![
            Cell::from("加载中…"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("自选股"))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_movers(frame: &mut Frame<'_>, area: Rect, app: &App, gainers: bool) {
    let title = if gainers { "涨幅榜" } else { "跌幅榜" };
    let focused = (app.focus == Focus::Gainers) == gainers;
    let title_color = if focused { Color::Yellow } else { Color::Gray };

    let widths = [
        Constraint::Length(5),
        Constraint::Length(9),
        Constraint::Min(8),
        Constraint::Length(11),
        Constraint::Length(10),
    ];
    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("代码"),
        Cell::from("名称"),
        Cell::from("最新"),
        Cell::from("涨跌幅"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = match &app.data {
        Some(d) => {
            let list = if gainers {
                top_gainers(&d.spots, TOP_N)
            } else {
                top_losers(&d.spots, TOP_N)
            };
            list.iter()
                .enumerate()
                .map(|(i, s)| {
                    let c = pct_color(s.change_pct);
                    Row::new(vec![
                        Cell::from(format!("{}", i + 1)),
                        Cell::from(s.code.clone()),
                        Cell::from(s.name.clone()),
                        Cell::from(format!("{:.2}", s.latest_price)),
                        Cell::from(format!("{:+.2}%", s.change_pct))
                            .style(Style::default().fg(c)),
                    ])
                })
                .collect()
        }
        None => vec![Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from("加载中…"),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let scroll = if gainers {
        app.scroll_gainers
    } else {
        app.scroll_losers
    };

    // ratatui 0.30 removed Table::scroll — implement scrolling by skipping rows.
    // The rank (#) column still reflects the true market rank.
    let rows: Vec<Row> = rows.into_iter().skip(scroll as usize).collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(Span::styled(title, Style::default().fg(title_color)))),
        )
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let p = Paragraph::new("akshare-rs · 实时行情 · 红涨绿跌 (中国习惯)")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}
