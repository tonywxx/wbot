//! 示例：对 `strategy.toml` 中的每条策略执行回测，并按策略生成
//! `<id> 策略回测报告.md` 文件到指定目录（默认 `reports`）。
//!
//! 运行：
//!   cargo run --example backtest_all            # 输出到 ./reports
//!   cargo run --example backtest_all my_reports # 输出到 ./my_reports
//!
//! 该示例与二进制 `wbot backtest` 子命令共用 `wbot::backtest_cli::generate_reports`。

use wbot::backtest_cli;

#[tokio::main]
async fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "reports".to_string());

    match backtest_cli::generate_reports(&out_dir).await {
        Ok(paths) => {
            println!("已生成 {} 份策略回测报告 -> {}", paths.len(), out_dir);
            for (id, p) in &paths {
                println!("  - {} : {}", id, p.display());
            }
        }
        Err(e) => {
            eprintln!("回测报告生成失败: {}", e);
            std::process::exit(1);
        }
    }
}
