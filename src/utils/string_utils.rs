/*!
 * 字符串工具模块
 * 提供安全的字符串处理功能
 */

/// 字符串工具
pub struct StringUtils;

impl StringUtils {
    /// 将字符串转换为ASCII安全格式
    pub fn to_ascii_safe(input: &str, replacement: char) -> String {
        // 使用预分配的String提高性能
        let mut result = String::with_capacity(input.len());
        for c in input.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                result.push(c);
            } else {
                result.push(replacement);
            }
        }
        result
    }
    
    /// 生成安全的文件名
    pub fn safe_filename(input: &str, index: usize) -> String {
        let safe_name = Self::to_ascii_safe(input, '_');
        
        if safe_name.is_empty() || safe_name.chars().all(|c| c == '_') {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(10);
            result.push_str("track_");
            result.push_str(&format!("{:03}", index));
            result
        } else {
            safe_name
        }
    }

    /// 从文件路径生成轨道名称
    pub fn generate_track_name_from_path(path: &std::path::Path, index: usize) -> String {
        if let Some(filename) = path.file_stem() {
            let name = filename.to_string_lossy();
            Self::safe_filename(&name, index)
        } else {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(10);
            result.push_str("track_");
            result.push_str(&format!("{:03}", index));
            result
        }
    }

    /// 生成类名
    pub fn generate_class_name(track_name: &str, base_class: &str, _index: usize) -> String {
        let safe_track_name = Self::to_ascii_safe(track_name, '_');
        // 使用预分配的String避免多次分配
        let mut result = String::with_capacity(base_class.len() + safe_track_name.len() + 1);
        result.push_str(base_class);
        result.push('_');
        result.push_str(&safe_track_name);
        result
    }
}