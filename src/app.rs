use eframe::egui;
use log::info;

use crate::models::AppState;
use crate::ui::UIComponents;

/// 主应用程序
pub struct ZeusMusicApp {
    state: AppState,
}

impl ZeusMusicApp {
    pub fn new() -> Self {
        info!("初始化Zeus Music Mod Generator");
        Self {
            state: AppState::default(),
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

        // 显示对话框
        UIComponents::show_project_settings_dialog(ctx, &mut self.state);
        UIComponents::show_export_dialog(ctx, &mut self.state);
        UIComponents::show_about_dialog(ctx, &mut self.state);
        UIComponents::show_track_editor_dialog(ctx, &mut self.state);
        UIComponents::show_paa_converter_dialog(ctx, &mut self.state);
        UIComponents::show_preview_dialog(ctx, &mut self.state);
        UIComponents::show_export_result_dialog(ctx, &mut self.state);
        UIComponents::show_track_count_dialog(ctx, &mut self.state);
        UIComponents::show_paa_result_dialog(ctx, &mut self.state);
        UIComponents::show_audio_decrypt_dialog(ctx, &mut self.state);
        UIComponents::show_audio_decrypt_result_dialog(ctx, &mut self.state);
        
        // 检查是否需要执行音频解密
        if self.state.should_decrypt_audio {
            if let Some(ref output_dir) = self.state.audio_decrypt_output_directory {
                let output_dir = output_dir.clone();
                let selected_files = self.state.audio_decrypt_selected_files.clone();
                UIComponents::decrypt_audio_files_with_state(&mut self.state, selected_files, output_dir);
            }
            self.state.should_decrypt_audio = false;
            self.state.show_audio_decrypt = false;
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("应用程序退出");
    }
}
