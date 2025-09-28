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
                // 处理标点符号 - 保留常用的安全符号
                match c {
                    ' ' | '-' | '_' | '.' | ',' | '!' | '?' | ':' | ';' | '(' | ')' => result.push(c),
                    _ => result.push('_'),
                }
            } else if Self::is_chinese_char(c) {
                // 中文字符转换为拼音
                let pinyin_result = c.to_pinyin();
                if let Some(pinyin) = pinyin_result {
                    result.push_str(&pinyin.plain());
                } else {
                    // 无法转换的中文字符，使用下划线替代，避免Unicode编码
                    result.push('_');
                }
            } else if Self::is_japanese_kana(c) {
                // 日语假名转换为罗马字
                if let Some(romaji) = Self::hiragana_to_romaji(c) {
                    result.push_str(romaji);
                } else {
                    // 无法转换的假名，使用下划线替代
                    result.push('_');
                }
            } else if Self::is_russian_cyrillic(c) {
                // 俄语西里尔字母转换为拉丁字母
                if let Some(latin) = Self::cyrillic_to_latin(c) {
                    result.push_str(latin);
                } else {
                    // 无法转换的西里尔字母，使用下划线替代
                    result.push('_');
                }
            } else {
                // 处理西班牙语重音符号和其他字符
                let normalized = Self::remove_spanish_accents(c);
                if normalized != c {
                    result.push(normalized);
                } else if c.is_whitespace() {
                    // 保留空白字符
                    result.push(' ');
                } else {
                    // 对于其他非ASCII字符，使用下划线替代，避免Unicode编码
                    result.push('_');
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
            // 基本假名
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
            
            // 小字符
            'っ' => Some("tsu"), 'ぁ' => Some("a"), 'ぃ' => Some("i"), 'ぅ' => Some("u"), 'ぇ' => Some("e"), 'ぉ' => Some("o"),
            
            // 浊音
            'が' => Some("ga"), 'ぎ' => Some("gi"), 'ぐ' => Some("gu"), 'げ' => Some("ge"), 'ご' => Some("go"),
            'ざ' => Some("za"), 'じ' => Some("ji"), 'ず' => Some("zu"), 'ぜ' => Some("ze"), 'ぞ' => Some("zo"),
            'だ' => Some("da"), 'ぢ' => Some("ji"), 'づ' => Some("zu"), 'で' => Some("de"), 'ど' => Some("do"),
            'ば' => Some("ba"), 'び' => Some("bi"), 'ぶ' => Some("bu"), 'べ' => Some("be"), 'ぼ' => Some("bo"),
            
            // 半浊音
            'ぱ' => Some("pa"), 'ぴ' => Some("pi"), 'ぷ' => Some("pu"), 'ぺ' => Some("pe"), 'ぽ' => Some("po"),
            
            
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
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '.' {
                result.push(c);
            } else {
                result.push('_');
            }
        }
        
        // 清理多余的下划线和空格，生成更可读的文件名
        let cleaned = result
            .replace("__", "_")  // 替换连续下划线
            .replace(" _", "_")  // 替换空格+下划线
            .replace("_ ", "_")  // 替换下划线+空格
            .replace("  ", " ")  // 替换连续空格
            .replace("___", "_") // 替换三个连续下划线
            .trim_matches('_')   // 移除开头和结尾的下划线
            .trim()              // 移除开头和结尾的空格
            .to_string();
            
        // 如果清理后为空或只包含下划线，返回默认名称
        if cleaned.is_empty() || cleaned.chars().all(|c| c == '_') {
            "track".to_string()
        } else {
            // 确保文件名不会太长（限制在50个字符以内）
            if cleaned.len() > 50 {
                let truncated = cleaned.chars().take(47).collect::<String>();
                format!("{}...", truncated)
            } else {
                cleaned
            }
        }
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
            // 进一步优化文件名，确保可读性
            let optimized = Self::optimize_filename(&safe_name);
            if optimized.is_empty() || optimized.chars().all(|c| c == '_') {
                format!("track{:03}", index)
            } else {
                optimized
            }
        }
    }

    /// 优化文件名，提高可读性
    fn optimize_filename(input: &str) -> String {
        let mut result = input.to_string();
        
        // 移除开头和结尾的下划线
        result = result.trim_matches('_').to_string();
        
        // 将多个连续下划线替换为单个下划线
        while result.contains("__") {
            result = result.replace("__", "_");
        }
        
        // 移除开头和结尾的空格
        result = result.trim().to_string();
        
        // 如果结果为空或只包含下划线，返回空字符串
        if result.is_empty() || result.chars().all(|c| c == '_') {
            String::new()
        } else {
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