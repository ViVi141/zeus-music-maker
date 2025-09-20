use eframe::egui;
use log::info;

use crate::models::AppState;
use crate::ui::UIComponents;
use crate::threading::ThreadedTaskProcessor;

/// 主应用程序
pub struct ZeusMusicApp {
    state: AppState,
    /// 多线程任务处理器
    task_processor: ThreadedTaskProcessor,
}

impl ZeusMusicApp {
    pub fn new() -> Self {
        info!("初始化Zeus Music Mod Generator");
        Self {
            state: AppState::default(),
            task_processor: ThreadedTaskProcessor::new(),
        }
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
        UIComponents::show_about_dialog(ctx, &mut self.state);
        UIComponents::show_track_editor_dialog(ctx, &mut self.state);
        UIComponents::show_paa_converter_dialog(ctx, &mut self.state, Some(&mut self.task_processor));
        UIComponents::show_preview_dialog(ctx, &mut self.state);
        UIComponents::show_export_result_dialog(ctx, &mut self.state);
        UIComponents::show_track_count_dialog(ctx, &mut self.state);
        UIComponents::show_paa_result_dialog(ctx, &mut self.state);
        UIComponents::show_audio_decrypt_dialog(ctx, &mut self.state);
        UIComponents::show_audio_decrypt_result_dialog(ctx, &mut self.state);
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
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("应用程序退出");
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

}
