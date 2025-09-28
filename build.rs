/*!
 * 宙斯音乐制作器 - 构建脚本
 * 配置Windows资源文件和便携版设置
 */

use std::env;

fn main() {
    // 只在Windows平台设置资源
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        
        // 设置应用程序信息
        res.set("ProductName", "Zeus Music Maker");
        res.set("FileDescription", "Zeus Music Maker - Arma 3 Music/Video Mod Creator");
        res.set("CompanyName", "ViVi141");
        res.set("LegalCopyright", "Copyright (C) 2025 ViVi141");
        res.set("ProductVersion", "2.0.0.0");
        res.set("FileVersion", "2.0.0.0");
        
        // 设置应用程序类型为Windows GUI应用（无控制台窗口）
        res.set("ApplicationManifest", "app.manifest");
        
        // 暂时禁用图标嵌入，避免构建失败
        // TODO: 后续可以通过其他方式添加图标
        println!("cargo:warning=暂时跳过图标嵌入，专注于功能实现");
        
        // 编译资源文件
        match res.compile() {
            Ok(_) => {
                println!("cargo:warning=Windows资源文件编译成功");
            }
            Err(e) => {
                println!("cargo:warning=Windows资源文件编译失败: {}", e);
                // 不退出构建，继续编译
            }
        }
    }
    
    // 设置便携版相关环境变量
    println!("cargo:rustc-cfg=portable");
    
    // 重新构建条件
    println!("cargo:rerun-if-changed=favicon.ico");
    println!("cargo:rerun-if-changed=assets/zeus_music_maker.png");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=app.manifest");
}
