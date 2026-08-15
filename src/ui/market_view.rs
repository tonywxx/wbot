//! 行情视图：美股市场广度 + 自选股 + 策略通知日志（Strategy Log）。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use std::collections::VecDeque;

use crate::app::App;
use crate::i18n::{total_n, tr, Lang};
use crate::signals::Side;
use crate::ui::{color_scheme, pct_color};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // 2x2 网格：上排 指数 | 市场广度，下排 自选股 | 策略日志。
    // 上下两排使用相同的左右列宽（42% / 58%），保证左右两列对齐。
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(6)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(rows[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(rows[1]);

    crate::ui::render_indices(frame, top[0], app); // 指数（上排左）
    render_breadth(frame, top[1], app);            // 市场广度（上排右，与指数对齐）
    render_watchlist(frame, bottom[0], app);       // 自选股（下排左）
    render_strategy_log(frame, bottom[1], app);    // 策略日志（下排右，与自选股对齐）
}

fn render_breadth(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    let lines = match &app.breadth {
        Some(b) => {
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
                        format!("{} {:>4}  ", tr("strong_up", lang), b.limit_up),
                        Style::default().fg(scheme.up).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} {:>4}", tr("strong_down", lang), b.limit_down),
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

/// UTF-8 迷你走势图：把近期价格序列按高低归一化为 9 级方块字符
/// （` ` ▁▂▃▄▅▆▇█），整体首尾方向决定着色（涨绿 / 跌红）。
/// `width` 为渲染宽度（字符数），数据不足时左补空格、最新价靠右。
fn sparkline_cell(hist: &VecDeque<f64>, width: u16, up: Color, down: Color) -> Cell<'_> {
    let w = width as usize;
    let n = hist.len();
    if n < 2 {
        return Cell::from(" ".repeat(w));
    }
    let vals: Vec<f64> = hist.iter().copied().collect();
    let start = n.saturating_sub(w);
    let window: &[f64] = &vals[start..];
    let min = window.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    const BARS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut s = String::with_capacity(w);
    for _ in 0..w.saturating_sub(window.len()) {
        s.push(' ');
    }
    if span <= f64::EPSILON {
        // 平盘：整段以空格表示，避免伪造一根中线。
        while s.len() < w {
            s.push(' ');
        }
    } else {
        let scale = (BARS.len() - 1) as f64;
        for &v in window {
            let level = (((v - min) / span) * scale).round() as usize;
            s.push(BARS[level.min(BARS.len() - 1)]);
        }
    }
    let rising = window[window.len() - 1] >= window[0];
    Cell::from(s).style(Style::default().fg(if rising { up } else { down }))
}

fn render_watchlist(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    // 名称列默认隐藏（`show_name == false`）；按 `n` 切换。隐藏时为 Latest 价差
    // 列让出宽度，使其能完整显示 `▲ 225.29 (+0.12)` 这类带价差的价格。
    let show_name = app.show_name;

    // Latest 价格单元格：无变动显示裸价；有变动则显示方向箭头 + 最新价 + 价差。
    let price_cell = |code: &str, price: f64| -> Cell {
        if let Some(f) = app.price_flash.get(code) {
            let fg = if f.dir > 0 { scheme.up } else { scheme.down };
            let arrow = if f.dir > 0 { "▲" } else { "▼" };
            Cell::from(format!("{} {:.2} ({:+.2})", arrow, price, f.delta))
                .style(Style::default().fg(fg).add_modifier(Modifier::BOLD))
        } else {
            Cell::from(format!("{:.2}", price))
        }
    };

    // 走势图列宽：名称隐藏时更宽，给迷你图更多空间。
    let trend_width = if show_name { 12 } else { 16 };

    let (widths, header) = if show_name {
        (
            vec![
                Constraint::Length(9),
                Constraint::Min(6),
                Constraint::Length(16),
                Constraint::Length(9),
                Constraint::Length(trend_width),
            ],
            Row::new(vec![
                Cell::from(tr("code", lang)),
                Cell::from(tr("name", lang)),
                Cell::from(tr("latest", lang)),
                Cell::from(tr("change", lang)),
                Cell::from(tr("trend", lang)),
            ])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
        )
    } else {
        (
            vec![
                Constraint::Length(9),
                Constraint::Length(16),
                Constraint::Length(9),
                Constraint::Length(trend_width),
            ],
            Row::new(vec![
                Cell::from(tr("code", lang)),
                Cell::from(tr("latest", lang)),
                Cell::from(tr("change", lang)),
                Cell::from(tr("trend", lang)),
            ])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
        )
    };

    let rows: Vec<Row> = app
        .watchlist
        .iter()
        .map(|code| {
            // watchlist 表格以统一实时报价（A 股 / 美股 / 加密货币）为准；
            // A 股名称优先取自实时报价，缺失时回退到代码。
            match app.quotes.get(code) {
                Some(q) => {
                    let c = pct_color(q.change_pct, &scheme);
                    let latest = price_cell(code, q.latest_price);
                    let trend = match app.price_history.get(code) {
                        Some(h) => sparkline_cell(h, trend_width, scheme.up, scheme.down),
                        None => Cell::from(" ".repeat(trend_width as usize)),
                    };
                    if show_name {
                        Row::new(vec![
                            Cell::from(code.clone()),
                            Cell::from(q.name.clone()),
                            latest,
                            Cell::from(format!("{:+.2}%", q.change_pct))
                                .style(Style::default().fg(c)),
                            trend,
                        ])
                    } else {
                        Row::new(vec![
                            Cell::from(code.clone()),
                            latest,
                            Cell::from(format!("{:+.2}%", q.change_pct))
                                .style(Style::default().fg(c)),
                            trend,
                        ])
                    }
                }
                None => {
                    if show_name {
                        Row::new(vec![
                            Cell::from(code.clone()),
                            Cell::from("—"),
                            Cell::from("—"),
                            Cell::from("—").style(Style::default().fg(Color::DarkGray)),
                            Cell::from(" ".repeat(trend_width as usize)),
                        ])
                    } else {
                        Row::new(vec![
                            Cell::from(code.clone()),
                            Cell::from("—"),
                            Cell::from("—").style(Style::default().fg(Color::DarkGray)),
                            Cell::from(" ".repeat(trend_width as usize)),
                        ])
                    }
                }
            }
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(tr("watchlist", lang)))
        .column_spacing(1);
    frame.render_widget(table, area);
}

/// 策略通知日志（Strategy Log）：展示所有策略 noti 提示，最新在前，可滚动。
/// 每条含触发时间、买卖方向标签与提示原文；长文本自动换行。
fn render_strategy_log(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    let mut lines: Vec<Line> = Vec::new();
    if app.strategy_log.is_empty() {
        lines.push(Line::from(tr("no_strategy_log", lang)));
    } else {
        for entry in app.strategy_log.iter().skip(app.log_scroll as usize) {
            let side_color = match entry.side {
                Side::Buy => scheme.up,
                Side::Sell => scheme.down,
            };
            let tag = if entry.side == Side::Buy { "B" } else { "S" };
            // 第一行：时间 + [买/卖] 信号 + 代码 名称。
            let ts_part = format!("{} ", entry.ts);
            let tag_part = format!("[{}] ", tag);
            lines.push(Line::from(vec![
                Span::styled(ts_part.clone(), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    tag_part.clone(),
                    Style::default().fg(side_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{} {}", entry.code, entry.name)),
            ]));
            // 第二行：原因，与首行 [买/卖] 信号对齐（缩进时间戳宽度，加「原因：」前缀）。
            let indent = " ".repeat(disp_width(&ts_part));
            lines.push(Line::from(vec![
                Span::raw(indent.clone()),
                Span::styled(
                    "原因：".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(entry.reason.clone()),
            ]));
            // 第三行：判断周期，标明该信号依据什么周期（日线 / 15m 等）触发。
            lines.push(Line::from(vec![
                Span::raw(indent.clone()),
                Span::styled(
                    "周期：".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(entry.period.clone()),
            ]));
        }
    }
    let title = format!("{} ({})", tr("strategy_log", lang), app.strategy_log.len());
    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

/// 终端显示宽度：CJK（含中文）按 2 列计，其余按 1 列计，用于策略日志第二行缩进对齐。
fn disp_width(s: &str) -> usize {
    s.chars()
        .map(|c| if (c as u32) > 0x1100 { 2 } else { 1 })
        .sum()
}
