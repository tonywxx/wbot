//! 桌面通知（macOS 用 `osascript`；其它平台降级为 stderr 输出）。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 带冷却的去重通知器。同一 `(code, rule_id)` 在冷却期内只弹一次。
pub struct Notifier {
    enabled: bool,
    cooldown: Duration,
    last: HashMap<(String, String), Instant>,
}

impl Notifier {
    pub fn new(enabled: bool, cooldown_secs: u64) -> Self {
        Notifier {
            enabled,
            cooldown: Duration::from_secs(cooldown_secs),
            last: HashMap::new(),
        }
    }

    /// 发送一条通知；冷却期内重复同键不发送。未启用时静默丢弃。
    /// 返回 `true` 表示本次真正发出了通知（用于驱动策略通知日志）。
    pub fn notify(&mut self, rule_id: &str, code: &str, title: &str, message: &str) -> bool {
        if !self.enabled {
            return false;
        }
        let key = (code.to_string(), rule_id.to_string());
        if let Some(t) = self.last.get(&key)
            && t.elapsed() < self.cooldown
        {
            return false;
        }
        send_notification(title, message);
        self.last.insert(key, Instant::now());
        true
    }
}

#[cfg(target_os = "macos")]
fn send_notification(title: &str, msg: &str) {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        esc(msg),
        esc(title)
    );
    if std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .is_err()
    {
        eprintln!("[NOTIFY] {}: {}", title, msg);
    }
}

#[cfg(not(target_os = "macos"))]
fn send_notification(title: &str, msg: &str) {
    eprintln!("[NOTIFY] {}: {}", title, msg);
}

/// 转义 AppleScript 字符串中的反斜杠与双引号。
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
