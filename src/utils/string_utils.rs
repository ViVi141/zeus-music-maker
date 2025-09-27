/*!
 * 字符串工具模块
 * 提供安全的字符串处理功能
 */

/// 字符串工具
pub struct StringUtils;

impl StringUtils {
    /// 将字符串转换为ASCII安全格式
    pub fn to_ascii_safe(input: &str, replacement: char) -> String {
        input.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                replacement
            })
            .collect()
    }
    
    /// 生成安全的文件名
    pub fn safe_filename(input: &str, index: usize) -> String {
        let safe_name = Self::to_ascii_safe(input, '_');
        
        if safe_name.is_empty() || safe_name.chars().all(|c| c == '_') {
            format!("track_{:03}", index)
        } else {
            safe_name
        }
    }

    /// 从文件路径生成轨道名称
    pub fn generate_track_name_from_path(path: &std::path::Path, index: usize) -> String {
        if let Some(filename) = path.file_stem() {
            let name = filename.to_string_lossy().to_string();
            Self::safe_filename(&name, index)
        } else {
            format!("track_{:03}", index)
        }
    }

    /// 生成类名
    pub fn generate_class_name(track_name: &str, base_class: &str, _index: usize) -> String {
        let safe_track_name = Self::to_ascii_safe(track_name, '_');
        format!("{}_{}", base_class, safe_track_name)
    }
}