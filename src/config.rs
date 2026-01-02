//! 配置管理模块
//!
//! 处理配置文件的读取、写入和验证

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 配置文件结构
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: String,
    pub sound_enabled: bool,
    pub focus_assistant_mode: FocusAssistantMode,
    pub threshold: ThresholdConfig,
    pub templates: TemplatesConfig,
    pub aggregation: AggregationConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FocusAssistantMode {
    Respect,
    Always,
    Never,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThresholdConfig {
    pub min_duration: u64,
    pub whitelist: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TemplatesConfig {
    pub default: TemplateConfig,
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, TemplateConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TemplateConfig {
    pub icon: String,
    pub sound: String,
    pub duration: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AggregationConfig {
    pub enabled: bool,
    pub window: u64,
    pub max_toasts: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            sound_enabled: true,
            focus_assistant_mode: FocusAssistantMode::Respect,
            threshold: ThresholdConfig {
                min_duration: 10,
                whitelist: vec![],
            },
            templates: TemplatesConfig {
                default: TemplateConfig {
                    icon: "auto".to_string(),
                    sound: "default".to_string(),
                    duration: 5000,
                },
                custom: std::collections::HashMap::new(),
            },
            aggregation: AggregationConfig {
                enabled: true,
                window: 5000,
                max_toasts: 3,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
            },
        }
    }
}

/// 获取配置文件路径
pub fn get_config_path() -> PathBuf {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("claude-code-notify").join("config.yaml")
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("claude-code-notify")
            .join("config.yaml")
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("claude-code-notify").join("config.yaml")
    }
}

/// 加载配置文件
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();

    if !config_path.exists() {
        log::info!("配置文件不存在，创建默认配置");
        let config = Config::default();
        save_config(&config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("无法读取配置文件: {:?}", config_path))?;

    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("配置文件格式错误: {:?}", config_path))?;

    Ok(config)
}

/// 保存配置文件
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path();

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建配置目录: {:?}", parent))?;
    }

    let content = serde_yaml::to_string(config)
        .with_context(|| "序列化配置失败")?;

    fs::write(&config_path, content)
        .with_context(|| format!("无法写入配置文件: {:?}", config_path))?;

    Ok(())
}
