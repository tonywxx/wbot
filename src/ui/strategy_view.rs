//! 策略选择视图：列出全部策略（备注/说明/适用场景），展示当前选中个股的回测胜率，
//! 支持 [Space] 启用/停用与 [↑/↓] 选择。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::i18n::{backtest_line, note, period_min, strategies, tr, Lang};
use crate::signals::Side;
use crate::ui::color_scheme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(area);

    render_list(frame, chunks[0], app);
    render_detail(frame, chunks[1], app);
}

fn render_list(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let scheme = color_scheme(&app.config);
    let widths = [
        Constraint::Length(4),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Min(18),
        Constraint::Length(9),
        Constraint::Length(6),
    ];
    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from(tr("status", lang)),
        Cell::from(tr("side", lang)),
        Cell::from(tr("strategy", lang)),
        Cell::from(tr("winrate", lang)),
        Cell::from(tr("count", lang)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let code = app.selected_code.clone().unwrap_or_default();
    let rows: Vec<Row> = app
        .strategies
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let active = i == app.strategy_cursor;
            let status = if r.enabled { "✓" } else { "✗" };
            let status_color = if r.enabled {
                Color::Green
            } else {
                Color::DarkGray
            };
            let (side_str, side_color) = match r.side {
                Side::Buy => (tr("buy", lang), scheme.up),
                Side::Sell => (tr("sell", lang), scheme.down),
            };
            let bt = app.backtests.get(&r.id);
            let win = match bt {
                Some(b) if b.trades > 0 => format!("{:.1}", b.win_rate * 100.0),
                _ => "—".into(),
            };
            let cnt = match bt {
                Some(b) => format!("{}", b.trades),
                None => "—".into(),
            };
            let mut row = Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(status).style(Style::default().fg(status_color)),
                Cell::from(side_str).style(Style::default().fg(side_color)),
                Cell::from(r.label.clone()),
                Cell::from(win),
                Cell::from(cnt),
            ]);
            if active {
                row = row.style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
            }
            row
        })
        .collect();

    let title = strategies(
        app.strategies.len(),
        if code.is_empty() { "—" } else { &code },
        lang,
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    if app.strategies.is_empty() {
        let p = Paragraph::new(tr("no_strategy", lang))
            .block(Block::default().borders(Borders::ALL).title(tr("detail", lang)));
        frame.render_widget(p, area);
        return;
    }
    let idx = app.strategy_cursor.min(app.strategies.len() - 1);
    let r = &app.strategies[idx];
    let code = app.selected_code.clone().unwrap_or_default();
    let enabled = if r.enabled {
        tr("enabled", lang)
    } else {
        tr("disabled", lang)
    };
    let side = match r.side {
        Side::Buy => tr("buy", lang),
        Side::Sell => tr("sell", lang),
    };
    let period = match &r.timeframe {
        Some(tf) => period_min(tf, lang),
        None => tr("daily", lang).to_string(),
    };
    let bt = app.backtests.get(&r.id);
    let mut lines = vec![
        format!(
            "[{}] {}  {}  {}:{}  {}:{}",
            enabled,
            r.id,
            r.label,
            tr("direction", lang),
            side,
            tr("period_lbl", lang),
            period
        ),
        note(
            if r.note.is_empty() {
                tr("no_note", lang)
            } else {
                &r.note
            },
            lang,
        ),
    ];
    if let Some(b) = bt {
        let pf = if b.profit_factor.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}", b.profit_factor)
        };
        lines.push(backtest_line(
            if code.is_empty() { "—" } else { &code },
            b.trades,
            b.win_rate * 100.0,
            b.avg_win * 100.0,
            b.avg_loss * 100.0,
            &pf,
            b.max_drawdown * 100.0,
            b.total_return * 100.0,
            lang,
        ));
    } else {
        lines.push(tr("backtest_none", lang).into());
    }
    lines.push(tr("space_toggle", lang).into());

    let p = Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<Line>>())
        .block(Block::default().borders(Borders::ALL).title(tr("detail", lang)));
    frame.render_widget(p, area);
}
