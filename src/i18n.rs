//! 国际化（i18n）：界面语言切换。
//!
//! 默认英文（`En`）；用户可在 `config.toml` 将 `language` 设为 `"zh"` 切换为简体中文。
//! 所有界面文案通过 [`tr`] 取词，未知 key 回退英文，绝不 panic。

/// 界面语言。默认 [`Lang::En`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    /// 英文（默认）。
    #[default]
    En,
    /// 简体中文。
    Zh,
}

impl Lang {
    /// 由 `config.toml` 的 language 字段解析；非 zh 系列一律视为英文。
    pub fn from_config(s: &str) -> Lang {
        match s.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_cn" | "chinese" | "简体" | "中文" | "中文版" => Lang::Zh,
            _ => Lang::En,
        }
    }

    /// 回写为配置字符串。
    pub fn as_config(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }
}

/// 取词：先返回英文默认值，若语言为简体中文且存在对应译文则覆盖。
///
/// 约定：含动态参数的文案以 `{}` 占位，调用方用 `format!` 填充；纯静态文案直接显示。
/// 未知 key 回退为静态空串（便于发现遗漏），绝不 panic。
pub fn tr(key: &str, lang: Lang) -> &'static str {
    // 英文基线（同时作为未知 key 的回退）。
    let en = match key {
        // ---- 通用 / 根布局 ----
        "views" => "Views",
        "status" => "Status",
        "refresh" => "Refresh",
        "updated_ago" => "Updated: {}s ago",
        "title" => "Crypto-ready A-share / US / Crypto Sim Trader",
        "hint" => "[1/2/3/4/5] Views  [↑/↓] Scroll  [Space] Enable/Disable  [Enter] Trade  [r] Refresh  [q] Quit",
        "indices" => "Indices",
        "loading" => "Loading…",
        "footer" => "real quotes via akshare-rs / yfinance-rs / okx-rs · red up, green down (China convention) · simulated trading for learning",
        "error" => "Error",
        "ok" => "OK",

        // ---- 视图 Tab 标签（数字前缀 + 视图名）----
        "v_market" => "Market",
        "v_indicators" => "Indicators",
        "v_signals" => "Signals",
        "v_account" => "Account",
        "v_strategies" => "Strategies",
        // ---- 账户成交表头 ----
        "hdr_realized" => "Realized",

        // ---- 行情视图 ----
        "market_breadth" => "Market Breadth",
        "up" => "Up",
        "down" => "Down",
        "flat" => "Flat",
        "limit_up" => "Limit Up",
        "limit_down" => "Limit Down",
        "total_n" => "Total {}",
        "watchlist" => "Watchlist",
        "code" => "Code",
        "name" => "Name",
        "latest" => "Latest",
        "change" => "Chg%",
        "gainers" => "Top Gainers",
        "losers" => "Top Losers",

        // ---- 指标视图 ----
        "technicals" => "Technical Indicators (↑/↓ switch symbol)",
        "no_symbol" => "No symbol selected",
        "no_kline" => "No K-line data",
        "latest_price" => "Latest: {}",
        "last_close" => "Last Close: {}",
        "bullish" => "Short-term MA bullish (MA5 > MA10)",
        "bearish" => "Short-term MA bearish (MA5 < MA10)",

        // ---- 信号视图 ----
        "signals" => "Signals ({} triggered) — [Enter] Trade",
        "no_signals" => "No triggered signals right now.\n{} strategy rules loaded (see strategy.toml).\nAfter a signal fires, press [Enter] to trade at the latest price.",
        "direction" => "Side",
        "rule" => "Rule",
        "time" => "Time",
        "buy" => "BUY",
        "sell" => "SELL",

        // ---- 账户视图 ----
        "account_overview" => "Account Overview ([Enter] trade on selected symbol)",
        "initial" => "Initial: {:.2}",
        "cash" => "Cash: {:.2}",
        "total_assets" => "Total Assets: {:.2}",
        "total_pnl" => "Total P/L: ",
        "unrealized" => "Unrealized: ",
        "realized" => "Realized: ",
        "crypto_usdt" => "Crypto USDT: {:.2}",
        "positions" => "Positions",
        "no_position" => "No positions",
        "qty" => "Qty",
        "cost" => "Cost",
        "cur_price" => "Price",
        "mkt_value" => "Value",
        "pnl" => "P/L",
        "trades" => "Trades (↑/↓ browse)",
        "no_trade" => "No trades",
        "price" => "Price",
        "b" => "B",
        "s" => "S",

        // ---- 策略视图 ----
        "strategies" => "Strategy Selection ({} rules) — backtest win rate for {}",
        "side" => "Side",
        "strategy" => "Strategy",
        "winrate" => "Win%",
        "count" => "Trades",
        "detail" => "Strategy Notes / Backtest",
        "no_strategy" => "No strategies.",
        "enabled" => "Enabled",
        "disabled" => "Disabled",
        "daily" => "Daily",
        "period_min" => "{} min",
        "note" => "Note: {}",
        "no_note" => "(no note)",
        "backtest_none" => "Backtest: no data yet (waiting for K-line load)",
        "backtest_line" => "Backtest({}): {} trades  Win {:.1}%  AvgWin {:.2}%  AvgLoss {:.2}%  PF {}  MaxDD {:.1}%  Total {:.2}%",
        "space_toggle" => "[Space] Enable/Disable  [↑/↓] Select",

        // ---- 回测报告 ----
        "report_title" => "{} Strategy Backtest Report",
        "report_id" => "Strategy ID",
        "strategy_info" => "Strategy Info",
        "market_lbl" => "Market",
        "name_lbl" => "Name",
        "direction_lbl" => "Direction",
        "period_lbl" => "Period",
        "signal_lbl" => "Signal",
        "note_lbl" => "Note",
        "params" => "Backtest Params",
        "data_range" => "Data Range",
        "codes_count" => "Symbols Backtested",
        "summary" => "Backtest Summary (cross-symbol)",
        "triggers" => "Triggers (trades)",
        "wins_lbl" => "Winning",
        "win_rate_lbl" => "Win Rate",
        "avg_win_lbl" => "Avg Win",
        "avg_loss_lbl" => "Avg Loss",
        "profit_factor_lbl" => "Profit Factor",
        "total_return_lbl" => "Total Net Return (sum)",
        "max_dd_lbl" => "Max Drawdown",
        "per_code" => "Per-Symbol Detail",
        "na" => "N/A",
        "notes_section" => "Notes",
        "content" => "Content",
        "hold_n" => "Hold {} bars after signal",
        "one_way_fee" => "one-way commission",
        "round_trip" => "round-trip ~",
        "note_1" => "This backtest replays strategy signals on historical candles of the selected symbols, measuring performance by forward return after holding a fixed number of bars following each trigger.",
        "note_2" => "Edge-triggered (counted only on false→true) to avoid double-counting persistent signals; commission is deducted at one-way rate x2, excluding stamp tax and slippage.",
        "note_3" => "Total net return is the sum of per-symbol equity-curve net returns, for horizontal comparison only and not representative of live returns.",
        "note_4" => "Historical backtest results do not represent future returns. This report is for reference only and does not constitute any investment advice.",
        "long_dir" => "Buy (Long)",
        "short_dir" => "Sell (Short)",
        "infinity" => "∞ (no loss)",

        // ---- 运行时提示 ----
        "net_request_failed" => "Network request failed, check connection",
        "buy_signal" => "Buy Signal",
        "sell_signal" => "Sell Signal",
        "order_failed" => "Order failed: {}",
        "record_failed" => "Filled but failed to record: {}",
        "traded_buy" => "Bought",
        "traded_sell" => "Sold",
        "traded_fee" => " (fee {:.2})",

            // 未知 key：回退为静态空串（便于发现遗漏）。
        _ => "",
    };

    if lang == Lang::Zh {
        return match key {
            // ---- 通用 / 根布局 ----
            "views" => "视图",
            "status" => "状态",
            "refresh" => "刷新",
            "updated_ago" => "更新: {}s 前",
            "title" => "加密货币就绪的 A股 / 美股 / 加密货币 模拟交易",
            "hint" => "[1/2/3/4/5] 视图  [↑/↓] 滚动  [Space] 启用/停用  [Enter] 下单  [r] 刷新  [q] 退出",
            "indices" => "指数",
            "loading" => "加载中…",
            "footer" => "akshare-rs / yfinance-rs / okx-rs 真实行情 · 红涨绿跌（中国习惯）· 模拟交易仅供学习",
            "error" => "错误",
            "ok" => "OK",

            // ---- 视图 Tab 标签（数字前缀 + 视图名）----
            "v_market" => "行情",
            "v_indicators" => "指标",
            "v_signals" => "信号",
            "v_account" => "账户",
            "v_strategies" => "策略",
            // ---- 账户成交表头 ----
            "hdr_realized" => "已实现",

            // ---- 行情视图 ----
            "market_breadth" => "市场广度",
            "up" => "上涨",
            "down" => "下跌",
            "flat" => "平",
            "limit_up" => "涨停",
            "limit_down" => "跌停",
            "total_n" => "总计 {} 只",
            "watchlist" => "自选股",
            "code" => "代码",
            "name" => "名称",
            "latest" => "最新",
            "change" => "涨跌幅",
            "gainers" => "涨幅榜",
            "losers" => "跌幅榜",

            // ---- 指标视图 ----
            "technicals" => "技术指标 (↑/↓ 切换标的)",
            "no_symbol" => "未选择标的",
            "no_kline" => "暂无 K 线数据",
            "latest_price" => "最新价: {}",
            "last_close" => "末根收盘: {}",
            "bullish" => "短期均线多头排列 (MA5 > MA10)",
            "bearish" => "短期均线空头排列 (MA5 < MA10)",

            // ---- 信号视图 ----
            "signals" => "信号 ({} 触发) — [Enter] 下单",
            "no_signals" => "当前无触发信号。\n已加载 {} 条策略规则（见 strategy.toml）。\n信号触发后按 [Enter] 以最新价下单。",
            "direction" => "方向",
            "rule" => "规则",
            "time" => "时间",
            "buy" => "买入",
            "sell" => "卖出",

            // ---- 账户视图 ----
            "account_overview" => "账户概览 ([Enter]对选中标的下单)",
            "initial" => "初始资金: {:.2}",
            "cash" => "现金:     {:.2}",
            "total_assets" => "总资产:   {:.2}",
            "total_pnl" => "总盈亏:    ",
            "unrealized" => "浮动盈亏:  ",
            "realized" => "已实现盈亏:",
            "crypto_usdt" => "加密 USDT: {:.2}",
            "positions" => "持仓",
            "no_position" => "无持仓",
            "qty" => "数量",
            "cost" => "成本",
            "cur_price" => "现价",
            "mkt_value" => "市值",
            "pnl" => "盈亏",
            "trades" => "成交记录 (↑/↓ 浏览)",
            "no_trade" => "无成交",
            "price" => "价格",
            "b" => "买",
            "s" => "卖",

            // ---- 策略视图 ----
            "strategies" => "策略选择 ({} 条) — 个股 {} 回测胜率",
            "side" => "方向",
            "strategy" => "策略",
            "winrate" => "胜率%",
            "count" => "次数",
            "detail" => "策略说明 / 回测",
            "no_strategy" => "无策略。",
            "enabled" => "已启用",
            "disabled" => "已停用",
            "daily" => "日线",
            "period_min" => "{} 分钟",
            "note" => "说明: {}",
            "no_note" => "（无备注）",
            "backtest_none" => "回测: 暂无数据（等待对应 K 线加载）",
            "backtest_line" => "回测({}): 交易{}次 胜率{:.1}% 均盈{:.2}% 均亏{:.2}% 盈亏比{} 最大回撤{:.1}% 累计{:.2}%",
            "space_toggle" => "[Space] 启用/停用  [↑/↓] 选择",

            // ---- 回测报告 ----
            "report_title" => "{} 策略回测报告",
            "report_id" => "策略 ID",
            "strategy_info" => "策略信息",
            "market_lbl" => "市场",
            "name_lbl" => "名称",
            "direction_lbl" => "方向",
            "period_lbl" => "周期",
            "signal_lbl" => "信号",
            "note_lbl" => "备注",
            "params" => "回测参数",
            "data_range" => "数据区间",
            "codes_count" => "参与回测标的数",
            "summary" => "回测汇总（跨标的）",
            "triggers" => "触发次数（交易笔数）",
            "wins_lbl" => "盈利笔数",
            "win_rate_lbl" => "胜率",
            "avg_win_lbl" => "平均盈利",
            "avg_loss_lbl" => "平均亏损",
            "profit_factor_lbl" => "盈亏比",
            "total_return_lbl" => "累计净收益（多标的合计）",
            "max_dd_lbl" => "最大回撤",
            "per_code" => "分标的明细",
            "na" => "N/A",
            "notes_section" => "说明",
            "content" => "内容",
            "hold_n" => "信号触发后持有 {} 根",
            "one_way_fee" => "单边佣金",
            "round_trip" => "往返约",
            "note_1" => "本回测在所选标的的历史 K 线上重放策略信号，以「信号触发后持有固定根数」的前向收益衡量策略表现。",
            "note_2" => "采用沿触发（false→true 才计入），避免持续信号被重复计数；手续费按单边佣金 ×2 扣减，未计印花税与滑点。",
            "note_3" => "累计净收益为多标的各自权益曲线净收益的合计，仅供横向比较，不代表实盘收益。",
            "note_4" => "历史回测结果不代表未来收益，本报告仅供参考，不构成任何投资建议。",
            "long_dir" => "买入（做多）",
            "short_dir" => "卖出（做空）",
            "infinity" => "∞（无亏损）",

            // ---- 运行时提示 ----
            "net_request_failed" => "网络请求失败，请检查网络连接",
            "buy_signal" => "买入信号",
            "sell_signal" => "卖出信号",
            "order_failed" => "下单失败: {}",
            "record_failed" => "成交但记录失败: {}",
            "traded_buy" => "已买入",
            "traded_sell" => "已卖出",
            "traded_fee" => " (费用 {:.2})",

            // 未知 key 回退英文。
            _ => en,
        };
    }

    en
}

// ---------------------------------------------------------------------------
// 含参数的本地化辅助函数。
//
// Rust 的 `format!` 要求格式串必须是**字面量**，无法把 `tr(key)` 返回的模板字符串
// 直接作为 `format!` 的第一个参数。因此所有「带占位符 `{}` / `{:.2}`」的文案都收敛
// 为下面的辅助函数：函数内部按语言选用字面量格式串，对外暴露类型化的参数。
// （`tr` 仍保留这些占位符 key 作为文案参考，但格式化请走辅助函数。）
// ---------------------------------------------------------------------------

/// 账户初始资金行。
pub fn initial(v: f64, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("初始资金: {:.2}", v)
    } else {
        format!("Initial: {:.2}", v)
    }
}

/// 账户现金行。
pub fn cash(v: f64, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("现金:     {:.2}", v)
    } else {
        format!("Cash: {:.2}", v)
    }
}

/// 账户总资产行。
pub fn total_assets(v: f64, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("总资产:   {:.2}", v)
    } else {
        format!("Total Assets: {:.2}", v)
    }
}

/// 加密货币账户 USDT 权益行。
pub fn crypto_usdt(v: f64, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("加密 USDT: {:.2}", v)
    } else {
        format!("Crypto USDT: {:.2}", v)
    }
}

/// 指标视图：最新价行（参数为已格式化好的价格字符串）。
pub fn latest_price(s: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("最新价: {}", s)
    } else {
        format!("Latest: {}", s)
    }
}

/// 指标视图：末根收盘行（参数为已格式化好的价格字符串）。
pub fn last_close(s: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("末根收盘: {}", s)
    } else {
        format!("Last Close: {}", s)
    }
}

/// 头部：更新于 N 秒前。
pub fn updated_ago(secs: u64, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("更新: {}s 前", secs)
    } else {
        format!("Updated: {}s ago", secs)
    }
}

/// 行情广度：总计 N 只。
pub fn total_n(n: usize, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("总计 {} 只", n)
    } else {
        format!("Total {}", n)
    }
}

/// 策略视图：分钟周期标签（参数为周期字符串如 "15"）。
pub fn period_min(tf: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("{} 分钟", tf)
    } else {
        format!("{} min", tf)
    }
}

/// 回测报告：信号触发后持有 N 根。
pub fn hold_n(n: usize, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("信号触发后持有 {} 根", n)
    } else {
        format!("Hold {} bars after signal", n)
    }
}

/// 回测报告标题（参数为策略名）。
pub fn report_title(label: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("{} 策略回测报告", label)
    } else {
        format!("{} Strategy Backtest Report", label)
    }
}

/// 策略视图标题（参数：策略数、当前个股）。
pub fn strategies(count: usize, code: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("策略选择 ({} 条) — 个股 {} 回测胜率", count, code)
    } else {
        format!(
            "Strategy Selection ({} rules) — backtest win rate for {}",
            count, code
        )
    }
}

/// 策略详情：说明行（参数为说明文本）。
pub fn note(text: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("说明: {}", text)
    } else {
        format!("Note: {}", text)
    }
}

/// 策略详情：回测单行（参数：个股、交易数、胜率%、均盈%、均亏%、盈亏比、最大回撤%、累计%）。
pub fn backtest_line(
    code: &str,
    trades: usize,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    pf: &str,
    max_dd: f64,
    total: f64,
    lang: Lang,
) -> String {
    if lang == Lang::Zh {
        format!(
            "回测({}): 交易{}次 胜率{:.1}% 均盈{:.2}% 均亏{:.2}% 盈亏比{} 最大回撤{:.1}% 累计{:.2}%",
            code, trades, win_rate, avg_win, avg_loss, pf, max_dd, total
        )
    } else {
        format!(
            "Backtest({}): {} trades  Win {:.1}%  AvgWin {:.2}%  AvgLoss {:.2}%  PF {}  MaxDD {:.1}%  Total {:.2}%",
            code, trades, win_rate, avg_win, avg_loss, pf, max_dd, total
        )
    }
}

/// 信号视图：空状态文案（参数为策略数）。
pub fn no_signals(n: usize, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!(
            "当前无触发信号。\n已加载 {} 条策略规则（见 strategy.toml）。\n信号触发后按 [Enter] 以最新价下单。",
            n
        )
    } else {
        format!(
            "No triggered signals right now.\n{} strategy rules loaded (see strategy.toml).\nAfter a signal fires, press [Enter] to trade at the latest price.",
            n
        )
    }
}

/// 信号视图标题（参数为信号数）。
pub fn signals(n: usize, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("信号 ({} 触发) — [Enter] 下单", n)
    } else {
        format!("Signals ({} triggered) — [Enter] Trade", n)
    }
}

/// 运行时：下单失败（参数为错误描述）。
pub fn order_failed(e: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("下单失败: {}", e)
    } else {
        format!("Order failed: {}", e)
    }
}

/// 运行时：成交但记录失败（参数为错误描述）。
pub fn record_failed(e: &str, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!("成交但记录失败: {}", e)
    } else {
        format!("Filled but failed to record: {}", e)
    }
}

/// 运行时：成交费用后缀（参数为费用数值）。
pub fn traded_fee(v: f64, lang: Lang) -> String {
    if lang == Lang::Zh {
        format!(" (费用 {:.2})", v)
    } else {
        format!(" (fee {:.2})", v)
    }
}
