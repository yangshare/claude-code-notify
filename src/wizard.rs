//! 交互式配置向导模块
//!
//! 提供简单的问答式配置界面

use anyhow::Result;
use std::io::{self, Write};

use crate::config::{save_config, Config, FocusAssistantMode};

/// 配置向导
pub struct ConfigWizard;

impl ConfigWizard {
    pub fn new() -> Self {
        Self
    }

    /// 运行配置向导
    pub fn run(&self) -> Result<Config> {
        println!("╔════════════════════════════════════════╗");
        println!("║   Claude Code Notify 配置向导         ║");
        println!("╚════════════════════════════════════════╝");
        println!();

        // 加载现有配置或创建默认配置
        let mut config = Config::default();

        // 配置声音
        config.sound_enabled = self.ask_bool("是否启用通知声音？", true)?;

        // 配置专注助手模式
        println!("\n专注助手模式：");
        println!("  1. respect - 尊重系统专注助手设置");
        println!("  2. always - 始终播放声音");
        println!("  3. never - 从不播放声音");
        let mode_choice = self.ask_choice("选择专注助手模式", &["respect", "always", "never"], 0)?;
        config.focus_assistant_mode = match mode_choice {
            0 => FocusAssistantMode::Respect,
            1 => FocusAssistantMode::Always,
            2 => FocusAssistantMode::Never,
            _ => FocusAssistantMode::Respect,
        };

        // 配置最小阈值
        println!("\n最小通知阈值：");
        println!("  执行时间低于此值的任务将不会发送通知（错误除外）");
        config.threshold.min_duration = self.ask_number("输入最小阈值（秒）", 10, 1, 3600)?;

        // 配置白名单
        println!("\n白名单命令：");
        println!("  在白名单中的命令无论耗时多少都会发送通知");
        println!("  输入命令关键字（如 deploy, release），或直接回车跳过");
        let whitelist_input = self.ask_input("输入白名单命令（用逗号分隔）", "")?;
        if !whitelist_input.is_empty() {
            config.threshold.whitelist = whitelist_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 配置聚合
        config.aggregation.enabled = self.ask_bool("\n是否启用通知聚合？", true)?;
        if config.aggregation.enabled {
            config.aggregation.window = self.ask_number("聚合时间窗口（毫秒）", 5000, 1000, 60000)?;
            config.aggregation.max_toasts = self.ask_number("最大聚合数量", 3, 1, 10)? as usize;
        }

        // 配置日志级别
        println!("\n日志级别：");
        println!("  1. debug - 详细调试信息");
        println!("  2. info - 一般信息（默认）");
        println!("  3. warn - 警告信息");
        println!("  4. error - 仅错误信息");
        let log_choice = self.ask_choice("选择日志级别", &["debug", "info", "warn", "error"], 1)?;
        config.logging.level = match log_choice {
            0 => "debug".to_string(),
            1 => "info".to_string(),
            2 => "warn".to_string(),
            3 => "error".to_string(),
            _ => "info".to_string(),
        };

        // 显示配置预览
        self.show_preview(&config);

        // 确认保存
        let confirmed = self.ask_bool("\n确认保存配置？", true)?;

        if confirmed {
            save_config(&config)?;
            println!("\n✅ 配置已保存！");
        } else {
            println!("\n配置未保存");
        }

        Ok(config)
    }

    /// 显示配置预览
    fn show_preview(&self, config: &Config) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║          配置预览                     ║");
        println!("╚════════════════════════════════════════╝");
        println!("声音: {}", if config.sound_enabled { "✅ 启用" } else { "❌ 禁用" });
        println!("专注助手: {:?}", config.focus_assistant_mode);
        println!("最小阈值: {} 秒", config.threshold.min_duration);
        println!("白名单: {:?}", config.threshold.whitelist);
        println!("聚合: {}", if config.aggregation.enabled { "✅ 启用" } else { "❌ 禁用" });
        if config.aggregation.enabled {
            println!("  - 聚合窗口: {} 毫秒", config.aggregation.window);
            println!("  - 最大数量: {}", config.aggregation.max_toasts);
        }
        println!("日志级别: {}", config.logging.level);
    }

    /// 询问布尔值问题
    fn ask_bool(&self, prompt: &str, default: bool) -> Result<bool> {
        let default_str = if default { "Y/n" } else { "y/N" };
        loop {
            print!("{} [{}]: ", prompt, default_str);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim().to_lowercase();

            if input.is_empty() {
                return Ok(default);
            }

            match input.as_str() {
                "y" | "yes" | "是" => return Ok(true),
                "n" | "no" | "否" => return Ok(false),
                _ => {
                    println!("请输入 y 或 n");
                }
            }
        }
    }

    /// 询问数字
    fn ask_number(&self, prompt: &str, default: u64, min: u64, max: u64) -> Result<u64> {
        loop {
            print!("{} [默认: {}]: ", prompt, default);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();

            if input.is_empty() {
                return Ok(default);
            }

            match input.parse::<u64>() {
                Ok(n) if n >= min && n <= max => return Ok(n),
                Ok(_) => {
                    println!("请输入 {} 到 {} 之间的数字", min, max);
                }
                Err(_) => {
                    println!("请输入有效的数字");
                }
            }
        }
    }

    /// 询问文本输入
    fn ask_input(&self, prompt: &str, default: &str) -> Result<String> {
        print!("{} [默认: {}]: ", prompt, default);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_string();

        if input.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(input)
        }
    }

    /// 询问选择
    fn ask_choice(&self, prompt: &str, options: &[&str], default: usize) -> Result<usize> {
        loop {
            println!("{}", prompt);
            for (i, opt) in options.iter().enumerate() {
                println!("  {}. {}", i + 1, opt);
            }
            print!("请选择 [默认: {}]: ", default + 1);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();

            if input.is_empty() {
                return Ok(default);
            }

            match input.parse::<usize>() {
                Ok(n) if n >= 1 && n <= options.len() => return Ok(n - 1),
                _ => {
                    println!("请输入 1 到 {} 之间的数字", options.len());
                }
            }
        }
    }
}
