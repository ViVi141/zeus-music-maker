/*!
 * 视频分片并行处理器
 * 支持多个视频文件的分片并行转换，最大化多线程利用率
 */

use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use log::{info, warn, debug};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::video_chunk_converter::{VideoChunkConverter, VideoChunk, VideoChunkConfig, VideoChunkConversionResult};
use crate::resource_manager::GlobalResourceManager;

/// 分片转换任务
#[derive(Debug, Clone)]
pub struct ChunkConversionTask {
    /// 任务ID
    pub task_id: usize,
    /// 原始视频文件路径
    pub input_path: PathBuf,
    /// 分片信息
    pub chunks: Vec<VideoChunk>,
    /// 视频质量
    pub video_quality: u8,
    /// 音频质量
    pub audio_quality: u8,
    /// 最终输出文件路径
    pub final_output_path: PathBuf,
}

/// 分片转换结果
#[derive(Debug, Clone)]
pub struct ChunkConversionTaskResult {
    /// 任务ID
    pub task_id: usize,
    /// 转换结果
    pub result: VideoChunkConversionResult,
}

/// 分片进度更新消息
#[derive(Debug, Clone)]
pub enum ChunkProgressUpdate {
    /// 任务开始
    TaskStarted {
        task_id: usize,
        input_path: PathBuf,
        chunk_count: usize,
    },
    /// 分片开始转换
    ChunkStarted {
        task_id: usize,
        chunk_index: usize,
        chunk_path: PathBuf,
    },
    /// 分片转换完成
    ChunkCompleted {
        task_id: usize,
        chunk_index: usize,
        success: bool,
        error: Option<String>,
    },
    /// 任务完成
    TaskCompleted {
        task_id: usize,
        result: ChunkConversionTaskResult,
    },
    /// 所有任务完成
    AllTasksCompleted {
        success_count: usize,
        error_count: usize,
        total_duration: Duration,
        results: Vec<ChunkConversionTaskResult>,
    },
}

/// 视频分片并行处理器
pub struct VideoChunkParallelProcessor {
    /// 配置
    config: VideoChunkConfig,
    /// 最大并发线程数
    max_threads: usize,
    /// 进度更新发送器
    progress_sender: Sender<ChunkProgressUpdate>,
    /// 进度更新接收器
    progress_receiver: Receiver<ChunkProgressUpdate>,
    /// 取消标志
    cancel_flag: Arc<Mutex<bool>>,
    /// 统计信息
    stats: Arc<Mutex<ChunkConversionStats>>,
    /// 资源管理器
    resource_manager: Arc<GlobalResourceManager>,
}

/// 分片转换统计信息
#[derive(Debug, Default, Clone)]
struct ChunkConversionStats {
    total_tasks: usize,
    completed_tasks: usize,
    successful_tasks: usize,
    failed_tasks: usize,
    total_chunks: usize,
    completed_chunks: usize,
    successful_chunks: usize,
    failed_chunks: usize,
    start_time: Option<Instant>,
}

impl VideoChunkParallelProcessor {
    /// 创建新的分片并行处理器
    pub fn new(config: VideoChunkConfig) -> Self {
        let max_threads = Self::calculate_optimal_threads();
        let (progress_sender, progress_receiver) = bounded(1000);
        
        Self {
            config,
            max_threads,
            progress_sender,
            progress_receiver,
            cancel_flag: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::default()),
            resource_manager: Arc::new(GlobalResourceManager::new()),
        }
    }

    /// 计算最优线程数
    fn calculate_optimal_threads() -> usize {
        let cpu_count = num_cpus::get();
        // 对于视频转换这种CPU密集型任务，使用CPU核心数
        // 但考虑到内存使用，限制最大线程数
        (cpu_count * 2).min(12).max(2)
    }

    /// 处理多个视频文件的分片并行转换
    pub fn process_videos_parallel(
        &self,
        input_files: Vec<PathBuf>,
        output_dir: PathBuf,
        video_quality: u8,
        audio_quality: u8,
    ) -> Result<()> {
        info!("开始分片并行转换 {} 个视频文件", input_files.len());
        
        // 重置统计信息
        {
            let mut stats = self.stats.lock().unwrap_or_else(|e| {
                warn!("统计信息Mutex poisoned: {:?}，使用默认值", e);
                e.into_inner()
            });
            *stats = ChunkConversionStats::default();
            stats.start_time = Some(Instant::now());
        }

        // 重置取消标志
        {
            let mut cancel_flag = self.cancel_flag.lock().unwrap_or_else(|e| {
                warn!("取消标志Mutex poisoned: {:?}，使用默认值", e);
                e.into_inner()
            });
            *cancel_flag = false;
        }

        // 创建分片转换任务
        let tasks = self.create_conversion_tasks(input_files, &output_dir, video_quality, audio_quality)?;
        
        if tasks.is_empty() {
            warn!("没有有效的转换任务");
            return Ok(());
        }

        // 更新统计信息
        {
            let mut stats = self.stats.lock().unwrap_or_else(|e| {
                warn!("统计信息Mutex poisoned: {:?}，使用默认值", e);
                e.into_inner()
            });
            stats.total_tasks = tasks.len();
            stats.total_chunks = tasks.iter().map(|t| t.chunks.len()).sum();
        }

        info!("创建了 {} 个转换任务，总计 {} 个分片", tasks.len(), 
              tasks.iter().map(|t| t.chunks.len()).sum::<usize>());

        // 启动并行处理
        self.start_parallel_processing(tasks)?;

        Ok(())
    }

    /// 创建转换任务
    fn create_conversion_tasks(
        &self,
        input_files: Vec<PathBuf>,
        output_dir: &std::path::Path,
        video_quality: u8,
        audio_quality: u8,
    ) -> Result<Vec<ChunkConversionTask>> {
        let mut tasks = Vec::new();
        let converter = VideoChunkConverter::new(self.config.clone())?;

        for (task_id, input_path) in input_files.into_iter().enumerate() {
            // 检查文件是否存在
            if !input_path.exists() {
                warn!("输入文件不存在，跳过: {}", input_path.display());
                continue;
            }

            // 为每个视频创建单独的输出目录（使用安全文件名）
            let safe_dir_name = if let Some(file_stem) = input_path.file_stem() {
                crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&file_stem.to_string_lossy())
            } else {
                format!("video_{:03}", task_id)
            };
            let video_output_dir = output_dir.join(safe_dir_name);

            // 生成分片计划
            match converter.create_chunk_plan(&input_path, &video_output_dir) {
                Ok(chunks) => {
                    // 使用安全文件名
                    let safe_filename = if let Some(file_stem) = input_path.file_stem() {
                        crate::utils::string_utils::StringUtils::to_ascii_safe_pinyin(&file_stem.to_string_lossy())
                    } else {
                        format!("video_{:03}", task_id)
                    };
                    let mut final_output_path = output_dir.join(format!("{}.ogv", safe_filename));
                    // 确保路径长度在限制内
                    final_output_path = crate::utils::string_utils::StringUtils::ensure_path_length(&final_output_path, 260)
                        .unwrap_or_else(|_| final_output_path.clone());
                    // 确保文件名唯一
                    final_output_path = crate::utils::string_utils::StringUtils::ensure_unique_path(final_output_path);

                    let chunk_count = chunks.len();
                    tasks.push(ChunkConversionTask {
                        task_id,
                        input_path: input_path.clone(),
                        chunks,
                        video_quality,
                        audio_quality,
                        final_output_path,
                    });

                    info!("为视频创建了转换任务: {} ({}个分片)", 
                          input_path.display(), chunk_count);
                }
                Err(e) => {
                    warn!("为视频创建分片计划失败: {} - {}", input_path.display(), e);
                }
            }
        }

        Ok(tasks)
    }

    /// 启动并行处理
    fn start_parallel_processing(&self, tasks: Vec<ChunkConversionTask>) -> Result<()> {
        // 创建任务队列
        let (task_sender, task_receiver) = bounded(tasks.len());
        
        // 发送所有任务到队列
        for task in tasks {
            if let Err(e) = task_sender.send(task) {
                warn!("发送任务到队列失败: {}", e);
            }
        }
        drop(task_sender);

        // 获取智能线程池
        let thread_pool = self.resource_manager.get_thread_pool();
        thread_pool.adjust_thread_count();
        let actual_thread_count = thread_pool.get_max_threads().min(self.max_threads);
        
        info!("使用 {} 个线程进行分片并行转换", actual_thread_count);

        // 启动工作线程
        let mut handles = Vec::new();
        for worker_id in 0..actual_thread_count {
            let task_receiver = task_receiver.clone();
            let progress_sender = self.progress_sender.clone();
            let cancel_flag = self.cancel_flag.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();

            let handle = thread::spawn(move || {
                Self::worker_thread(
                    worker_id,
                    task_receiver,
                    progress_sender,
                    cancel_flag,
                    stats,
                    config,
                );
            });

            handles.push(handle);
        }

        // 等待所有工作线程完成并处理最终结果
        let progress_sender = self.progress_sender.clone();
        let stats = self.stats.clone();
        thread::spawn(move || {
            for handle in handles {
                if let Err(e) = handle.join() {
                    warn!("工作线程异常退出: {:?}", e);
                }
            }

            // 发送完成消息
            let final_stats = stats.lock().unwrap_or_else(|e| {
                warn!("统计信息Mutex poisoned: {:?}，使用默认值", e);
                e.into_inner()
            });
            let total_duration = final_stats.start_time
                .map(|start| start.elapsed())
                .unwrap_or_default();

            let results = vec![]; // TODO: 收集所有结果
            let _ = progress_sender.send(ChunkProgressUpdate::AllTasksCompleted {
                success_count: final_stats.successful_tasks,
                error_count: final_stats.failed_tasks,
                total_duration,
                results,
            });
        });

        Ok(())
    }

    /// 工作线程函数
    fn worker_thread(
        worker_id: usize,
        task_receiver: Receiver<ChunkConversionTask>,
        progress_sender: Sender<ChunkProgressUpdate>,
        cancel_flag: Arc<Mutex<bool>>,
        stats: Arc<Mutex<ChunkConversionStats>>,
        config: VideoChunkConfig,
    ) {
        info!("分片转换工作线程 {} 启动", worker_id);

        while let Ok(task) = task_receiver.recv() {
            // 检查取消标志
            if *cancel_flag.lock().unwrap_or_else(|e| {
                warn!("取消标志Mutex poisoned: {:?}，假设任务被取消", e);
                e.into_inner()
            }) {
                info!("工作线程 {} 收到取消信号，退出", worker_id);
                break;
            }

            // 发送任务开始消息
            let _ = progress_sender.send(ChunkProgressUpdate::TaskStarted {
                task_id: task.task_id,
                input_path: task.input_path.clone(),
                chunk_count: task.chunks.len(),
            });

            // 执行分片转换
            match Self::process_single_video(task, &progress_sender, &cancel_flag, &stats, &config) {
                Ok(result) => {
                    let _ = progress_sender.send(ChunkProgressUpdate::TaskCompleted {
                        task_id: result.task_id,
                        result: result.clone(),
                    });

                    // 更新统计信息
                    let mut stats = stats.lock().unwrap_or_else(|e| {
                        warn!("统计信息Mutex poisoned: {:?}，使用默认值", e);
                        e.into_inner()
                    });
                    stats.completed_tasks += 1;
                    if result.result.success {
                        stats.successful_tasks += 1;
                    } else {
                        stats.failed_tasks += 1;
                    }
                }
                Err(e) => {
                    warn!("处理视频任务失败: {}", e);
                    
                    // 更新统计信息
                    let mut stats = stats.lock().unwrap_or_else(|e| {
                        warn!("统计信息Mutex poisoned: {:?}，使用默认值", e);
                        e.into_inner()
                    });
                    stats.completed_tasks += 1;
                    stats.failed_tasks += 1;
                }
            }
        }

        info!("分片转换工作线程 {} 退出", worker_id);
    }

    /// 处理单个视频的分片转换
    fn process_single_video(
        task: ChunkConversionTask,
        progress_sender: &Sender<ChunkProgressUpdate>,
        cancel_flag: &Arc<Mutex<bool>>,
        stats: &Arc<Mutex<ChunkConversionStats>>,
        config: &VideoChunkConfig,
    ) -> Result<ChunkConversionTaskResult> {
        let _start_time = Instant::now();
        
        // 创建分片转换器
        let converter = VideoChunkConverter::new(config.clone())?;
        
        let mut successful_chunks = 0;
        let mut failed_chunks = 0;
        let mut error_messages = Vec::new();

        // 并行转换所有分片
        let chunk_results = Self::convert_chunks_parallel(
            &converter,
            &task.chunks,
            task.video_quality,
            task.audio_quality,
            progress_sender,
            &task.task_id,
            cancel_flag,
        )?;

        // 统计分片结果
        for (chunk_index, result) in chunk_results.iter().enumerate() {
            match result {
                Ok(_) => {
                    successful_chunks += 1;
                    debug!("分片 {} 转换成功", chunk_index);
                }
                Err(e) => {
                    failed_chunks += 1;
                    error_messages.push(format!("分片 {} 转换失败: {}", chunk_index, e));
                    warn!("分片 {} 转换失败: {}", chunk_index, e);
                }
            }
        }

        // 更新分片统计信息
        {
            let mut stats = stats.lock().unwrap_or_else(|e| {
                warn!("统计信息Mutex poisoned: {:?}，使用默认值", e);
                e.into_inner()
            });
            stats.completed_chunks += task.chunks.len();
            stats.successful_chunks += successful_chunks;
            stats.failed_chunks += failed_chunks;
        }

        // 如果所有分片都成功，合并分片
        let success = failed_chunks == 0;
        let error = if success {
            None
        } else {
            Some(error_messages.join("; "))
        };

        if success && task.chunks.len() > 1 {
            // 合并分片
            if let Err(e) = converter.merge_chunks(&task.chunks, &task.final_output_path) {
                warn!("合并分片失败: {}", e);
                let _input_path = task.input_path.clone();
                return Ok(ChunkConversionTaskResult {
                    task_id: task.task_id,
                    result: VideoChunkConversionResult {
                        output_path: task.final_output_path.clone(),
                        chunks: task.chunks,
                        success: false,
                        error: Some(format!("合并分片失败: {}", e)),
                    },
                });
            }
        }

        // 清理临时分片文件
        converter.cleanup_chunks(&task.chunks);

        Ok(ChunkConversionTaskResult {
            task_id: task.task_id,
            result: VideoChunkConversionResult {
                output_path: task.final_output_path,
                chunks: task.chunks,
                success,
                error,
            },
        })
    }

    /// 并行转换分片
    fn convert_chunks_parallel(
        converter: &VideoChunkConverter,
        chunks: &[VideoChunk],
        video_quality: u8,
        audio_quality: u8,
        progress_sender: &Sender<ChunkProgressUpdate>,
        task_id: &usize,
        cancel_flag: &Arc<Mutex<bool>>,
    ) -> Result<Vec<Result<(), anyhow::Error>>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

        // 创建分片任务队列
        let (chunk_sender, chunk_receiver) = bounded(chunks.len());
        
        // 发送所有分片到队列
        for (index, chunk) in chunks.iter().enumerate() {
            if let Err(e) = chunk_sender.send((index, chunk.clone())) {
                warn!("发送分片任务失败: {}", e);
            }
        }
        drop(chunk_sender);

        // 使用线程池并行转换分片
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads((chunks.len() / 2).max(2).min(8)) // 根据分片数量调整线程数
            .build()
            .context("创建分片转换线程池失败")?;

        let mut results: Vec<Result<(), anyhow::Error>> = Vec::with_capacity(chunks.len());
        for _ in 0..chunks.len() {
            results.push(Ok(()));
        }
        let results_mutex = Arc::new(Mutex::new(results));

        thread_pool.scope(|s| {
            while let Ok((chunk_index, chunk)) = chunk_receiver.recv() {
                // 检查取消标志
                if *cancel_flag.lock().unwrap_or_else(|e| {
                    warn!("取消标志Mutex poisoned: {:?}，假设任务被取消", e);
                    e.into_inner()
                }) {
                    break;
                }

                let progress_sender = progress_sender.clone();
                let task_id = *task_id;
                let results_mutex = results_mutex.clone();

                s.spawn(move |_| {
                    // 发送分片开始消息
                    let _ = progress_sender.send(ChunkProgressUpdate::ChunkStarted {
                        task_id,
                        chunk_index,
                        chunk_path: chunk.output_path.clone(),
                    });

                    // 转换分片
                    let result = converter.convert_chunk(&chunk, video_quality, audio_quality);

                    // 发送分片完成消息
                    let (success, error) = match &result {
                        Ok(_) => (true, None),
                        Err(e) => (false, Some(e.to_string())),
                    };

                    let _ = progress_sender.send(ChunkProgressUpdate::ChunkCompleted {
                        task_id,
                        chunk_index,
                        success,
                        error,
                    });

                    // 存储结果
                    if let Ok(mut results) = results_mutex.lock() {
                        if chunk_index < results.len() {
                            results[chunk_index] = result;
                        }
                    }
                });
            }
        });

        let results = results_mutex.lock().unwrap_or_else(|e| {
            warn!("结果Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        });
        let mut final_results = Vec::new();
        for result in results.iter() {
            final_results.push(match result {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            });
        }
        Ok(final_results)
    }

    /// 获取进度接收器
    pub fn get_progress_receiver(&self) -> &Receiver<ChunkProgressUpdate> {
        &self.progress_receiver
    }

}


