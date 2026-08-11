//! wbot 库：模拟交易 / 信号引擎 / 回测 的共享模块。
//! 二进制 (`main.rs`) 与 `examples/` 均通过 `wbot::` 引用本库。

pub mod app;
pub mod market;
pub mod ui;
pub mod indicators;
pub mod signals;
/// 规则驱动的序列选取与持仓门槛（统一真相源，供 eval / backtest / app 共用）。
pub mod series;
pub mod sim;
pub mod config;
pub mod persist;
pub mod notify;
pub mod backtest;
/// 加密货币（OKX）集成：行情拉取 / 真实下单 / 模拟账户。
pub mod crypto;
/// 国际化：界面语言（默认英文，可切换简体中文）。
pub mod i18n;
/// 回测报告编排（异步拉取数据 + 生成 markdown 报告）。
pub mod backtest_cli;

#[cfg(test)]
mod tests;
