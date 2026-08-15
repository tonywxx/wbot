//! 指标视图：展示选中标的的 MA / RSI / MACD 数值与多空状态。

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::i18n::{last_close, latest_price, tr, Lang};
use crate::indicators::{last_value, IndicatorId, IndicatorRegistry, PriceSource};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lang = Lang::from_config(&app.config.language);
    let code = match &app.selected_code {
        Some(c) => c.clone(),
        None => {
            render_empty(frame, area, tr("no_symbol", lang), lang);
            return;
        }
    };
    let series = match app.klines.get(&code) {
        Some(s) if !s.is_empty() => s,
        _ => {
            render_empty(frame, area, tr("no_kline", lang), lang);
            return;
        }
    };

    let reg = IndicatorRegistry::new();
    let ma5 = last_value(&reg, &id("MA", PriceSource::Close, &[5.0], None), series);
    let ma10 = last_value(&reg, &id("MA", PriceSource::Close, &[10.0], None), series);
    let ma20 = last_value(&reg, &id("MA", PriceSource::Close, &[20.0], None), series);
    let rsi14 = last_value(&reg, &id("RSI", PriceSource::Close, &[14.0], None), series);
    let dif = last_value(
        &reg,
        &id("MACD", PriceSource::Close, &[12.0, 26.0, 9.0], Some("dif".into())),
        series,
    );
    let dea = last_value(
        &reg,
        &id("MACD", PriceSource::Close, &[12.0, 26.0, 9.0], Some("dea".into())),
        series,
    );
    let hist = last_value(
        &reg,
        &id("MACD", PriceSource::Close, &[12.0, 26.0, 9.0], Some("hist".into())),
        series,
    );

    let price = app.prices.get(&code).copied();
    let last_close_val = series.last().map(|c| c.close);
    // 名称优先取统一实时报价（A 股 / 美股 / 加密货币）里的 name，缺失则为空。
    let name = app
        .quotes
        .get(&code)
        .map(|q| q.name.clone())
        .unwrap_or_default();

    let f = |v: Option<f64>| v.map(|x| format!("{:.2}", x)).unwrap_or("—".into());

    let lp = price.map(|x| format!("{:.2}", x)).unwrap_or_else(|| "—".to_string());
    let lc = last_close_val.map(|x| format!("{:.2}", x)).unwrap_or_else(|| "—".to_string());

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", code),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(name, Style::default().fg(Color::Gray)),
        ]),
        Line::from(format!(
            "{}    {}",
            latest_price(&lp, lang),
            last_close(&lc, lang)
        )),
        Line::from(""),
        Line::from(format!("MA5:   {}", f(ma5))),
        Line::from(format!("MA10:  {}", f(ma10))),
        Line::from(format!("MA20:  {}", f(ma20))),
        Line::from(format!("RSI14: {}", f(rsi14))),
        Line::from(""),
        Line::from(format!("MACD DIF:  {}", f(dif))),
        Line::from(format!("MACD DEA:  {}", f(dea))),
        Line::from(vec![
            Span::raw("MACD HIST: "),
            Span::styled(
                hist.map(|x| format!("{:+.3}", x)).unwrap_or("—".into()),
                Style::default().fg(match hist {
                    Some(h) if h >= 0.0 => Color::Red,
                    Some(_) => Color::Green,
                    None => Color::Gray,
                }),
            ),
        ]),
    ];

    let bullish = ma5.unwrap_or(f64::NAN) > ma10.unwrap_or(f64::NAN);
    lines.push(Line::from(Span::styled(
        if bullish {
            tr("bullish", lang)
        } else {
            tr("bearish", lang)
        },
        Style::default().fg(if bullish { Color::Red } else { Color::Green }),
    )));

    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(tr("technicals", lang)));
    frame.render_widget(p, area);
}

fn id(kind: &str, src: PriceSource, params: &[f64], field: Option<String>) -> IndicatorId {
    IndicatorId {
        kind: kind.to_string(),
        source: src,
        params: params.to_vec(),
        field,
    }
}

fn render_empty(frame: &mut Frame<'_>, area: Rect, msg: &str, lang: Lang) {
    let p = Paragraph::new(msg)
        .block(Block::default().borders(Borders::ALL).title(tr("technicals", lang)));
    frame.render_widget(p, area);
}
