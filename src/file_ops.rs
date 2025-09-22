use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

use crate::audio::AudioProcessor;
use crate::models::{ProjectSettings, Track};
use crate::utils::{FileUtils, StringUtils};
use crate::utils::constants::file_ops;

/// 文件操作工具
pub struct FileOperations;

impl FileOperations {
    /// 选择音频文件（仅支持OGG格式）
    pub fn select_audio_files() -> Option<Vec<PathBuf>> {
        FileUtils::select_audio_files()
    }

    /// 选择Logo文件
    pub fn select_logo_file() -> Option<PathBuf> {
        FileUtils::select_paa_file()
    }

    /// 选择PBO文件
    pub fn select_pbo_file() -> Option<PathBuf> {
        FileUtils::select_pbo_file()
    }


    /// 选择加密音频文件
    pub fn select_encrypted_audio_files() -> Option<Vec<PathBuf>> {
        FileUtils::select_encrypted_audio_files()
    }


    /// 选择导出目录
    pub fn select_export_directory() -> Option<PathBuf> {
        FileUtils::select_export_directory()
    }

    /// 加载音频文件并创建轨道
    pub fn load_audio_files(paths: Vec<PathBuf>, class_name: &str) -> Result<Vec<Track>> {
        let mut tracks = Vec::new();

        for (index, path) in paths.iter().enumerate() {
            // 验证文件
            if let Err(e) = FileUtils::validate_file(path) {
                warn!("文件验证失败 {:?}: {}", path, e);
                continue;
            }

            // 验证文件格式（确保是OGG文件）
            if !FileUtils::is_supported_audio_file(path) {
                warn!("文件不是支持的音频格式: {:?}", path);
                continue;
            }

            // 检查文件大小
            if let Ok(true) = FileUtils::is_file_too_large(path) {
                warn!("文件过大，跳过: {:?}", path);
                continue;
            }

            // 生成安全的轨道名称
            let track_name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            let safe_track_name = StringUtils::safe_filename(&track_name, index);

            // 创建轨道
            let mut track = Track::new(path.clone(), safe_track_name, class_name.to_string());

            // 获取音频信息
            match AudioProcessor::get_audio_info(path) {
                Ok(audio_info) => {
                    track.set_original_values(audio_info.duration, file_ops::DEFAULT_DECIBELS);
                    debug!("加载音频文件: {:?}, 时长: {}秒", path, audio_info.duration);
                }
                Err(e) => {
                    warn!("无法读取音频信息 {:?}: {}", path, e);
                    // 即使无法读取音频信息，也设置默认值
                    track.set_original_values(file_ops::DEFAULT_TRACK_DURATION, file_ops::DEFAULT_DECIBELS);
                }
            }

            tracks.push(track);
        }

        info!("成功加载 {} 个音频文件", tracks.len());
        Ok(tracks)
    }

    /// 创建模组目录结构
    pub fn create_mod_structure(project: &ProjectSettings, export_dir: &Path) -> Result<PathBuf> {
        let mod_name_no_spaces = project.mod_name_no_spaces();
        let mod_dir = export_dir.join(&mod_name_no_spaces);

        // 创建主目录
        fs::create_dir_all(&mod_dir)
            .with_context(|| format!("无法创建模组目录: {:?}", mod_dir))?;

        // 创建轨道目录
        let tracks_dir = mod_dir.join("folderwithtracks");
        fs::create_dir_all(&tracks_dir)
            .with_context(|| format!("无法创建轨道目录: {:?}", tracks_dir))?;

        info!("创建模组目录结构: {:?}", mod_dir);
        Ok(mod_dir)
    }

    /// 生成ASCII安全的文件名
    pub fn generate_ascii_filename(original_name: &str, index: usize) -> String {
        StringUtils::safe_filename(original_name, index)
    }

    /// 复制轨道文件到模组目录并自动重命名
    pub fn copy_track_files(tracks: &[Track], mod_dir: &Path) -> Result<Vec<String>> {
        let tracks_dir = mod_dir.join("folderwithtracks");
        let mut copied_files = Vec::new();

        for (i, track) in tracks.iter().enumerate() {
            let source = &track.path;
            
            // 生成ASCII安全的文件名
            let ascii_filename = Self::generate_ascii_filename(&track.track_name, i);
            let new_filename = format!("{}.ogg", ascii_filename);
            let destination = tracks_dir.join(&new_filename);

            if !source.exists() {
                warn!("源文件不存在: {:?}", source);
                continue;
            }

            fs::copy(source, &destination)
                .with_context(|| format!("无法复制文件: {:?} -> {:?}", source, destination))?;

            copied_files.push(new_filename);
            debug!("复制文件: {:?} -> {:?}", source, destination);
        }

        info!("成功复制 {} 个轨道文件", copied_files.len());
        Ok(copied_files)
    }

    /// 复制Logo文件
    pub fn copy_logo_file(project: &ProjectSettings, mod_dir: &Path) -> Result<()> {
        let logo_dest = mod_dir.join("logo.paa");

        if let Some(logo_path) = &project.logo_path {
            if logo_path.exists() {
                fs::copy(logo_path, &logo_dest)
                    .with_context(|| format!("无法复制Logo文件: {:?} -> {:?}", logo_path, logo_dest))?;
                info!("复制自定义Logo: {:?}", logo_path);
            } else {
                warn!("Logo文件不存在: {:?}", logo_path);
                Self::copy_default_logo(mod_dir)?;
            }
        } else {
            Self::copy_default_logo(mod_dir)?;
        }

        Ok(())
    }

    /// 复制默认Logo文件
    fn copy_default_logo(mod_dir: &Path) -> Result<()> {
        let logo_dest = mod_dir.join("logo.paa");
        let default_logo_path = Path::new("assets/logo.paa");
        
        if default_logo_path.exists() {
            // 复制默认Logo文件
            fs::copy(default_logo_path, &logo_dest)
                .with_context(|| format!("无法复制默认Logo: {:?} -> {:?}", default_logo_path, logo_dest))?;
            info!("使用默认Logo: {:?}", default_logo_path);
        } else {
            // 如果默认Logo不存在，创建一个占位符文件
            let logo_content = b"# Default logo placeholder";
            fs::write(&logo_dest, logo_content)
                .with_context(|| format!("无法创建默认Logo: {:?}", logo_dest))?;
            warn!("默认Logo文件不存在，使用占位符");
        }
        
        Ok(())
    }

    /// 复制Steam Logo文件
    pub fn copy_steam_logo(mod_dir: &Path) -> Result<()> {
        let steam_logo_dest = mod_dir.join("steamLogo.png");
        let default_steam_logo_path = Path::new("assets/zeus_steam_logo.png");
        
        if default_steam_logo_path.exists() {
            // 复制默认Steam Logo文件
            fs::copy(default_steam_logo_path, &steam_logo_dest)
                .with_context(|| format!("无法复制Steam Logo: {:?} -> {:?}", default_steam_logo_path, steam_logo_dest))?;
            info!("使用默认Steam Logo: {:?}", default_steam_logo_path);
        } else {
            // 如果默认Steam Logo不存在，创建一个占位符文件
            let steam_logo_content = b"# Steam logo placeholder";
            fs::write(&steam_logo_dest, steam_logo_content)
                .with_context(|| format!("无法创建Steam Logo: {:?}", steam_logo_dest))?;
            warn!("默认Steam Logo文件不存在，使用占位符");
        }
        
        Ok(())
    }

    /// 创建PBO模组结构
    pub fn create_pbo_mod_structure(project: &ProjectSettings, pbo_path: &Path, export_dir: &Path) -> Result<PathBuf> {
        let mod_name_no_spaces = format!("@{}", project.mod_name_no_spaces());
        let mod_dir = export_dir.join(&mod_name_no_spaces);

        // 创建主目录
        fs::create_dir_all(&mod_dir)
            .with_context(|| format!("无法创建PBO模组目录: {:?}", mod_dir))?;

        // 复制Logo文件
        Self::copy_logo_file(project, &mod_dir)?;
        Self::copy_steam_logo(&mod_dir)?;

        // 创建Addons目录
        let addons_dir = mod_dir.join("Addons");
        fs::create_dir_all(&addons_dir)
            .with_context(|| format!("无法创建Addons目录: {:?}", addons_dir))?;

        // 复制PBO文件
        let pbo_dest = addons_dir.join("MusicModPBO.pbo");
        fs::copy(pbo_path, &pbo_dest)
            .with_context(|| format!("无法复制PBO文件: {:?} -> {:?}", pbo_path, pbo_dest))?;

        info!("创建PBO模组结构: {:?}", mod_dir);
        Ok(mod_dir)
    }

}

