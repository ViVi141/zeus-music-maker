/*!
 * 视频分片转换模块
 * 支持将大视频文件分割成多个片段进行并行转换，提高多线程利用率
 */

use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use log::{info, error, debug, warn};
use serde::{Serialize, Deserialize};
use std::fs;

use crate::ffmpeg_plugin::FFmpegPlugin;
use crate::video_converter::VideoInfo;

/// 视频分片配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoChunkConfig {
    /// 每个分片的时长（秒）
    pub chunk_duration: u32,
    /// 分片重叠时长（秒）- 避免音视频不同步
    pub overlap_duration: u32,
    /// 最大分片数量
    pub max_chunks: usize,
    /// 是否启用智能分片（根据视频时长自动调整）
    pub smart_chunking: bool,
    /// 最小分片时长（秒）
    pub min_chunk_duration: u32,
    /// 快速转换模式（针对短视频优化）
    pub fast_mode: bool,
}

impl Default for VideoChunkConfig {
    fn default() -> Self {
        Self {
            chunk_duration: 60,      // 默认60秒一个分片
            overlap_duration: 2,     // 2秒重叠
            max_chunks: 16,          // 最多16个分片
            smart_chunking: true,    // 启用智能分片
            min_chunk_duration: 30,  // 最小30秒
            fast_mode: false,        // 默认不启用快速模式
        }
    }
}

impl VideoChunkConfig {
    /// 根据视频信息智能调整分片配置
    pub fn adjust_for_video(&mut self, video_info: &VideoInfo) {
        if !self.smart_chunking {
            return;
        }

        let duration = video_info.duration;
        
        // 根据视频时长调整分片策略
        if duration <= 120 {
            // 短视频（≤2分钟）：不分片，启用快速模式
            self.chunk_duration = duration;
            self.max_chunks = 1;
            self.fast_mode = true;
        } else if duration <= 600 {
            // 中等视频（2-10分钟）：2-4个分片
            self.chunk_duration = duration / 3;
            self.max_chunks = 3;
        } else if duration <= 1800 {
            // 长视频（10-30分钟）：4-8个分片
            self.chunk_duration = duration / 6;
            self.max_chunks = 6;
        } else {
            // 超长视频（>30分钟）：8-16个分片
            self.chunk_duration = duration / 12;
            self.max_chunks = 12.min(self.max_chunks);
        }

        // 确保分片时长不小于最小值
        self.chunk_duration = self.chunk_duration.max(self.min_chunk_duration);
        
        info!("视频分片配置调整: 时长={}s, 分片时长={}s, 最大分片数={}", 
              duration, self.chunk_duration, self.max_chunks);
    }

    /// 计算实际分片数量
    pub fn calculate_chunk_count(&self, video_duration: u32) -> usize {
        if video_duration <= self.chunk_duration {
            return 1;
        }

        let chunks = (video_duration as f32 / self.chunk_duration as f32).ceil() as usize;
        chunks.min(self.max_chunks)
    }
}

/// 视频分片信息
#[derive(Debug, Clone)]
pub struct VideoChunk {
    /// 分片索引
    pub index: usize,
    /// 输入文件路径
    pub input_path: PathBuf,
    /// 分片开始时间（秒）
    pub start_time: u32,
    /// 分片时长（秒）
    pub duration: u32,
    /// 输出文件路径
    pub output_path: PathBuf,
}

/// 视频分片转换器
pub struct VideoChunkConverter {
    pub ffmpeg_path: PathBuf,
    config: VideoChunkConfig,
}

impl VideoChunkConverter {
    /// 创建新的分片转换器
    pub fn new(config: VideoChunkConfig) -> Result<Self> {
        let plugin = FFmpegPlugin::new()?;
        let ffmpeg_path = plugin.get_ffmpeg_path()
            .ok_or_else(|| anyhow::anyhow!("FFmpeg 未找到"))?;

        Ok(Self {
            ffmpeg_path,
            config,
        })
    }

    /// 分析视频并生成分片计划
    pub fn create_chunk_plan(&self, input_path: &Path, output_dir: &Path) -> Result<Vec<VideoChunk>> {
        info!("创建视频分片计划: {:?}", input_path);
        
        // 获取视频信息
        let video_info = self.get_video_info(input_path)?;
        
        // 调整配置
        let mut config = self.config.clone();
        config.adjust_for_video(&video_info);
        
        // 计算分片数量
        let chunk_count = config.calculate_chunk_count(video_info.duration);
        
        if chunk_count == 1 {
            // 不需要分片，直接返回单个分片
            let output_path = self.get_output_path(input_path, output_dir, 0)?;
            return Ok(vec![VideoChunk {
                index: 0,
                input_path: input_path.to_path_buf(),
                start_time: 0,
                duration: video_info.duration,
                output_path,
            }]);
        }

        // 计算实际分片时长（考虑重叠）
        let effective_chunk_duration = if chunk_count > 1 {
            (video_info.duration as f32 / chunk_count as f32).ceil() as u32
        } else {
            video_info.duration
        };

        let mut chunks = Vec::new();
        
        for i in 0..chunk_count {
            let start_time = if i == 0 {
                0
            } else {
                i as u32 * effective_chunk_duration - config.overlap_duration
            };
            
            let duration = if i == chunk_count - 1 {
                // 最后一个分片包含到视频结尾
                video_info.duration - start_time
            } else {
                effective_chunk_duration + config.overlap_duration
            };

            let output_path = self.get_output_path(input_path, output_dir, i)?;
            
            chunks.push(VideoChunk {
                index: i,
                input_path: input_path.to_path_buf(),
                start_time,
                duration,
                output_path,
            });
        }

        info!("生成分片计划: {} 个分片", chunks.len());
        Ok(chunks)
    }

    /// 转换单个分片
    pub fn convert_chunk(&self, chunk: &VideoChunk, video_quality: u8, audio_quality: u8) -> Result<()> {
        info!("转换分片 {}: {}s-{}s ({})", 
              chunk.index, chunk.start_time, chunk.start_time + chunk.duration, 
              chunk.input_path.display());

        // 确保输出目录存在
        if let Some(parent) = chunk.output_path.parent() {
            fs::create_dir_all(parent)
                .context("创建输出目录失败")?;
        }

        // 构建FFmpeg命令
        let mut cmd = Command::new(&self.ffmpeg_path);
        
        let input_str = chunk.input_path.to_str()
            .ok_or_else(|| anyhow!("分片输入路径包含无效UTF-8字符: {:?}", chunk.input_path))?;
        let output_str = chunk.output_path.to_str()
            .ok_or_else(|| anyhow!("分片输出路径包含无效UTF-8字符: {:?}", chunk.output_path))?;
        
        if self.config.fast_mode {
            // 快速模式：针对短视频优化
            cmd.args(&[
                "-i", input_str,
                "-ss", &chunk.start_time.to_string(),
                "-t", &chunk.duration.to_string(),
                "-c:v", "libtheora",
                "-q:v", "6",                       // 固定质量，避免计算开销
                "-speed", "8",
                "-threads", "0",
                "-c:a", "libvorbis",
                "-q:a", "6",                       // 固定质量
                "-ac", "2",
                "-avoid_negative_ts", "make_zero",
                "-preset", "ultrafast",
                "-tune", "fastdecode",
                "-movflags", "+faststart",
                "-no-scenecut", "1",               // 禁用场景检测
                "-g", "30",                        // 固定关键帧间隔
                "-y",
                output_str
            ]);
        } else {
            // 标准模式
            cmd.args(&[
                "-i", input_str,
                "-ss", &chunk.start_time.to_string(),
                "-t", &chunk.duration.to_string(),
                "-c:v", "libtheora",
                "-q:v", &video_quality.to_string(),
                "-speed", "8",
                "-threads", "0",
                "-c:a", "libvorbis",
                "-q:a", &audio_quality.to_string(),
                "-ac", "2",
                "-avoid_negative_ts", "make_zero",
                "-preset", "ultrafast",
                "-tune", "fastdecode",
                "-movflags", "+faststart",
                "-y",
                output_str
            ]);
        }

        // 在Windows上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        debug!("执行分片转换命令: {:?}", cmd);
        
        if self.config.fast_mode {
            info!("使用快速模式转换短视频分片: {} ({}秒)", 
                  chunk.input_path.display(), chunk.duration);
        }

        // 执行转换
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("启动FFmpeg进程失败")?;

        // 设置进程优先级
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
            .context("等待FFmpeg进程完成失败")?;

        if output.status.success() {
            info!("分片转换成功: {:?}", chunk.output_path);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("分片转换失败: {}", error_msg);
            Err(anyhow::anyhow!("分片转换失败: {}", error_msg))
        }
    }

    /// 合并分片为完整视频
    pub fn merge_chunks(&self, chunks: &[VideoChunk], output_path: &Path) -> Result<()> {
        if chunks.len() == 1 {
            // 只有一个分片，直接复制
            fs::copy(&chunks[0].output_path, output_path)
                .context("复制单个分片失败")?;
            return Ok(());
        }

        info!("合并 {} 个分片为完整视频", chunks.len());

        // 创建文件列表
        let file_list_path = output_path.with_extension("filelist.txt");
        let mut file_list = String::new();
        
        for chunk in chunks {
            file_list.push_str(&format!("file '{}'\n", chunk.output_path.display()));
        }
        
        fs::write(&file_list_path, file_list)
            .context("创建文件列表失败")?;

        // 构建合并命令
        let file_list_str = file_list_path.to_str()
            .ok_or_else(|| anyhow!("文件列表路径包含无效UTF-8字符: {:?}", file_list_path))?;
        let output_str = output_path.to_str()
            .ok_or_else(|| anyhow!("输出路径包含无效UTF-8字符: {:?}", output_path))?;
        
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(&[
            "-f", "concat",
            "-safe", "0",
            "-i", file_list_str,
            "-c", "copy",  // 直接复制，不重新编码
            "-y",
            output_str
        ]);

        // 在Windows上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        debug!("执行合并命令: {:?}", cmd);

        // 执行合并
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("启动合并进程失败")?;

        let output = child
            .wait_with_output()
            .context("等待合并进程完成失败")?;

        // 清理临时文件
        let _ = fs::remove_file(&file_list_path);

        if output.status.success() {
            info!("分片合并成功: {:?}", output_path);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("分片合并失败: {}", error_msg);
            Err(anyhow::anyhow!("分片合并失败: {}", error_msg))
        }
    }

    /// 获取视频信息
    fn get_video_info(&self, input_path: &Path) -> Result<VideoInfo> {
        let input_str = input_path.to_str()
            .ok_or_else(|| anyhow!("输入路径包含无效UTF-8字符: {:?}", input_path))?;
        
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(&[
            "-i", input_str,
            "-f", "null",
            "-"
        ]);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        let output = cmd
            .stderr(Stdio::piped())
            .output()
            .context("执行FFmpeg信息获取失败")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        self.parse_video_info(&stderr)
    }

    /// 解析视频信息
    fn parse_video_info(&self, output: &str) -> Result<VideoInfo> {
        let mut duration = 0u32;
        let mut resolution = (0u32, 0u32);

        // 解析时长
        if let Some(duration_line) = output.lines().find(|line| line.contains("Duration:")) {
            if let Some(duration_str) = duration_line.split("Duration:").nth(1) {
                if let Some(duration_part) = duration_str.split(',').next() {
                    duration = self.parse_duration(duration_part.trim());
                }
            }
        }

        // 解析分辨率
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

    /// 解析时长字符串
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

    /// 生成输出文件路径
    fn get_output_path(&self, input_path: &Path, output_dir: &Path, chunk_index: usize) -> Result<PathBuf> {
        let file_stem = input_path.file_stem()
            .ok_or_else(|| anyhow::anyhow!("无效的文件名"))?
            .to_string_lossy();
        
        // 使用安全文件名并确保分片索引正确格式化
        let safe_filename = crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&file_stem);
        let chunk_filename = format!("{}_chunk_{:03}.ogv", safe_filename, chunk_index);
        
        let mut output_path = output_dir.join(chunk_filename);
        // 确保路径长度在限制内
        output_path = crate::utils::string_utils::StringUtils::ensure_path_length(&output_path, 260)
            .unwrap_or_else(|_| output_path.clone());
        // 确保文件名唯一（分片文件名通常不会冲突，但为了安全还是检查）
        output_path = crate::utils::string_utils::StringUtils::ensure_unique_path(output_path);
        
        Ok(output_path)
    }

    /// 清理临时分片文件
    pub fn cleanup_chunks(&self, chunks: &[VideoChunk]) {
        for chunk in chunks {
            if chunk.output_path.exists() {
                if let Err(e) = fs::remove_file(&chunk.output_path) {
                    warn!("清理临时分片文件失败: {} - {}", chunk.output_path.display(), e);
                } else {
                    debug!("已清理临时分片文件: {}", chunk.output_path.display());
                }
            }
        }
    }
}

/// 视频分片转换结果
#[derive(Debug, Clone)]
pub struct VideoChunkConversionResult {
    /// 最终输出文件
    pub output_path: PathBuf,
    /// 分片信息
    pub chunks: Vec<VideoChunk>,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl VideoChunkConversionResult {

    /// 获取成功消息
    pub fn get_success_message(&self) -> String {
        if self.chunks.len() == 1 {
            format!("视频转换成功: {}", self.output_path.display())
        } else {
            format!("视频分片转换成功: {} ({}个分片)", 
                   self.output_path.display(), self.chunks.len())
        }
    }

    /// 获取错误消息
    pub fn get_error_message(&self) -> String {
        self.error.clone().unwrap_or_else(|| "未知错误".to_string())
    }
}
