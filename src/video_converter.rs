use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use log::{info, error, debug};
use crate::ffmpeg_plugin::FFmpegPlugin;

/// 视频转换器
pub struct VideoConverter {
    pub ffmpeg_path: PathBuf,
}

impl VideoConverter {
    /// 创建新的视频转换器实例
    pub fn new() -> Result<Self> {
        Self::new_with_plugin(&FFmpegPlugin::new()?)
    }
    
    /// 使用FFmpeg插件创建视频转换器实例
    pub fn new_with_plugin(plugin: &FFmpegPlugin) -> Result<Self> {
        if let Some(path) = plugin.get_ffmpeg_path() {
            info!("使用FFmpeg插件找到路径: {:?}", path);
            Ok(Self { ffmpeg_path: path })
        } else {
            Err(anyhow::anyhow!("FFmpeg 未找到。请选择：\n1. 使用自动下载功能\n2. 手动安装 FFmpeg 到系统 PATH\n3. 手动选择 FFmpeg 路径"))
        }
    }
    
    
    /// 转换视频文件为 OGV 格式（标准模式）
    pub fn convert_to_ogv(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.convert_to_ogv_with_quality(input_path, output_path, 5, 3)
    }


    /// 转换视频文件为 OGV 格式（自定义质量）
    fn convert_to_ogv_with_quality(&self, input_path: &Path, output_path: &Path, video_quality: u8, audio_quality: u8) -> Result<()> {
        info!("开始转换视频: {:?} -> {:?}", input_path, output_path);
        
        // 确保输出目录存在
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .context("创建输出目录失败")?;
        }
        
        // 构建 FFmpeg 命令
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(&[
            "-i", input_path.to_str().unwrap(),
            "-c:v", "libtheora",  // 视频编码器：Theora
            "-q:v", &video_quality.to_string(),  // 视频质量（动态设置）
            "-speed", "8",        // 编码速度优化（0-10，8为最快速度）
            "-threads", "0",      // 使用所有可用CPU核心
            "-c:a", "libvorbis",  // 音频编码器：Vorbis
            "-q:a", &audio_quality.to_string(),  // 音频质量（动态设置）
            "-ac", "2",           // 立体声音频，减少处理时间
            "-y",                 // 覆盖输出文件
            output_path.to_str().unwrap()
        ]);
        
        // 在 Windows 上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        debug!("执行 FFmpeg 命令: {:?}", cmd);
        
        // 执行转换
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("启动 FFmpeg 进程失败")?;
            
        // 设置进程优先级为高优先级（Windows）
        #[cfg(target_os = "windows")]
        {
            let handle = child.id();
            unsafe {
                use winapi::um::processthreadsapi::SetPriorityClass;
                use winapi::um::winbase::HIGH_PRIORITY_CLASS;
                SetPriorityClass(handle as _, HIGH_PRIORITY_CLASS);
            }
        }
        
        let output = child
            .wait_with_output()
            .context("等待 FFmpeg 进程完成失败")?;
        
        if output.status.success() {
            info!("视频转换成功: {:?}", output_path);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("视频转换失败: {}", error_msg);
            Err(anyhow::anyhow!("视频转换失败: {}", error_msg))
        }
    }
    
    /// 获取视频信息
    pub fn get_video_info(&self, input_path: &Path) -> Result<VideoInfo> {
        info!("获取视频信息: {:?}", input_path);
        
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(&[
            "-i", input_path.to_str().unwrap(),
            "-f", "null",
            "-"
        ]);
        
        // 在 Windows 上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        // 执行命令并捕获输出
        let output = cmd
            .stderr(Stdio::piped())
            .output()
            .context("执行 FFmpeg 信息获取失败")?;
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("FFmpeg 输出: {}", stderr);
        
        // 解析视频信息
        self.parse_video_info(&stderr)
    }
    
    /// 解析 FFmpeg 输出的视频信息
    fn parse_video_info(&self, output: &str) -> Result<VideoInfo> {
        let mut duration = 0u32;
        let mut resolution = (0u32, 0u32);
        
        // 解析时长 (Duration: HH:MM:SS.mmm)
        if let Some(duration_line) = output.lines().find(|line| line.contains("Duration:")) {
            if let Some(duration_str) = duration_line.split("Duration:").nth(1) {
                if let Some(duration_part) = duration_str.split(',').next() {
                    duration = self.parse_duration(duration_part.trim());
                }
            }
        }
        
        // 解析分辨率 (Stream #0:0: Video: ... 1920x1080 ...)
        if let Some(stream_line) = output.lines().find(|line| line.contains("Video:") && line.contains("x")) {
            if let Some(resolution_part) = self.extract_resolution(stream_line) {
                resolution = resolution_part;
            }
        }
        
        Ok(VideoInfo {
            duration,
            resolution,
        })
    }
    
    /// 解析时长字符串 (HH:MM:SS.mmm)
    fn parse_duration(&self, duration_str: &str) -> u32 {
        let parts: Vec<&str> = duration_str.split(':').collect();
        if parts.len() == 3 {
            let hours: u32 = parts[0].parse().unwrap_or(0);
            let minutes: u32 = parts[1].parse().unwrap_or(0);
            let seconds_part = parts[2];
            
            let seconds: u32 = if let Some(dot_pos) = seconds_part.find('.') {
                seconds_part[..dot_pos].parse().unwrap_or(0)
            } else {
                seconds_part.parse().unwrap_or(0)
            };
            
            hours * 3600 + minutes * 60 + seconds
        } else {
            0
        }
    }
    
    /// 从流信息中提取分辨率
    fn extract_resolution(&self, stream_line: &str) -> Option<(u32, u32)> {
        // 查找 "1234x5678" 格式的分辨率
        if let Some(start) = stream_line.find(|c: char| c.is_ascii_digit()) {
            let resolution_part = &stream_line[start..];
            if let Some(end) = resolution_part.find(' ') {
                let resolution_str = &resolution_part[..end];
                if let Some(x_pos) = resolution_str.find('x') {
                    let width_str = &resolution_str[..x_pos];
                    let height_str = &resolution_str[x_pos + 1..];
                    
                    if let (Ok(width), Ok(height)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
                        return Some((width, height));
                    }
                }
            }
        }
        None
    }
    
    
    /// 检查输入文件是否为支持的视频格式
    pub fn is_supported_video_format(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), 
                "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "m4v" | "3gp" | "ogv"
            )
        } else {
            false
        }
    }
}

/// 视频信息
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub duration: u32,        // 时长（秒）
    pub resolution: (u32, u32), // 分辨率 (宽度, 高度)
}

impl VideoInfo {
    pub fn new() -> Self {
        Self {
            duration: 0,
            resolution: (0, 0),
        }
    }
    
}

impl Default for VideoInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_parse_duration() {
        let converter = VideoConverter::new_with_path(PathBuf::from("ffmpeg")).unwrap_or_else(|_| {
            // 如果无法创建真实的转换器，创建一个模拟的用于测试
            VideoConverter { ffmpeg_path: PathBuf::from("ffmpeg") }
        });
        
        assert_eq!(converter.parse_duration("01:30:45"), 5445); // 1小时30分45秒
        assert_eq!(converter.parse_duration("02:15:30.500"), 8130); // 2小时15分30秒
        assert_eq!(converter.parse_duration("00:05:20"), 320); // 5分20秒
    }
    
    #[test]
    fn test_extract_resolution() {
        let converter = VideoConverter::new_with_path(PathBuf::from("ffmpeg")).unwrap_or_else(|_| {
            VideoConverter { ffmpeg_path: PathBuf::from("ffmpeg") }
        });
        
        let stream_line = "Stream #0:0: Video: h264, yuv420p, 1920x1080, 25 fps";
        assert_eq!(converter.extract_resolution(stream_line), Some((1920, 1080)));
        
        let stream_line2 = "Stream #0:0: Video: mpeg4, yuv420p, 1280x720, 30 fps";
        assert_eq!(converter.extract_resolution(stream_line2), Some((1280, 720)));
    }
    
    #[test]
    fn test_is_supported_video_format() {
        let converter = VideoConverter::new_with_path(PathBuf::from("ffmpeg")).unwrap_or_else(|_| {
            VideoConverter { ffmpeg_path: PathBuf::from("ffmpeg") }
        });
        
        assert!(converter.is_supported_video_format(&PathBuf::from("test.mp4")));
        assert!(converter.is_supported_video_format(&PathBuf::from("test.avi")));
        assert!(converter.is_supported_video_format(&PathBuf::from("test.ogv")));
        assert!(!converter.is_supported_video_format(&PathBuf::from("test.txt")));
        assert!(!converter.is_supported_video_format(&PathBuf::from("test")));
    }
}
