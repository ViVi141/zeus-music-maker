/*!
 * 构建脚本 - 处理Windows资源
 */

fn main() {
    // 仅在Windows平台上处理资源
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=favicon.ico");
        
        // 使用winres来处理Windows资源
        if let Err(e) = winres::WindowsResource::new()
            .set_icon("favicon.ico")
            .set("ProductName", "Zeus Music Maker")
            .set("FileDescription", "Zeus Music Maker - Arma 3 Music Mod Generator")
            .set("CompanyName", "ViVi141")
            .set("LegalCopyright", "Copyright (c) 2025 ViVi141")
            .set("ProductVersion", "1.0.0")
            .set("FileVersion", "1.0.0.0")
            .set("Subsystem", "windows")
            .set("Console", "false")
            .compile()
        {
            eprintln!("Failed to compile Windows resources: {}", e);
        }
        
        // 强制设置链接器参数
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
}
