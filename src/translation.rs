/*!
 * Google Translate 非官方免费接口（client=gtx）
 * 参考: https://github.com/ViVi141/ts3-chinese-translator
 */

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use log::warn;
use reqwest::blocking::Client;

use crate::utils::string_utils::StringUtils;

const GOOGLE_TRANSLATE_URL: &str = "https://translate.googleapis.com/translate_a/single";
const REQUEST_TIMEOUT_SECS: u64 = 15;

/// Google Translate 免费接口客户端（带内存缓存）
pub struct GoogleTranslateClient {
    client: Client,
    cache: HashMap<(String, String), String>,
}

impl GoogleTranslateClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("ZeusMusicMaker/2.1")
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            cache: HashMap::new(),
        })
    }

    /// 翻译文本到目标语言（源语言 auto 自动检测）
    pub fn translate(&mut self, text: &str, target_lang: &str) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        let cache_key = (text.to_string(), target_lang.to_string());
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let response = self
            .client
            .get(GOOGLE_TRANSLATE_URL)
            .query(&[
                ("client", "gtx"),
                ("sl", "auto"),
                ("tl", target_lang),
                ("dt", "t"),
                ("q", text),
            ])
            .send()
            .context("Google 翻译请求失败")?;

        if !response.status().is_success() {
            anyhow::bail!("Google 翻译 HTTP 错误: {}", response.status());
        }

        let body = response
            .text()
            .context("读取 Google 翻译响应失败")?;

        let translated = parse_google_translate_response(&body)?;
        self.cache.insert(cache_key, translated.clone());
        Ok(translated)
    }

    pub fn translate_to_english(&mut self, text: &str) -> Result<String> {
        self.translate(text, "en")
    }
}

/// 解析 Google 返回的 JSON：[[["译文",...],...],...]
fn parse_google_translate_response(body: &str) -> Result<String> {
    let json: serde_json::Value =
        serde_json::from_str(body).context("Google 翻译响应不是有效 JSON")?;

    let mut result = String::new();
    if let Some(segments) = json.get(0).and_then(|value| value.as_array()) {
        for segment in segments {
            if let Some(text) = segment.get(0).and_then(|value| value.as_str()) {
                result.push_str(text);
            }
        }
    }

    if result.is_empty() {
        anyhow::bail!("无法从 Google 翻译响应中提取译文");
    }

    Ok(result)
}

/// 优先 Google 翻译，失败则回退拼音
pub fn english_display_text(
    text: &str,
    translator: Option<&mut GoogleTranslateClient>,
) -> String {
    if StringUtils::is_english_only(text) {
        return text.to_string();
    }

    if let Some(client) = translator {
        match client.translate_to_english(text) {
            Ok(translated) if !translated.trim().is_empty() => {
                return translated;
            }
            Ok(_) => {
                warn!("Google 翻译返回空结果，回退拼音: {}", text);
            }
            Err(error) => {
                warn!("Google 翻译失败，回退拼音: {} - {}", text, error);
            }
        }
    }

    StringUtils::to_ascii_safe_pinyin(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_google_translate_response() {
        let body = r#"[[["Hello world",null,null,0]],null,"en"]"#;
        let result = parse_google_translate_response(body).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_parse_google_translate_response_multi_segment() {
        let body = r#"[[["Hello ",null,null,0],["world",null,null,0]],null,"en"]"#;
        let result = parse_google_translate_response(body).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_english_display_text_without_translator() {
        let result = english_display_text("夜曲", None);
        assert!(!result.is_empty());
        assert_ne!(result, "夜曲");
    }
}
