//! 信号视图：列出本周期新触发的买卖信号，光标可选中并按 Enter 下单。

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::signals::Side;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if app.signals.is_empty() {
        let p = Paragraph::new(format!(
            "当前无触发信号。\n已加载 {} 条策略规则（见 strategy.toml）。\n命中信号后按 [Enter] 以最新价下单。",
            app.strategies.len()
        ))
        .block(Block::default().borders(Borders::ALL).title("信号 (Signals)"));
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
        Cell::from("代码"),
        Cell::from("方向"),
        Cell::from("规则"),
        Cell::from("时间"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = app
        .signals
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let active = i == app.signal_cursor;
            let (side_str, side_color) = match s.side {
                Side::Buy => ("买入", Color::Red),
                Side::Sell => ("卖出", Color::Green),
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

    let title = format!("信号 ({} 条) — [Enter]下单", app.signals.len());
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(1);
    frame.render_widget(table, area);
}
