//! 音频播放模块
//!
//! 支持播放系统提示音和自定义 WAV 文件

use anyhow::Result;
use std::path::Path;

/// 音频播放器
pub struct SoundPlayer {
    enabled: bool,
}

impl SoundPlayer {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 播放系统提示音
    pub fn play_system_sound(&self, sound_type: SystemSound) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(windows)]
        {
            self.play_windows_system_sound(sound_type)?;
        }

        #[cfg(not(windows))]
        {
            log::info!("系统提示音: {:?}", sound_type);
        }

        Ok(())
    }

    /// 播放自定义音频文件
    pub fn play_sound_file(&self, file_path: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let path = Path::new(file_path);

        if !path.exists() {
            log::warn!("音频文件不存在: {:?}", file_path);
            return Ok(());
        }

        // 检查是否启用了 sound 功能
        #[cfg(feature = "sound")]
        {
            self.play_with_rodio(path)?;
        }

        #[cfg(not(feature = "sound"))]
        {
            log::info!("播放音频文件（未启用 sound 功能）: {:?}", file_path);
        }

        Ok(())
    }

    #[cfg(windows)]
    fn play_windows_system_sound(&self, sound_type: SystemSound) -> Result<()> {
        // Windows 系统音效播放需要额外的 Windows API
        // 暂时记录日志，后续可以通过添加更多 Windows features 实现
        log::info!("播放系统提示音: {:?}", sound_type);
        Ok(())
    }

    #[cfg(feature = "sound")]
    fn play_with_rodio(&self, path: &Path) -> Result<()> {
        use rodio::{Decoder, OutputStream};
        use std::fs::File;
        use std::io::BufReader;

        // 获取输出流
        let (_stream, stream_handle) = OutputStream::try_default()?;

        // 打开音频文件
        let file = File::open(path)?;
        let source = BufReader::new(file);

        // 解码音频
        let decoder = rodio::Decoder::new(source)?;

        // 播放
        rodio::play_source(decoder, &stream_handle)?;
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(())
    }
}

/// 系统提示音类型
#[derive(Debug, Clone, Copy)]
pub enum SystemSound {
    Success,
    Error,
    Notification,
}
