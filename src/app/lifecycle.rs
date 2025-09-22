/*!
 * 应用程序生命周期管理
 * 提供基本的启动时间跟踪
 */

use std::time::{Duration, Instant};

/// 应用程序生命周期管理器
pub struct AppLifecycle {
    /// 启动时间
    start_time: Instant,
}

impl AppLifecycle {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// 获取运行时间
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for AppLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
