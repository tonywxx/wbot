//! 模拟交易核心：账户、持仓、下单成交、盈亏。
//! 子模块：`account`（股票账户状态与成交）、`crypto_ledger`（加密模拟账本）、
//! `history`（成交记录持久化）。

pub mod account;
pub mod crypto_ledger;
pub mod history;
