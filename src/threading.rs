use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use log::{info, warn};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::audio_decrypt::AudioDecryptManager;
use crate::paa_converter::{PaaConverter, PaaOptions};

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
}

/// 多线程任务处理器
pub struct ThreadedTaskProcessor {
    /// 进度更新发送器
    progress_sender: Sender<TaskMessage>,
    /// 进度更新接收器
    progress_receiver: Receiver<TaskMessage>,
    /// 取消标志
    cancel_flag: Arc<Mutex<bool>>,
}

impl ThreadedTaskProcessor {
    pub fn new() -> Self {
        let (progress_sender, progress_receiver) = bounded(1000);
        Self {
            progress_sender,
            progress_receiver,
            cancel_flag: Arc::new(Mutex::new(false)),
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
                if *cancel_flag.lock().unwrap() {
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
                let result = if AudioDecryptManager::is_kugou_file(input_path) {
                    match AudioDecryptManager::decrypt_kugou_file(input_path, &output_dir) {
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
                if *cancel_flag.lock().unwrap() {
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


    /// 获取进度接收器
    pub fn get_progress_receiver(&self) -> &Receiver<TaskMessage> {
        &self.progress_receiver
    }

    /// 取消当前任务
    pub fn cancel_task(&self) {
        *self.cancel_flag.lock().unwrap() = true;
        info!("任务取消信号已发送");
    }

    /// 重置取消标志
    pub fn reset_cancel_flag(&self) {
        *self.cancel_flag.lock().unwrap() = false;
    }
}

impl Default for ThreadedTaskProcessor {
    fn default() -> Self {
        Self::new()
    }
}
