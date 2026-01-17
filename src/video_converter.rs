use anyhow::{Context, Result, anyhow};
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
        let input_str = input_path.to_str()
            .ok_or_else(|| anyhow!("输入路径包含无效UTF-8字符: {:?}", input_path))?;
        let output_str = output_path.to_str()
            .ok_or_else(|| anyhow!("输出路径包含无效UTF-8字符: {:?}", output_path))?;
        
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(&[
            "-i", input_str,
            "-c:v", "libtheora",  // 视频编码器：Theora
            "-q:v", &video_quality.to_string(),  // 视频质量（动态设置）
            "-speed", "8",        // 编码速度优化（0-10，8为最快速度）
            "-threads", "0",      // 使用所有可用CPU核心
            "-c:a", "libvorbis",  // 音频编码器：Vorbis
            "-q:a", &audio_quality.to_string(),  // 音频质量（动态设置）
            "-ac", "2",           // 立体声音频，减少处理时间
            "-y",                 // 覆盖输出文件
            output_str
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
        
        let input_str = input_path.to_str()
            .ok_or_else(|| anyhow!("输入路径包含无效UTF-8字符: {:?}", input_path))?;
        
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(&[
            "-i", input_str,
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
        debug!("开始解析FFmpeg输出:\n{}", output);
        
        let mut duration = 0u32;
        let mut resolution = (0u32, 0u32);
        
        // 解析时长 (Duration: HH:MM:SS.mmm)
        if let Some(duration_line) = output.lines().find(|line| line.contains("Duration:")) {
            debug!("找到时长信息行: {}", duration_line);
            if let Some(duration_str) = duration_line.split("Duration:").nth(1) {
                if let Some(duration_part) = duration_str.split(',').next() {
                    duration = self.parse_duration(duration_part.trim());
                    debug!("解析到时长: {} 秒", duration);
                }
            }
        }
        
        // 解析分辨率 (Stream #0:0: Video: ... 1920x1080 ...)
        // 查找包含 "Video:" 的行，然后尝试提取分辨率
        for line in output.lines() {
            if line.contains("Video:") {
                debug!("找到视频流信息行: {}", line);
                if let Some(resolution_part) = self.extract_resolution(line) {
                    resolution = resolution_part;
                    debug!("成功解析分辨率: {}x{}", resolution.0, resolution.1);
                    break;
                }
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
        debug!("解析分辨率字符串: {}", stream_line);
        
        // 使用正则表达式查找 "数字x数字" 格式的分辨率
        // 查找所有可能的 "1234x5678" 格式
        for (i, c) in stream_line.char_indices() {
            if c.is_ascii_digit() {
                // 找到数字，尝试提取分辨率
                let remaining = &stream_line[i..];
                
                // 查找 'x' 字符
                if let Some(x_pos) = remaining.find('x') {
                    let width_part = &remaining[..x_pos];
                    let after_x = &remaining[x_pos + 1..];
                    
                    // 提取宽度
                    if let Ok(width) = width_part.parse::<u32>() {
                        // 提取高度（到下一个非数字字符为止）
                        let mut height_end = 0;
                        for (j, c) in after_x.char_indices() {
                            if !c.is_ascii_digit() {
                                height_end = j;
                                break;
                            }
                            height_end = j + 1;
                        }
                        
                        if height_end > 0 {
                            let height_part = &after_x[..height_end];
                            if let Ok(height) = height_part.parse::<u32>() {
                                if width > 0 && height > 0 {
                                    debug!("解析到分辨率: {}x{}", width, height);
                                    return Some((width, height));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        debug!("未能解析到分辨率");
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

