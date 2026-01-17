/*!
 * 资源管理模块
 * 用于优化内存使用、CPU调度和磁盘I/O
 */

use log::{info, debug, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;


/// 智能线程池管理器
pub struct SmartThreadPool {
    /// 当前活跃线程数
    active_threads: Arc<Mutex<usize>>,
    /// 最大线程数
    max_threads: Arc<Mutex<usize>>,
    /// 线程性能统计
    thread_stats: Arc<Mutex<HashMap<usize, ThreadStats>>>,
}

/// 线程性能统计
#[derive(Debug)]
struct ThreadStats {
    tasks_completed: usize,
    total_time: Duration,
    last_activity: Instant,
}

impl Default for ThreadStats {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            total_time: Duration::default(),
            last_activity: Instant::now(),
        }
    }
}

impl SmartThreadPool {
    pub fn new(max_threads: usize) -> Self {
        Self {
            active_threads: Arc::new(Mutex::new(0)),
            max_threads: Arc::new(Mutex::new(max_threads)),
            thread_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 动态调整最大线程数（快速版本，避免启动延迟）
    pub fn adjust_thread_count(&self) {
        let mut max_threads = self.max_threads.lock().unwrap_or_else(|e| {
            warn!("最大线程数Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        });
        let original_count = *max_threads;
        
        // 简化逻辑：直接使用CPU核心数作为基础，避免耗时的系统调用
        let cpu_cores = num_cpus::get();
        let optimal_threads = cpu_cores.min(8); // 最多8个线程
        
        if *max_threads != optimal_threads {
            *max_threads = optimal_threads;
            info!("调整线程数: {} -> {} (基于CPU核心数: {})", original_count, *max_threads, cpu_cores);
        }
    }
    
    /// 获取当前最大线程数
    pub fn get_max_threads(&self) -> usize {
        *self.max_threads.lock().unwrap_or_else(|e| {
            warn!("最大线程数Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        })
    }
    
    /// 获取当前活跃线程数
    pub fn get_active_threads(&self) -> usize {
        *self.active_threads.lock().unwrap_or_else(|e| {
            warn!("活跃线程数Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        })
    }
    
    /// 线程开始工作
    pub fn thread_start(&self, thread_id: usize) {
        *self.active_threads.lock().unwrap_or_else(|e| {
            warn!("活跃线程数Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        }) += 1;
        
        let mut stats = self.thread_stats.lock().unwrap_or_else(|e| {
            warn!("线程统计Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        });
        stats.entry(thread_id).or_insert_with(ThreadStats::default).last_activity = Instant::now();
        
        debug!("线程 {} 开始工作，当前活跃线程数: {}", thread_id, self.get_active_threads());
    }
    
    /// 线程完成工作
    pub fn thread_finish(&self, thread_id: usize, task_duration: Duration) {
        *self.active_threads.lock().unwrap_or_else(|e| {
            warn!("活跃线程数Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        }) -= 1;
        
        let mut stats = self.thread_stats.lock().unwrap_or_else(|e| {
            warn!("线程统计Mutex poisoned: {:?}，使用默认值", e);
            e.into_inner()
        });
        if let Some(thread_stat) = stats.get_mut(&thread_id) {
            thread_stat.tasks_completed += 1;
            thread_stat.total_time += task_duration;
        }
        
        debug!("线程 {} 完成工作，耗时: {:?}", thread_id, task_duration);
    }
    
}


/// 磁盘I/O优化器
pub struct DiskIOOptimizer {
    /// 写入缓冲区大小
    write_buffer_size: usize,
    /// 读取缓冲区大小
    read_buffer_size: usize,
    /// 并发I/O操作数
    concurrent_io_ops: usize,
}

impl DiskIOOptimizer {
    pub fn new() -> Self {
        Self {
            write_buffer_size: 64 * 1024, // 64KB
            read_buffer_size: 64 * 1024,   // 64KB
            concurrent_io_ops: 4,          // 最多4个并发I/O操作
        }
    }
    
    
    /// 获取写入缓冲区大小
    #[allow(dead_code)]
    pub fn get_write_buffer_size(&self) -> usize {
        self.write_buffer_size
    }
    
    /// 获取读取缓冲区大小
    #[allow(dead_code)]
    pub fn get_read_buffer_size(&self) -> usize {
        self.read_buffer_size
    }
    
    /// 获取并发I/O操作数
    #[allow(dead_code)]
    pub fn get_concurrent_io_ops(&self) -> usize {
        self.concurrent_io_ops
    }
}

impl Default for DiskIOOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局资源管理器
pub struct GlobalResourceManager {
    /// 智能线程池
    thread_pool: Arc<SmartThreadPool>,
    /// 磁盘I/O优化器
    #[allow(dead_code)]
    disk_optimizer: Arc<Mutex<DiskIOOptimizer>>,
}

impl GlobalResourceManager {
    pub fn new() -> Self {
        let thread_pool = Arc::new(SmartThreadPool::new(Self::get_initial_thread_count()));
        let disk_optimizer = DiskIOOptimizer::new(); // 使用默认配置，避免系统调用
        
        Self {
            thread_pool,
            disk_optimizer: Arc::new(Mutex::new(disk_optimizer)),
        }
    }
    
    /// 获取初始线程数
    fn get_initial_thread_count() -> usize {
        let cpu_count = num_cpus::get();
        (cpu_count * 2).min(8).max(2)
    }
    
    /// 获取智能线程池
    pub fn get_thread_pool(&self) -> Arc<SmartThreadPool> {
        self.thread_pool.clone()
    }
    
    
}

impl Default for GlobalResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn test_smart_thread_pool() {
        let pool = SmartThreadPool::new(4);
        assert_eq!(pool.get_max_threads(), 4);
        assert_eq!(pool.get_active_threads(), 0);
        
        pool.thread_start(1);
        assert_eq!(pool.get_active_threads(), 1);
        
        pool.thread_finish(1, Duration::from_secs(1));
        assert_eq!(pool.get_active_threads(), 0);
    }
    
    #[test]
    fn test_disk_io_optimizer() {
        let optimizer = DiskIOOptimizer::new();
        
        assert!(optimizer.get_write_buffer_size() > 0);
        assert!(optimizer.get_read_buffer_size() > 0);
        assert!(optimizer.get_concurrent_io_ops() > 0);
    }
}
