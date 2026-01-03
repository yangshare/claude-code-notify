//! 智能策略引擎模块
//!
//! 处理通知阈值过滤、聚合和场景模板匹配

use crate::config::Config;
use crate::notification::NotificationStatus;

/// 策略引擎
pub struct PolicyEngine {
    config: Config,
}

impl PolicyEngine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 检查是否应该发送通知（基于阈值和策略）
    pub fn should_notify(&self, status: NotificationStatus, duration_sec: u64, cmd: &str) -> bool {
        // 错误状态强制通知
        if matches!(status, NotificationStatus::Error) {
            return true;
        }

        // 检查白名单
        for whitelist_cmd in &self.config.threshold.whitelist {
            if cmd.contains(whitelist_cmd) {
                return true;
            }
        }

        // 检查时间阈值
        duration_sec >= self.config.threshold.min_duration
    }

    /// 匹配场景模板
    pub fn match_template(&self, cmd: &str) -> Option<String> {
        // 尝试匹配自定义模板
        for (name, _) in &self.config.templates.custom {
            if cmd.contains(name) {
                return Some(name.clone());
            }
        }

        // 返回默认模板
        Some("default".to_string())
    }

    /// 检查是否应该聚合通知
    #[allow(dead_code)]
    pub fn should_aggregate(&self) -> bool {
        self.config.aggregation.enabled
    }

    /// 获取聚合窗口时间（毫秒）
    #[allow(dead_code)]
    pub fn aggregation_window(&self) -> u64 {
        self.config.aggregation.window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_notify_error() {
        let config = Config::default();
        let engine = PolicyEngine::new(config);

        // 错误应该始终通知
        assert!(engine.should_notify(NotificationStatus::Error, 1, "test"));
        assert!(engine.should_notify(NotificationStatus::Error, 100, "test"));
    }

    #[test]
    fn test_should_notify_threshold() {
        let config = Config::default();
        let engine = PolicyEngine::new(config);

        // 低于阈值不通知
        assert!(!engine.should_notify(NotificationStatus::Success, 5, "test"));

        // 高于阈值通知
        assert!(engine.should_notify(NotificationStatus::Success, 15, "test"));
    }

    #[test]
    fn test_whitelist() {
        let mut config = Config::default();
        config.threshold.whitelist = vec!["deploy".to_string()];
        let engine = PolicyEngine::new(config);

        // 白名单命令即使短也通知
        assert!(engine.should_notify(NotificationStatus::Success, 3, "deploy"));
    }

    #[test]
    fn test_match_template() {
        let mut config = Config::default();
        config.templates.custom.insert(
            "build".to_string(),
            crate::config::TemplateConfig {
                icon: "build.png".to_string(),
                sound: "build.wav".to_string(),
                duration: 8000,
            },
        );
        let engine = PolicyEngine::new(config);

        // 匹配构建模板
        assert_eq!(engine.match_template("npm run build"), Some("build".to_string()));

        // 默认模板
        assert_eq!(engine.match_template("npm test"), Some("default".to_string()));
    }
}
