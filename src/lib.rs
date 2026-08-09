//! wbot 库：模拟交易 / 信号引擎 / 回测 的共享模块。
//! 二进制 (`main.rs`) 与 `examples/` 均通过 `wbot::` 引用本库。

pub mod app;
pub mod market;
pub mod ui;
pub mod indicators;
pub mod signals;
pub mod sim;
pub mod config;
pub mod persist;
pub mod notify;
pub mod backtest;
/// 回测报告编排（异步拉取数据 + 生成 markdown 报告）。
pub mod backtest_cli;

#[cfg(test)]
mod tests;
