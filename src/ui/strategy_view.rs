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
use crate::signals::Side;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(area);

    render_list(frame, chunks[0], app);
    render_detail(frame, chunks[1], app);
}

fn render_list(frame: &mut Frame<'_>, area: Rect, app: &App) {
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
        Cell::from("状态"),
        Cell::from("方向"),
        Cell::from("策略"),
        Cell::from("胜率%"),
        Cell::from("次数"),
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
                Side::Buy => ("买入", Color::Red),
                Side::Sell => ("卖出", Color::Green),
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

    let title = format!(
        "策略选择 ({} 条) — 个股 {} 回测胜率",
        app.strategies.len(),
        if code.is_empty() { "—".into() } else { code }
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if app.strategies.is_empty() {
        let p = Paragraph::new("无策略。")
            .block(Block::default().borders(Borders::ALL).title("策略说明 / 回测"));
        frame.render_widget(p, area);
        return;
    }
    let idx = app.strategy_cursor.min(app.strategies.len() - 1);
    let r = &app.strategies[idx];
    let code = app.selected_code.clone().unwrap_or_default();
    let enabled = if r.enabled { "已启用" } else { "已停用" };
    let side = match r.side {
        Side::Buy => "买入",
        Side::Sell => "卖出",
    };
    let bt = app.backtests.get(&r.id);
    let mut lines = vec![
        format!(
            "[{}] {}  {}  方向:{}  周期:{}",
            enabled,
            r.id,
            r.label,
            side,
            r.timeframe.clone().unwrap_or_else(|| "日线".to_string())
        ),
        format!("说明: {}", if r.note.is_empty() { "（无备注）" } else { &r.note }),
    ];
    if let Some(b) = bt {
        let pf = if b.profit_factor.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.2}", b.profit_factor)
        };
        lines.push(format!(
            "回测({}): 交易{}次 胜率{:.1}% 均盈{:.2}% 均亏{:.2}% 盈亏比{} 最大回撤{:.1}% 累计{:.2}%",
            if code.is_empty() { "—".into() } else { code },
            b.trades,
            b.win_rate * 100.0,
            b.avg_win * 100.0,
            b.avg_loss * 100.0,
            pf,
            b.max_drawdown * 100.0,
            b.total_return * 100.0,
        ));
    } else {
        lines.push("回测: 暂无数据（等待对应 K 线加载）".into());
    }
    lines.push("[Space] 启用/停用  [↑/↓] 选择".into());

    let p = Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<Line>>())
        .block(Block::default().borders(Borders::ALL).title("策略说明 / 回测"));
    frame.render_widget(p, area);
}
