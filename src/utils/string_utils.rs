/*!
 * 字符串工具模块
 * 提供安全的字符串处理功能
 */

use pinyin::ToPinyin;

/// 字符串工具
pub struct StringUtils;

impl StringUtils {
    /// 将中文字符串转换为拼音（罗马字母）
    pub fn chinese_to_pinyin(input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 2);
        
        for c in input.chars() {
            if c.is_ascii_alphanumeric() {
                // 保留ASCII字母数字
                result.push(c);
            } else if c.is_ascii_punctuation() {
                // 处理标点符号
                match c {
                    ' ' | '-' | '_' | '.' | ',' | '!' | '?' => result.push(c),
                    _ => result.push('_'),
                }
            } else if Self::is_chinese_char(c) {
                // 只对中文字符尝试转换为拼音
                let pinyin_result = c.to_pinyin();
                if let Some(pinyin) = pinyin_result {
                    // 使用第一个拼音读音
                    result.push_str(&pinyin.plain());
                } else {
                    // 如果无法转换为拼音，使用Unicode码点
                    result.push_str(&format!("u{:04x}", c as u32));
                }
            } else {
                // 对于其他非ASCII字符（如拉丁字母、数字等），保持原样
                result.push(c);
            }
        }
        
        result
    }

    /// 判断字符是否为中文字符
    fn is_chinese_char(c: char) -> bool {
        let code = c as u32;
        // 中文字符的Unicode范围
        (0x4E00..=0x9FFF).contains(&code) ||  // CJK统一汉字
        (0x3400..=0x4DBF).contains(&code) ||  // CJK扩展A
        (0x20000..=0x2A6DF).contains(&code) || // CJK扩展B
        (0x2A700..=0x2B73F).contains(&code) || // CJK扩展C
        (0x2B740..=0x2B81F).contains(&code) || // CJK扩展D
        (0x2B820..=0x2CEAF).contains(&code) || // CJK扩展E
        (0x2CEB0..=0x2EBEF).contains(&code)    // CJK扩展F
    }

    /// 将字符串转换为ASCII安全格式（拼音风格）
    pub fn to_ascii_safe_pinyin(input: &str) -> String {
        // 首先将中文字符转换为拼音
        let pinyin_input = Self::chinese_to_pinyin(input);
        
        // 然后确保所有字符都是ASCII安全的
        let mut result = String::with_capacity(pinyin_input.len());
        for c in pinyin_input.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                result.push(c);
            } else {
                result.push('_');
            }
        }
        result
    }

    
    /// 生成安全的文件名（拼音风格）
    pub fn safe_filename_pinyin(input: &str, index: usize) -> String {
        let safe_name = Self::to_ascii_safe_pinyin(input);
        
        if safe_name.is_empty() || safe_name.chars().all(|c| c == '_') {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(10);
            result.push_str("track");
            result.push_str(&format!("{:03}", index));
            result
        } else {
            safe_name
        }
    }


    /// 从文件路径生成轨道名称（拼音风格）
    #[allow(dead_code)]
    pub fn generate_track_name_from_path_pinyin(path: &std::path::Path, index: usize) -> String {
        if let Some(filename) = path.file_stem() {
            let name = filename.to_string_lossy();
            Self::safe_filename_pinyin(&name, index)
        } else {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(10);
            result.push_str("track");
            result.push_str(&format!("{:03}", index));
            result
        }
    }

    /// 从文件路径生成轨道名称
    pub fn generate_track_name_from_path(path: &std::path::Path, index: usize) -> String {
        if let Some(filename) = path.file_stem() {
            let name = filename.to_string_lossy();
            Self::safe_filename_pinyin(&name, index)
        } else {
            // 使用预分配的String避免多次分配
            let mut result = String::with_capacity(10);
            result.push_str("track_");
            result.push_str(&format!("{:03}", index));
            result
        }
    }

    /// 生成类名（拼音风格）
    #[allow(dead_code)]
    pub fn generate_class_name_pinyin(track_name: &str, base_class: &str, _index: usize) -> String {
        let safe_track_name = Self::to_ascii_safe_pinyin(track_name);
        // 使用预分配的String避免多次分配
        let mut result = String::with_capacity(base_class.len() + safe_track_name.len() + 1);
        result.push_str(base_class);
        result.push('_');
        result.push_str(&safe_track_name);
        result
    }

    /// 生成类名
    pub fn generate_class_name(track_name: &str, base_class: &str, _index: usize) -> String {
        let safe_track_name = Self::to_ascii_safe_pinyin(track_name);
        // 使用预分配的String避免多次分配
        let mut result = String::with_capacity(base_class.len() + safe_track_name.len() + 1);
        result.push_str(base_class);
        result.push('_');
        result.push_str(&safe_track_name);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_filenames_fixed() {
        // 测试修复后的混合文件名处理
        assert_eq!(StringUtils::chinese_to_pinyin("Carte Blanq,Maxx Power - 33 Max Verstappen"), 
                   "Carte Blanq,Maxx Power - 33 Max Verstappen");
        assert_eq!(StringUtils::safe_filename_pinyin("Carte Blanq,Maxx Power - 33 Max Verstappen", 0), 
                   "Carte_Blanq_Maxx_Power_-_33_Max_Verstappen");
        
        // 测试中英混合
        assert_eq!(StringUtils::chinese_to_pinyin("音乐Music"), "yinyueMusic");
        assert_eq!(StringUtils::chinese_to_pinyin("Beautiful音乐"), "Beautifulyinyue");
        
        // 测试纯中文
        assert_eq!(StringUtils::chinese_to_pinyin("音乐"), "yinyue");
        assert_eq!(StringUtils::chinese_to_pinyin("歌曲"), "gequ");
    }
}
