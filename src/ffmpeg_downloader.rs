use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use log::{info, warn};
use reqwest;
use indicatif::{ProgressBar, ProgressStyle};

/// FFmpeg 下载器
pub struct FFmpegDownloader {
    download_url: String,
    output_path: PathBuf,
}

impl FFmpegDownloader {
    /// 创建新的下载器实例
    pub fn new(output_dir: &Path) -> Self {
        // 使用最佳下载源（优先中国镜像）
        let download_url = Self::get_best_download_url();
        let output_path = output_dir.join("ffmpeg.exe");
        
        Self {
            download_url,
            output_path,
        }
    }

    /// 获取最佳下载URL（支持多个镜像源）
    fn get_best_download_url() -> String {
        // 优先使用中国友好的镜像源
        let urls = Self::get_all_download_urls();
        
        // 返回第一个URL（GitHub代理镜像2，最稳定）
        info!("使用下载源: {}", urls[0]);
        urls[0].clone()
    }

    /// 获取所有可用的下载URL
    fn get_all_download_urls() -> Vec<String> {
        vec![
            // GitHub代理镜像2（推荐，最稳定）
            "https://ghproxy.net/https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
            // GitHub官方（备用）
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
            // GitHub代理镜像1（备用2）
            "https://ghproxy.com/https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
            // GitHub代理镜像3（最后备用）
            "https://mirror.ghproxy.com/https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
        ]
    }

    /// 智能下载 FFmpeg（支持多源自动切换）
    pub async fn download_ffmpeg_with_fallback<F>(
        &self,
        progress_callback: F,
    ) -> Result<PathBuf>
    where
        F: Fn(f64, &str) -> Result<()>,
    {
        let urls = Self::get_all_download_urls();
        
        for (index, url) in urls.iter().enumerate() {
            let source_name = match index {
                0 => "GitHub代理镜像2 (推荐)",
                1 => "GitHub官方", 
                2 => "GitHub代理镜像1",
                3 => "GitHub代理镜像3",
                _ => "未知源",
            };
            
            info!("尝试从 {} 下载 FFmpeg: {}", source_name, url);
            
            // 发送初始进度
            if let Err(e) = progress_callback(0.0, &format!("正在连接 {}...", source_name)) {
                warn!("发送初始进度失败: {}", e);
            }
            
            // 创建临时下载器尝试下载
            let temp_downloader = FFmpegDownloader {
                download_url: url.clone(),
                output_path: self.output_path.clone(),
            };
            
            match temp_downloader.download_ffmpeg(&progress_callback).await {
                Ok(path) => {
                    info!("从 {} 下载成功", source_name);
                    return Ok(path);
                }
                Err(e) => {
                    warn!("从 {} 下载失败: {}", source_name, e);
                    if index < urls.len() - 1 {
                        info!("尝试下一个下载源...");
                        if let Err(e) = progress_callback(0.0, &format!("{} 失败，尝试下一个源...", source_name)) {
                            warn!("发送进度更新失败: {}", e);
                        }
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("所有下载源都失败了，请检查网络连接或手动下载 FFmpeg"))
    }
    
    /// 获取用户工作空间目录
    pub fn get_user_workspace() -> Result<PathBuf> {
        let documents_dir = dirs::document_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户文档目录"))?;
        
        let workspace_dir = documents_dir.join("ZeusMusicMaker");
        
        // 创建工作空间目录
        fs::create_dir_all(&workspace_dir)
            .with_context(|| format!("无法创建工作空间目录: {:?}", workspace_dir))?;
        
        info!("用户工作空间目录: {:?}", workspace_dir);
        Ok(workspace_dir)
    }
    
    /// 获取 FFmpeg 存储目录
    pub fn get_ffmpeg_directory() -> Result<PathBuf> {
        let workspace = Self::get_user_workspace()?;
        let ffmpeg_dir = workspace.join("ffmpeg");
        
        // 创建 FFmpeg 目录
        fs::create_dir_all(&ffmpeg_dir)
            .with_context(|| format!("无法创建 FFmpeg 目录: {:?}", ffmpeg_dir))?;
        
        Ok(ffmpeg_dir)
    }
    

    /// 创建用户工作空间下载器（支持多源）
    pub fn new_user_workspace_with_fallback() -> Result<Self> {
        let ffmpeg_dir = Self::get_ffmpeg_directory()?;
        info!("FFmpeg 将下载到: {:?} (支持多源)", ffmpeg_dir);
        Ok(Self::new(&ffmpeg_dir))
    }
    
    /// 检查 FFmpeg 是否已存在且可用
    pub fn is_ffmpeg_available(ffmpeg_path: &Path) -> bool {
        if !ffmpeg_path.exists() {
            return false;
        }
        
        // 尝试运行 ffmpeg -version 来验证
        let mut cmd = std::process::Command::new(ffmpeg_path);
        cmd.arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
            
        // 在 Windows 上隐藏命令行窗口
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let result = cmd.status();
            
        result.map(|status| status.success()).unwrap_or(false)
    }
    
    /// 下载 FFmpeg
    pub async fn download_ffmpeg<F>(
        &self,
        progress_callback: F,
    ) -> Result<PathBuf>
    where
        F: Fn(f64, &str) -> Result<()>,
    {
        info!("开始下载 FFmpeg...");
        
        // 创建输出目录
        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 创建 HTTP 客户端
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5分钟超时
            .connect_timeout(std::time::Duration::from_secs(30)) // 30秒连接超时
            .tcp_keepalive(std::time::Duration::from_secs(60)) // TCP保活
            .pool_max_idle_per_host(10) // 连接池优化
            .build()?;
        
        // 发送请求获取文件大小
        let response = client.head(&self.download_url).send().await?;
        let total_size = response.headers()
            .get("content-length")
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse::<u64>().ok())
            .unwrap_or(0);
        
        info!("FFmpeg 文件大小: {} bytes", total_size);
        
        // 下载文件
        let mut response = client.get(&self.download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("下载失败: HTTP {}", response.status()));
        }
        
        // 创建临时文件
        let temp_path = self.output_path.with_extension("tmp");
        let file = fs::File::create(&temp_path)?;
        let mut downloaded: u64 = 0;
        
        // 创建进度条
        let progress_bar = if total_size > 0 {
            ProgressBar::new(total_size)
        } else {
            ProgressBar::new_spinner()
        };
        
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        // 发送初始进度
        if let Err(e) = progress_callback(0.0, "开始下载...") {
            warn!("发送初始进度失败: {}", e);
        }
        
        // 下载数据块
        let mut chunk_count = 0;
        // 使用缓冲写入以提高I/O效率
        use std::io::{BufWriter, Write};
        let mut writer = BufWriter::with_capacity(64 * 1024, file); // 64KB 缓冲区
        
        while let Some(chunk) = response.chunk().await? {
            writer.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            chunk_count += 1;
            
            // 更新进度
            progress_bar.set_position(downloaded);
            
            // 每下载 100KB 或每 10 个块调用一次回调，提供更频繁的进度更新
            if chunk_count % 10 == 0 || downloaded % (100 * 1024) == 0 {
                let progress = if total_size > 0 {
                    (downloaded as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                };
                
                let status = if total_size > 0 {
                    format!("下载中... {:.1}% ({}/{} bytes)", progress, downloaded, total_size)
                } else {
                    format!("下载中... {} bytes", downloaded)
                };
                
                if let Err(e) = progress_callback(progress, &status) {
                    warn!("进度回调失败: {}", e);
                }
            }
        }
        
        // 确保所有数据都写入文件
        writer.flush()?;
        progress_bar.finish_with_message("下载完成");
        
        // 解压文件
        info!("开始解压 FFmpeg...");
        self.extract_ffmpeg(&temp_path)?;
        
        // 删除临时文件
        fs::remove_file(&temp_path)?;
        
        // 验证下载的文件
        if Self::is_ffmpeg_available(&self.output_path) {
            info!("FFmpeg 下载并验证成功: {:?}", self.output_path);
            Ok(self.output_path.clone())
        } else {
            Err(anyhow::anyhow!("下载的 FFmpeg 文件无效"))
        }
    }
    
    /// 解压 FFmpeg 文件
    fn extract_ffmpeg(&self, zip_path: &Path) -> Result<()> {
        use std::io::Read;
        
        let file = fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        // 查找 ffmpeg.exe 文件
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let filename = file.name();
            
            if filename.ends_with("ffmpeg.exe") {
                info!("找到 FFmpeg 可执行文件: {}", filename);
                
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                
                // 写入到目标位置
                fs::write(&self.output_path, &buffer)?;
                
                // 设置执行权限（在 Windows 上通常不需要，但为了兼容性）
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&self.output_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&self.output_path, perms)?;
                }
                
                return Ok(());
            }
        }
        
        Err(anyhow::anyhow!("在 ZIP 文件中未找到 ffmpeg.exe"))
    }
    
    /// 获取 FFmpeg 信息
    pub fn get_ffmpeg_info() -> FFmpegInfo {
        FFmpegInfo {
            name: "FFmpeg".to_string(),
            version: "Latest (GPL Build)".to_string(),
            description: "FFmpeg 是一个功能强大的开源音视频处理工具，支持几乎所有主流格式".to_string(),
            download_size: "约 184MB (压缩包)".to_string(),
            features: vec![
                "支持几乎所有音视频格式 (MP4, AVI, MOV, FLV, OGG等)".to_string(),
                "高质量编码和解码，支持多种编码器".to_string(),
                "快速批量转换处理".to_string(),
                "开源免费，社区活跃".to_string(),
                "专业级音视频处理能力".to_string(),
            ],
        }
    }
    
    
    /// 保存 FFmpeg 路径到配置文件
    pub fn save_ffmpeg_path(ffmpeg_path: &Path) -> Result<()> {
        let workspace = Self::get_user_workspace()?;
        let config_file = workspace.join("ffmpeg_path.txt");
        
        let path_str = ffmpeg_path.to_string_lossy().to_string();
        fs::write(&config_file, path_str)
            .with_context(|| format!("无法保存 FFmpeg 路径配置: {:?}", config_file))?;
        
        info!("FFmpeg 路径已保存: {:?} -> {:?}", ffmpeg_path, config_file);
        Ok(())
    }
    
}

/// FFmpeg 信息
#[derive(Debug, Clone)]
pub struct FFmpegInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub download_size: String,
    pub features: Vec<String>,
}


