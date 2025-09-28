/*!
 * 字符串工具模块
 * 提供安全的字符串处理功能
 */

use pinyin::ToPinyin;

/// 字符串工具
pub struct StringUtils;

impl StringUtils {
    /// 将多语言字符串转换为拉丁字母（支持中文、日语、俄语、西班牙语）
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
                // 中文字符转换为拼音
                let pinyin_result = c.to_pinyin();
                if let Some(pinyin) = pinyin_result {
                    result.push_str(&pinyin.plain());
                } else {
                    result.push_str(&format!("u{:04x}", c as u32));
                }
            } else if Self::is_japanese_kana(c) {
                // 日语假名转换为罗马字
                if let Some(romaji) = Self::hiragana_to_romaji(c) {
                    result.push_str(romaji);
                } else {
                    result.push_str(&format!("u{:04x}", c as u32));
                }
            } else if Self::is_russian_cyrillic(c) {
                // 俄语西里尔字母转换为拉丁字母
                if let Some(latin) = Self::cyrillic_to_latin(c) {
                    result.push_str(latin);
                } else {
                    result.push_str(&format!("u{:04x}", c as u32));
                }
            } else {
                // 处理西班牙语重音符号和其他字符
                let normalized = Self::remove_spanish_accents(c);
                if normalized != c {
                    result.push(normalized);
                } else {
                    // 对于其他非ASCII字符，保持原样
                    result.push(c);
                }
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

    /// 判断字符是否为日文假名
    fn is_japanese_kana(c: char) -> bool {
        let code = c as u32;
        // 平假名: 3040-309F, 片假名: 30A0-30FF
        (0x3040..=0x309F).contains(&code) || (0x30A0..=0x30FF).contains(&code)
    }


    /// 判断字符是否为俄语西里尔字母
    fn is_russian_cyrillic(c: char) -> bool {
        let code = c as u32;
        // 俄语西里尔字母: 0400-04FF
        (0x0400..=0x04FF).contains(&code)
    }

    /// 简单的日语假名转罗马字（平假名）
    fn hiragana_to_romaji(c: char) -> Option<&'static str> {
        match c {
            'あ' => Some("a"), 'い' => Some("i"), 'う' => Some("u"), 'え' => Some("e"), 'お' => Some("o"),
            'か' => Some("ka"), 'き' => Some("ki"), 'く' => Some("ku"), 'け' => Some("ke"), 'こ' => Some("ko"),
            'さ' => Some("sa"), 'し' => Some("shi"), 'す' => Some("su"), 'せ' => Some("se"), 'そ' => Some("so"),
            'た' => Some("ta"), 'ち' => Some("chi"), 'つ' => Some("tsu"), 'て' => Some("te"), 'と' => Some("to"),
            'な' => Some("na"), 'に' => Some("ni"), 'ぬ' => Some("nu"), 'ね' => Some("ne"), 'の' => Some("no"),
            'は' => Some("ha"), 'ひ' => Some("hi"), 'ふ' => Some("fu"), 'へ' => Some("he"), 'ほ' => Some("ho"),
            'ま' => Some("ma"), 'み' => Some("mi"), 'む' => Some("mu"), 'め' => Some("me"), 'も' => Some("mo"),
            'や' => Some("ya"), 'ゆ' => Some("yu"), 'よ' => Some("yo"),
            'ら' => Some("ra"), 'り' => Some("ri"), 'る' => Some("ru"), 'れ' => Some("re"), 'ろ' => Some("ro"),
            'わ' => Some("wa"), 'を' => Some("wo"), 'ん' => Some("n"),
            _ => None,
        }
    }

    /// 简单的俄语西里尔字母转拉丁字母
    fn cyrillic_to_latin(c: char) -> Option<&'static str> {
        match c {
            'А' => Some("A"), 'Б' => Some("B"), 'В' => Some("V"), 'Г' => Some("G"), 'Д' => Some("D"),
            'Е' => Some("E"), 'Ё' => Some("Yo"), 'Ж' => Some("Zh"), 'З' => Some("Z"), 'И' => Some("I"),
            'Й' => Some("Y"), 'К' => Some("K"), 'Л' => Some("L"), 'М' => Some("M"), 'Н' => Some("N"),
            'О' => Some("O"), 'П' => Some("P"), 'Р' => Some("R"), 'С' => Some("S"), 'Т' => Some("T"),
            'У' => Some("U"), 'Ф' => Some("F"), 'Х' => Some("Kh"), 'Ц' => Some("Ts"), 'Ч' => Some("Ch"),
            'Ш' => Some("Sh"), 'Щ' => Some("Shch"), 'Ъ' => Some(""), 'Ы' => Some("Y"), 'Ь' => Some(""),
            'Э' => Some("E"), 'Ю' => Some("Yu"), 'Я' => Some("Ya"),
            'а' => Some("a"), 'б' => Some("b"), 'в' => Some("v"), 'г' => Some("g"), 'д' => Some("d"),
            'е' => Some("e"), 'ё' => Some("yo"), 'ж' => Some("zh"), 'з' => Some("z"), 'и' => Some("i"),
            'й' => Some("y"), 'к' => Some("k"), 'л' => Some("l"), 'м' => Some("m"), 'н' => Some("n"),
            'о' => Some("o"), 'п' => Some("p"), 'р' => Some("r"), 'с' => Some("s"), 'т' => Some("t"),
            'у' => Some("u"), 'ф' => Some("f"), 'х' => Some("kh"), 'ц' => Some("ts"), 'ч' => Some("ch"),
            'ш' => Some("sh"), 'щ' => Some("shch"), 'ъ' => Some(""), 'ы' => Some("y"), 'ь' => Some(""),
            'э' => Some("e"), 'ю' => Some("yu"), 'я' => Some("ya"),
            _ => None,
        }
    }

    /// 去除西班牙语重音符号
    fn remove_spanish_accents(c: char) -> char {
        match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            'Á' | 'À' | 'Ä' | 'Â' => 'A',
            'É' | 'È' | 'Ë' | 'Ê' => 'E',
            'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
            'Ó' | 'Ò' | 'Ö' | 'Ô' => 'O',
            'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
            'Ñ' => 'N',
            'Ç' => 'C',
            _ => c,
        }
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

    #[test]
    fn test_multilingual_support() {
        // 测试日语假名
        assert_eq!(StringUtils::chinese_to_pinyin("こんにちは"), "konnichiwa");
        assert_eq!(StringUtils::chinese_to_pinyin("ありがとう"), "arigatou");
        
        // 测试俄语
        assert_eq!(StringUtils::chinese_to_pinyin("Привет"), "Privet");
        assert_eq!(StringUtils::chinese_to_pinyin("Москва"), "Moskva");
        
        // 测试西班牙语重音符号
        assert_eq!(StringUtils::chinese_to_pinyin("España"), "Espana");
        assert_eq!(StringUtils::chinese_to_pinyin("niño"), "nino");
        
        // 测试混合语言
        assert_eq!(StringUtils::chinese_to_pinyin("音乐こんにちはMusic"), "yinyuekonnichiwaMusic");
        assert_eq!(StringUtils::chinese_to_pinyin("Привет音乐"), "Privetyinyue");
    }
}
