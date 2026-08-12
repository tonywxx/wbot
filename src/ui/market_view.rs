//! 行情视图：市场广度 + 自选股 + 涨幅/跌幅榜（沿用原看板主体）。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, Focus};
use crate::i18n::{total_n, tr, Lang};
use crate::market::{find_spot, top_gainers, top_losers, Breadth};
use crate::ui::{color_scheme, pct_color};

const TOP_N: usize = 30;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
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
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    let lines = match &app.data {
        Some(d) => {
            let b = Breadth::compute(&d.spots);
            vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} {:>4}  ", tr("up", lang), b.up),
                        Style::default().fg(scheme.up),
                    ),
                    Span::styled(
                        format!("{} {:>4}  ", tr("down", lang), b.down),
                        Style::default().fg(scheme.down),
                    ),
                    Span::styled(
                        format!("{} {:>4}", tr("flat", lang), b.flat),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("{} {:>4}  ", tr("limit_up", lang), b.limit_up),
                        Style::default().fg(scheme.up).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} {:>4}", tr("limit_down", lang), b.limit_down),
                        Style::default().fg(scheme.down).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(total_n(b.total, lang)),
            ]
        }
        None => vec![Line::from(tr("loading", lang))],
    };
    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(tr("market_breadth", lang)));
    frame.render_widget(p, area);
}

fn render_watchlist(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    let widths = [
        Constraint::Length(9),
        Constraint::Min(8),
        Constraint::Length(11),
        Constraint::Length(10),
    ];
    let header = Row::new(vec![
        Cell::from(tr("code", lang)),
        Cell::from(tr("name", lang)),
        Cell::from(tr("latest", lang)),
        Cell::from(tr("change", lang)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = app
        .watchlist
        .iter()
        .map(|code| {
            // watchlist 表格以统一实时报价（A 股 / 美股 / 加密货币）为准；
            // A 股名称优先取自全市场盘口快照（更准），缺失时回退到报价里的 name。
            let board_name = app
                .data
                .as_ref()
                .and_then(|d| find_spot(&d.spots, code))
                .map(|s| s.name.clone());
            match app.quotes.get(code) {
                Some(q) => {
                    let c = pct_color(q.change_pct, &scheme);
                    let name = board_name.unwrap_or_else(|| q.name.clone());
                    Row::new(vec![
                        Cell::from(code.clone()),
                        Cell::from(name),
                        Cell::from(format!("{:.2}", q.latest_price)),
                        Cell::from(format!("{:+.2}%", q.change_pct)).style(Style::default().fg(c)),
                    ])
                }
                None => Row::new(vec![
                    Cell::from(code.clone()),
                    Cell::from(board_name.unwrap_or_else(|| "—".into())),
                    Cell::from("—"),
                    Cell::from("—").style(Style::default().fg(Color::DarkGray)),
                ]),
            }
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(tr("watchlist", lang)))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_movers(frame: &mut Frame<'_>, area: Rect, app: &App, gainers: bool) {
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    let title = if gainers { tr("gainers", lang) } else { tr("losers", lang) };
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
        Cell::from(tr("code", lang)),
        Cell::from(tr("name", lang)),
        Cell::from(tr("latest", lang)),
        Cell::from(tr("change", lang)),
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
                    let c = pct_color(s.change_pct, &scheme);
                    Row::new(vec![
                        Cell::from(format!("{}", i + 1)),
                        Cell::from(s.code.clone()),
                        Cell::from(s.name.clone()),
                        Cell::from(format!("{:.2}", s.latest_price)),
                        Cell::from(format!("{:+.2}%", s.change_pct)).style(Style::default().fg(c)),
                    ])
                })
                .collect()
        }
        None => vec![Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(tr("loading", lang)),
            Cell::from(""),
            Cell::from(""),
        ])],
    };

    let scroll = if gainers {
        app.scroll_gainers
    } else {
        app.scroll_losers
    };
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
