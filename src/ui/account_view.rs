//! 账户视图：账户概览 + 持仓表 + 成交记录（按 [Enter] 对选中标的下单）。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::i18n::{cash, crypto_usdt, initial, total_assets, tr, Lang};
use crate::signals::Side;
use crate::ui::pct_color;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let prices = &app.prices;
    let total = app.account.total_assets(prices);
    let unreal = app.account.unrealized_pnl(prices);
    let realized: f64 = app.trades.iter().map(|t| t.realized_pnl).sum();
    let pnl = total - app.account.initial;
    // 加密货币模拟账户权益（USDT）：现金 + 持仓市值。
    let crypto_val = app.crypto.total_value(prices);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(6)])
        .split(area);

    let summary = vec![
        Line::from(initial(app.account.initial, lang)),
        Line::from(cash(app.account.cash, lang)),
        Line::from(total_assets(total, lang)),
        Line::from(vec![
            Span::raw(tr("total_pnl", lang)),
            Span::styled(format!("{:+.2}", pnl), Style::default().fg(pct_color(pnl))),
        ]),
        Line::from(vec![
            Span::raw(tr("unrealized", lang)),
            Span::styled(format!("{:+.2}", unreal), Style::default().fg(pct_color(unreal))),
        ]),
        Line::from(vec![
            Span::raw(tr("realized", lang)),
            Span::styled(format!("{:+.2}", realized), Style::default().fg(pct_color(realized))),
        ]),
        Line::from(vec![
            Span::raw(crypto_usdt(crypto_val, lang)),
            Span::styled(format!("{:+.2}", crypto_val - app.crypto.usdt), Style::default().fg(pct_color(crypto_val - app.crypto.usdt))),
        ]),
    ];
    let p = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title(tr("account_overview", lang)));
    frame.render_widget(p, left[0]);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(left[1]);
    render_positions(frame, body[0], app);
    render_trades(frame, body[1], app);
}

fn render_positions(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let widths = [
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(12),
    ];
    let header = Row::new(vec![
        Cell::from(tr("code", lang)),
        Cell::from(tr("qty", lang)),
        Cell::from(tr("cost", lang)),
        Cell::from(tr("cur_price", lang)),
        Cell::from(tr("mkt_value", lang)),
        Cell::from(tr("pnl", lang)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows: Vec<Row> = if app.account.positions.is_empty() {
        vec![Row::new(vec![
            Cell::from(tr("no_position", lang)),
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
        .block(Block::default().borders(Borders::ALL).title(tr("positions", lang)))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_trades(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let widths = [
        Constraint::Length(9),
        Constraint::Length(5),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(11),
    ];
    let header = Row::new(vec![
        Cell::from(tr("code", lang)),
        Cell::from(tr("direction", lang)),
        Cell::from(tr("price", lang)),
        Cell::from(tr("qty", lang)),
        Cell::from(tr("hdr_realized", lang)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let n = app.trades.len();
    let start = if n > 0 { app.trade_cursor.min(n - 1) } else { 0 };
    let rows: Vec<Row> = if app.trades.is_empty() {
        vec![Row::new(vec![
            Cell::from(tr("no_trade", lang)),
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
                    Side::Buy => (tr("b", lang), Color::Red),
                    Side::Sell => (tr("s", lang), Color::Green),
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
        .block(Block::default().borders(Borders::ALL).title(tr("trades", lang)))
        .column_spacing(1);
    frame.render_widget(table, area);
}
