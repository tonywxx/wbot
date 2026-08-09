//! 示例：对 `strategy.toml` 中的每条策略执行回测，并按策略生成
//! `<id> 策略回测报告.md` 文件到指定目录（默认 `reports`）。
//!
//! 运行：
//!   cargo run --example backtest_all            # A 股回测，输出到 ./reports
//!   cargo run --example backtest_all us         # 美股回测，输出到 ./reports_us
//!   cargo run --example backtest_all us my_us   # 美股回测，输出到 ./my_us
//!   cargo run --example backtest_all my_reports # A 股回测，输出到 ./my_reports
//!
//! 该示例与二进制 `wbot backtest [us]` 子命令共用 `wbot::backtest_cli`。

use wbot::backtest_cli;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (us, out_dir) = match args.get(1).map(|s| s.as_str()) {
        Some("us") => (
            true,
            args.get(2)
                .cloned()
                .unwrap_or_else(|| "reports_us".to_string()),
        ),
        Some(d) => (false, d.to_string()),
        None => (false, "reports".to_string()),
    };

    let result = if us {
        backtest_cli::generate_reports_us(&out_dir).await
    } else {
        backtest_cli::generate_reports(&out_dir).await
    };

    match result {
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
