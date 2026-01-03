//! 通知聚合模块
//!
//! 管理通知的聚合和批量发送

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 聚合状态文件
#[derive(Debug, Serialize, Deserialize)]
struct AggregationState {
    notifications: Vec<AggregatedNotification>,
    window_start: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AggregatedNotification {
    status: String,
    duration: u64,
    cmd: String,
    timestamp: u64,
}

impl AggregationState {
    fn new(window: u64) -> Self {
        Self {
            notifications: Vec::new(),
            window_start: now(),
        }
    }

    /// 检查窗口是否过期
    fn is_expired(&self, window_ms: u64) -> bool {
        let elapsed = now().saturating_sub(self.window_start);
        elapsed > window_ms / 1000
    }

    /// 添加通知
    fn add(&mut self, status: &str, duration: u64, cmd: &str) {
        self.notifications.push(AggregatedNotification {
            status: status.to_string(),
            duration,
            cmd: cmd.to_string(),
            timestamp: now(),
        });
    }

    /// 获取统计信息
    fn get_stats(&self) -> (usize, usize, usize) {
        let success = self.notifications.iter().filter(|n| n.status == "success").count();
        let error = self.notifications.iter().filter(|n| n.status == "error").count();
        let total = self.notifications.len();
        (total, success, error)
    }
}

/// 通知聚合器
pub struct NotificationAggregator {
    state_file: PathBuf,
    window_ms: u64,
    max_toasts: usize,
}

impl NotificationAggregator {
    pub fn new(state_file: PathBuf, window_ms: u64, max_toasts: usize) -> Self {
        Self {
            state_file,
            window_ms,
            max_toasts,
        }
    }

    /// 添加通知到聚合器
    pub fn add_notification(&self, status: &str, duration: u64, cmd: &str) -> Result<Option<AggregatedResult>> {
        let mut state = self.load_state().unwrap_or_else(|_| AggregationState::new(self.window_ms));

        // 检查窗口是否过期
        if state.is_expired(self.window_ms) {
            // 窗口过期，重置状态
            state = AggregationState::new(self.window_ms);
        }

        // 添加新通知
        state.add(status, duration, cmd);

        // 检查是否达到最大数量
        let should_send = state.notifications.len() >= self.max_toasts;

        if should_send {
            // 保存状态并发送聚合通知
            self.save_state(&state)?;
            Ok(Some(self.build_result(&state)))
        } else {
            // 保存状态但不发送
            self.save_state(&state)?;
            Ok(None)
        }
    }

    /// 刷新待发送的通知
    pub fn flush(&self) -> Result<Option<AggregatedResult>> {
        let state = self.load_state()?;

        if state.notifications.is_empty() {
            return Ok(None);
        }

        // 清空状态
        self.clear_state()?;
        Ok(Some(self.build_result(&state)))
    }

    /// 检查是否有待发送的通知
    pub fn has_pending(&self) -> bool {
        if let Ok(state) = self.load_state() {
            !state.notifications.is_empty() && !state.is_expired(self.window_ms)
        } else {
            false
        }
    }

    fn load_state(&self) -> Result<AggregationState> {
        if !self.state_file.exists() {
            return Ok(AggregationState::new(self.window_ms));
        }

        let content = fs::read_to_string(&self.state_file)?;
        serde_json::from_str(&content).map_err(Into::into)
    }

    fn save_state(&self, state: &AggregationState) -> Result<()> {
        let content = serde_json::to_string(state)?;
        fs::write(&self.state_file, content)?;
        Ok(())
    }

    fn clear_state(&self) -> Result<()> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)?;
        }
        Ok(())
    }

    fn build_result(&self, state: &AggregationState) -> AggregatedResult {
        let (total, success, error) = state.get_stats();
        AggregatedResult {
            total,
            success,
            error,
            notifications: state.notifications.clone(),
        }
    }
}

/// 聚合结果
#[derive(Debug, Clone)]
pub struct AggregatedResult {
    pub total: usize,
    pub success: usize,
    pub error: usize,
    pub notifications: Vec<AggregatedNotification>,
}

impl AggregatedResult {
    /// 生成聚合通知的标题
    pub fn title(&self) -> String {
        if self.error > 0 {
            format!("{} 个任务完成 ({} 成功, {} 失败)", self.total, self.success, self.error)
        } else {
            format!("{} 个任务已完成", self.total)
        }
    }

    /// 生成聚合通知的消息
    pub fn message(&self) -> String {
        let mut lines = Vec::new();

        if self.success > 0 {
            lines.push(format!("✅ 成功: {} 个", self.success));
        }
        if self.error > 0 {
            lines.push(format!("❌ 失败: {} 个", self.error));
        }

        if !self.notifications.is_empty() && self.notifications.len() <= 5 {
            lines.push(String::from("\n最近的任务:"));
            for notif in self.notifications.iter().take(5) {
                let icon = if notif.status == "success" { "✅" } else { "❌" };
                lines.push(format!("  {} {} ({}秒)", icon, notif.cmd, notif.duration));
            }
        }

        lines.join("\n")
    }

    /// 获取状态（用于发送通知）
    pub fn status(&self) -> &str {
        if self.error > 0 { "error" } else { "success" }
    }
}

/// 获取当前时间戳（秒）
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 获取聚合状态文件路径
pub fn get_state_file_path() -> PathBuf {
    #[cfg(windows)]
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());

    #[cfg(target_os = "macos")]
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    #[cfg(not(any(windows, target_os = "macos")))]
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    PathBuf::from(base).join("claude-code-notify").join("aggregation.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_stats() {
        let mut state = AggregationState::new(5000);
        state.add("success", 10, "npm test");
        state.add("error", 5, "cargo build");
        state.add("success", 20, "make deploy");

        let (total, success, error) = state.get_stats();
        assert_eq!(total, 3);
        assert_eq!(success, 2);
        assert_eq!(error, 1);
    }

    #[test]
    fn test_aggregated_result_title() {
        let result = AggregatedResult {
            total: 5,
            success: 3,
            error: 2,
            notifications: vec![],
        };

        let title = result.title();
        assert!(title.contains("5"));
        assert!(title.contains("3 成功"));
        assert!(title.contains("2 失败"));
    }
}
