//! 账户视图：账户概览 + 持仓表 + 成交记录（按 [Enter] 对选中标的下单）。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::signals::Side;
use crate::ui::pct_color;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let prices = &app.prices;
    let total = app.account.total_assets(prices);
    let unreal = app.account.unrealized_pnl(prices);
    let realized: f64 = app.trades.iter().map(|t| t.realized_pnl).sum();
    let pnl = total - app.account.initial;

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(6)])
        .split(area);

    let summary = vec![
        Line::from(format!("初始资金: {:.2}", app.account.initial)),
        Line::from(format!("现金:     {:.2}", app.account.cash)),
        Line::from(format!("总资产:   {:.2}", total)),
        Line::from(vec![
            Span::raw("总盈亏:    "),
            Span::styled(format!("{:+.2}", pnl), Style::default().fg(pct_color(pnl))),
        ]),
        Line::from(vec![
            Span::raw("浮动盈亏:  "),
            Span::styled(format!("{:+.2}", unreal), Style::default().fg(pct_color(unreal))),
        ]),
        Line::from(vec![
            Span::raw("已实现盈亏:"),
            Span::styled(format!("{:+.2}", realized), Style::default().fg(pct_color(realized))),
        ]),
    ];
    let p = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title("账户概览 ([Enter]对选中标的下单)"));
    frame.render_widget(p, left[0]);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(left[1]);
    render_positions(frame, body[0], app);
    render_trades(frame, body[1], app);
}

fn render_positions(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let widths = [
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(12),
    ];
    let header = Row::new(vec![
        Cell::from("代码"),
        Cell::from("数量"),
        Cell::from("成本"),
        Cell::from("现价"),
        Cell::from("市值"),
        Cell::from("盈亏"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = if app.account.positions.is_empty() {
        vec![Row::new(vec![
            Cell::from("无持仓"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        app.account
            .positions
            .values()
            .map(|pos| {
                let price = app.prices.get(&pos.code).copied().unwrap_or(pos.avg_cost);
                let mv = pos.qty as f64 * price;
                let pnl = (price - pos.avg_cost) * pos.qty as f64;
                Row::new(vec![
                    Cell::from(pos.code.clone()),
                    Cell::from(format!("{}", pos.qty)),
                    Cell::from(format!("{:.2}", pos.avg_cost)),
                    Cell::from(format!("{:.2}", price)),
                    Cell::from(format!("{:.2}", mv)),
                    Cell::from(format!("{:+.2}", pnl)).style(Style::default().fg(pct_color(pnl))),
                ])
            })
            .collect()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("持仓"))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_trades(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let widths = [
        Constraint::Length(9),
        Constraint::Length(5),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(11),
    ];
    let header = Row::new(vec![
        Cell::from("代码"),
        Cell::from("方向"),
        Cell::from("价格"),
        Cell::from("数量"),
        Cell::from("已实现"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let n = app.trades.len();
    let start = if n > 0 { app.trade_cursor.min(n - 1) } else { 0 };
    let rows: Vec<Row> = if app.trades.is_empty() {
        vec![Row::new(vec![
            Cell::from("无成交"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        // 倒序：最新成交在顶部附近，按 cursor 高亮
        let mut all: Vec<Row> = app
            .trades
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let (side_str, side_color) = match t.side {
                    Side::Buy => ("买", Color::Red),
                    Side::Sell => ("卖", Color::Green),
                };
                let active = i == start;
                let mut row = Row::new(vec![
                    Cell::from(t.code.clone()),
                    Cell::from(side_str).style(Style::default().fg(side_color)),
                    Cell::from(format!("{:.2}", t.price)),
                    Cell::from(format!("{}", t.qty)),
                    Cell::from(format!("{:+.2}", t.realized_pnl))
                        .style(Style::default().fg(pct_color(t.realized_pnl))),
                ]);
                if active {
                    row = row.style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
                }
                row
            })
            .collect();
        all.reverse();
        all
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("成交记录 (↑/↓ 浏览)"))
        .column_spacing(1);
    frame.render_widget(table, area);
}
