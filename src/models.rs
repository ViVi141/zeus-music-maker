use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
            format!("[{}] {}", self.tag, self.track_name)
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
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            mod_name: "New Music Mod".to_string(),
            author_name: "Your username".to_string(),
            logo_path: None,
            use_default_logo: true,
            class_name: "MyMusicClass".to_string(),
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
            "NewMusicMod".to_string()
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
    /// 选中的轨道索引
    pub selected_track: Option<usize>,
    /// 导出设置
    pub export_settings: ExportSettings,
    /// 是否显示项目设置对话框
    pub show_project_settings: bool,
    /// 是否显示导出对话框
    pub show_export_dialog: bool,
    /// 是否显示关于对话框
    pub show_about: bool,
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
}


impl AppState {
    /// 添加轨道
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    /// 移除选中的轨道
    pub fn remove_selected_track(&mut self) {
        if let Some(index) = self.selected_track {
            if index < self.tracks.len() {
                self.tracks.remove(index);
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
        self.selected_track = None;
    }

    /// 获取轨道数量
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project: ProjectSettings::default(),
            tracks: Vec::new(),
            selected_track: None,
            export_settings: ExportSettings::default(),
            show_project_settings: false,
            show_export_dialog: false,
            show_about: false,
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
        }
    }
}
