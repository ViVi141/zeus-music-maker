use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

use crate::audio::AudioProcessor;
use crate::models::{ProjectSettings, Track, VideoFile};
use crate::video_converter::VideoConverter;
use crate::utils::{FileUtils, StringUtils};
use crate::utils::constants::file_ops;

/// 文件操作工具
pub struct FileOperations;

impl FileOperations {
    /// 优化的文件复制方法
    fn copy_file_optimized(source: &Path, destination: &Path) -> Result<()> {
        use std::io::{BufReader, BufWriter, Read, Write};
        
        // 创建目标目录
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 打开源文件
        let source_file = fs::File::open(source)?;
        let mut reader = BufReader::with_capacity(64 * 1024, source_file); // 64KB 缓冲区
        
        // 创建目标文件
        let dest_file = fs::File::create(destination)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, dest_file); // 64KB 缓冲区
        
        // 复制数据
        let mut buffer = [0u8; 64 * 1024]; // 64KB 缓冲区
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            writer.write_all(&buffer[..bytes_read])?;
        }
        
        writer.flush()?;
        Ok(())
    }
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

    /// 选择视频文件
    pub fn select_video_files() -> Option<Vec<PathBuf>> {
        FileUtils::select_video_files()
    }

    /// 选择OGV视频文件（用于视频模组）
    pub fn select_ogv_video_files() -> Option<Vec<PathBuf>> {
        FileUtils::select_ogv_video_files()
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
            
            let safe_track_name = StringUtils::safe_filename_pinyin(&track_name, index);

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

    /// 加载视频文件并创建视频文件记录
    pub fn load_video_files(paths: Vec<PathBuf>, class_name: &str) -> Result<Vec<VideoFile>> {
        let mut video_files = Vec::new();

        for (index, path) in paths.iter().enumerate() {
            // 验证文件
            if let Err(e) = FileUtils::validate_file(path) {
                warn!("文件验证失败 {:?}: {}", path, e);
                continue;
            }

            // 生成轨道名称和类名
            let video_name = StringUtils::generate_track_name_from_path(path, index);
            let video_class_name = StringUtils::generate_class_name(&video_name, class_name, index);

            // 创建视频文件记录
            let mut video_file = VideoFile::new(path.clone(), video_name, video_class_name);

            // 尝试获取视频信息
            if let Ok(converter) = VideoConverter::new() {
                if converter.is_supported_video_format(path) {
                    match converter.get_video_info(path) {
                        Ok(video_info) => {
                            let file_size = std::fs::metadata(path)
                                .map(|m| m.len())
                                .unwrap_or(0);
                            
                            video_file.set_video_info(
                                video_info.duration,
                                video_info.resolution,
                                file_size,
                            );
                            
                            info!("视频信息加载成功: {:?} - {}x{}, {}秒", 
                                path, video_info.resolution.0, video_info.resolution.1, video_info.duration);
                        }
                        Err(e) => {
                            warn!("获取视频信息失败 {:?}: {}", path, e);
                            // 即使无法获取信息，也添加文件记录
                        }
                    }
                } else {
                    warn!("不支持的视频格式: {:?}", path);
                    continue;
                }
            } else {
                warn!("无法创建视频转换器，跳过视频信息获取: {:?}", path);
            }

            video_files.push(video_file);
        }

        info!("成功加载 {} 个视频文件", video_files.len());
        Ok(video_files)
    }

    /// 创建模组目录结构
    pub fn create_mod_structure(project: &ProjectSettings, export_dir: &Path) -> Result<PathBuf> {
        let mod_name_no_spaces = project.mod_name_no_spaces();
        let mod_dir = export_dir.join(&mod_name_no_spaces);

        // 创建主目录
        fs::create_dir_all(&mod_dir)
            .with_context(|| format!("无法创建模组目录: {:?}", mod_dir))?;

        // 根据模组类型创建不同的目录结构
        match project.mod_type {
            crate::models::ModType::Music => {
                // 音乐模组：创建轨道目录
                let tracks_dir = mod_dir.join("folderwithtracks");
                fs::create_dir_all(&tracks_dir)
                    .with_context(|| format!("无法创建轨道目录: {:?}", tracks_dir))?;
                info!("创建音乐模组目录结构: {:?} (包含folderwithtracks)", mod_dir);
            }
            crate::models::ModType::Video => {
                // 视频模组：不需要创建额外文件夹
                info!("创建视频模组目录结构: {:?} (仅根目录)", mod_dir);
            }
        }

        Ok(mod_dir)
    }

    /// 生成ASCII安全的文件名（拼音风格）
    pub fn generate_ascii_filename_pinyin(original_name: &str, index: usize) -> String {
        StringUtils::safe_filename_pinyin(original_name, index)
    }


    /// 通用的文件复制函数，支持音频和视频文件
    /// 返回 (复制的文件名列表, 跳过的重复文件数量)
    fn copy_files_pinyin_generic<T>(
        items: &[T],
        mod_dir: &Path,
        get_path: fn(&T) -> &Path,
        get_name: fn(&T) -> &str,
        extension: &str,
        item_type: &str,
    ) -> Result<(Vec<String>, usize)>
    where
        T: std::fmt::Debug,
    {
        let tracks_dir = mod_dir.join("folderwithtracks");
        // 预分配容量，避免多次重新分配
        let mut copied_files = Vec::with_capacity(items.len());
        // 用于跟踪已使用的文件名，避免重复
        let mut used_filenames = std::collections::HashSet::new();
        let mut skipped_count = 0;

        for (i, item) in items.iter().enumerate() {
            let source = get_path(item);
            
            // 生成ASCII安全的文件名（拼音风格）
            let ascii_filename = Self::generate_ascii_filename_pinyin(get_name(item), i);
            // 使用预分配的String避免多次分配
            let mut new_filename = String::with_capacity(ascii_filename.len() + extension.len() + 1);
            new_filename.push_str(&ascii_filename);
            new_filename.push_str(extension);
            
            // 检查文件名是否已存在，如果存在则添加数字后缀
            let mut final_filename = new_filename.clone();
            let mut counter = 1;
            while used_filenames.contains(&final_filename) || tracks_dir.join(&final_filename).exists() {
                final_filename = format!("{}_{}{}", ascii_filename, counter, extension);
                counter += 1;
            }
            
            let destination = tracks_dir.join(&final_filename);

            if !source.exists() {
                warn!("源文件不存在: {:?}", source);
                continue;
            }

            // 检查目标文件是否已存在且内容相同，避免重复复制
            if destination.exists() {
                if let (Ok(source_metadata), Ok(dest_metadata)) = (source.metadata(), destination.metadata()) {
                    if source_metadata.len() == dest_metadata.len() {
                        debug!("跳过重复文件: {:?}", destination);
                        copied_files.push(final_filename.clone());
                        used_filenames.insert(final_filename);
                        skipped_count += 1;
                        continue;
                    }
                }
            }

            // 使用更高效的文件复制方法
            Self::copy_file_optimized(source, &destination)
                .with_context(|| format!("无法复制文件: {:?} -> {:?}", source, destination))?;

            copied_files.push(final_filename.clone());
            used_filenames.insert(final_filename);
            debug!("复制文件: {:?} -> {:?}", source, destination);
        }

        info!("成功复制 {} 个{}，跳过 {} 个重复文件", copied_files.len(), item_type, skipped_count);
        Ok((copied_files, skipped_count))
    }

    /// 复制轨道文件到模组目录并自动重命名（拼音风格）
    /// 返回 (复制的文件名列表, 跳过的重复文件数量)
    pub fn copy_track_files_pinyin(tracks: &[Track], mod_dir: &Path) -> Result<(Vec<String>, usize)> {
        Self::copy_files_pinyin_generic(
            tracks,
            mod_dir,
            |track| &track.path,
            |track| &track.track_name,
            ".ogg",
            "轨道文件",
        )
    }

    /// 复制视频文件到模组目录并自动重命名（拼音风格）
    /// 返回 (复制的文件名列表, 跳过的重复文件数量)
    pub fn copy_video_files_pinyin(video_files: &[VideoFile], mod_dir: &Path) -> Result<(Vec<String>, usize)> {
        // 视频文件直接放在模组根目录，不需要folderwithtracks文件夹
        let mut copied_files = Vec::with_capacity(video_files.len());
        let mut used_filenames = std::collections::HashSet::new();
        let mut skipped_count = 0;

        for (i, video_file) in video_files.iter().enumerate() {
            let source = &video_file.path;
            
            // 生成ASCII安全的文件名（拼音风格）
            let ascii_filename = Self::generate_ascii_filename_pinyin(&video_file.video_name, i);
            // 使用预分配的String避免多次分配
            let mut new_filename = String::with_capacity(ascii_filename.len() + 5);
            new_filename.push_str(&ascii_filename);
            new_filename.push_str(".ogv");
            
            // 检查文件名是否已存在，如果存在则添加数字后缀
            let mut final_filename = new_filename.clone();
            let mut counter = 1;
            while used_filenames.contains(&final_filename) || mod_dir.join(&final_filename).exists() {
                final_filename = format!("{}_{}.ogv", ascii_filename, counter);
                counter += 1;
            }
            
            let destination = mod_dir.join(&final_filename);

            if !source.exists() {
                warn!("源文件不存在: {:?}", source);
                continue;
            }

            // 检查目标文件是否已存在且内容相同，避免重复复制
            if destination.exists() {
                if let (Ok(source_metadata), Ok(dest_metadata)) = (source.metadata(), destination.metadata()) {
                    if source_metadata.len() == dest_metadata.len() {
                        debug!("跳过重复文件: {:?}", destination);
                        copied_files.push(final_filename.clone());
                        used_filenames.insert(final_filename);
                        skipped_count += 1;
                        continue;
                    }
                }
            }

            // 使用更高效的文件复制方法
            Self::copy_file_optimized(source, &destination)
                .with_context(|| format!("无法复制文件: {:?} -> {:?}", source, destination))?;

            copied_files.push(final_filename.clone());
            used_filenames.insert(final_filename);
            debug!("复制文件: {:?} -> {:?}", source, destination);
        }

        info!("成功复制 {} 个视频文件到根目录，跳过 {} 个重复文件", copied_files.len(), skipped_count);
        Ok((copied_files, skipped_count))
    }


    /// 复制Logo文件
    pub fn copy_logo_file(project: &ProjectSettings, mod_dir: &Path) -> Result<()> {
        let logo_dest = mod_dir.join("logo.paa");

        if let Some(logo_path) = &project.logo_path {
            if logo_path.exists() {
                // 使用更高效的文件复制方法
                Self::copy_file_optimized(logo_path, &logo_dest)
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

