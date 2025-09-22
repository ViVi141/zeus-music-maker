/*!
 * 嵌入资源模块
 * 将所有外部资源嵌入到可执行文件中
 */

use rust_embed::RustEmbed;

/// 嵌入的模板文件
#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;

/// 嵌入的资产文件
#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

/// 嵌入的库文件
#[derive(RustEmbed)]
#[folder = "lib/"]
pub struct Libraries;

/// 嵌入资源管理器
pub struct EmbeddedResources;

impl EmbeddedResources {
    /// 获取模板内容
    pub fn get_template(&self, name: &str) -> Option<String> {
        let filename = format!("{}.txt", name);
        Templates::get(&filename)
            .and_then(|file| String::from_utf8(file.data.into_owned()).ok())
    }

    /// 获取资源文件内容
    pub fn get_asset(&self, name: &str) -> Option<Vec<u8>> {
        Assets::get(name).map(|file| file.data.into_owned())
    }

    /// 获取库文件内容
    pub fn get_library(&self, name: &str) -> Option<Vec<u8>> {
        Libraries::get(name).map(|file| file.data.into_owned())
    }

    /// 提取库文件到临时目录
    pub fn extract_library_to_temp(&self, name: &str) -> Option<std::path::PathBuf> {
        if let Some(data) = self.get_library(name) {
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(name);
            
            if std::fs::write(&temp_path, data).is_ok() {
                return Some(temp_path);
            }
        }
        None
    }


    /// 获取酷狗密钥数据
    pub fn get_kugou_key(&self) -> Option<Vec<u8>> {
        self.get_asset("kugou_key.xz")
    }

    /// 获取应用图标
    pub fn get_app_icon(&self) -> Option<Vec<u8>> {
        self.get_asset("zeus_music_maker.png")
    }

}

/// 全局嵌入资源实例
pub static EMBEDDED_RESOURCES: EmbeddedResources = EmbeddedResources;
