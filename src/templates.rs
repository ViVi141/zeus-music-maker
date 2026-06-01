use anyhow::{Context, Result};
use handlebars::Handlebars;
use log::{debug, info, warn};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::embedded::EMBEDDED_RESOURCES;
use crate::models::{ProjectSettings, Track};
use crate::stringtable::{
    generate_music_stringtable, generate_video_stringtable, key_author, key_mod_name,
    key_music_class, key_track, localization_prefix, str_reference, write_stringtable,
};
use crate::translation::GoogleTranslateClient;

/// 模板数据
#[derive(Debug, Serialize)]
pub struct ConfigTemplateData {
    pub mod_name_no_spaces: String,
    pub mod_name: String,
    pub author_name: String,
    pub class_name: String,
    pub music_class_name: String,
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

        Self::register_templates(&mut handlebars)?;

        Ok(Self { handlebars })
    }

    fn register_templates(handlebars: &mut Handlebars) -> Result<()> {
        let config_template = EMBEDDED_RESOURCES
            .get_template("config")
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded config template"))?;
        handlebars
            .register_template_string("config", config_template)
            .context("注册config模板失败")?;

        let mod_template = EMBEDDED_RESOURCES
            .get_template("mod")
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded mod template"))?;
        handlebars
            .register_template_string("mod", mod_template)
            .context("注册mod模板失败")?;

        let track_template = EMBEDDED_RESOURCES
            .get_template("FileListWithMusicTracks")
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded track template"))?;
        handlebars
            .register_template_string("track", track_template)
            .context("注册track模板失败")?;

        info!("模板注册完成");
        Ok(())
    }

    fn write_arma_config_file(path: &Path, content: &str) -> Result<()> {
        let mut content_with_crlf = content.replace('\n', "\r\n");
        if !content_with_crlf.ends_with("\r\n") {
            content_with_crlf.push_str("\r\n");
        }
        fs::write(path, content_with_crlf.as_bytes())
            .with_context(|| format!("写入配置文件失败: {:?}", path))
    }

    fn mod_display_name(project: &ProjectSettings, use_stringtable: bool) -> String {
        if use_stringtable {
            str_reference(&key_mod_name(&localization_prefix(&project.class_name)))
        } else {
            crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&project.mod_name)
        }
    }

    fn author_display_name(project: &ProjectSettings, use_stringtable: bool) -> String {
        if use_stringtable {
            str_reference(&key_author(&localization_prefix(&project.class_name)))
        } else {
            crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&project.author_name)
        }
    }

    fn music_class_identifier(project: &ProjectSettings) -> String {
        crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&project.class_name)
    }

    /// 生成config.cpp文件
    pub fn generate_config_cpp(
        &self,
        project: &ProjectSettings,
        use_stringtable: bool,
        output_path: &Path,
    ) -> Result<()> {
        let prefix = localization_prefix(&project.class_name);
        let music_class_name = if use_stringtable {
            str_reference(&key_music_class(&prefix))
        } else {
            Self::mod_display_name(project, false)
        };

        let data = ConfigTemplateData {
            mod_name_no_spaces: project.mod_name_no_spaces(),
            mod_name: Self::mod_display_name(project, use_stringtable),
            author_name: Self::author_display_name(project, use_stringtable),
            class_name: project.class_name.clone(),
            music_class_name,
        };

        let content = self
            .handlebars
            .render("config", &data)
            .context("渲染config模板失败")?;

        Self::write_arma_config_file(output_path, &content)?;
        debug!("生成config.cpp: {:?}", output_path);
        Ok(())
    }

    /// 生成mod.cpp文件
    pub fn generate_mod_cpp(
        &self,
        project: &ProjectSettings,
        use_stringtable: bool,
        output_path: &Path,
    ) -> Result<()> {
        let data = ModTemplateData {
            mod_name: Self::mod_display_name(project, use_stringtable),
            author_name: Self::author_display_name(project, use_stringtable),
        };

        let content = self
            .handlebars
            .render("mod", &data)
            .context("渲染mod模板失败")?;

        Self::write_arma_config_file(output_path, &content)?;
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
        use_stringtable: bool,
        output_path: &Path,
    ) -> Result<()> {
        let prefix = localization_prefix(&project.class_name);
        let class_id = Self::music_class_identifier(project);
        let mut content = String::new();

        for (i, track) in tracks.iter().enumerate() {
            let track_name = if use_stringtable {
                str_reference(&key_track(&prefix, i))
            } else {
                let display = if use_tags && !track.tag.is_empty() {
                    format!("[{}] {}", track.tag, track.track_name)
                } else {
                    track.track_name.clone()
                };
                crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&display)
            };

            let track_class = format!("{}Song{}", class_id, i);
            let filename = copied_files.get(i).map(|s| s.as_str()).unwrap_or("track.ogg");
            let track_path = format!(
                "{}\\folderwithtracks\\{}",
                project.mod_name_no_spaces(),
                filename
            );
            let decibels = if track.decibels >= 0 {
                format!("+{}", track.decibels)
            } else {
                track.decibels.to_string()
            };

            let data = TrackTemplateData {
                track_class,
                track_name,
                track_path,
                decibels,
                duration: track.duration,
                class_name: class_id.clone(),
            };

            let track_content = self
                .handlebars
                .render("track", &data)
                .context("渲染track模板失败")?;

            content.push_str(&track_content);
            content.push('\n');
        }

        Self::write_arma_config_file(output_path, &content)?;
        debug!("生成FileListWithMusicTracks.hpp: {:?}", output_path);
        Ok(())
    }

    /// 生成视频模组的config.cpp文件
    pub fn generate_video_config_cpp(
        &self,
        project: &ProjectSettings,
        use_stringtable: bool,
        output_path: &Path,
    ) -> Result<()> {
        let author = Self::author_display_name(project, use_stringtable);
        let name = Self::mod_display_name(project, use_stringtable);

        let video_config_content = format!(
            "class CfgPatches\n{{\n    class {}\n    {{\n        units[] = {{}};\n        weapons[] = {{}};\n        requiredVersion = 0.1;\n        requiredAddons[] = {{}};\n        author = \"{}\";\n        name = \"{}\";\n    }};\n}}\n",
            project.class_name, author, name
        );

        Self::write_arma_config_file(output_path, &video_config_content)?;
        debug!("生成视频config.cpp: {:?}", output_path);
        Ok(())
    }

    /// 生成 stringtable.xml
    pub fn generate_stringtable_xml(
        &self,
        project: &ProjectSettings,
        tracks: &[Track],
        use_tags: bool,
        use_google_translate: bool,
        mod_dir: &Path,
    ) -> Result<()> {
        let mut translator = if use_google_translate {
            match GoogleTranslateClient::new() {
                Ok(client) => Some(client),
                Err(error) => {
                    warn!("Google 翻译客户端初始化失败，将使用拼音回退: {}", error);
                    None
                }
            }
        } else {
            None
        };

        let content = match project.mod_type {
            crate::models::ModType::Music => {
                generate_music_stringtable(project, tracks, use_tags, &mut translator)
            }
            crate::models::ModType::Video => {
                generate_video_stringtable(project, &mut translator)
            }
        };
        let path = mod_dir.join("stringtable.xml");
        write_stringtable(&path, &content)?;
        debug!("生成 stringtable.xml: {:?}", path);
        Ok(())
    }

    /// 生成所有配置文件
    pub fn generate_all_configs(
        &self,
        project: &ProjectSettings,
        tracks: &[Track],
        copied_files: &[String],
        use_tags: bool,
        use_stringtable: bool,
        use_google_translate: bool,
        mod_dir: &Path,
    ) -> Result<()> {
        if use_stringtable {
            self.generate_stringtable_xml(
                project,
                tracks,
                use_tags,
                use_google_translate,
                mod_dir,
            )?;
        }

        match project.mod_type {
            crate::models::ModType::Music => {
                let config_path = mod_dir.join("config.cpp");
                self.generate_config_cpp(project, use_stringtable, &config_path)?;

                let mod_path = mod_dir.join("mod.cpp");
                self.generate_mod_cpp(project, use_stringtable, &mod_path)?;

                let tracks_path = mod_dir.join("FileListWithMusicTracks.hpp");
                self.generate_tracks_hpp(
                    project,
                    tracks,
                    copied_files,
                    use_tags,
                    use_stringtable,
                    &tracks_path,
                )?;
            }
            crate::models::ModType::Video => {
                let config_path = mod_dir.join("config.cpp");
                self.generate_video_config_cpp(project, use_stringtable, &config_path)?;

                let mod_path = mod_dir.join("mod.cpp");
                self.generate_mod_cpp(project, use_stringtable, &mod_path)?;
            }
        }

        info!("生成所有配置文件完成");
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!("警告: 无法创建模板引擎: {}", e);
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
            music_class_name: project.mod_name.clone(),
        };
        assert!(!data.mod_name.is_empty());
    }
}
