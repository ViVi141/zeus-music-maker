/*!
 * 并行FFmpeg转换模块
 * 支持音频和视频的多线程并行转换
 */

use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use log::{info, warn, debug};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::fmt;
use crate::audio_converter::AudioConverter;
use crate::video_converter::VideoConverter;
use crate::resource_manager::{GlobalResourceManager, SmartThreadPool};

/// 音频转换器trait
pub trait AudioConverterTrait {
    fn convert_to_ogg_with_cancel<F>(&self, input_path: &std::path::Path, output_path: &std::path::Path, should_cancel: &F) -> Result<String, anyhow::Error>
    where
        F: Fn() -> bool + ?Sized;
}

/// 视频转换器trait
pub trait VideoConverterTrait {
    fn convert_to_ogv(&self, input_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), anyhow::Error>;
}

// 为AudioConverter实现trait
impl AudioConverterTrait for AudioConverter {
    fn convert_to_ogg_with_cancel<F>(&self, input_path: &std::path::Path, output_path: &std::path::Path, should_cancel: &F) -> Result<String, anyhow::Error>
    where
        F: Fn() -> bool + ?Sized,
    {
        self.convert_to_ogg_with_cancel(input_path, output_path, should_cancel)
    }
}

impl VideoConverterTrait for AudioConverter {
    fn convert_to_ogv(&self, _input_path: &std::path::Path, _output_path: &std::path::Path) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("AudioConverter不支持视频转换"))
    }
}

// 为VideoConverter实现trait
impl AudioConverterTrait for VideoConverter {
    fn convert_to_ogg_with_cancel<F>(&self, _input_path: &std::path::Path, _output_path: &std::path::Path, _should_cancel: &F) -> Result<String, anyhow::Error>
    where
        F: Fn() -> bool + ?Sized,
    {
        Err(anyhow::anyhow!("VideoConverter不支持音频转换"))
    }
}

impl VideoConverterTrait for VideoConverter {
    fn convert_to_ogv(&self, input_path: &std::path::Path, output_path: &std::path::Path) -> Result<(), anyhow::Error> {
        self.convert_to_ogv(input_path, output_path)
    }
}

/// 并行转换配置
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// 最大并发线程数
    pub max_threads: usize,
    /// 任务队列大小
    pub queue_size: usize,
    /// 是否启用智能线程调度
    pub smart_scheduling: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: Self::get_optimal_thread_count(),
            queue_size: 1000,
            smart_scheduling: true,
        }
    }
}

impl ParallelConfig {
    /// 获取最优线程数
    fn get_optimal_thread_count() -> usize {
        let cpu_count = num_cpus::get();
        // 对于I/O密集型任务，使用CPU核心数的2倍
        // 但不超过8个线程，避免过度竞争
        (cpu_count * 2).min(8).max(2)
    }
    
    /// 根据文件大小动态调整线程数
    pub fn adjust_for_file_size(&mut self, total_files: usize, avg_file_size_mb: f64) {
        if self.smart_scheduling {
            if avg_file_size_mb > 100.0 {
                // 大文件，减少并发数避免内存压力
                self.max_threads = (self.max_threads / 2).max(2);
            } else if avg_file_size_mb < 10.0 && total_files > 50 {
                // 小文件大批量，可以增加并发数
                self.max_threads = (self.max_threads * 3 / 2).min(12);
            }
        }
        
        info!("调整后并发线程数: {}", self.max_threads);
    }
}

/// 转换任务类型
#[derive(Debug, Clone)]
pub enum ConversionTask {
    Audio {
        input_path: PathBuf,
        output_path: PathBuf,
        task_id: usize,
    },
    Video {
        input_path: PathBuf,
        output_path: PathBuf,
        task_id: usize,
    },
}

/// 转换结果
#[derive(Debug, Clone)]
pub enum ConversionResult {
    Success {
        #[allow(dead_code)]
        task_id: usize,
        input_path: PathBuf,
        #[allow(dead_code)]
        output_path: PathBuf,
        #[allow(dead_code)]
        duration: Duration,
        message: String,
    },
    Error {
        #[allow(dead_code)]
        task_id: usize,
        input_path: PathBuf,
        error: String,
    },
}

/// 进度更新消息
#[derive(Debug, Clone)]
pub enum ProgressUpdate {
    TaskStarted {
        task_id: usize,
        filename: String,
        total_tasks: usize,
    },
    TaskCompleted {
        task_id: usize,
        result: ConversionResult,
        completed_count: usize,
        total_tasks: usize,
    },
    AllTasksCompleted {
        success_count: usize,
        error_count: usize,
        total_duration: Duration,
        results: Vec<ConversionResult>,
    },
}

/// 并行转换器
pub struct ParallelConverter {
    /// 配置
    config: ParallelConfig,
    /// 进度更新发送器
    progress_sender: Sender<ProgressUpdate>,
    /// 进度更新接收器
    progress_receiver: Receiver<ProgressUpdate>,
    /// 取消标志
    cancel_flag: Arc<Mutex<bool>>,
    /// 统计信息
    stats: Arc<Mutex<ConversionStats>>,
    /// 资源管理器
    resource_manager: Arc<GlobalResourceManager>,
}

/// 转换统计信息
#[derive(Debug, Default)]
struct ConversionStats {
    total_tasks: usize,
    completed_tasks: usize,
    successful_tasks: usize,
    failed_tasks: usize,
    start_time: Option<Instant>,
    #[allow(dead_code)]
    total_duration: Duration,
}

impl ParallelConverter {
    /// 创建新的并行转换器
    pub fn new(config: ParallelConfig) -> Self {
        let (progress_sender, progress_receiver) = bounded(config.queue_size);
        
        Self {
            config,
            progress_sender,
            progress_receiver,
            cancel_flag: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(ConversionStats::default())),
            resource_manager: Arc::new(GlobalResourceManager::new()),
        }
    }
    
    /// 并行转换音频文件
    pub fn convert_audio_files_parallel(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        info!("开始并行音频转换，文件数: {}, 线程数: {}", files.len(), self.config.max_threads);
        
        // 重置统计信息
        self.reset_stats();
        
        // 创建音频转换器
        let converter = AudioConverter::new()
            .context("无法创建音频转换器，请确保FFmpeg已安装")?;
        
        // 准备转换任务
        let tasks = self.prepare_audio_tasks(files, output_dir)?;
        
        // 启动并行转换
        self.start_parallel_conversion(tasks, converter)
    }
    
    /// 并行转换视频文件
    pub fn convert_video_files_parallel(
        &self,
        files: Vec<PathBuf>,
        output_dir: PathBuf,
    ) -> Result<()> {
        info!("开始并行视频转换，文件数: {}, 线程数: {}", files.len(), self.config.max_threads);
        
        // 重置统计信息
        self.reset_stats();
        
        // 创建视频转换器
        let converter = VideoConverter::new()
            .context("无法创建视频转换器，请确保FFmpeg已安装")?;
        
        // 准备转换任务
        let tasks = self.prepare_video_tasks(files, output_dir)?;
        
        // 启动并行转换
        self.start_parallel_conversion(tasks, converter)
    }
    
    /// 准备音频转换任务
    fn prepare_audio_tasks(&self, files: Vec<PathBuf>, output_dir: PathBuf) -> Result<Vec<ConversionTask>> {
        let mut tasks = Vec::new();
        
        for (i, input_path) in files.iter().enumerate() {
            // 生成输出文件名
            let output_filename = if let Some(file_stem) = input_path.file_stem() {
                let pinyin_filename = crate::utils::string_utils::StringUtils::safe_filename_pinyin(
                    &file_stem.to_string_lossy(), 
                    i
                );
                format!("{}.ogg", pinyin_filename)
            } else {
                format!("audio{:03}.ogg", i)
            };
            
            let output_path = output_dir.join(output_filename);
            
            tasks.push(ConversionTask::Audio {
                input_path: input_path.clone(),
                output_path,
                task_id: i,
            });
        }
        
        // 更新统计信息
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_tasks = tasks.len();
            stats.start_time = Some(Instant::now());
        }
        
        Ok(tasks)
    }
    
    /// 准备视频转换任务
    fn prepare_video_tasks(&self, files: Vec<PathBuf>, output_dir: PathBuf) -> Result<Vec<ConversionTask>> {
        let mut tasks = Vec::new();
        
        for (i, input_path) in files.iter().enumerate() {
            // 生成输出文件名
            let output_filename = if let Some(file_stem) = input_path.file_stem() {
                let pinyin_filename = crate::utils::string_utils::StringUtils::safe_filename_pinyin(
                    &file_stem.to_string_lossy(), 
                    i
                );
                format!("{}.ogv", pinyin_filename)
            } else {
                format!("video{:03}.ogv", i)
            };
            
            let output_path = output_dir.join(output_filename);
            
            tasks.push(ConversionTask::Video {
                input_path: input_path.clone(),
                output_path,
                task_id: i,
            });
        }
        
        // 更新统计信息
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_tasks = tasks.len();
            stats.start_time = Some(Instant::now());
        }
        
        Ok(tasks)
    }
    
    /// 启动并行转换
    fn start_parallel_conversion<C>(&self, tasks: Vec<ConversionTask>, converter: C) -> Result<()>
    where
        C: Send + Sync + Clone + AudioConverterTrait + VideoConverterTrait + 'static,
    {
        let progress_sender = self.progress_sender.clone();
        let cancel_flag = self.cancel_flag.clone();
        let stats = self.stats.clone();
        
        // 创建任务队列
        let (task_sender, task_receiver) = bounded(tasks.len());
        
        // 发送所有任务到队列
        for task in tasks {
            if let Err(e) = task_sender.send(task) {
                warn!("发送任务到队列失败: {}", e);
            }
        }
        drop(task_sender); // 关闭发送端，表示不再有新任务
        
        // 获取智能线程池
        let thread_pool = self.resource_manager.get_thread_pool();
        
        // 动态调整线程数
        thread_pool.adjust_thread_count();
        let actual_thread_count = thread_pool.get_max_threads().min(self.config.max_threads);
        
        info!("使用智能线程池，实际线程数: {}", actual_thread_count);
        
        // 启动工作线程
        let mut handles = Vec::new();
        for worker_id in 0..actual_thread_count {
            let task_receiver = task_receiver.clone();
            let progress_sender = progress_sender.clone();
            let cancel_flag = cancel_flag.clone();
            let stats = stats.clone();
            let converter = converter.clone();
            let thread_pool = thread_pool.clone();
            
            let handle = thread::spawn(move || {
                Self::worker_thread(
                    worker_id,
                    task_receiver,
                    progress_sender,
                    cancel_flag,
                    stats,
                    converter,
                    thread_pool,
                );
            });
            
            handles.push(handle);
        }
        
        // 等待所有工作线程完成
        thread::spawn(move || {
            for handle in handles {
                if let Err(e) = handle.join() {
                    warn!("工作线程异常退出: {:?}", e);
                }
            }
            
            // 发送完成消息
            let final_stats = stats.lock().unwrap();
            let total_duration = final_stats.start_time
                .map(|start| start.elapsed())
                .unwrap_or_default();
            
            let results = vec![]; // TODO: 收集所有结果
            let _ = progress_sender.send(ProgressUpdate::AllTasksCompleted {
                success_count: final_stats.successful_tasks,
                error_count: final_stats.failed_tasks,
                total_duration,
                results,
            });
        });
        
        Ok(())
    }
    
    /// 执行音频转换任务的辅助方法
    fn convert_audio_task<C>(
        converter: &C,
        input_path: &std::path::Path,
        output_path: &std::path::Path,
        cancel_check: &dyn Fn() -> bool,
    ) -> Result<String, anyhow::Error>
    where
        C: AudioConverterTrait,
    {
        converter.convert_to_ogg_with_cancel(input_path, output_path, cancel_check)
    }
    
    /// 执行视频转换任务的辅助方法
    fn convert_video_task<C>(
        converter: &C,
        input_path: &std::path::Path,
        output_path: &std::path::Path,
    ) -> Result<(), anyhow::Error>
    where
        C: VideoConverterTrait,
    {
        converter.convert_to_ogv(input_path, output_path)
    }
    
    /// 工作线程函数
    fn worker_thread<C>(
        worker_id: usize,
        task_receiver: Receiver<ConversionTask>,
        progress_sender: Sender<ProgressUpdate>,
        cancel_flag: Arc<Mutex<bool>>,
        stats: Arc<Mutex<ConversionStats>>,
        converter: C,
        thread_pool: Arc<SmartThreadPool>,
    ) where
        C: Clone + Send + Sync + AudioConverterTrait + VideoConverterTrait + 'static,
    {
        info!("工作线程 {} 启动", worker_id);
        
        // 通知线程池线程开始工作
        thread_pool.thread_start(worker_id);
        
        while let Ok(task) = task_receiver.recv() {
            // 检查取消标志
            if *cancel_flag.lock().unwrap_or_else(|_| {
                warn!("获取取消标志失败，假设任务被取消");
                panic!("Mutex poisoned, cannot continue")
            }) {
                info!("工作线程 {} 收到取消信号", worker_id);
                break;
            }
            
            // 发送任务开始消息
            let filename = task.input_path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let total_tasks = stats.lock().unwrap().total_tasks;
            let _ = progress_sender.send(ProgressUpdate::TaskStarted {
                task_id: task.task_id(),
                filename: filename.clone(),
                total_tasks,
            });
            
            // 执行转换任务
            let start_time = Instant::now();
            let result = match &task {
                ConversionTask::Audio { input_path, output_path, task_id } => {
                    let cancel_check = || *cancel_flag.lock().unwrap_or_else(|_| {
                        warn!("获取取消标志失败，假设任务被取消");
                        panic!("Mutex poisoned, cannot continue")
                    });
                    
                    // 使用trait方法进行音频转换
                    match Self::convert_audio_task(&converter, input_path, output_path, &cancel_check) {
                        Ok(_) => {
                            ConversionResult::Success {
                                task_id: *task_id,
                                input_path: input_path.clone(),
                                output_path: output_path.clone(),
                                duration: start_time.elapsed(),
                                message: "音频转换成功".to_string(),
                            }
                        }
                        Err(e) => {
                            ConversionResult::Error {
                                task_id: *task_id,
                                input_path: input_path.clone(),
                                error: format!("音频转换失败: {}", e),
                            }
                        }
                    }
                }
                ConversionTask::Video { input_path, output_path, task_id } => {
                    // 使用trait方法进行视频转换
                    match Self::convert_video_task(&converter, input_path, output_path) {
                        Ok(_) => {
                            ConversionResult::Success {
                                task_id: *task_id,
                                input_path: input_path.clone(),
                                output_path: output_path.clone(),
                                duration: start_time.elapsed(),
                                message: "视频转换成功".to_string(),
                            }
                        }
                        Err(e) => {
                            ConversionResult::Error {
                                task_id: *task_id,
                                input_path: input_path.clone(),
                                error: format!("视频转换失败: {}", e),
                            }
                        }
                    }
                }
            };
            
            // 更新统计信息
            {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.completed_tasks += 1;
                match &result {
                    ConversionResult::Success { .. } => {
                        stats_guard.successful_tasks += 1;
                    }
                    ConversionResult::Error { .. } => {
                        stats_guard.failed_tasks += 1;
                    }
                }
            }
            
            // 发送任务完成消息
            let completed_count = stats.lock().unwrap().completed_tasks;
            let _ = progress_sender.send(ProgressUpdate::TaskCompleted {
                task_id: task.task_id(),
                result: result.clone(),
                completed_count,
                total_tasks,
            });
            
            debug!("工作线程 {} 完成任务 {}, 结果: {:?}", worker_id, task.task_id(), result);
        }
        
        // 通知线程池线程完成工作
        thread_pool.thread_finish(worker_id, Duration::from_secs(0)); // 这里可以记录实际耗时
        
        info!("工作线程 {} 退出", worker_id);
    }
    
    /// 获取进度接收器
    pub fn get_progress_receiver(&self) -> &Receiver<ProgressUpdate> {
        &self.progress_receiver
    }
    
    /// 取消所有任务
    pub fn cancel_all_tasks(&self) {
        *self.cancel_flag.lock().unwrap() = true;
        info!("并行转换任务取消信号已发送");
    }
    
    /// 重置统计信息
    fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = ConversionStats::default();
        }
    }
}

// 为ConversionTask实现辅助方法
impl ConversionTask {
    pub fn task_id(&self) -> usize {
        match self {
            ConversionTask::Audio { task_id, .. } => *task_id,
            ConversionTask::Video { task_id, .. } => *task_id,
        }
    }
    
    pub fn input_path(&self) -> &PathBuf {
        match self {
            ConversionTask::Audio { input_path, .. } => input_path,
            ConversionTask::Video { input_path, .. } => input_path,
        }
    }
}

// 为ConversionResult实现辅助方法
impl ConversionResult {
    pub fn input_path(&self) -> &PathBuf {
        match self {
            ConversionResult::Success { input_path, .. } => input_path,
            ConversionResult::Error { input_path, .. } => input_path,
        }
    }
}

// 为ConversionResult实现Display trait
impl fmt::Display for ConversionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionResult::Success { message, .. } => write!(f, "{}", message),
            ConversionResult::Error { error, .. } => write!(f, "{}", error),
        }
    }
}

// 为AudioConverter和VideoConverter实现Clone trait
impl Clone for AudioConverter {
    fn clone(&self) -> Self {
        Self {
            ffmpeg_path: self.ffmpeg_path.clone(),
        }
    }
}

impl Clone for VideoConverter {
    fn clone(&self) -> Self {
        Self {
            ffmpeg_path: self.ffmpeg_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.max_threads >= 2);
        assert!(config.max_threads <= 12);
        assert_eq!(config.queue_size, 1000);
        assert!(config.smart_scheduling);
    }
    
    #[test]
    fn test_parallel_config_adjustment() {
        let mut config = ParallelConfig::default();
        let original_threads = config.max_threads;
        
        // 测试大文件调整
        config.adjust_for_file_size(10, 150.0);
        assert!(config.max_threads <= original_threads);
        
        // 测试小文件大批量调整
        config.max_threads = original_threads;
        config.adjust_for_file_size(100, 5.0);
        assert!(config.max_threads >= original_threads);
    }
}
