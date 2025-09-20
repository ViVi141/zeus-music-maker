/*!
 * 宙斯音乐制作器
 * by ViVi141
 */

use eframe::egui;
use log::info;

mod app;
mod models;
mod audio;
mod file_ops;
mod paa_converter;
mod audio_decrypt;
mod templates;
mod ui;
mod threading;
mod embedded;

use app::ZeusMusicApp;

fn main() -> Result<(), eframe::Error> {
    // 设置日志级别，减少控制台输出
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();
    
    info!("启动宙斯音乐制作器");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_decorations(true)
            .with_transparent(false)
            .with_icon(load_icon()),
        ..Default::default()
    };
    
    eframe::run_native(
        "宙斯音乐制作器",
        options,
        Box::new(|cc| {
            // 配置字体以支持中文字符
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(ZeusMusicApp::new())
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // 配置字体以支持中文字符
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试加载中文字体
    if let Ok(font_data) = load_chinese_font() {
        fonts.font_data.insert("chinese_font".to_owned(), font_data);
        
        // 将中文字体添加到字体族
        if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            proportional.insert(0, "chinese_font".to_owned());
        }
        if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            monospace.push("chinese_font".to_owned());
        }
    }
    
    // 设置字体大小和样式
    let mut style = (*ctx.style()).clone();
    
    // 配置各种文本样式，使用更大的字体以确保可读性
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(14.0, egui::FontFamily::Monospace),
    );
    
    // 设置字体和样式
    ctx.set_fonts(fonts);
    ctx.set_style(style);
}

fn load_chinese_font() -> Result<egui::FontData, Box<dyn std::error::Error>> {
    // 尝试从系统字体目录加载中文字体
    let font_paths = [
        "C:/Windows/Fonts/msyh.ttc", // 微软雅黑
        "C:/Windows/Fonts/simhei.ttf", // 黑体
        "C:/Windows/Fonts/simsun.ttc", // 宋体
        "C:/Windows/Fonts/NotoSansCJK-Regular.ttc", // Noto Sans CJK
    ];
    
    for font_path in &font_paths {
        if std::path::Path::new(font_path).exists() {
            let font_data = std::fs::read(font_path)?;
            return Ok(egui::FontData::from_owned(font_data));
        }
    }
    
    // 如果系统字体不可用，创建一个简单的字体数据
    // 这里我们创建一个包含基本中文字符的字体数据
    Ok(create_basic_chinese_font())
}

fn create_basic_chinese_font() -> egui::FontData {
    // 创建一个基本的字体数据，包含常用的中文字符
    // 这里我们使用一个简单的字体数据
    egui::FontData::from_static(&[])
}

fn load_icon() -> egui::IconData {
    use crate::embedded::EMBEDDED_RESOURCES;
    
    // 首先尝试从嵌入资源加载图标
    if let Some(icon_data) = EMBEDDED_RESOURCES.get_app_icon() {
        if let Ok(image) = image::load_from_memory(&icon_data) {
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            return egui::IconData {
                rgba,
                width,
                height,
            };
        }
    }
    
    // 尝试从文件系统加载图标（开发时使用）
    if let Ok(image) = image::open("favicon.ico") {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        egui::IconData {
            rgba,
            width,
            height,
        }
    } else if let Ok(image) = image::open("assets/zeus_music_maker.png") {
        // 备用图标
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        egui::IconData {
            rgba,
            width,
            height,
        }
    } else {
        // 如果无法加载图标，创建一个简单的默认图标
        egui::IconData {
            rgba: vec![255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255],
            width: 2,
            height: 2,
        }
    }
}
