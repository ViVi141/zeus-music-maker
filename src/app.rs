use eframe::egui;
use log::{info, warn};

use crate::models::AppState;
use crate::ui::UIComponents;
use crate::threading::ThreadedTaskProcessor;
use crate::parallel_converter::ProgressUpdate;

pub mod lifecycle;

/// 主应用程序
pub struct ZeusMusicApp {
    state: AppState,
    /// 多线程任务处理器
    task_processor: ThreadedTaskProcessor,
    /// 生命周期管理器
    lifecycle: lifecycle::AppLifecycle,
}

impl ZeusMusicApp {
    pub fn new() -> Self {
        info!("初始化Zeus Music Mod Generator");
        
        // 从配置文件加载状态
        let state = AppState::load_config();
        
        let mut app = Self {
            state,
            task_processor: ThreadedTaskProcessor::new(),
            lifecycle: lifecycle::AppLifecycle::new(),
        };
        
        // 首次启动时自动显示用户指导
        if app.state.is_first_launch {
            app.state.show_user_guide = true;
            app.state.is_first_launch = false;
            info!("首次启动，显示新用户指导");
        } else if app.state.auto_show_guide {
            app.state.show_user_guide = true;
            info!("自动显示用户指导");
        }
        
        app
    }
}

impl eframe::App for ZeusMusicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 渲染菜单栏
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            UIComponents::render_menu_bar(ui, &mut self.state);
        });

        // 渲染主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // 显示项目信息
                ui.horizontal(|ui| {
                    ui.label(format!("项目: {}", self.state.project.mod_name));
                    ui.separator();
                    ui.label(format!("作者: {}", self.state.project.author_name));
                    ui.separator();
                    ui.label(format!("轨道数: {}", self.state.track_count()));
                });

                ui.add_space(10.0);

                // 显示文件操作提示信息
                if let Some(ref message) = self.state.file_operation_message {
                    ui.colored_label(egui::Color32::from_rgb(0, 150, 0), message);
                    ui.add_space(5.0);
                }

                // 显示轨道列表
                UIComponents::render_track_list(ui, &mut self.state);
            });
        });

        // 渲染底部按钮
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            UIComponents::render_bottom_buttons(ui, &mut self.state);
        });

        // 处理多线程任务进度更新
        self.process_threaded_tasks();

        // 显示对话框
        UIComponents::show_project_settings_dialog(ctx, &mut self.state);
        UIComponents::show_export_dialog(ctx, &mut self.state);
        let uptime = self.get_uptime();
        UIComponents::show_about_dialog(ctx, &mut self.state, uptime);
        UIComponents::show_user_guide_dialog(ctx, &mut self.state);
        UIComponents::show_track_editor_dialog(ctx, &mut self.state);
        UIComponents::show_paa_converter_dialog(ctx, &mut self.state, Some(&mut self.task_processor));
        UIComponents::show_preview_dialog(ctx, &mut self.state);
        UIComponents::show_export_result_dialog(ctx, &mut self.state);
        UIComponents::show_track_count_dialog(ctx, &mut self.state);
        UIComponents::show_paa_result_dialog(ctx, &mut self.state);
        UIComponents::show_audio_decrypt_dialog(ctx, &mut self.state);
        UIComponents::show_audio_decrypt_result_dialog(ctx, &mut self.state);
        UIComponents::show_audio_converter_dialog(ctx, &mut self.state);
        UIComponents::show_audio_convert_result_dialog(ctx, &mut self.state);
        UIComponents::show_video_converter_dialog(ctx, &mut self.state);
        UIComponents::show_video_convert_result_dialog(ctx, &mut self.state);
        UIComponents::show_ffmpeg_download_dialog(ctx, &mut self.state);
        UIComponents::show_ffmpeg_plugin_dialog(ctx, &mut self.state);
        UIComponents::show_manual_path_selection_dialog(ctx, &mut self.state);
        UIComponents::show_progress_dialog(ctx, &mut self.state, &mut self.task_processor);
        
        // 检查是否需要执行音频解密
        if self.state.should_decrypt_audio {
            if let Some(ref output_dir) = self.state.audio_decrypt_output_directory {
                let output_dir = output_dir.clone();
                let selected_files = self.state.audio_decrypt_selected_files.clone();
                self.start_audio_decrypt_task(selected_files, output_dir);
            }
            self.state.should_decrypt_audio = false;
            self.state.show_audio_decrypt = false;
        }
        
        // 检查是否需要执行音频转换
        if self.state.should_convert_audio {
            if let Some(ref output_dir) = self.state.audio_convert_output_directory {
                let output_dir = output_dir.clone();
                let selected_files = self.state.audio_convert_selected_files.clone();
                self.start_audio_convert_task(selected_files, output_dir);
            }
            self.state.should_convert_audio = false;
            self.state.show_audio_converter = false;
        }
        
        // 检查是否需要执行视频转换
        if self.state.should_convert_video {
            if let Some(ref output_dir) = self.state.video_convert_output_directory {
                let output_dir = output_dir.clone();
                let selected_files = self.state.video_convert_selected_files.clone();
                self.start_video_convert_task(selected_files, output_dir);
            }
            self.state.should_convert_video = false;
            self.state.show_video_converter = false;
        }
        
        // 检查是否需要下载 FFmpeg
        if self.state.is_downloading_ffmpeg && !self.state.ffmpeg_download_started {
            self.start_ffmpeg_download_task();
        }
        
        // 如果有任务正在运行，请求持续重绘以确保UI实时更新
        // 使用更智能的重绘策略，避免过度重绘
        if self.state.task_manager.is_running() || 
           self.state.is_downloading_ffmpeg || 
           self.state.task_manager.show_progress {
            // 使用request_repaint_after来减少重绘频率
            ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("程序开始优雅关闭...");
        
        // 1. 保存配置文件
        if let Err(e) = self.state.save_config() {
            warn!("保存配置文件失败: {}", e);
        }
        
        // 2. 取消所有正在运行的任务
        self.task_processor.cancel_task();
        
        // 3. 等待任务完成（最多等待5秒）
        if !self.task_processor.wait_for_completion(5000) {
            warn!("任务未在超时时间内完成，继续关闭");
        }
        
        // 4. 清理资源
        self.cleanup_resources();
        
        // 5. 记录运行时间
        let uptime = self.lifecycle.get_uptime();
        info!("应用程序已关闭，运行时间: {:.2}秒", uptime.as_secs_f64());
        
        // 6. 正常退出
        std::process::exit(0);
    }
}

impl ZeusMusicApp {
    /// 处理多线程任务进度更新
    pub fn process_threaded_tasks(&mut self) {
        use crate::threading::TaskMessage;
        
        // 处理所有待处理的进度消息
        while let Ok(message) = self.task_processor.get_progress_receiver().try_recv() {
            match message {
                TaskMessage::UpdateProgress { current_file, filename } => {
                    if let Some(ref mut _task) = self.state.task_manager.current_task {
                        self.state.task_manager.update_progress(current_file, &filename);
                    }
                }
                TaskMessage::ParallelProgressUpdate(update) => {
                    self.handle_parallel_progress_update(update);
                }
                TaskMessage::ChunkProgressUpdate(update) => {
                    self.handle_chunk_progress_update(update);
                }
                TaskMessage::FFmpegDownloadProgress { progress, status } => {
                    self.state.ffmpeg_download_progress = progress;
                    // 添加调试日志
                    log::info!("FFmpeg 下载进度更新: {:.1}% - {}", progress, status);
                    self.state.ffmpeg_download_status = status;
                    // 注意：这里不能直接调用 ctx.request_repaint()，因为 ctx 不在作用域内
                    // egui 会在下一帧自动重绘，所以进度更新应该能正常显示
                }
                TaskMessage::FFmpegDownloadCompleted { success, message } => {
                    // 下载完成，重置所有下载相关标志
                    self.state.is_downloading_ffmpeg = false;
                    self.state.ffmpeg_download_started = false;
                    self.state.ffmpeg_download_progress = if success { 100.0 } else { 0.0 };
                    
                    if success {
                        self.state.ffmpeg_download_status = "下载完成！".to_string();
                        self.state.audio_convert_result = Some(message);
                        self.state.show_audio_convert_result = true;
                        // 不立即关闭下载对话框，让用户看到完成状态
                    } else {
                        self.state.ffmpeg_download_status = "下载失败！".to_string();
                        self.state.audio_convert_result = Some(message);
                        self.state.show_audio_convert_result = true;
                    }
                }
                TaskMessage::TaskCompleted { success_count, error_count, results } => {
                    self.state.task_manager.complete_task();
                    
                    // 根据任务类型处理结果
                    if let Some(ref task) = self.state.task_manager.task_history.last() {
                        match task.task_type {
                            crate::models::TaskType::AudioDecrypt => {
                                self.state.audio_decrypt_result = Some(format!(
                                    "音频解密完成！\n\n成功: {}\n失败: {}\n\n详细结果:\n{}",
                                    success_count,
                                    error_count,
                                    results.join("\n")
                                ));
                                self.state.show_audio_decrypt_result = true;
                            }
                            crate::models::TaskType::PaaConvert => {
                                self.state.paa_result = Some(format!(
                                    "PAA转换完成！\n\n成功: {}\n失败: {}\n\n详细结果:\n{}",
                                    success_count,
                                    error_count,
                                    results.join("\n")
                                ));
                                self.state.show_paa_result = true;
                            }
                            crate::models::TaskType::AudioConvert => {
                                self.state.audio_convert_result = Some(format!(
                                    "音频转换完成！\n\n成功: {}\n失败: {}\n\n详细结果:\n{}",
                                    success_count,
                                    error_count,
                                    results.join("\n")
                                ));
                                self.state.show_audio_convert_result = true;
                            }
                            crate::models::TaskType::VideoConvert => {
                                self.state.video_convert_result = Some(format!(
                                    "视频转换完成！\n\n成功: {}\n失败: {}\n\n详细结果:\n{}",
                                    success_count,
                                    error_count,
                                    results.join("\n")
                                ));
                                self.state.show_video_convert_result = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// 开始音频解密任务
    pub fn start_audio_decrypt_task(&mut self, files: Vec<std::path::PathBuf>, output_dir: std::path::PathBuf) {
        self.state.task_manager.start_task(crate::models::TaskType::AudioDecrypt, files.len());
        self.task_processor.reset_cancel_flag();
        
        if let Err(e) = self.task_processor.process_audio_decrypt(files, output_dir) {
            self.state.task_manager.fail_task(format!("启动音频解密任务失败: {}", e));
        }
    }

    /// 开始音频转换任务
    pub fn start_audio_convert_task(&mut self, files: Vec<std::path::PathBuf>, output_dir: std::path::PathBuf) {
        self.state.task_manager.start_task(crate::models::TaskType::AudioConvert, files.len());
        self.task_processor.reset_cancel_flag();
        
        // 优先使用并行转换，如果文件数量较少则使用串行转换
        if files.len() > 3 {
            info!("使用并行转换处理 {} 个音频文件", files.len());
            
            // 延迟启动并行转换，确保进度对话框先显示
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            if let Err(e) = self.task_processor.process_audio_convert_parallel(files, output_dir) {
                self.state.task_manager.fail_task(format!("启动并行音频转换任务失败: {}", e));
            }
        } else {
            info!("使用串行转换处理 {} 个音频文件", files.len());
            if let Err(e) = self.task_processor.process_audio_convert(files, output_dir) {
                self.state.task_manager.fail_task(format!("启动音频转换任务失败: {}", e));
            }
        }
    }

    /// 开始视频转换任务
    pub fn start_video_convert_task(&mut self, files: Vec<std::path::PathBuf>, output_dir: std::path::PathBuf) {
        self.state.task_manager.start_task(crate::models::TaskType::VideoConvert, files.len());
        self.task_processor.reset_cancel_flag();
        
        // 智能选择转换策略
        let total_files = files.len();
        let avg_file_size = self.calculate_average_file_size(&files);
        
        // 根据文件数量和大小选择最佳转换策略
        if total_files > 3 || avg_file_size > 100_000_000 { // 大于100MB或超过3个文件
            info!("使用分片并行转换处理 {} 个视频文件 (平均大小: {:.1}MB)", 
                  total_files, avg_file_size as f64 / 1_000_000.0);
            
            // 延迟启动分片转换，确保进度对话框先显示
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            if let Err(e) = self.task_processor.process_video_convert_chunked(files, output_dir) {
                self.state.task_manager.fail_task(format!("启动分片并行视频转换任务失败: {}", e));
            }
        } else if total_files > 2 {
            info!("使用并行转换处理 {} 个视频文件", total_files);
            
            // 延迟启动并行转换，确保进度对话框先显示
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            if let Err(e) = self.task_processor.process_video_convert_parallel(files, output_dir) {
                self.state.task_manager.fail_task(format!("启动并行视频转换任务失败: {}", e));
            }
        } else {
            info!("使用串行转换处理 {} 个视频文件", total_files);
            if let Err(e) = self.task_processor.process_video_convert(files, output_dir) {
                self.state.task_manager.fail_task(format!("启动视频转换任务失败: {}", e));
            }
        }
    }

    /// 计算文件平均大小
    fn calculate_average_file_size(&self, files: &[std::path::PathBuf]) -> u64 {
        if files.is_empty() {
            return 0;
        }
        
        let total_size: u64 = files.iter()
            .filter_map(|path| std::fs::metadata(path).ok())
            .map(|metadata| metadata.len())
            .sum();
            
        total_size / files.len() as u64
    }

    /// 开始 FFmpeg 下载任务
    pub fn start_ffmpeg_download_task(&mut self) {
        // 标记下载任务已启动
        self.state.ffmpeg_download_started = true;
        self.task_processor.reset_cancel_flag();
        
        if let Err(e) = self.task_processor.process_ffmpeg_download() {
            // 启动失败时才重置状态
            self.state.is_downloading_ffmpeg = false;
            self.state.ffmpeg_download_started = false;
            self.state.ffmpeg_download_status = format!("启动下载任务失败: {}", e);
            self.state.audio_convert_result = Some(format!("FFmpeg 下载失败: {}", e));
            self.state.show_audio_convert_result = true;
        }
    }

    /// 清理资源
    fn cleanup_resources(&mut self) {
        info!("开始清理资源...");
        
        // 清理任务处理器
        self.task_processor.cancel_task();
        
        // 清理状态
        self.state.tracks.clear();
        self.state.selected_track = None;
        
        // 清理UI状态
        self.state.show_project_settings = false;
        self.state.show_export_dialog = false;
        self.state.show_about = false;
        self.state.show_track_editor = false;
        self.state.show_paa_converter = false;
        self.state.show_audio_decrypt = false;
        
        info!("资源清理完成");
    }

    /// 获取应用程序运行时间
    pub fn get_uptime(&self) -> std::time::Duration {
        self.lifecycle.get_uptime()
    }
    
    /// 处理并行转换进度更新
    fn handle_parallel_progress_update(&mut self, update: ProgressUpdate) {
        match update {
            ProgressUpdate::TaskStarted { task_id, filename, total_tasks } => {
                info!("并行任务开始: {} ({}), 总计: {}", task_id, filename, total_tasks);
                
                // 更新任务管理器进度
                if let Some(ref mut task) = self.state.task_manager.current_task {
                    task.current_file = task_id + 1; // 显示当前正在处理的任务编号（从1开始）
                    task.current_filename = filename;
                    task.total_files = total_tasks;
                    // 任务开始时进度保持不变，等待任务完成时再更新
                }
            }
            ProgressUpdate::TaskCompleted { task_id, result, completed_count, total_tasks } => {
                info!("并行任务完成: {} ({}), 已完成: {}/{}", task_id, result.input_path().display(), completed_count, total_tasks);
                
                // 更新进度
                if let Some(ref mut task) = self.state.task_manager.current_task {
                    task.current_file = completed_count;
                    task.progress = completed_count as f32 / total_tasks as f32;
                    
                    // 根据结果更新任务信息
                    match result {
                        crate::parallel_converter::ConversionResult::Success { message, .. } => {
                            task.current_filename = format!("✓ {}", message);
                        }
                        crate::parallel_converter::ConversionResult::Error { error, .. } => {
                            task.current_filename = format!("✗ {}", error);
                        }
                    }
                    
                    // 更新处理速度
                    if let Some(start_time) = task.start_time {
                        let elapsed = start_time.elapsed().unwrap_or_default();
                        if elapsed.as_secs_f32() > 0.0 {
                            task.processing_speed = Some(completed_count as f32 / elapsed.as_secs_f32());
                        }
                        
                        // 估算剩余时间
                        if completed_count > 0 && completed_count < total_tasks {
                            let remaining_files = total_tasks - completed_count;
                            if let Some(speed) = task.processing_speed {
                                task.estimated_remaining = Some((remaining_files as f32 / speed) as u64);
                            }
                        }
                    }
                }
            }
            ProgressUpdate::AllTasksCompleted { success_count, error_count, total_duration, results } => {
                info!("所有并行任务完成: 成功={}, 失败={}, 耗时={:.2}秒", 
                    success_count, error_count, total_duration.as_secs_f64());
                
                // 完成任务
                self.state.task_manager.complete_task();
                
                // 根据任务类型处理结果
                if let Some(ref task) = self.state.task_manager.task_history.last() {
                    match task.task_type {
                        crate::models::TaskType::AudioConvert => {
                            let result_message = format!(
                                "并行音频转换完成！\n\n成功: {}\n失败: {}\n总耗时: {:.2}秒\n\n详细结果:\n{}",
                                success_count,
                                error_count,
                                total_duration.as_secs_f64(),
                                results.iter().map(|r| format!("• {}", r)).collect::<Vec<_>>().join("\n")
                            );
                            self.state.audio_convert_result = Some(result_message);
                            self.state.show_audio_convert_result = true;
                        }
                        crate::models::TaskType::VideoConvert => {
                            let result_message = format!(
                                "并行视频转换完成！\n\n成功: {}\n失败: {}\n总耗时: {:.2}秒\n\n详细结果:\n{}",
                                success_count,
                                error_count,
                                total_duration.as_secs_f64(),
                                results.iter().map(|r| format!("• {}", r)).collect::<Vec<_>>().join("\n")
                            );
                            self.state.video_convert_result = Some(result_message);
                            self.state.show_video_convert_result = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// 处理分片转换进度更新
    fn handle_chunk_progress_update(&mut self, update: crate::video_chunk_parallel_processor::ChunkProgressUpdate) {
        use crate::video_chunk_parallel_processor::ChunkProgressUpdate;
        
        match update {
            ChunkProgressUpdate::TaskStarted { task_id, input_path, chunk_count } => {
                info!("分片转换任务开始: {} ({}), 分片数: {}", task_id, input_path.display(), chunk_count);
                
                // 更新任务管理器进度
                if let Some(ref mut task) = self.state.task_manager.current_task {
                    task.current_file = task_id + 1;
                    task.current_filename = format!("{} ({}个分片)", input_path.display(), chunk_count);
                    task.total_files = task.total_files; // 保持原有总数
                }
            }
            ChunkProgressUpdate::ChunkStarted { task_id, chunk_index, chunk_path } => {
                info!("分片转换开始: 任务{} 分片{} - {}", task_id, chunk_index, chunk_path.display());
                
                if let Some(ref mut task) = self.state.task_manager.current_task {
                    task.current_filename = format!("分片 {}/{}: {}", 
                        chunk_index + 1, 
                        task.total_files, 
                        chunk_path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
            }
            ChunkProgressUpdate::ChunkCompleted { task_id, chunk_index, success, error } => {
                if success {
                    info!("分片转换完成: 任务{} 分片{}", task_id, chunk_index);
                } else {
                    warn!("分片转换失败: 任务{} 分片{} - {}", task_id, chunk_index, error.as_deref().unwrap_or("未知错误"));
                }
            }
            ChunkProgressUpdate::TaskCompleted { task_id, result } => {
                info!("分片转换任务完成: {} - 成功: {}", task_id, result.result.success);
                
                // 更新进度
                if let Some(ref mut task) = self.state.task_manager.current_task {
                    task.current_file = task.current_file + 1;
                    task.progress = task.current_file as f32 / task.total_files as f32;
                    
                    if result.result.success {
                        task.current_filename = result.result.get_success_message();
                    } else {
                        task.current_filename = result.result.get_error_message();
                    }
                }
            }
            ChunkProgressUpdate::AllTasksCompleted { success_count, error_count, total_duration, results } => {
                info!("所有分片转换任务完成: 成功={}, 失败={}, 耗时={:.2}秒", 
                    success_count, error_count, total_duration.as_secs_f64());
                
                // 完成任务
                self.state.task_manager.complete_task();
                
                // 根据任务类型处理结果
                if let Some(ref task) = self.state.task_manager.task_history.last() {
                    match task.task_type {
                        crate::models::TaskType::VideoConvert => {
                            let result_message = format!(
                                "分片并行视频转换完成！\n\n成功: {}\n失败: {}\n总耗时: {:.2}秒\n\n详细结果:\n{}",
                                success_count,
                                error_count,
                                total_duration.as_secs_f64(),
                                results.iter().map(|r| format!("• {}", r.result.get_success_message())).collect::<Vec<_>>().join("\n")
                            );
                            self.state.video_convert_result = Some(result_message);
                            self.state.show_video_convert_result = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
