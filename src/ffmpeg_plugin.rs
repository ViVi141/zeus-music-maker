/*!
 * FFmpeg 插件模块
 * 提供独立的FFmpeg下载、检查和路径管理功能
 */

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;
use log::{info, warn, debug};
use serde::{Serialize, Deserialize};
use std::fs;

/// FFmpeg插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFmpegConfig {
    /// FFmpeg可执行文件路径
    pub ffmpeg_path: Option<PathBuf>,
    /// 配置文件路径
    pub config_path: PathBuf,
    /// 是否自动下载
    pub auto_download: bool,
    /// 下载镜像源
    pub mirror_source: MirrorSource,
}

/// 镜像源枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorSource {
    GitHub,
    GitHubProxy,
    Custom(String),
}

impl Default for MirrorSource {
    fn default() -> Self {
        MirrorSource::GitHub
    }
}

/// FFmpeg插件
pub struct FFmpegPlugin {
    config: FFmpegConfig,
}

impl FFmpegPlugin {
    /// 创建新的FFmpeg插件实例
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_config(&config_path)?;
        Ok(Self { config })
    }


    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("无法获取配置目录"))?
            .join("zeus_music_maker");
        
        // 确保配置目录存在
        fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("ffmpeg_config.json"))
    }

    /// 加载配置
    fn load_config(config_path: &Path) -> Result<FFmpegConfig> {
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            match serde_json::from_str::<FFmpegConfig>(&content) {
                Ok(mut config) => {
                    config.config_path = config_path.to_path_buf();
                    Ok(config)
                }
                Err(e) => {
                    warn!("配置文件格式错误，使用默认配置: {}", e);
                    Ok(FFmpegConfig::default(config_path.to_path_buf()))
                }
            }
        } else {
            Ok(FFmpegConfig::default(config_path.to_path_buf()))
        }
    }

    /// 保存配置
    fn save_config(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config.config_path, content)?;
        Ok(())
    }

    /// 检查FFmpeg是否可用
    pub fn check_ffmpeg_available(&self) -> bool {
        if let Some(ref path) = self.config.ffmpeg_path {
            self.test_ffmpeg_executable(path).is_ok()
        } else {
            // 尝试从PATH查找
            self.find_ffmpeg_in_path().is_some()
        }
    }

    /// 测试FFmpeg可执行文件
    fn test_ffmpeg_executable(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("FFmpeg可执行文件不存在: {:?}", path));
        }

        let mut cmd = Command::new(path);
        cmd.arg("-version")
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());
           
        // 在 Windows 上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let output = cmd.output()
            .map_err(|e| anyhow!("无法执行FFmpeg: {}", e))?;

        if output.status.success() {
            debug!("FFmpeg测试成功: {:?}", path);
            Ok(())
        } else {
            Err(anyhow!("FFmpeg测试失败: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// 从PATH环境变量中查找FFmpeg
    fn find_ffmpeg_in_path(&self) -> Option<PathBuf> {
        let ffmpeg_names = if cfg!(windows) {
            vec!["ffmpeg.exe", "ffmpeg"]
        } else {
            vec!["ffmpeg"]
        };

        for name in ffmpeg_names {
            if let Ok(path) = which::which(name) {
                if self.test_ffmpeg_executable(&path).is_ok() {
                    return Some(path);
                }
            }
        }
        None
    }

    /// 获取FFmpeg路径
    pub fn get_ffmpeg_path(&self) -> Option<PathBuf> {
        if let Some(ref path) = self.config.ffmpeg_path {
            // 如果配置的路径存在，直接返回，避免重复测试
            if path.exists() {
                return Some(path.clone());
            }
        }

        // 尝试从PATH查找
        self.find_ffmpeg_in_path()
    }

    /// 设置FFmpeg路径
    pub fn set_ffmpeg_path(&mut self, path: PathBuf) -> Result<()> {
        self.test_ffmpeg_executable(&path)?;
        self.config.ffmpeg_path = Some(path);
        self.save_config()?;
        info!("FFmpeg路径已设置并保存");
        Ok(())
    }


    /// 获取FFmpeg版本信息
    pub fn get_ffmpeg_version(&self) -> Result<String> {
        let path = self.get_ffmpeg_path()
            .ok_or_else(|| anyhow!("FFmpeg未找到"))?;

        let mut cmd = Command::new(&path);
        cmd.arg("-version")
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());
           
        // 在 Windows 上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let output = cmd.output()
            .map_err(|e| anyhow!("无法获取FFmpeg版本: {}", e))?;

        if output.status.success() {
            let version_text = String::from_utf8_lossy(&output.stdout);
            // 提取版本号（第一行）
            if let Some(first_line) = version_text.lines().next() {
                Ok(first_line.to_string())
            } else {
                Ok("未知版本".to_string())
            }
        } else {
            Err(anyhow!("获取FFmpeg版本失败"))
        }
    }



    /// 重置配置为默认值
    pub fn reset_config(&mut self) -> Result<()> {
        self.config = FFmpegConfig::default(self.config.config_path.clone());
        self.save_config()?;
        Ok(())
    }

}

impl Default for FFmpegConfig {
    fn default() -> Self {
        Self::default(PathBuf::from("config.json"))
    }
}

impl FFmpegConfig {
    pub fn default(config_path: PathBuf) -> Self {
        Self {
            ffmpeg_path: None,
            config_path,
            auto_download: true,
            mirror_source: MirrorSource::default(),
        }
    }
}

/// FFmpeg状态信息
#[derive(Debug, Clone)]
pub struct FFmpegStatus {
    pub available: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub config_path: PathBuf,
}

impl FFmpegPlugin {
    /// 获取FFmpeg状态信息
    pub fn get_status(&self) -> FFmpegStatus {
        let available = self.check_ffmpeg_available();
        let path = if available {
            self.get_ffmpeg_path()
        } else {
            None
        };
        let version = if available {
            self.get_ffmpeg_version().ok()
        } else {
            None
        };

        FFmpegStatus {
            available,
            path,
            version,
            config_path: self.config.config_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_config_default() {
        let config = FFmpegConfig::default(PathBuf::from("test.json"));
        assert_eq!(config.auto_download, true);
        assert_eq!(config.mirror_source, MirrorSource::GitHub);
        assert!(config.ffmpeg_path.is_none());
    }

    #[test]
    fn test_mirror_source_enum() {
        assert_eq!(MirrorSource::default(), MirrorSource::GitHub);
        assert_eq!(MirrorSource::GitHub, MirrorSource::GitHub);
        assert_eq!(MirrorSource::GitHubProxy, MirrorSource::GitHubProxy);
        assert_eq!(MirrorSource::Custom("test".to_string()), MirrorSource::Custom("test".to_string()));
    }
}
