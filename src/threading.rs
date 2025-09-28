use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use log::{info, warn};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::audio_decrypt::AudioDecryptManager;
use crate::paa_converter::{PaaConverter, PaaOptions};
use crate::audio_converter::AudioConverter;
use crate::video_converter::VideoConverter;
use crate::ffmpeg_downloader::FFmpegDownloader;
use crate::parallel_converter::{ParallelConverter, ParallelConfig, ProgressUpdate};
use crate::video_chunk_parallel_processor::{VideoChunkParallelProcessor, ChunkProgressUpdate};
use crate::video_chunk_converter::VideoChunkConfig;

/// 任务消息
#[derive(Debug, Clone)]
pub enum TaskMessage {
    /// 更新进度
    UpdateProgress {
        current_file: usize,
        filename: String,
    },
    /// 任务完成
    TaskCompleted {
        success_count: usize,
        error_count: usize,
        results: Vec<String>,
    },
    /// FFmpeg下载进度更新
    FFmpegDownloadProgress {
        progress: f64,
        status: String,
    },
    /// FFmpeg下载完成
    FFmpegDownloadCompleted {
        success: bool,
        message: String,
    },
    /// 并行转换进度更新
    ParallelProgressUpdate(ProgressUpdate),
    /// 分片转换进度更新
    ChunkProgressUpdate(ChunkProgressUpdate),
}

/// 多线程任务处理器
pub struct ThreadedTaskProcessor {
    /// 进度更新发送器
    progress_sender: Sender<TaskMessage>,
    /// 进度更新接收器
    progress_receiver: Receiver<TaskMessage>,
    /// 取消标志
    cancel_flag: Arc<Mutex<bool>>,
    /// 并行转换器
    parallel_converter: Option<ParallelConverter>,
}

impl ThreadedTaskProcessor {
    pub fn new() -> Self {
        // 增大通道缓冲区以提高并发性能
        let (progress_sender, progress_receiver) = bounded(5000);
        Self {
            progress_sender,
            progress_receiver,
            cancel_flag: Arc::new(Mutex::new(false)),
            parallel_converter: None,
        }
    }

    /// 处理音频解密任务
    pub fn process_audio_decrypt(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();

        thread::spawn(move || {
            let mut success_count = 0;
            let mut error_count = 0;
            let mut results = Vec::new();

            for (i, input_path) in files.iter().enumerate() {
                // 检查取消标志
                if *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                }) {
                    info!("音频解密任务被取消");
                    // 立即发送取消完成消息
                    let _ = progress_sender.send(TaskMessage::TaskCompleted {
                        success_count,
                        error_count,
                        results: vec!["任务被用户取消".to_string()],
                    });
                    return;
                }

                // 发送进度更新
                let filename = input_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if let Err(e) = progress_sender.send(TaskMessage::UpdateProgress {
                    current_file: i,
                    filename: filename.clone(),
                }) {
                    warn!("发送进度更新失败: {}", e);
                }

                // 处理文件
                let cancel_check = || *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                });
                let result = if AudioDecryptManager::is_kugou_file(input_path) {
                    match AudioDecryptManager::decrypt_kugou_file_with_cancel(input_path, &output_dir, &cancel_check) {
                        Ok(output_path) => {
                            success_count += 1;
                            Ok(format!("酷狗: {} -> {}", 
                                filename,
                                std::path::Path::new(&output_path).file_name().unwrap_or_default().to_string_lossy()
                            ))
                        }
                        Err(e) => {
                            error_count += 1;
                            Err(format!("酷狗: {} - {}", filename, e))
                        }
                    }
                } else if AudioDecryptManager::is_netease_file(input_path) {
                    match AudioDecryptManager::decrypt_netease_file(input_path, &output_dir) {
                        Ok(output_path) => {
                            success_count += 1;
                            Ok(format!("网易云: {} -> {}", 
                                filename,
                                std::path::Path::new(&output_path).file_name().unwrap_or_default().to_string_lossy()
                            ))
                        }
                        Err(e) => {
                            error_count += 1;
                            Err(format!("网易云: {} - {}", filename, e))
                        }
                    }
                } else {
                    error_count += 1;
                    Err(format!("不支持: {} - 不支持的音频格式", filename))
                };

                match result {
                    Ok(msg) => results.push(msg),
                    Err(msg) => results.push(msg),
                }
            }

            // 发送完成消息
            if let Err(e) = progress_sender.send(TaskMessage::TaskCompleted {
                success_count,
                error_count,
                results,
            }) {
                warn!("发送任务完成消息失败: {}", e);
            }
        });

        Ok(())
    }

    /// 处理PAA转换任务
    pub fn process_paa_convert(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
        options: PaaOptions,
    ) -> Result<()> {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();

        thread::spawn(move || {
            let mut success_count = 0;
            let mut error_count = 0;
            let mut results = Vec::new();

            for (i, input_path) in files.iter().enumerate() {
                // 检查取消标志
                if *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                }) {
                    info!("PAA转换任务被取消");
                    // 立即发送取消完成消息
                    let _ = progress_sender.send(TaskMessage::TaskCompleted {
                        success_count,
                        error_count,
                        results: vec!["任务被用户取消".to_string()],
                    });
                    return;
                }

                // 发送进度更新
                let filename = input_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if let Err(e) = progress_sender.send(TaskMessage::UpdateProgress {
                    current_file: i,
                    filename: filename.clone(),
                }) {
                    warn!("发送进度更新失败: {}", e);
                }

                // 处理文件
                if let Some(file_stem) = input_path.file_stem() {
                    let output_path = output_dir.join(format!("{}.paa", file_stem.to_string_lossy()));
                    
                    match PaaConverter::convert_image_to_paa_with_crop(
                        input_path, 
                        &output_path, 
                        options.clone(),
                        None
                    ) {
                        Ok(_) => {
                            success_count += 1;
                            results.push(format!("转换成功: {}", filename));
                            info!("PAA转换成功: {:?}", output_path);
                        }
                        Err(e) => {
                            error_count += 1;
                            results.push(format!("转换失败: {} - {}", filename, e));
                            warn!("PAA转换失败: {:?} - {}", input_path, e);
                        }
                    }
                }
            }

            // 发送完成消息
            if let Err(e) = progress_sender.send(TaskMessage::TaskCompleted {
                success_count,
                error_count,
                results,
            }) {
                warn!("发送任务完成消息失败: {}", e);
            }
        });

        Ok(())
    }

    /// 处理音频格式转换任务（并行版本）
    pub fn process_audio_convert_parallel(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        info!("使用并行转换处理音频文件: {} 个文件", files.len());
        
        // 创建并行转换器
        let mut config = ParallelConfig::default();
        
        // 根据文件数量和大小调整配置
        if files.len() > 10 {
            config.adjust_for_file_size(files.len(), 50.0); // 假设平均50MB
        }
        
        let parallel_converter = ParallelConverter::new(config);
        
        // 启动并行转换
        parallel_converter.convert_audio_files_parallel(files, output_dir)?;
        
        // 启动进度转发线程
        self.start_progress_forwarding(parallel_converter);
        
        Ok(())
    }
    
    /// 处理音频格式转换任务（串行版本，保持向后兼容）
    pub fn process_audio_convert(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();

        thread::spawn(move || {
            // 使用多线程运行时以提高并发性能
            let _rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
                warn!("创建Tokio运行时失败: {}", e);
                panic!("无法创建Tokio运行时");
            });
            let mut success_count = 0;
            let mut error_count = 0;
            let mut results = Vec::new();
            
            // 尝试创建转换器，如果失败则提示下载
            let converter = match AudioConverter::new() {
                Ok(conv) => conv,
                Err(e) => {
                    warn!("FFmpeg 未找到: {}", e);
                    let _ = progress_sender.send(TaskMessage::TaskCompleted {
                        success_count: 0,
                        error_count: files.len(),
                        results: vec![format!("FFmpeg 未找到: {}\n\n请使用软件的自动下载功能或手动安装 FFmpeg", e)],
                    });
                    return;
                }
            };

            for (i, input_path) in files.iter().enumerate() {
                // 检查取消标志
                if *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                }) {
                    info!("音频转换任务被取消");
                    let _ = progress_sender.send(TaskMessage::TaskCompleted {
                        success_count,
                        error_count,
                        results: vec!["任务被用户取消".to_string()],
                    });
                    return;
                }

                // 发送进度更新
                let filename = input_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if let Err(e) = progress_sender.send(TaskMessage::UpdateProgress {
                    current_file: i,
                    filename: filename.clone(),
                }) {
                    warn!("发送进度更新失败: {}", e);
                }

                // 生成输出路径（使用拼音风格重命名）
                if let Some(file_stem) = input_path.file_stem() {
                    // 使用拼音风格生成文件名
                    let pinyin_filename = crate::utils::string_utils::StringUtils::safe_filename_pinyin(
                        &file_stem.to_string_lossy(), 
                        i
                    );
                    let output_path = output_dir.join(format!("{}.ogg", pinyin_filename));
                    
                    // 执行转换
                    let cancel_check = || *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                });
                    match converter.convert_to_ogg_with_cancel(input_path, &output_path, &cancel_check) {
                        Ok(_) => {
                            success_count += 1;
                            results.push(format!("转换成功: {} -> {}.ogg", filename, pinyin_filename));
                            info!("音频转换成功: {:?}", output_path);
                        }
                        Err(e) => {
                            error_count += 1;
                            results.push(format!("转换失败: {} - {}", filename, e));
                            warn!("音频转换失败: {:?} - {}", input_path, e);
                        }
                    }
                } else {
                    error_count += 1;
                    results.push(format!("转换失败: {} - 无法获取文件名", filename));
                }
            }

            // 发送完成消息
            if let Err(e) = progress_sender.send(TaskMessage::TaskCompleted {
                success_count,
                error_count,
                results,
            }) {
                warn!("发送任务完成消息失败: {}", e);
            }
        });

        Ok(())
    }

    /// 处理视频格式转换任务（并行版本）
    pub fn process_video_convert_parallel(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        info!("使用并行转换处理视频文件: {} 个文件", files.len());
        
        // 创建并行转换器
        let mut config = ParallelConfig::default();
        
        // 视频转换通常更消耗资源，减少并发数
        config.max_threads = (config.max_threads / 2).max(2);
        
        // 根据文件数量和大小调整配置
        if files.len() > 5 {
            config.adjust_for_file_size(files.len(), 200.0); // 假设平均200MB
        }
        
        let parallel_converter = ParallelConverter::new(config);
        
        // 启动并行转换
        parallel_converter.convert_video_files_parallel(files, output_dir)?;
        
        // 启动进度转发线程
        self.start_progress_forwarding(parallel_converter);
        
        Ok(())
    }
    
    /// 处理视频格式转换任务（分片并行版本）
    pub fn process_video_convert_chunked(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        let progress_sender = self.progress_sender.clone();
        let _cancel_flag = self.cancel_flag.clone();

        // 创建分片配置
        let chunk_config = VideoChunkConfig::default();
        let chunk_processor = VideoChunkParallelProcessor::new(chunk_config);

        thread::spawn(move || {
            info!("开始分片并行视频转换: {} 个文件", files.len());

            // 启动分片并行转换
            if let Err(e) = chunk_processor.process_videos_parallel(files.clone(), output_dir, 5, 3) {
                warn!("分片并行视频转换失败: {}", e);
                let _ = progress_sender.send(TaskMessage::TaskCompleted {
                    success_count: 0,
                    error_count: files.len(),
                    results: vec![format!("分片并行视频转换失败: {}", e)],
                });
                return;
            }

            // 监听分片转换进度
            Self::monitor_chunk_progress(chunk_processor, progress_sender);
        });

        Ok(())
    }

    /// 监听分片转换进度
    fn monitor_chunk_progress(
        chunk_processor: VideoChunkParallelProcessor,
        progress_sender: Sender<TaskMessage>,
    ) {
        let receiver = chunk_processor.get_progress_receiver();
        
        while let Ok(update) = receiver.recv() {
            let _ = progress_sender.send(TaskMessage::ChunkProgressUpdate(update));
        }
    }

    /// 处理视频格式转换任务（串行版本，保持向后兼容）
    pub fn process_video_convert(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();

        thread::spawn(move || {
            // 使用多线程运行时以提高并发性能
            let _rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
                warn!("创建Tokio运行时失败: {}", e);
                panic!("无法创建Tokio运行时");
            });
            let mut success_count = 0;
            let mut error_count = 0;
            let mut results = Vec::new();
            
            // 尝试创建视频转换器，如果失败则提示下载
            let converter = match VideoConverter::new() {
                Ok(conv) => conv,
                Err(e) => {
                    warn!("FFmpeg 未找到: {}", e);
                    let _ = progress_sender.send(TaskMessage::TaskCompleted {
                        success_count: 0,
                        error_count: files.len(),
                        results: vec![format!("FFmpeg 未找到: {}\n\n请使用软件的自动下载功能或手动安装 FFmpeg", e)],
                    });
                    return;
                }
            };

            for (i, input_path) in files.iter().enumerate() {
                // 检查取消标志
                if *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                }) {
                    info!("视频转换任务被取消");
                    let _ = progress_sender.send(TaskMessage::TaskCompleted {
                        success_count,
                        error_count,
                        results: vec!["任务被用户取消".to_string()],
                    });
                    return;
                }

                // 发送进度更新
                let filename = input_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if let Err(e) = progress_sender.send(TaskMessage::UpdateProgress {
                    current_file: i,
                    filename: filename.clone(),
                }) {
                    warn!("发送进度更新失败: {}", e);
                }

                // 生成输出文件名（使用拼音风格重命名）
                let pinyin_filename = if let Some(file_stem) = input_path.file_stem() {
                    crate::utils::string_utils::StringUtils::safe_filename_pinyin(
                        &file_stem.to_string_lossy(), 
                        i
                    )
                } else {
                    format!("video{:03}", i)
                };
                let output_filename = pinyin_filename + ".ogv";
                let output_path = output_dir.join(output_filename);

                // 执行视频转换
                match converter.convert_to_ogv(input_path, &output_path) {
                    Ok(_) => {
                        success_count += 1;
                        results.push(format!("✓ 成功转换: {} -> {}", filename, output_path.display()));
                        info!("视频转换成功: {} -> {}", input_path.display(), output_path.display());
                    }
                    Err(e) => {
                        error_count += 1;
                        results.push(format!("✗ 转换失败: {} - {}", filename, e));
                        warn!("视频转换失败: {} - {}", input_path.display(), e);
                    }
                }
            }

            // 发送完成消息
            let final_message = if success_count > 0 && error_count == 0 {
                format!("视频转换全部成功！\n\n成功转换: {} 个文件\n输出目录: {}", success_count, output_dir.display())
            } else if success_count > 0 {
                format!("视频转换部分成功\n\n成功: {} 个文件\n失败: {} 个文件\n输出目录: {}", success_count, error_count, output_dir.display())
            } else {
                format!("视频转换全部失败\n\n失败: {} 个文件", error_count)
            };

            results.insert(0, final_message);

            if let Err(e) = progress_sender.send(TaskMessage::TaskCompleted {
                success_count,
                error_count,
                results,
            }) {
                warn!("发送任务完成消息失败: {}", e);
            }
        });

        Ok(())
    }

    /// 处理 FFmpeg 下载任务
    pub fn process_ffmpeg_download(&self) -> Result<()> {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            // 发送初始进度
            let _ = progress_sender.send(TaskMessage::FFmpegDownloadProgress {
                progress: 0.0,
                status: "准备下载 FFmpeg...".to_string(),
            });

            // 创建下载器
                    let downloader = match FFmpegDownloader::new_user_workspace_with_fallback() {
                Ok(downloader) => downloader,
                Err(e) => {
                    let _ = progress_sender.send(TaskMessage::FFmpegDownloadCompleted {
                        success: false,
                        message: format!("创建下载器失败: {}", e),
                    });
                    return;
                }
            };

            // 执行下载
            let result = rt.block_on(async {
                downloader.download_ffmpeg_with_fallback(|progress, status| {
                    // 检查取消标志
                    if *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                }) {
                        return Err(anyhow::anyhow!("下载被取消"));
                    }

                    // 发送进度更新
                    if let Err(e) = progress_sender.send(TaskMessage::FFmpegDownloadProgress {
                        progress,
                        status: status.to_string(),
                    }) {
                        warn!("发送下载进度失败: {}", e);
                    }

                    Ok(())
                }).await
            });

            // 发送完成消息
            match result {
                Ok(ffmpeg_path) => {
                    // 保存路径配置
                    if let Err(e) = FFmpegDownloader::save_ffmpeg_path(&ffmpeg_path) {
                        warn!("保存 FFmpeg 路径失败: {}", e);
                    }

                    let _ = progress_sender.send(TaskMessage::FFmpegDownloadCompleted {
                        success: true,
                        message: format!("FFmpeg 下载成功！\n路径: {}", ffmpeg_path.display()),
                    });
                }
                Err(e) => {
                    let _ = progress_sender.send(TaskMessage::FFmpegDownloadCompleted {
                        success: false,
                        message: format!("FFmpeg 下载失败: {}", e),
                    });
                }
            }
        });

        Ok(())
    }

    /// 获取进度接收器
    pub fn get_progress_receiver(&self) -> &Receiver<TaskMessage> {
        &self.progress_receiver
    }

    /// 启动进度转发线程
    fn start_progress_forwarding(&self, parallel_converter: ParallelConverter) {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();
        
        thread::spawn(move || {
            let receiver = parallel_converter.get_progress_receiver();
            
            while let Ok(update) = receiver.recv() {
                // 检查取消标志
                if *cancel_flag.lock().unwrap_or_else(|_| {
                    warn!("获取取消标志失败，假设任务被取消");
                    panic!("Mutex poisoned, cannot continue")
                }) {
                    info!("进度转发线程收到取消信号");
                    break;
                }
                
                // 转发进度更新
                if let Err(e) = progress_sender.send(TaskMessage::ParallelProgressUpdate(update)) {
                    warn!("转发进度更新失败: {}", e);
                    break;
                }
            }
            
            info!("进度转发线程退出");
        });
    }
    
    /// 取消当前任务
    pub fn cancel_task(&self) {
        *self.cancel_flag.lock().unwrap() = true;
        
        // 如果存在并行转换器，也取消它
        if let Some(ref converter) = self.parallel_converter {
            converter.cancel_all_tasks();
        }
        
        info!("任务取消信号已发送");
    }

    /// 等待所有任务完成（用于优雅关闭）
    pub fn wait_for_completion(&self, timeout_ms: u64) -> bool {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        
        while start_time.elapsed() < timeout {
            // 检查是否还有未完成的任务
            if let Ok(_) = self.progress_receiver.try_recv() {
                // 还有消息在处理，使用更短的等待时间提高响应性
                std::thread::sleep(std::time::Duration::from_millis(5));
            } else {
                // 没有更多消息，任务可能已完成
                break;
            }
        }
        
        start_time.elapsed() < timeout
    }

    /// 重置取消标志
    pub fn reset_cancel_flag(&self) {
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = false;
        } else {
            warn!("重置取消标志失败");
        }
    }
}

impl Default for ThreadedTaskProcessor {
    fn default() -> Self {
        Self::new()
    }
}
