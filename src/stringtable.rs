/*!
 * Arma 3 Stringtable.xml 生成
 * 方案 B：通过 $STR_ 键支持中文曲目名与多语言显示
 */

use std::path::Path;

use anyhow::{Context, Result};
use log::warn;

use crate::models::{ProjectSettings, Track};
use crate::translation::GoogleTranslateClient;
use crate::utils::string_utils::StringUtils;

/// Stringtable 键前缀（由 class_name 派生，仅 ASCII 大写）
pub fn localization_prefix(class_name: &str) -> String {
    let filtered: String = class_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if filtered.is_empty() {
        return "ZMM".to_string();
    }
    filtered.to_ascii_uppercase()
}

/// 模组名称键
pub fn key_mod_name(prefix: &str) -> String {
    format!("STR_{}_MOD_NAME", prefix)
}

/// 作者键
pub fn key_author(prefix: &str) -> String {
    format!("STR_{}_AUTHOR", prefix)
}

/// 音乐分类显示名键
pub fn key_music_class(prefix: &str) -> String {
    format!("STR_{}_MUSIC_CLASS", prefix)
}

/// 曲目键（index 从 0 开始）
pub fn key_track(prefix: &str, index: usize) -> String {
    format!("STR_{}_TRACK_{:03}", prefix, index)
}

/// 配置文件中使用的 $STR_ 引用
pub fn str_reference(key_id: &str) -> String {
    format!("${}", key_id)
}

fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

fn is_cyrillic(c: char) -> bool {
    matches!(c as u32, 0x0400..=0x04FF)
}

fn is_japanese(c: char) -> bool {
    let code = c as u32;
    matches!(code, 0x3040..=0x309F | 0x30A0..=0x30FF)
}

fn is_korean(c: char) -> bool {
    matches!(c as u32, 0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F)
}

fn contains_cjk(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF)
}

fn has_cjk_text(text: &str) -> bool {
    text.chars().any(contains_cjk)
}

fn offline_english_fallback(text: &str) -> String {
    if StringUtils::is_english_only(text) {
        text.to_string()
    } else {
        StringUtils::to_ascii_safe_pinyin(text)
    }
}

fn translate_or_fallback(
    translator: &mut Option<GoogleTranslateClient>,
    text: &str,
    target_lang: &str,
    fallback: &str,
) -> String {
    let Some(client) = translator else {
        return fallback.to_string();
    };

    match client.translate(text, target_lang) {
        Ok(translated) if !translated.trim().is_empty() => translated,
        Ok(_) => {
            warn!("Google 翻译返回空结果: {} -> {}", text, target_lang);
            fallback.to_string()
        }
        Err(error) => {
            warn!("Google 翻译失败: {} -> {}: {}", text, target_lang, error);
            fallback.to_string()
        }
    }
}

struct StringtableEntry {
    id: String,
    english: String,
    chinese_simp: String,
    chinese_trad: String,
    japanese: Option<String>,
    korean: Option<String>,
    russian: Option<String>,
    french: String,
    german: String,
    spanish: String,
    italian: String,
    polish: String,
    czech: String,
    portuguese: String,
    turkish: String,
}

impl StringtableEntry {
    fn from_text(
        id: String,
        original: &str,
        translator: &mut Option<GoogleTranslateClient>,
        skip_translation: bool,
    ) -> Self {
        if skip_translation {
            return Self::same_text_all_languages(id, original);
        }

        let use_api = translator.is_some();
        let english_offline = offline_english_fallback(original);

        let english = if use_api {
            translate_or_fallback(translator, original, "en", &english_offline)
        } else {
            english_offline
        };

        let chinese_simp = if has_cjk_text(original) {
            original.to_string()
        } else if use_api {
            translate_or_fallback(translator, original, "zh-CN", original)
        } else {
            original.to_string()
        };

        let chinese_trad = if has_cjk_text(original) {
            original.to_string()
        } else if use_api {
            translate_or_fallback(translator, original, "zh-TW", original)
        } else {
            original.to_string()
        };

        let japanese = if use_api {
            Some(translate_or_fallback(translator, original, "ja", original))
        } else if original.chars().any(is_japanese) || has_cjk_text(original) {
            Some(original.to_string())
        } else {
            None
        };

        let korean = if use_api {
            Some(translate_or_fallback(translator, original, "ko", original))
        } else if original.chars().any(is_korean) {
            Some(original.to_string())
        } else {
            None
        };

        let russian = if use_api {
            Some(translate_or_fallback(translator, original, "ru", &english))
        } else if original.chars().any(is_cyrillic) {
            Some(original.to_string())
        } else if !StringUtils::is_english_only(original) {
            Some(original.to_string())
        } else {
            None
        };

        let french = if use_api {
            translate_or_fallback(translator, original, "fr", &english)
        } else {
            english.clone()
        };
        let german = if use_api {
            translate_or_fallback(translator, original, "de", &english)
        } else {
            english.clone()
        };
        let spanish = if use_api {
            translate_or_fallback(translator, original, "es", &english)
        } else {
            english.clone()
        };
        let italian = if use_api {
            translate_or_fallback(translator, original, "it", &english)
        } else {
            english.clone()
        };
        let polish = if use_api {
            translate_or_fallback(translator, original, "pl", &english)
        } else {
            english.clone()
        };
        let czech = if use_api {
            translate_or_fallback(translator, original, "cs", &english)
        } else {
            english.clone()
        };
        let portuguese = if use_api {
            translate_or_fallback(translator, original, "pt", &english)
        } else {
            english.clone()
        };
        let turkish = if use_api {
            translate_or_fallback(translator, original, "tr", &english)
        } else {
            english.clone()
        };

        Self {
            id,
            english,
            chinese_simp,
            chinese_trad,
            japanese,
            korean,
            russian,
            french,
            german,
            spanish,
            italian,
            polish,
            czech,
            portuguese,
            turkish,
        }
    }

    /// 内部重命名或不宜翻译时：所有语言标签使用同一文本，不调用 API
    fn same_text_all_languages(id: String, text: &str) -> Self {
        let english_offline = offline_english_fallback(text);
        Self {
            id,
            english: english_offline.clone(),
            chinese_simp: text.to_string(),
            chinese_trad: text.to_string(),
            japanese: if text.chars().any(is_japanese) || has_cjk_text(text) {
                Some(text.to_string())
            } else {
                None
            },
            korean: if text.chars().any(is_korean) {
                Some(text.to_string())
            } else {
                None
            },
            russian: if text.chars().any(is_cyrillic) {
                Some(text.to_string())
            } else if !StringUtils::is_english_only(text) {
                Some(text.to_string())
            } else {
                None
            },
            french: english_offline.clone(),
            german: english_offline.clone(),
            spanish: english_offline.clone(),
            italian: english_offline.clone(),
            polish: english_offline.clone(),
            czech: english_offline.clone(),
            portuguese: english_offline.clone(),
            turkish: english_offline,
        }
    }

    fn append_xml(&self, out: &mut String) {
        out.push_str("            <Key ID=\"");
        out.push_str(&escape_xml(&self.id));
        out.push_str("\">\n");

        out.push_str("                <English>");
        out.push_str(&escape_xml(&self.english));
        out.push_str("</English>\n");

        out.push_str("                <Chinesesimp>");
        out.push_str(&escape_xml(&self.chinese_simp));
        out.push_str("</Chinesesimp>\n");

        out.push_str("                <Chinese>");
        out.push_str(&escape_xml(&self.chinese_trad));
        out.push_str("</Chinese>\n");

        if let Some(japanese) = &self.japanese {
            out.push_str("                <Japanese>");
            out.push_str(&escape_xml(japanese));
            out.push_str("</Japanese>\n");
        }

        if let Some(korean) = &self.korean {
            out.push_str("                <Korean>");
            out.push_str(&escape_xml(korean));
            out.push_str("</Korean>\n");
        }

        if let Some(russian) = &self.russian {
            out.push_str("                <Russian>");
            out.push_str(&escape_xml(russian));
            out.push_str("</Russian>\n");
        }

        self.append_european_tag(out, "French", &self.french);
        self.append_european_tag(out, "German", &self.german);
        self.append_european_tag(out, "Spanish", &self.spanish);
        self.append_european_tag(out, "Italian", &self.italian);
        self.append_european_tag(out, "Polish", &self.polish);
        self.append_european_tag(out, "Czech", &self.czech);
        self.append_european_tag(out, "Portuguese", &self.portuguese);
        self.append_european_tag(out, "Turkish", &self.turkish);

        out.push_str("            </Key>\n");
    }

    fn append_european_tag(&self, out: &mut String, lang: &str, value: &str) {
        out.push_str("                <");
        out.push_str(lang);
        out.push_str(">");
        out.push_str(&escape_xml(value));
        out.push_str("</");
        out.push_str(lang);
        out.push_str(">\n");
    }
}

fn should_skip_translation(text: &str, flagged_internally_renamed: bool) -> bool {
    flagged_internally_renamed || StringUtils::is_internal_rename_result(text)
}

/// 生成音乐模组的 stringtable.xml 内容
pub fn generate_music_stringtable(
    project: &ProjectSettings,
    tracks: &[Track],
    use_tags: bool,
    translator: &mut Option<GoogleTranslateClient>,
) -> String {
    let prefix = localization_prefix(&project.class_name);
    let mut entries: Vec<StringtableEntry> = Vec::new();

    entries.push(StringtableEntry::from_text(
        key_mod_name(&prefix),
        &project.mod_name,
        translator,
        should_skip_translation(&project.mod_name, false),
    ));
    entries.push(StringtableEntry::from_text(
        key_author(&prefix),
        &project.author_name,
        translator,
        should_skip_translation(&project.author_name, false),
    ));
    entries.push(StringtableEntry::from_text(
        key_music_class(&prefix),
        &project.mod_name,
        translator,
        should_skip_translation(&project.mod_name, false),
    ));

    for (i, track) in tracks.iter().enumerate() {
        let display = if use_tags && !track.tag.is_empty() {
            format!("[{}] {}", track.tag, track.track_name)
        } else {
            track.track_name.clone()
        };

        entries.push(StringtableEntry::from_text(
            key_track(&prefix, i),
            &display,
            translator,
            should_skip_translation(&display, track.internally_renamed),
        ));
    }

    build_xml(&prefix, &entries)
}

/// 生成视频模组 mod/config 用的 stringtable（仅模组元信息）
pub fn generate_video_stringtable(
    project: &ProjectSettings,
    translator: &mut Option<GoogleTranslateClient>,
) -> String {
    let prefix = localization_prefix(&project.class_name);
    let entries = vec![
        StringtableEntry::from_text(
            key_mod_name(&prefix),
            &project.mod_name,
            translator,
            should_skip_translation(&project.mod_name, false),
        ),
        StringtableEntry::from_text(
            key_author(&prefix),
            &project.author_name,
            translator,
            should_skip_translation(&project.author_name, false),
        ),
    ];
    build_xml(&prefix, &entries)
}

fn build_xml(prefix: &str, entries: &[StringtableEntry]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<Project name=\"ZeusMusicMaker\">\n");
    out.push_str("    <Package name=\"");
    out.push_str(&escape_xml(prefix));
    out.push_str("\">\n");
    out.push_str("        <Container name=\"Strings\">\n");
    for entry in entries {
        entry.append_xml(&mut out);
    }
    out.push_str("        </Container>\n");
    out.push_str("    </Package>\n");
    out.push_str("</Project>\n");
    out
}

/// 写入 UTF-8 无 BOM 的 stringtable.xml
pub fn write_stringtable(path: &Path, content: &str) -> Result<()> {
    let mut bytes = content.replace('\n', "\r\n").into_bytes();
    if !bytes.ends_with(b"\r\n") {
        bytes.extend_from_slice(b"\r\n");
    }
    std::fs::write(path, bytes)
        .with_context(|| format!("写入 stringtable.xml 失败: {:?}", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ModType, ProjectSettings, Track};
    use std::path::PathBuf;

    #[test]
    fn test_localization_prefix() {
        assert_eq!(localization_prefix("MyMusicClass"), "MYMUSICCLASS");
        assert_eq!(localization_prefix("我的类"), "ZMM");
    }

    #[test]
    fn test_generate_music_stringtable_contains_keys() {
        let project = ProjectSettings {
            mod_name: "我的音乐包".to_string(),
            author_name: "作者".to_string(),
            class_name: "MyMusicClass".to_string(),
            mod_type: ModType::Music,
            ..Default::default()
        };
        let tracks = vec![Track::new(
            PathBuf::from("test.ogg"),
            "夜曲".to_string(),
            "MyMusicClass".to_string(),
        )];
        let xml = generate_music_stringtable(&project, &tracks, false, &mut None);
        assert!(xml.contains("STR_MYMUSICCLASS_MOD_NAME"));
        assert!(xml.contains("STR_MYMUSICCLASS_TRACK_000"));
        assert!(xml.contains("<Chinesesimp>夜曲</Chinesesimp>"));
        assert!(xml.contains("<French>"));
        assert!(xml.contains("<Japanese>"));
        assert!(xml.contains("encoding=\"utf-8\""));
    }

    #[test]
    fn test_generate_music_stringtable_skips_internal_rename() {
        let project = ProjectSettings {
            mod_name: "New Music Mod".to_string(),
            author_name: "Your username".to_string(),
            class_name: "NewMusicMod".to_string(),
            mod_type: ModType::Music,
            ..Default::default()
        };
        let mut track = Track::new(
            PathBuf::from("test.ogg"),
            "Alternative ending_ARM -_no_sansuujiaoshi".to_string(),
            "NewMusicMod".to_string(),
        );
        track.internally_renamed = true;
        let tracks = vec![track];
        let xml = generate_music_stringtable(&project, &tracks, false, &mut Some(
            GoogleTranslateClient::new().expect("client"),
        ));
        assert!(xml.contains("STR_NEWMUSICMOD_TRACK_000"));
        let track_section = xml
            .split("STR_NEWMUSICMOD_TRACK_000")
            .nth(1)
            .expect("track key");
        assert!(track_section.contains("sansuujiaoshi"));
        assert!(!track_section.contains("<Japanese>"));
    }

    #[test]
    fn test_str_reference() {
        assert_eq!(str_reference("STR_X_TRACK_000"), "$STR_X_TRACK_000");
    }
}
