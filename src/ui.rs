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
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, View};
use crate::config::AppConfig;
use crate::i18n::{help_items, tr, updated_ago, Lang};

/// 涨跌配色方案：由 `config.toml` 的 `up_color` / `down_color` 决定，默认 涨=绿、跌=红。
pub struct ColorScheme {
    pub up: Color,
    pub down: Color,
}

/// 把 `config.toml` 中的颜色字符串解析为 `ratatui::Color`。
/// 支持命名色（red/green/yellow/blue/cyan/magenta/white/gray/darkgray 及 light* 变体）
/// 与 `#rrggbb`；无法识别时回退到默认涨/跌色，绝不 panic。
pub fn parse_color(s: &str) -> Option<Color> {
    match s.trim().to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "white" => Some(Color::White),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        other => {
            let h = other.trim_start_matches('#');
            if h.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&h[0..2], 16),
                    u8::from_str_radix(&h[2..4], 16),
                    u8::from_str_radix(&h[4..6], 16),
                ) {
                    return Some(Color::Rgb(r, g, b));
                }
            }
            None
        }
    }
}

/// 由配置构造涨跌配色方案；非法颜色回退到默认（涨=绿、跌=红）。
pub fn color_scheme(cfg: &AppConfig) -> ColorScheme {
    let up = parse_color(&cfg.up_color).unwrap_or(Color::Green);
    let down = parse_color(&cfg.down_color).unwrap_or(Color::Red);
    ColorScheme { up, down }
}

/// 涨跌配色：涨=up_color、跌=down_color、平=灰。具体取值来自 [`color_scheme`]。
pub fn pct_color(v: f64, scheme: &ColorScheme) -> Color {
    if v > 0.0 {
        scheme.up
    } else if v < 0.0 {
        scheme.down
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

    // 帮助弹窗覆盖在其它视图之上（按 `h` 切换）。
    if app.show_help {
        render_help_overlay(frame, app);
    }
}

/// 帮助弹窗：居中的按键说明面板，文案按当前语言本地化。
fn render_help_overlay(frame: &mut Frame<'_>, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let items = help_items(lang);

    // 列宽：按键列取最长按键串，说明列取最长显示宽度（CJK 计 2 列）。
    let key_w = items
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(0);
    let desc_w = items
        .iter()
        .map(|(_, d)| d.chars().map(|c| if (c as u32) > 0x1100 { 2 } else { 1 }).sum::<usize>())
        .max()
        .unwrap_or(0);

    let mut lines: Vec<Line> = Vec::with_capacity(items.len() + 2);
    for (k, d) in &items {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<width$}", k, width = key_w),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(*d, Style::default().fg(Color::White)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        tr("help_close", lang),
        Style::default().fg(Color::DarkGray),
    )));

    let size = frame.area();
    let popup_w = (key_w + 2 + desc_w + 4)
        .clamp(40, size.width.saturating_sub(4) as usize)
        as u16;
    let popup_h = ((items.len() + 2) as u16 + 4).min(size.height.saturating_sub(2));

    // 水平 + 垂直居中。
    let area = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(popup_h),
        Constraint::Fill(1),
    ])
    .split(size)[1];
    let area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(popup_w),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(tr("help_title", lang))
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left),
        area,
    );
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
    let scheme = color_scheme(&app.config);
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
                    Style::default().fg(pct_color(pc, &scheme)),
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
