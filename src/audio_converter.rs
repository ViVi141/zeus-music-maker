use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use log::{info, error};
use crate::ffmpeg_downloader::FFmpegDownloader;

/// FFmpeg 音频转换器
pub struct AudioConverter {
    ffmpeg_path: PathBuf,
}

impl AudioConverter {
    /// 创建新的音频转换器实例
    pub fn new() -> Result<Self> {
        Self::new_with_download(false)
    }
    
    /// 创建新的音频转换器实例（可选择是否自动下载）
    pub fn new_with_download(auto_download: bool) -> Result<Self> {
        // 首先尝试从配置文件加载路径
        if let Some(path) = crate::ffmpeg_downloader::FFmpegDownloader::load_ffmpeg_path() {
            info!("从配置文件加载 FFmpeg: {:?}", path);
            return Ok(Self { ffmpeg_path: path });
        }
        
        // 然后尝试自动查找
        if let Some(path) = crate::ffmpeg_downloader::FFmpegDownloader::find_ffmpeg_path() {
            // 保存找到的路径到配置文件
            let _ = crate::ffmpeg_downloader::FFmpegDownloader::save_ffmpeg_path(&path);
            return Ok(Self { ffmpeg_path: path });
        }
        
        if auto_download {
            // 尝试自动下载
            return Self::download_and_create();
        }
        
        Err(anyhow::anyhow!("FFmpeg 未找到。请选择：\n1. 使用自动下载功能\n2. 手动安装 FFmpeg 到系统 PATH\n3. 手动选择 FFmpeg 路径"))
    }
    
    /// 下载并创建音频转换器
    fn download_and_create() -> Result<Self> {
        // 使用用户工作空间下载
        let _downloader = FFmpegDownloader::new_user_workspace()?;
        
        // 这里我们需要异步运行时，但当前函数是同步的
        // 我们需要在调用处处理异步下载
        Err(anyhow::anyhow!("需要异步下载 FFmpeg，请使用 download_ffmpeg_if_needed 方法"))
    }
    
    
    
    
    
    /// 将音频文件转换为 OGG 格式（支持取消检查）
    pub fn convert_to_ogg_with_cancel<F>(
        &self,
        input_path: &Path,
        output_path: &Path,
        should_cancel: &F,
    ) -> Result<String>
    where
        F: Fn() -> bool,
    {
        // 检查取消标志
        if should_cancel() {
            return Err(anyhow::anyhow!("转换任务被取消"));
        }
        
        info!("开始转换: {:?} -> {:?}", input_path, output_path);
        
        // 检查输入文件是否存在
        if !input_path.exists() {
            return Err(anyhow::anyhow!("输入文件不存在: {:?}", input_path));
        }
        
        // 创建输出目录
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 构建 FFmpeg 命令
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-i", input_path.to_str().unwrap(),
            "-c:a", "libvorbis",  // 使用 Vorbis 编码器
            "-q:a", "5",          // 质量设置 (0-10, 5 是平衡点)
            "-y",                 // 覆盖输出文件
            output_path.to_str().unwrap(),
        ]);
        
        // 执行转换
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // 在 Windows 上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            child = child.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let mut child = child.spawn().context("启动 FFmpeg 失败")?;
        
        // 等待完成并检查取消
        let result = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {
                    // 进程还在运行，检查是否应该取消
                    if should_cancel() {
                        // 尝试终止进程
                        let _ = child.kill();
                        return Err(anyhow::anyhow!("转换任务被取消"));
                    }
                    // 短暂等待
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => break Err(e),
            }
        };
        
        let status = result.context("FFmpeg 执行失败")?;
        
        if status.success() {
            info!("转换成功: {:?}", output_path);
            Ok("转换成功".to_string())
        } else {
            // 获取错误输出
            let error_msg = if let Ok(output) = child.wait_with_output() {
                String::from_utf8_lossy(&output.stderr).to_string()
            } else {
                "FFmpeg execution failed".to_string()
            };
            error!("FFmpeg 转换失败: {}", error_msg);
            Err(anyhow::anyhow!("FFmpeg 转换失败: {}", error_msg))
        }
    }
    
    
    
}

impl Default for AudioConverter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("无法创建 AudioConverter: {}", e);
            // 返回一个无效的实例，会在使用时失败
            Self {
                ffmpeg_path: PathBuf::from("ffmpeg_not_found"),
            }
        })
    }
}

