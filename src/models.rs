use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashSet;

/// 音乐轨道数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// 轨道名称（在游戏中显示）
    pub track_name: String,
    /// 类名（用于Arma 3配置）
    pub class_name: String,
    /// 标签（可选，会显示为[Tag] Track Name）
    pub tag: String,
    /// 文件路径
    pub path: PathBuf,
    /// 时长（秒）
    pub duration: u32,
    /// 分贝调整值
    pub decibels: i32,
    /// 原始时长（用于恢复默认值）
    pub original_duration: u32,
    /// 原始分贝值（用于恢复默认值）
    pub original_decibels: i32,
}

impl Track {
    pub fn new(path: PathBuf, track_name: String, class_name: String) -> Self {
        Self {
            track_name,
            class_name,
            tag: String::new(),
            path,
            duration: 0,
            decibels: 0,
            original_duration: 0,
            original_decibels: 0,
        }
    }

    /// 获取显示名称（包含标签）
    pub fn display_name(&self) -> String {
        if self.tag.is_empty() {
            self.track_name.clone()
        } else {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(self.tag.len() + self.track_name.len() + 4);
            result.push('[');
            result.push_str(&self.tag);
            result.push_str("] ");
            result.push_str(&self.track_name);
            result
        }
    }

    /// 设置原始值（在加载音频信息时调用）
    pub fn set_original_values(&mut self, duration: u32, decibels: i32) {
        self.original_duration = duration;
        self.original_decibels = decibels;
        self.duration = duration;
        self.decibels = decibels;
    }

    /// 恢复到默认值
    pub fn reset_to_default(&mut self) {
        self.duration = self.original_duration;
        self.decibels = self.original_decibels;
    }

    /// 检查是否已修改
    pub fn is_modified(&self) -> bool {
        self.duration != self.original_duration || self.decibels != self.original_decibels
    }

}

/// 模组类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModType {
    Music,
    Video,
}

/// 视频文件数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFile {
    /// 视频名称（在游戏中显示）
    pub video_name: String,
    /// 类名（用于Arma 3配置）
    pub class_name: String,
    /// 标签（可选，会显示为[Tag] Video Name）
    pub tag: String,
    /// 文件路径
    pub path: PathBuf,
    /// 时长（秒）
    pub duration: u32,
    /// 分辨率
    pub resolution: (u32, u32),
    /// 文件大小（字节）
    pub file_size: u64,
}

impl VideoFile {
    pub fn new(path: PathBuf, video_name: String, class_name: String) -> Self {
        Self {
            video_name,
            class_name,
            tag: String::new(),
            path,
            duration: 0,
            resolution: (0, 0),
            file_size: 0,
        }
    }

    /// 获取显示名称（包含标签）
    pub fn display_name(&self) -> String {
        if self.tag.is_empty() {
            self.video_name.clone()
        } else {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(self.tag.len() + self.video_name.len() + 4);
            result.push('[');
            result.push_str(&self.tag);
            result.push_str("] ");
            result.push_str(&self.video_name);
            result
        }
    }

    /// 设置视频信息
    pub fn set_video_info(&mut self, duration: u32, resolution: (u32, u32), file_size: u64) {
        self.duration = duration;
        self.resolution = resolution;
        self.file_size = file_size;
    }
}

/// 项目设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// 模组名称
    pub mod_name: String,
    /// 作者名称
    pub author_name: String,
    /// Logo路径
    pub logo_path: Option<PathBuf>,
    /// 是否使用默认Logo
    pub use_default_logo: bool,
    /// 类名（从模组名称生成）
    pub class_name: String,
    /// 模组类型
    pub mod_type: ModType,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            mod_name: "New Music Mod".to_string(),
            author_name: "Your username".to_string(),
            logo_path: None,
            use_default_logo: true,
            class_name: "MyMusicClass".to_string(),
            mod_type: ModType::Music,
        }
    }
}

impl ProjectSettings {
    /// 更新类名（从模组名称生成）
    pub fn update_class_name(&mut self) {
        self.class_name = self
            .mod_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() {
                c
            } else {
                '_'
            })
            .collect::<String>();
    }

    /// 根据模组类型设置默认名称
    pub fn set_default_name_for_mod_type(&mut self) {
        log::info!("设置默认名称，当前模组类型: {:?}", self.mod_type);
        match self.mod_type {
            ModType::Music => {
                self.mod_name = "New Music Mod".to_string();
                self.class_name = "MyMusicClass".to_string();
                log::info!("设置为音乐模组: {} / {}", self.mod_name, self.class_name);
            }
            ModType::Video => {
                self.mod_name = "New Video Mod".to_string();
                self.class_name = "MyVideoClass".to_string();
                log::info!("设置为视频模组: {} / {}", self.mod_name, self.class_name);
            }
        }
    }

    /// 获取模组名称（无空格，用于文件夹名）
    pub fn mod_name_no_spaces(&self) -> String {
        let result: String = self.mod_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            })
            .collect();
        
        if result.is_empty() || result.chars().all(|c| c == '_') {
            // 根据模组类型返回不同的默认名称
            match self.mod_type {
                ModType::Music => "NewMusicMod".to_string(),
                ModType::Video => "NewVideoMod".to_string(),
            }
        } else {
            result
        }
    }
}

/// 导出设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// 是否在轨道名称前添加标签
    pub append_tags: bool,
    /// 是否使用默认Logo
    pub use_default_logo: bool,
}

/// 任务类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    AudioDecrypt,
    PaaConvert,
    ModExport,
    AudioLoad,
    AudioConvert,
    VideoConvert,
    VideoModExport,
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// 进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// 任务类型
    pub task_type: TaskType,
    /// 任务状态
    pub status: TaskStatus,
    /// 当前进度 (0.0 - 1.0)
    pub progress: f32,
    /// 当前处理的文件索引
    pub current_file: usize,
    /// 总文件数
    pub total_files: usize,
    /// 当前处理的文件名
    pub current_filename: String,
    /// 开始时间
    pub start_time: Option<std::time::SystemTime>,
    /// 预计剩余时间（秒）
    pub estimated_remaining: Option<u64>,
    /// 处理速度（文件/秒）
    pub processing_speed: Option<f32>,
}

impl Default for ProgressInfo {
    fn default() -> Self {
        Self {
            task_type: TaskType::AudioDecrypt,
            status: TaskStatus::Pending,
            progress: 0.0,
            current_file: 0,
            total_files: 0,
            current_filename: String::new(),
            start_time: None,
            estimated_remaining: None,
            processing_speed: None,
        }
    }
}

/// 任务管理器
#[derive(Debug, Clone)]
pub struct TaskManager {
    /// 当前运行的任务
    pub current_task: Option<ProgressInfo>,
    /// 任务历史
    pub task_history: Vec<ProgressInfo>,
    /// 是否显示进度对话框
    pub show_progress: bool,
    /// 是否允许取消当前任务
    pub can_cancel: bool,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            current_task: None,
            task_history: Vec::new(),
            show_progress: false,
            can_cancel: false,
        }
    }
}

impl TaskManager {
    /// 开始新任务
    pub fn start_task(&mut self, task_type: TaskType, total_files: usize) {
        self.current_task = Some(ProgressInfo {
            task_type,
            status: TaskStatus::Running,
            progress: 0.0,
            current_file: 0,
            total_files,
            current_filename: String::new(),
            start_time: Some(std::time::SystemTime::now()),
            estimated_remaining: None,
            processing_speed: None,
        });
        self.show_progress = true;
        self.can_cancel = true;
    }

    /// 更新进度
    pub fn update_progress(&mut self, current_file: usize, filename: &str) {
        if let Some(ref mut task) = self.current_task {
            task.current_file = current_file;
            // 避免不必要的字符串克隆，只在文件名变化时更新
            if task.current_filename != filename {
                task.current_filename = filename.to_string();
            }
            task.progress = if task.total_files > 0 {
                current_file as f32 / task.total_files as f32
            } else {
                0.0
            };

            // 计算处理速度和预计剩余时间
            if let Some(start_time) = task.start_time {
                let elapsed = start_time.elapsed().unwrap_or_default();
                if elapsed.as_secs() > 0 && current_file > 0 {
                    task.processing_speed = Some(current_file as f32 / elapsed.as_secs_f32());
                    if let Some(speed) = task.processing_speed {
                        let remaining_files = task.total_files - current_file;
                        task.estimated_remaining = Some((remaining_files as f32 / speed) as u64);
                    }
                }
            }
        }
    }

    /// 完成任务
    pub fn complete_task(&mut self) {
        if let Some(mut task) = self.current_task.take() {
            task.status = TaskStatus::Completed;
            task.progress = 1.0;
            self.task_history.push(task);
        }
        self.show_progress = false;
        self.can_cancel = false;
    }

    /// 任务失败
    pub fn fail_task(&mut self, error: String) {
        if let Some(mut task) = self.current_task.take() {
            task.status = TaskStatus::Failed(error);
            self.task_history.push(task);
        }
        self.show_progress = false;
        self.can_cancel = false;
    }

    /// 取消任务
    pub fn cancel_task(&mut self) {
        if let Some(mut task) = self.current_task.take() {
            task.status = TaskStatus::Cancelled;
            self.task_history.push(task);
        }
        self.show_progress = false;
        self.can_cancel = false;
    }

    /// 获取当前进度
    pub fn get_current_progress(&self) -> Option<&ProgressInfo> {
        self.current_task.as_ref()
    }
    
    /// 检查是否有任务正在运行
    pub fn is_running(&self) -> bool {
        self.current_task.as_ref()
            .map(|task| task.status == TaskStatus::Running)
            .unwrap_or(false)
    }

}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            append_tags: true,
            use_default_logo: true,
        }
    }
}

/// 应用程序状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// 项目设置
    pub project: ProjectSettings,
    /// 轨道列表
    pub tracks: Vec<Track>,
    /// 视频文件列表
    pub video_files: Vec<VideoFile>,
    /// 轨道路径缓存（用于快速重复检测）
    #[serde(skip)]
    pub track_paths: HashSet<PathBuf>,
    /// 视频路径缓存（用于快速重复检测）
    #[serde(skip)]
    pub video_paths: HashSet<PathBuf>,
    /// 选中的轨道索引
    pub selected_track: Option<usize>,
    /// 选中的视频文件索引
    pub selected_video: Option<usize>,
    /// 导出设置
    pub export_settings: ExportSettings,
    /// 是否显示项目设置对话框
    pub show_project_settings: bool,
    /// 是否显示导出对话框
    pub show_export_dialog: bool,
    /// 是否显示关于对话框
    pub show_about: bool,
    /// 是否显示新用户指导对话框
    pub show_user_guide: bool,
    /// 是否首次启动（用于自动显示指导）
    pub is_first_launch: bool,
    /// 配置文件路径（用于持久化设置）
    pub config_file_path: Option<PathBuf>,
    /// 是否在启动时自动显示用户指导
    pub auto_show_guide: bool,
    /// 是否显示轨道编辑器
    pub show_track_editor: bool,
    /// 是否显示PAA转换对话框
    pub show_paa_converter: bool,
    /// PAA转换选中的文件
    pub paa_selected_files: Vec<std::path::PathBuf>,
    /// PAA转换输出目录
    pub paa_output_directory: Option<std::path::PathBuf>,
    /// PAA转换选项
    pub paa_options: crate::paa_converter::PaaOptions,
    /// 是否显示预览对话框
    pub show_paa_preview: bool,
    /// 是否显示PAA转换结果对话框
    pub show_paa_result: bool,
    /// PAA转换结果消息
    pub paa_result: Option<String>,
    /// 是否显示轨道计数对话框
    pub show_track_count: bool,
    /// 图片纹理管理器
    pub image_texture_manager: crate::paa_converter::ImageTextureManager,
    /// 运行时图片纹理管理器
    #[serde(skip)]
    pub runtime_texture_manager: Option<crate::paa_converter::RuntimeImageTextureManager>,
    /// 是否显示导出结果对话框
    pub show_export_result: bool,
    /// 导出结果信息
    pub export_result: Option<String>,
    /// 是否显示音频解密对话框
    pub show_audio_decrypt: bool,
    /// 音频解密选中的文件
    pub audio_decrypt_selected_files: Vec<std::path::PathBuf>,
    /// 音频解密输出目录
    pub audio_decrypt_output_directory: Option<std::path::PathBuf>,
    /// 音频解密结果
    pub audio_decrypt_result: Option<String>,
    /// 是否显示音频解密结果对话框
    pub show_audio_decrypt_result: bool,
    /// 是否执行音频解密
    pub should_decrypt_audio: bool,
    /// 是否显示音频转换对话框
    pub show_audio_converter: bool,
    /// 音频转换选中的文件
    pub audio_convert_selected_files: Vec<std::path::PathBuf>,
    /// 音频转换输出目录
    pub audio_convert_output_directory: Option<std::path::PathBuf>,
    /// 音频转换结果
    pub audio_convert_result: Option<String>,
    /// 是否显示音频转换结果对话框
    pub show_audio_convert_result: bool,
    /// 是否执行音频转换
    pub should_convert_audio: bool,
    /// 是否显示FFmpeg下载对话框
    pub show_ffmpeg_download: bool,
    /// FFmpeg下载进度 (0.0-100.0)
    pub ffmpeg_download_progress: f64,
    /// FFmpeg下载状态消息
    pub ffmpeg_download_status: String,
    /// 是否正在下载FFmpeg
    pub is_downloading_ffmpeg: bool,
    /// 是否已经启动了下载任务
    pub ffmpeg_download_started: bool,
    /// 手动选择的FFmpeg路径
    pub manual_ffmpeg_path: Option<std::path::PathBuf>,
    /// 是否显示手动路径选择对话框
    pub show_manual_path_selection: bool,
    /// 是否显示视频转换对话框
    pub show_video_converter: bool,
    /// 视频转换选中的文件
    pub video_convert_selected_files: Vec<std::path::PathBuf>,
    /// 视频转换输出目录
    pub video_convert_output_directory: Option<std::path::PathBuf>,
    /// 视频转换结果
    pub video_convert_result: Option<String>,
    /// 是否显示视频转换结果对话框
    pub show_video_convert_result: bool,
    /// 是否执行视频转换
    pub should_convert_video: bool,
    /// 是否显示FFmpeg插件管理对话框
    pub show_ffmpeg_plugin: bool,
    /// FFmpeg镜像源
    pub ffmpeg_mirror_source: crate::ffmpeg_plugin::MirrorSource,
    /// FFmpeg自动下载选项
    pub ffmpeg_auto_download: bool,
    /// 文件操作提示信息
    pub file_operation_message: Option<String>,
    /// 任务管理器
    #[serde(skip)]
    pub task_manager: TaskManager,
}


impl AppState {
    /// 防重复添加轨道（基于文件路径）
    pub fn add_track_with_duplicate_check(&mut self, track: Track) -> bool {
        // 使用HashSet进行O(1)重复检测
        if self.track_paths.contains(&track.path) {
            return false; // 重复，未添加
        }
        self.track_paths.insert(track.path.clone());
        self.tracks.push(track);
        true // 成功添加
    }

    /// 批量添加轨道（带重复检测）
    pub fn add_tracks_with_duplicate_check(&mut self, tracks: Vec<Track>) -> (usize, usize) {
        let mut added_count = 0;
        let mut duplicate_count = 0;
        
        for track in tracks {
            if self.add_track_with_duplicate_check(track) {
                added_count += 1;
            } else {
                duplicate_count += 1;
            }
        }
        
        (added_count, duplicate_count)
    }

    /// 获取轨道重复统计信息
    pub fn get_track_duplicate_info(&self) -> String {
        let total_tracks = self.tracks.len();
        let unique_paths: std::collections::HashSet<_> = self.tracks.iter().map(|t| &t.path).collect();
        let unique_count = unique_paths.len();
        let duplicate_count = total_tracks - unique_count;
        
        if duplicate_count > 0 {
            format!("⚠️ 总轨道数: {} (其中 {} 个重复)", total_tracks, duplicate_count)
        } else {
            format!("总轨道数: {}", total_tracks)
        }
    }

    /// 移除选中的轨道
    pub fn remove_selected_track(&mut self) {
        if let Some(index) = self.selected_track {
            if index < self.tracks.len() {
                let removed_track = self.tracks.remove(index);
                // 从路径缓存中移除
                self.track_paths.remove(&removed_track.path);
                // 调整选中索引，如果删除的是最后一个，则选择前一个
                if index >= self.tracks.len() && !self.tracks.is_empty() {
                    self.selected_track = Some(index - 1);
                } else if self.tracks.is_empty() {
                    self.selected_track = None;
                }
            } else {
                // 索引越界，清除选中状态
                self.selected_track = None;
            }
        }
    }

    /// 清空所有轨道
    pub fn clear_tracks(&mut self) {
        self.tracks.clear();
        self.track_paths.clear();
        self.selected_track = None;
    }

    /// 获取轨道数量
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// 获取视频文件数量
    pub fn video_count(&self) -> usize {
        self.video_files.len()
    }

    /// 防重复添加视频文件（基于文件路径）
    pub fn add_video_with_duplicate_check(&mut self, video: VideoFile) -> bool {
        // 使用HashSet进行O(1)重复检测
        if self.video_paths.contains(&video.path) {
            return false; // 重复，未添加
        }
        self.video_paths.insert(video.path.clone());
        self.video_files.push(video);
        true // 成功添加
    }

    /// 批量添加视频文件（带重复检测）
    pub fn add_videos_with_duplicate_check(&mut self, videos: Vec<VideoFile>) -> (usize, usize) {
        let mut added_count = 0;
        let mut duplicate_count = 0;
        
        for video in videos {
            if self.add_video_with_duplicate_check(video) {
                added_count += 1;
            } else {
                duplicate_count += 1;
            }
        }
        
        (added_count, duplicate_count)
    }

    /// 移除选中的视频文件
    pub fn remove_selected_video(&mut self) {
        if let Some(index) = self.selected_video {
            if index < self.video_files.len() {
                let removed_video = self.video_files.remove(index);
                // 从路径缓存中移除
                self.video_paths.remove(&removed_video.path);
                // 调整选中索引，如果删除的是最后一个，则选择前一个
                if index >= self.video_files.len() && !self.video_files.is_empty() {
                    self.selected_video = Some(index - 1);
                } else if self.video_files.is_empty() {
                    self.selected_video = None;
                }
            } else {
                // 索引越界，清除选中状态
                self.selected_video = None;
            }
        }
    }

    /// 清空所有视频文件
    pub fn clear_videos(&mut self) {
        self.video_files.clear();
        self.video_paths.clear();
        self.selected_video = None;
    }
}

impl AppState {
    /// 从配置文件加载状态
    pub fn load_config() -> Self {
        let config_path = Self::get_config_path();
        
        // 尝试从配置文件加载
        if let Ok(config_content) = std::fs::read_to_string(&config_path) {
            if let Ok(mut state) = serde_json::from_str::<AppState>(&config_content) {
                // 恢复运行时状态
                state.restore_runtime_state();
                log::info!("从配置文件加载状态: {:?}", config_path);
                return state;
            } else {
                log::warn!("配置文件格式错误，使用默认状态");
            }
        } else {
            log::info!("配置文件不存在，使用默认状态");
        }
        
        // 使用默认状态
        Self::default()
    }
    
    /// 保存状态到配置文件
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 序列化状态（排除运行时状态）
        let config_json = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, config_json)?;
        
        log::info!("状态已保存到配置文件: {:?}", config_path);
        Ok(())
    }
    
    /// 获取配置文件路径
    fn get_config_path() -> std::path::PathBuf {
        // 使用用户配置目录
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("zeus-music-maker").join("config.json")
        } else {
            // 备用路径
            std::env::current_dir().unwrap().join("config.json")
        }
    }
    
    /// 恢复运行时状态（从配置文件加载后调用）
    fn restore_runtime_state(&mut self) {
        // 清空所有列表和选择（每次启动都重新开始）
        self.tracks.clear();
        self.video_files.clear();
        self.selected_track = None;
        self.selected_video = None;
        
        // 清空路径缓存
        self.track_paths.clear();
        self.video_paths.clear();
        
        // 清空PAA相关状态
        self.paa_selected_files.clear();
        self.paa_output_directory = None;
        self.paa_result = None;
        
        // 清空音频解密相关状态
        self.audio_decrypt_selected_files.clear();
        self.audio_decrypt_output_directory = None;
        self.audio_decrypt_result = None;
        
        // 清空音频转换相关状态
        self.audio_convert_selected_files.clear();
        self.audio_convert_output_directory = None;
        self.audio_convert_result = None;
        
        // 清空视频转换相关状态
        self.video_convert_selected_files.clear();
        self.video_convert_output_directory = None;
        self.video_convert_result = None;
        
        // 重置运行时状态
        self.runtime_texture_manager = None;
        
        // 重置UI状态（这些不应该被持久化）
        self.show_project_settings = false;
        self.show_export_dialog = false;
        self.show_about = false;
        self.show_track_editor = false;
        self.show_paa_preview = false;
        self.show_paa_result = false;
        self.show_export_result = false;
        self.show_audio_decrypt_result = false;
        self.show_audio_convert_result = false;
        self.show_video_convert_result = false;
        self.show_manual_path_selection = false;
        self.show_audio_converter = false;
        self.show_video_converter = false;
        self.show_paa_converter = false;
        self.show_audio_decrypt = false;
        
        // 重置任务状态
        self.task_manager = TaskManager::default();
        
        // 清空临时消息
        self.file_operation_message = None;
        self.export_result = None;
        
        // 重置下载状态
        self.ffmpeg_download_progress = 0.0;
        self.ffmpeg_download_status.clear();
        self.is_downloading_ffmpeg = false;
        self.ffmpeg_download_started = false;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project: ProjectSettings::default(),
            tracks: Vec::new(),
            video_files: Vec::new(),
            track_paths: HashSet::new(),
            video_paths: HashSet::new(),
            selected_track: None,
            selected_video: None,
            export_settings: ExportSettings::default(),
            show_project_settings: false,
            show_export_dialog: false,
            show_about: false,
            show_user_guide: false,
            is_first_launch: true,
            config_file_path: None,
            auto_show_guide: true,
            show_track_editor: false,
            paa_selected_files: Vec::new(),
            paa_output_directory: None,
            paa_options: crate::paa_converter::PaaOptions::default(),
            show_paa_preview: false,
            show_paa_result: false,
            paa_result: None,
            show_track_count: false,
            image_texture_manager: crate::paa_converter::ImageTextureManager::default(),
            runtime_texture_manager: None,
            show_export_result: false,
            export_result: None,
            show_paa_converter: false,
            show_audio_decrypt: false,
            audio_decrypt_selected_files: Vec::new(),
            audio_decrypt_output_directory: None,
            audio_decrypt_result: None,
            show_audio_decrypt_result: false,
            should_decrypt_audio: false,
            show_audio_converter: false,
            audio_convert_selected_files: Vec::new(),
            audio_convert_output_directory: None,
            audio_convert_result: None,
            show_audio_convert_result: false,
            should_convert_audio: false,
            show_ffmpeg_download: false,
            ffmpeg_download_progress: 0.0,
            ffmpeg_download_status: String::new(),
            is_downloading_ffmpeg: false,
            ffmpeg_download_started: false,
            manual_ffmpeg_path: None,
            show_manual_path_selection: false,
            show_video_converter: false,
            video_convert_selected_files: Vec::new(),
            video_convert_output_directory: None,
            video_convert_result: None,
            show_video_convert_result: false,
            should_convert_video: false,
            show_ffmpeg_plugin: false,
            ffmpeg_mirror_source: crate::ffmpeg_plugin::MirrorSource::default(),
            ffmpeg_auto_download: true,
            file_operation_message: None,
            task_manager: TaskManager::default(),
        }
    }
}
