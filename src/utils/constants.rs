/*!
 * 应用程序常量定义
 */

/// 音频解密相关常量
pub mod audio_decrypt {
    pub const HEADER_LEN: u64 = 1024;
    pub const OWN_KEY_LEN: u64 = 17;
    pub const PUB_KEY_LEN: u64 = 1170494464;
    pub const PUB_KEY_LEN_MAGNIFICATION: u64 = 16;
    
    /// 酷狗KGM文件魔数头
    pub const KUGOU_MAGIC_HEADER: [u8; 28] = [
        0x7c, 0xd5, 0x32, 0xeb, 0x86, 0x02, 0x7f, 0x4b, 0xa8, 0xaf, 0xa6, 0x8e, 0x0f, 0xff, 0x99,
        0x14, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    ];
    
}


/// 文件操作相关常量
pub mod file_ops {
    /// 默认轨道时长（秒）
    pub const DEFAULT_TRACK_DURATION: u32 = 180;
    /// 默认分贝值
    pub const DEFAULT_DECIBELS: i32 = 0;
    /// 最大文件大小（MB）
    pub const MAX_FILE_SIZE_MB: u64 = 500; // 500 MB
}


/// 应用程序相关常量
pub mod app {
    /// 应用程序名称
    pub const APP_NAME: &str = "宙斯音乐制作器";
    /// 最小栈大小
    pub const MIN_STACK_SIZE: u64 = 8388608; // 8MB
    /// 最大栈大小
    pub const MAX_STACK_SIZE: u64 = 8388608; // 8MB
}
