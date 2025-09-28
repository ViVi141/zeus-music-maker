use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use log::{info, error};
use crate::ffmpeg_plugin::FFmpegPlugin;

/// FFmpeg 音频转换器
pub struct AudioConverter {
    pub ffmpeg_path: PathBuf,
}

impl AudioConverter {
    /// 创建新的音频转换器实例
    pub fn new() -> Result<Self> {
        Self::new_with_plugin(&FFmpegPlugin::new()?)
    }
    
    /// 使用FFmpeg插件创建音频转换器实例
    pub fn new_with_plugin(plugin: &FFmpegPlugin) -> Result<Self> {
        if let Some(path) = plugin.get_ffmpeg_path() {
            info!("使用FFmpeg插件找到路径: {:?}", path);
            Ok(Self { ffmpeg_path: path })
        } else {
            Err(anyhow::anyhow!("FFmpeg 未找到。请选择：\n1. 使用自动下载功能\n2. 手动安装 FFmpeg 到系统 PATH\n3. 手动选择 FFmpeg 路径"))
        }
    }
    
    
    
    
    
    
    /// 将音频文件转换为 OGG 格式（支持取消检查）
    pub fn convert_to_ogg_with_cancel<F>(
        &self,
        input_path: &Path,
        output_path: &Path,
        should_cancel: &F,
    ) -> Result<String>
    where
        F: Fn() -> bool + ?Sized,
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
        let input_str = input_path.to_str()
            .ok_or_else(|| anyhow::anyhow!("输入路径包含无效字符: {:?}", input_path))?;
        let output_str = output_path.to_str()
            .ok_or_else(|| anyhow::anyhow!("输出路径包含无效字符: {:?}", output_path))?;
            
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args([
            "-i", input_str,
            "-c:a", "libvorbis",  // 使用 Vorbis 编码器
            "-q:a", "5",          // 质量设置 (0-10, 5 是平衡点)
            "-y",                 // 覆盖输出文件
            output_str,
        ]);
        
        // 在Windows上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        // 执行转换
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("启动 FFmpeg 失败")?;
        
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
                    // 短暂等待，减少CPU占用
                    std::thread::sleep(std::time::Duration::from_millis(50));
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

