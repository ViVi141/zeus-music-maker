use anyhow::{Context, Result};
use handlebars::Handlebars;
use log::{debug, info};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::models::{ProjectSettings, Track};
use crate::embedded::EMBEDDED_RESOURCES;

/// 模板数据
#[derive(Debug, Serialize)]
pub struct ConfigTemplateData {
    pub mod_name_no_spaces: String,
    pub mod_name: String,
    pub author_name: String,
    pub class_name: String,
}

#[derive(Debug, Serialize)]
pub struct ModTemplateData {
    pub mod_name: String,
    pub author_name: String,
}

#[derive(Debug, Serialize)]
pub struct TrackTemplateData {
    pub track_class: String,
    pub track_name: String,
    pub track_path: String,
    pub decibels: String,
    pub duration: u32,
    pub class_name: String,
}


/// 模板引擎
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // 注册模板
        Self::register_templates(&mut handlebars)?;

        Ok(Self { handlebars })
    }

    /// 注册所有模板
    fn register_templates(handlebars: &mut Handlebars) -> Result<()> {
        // 从嵌入资源获取config.cpp模板
        let config_template = EMBEDDED_RESOURCES.get_template("config")
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded config template"))?;
        handlebars
            .register_template_string("config", config_template)
            .context("注册config模板失败")?;

        // 从嵌入资源获取mod.cpp模板
        let mod_template = EMBEDDED_RESOURCES.get_template("mod")
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded mod template"))?;
        handlebars
            .register_template_string("mod", mod_template)
            .context("注册mod模板失败")?;

        // 从嵌入资源获取FileListWithMusicTracks.hpp模板
        let track_template = EMBEDDED_RESOURCES.get_template("FileListWithMusicTracks")
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded track template"))?;
        handlebars
            .register_template_string("track", track_template)
            .context("注册track模板失败")?;

        info!("模板注册完成");
        Ok(())
    }

    /// 生成config.cpp文件
    pub fn generate_config_cpp(&self, project: &ProjectSettings, _tracks: &[Track], _copied_files: &[String], _use_tags: bool, output_path: &Path) -> Result<()> {
        // 确保项目信息只包含ASCII字符
        let ascii_mod_name = project.mod_name.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace() {
                c
            } else {
                '_'
            })
            .collect::<String>();
        
        let ascii_author_name = project.author_name.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace() {
                c
            } else {
                '_'
            })
            .collect::<String>();

        let data = ConfigTemplateData {
            mod_name_no_spaces: project.mod_name_no_spaces(),
            mod_name: ascii_mod_name,
            author_name: ascii_author_name,
            class_name: project.class_name.clone(),
        };

        let content = self
            .handlebars
            .render("config", &data)
            .context("渲染config模板失败")?;

        // Arma 3 需要特定的文件格式：无BOM + Windows行结束符 + 文件结尾空行
        // 确保使用Windows行结束符（\r\n）并添加结尾空行
        let mut content_with_crlf = content.replace('\n', "\r\n");
        if !content_with_crlf.ends_with("\r\n") {
            content_with_crlf.push_str("\r\n");
        }
        let content_bytes = content_with_crlf.as_bytes();
        
        fs::write(output_path, content_bytes)
            .with_context(|| format!("写入config.cpp失败: {:?}", output_path))?;

        debug!("生成config.cpp: {:?}", output_path);
        Ok(())
    }

    /// 生成mod.cpp文件
    pub fn generate_mod_cpp(&self, project: &ProjectSettings, output_path: &Path) -> Result<()> {
        // 确保项目信息只包含ASCII字符
        let ascii_mod_name = project.mod_name.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace() {
                c
            } else {
                '_'
            })
            .collect::<String>();
        
        let ascii_author_name = project.author_name.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace() {
                c
            } else {
                '_'
            })
            .collect::<String>();

        let data = ModTemplateData {
            mod_name: ascii_mod_name,
            author_name: ascii_author_name,
        };

        let content = self
            .handlebars
            .render("mod", &data)
            .context("渲染mod模板失败")?;

        // Arma 3 需要特定的文件格式：无BOM + Windows行结束符 + 文件结尾空行
        // 确保使用Windows行结束符（\r\n）并添加结尾空行
        let mut content_with_crlf = content.replace('\n', "\r\n");
        if !content_with_crlf.ends_with("\r\n") {
            content_with_crlf.push_str("\r\n");
        }
        let content_bytes = content_with_crlf.as_bytes();
        
        fs::write(output_path, content_bytes)
            .with_context(|| format!("写入mod.cpp失败: {:?}", output_path))?;

        debug!("生成mod.cpp: {:?}", output_path);
        Ok(())
    }

    /// 生成FileListWithMusicTracks.hpp文件
    pub fn generate_tracks_hpp(
        &self,
        project: &ProjectSettings,
        tracks: &[Track],
        copied_files: &[String],
        use_tags: bool,
        output_path: &Path,
    ) -> Result<()> {
        let mut content = String::new();

        for (i, track) in tracks.iter().enumerate() {
            let track_name = if use_tags && !track.tag.is_empty() {
                format!("[{}] {}", track.tag, track.track_name)
            } else {
                track.track_name.clone()
            };

            // 确保轨道名称只包含ASCII字符
            let ascii_track_name = track_name.chars()
                .map(|c| if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace() {
                    c
                } else {
                    '_'
                })
                .collect::<String>();

            // 确保类名只包含ASCII字符
            let ascii_class_name = project.class_name.chars()
                .map(|c| if c.is_ascii_alphanumeric() {
                    c
                } else {
                    '_'
                })
                .collect::<String>();

            let track_class = format!("{}Song{}", ascii_class_name, i);
            // 使用重命名后的文件名
            let filename = copied_files.get(i).map(|s| s.as_str()).unwrap_or(&track.track_name);
            let track_path = format!("{}\\folderwithtracks\\{}", project.mod_name_no_spaces(), filename);
            let decibels = if track.decibels >= 0 {
                format!("+{}", track.decibels)
            } else {
                track.decibels.to_string()
            };

            let data = TrackTemplateData {
                track_class,
                track_name: ascii_track_name,
                track_path,
                decibels,
                duration: track.duration,
                class_name: ascii_class_name,
            };

            let track_content = self
                .handlebars
                .render("track", &data)
                .context("渲染track模板失败")?;

            content.push_str(&track_content);
            content.push('\n');
        }

        // Arma 3 需要特定的文件格式：无BOM + Windows行结束符 + 文件结尾空行
        // 确保使用Windows行结束符（\r\n）并添加结尾空行
        let mut content_with_crlf = content.replace('\n', "\r\n");
        if !content_with_crlf.ends_with("\r\n") {
            content_with_crlf.push_str("\r\n");
        }
        let content_bytes = content_with_crlf.as_bytes();
        
        fs::write(output_path, content_bytes)
            .with_context(|| format!("写入FileListWithMusicTracks.hpp失败: {:?}", output_path))?;

        debug!("生成FileListWithMusicTracks.hpp: {:?}", output_path);
        Ok(())
    }

    /// 生成所有配置文件
    pub fn generate_all_configs(
        &self,
        project: &ProjectSettings,
        tracks: &[Track],
        copied_files: &[String],
        use_tags: bool,
        mod_dir: &Path,
    ) -> Result<()> {
        // 生成config.cpp
        let config_path = mod_dir.join("config.cpp");
        self.generate_config_cpp(project, tracks, copied_files, use_tags, &config_path)?;

        // 生成mod.cpp
        let mod_path = mod_dir.join("mod.cpp");
        self.generate_mod_cpp(project, &mod_path)?;

        // 生成FileListWithMusicTracks.hpp
        let tracks_path = mod_dir.join("FileListWithMusicTracks.hpp");
        self.generate_tracks_hpp(project, tracks, copied_files, use_tags, &tracks_path)?;

        info!("生成所有配置文件完成");
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        // 如果模板引擎创建失败，返回一个基本的实例
        // 这样程序不会崩溃，但模板功能可能不可用
        Self::new().unwrap_or_else(|e| {
            eprintln!("警告: 无法创建模板引擎: {}", e);
            // 创建一个空的模板引擎作为后备
            Self {
                handlebars: Handlebars::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectSettings;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_config_template_data() {
        let project = ProjectSettings::default();
        let data = ConfigTemplateData {
            mod_name_no_spaces: project.mod_name_no_spaces(),
            mod_name: project.mod_name.clone(),
            author_name: project.author_name.clone(),
            class_name: project.class_name.clone(),
        };
        assert!(!data.mod_name.is_empty());
    }
}
