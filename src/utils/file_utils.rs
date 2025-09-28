/*!
 * 文件工具模块
 * 提供文件操作相关的工具函数
 */

use rfd::FileDialog;
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use crate::utils::constants::file_ops;

/// 文件工具
pub struct FileUtils;

impl FileUtils {
    /// 选择音频文件
    pub fn select_audio_files() -> Option<Vec<PathBuf>> {
        FileDialog::new()
            .add_filter("OGG音频文件", &["ogg"])
            .set_title("选择OGG音频文件")
            .pick_files()
    }

    /// 选择PAA文件
    pub fn select_paa_file() -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("PAA files", &["paa"])
            .set_title("选择Logo文件 (.paa)")
            .pick_file()
    }

    /// 选择PBO文件
    pub fn select_pbo_file() -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("PBO文件", &["pbo"])
            .set_title("选择PBO文件")
            .pick_file()
    }

    /// 选择加密音频文件
    pub fn select_encrypted_audio_files() -> Option<Vec<PathBuf>> {
        FileDialog::new()
            .add_filter("加密音频文件", &["kgm", "ncm"])
            .set_title("选择加密音频文件")
            .pick_files()
    }

    /// 选择导出目录
    pub fn select_export_directory() -> Option<PathBuf> {
        FileDialog::new()
            .set_title("选择导出目录")
            .pick_folder()
    }

    /// 选择视频文件
    pub fn select_video_files() -> Option<Vec<PathBuf>> {
        FileDialog::new()
            .add_filter("视频文件", &["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v", "3gp", "ogv"])
            .set_title("选择视频文件")
            .pick_files()
    }

    /// 选择OGV视频文件（用于视频模组）
    pub fn select_ogv_video_files() -> Option<Vec<PathBuf>> {
        FileDialog::new()
            .add_filter("OGV视频文件", &["ogv"])
            .set_title("选择OGV视频文件")
            .pick_files()
    }

    /// 验证文件
    pub fn validate_file(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("文件不存在: {:?}", path));
        }
        if !path.is_file() {
            return Err(anyhow!("不是一个有效的文件: {:?}", path));
        }
        Ok(())
    }

    /// 检查是否为支持的音频文件
    pub fn is_supported_audio_file(path: &Path) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.to_lowercase() == "ogg")
            .unwrap_or(false)
    }

    /// 检查文件是否过大
    pub fn is_file_too_large(path: &Path) -> Result<bool> {
        let metadata = std::fs::metadata(path)?;
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        Ok(file_size_mb > file_ops::MAX_FILE_SIZE_MB as f64)
    }
}