use eframe::egui;
use log::{info, warn};

use crate::file_ops::FileOperations;
use crate::models::AppState;
use crate::templates::TemplateEngine;

/// UI组件
pub struct UIComponents;

impl UIComponents {
    /// 计算安全的窗口位置，确保不超出屏幕边界
    fn calculate_safe_position(
        ctx: &egui::Context,
        window_size: egui::Vec2,
        preferred_pos: egui::Pos2,
    ) -> egui::Pos2 {
        let screen_size = ctx.available_rect().size();
        let mut pos = preferred_pos;
        
        // 确保窗口不超出右边界
        if pos.x + window_size.x > screen_size.x {
            pos.x = (screen_size.x - window_size.x).max(0.0);
        }
        
        // 确保窗口不超出下边界
        if pos.y + window_size.y > screen_size.y {
            pos.y = (screen_size.y - window_size.y).max(0.0);
        }
        
        // 确保窗口不超出左边界和上边界
        pos.x = pos.x.max(0.0);
        pos.y = pos.y.max(0.0);
        
        pos
    }
    /// 渲染主菜单栏
    pub fn render_menu_bar(ui: &mut egui::Ui, state: &mut AppState) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("文件", |ui| {
                ui.menu_button("项目设置", |ui| {
                    if ui.button("常规").clicked() {
                        state.show_project_settings = true;
                        ui.close_menu();
                    }
                    if ui.button("添加封面图片 (.paa)").clicked() {
                        if let Some(path) = FileOperations::select_logo_file() {
                            state.project.logo_path = Some(path);
                            state.project.use_default_logo = false;
                            info!("选择Logo文件: {:?}", state.project.logo_path);
                        }
                        ui.close_menu();
                    }
                });
                ui.separator();
                if ui.button("导出...").clicked() {
                    state.show_export_dialog = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("退出").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("工具", |ui| {
                if ui.button("构建插件...").clicked() {
                    if let Some(pbo_path) = FileOperations::select_pbo_file() {
                        Self::build_addon(state, &pbo_path);
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("转换图片为PAA...").clicked() {
                    state.show_paa_converter = true;
                    ui.close_menu();
                }
                if ui.button("音频解密...").clicked() {
                    state.show_audio_decrypt = true;
                    ui.close_menu();
                }
                if ui.button("轨道计数").clicked() {
                    state.show_track_count = true;
                    ui.close_menu();
                }
                if ui.button("清空所有轨道").clicked() {
                    state.clear_tracks();
                    ui.close_menu();
                }
            });

            ui.menu_button("帮助", |ui| {
                if ui.button("关于").clicked() {
                    state.show_about = true;
                    ui.close_menu();
                }
            });
        });
    }

    /// 渲染轨道列表
    pub fn render_track_list(ui: &mut egui::Ui, state: &mut AppState) {
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 50.0)
            .show(ui, |ui| {
                let mut selected_track = state.selected_track;
                
                if state.tracks.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label("暂无轨道，点击'添加歌曲'按钮选择OGG音频文件");
                        ui.add_space(10.0);
                        ui.label("注意：仅支持OGG格式的音频文件");
                        ui.add_space(20.0);
                    });
                } else {
                    for (i, track) in state.tracks.iter().enumerate() {
                        let is_selected = selected_track == Some(i);
                        
                        let response = ui.selectable_label(
                            is_selected,
                            format!("{} ({})", track.display_name(), track.duration)
                        );

                        if response.clicked() {
                            selected_track = Some(i);
                        }

                        // 双击编辑轨道
                        if response.double_clicked() {
                            state.selected_track = Some(i);
                            state.show_track_editor = true;
                        }
                    }
                }
                
                state.selected_track = selected_track;
            });
    }

    /// 渲染底部按钮
    pub fn render_bottom_buttons(ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            if ui.button("添加OGG歌曲").clicked() {
                if let Some(paths) = FileOperations::select_audio_files() {
                    match FileOperations::load_audio_files(paths, &state.project.class_name) {
                        Ok(tracks) => {
                            let track_count = tracks.len();
                            info!("开始添加 {} 个轨道", track_count);
                            for track in tracks {
                                let track_name = track.display_name();
                                state.add_track(track);
                                info!("添加轨道: {}", track_name);
                            }
                            info!("添加了 {} 个轨道，当前总轨道数: {}", track_count, state.track_count());
                            // 强制重绘UI
                            ui.ctx().request_repaint();
                        }
                        Err(e) => {
                            warn!("加载音频文件失败: {}", e);
                        }
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("删除歌曲").clicked() {
                    state.remove_selected_track();
                }
            });
        });
    }

    /// 显示项目设置对话框
    pub fn show_project_settings_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_project_settings {
            return;
        }

        let mut should_close = false;
        let mut should_save = false;

        let window_size = egui::Vec2::new(500.0, 400.0);
        let safe_pos = Self::calculate_safe_position(ctx, window_size, egui::Pos2::new(100.0, 100.0));
        
        egui::Window::new("项目设置")
            .open(&mut state.show_project_settings)
            .resizable(true)
            .default_size(window_size)
            .min_size([400.0, 300.0])
            .max_size([800.0, 600.0])
            .default_pos(safe_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 基本信息区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("基本信息");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("模组名称:");
                                ui.text_edit_singleline(&mut state.project.mod_name);
                            });
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("作者:");
                                ui.text_edit_singleline(&mut state.project.author_name);
                            });
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Logo设置区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Logo设置");
                            ui.add_space(5.0);
                            
                            ui.checkbox(&mut state.project.use_default_logo, "使用默认Logo");
                            
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("Logo路径:");
                                if let Some(ref logo_path) = state.project.logo_path {
                                    ui.label(logo_path.to_string_lossy());
                                } else {
                                    ui.label("未设置");
                                }
                            });
                        });
                    });
                    
                    ui.add_space(15.0);
                    
                    // 按钮区域
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("取消").clicked() {
                                should_close = true;
                            }
                            if ui.button("确定").clicked() {
                                state.project.update_class_name();
                                should_save = true;
                                should_close = true;
                            }
                        });
                    });
                });
            });

        if should_close {
            state.show_project_settings = false;
        }
    }

    /// 显示导出对话框
    pub fn show_export_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_export_dialog {
            return;
        }

        let mut append_tags = state.export_settings.append_tags;
        let mut use_default_logo = state.export_settings.use_default_logo;
        let mut should_close = false;
        let mut should_export = false;
        let mut export_dir = None;

        let window_size = egui::Vec2::new(600.0, 500.0);
        let safe_pos = Self::calculate_safe_position(ctx, window_size, egui::Pos2::new(150.0, 150.0));
        
        egui::Window::new("导出设置")
            .open(&mut state.show_export_dialog)
            .resizable(true)
            .default_size(window_size)
            .min_size([500.0, 400.0])
            .max_size([900.0, 700.0])
            .default_pos(safe_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 导出信息区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("导出信息");
                            ui.add_space(5.0);
                            ui.label(format!(
                                "模组将在程序可执行文件同目录下创建名为 {} 的文件夹。",
                                state.project.mod_name_no_spaces()
                            ));
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // 导出选项区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("导出选项");
                            ui.add_space(5.0);
                            
                            ui.checkbox(&mut append_tags, "在轨道名称前添加标签");
                            
                            ui.add_space(8.0);
                            
                            ui.checkbox(&mut use_default_logo, "使用默认Logo");
                        });
                    });
                    
                    ui.add_space(15.0);
                    
                    // 按钮区域
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("取消").clicked() {
                                should_close = true;
                            }
                            if ui.button("导出").clicked() {
                                if let Some(dir) = FileOperations::select_export_directory() {
                                    export_dir = Some(dir);
                                    should_export = true;
                                    should_close = true;
                                }
                            }
                        });
                    });
                });
            });

        if should_close {
            state.export_settings.append_tags = append_tags;
            state.export_settings.use_default_logo = use_default_logo;
            state.show_export_dialog = false;
        }

        if should_export {
            if let Some(dir) = export_dir {
                Self::export_mod(state, &dir);
            }
        }
    }

    /// 显示关于对话框
    pub fn show_about_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_about {
            return;
        }

        let mut should_close = false;

        let window_size = egui::Vec2::new(400.0, 300.0);
        let safe_pos = Self::calculate_safe_position(ctx, window_size, egui::Pos2::new(200.0, 200.0));
        
        egui::Window::new("关于")
            .open(&mut state.show_about)
            .resizable(true)
            .default_size(window_size)
            .min_size([350.0, 250.0])
            .max_size([600.0, 500.0])
            .default_pos(safe_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 软件信息区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("宙斯音乐制作器");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("版本:");
                                ui.label("1.0.0");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("作者:");
                                ui.label("ViVi141");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("邮箱:");
                                ui.label("747384120@qq.com");
                            });
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // 描述区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("软件描述");
                            ui.add_space(5.0);
                            ui.label("增强版Arma 3音乐模组生成工具");
                            ui.label("用于自动生成Arma 3音乐模组文件结构");
                        });
                    });
                    
                    ui.add_space(15.0);
                    
                    // 按钮区域
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("关闭").clicked() {
                                should_close = true;
                            }
                        });
                    });
                });
            });

        if should_close {
            state.show_about = false;
        }
    }

    /// 显示轨道编辑器
    pub fn show_track_editor_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_track_editor {
            return;
        }

        let track_index = match state.selected_track {
            Some(index) if index < state.tracks.len() => index,
            _ => {
                state.show_track_editor = false;
                return;
            }
        };

        let track = &mut state.tracks[track_index];
        let mut should_close = false;
        
        let window_size = egui::Vec2::new(500.0, 600.0);
        let safe_pos = Self::calculate_safe_position(ctx, window_size, egui::Pos2::new(100.0, 100.0));
        
        egui::Window::new("轨道编辑器")
            .open(&mut state.show_track_editor)
            .resizable(true)
            .default_size(window_size)
            .min_size([450.0, 500.0])
            .max_size([800.0, 800.0])
            .default_pos(safe_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 基本信息区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("基本信息");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("轨道名称:");
                                ui.text_edit_singleline(&mut track.track_name);
                            });
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("标签:");
                                ui.text_edit_singleline(&mut track.tag);
                            });
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // 音频属性区域
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("音频属性");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("时长 (秒):");
                                ui.add(egui::Slider::new(&mut track.duration, 0..=3600));
                            });
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("分贝 (dB):");
                                ui.add(egui::Slider::new(&mut track.decibels, -10..=5));
                            });
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // 状态显示区域
                    if track.is_modified() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::RED, "⚠️ 已修改");
                            });
                        });
                        ui.add_space(10.0);
                    }
                    
                    // 按钮区域
                    ui.horizontal(|ui| {
                        if ui.button("恢复默认").clicked() {
                            track.reset_to_default();
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("取消").clicked() {
                                should_close = true;
                            }
                            if ui.button("确定").clicked() {
                                should_close = true;
                            }
                        });
                    });
                });
            });
            
        if should_close {
            state.show_track_editor = false;
        }
    }

    /// 导出模组
    fn export_mod(state: &mut AppState, export_dir: &std::path::Path) {
        if state.tracks.is_empty() {
            state.export_result = Some("导出失败：没有轨道可以导出".to_string());
            state.show_export_result = true;
            return;
        }

        let mut success_steps = Vec::new();
        let mut error_steps = Vec::new();

        match FileOperations::create_mod_structure(&state.project, export_dir) {
            Ok(mod_dir) => {
                success_steps.push("创建模组目录结构".to_string());
                
                // 复制轨道文件并获取重命名后的文件名
                match FileOperations::copy_track_files(&state.tracks, &mod_dir) {
                    Ok(files) => {
                        success_steps.push(format!("复制轨道文件 ({} 个)", files.len()));
                        
                        // 复制Logo文件
                        match FileOperations::copy_logo_file(&state.project, &mod_dir) {
                            Ok(_) => success_steps.push("复制Logo文件".to_string()),
                            Err(e) => error_steps.push(format!("复制Logo文件失败: {}", e)),
                        }

                        // 复制Steam Logo
                        match FileOperations::copy_steam_logo(&mod_dir) {
                            Ok(_) => success_steps.push("复制Steam Logo".to_string()),
                            Err(e) => error_steps.push(format!("复制Steam Logo失败: {}", e)),
                        }

                        // 生成配置文件
                        let template_engine = TemplateEngine::default();
                        match template_engine.generate_all_configs(
                            &state.project,
                            &state.tracks,
                            &files,
                            state.export_settings.append_tags,
                            &mod_dir,
                        ) {
                            Ok(_) => {
                                success_steps.push("生成配置文件".to_string());
                                
                                // 构建最终结果消息
                                let mut result_message = format!("模组导出成功！\n\n输出目录: {}\n\n", mod_dir.display());
                                
                                if !success_steps.is_empty() {
                                    result_message.push_str("成功步骤:\n");
                                    for step in &success_steps {
                                        result_message.push_str(&format!("  {}\n", step));
                                    }
                                }
                                
                                if !error_steps.is_empty() {
                                    result_message.push_str("\n警告信息:\n");
                                    for step in &error_steps {
                                        result_message.push_str(&format!("  {}\n", step));
                                    }
                                }
                                
                                result_message.push_str(&format!("\n统计信息:\n  轨道数量: {}\n  模组名称: {}", 
                                    state.tracks.len(), 
                                    state.project.mod_name
                                ));
                                
                                state.export_result = Some(result_message);
                                state.show_export_result = true;
                                info!("模组导出成功: {:?}", mod_dir);
                            },
                            Err(e) => {
                                error_steps.push(format!("生成配置文件失败: {}", e));
                                let mut result_message = format!("模组导出失败！\n\n输出目录: {}\n\n", mod_dir.display());
                                
                                if !success_steps.is_empty() {
                                    result_message.push_str("成功步骤:\n");
                                    for step in &success_steps {
                                        result_message.push_str(&format!("  {}\n", step));
                                    }
                                }
                                
                                if !error_steps.is_empty() {
                                    result_message.push_str("\n错误信息:\n");
                                    for step in &error_steps {
                                        result_message.push_str(&format!("  {}\n", step));
                                    }
                                }
                                
                                state.export_result = Some(result_message);
                                state.show_export_result = true;
                            }
                        }
                    },
                    Err(e) => {
                        error_steps.push(format!("复制轨道文件失败: {}", e));
                        let mut result_message = format!("模组导出失败！\n\n输出目录: {}\n\n", mod_dir.display());
                        
                        if !success_steps.is_empty() {
                            result_message.push_str("成功步骤:\n");
                            for step in &success_steps {
                                result_message.push_str(&format!("  {}\n", step));
                            }
                        }
                        
                        if !error_steps.is_empty() {
                            result_message.push_str("\n错误信息:\n");
                            for step in &error_steps {
                                result_message.push_str(&format!("  {}\n", step));
                            }
                        }
                        
                        state.export_result = Some(result_message);
                        state.show_export_result = true;
                    }
                }
            }
            Err(e) => {
                error_steps.push(format!("创建模组结构失败: {}", e));
                let result_message = format!("模组导出失败！\n\n输出目录: {}\n\n错误信息:\n  {}", 
                    export_dir.display(), 
                    error_steps.join("\n  ")
                );
                state.export_result = Some(result_message);
                state.show_export_result = true;
            }
        }
    }

    /// 构建插件
    fn build_addon(state: &mut AppState, pbo_path: &std::path::Path) {
        if let Some(export_dir) = FileOperations::select_export_directory() {
            match FileOperations::create_pbo_mod_structure(&state.project, pbo_path, &export_dir) {
                Ok(mod_dir) => {
                    // 生成mod.cpp
                    let template_engine = TemplateEngine::default();
                    let mod_path = mod_dir.join("mod.cpp");
                    if let Err(e) = template_engine.generate_mod_cpp(&state.project, &mod_path) {
                        warn!("生成mod.cpp失败: {}", e);
                        return;
                    }

                    info!("插件构建成功: {:?}", mod_dir);
                }
                Err(e) => {
                    warn!("构建插件失败: {}", e);
                }
            }
        }
    }

    /// 显示PAA转换对话框
    pub fn show_paa_converter_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_paa_converter {
            return;
        }

        let mut should_close = false;
        let mut should_convert = false;

        let window_size = egui::Vec2::new(800.0, 600.0);
        let safe_pos = Self::calculate_safe_position(ctx, window_size, egui::Pos2::new(50.0, 50.0));
        
        egui::Window::new("图片转PAA转换器")
            .open(&mut state.show_paa_converter)
            .resizable(true)
            .default_size(window_size)
            .min_size([700.0, 500.0])
            .max_size([1200.0, 900.0])
            .default_pos(safe_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("图片转PAA格式转换");
                    ui.separator();

                    // 文件选择区域
                    ui.group(|ui| {
                        // 动态调整高度
                        let available_height = ui.available_height();
                        let min_height = (available_height * 0.25).max(150.0).min(300.0);
                        ui.set_min_height(min_height);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("选择图片文件").clicked() {
                                    if let Some(paths) = rfd::FileDialog::new()
                                        .add_filter("图片文件", &["png", "jpg", "jpeg", "bmp", "tga", "tiff", "webp"])
                                        .set_title("选择要转换的图片文件")
                                        .pick_files()
                                    {
                                        state.paa_selected_files = paths;
                                        if state.paa_output_directory.is_none() && !state.paa_selected_files.is_empty() {
                                            state.paa_output_directory = state.paa_selected_files[0].parent().map(|p| p.to_path_buf());
                                        }
                                    }
                                }

                                if ui.button("选择输出目录").clicked() {
                                    if let Some(output_dir) = rfd::FileDialog::new()
                                        .set_title("选择PAA文件输出目录")
                                        .pick_folder()
                                    {
                                        state.paa_output_directory = Some(output_dir);
                                    }
                                }

                                if ui.button("清空列表").clicked() {
                                    state.paa_selected_files.clear();
                                }
                            });

                            ui.add_space(5.0);

                            if state.paa_selected_files.is_empty() {
                                ui.label("未选择任何文件");
                            } else {
                                ui.label(format!("已选择 {} 个文件:", state.paa_selected_files.len()));
                                ui.add_space(5.0);
                                
                                egui::ScrollArea::vertical()
                                    .max_height(100.0)
                                    .show(ui, |ui| {
                                        let mut indices_to_remove = Vec::new();
                                        
                                        for (i, file) in state.paa_selected_files.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("{}. {}", i + 1, file.file_name().unwrap_or_default().to_string_lossy()));
                                                if ui.small_button("移除").clicked() {
                                                    indices_to_remove.push(i);
                                                }
                                            });
                                        }
                                        
                                        // 从后往前移除，避免索引问题
                                        for &index in indices_to_remove.iter().rev() {
                                            state.paa_selected_files.remove(index);
                                        }
                                    });
                            }

                            if let Some(ref output_dir) = state.paa_output_directory {
                                ui.label(format!("输出目录: {}", output_dir.display()));
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // 转换选项区域
                    ui.group(|ui| {
                        // 动态调整高度
                        let available_height = ui.available_height();
                        let min_height = (available_height * 0.2).max(100.0).min(200.0);
                        ui.set_min_height(min_height);
                        ui.vertical(|ui| {
                            ui.heading("转换选项");
                            ui.separator();

                            ui.checkbox(&mut state.paa_options.crop_to_power_of_two, "裁剪到2的次方尺寸 (推荐)");
                            
                            if state.paa_options.crop_to_power_of_two {
                                ui.horizontal(|ui| {
                                    ui.label("目标尺寸:");
                                    ui.radio_value(&mut state.paa_options.target_size, None, "自动选择");
                                    ui.radio_value(&mut state.paa_options.target_size, Some(256), "256x256");
                                    ui.radio_value(&mut state.paa_options.target_size, Some(512), "512x512");
                                    ui.radio_value(&mut state.paa_options.target_size, Some(1024), "1024x1024");
                                });

                                ui.horizontal(|ui| {
                                    ui.label("裁剪方式:");
                                    ui.radio_value(&mut state.paa_options.center_crop, true, "居中裁剪 (推荐)");
                                    ui.radio_value(&mut state.paa_options.center_crop, false, "保持原始比例");
                                });
                            }

                            ui.add_space(5.0);
                            ui.label("支持的图片格式: PNG, JPG, JPEG, BMP, TGA, TIFF, WEBP");
                        });
                    });

                    ui.add_space(10.0);

                    // 操作按钮区域
                    ui.horizontal(|ui| {
                        let can_convert = !state.paa_selected_files.is_empty() && state.paa_output_directory.is_some();
                        
                        if ui.add_enabled(can_convert, egui::Button::new("开始转换")).clicked() {
                            should_convert = true;
                        }

                        if ui.button("预览效果").clicked() {
                            state.show_paa_preview = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("关闭").clicked() {
                                should_close = true;
                            }
                        });
                    });
                });
            });

        if should_close {
            state.show_paa_converter = false;
        }
        
        // 在闭包外面执行转换，避免借用冲突
        if should_convert {
            if let Some(ref output_dir) = state.paa_output_directory {
                Self::convert_images_to_paa_simple(state.paa_selected_files.clone(), output_dir.clone(), state.paa_options.clone(), state);
            }
        }
    }



    /// 显示预览对话框
    pub fn show_preview_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_paa_preview {
            return;
        }

        let mut should_close = false;

        let window_size = egui::Vec2::new(900.0, 700.0);
        let safe_pos = Self::calculate_safe_position(ctx, window_size, egui::Pos2::new(100.0, 100.0));
        
        egui::Window::new("转换预览")
            .open(&mut state.show_paa_preview)
            .resizable(true)
            .default_size(window_size)
            .min_size([800.0, 600.0])
            .max_size([1400.0, 1000.0])
            .default_pos(safe_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("转换设置预览");
                    ui.separator();

                    // 显示设置信息
                    ui.label(format!("裁剪到2的次方尺寸: {}", if state.paa_options.crop_to_power_of_two { "是" } else { "否" }));
                    
                    if state.paa_options.crop_to_power_of_two {
                        match state.paa_options.target_size {
                            Some(size) => {
                                ui.label(format!("目标尺寸: {}x{}", size, size));
                            },
                            None => {
                                ui.label("目标尺寸: 自动选择");
                            },
                        }
                        ui.label(format!("裁剪方式: {}", if state.paa_options.center_crop { "居中裁剪" } else { "保持原始比例" }));
                    }

                    ui.add_space(10.0);

                    // 显示图片预览
                    if !state.paa_selected_files.is_empty() {
                        if let Some(ref rtm) = state.runtime_texture_manager {
                            if let Some(ref texture) = rtm.current_texture {
                                ui.group(|ui| {
                                    ui.heading("图片预览");
                                    
                                    // 显示原始图片
                                    ui.label("原始图片:");
                                    let image_size = rtm.base.display_size;
                                    ui.add(egui::Image::new((texture.id(), egui::Vec2::new(image_size.0, image_size.1))));
                                    
                                    ui.add_space(10.0);
                                    
                                    // 显示裁剪信息
                                    ui.label("裁剪方式:");
                                    if state.paa_options.center_crop {
                                        ui.label("居中裁剪");
                                    } else {
                                        ui.label("保持原始比例");
                                    }
                                });
                            }
                        }
                    }

                    ui.add_space(10.0);
                    ui.label("建议:");
                    ui.label("• 256x256: 适合小图标和按钮");
                    ui.label("• 512x512: 适合中等尺寸的Logo");
                    ui.label("• 1024x1024: 适合大型背景图");
                    ui.label("• 自动选择: 根据原图尺寸智能选择");

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            state.show_paa_preview = false;
        }
    }



    /// 显示导出结果对话框
    pub fn show_export_result_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_export_result {
            return;
        }

        let safe_pos = Self::calculate_safe_position(ctx, [600.0, 400.0].into(), [100.0, 100.0].into());
        let mut should_close = false;
        
        egui::Window::new("导出结果")
            .open(&mut state.show_export_result)
            .default_pos(safe_pos)
            .resizable(true)
            .default_size([600.0, 400.0])
            .min_size([400.0, 200.0])
            .max_size([800.0, 600.0])
            .show(ctx, |ui| {
                ui.set_min_height(ui.available_height());
                
                if let Some(ref result) = state.export_result {
                    ui.group(|ui| {
                        ui.heading("导出结果");
                        ui.add_space(5.0);
                        
                        // 使用ScrollArea来显示可能很长的结果文本
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 50.0)
                            .show(ui, |ui| {
                                // 按行分割结果文本并显示
                                for line in result.lines() {
                                    if line.contains("导出成功！") || line.contains("导出失败！") {
                                        ui.heading(line);
                                    } else if line.starts_with("  成功步骤:") || line.starts_with("  警告信息:") {
                                        ui.colored_label(egui::Color32::from_rgb(0, 150, 0), line);
                                    } else if line.starts_with("  错误信息:") {
                                        ui.colored_label(egui::Color32::from_rgb(200, 50, 50), line);
                                    } else if line.starts_with("输出目录:") || line.starts_with("统计信息:") {
                                        ui.colored_label(egui::Color32::from_rgb(100, 100, 255), line);
                                    } else if line.trim().is_empty() {
                                        ui.add_space(5.0);
                                    } else {
                                        ui.label(line);
                                    }
                                }
                            });
                    });
                }
                
                ui.add_space(10.0);
                
                // 按钮区域
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("确定").clicked() {
                        should_close = true;
                    }
                    
                    if ui.button("复制结果").clicked() {
                        if let Some(ref result) = state.export_result {
                            ui.output_mut(|o| o.copied_text = result.clone());
                        }
                    }
                });
            });
        
        if should_close {
            state.show_export_result = false;
            state.export_result = None;
        }
    }

    /// 显示音频解密对话框
    pub fn show_audio_decrypt_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_audio_decrypt {
            return;
        }

        let safe_pos = Self::calculate_safe_position(ctx, [600.0, 500.0].into(), [100.0, 100.0].into());
        let mut should_close = false;
        
        egui::Window::new("音频解密")
            .open(&mut state.show_audio_decrypt)
            .default_pos(safe_pos)
            .resizable(true)
            .default_size([600.0, 500.0])
            .min_size([500.0, 300.0])
            .max_size([800.0, 700.0])
            .show(ctx, |ui| {
                ui.set_min_height(ui.available_height());
                
                ui.group(|ui| {
                    ui.heading("文件选择");
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("选择加密音频文件").clicked() {
                            if let Some(files) = FileOperations::select_encrypted_audio_files() {
                                state.audio_decrypt_selected_files = files;
                            }
                        }
                        
                        if ui.button("清空列表").clicked() {
                            state.audio_decrypt_selected_files.clear();
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    if !state.audio_decrypt_selected_files.is_empty() {
                        ui.label(format!("已选择 {} 个文件:", state.audio_decrypt_selected_files.len()));
                        
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                let mut indices_to_remove = Vec::new();
                                for (i, file) in state.audio_decrypt_selected_files.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("• {}", file.file_name().unwrap_or_default().to_string_lossy()));
                                        if ui.small_button("移除").clicked() {
                                            indices_to_remove.push(i);
                                        }
                                    });
                                }
                                
                                // 从后往前删除，避免索引问题
                                for &i in indices_to_remove.iter().rev() {
                                    state.audio_decrypt_selected_files.remove(i);
                                }
                            });
                    } else {
                        ui.label("未选择任何文件");
                    }
                });
                
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.heading("输出设置");
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("输出目录:");
                        if let Some(ref output_dir) = state.audio_decrypt_output_directory {
                            ui.label(output_dir.display().to_string());
                        } else {
                            ui.label("未选择");
                        }
                        
                        if ui.button("选择输出目录").clicked() {
                            if let Some(dir) = FileOperations::select_export_directory() {
                                state.audio_decrypt_output_directory = Some(dir);
                            }
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.heading("支持格式");
                    ui.add_space(5.0);
                    ui.label("• 酷狗音乐 (.kgm) - 自动检测输出格式");
                    ui.label("• 网易云音乐 (.ncm) - 支持MP3/FLAC输出");
                    ui.label("• 其他加密格式 - 开发中");
                });
                
                ui.add_space(15.0);
                
                // 按钮区域
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("关闭").clicked() {
                        should_close = true;
                    }
                    
                    let can_decrypt = !state.audio_decrypt_selected_files.is_empty() 
                        && state.audio_decrypt_output_directory.is_some();
                    
                    if ui.add_enabled(can_decrypt, egui::Button::new("开始解密")).clicked() {
                        state.should_decrypt_audio = true;
                    }
                });
            });
        
        if should_close {
            state.show_audio_decrypt = false;
        }
    }

    /// 解密音频文件并存储结果到AppState
    pub fn decrypt_audio_files_with_state(state: &mut AppState, selected_files: Vec<std::path::PathBuf>, output_dir: std::path::PathBuf) {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut success_files = Vec::new();
        let mut error_files = Vec::new();
        
        for input_path in &selected_files {
            // 检查文件类型并解密
            if crate::audio_decrypt::AudioDecryptManager::is_kugou_file(input_path) {
                match crate::audio_decrypt::AudioDecryptManager::decrypt_kugou_file(input_path, &output_dir) {
                    Ok(output_path) => {
                        success_count += 1;
                        success_files.push(format!("酷狗: {} -> {}", 
                            input_path.file_name().unwrap_or_default().to_string_lossy(),
                            std::path::Path::new(&output_path).file_name().unwrap_or_default().to_string_lossy()
                        ));
                        log::info!("成功解密酷狗文件: {} -> {}", input_path.display(), output_path);
                    }
                    Err(e) => {
                        error_count += 1;
                        error_files.push(format!("酷狗: {} - {}", 
                            input_path.file_name().unwrap_or_default().to_string_lossy(),
                            e
                        ));
                        log::error!("解密酷狗文件失败: {} - {}", input_path.display(), e);
                    }
                }
            } else if crate::audio_decrypt::AudioDecryptManager::is_netease_file(input_path) {
                match crate::audio_decrypt::AudioDecryptManager::decrypt_netease_file(input_path, &output_dir) {
                    Ok(output_path) => {
                        success_count += 1;
                        success_files.push(format!("网易云: {} -> {}", 
                            input_path.file_name().unwrap_or_default().to_string_lossy(),
                            std::path::Path::new(&output_path).file_name().unwrap_or_default().to_string_lossy()
                        ));
                        log::info!("成功解密网易云文件: {} -> {}", input_path.display(), output_path);
                    }
                    Err(e) => {
                        error_count += 1;
                        error_files.push(format!("网易云: {} - {}", 
                            input_path.file_name().unwrap_or_default().to_string_lossy(),
                            e
                        ));
                        log::error!("解密网易云文件失败: {} - {}", input_path.display(), e);
                    }
                }
            } else {
                error_count += 1;
                error_files.push(format!("不支持: {} - 不支持的音频格式", 
                    input_path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
        
        // 构建解密结果消息
        let mut result_message = if error_count == 0 {
            format!("音频解密完成！\n\n输出目录: {}\n\n", output_dir.display())
        } else if success_count > 0 {
            format!("音频解密完成（部分失败）\n\n输出目录: {}\n\n", output_dir.display())
        } else {
            format!("音频解密失败！\n\n输出目录: {}\n\n", output_dir.display())
        };
        
        result_message.push_str(&format!("统计信息:\n  总文件数: {}\n  成功: {}\n  失败: {}\n", 
            selected_files.len(), success_count, error_count));
        
        if !success_files.is_empty() {
            result_message.push_str("\n成功解密的文件:\n");
            for file in &success_files {
                result_message.push_str(&format!("  {}\n", file));
            }
        }
        
        if !error_files.is_empty() {
            result_message.push_str("\n解密失败的文件:\n");
            for file in &error_files {
                result_message.push_str(&format!("  {}\n", file));
            }
        }
        
        // 存储结果到AppState
        state.audio_decrypt_result = Some(result_message);
        state.show_audio_decrypt_result = true;
        
        if success_count > 0 {
            log::info!("音频解密完成: 成功 {} 个，失败 {} 个", success_count, error_count);
        } else {
            log::warn!("所有音频文件解密失败");
        }
    }


    /// 转换图片为PAA格式（简单版本）
    fn convert_images_to_paa_simple(
        paths: Vec<std::path::PathBuf>, 
        output_dir: std::path::PathBuf, 
        options: crate::paa_converter::PaaOptions,
        state: &mut AppState
    ) {
        if paths.is_empty() {
            return;
        }

        info!("开始转换 {} 个图片文件", paths.len());
        
        let mut success_count = 0;
        let mut error_count = 0;
        let mut converted_files = Vec::new();
        
        for input_path in &paths {
            if let Some(file_name) = input_path.file_stem() {
                let output_path = output_dir.join(format!("{}.paa", file_name.to_string_lossy()));
                
                match crate::paa_converter::PaaConverter::convert_image_to_paa_with_crop(
                    input_path, 
                    &output_path, 
                    options.clone(),
                    None
                ) {
                    Ok(_) => {
                        success_count += 1;
                        converted_files.push(output_path.clone());
                        info!("转换成功: {:?}", output_path);
                    },
                    Err(e) => {
                        error_count += 1;
                        warn!("转换失败: {:?} - {}", input_path, e);
                    }
                }
            }
        }
        
        // 构建转换结果消息
        let mut result_message = if error_count == 0 {
            format!("PAA转换完成！\n\n输出目录: {}\n\n", output_dir.display())
        } else if success_count > 0 {
            format!("PAA转换完成（部分失败）\n\n输出目录: {}\n\n", output_dir.display())
        } else {
            format!("PAA转换失败！\n\n输出目录: {}\n\n", output_dir.display())
        };
        
        // 添加转换设置信息
        result_message.push_str("转换设置:\n");
        result_message.push_str(&format!("  裁剪方式: {}\n", if options.center_crop { "居中裁剪" } else { "保持原始比例" }));
        if let Some(size) = options.target_size {
            result_message.push_str(&format!("  目标尺寸: {}x{}\n", size, size));
        } else {
            result_message.push_str("  目标尺寸: 自动选择\n");
        }
        result_message.push_str(&format!("  裁剪到2的次方: {}\n", if options.crop_to_power_of_two { "是" } else { "否" }));
        
        result_message.push_str(&format!("\n统计信息:\n  总文件数: {}\n  成功: {}\n  失败: {}\n", 
            paths.len(), success_count, error_count));
        
        // 设置转换结果并显示对话框
        state.paa_result = Some(result_message);
        state.show_paa_result = true;
        
        if success_count > 0 {
            info!("转换完成: 成功 {} 个，失败 {} 个", success_count, error_count);
        } else {
            warn!("所有文件转换失败");
        }
    }

    /// 显示轨道计数对话框
    pub fn show_track_count_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_track_count {
            return;
        }

        // 在闭包外面计算轨道信息，避免借用冲突
        let track_count = state.track_count();
        let total_duration = if track_count > 0 {
            state.tracks.iter().map(|t| t.duration as f32).sum::<f32>()
        } else {
            0.0
        };

        let safe_pos = Self::calculate_safe_position(ctx, [300.0, 150.0].into(), [100.0, 100.0].into());
        let mut should_close = false;
        
        egui::Window::new("轨道计数")
            .open(&mut state.show_track_count)
            .default_pos(safe_pos)
            .resizable(false)
            .default_size([300.0, 150.0])
            .min_size([250.0, 120.0])
            .max_size([400.0, 200.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    
                    ui.heading("轨道统计");
                    ui.add_space(10.0);
                    
                    ui.label(format!("当前列表中有 {} 个轨道", track_count));
                    
                    if track_count > 0 {
                        ui.add_space(5.0);
                        ui.label(format!("总时长: {:.1} 秒", total_duration));
                    }
                    
                    ui.add_space(20.0);
                    
                    if ui.button("确定").clicked() {
                        should_close = true;
                    }
                });
            });
        
        if should_close {
            state.show_track_count = false;
        }
    }

    /// 显示PAA转换结果对话框
    pub fn show_paa_result_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_paa_result {
            return;
        }

        let safe_pos = Self::calculate_safe_position(ctx, [600.0, 400.0].into(), [100.0, 100.0].into());
        let mut should_close = false;
        
        egui::Window::new("PAA转换结果")
            .open(&mut state.show_paa_result)
            .default_pos(safe_pos)
            .resizable(true)
            .default_size([600.0, 400.0])
            .min_size([400.0, 200.0])
            .max_size([800.0, 600.0])
            .show(ctx, |ui| {
                ui.set_min_height(ui.available_height());
                
                if let Some(ref result) = state.paa_result {
                    ui.group(|ui| {
                        ui.heading("转换结果");
                        ui.add_space(5.0);
                        
                        // 使用ScrollArea来显示可能很长的结果文本
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 50.0)
                            .show(ui, |ui| {
                                // 按行分割结果文本并显示
                                for line in result.lines() {
                                    if line.contains("转换完成！") || line.contains("转换失败！") {
                                        ui.heading(line);
                                    } else if line.starts_with("  成功:") || line.starts_with("  失败:") {
                                        ui.colored_label(egui::Color32::from_rgb(0, 150, 0), line);
                                    } else if line.starts_with("输出目录:") || line.starts_with("统计信息:") || line.starts_with("转换设置:") {
                                        ui.colored_label(egui::Color32::from_rgb(100, 100, 255), line);
                                    } else if line.trim().is_empty() {
                                        ui.add_space(5.0);
                                    } else {
                                        ui.label(line);
                                    }
                                }
                            });
                    });
                }
                
                ui.add_space(10.0);
                
                // 按钮区域
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("确定").clicked() {
                        should_close = true;
                    }
                    
                    if ui.button("复制结果").clicked() {
                        if let Some(ref result) = state.paa_result {
                            ui.output_mut(|o| o.copied_text = result.clone());
                        }
                    }
                });
            });
        
        if should_close {
            state.show_paa_result = false;
            state.paa_result = None;
        }
    }

    /// 显示音频解密结果对话框
    pub fn show_audio_decrypt_result_dialog(ctx: &egui::Context, state: &mut AppState) {
        if !state.show_audio_decrypt_result {
            return;
        }

        let safe_pos = Self::calculate_safe_position(ctx, [600.0, 400.0].into(), [100.0, 100.0].into());
        let mut should_close = false;
        
        egui::Window::new("音频解密结果")
            .open(&mut state.show_audio_decrypt_result)
            .default_pos(safe_pos)
            .resizable(true)
            .default_size([600.0, 400.0])
            .min_size([400.0, 200.0])
            .max_size([800.0, 600.0])
            .show(ctx, |ui| {
                ui.set_min_height(ui.available_height());
                
                if let Some(ref result) = state.audio_decrypt_result {
                    ui.group(|ui| {
                        ui.heading("解密结果");
                        ui.add_space(5.0);
                        
                        // 使用ScrollArea来显示可能很长的结果文本
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 50.0)
                            .show(ui, |ui| {
                                // 按行分割结果文本并显示
                                for line in result.lines() {
                                    if line.contains("解密完成！") || line.contains("解密失败！") {
                                        ui.heading(line);
                                    } else if line.starts_with("  成功:") || line.starts_with("  失败:") {
                                        ui.colored_label(egui::Color32::from_rgb(0, 150, 0), line);
                                    } else if line.starts_with("输出目录:") || line.starts_with("统计信息:") {
                                        ui.colored_label(egui::Color32::from_rgb(100, 100, 255), line);
                                    } else if line.trim().is_empty() {
                                        ui.add_space(5.0);
                                    } else {
                                        ui.label(line);
                                    }
                                }
                            });
                    });
                }
                
                ui.add_space(10.0);
                
                // 按钮区域
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("确定").clicked() {
                        should_close = true;
                    }
                    
                    if ui.button("复制结果").clicked() {
                        if let Some(ref result) = state.audio_decrypt_result {
                            ui.output_mut(|o| o.copied_text = result.clone());
                        }
                    }
                });
            });
        
        if should_close {
            state.show_audio_decrypt_result = false;
            state.audio_decrypt_result = None;
        }
    }
}
             