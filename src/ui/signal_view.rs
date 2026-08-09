//! 信号视图：列出本周期新触发的买卖信号，光标可选中并按 Enter 下单。

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::i18n::{no_signals, signals, tr, Lang};
use crate::signals::Side;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    if app.signals.is_empty() {
        let p = Paragraph::new(no_signals(app.strategies.len(), lang))
            .block(Block::default().borders(Borders::ALL).title(signals(0, lang)));
        frame.render_widget(p, area);
        return;
    }

    let widths = [
        Constraint::Length(4),
        Constraint::Length(9),
        Constraint::Length(6),
        Constraint::Min(10),
        Constraint::Length(12),
    ];
    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from(tr("code", lang)),
        Cell::from(tr("direction", lang)),
        Cell::from(tr("rule", lang)),
        Cell::from(tr("time", lang)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = app
        .signals
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let active = i == app.signal_cursor;
            let (side_str, side_color) = match s.side {
                Side::Buy => (tr("buy", lang), Color::Red),
                Side::Sell => (tr("sell", lang), Color::Green),
            };
            let mut row = Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(s.code.clone()),
                Cell::from(side_str).style(Style::default().fg(side_color)),
                Cell::from(s.label.clone()),
                Cell::from(s.ts.to_string()),
            ]);
            if active {
                row = row.style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
            }
            row
        })
        .collect();

    let title = signals(app.signals.len(), lang);
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(1);
    frame.render_widget(table, area);
}
