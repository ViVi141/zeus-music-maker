use std::io::{Read, Write};
use std::path::Path;
use std::sync::LazyLock;
use xz2::read::XzDecoder;
use infer::Infer;
use anyhow::{Result, anyhow};
use std::ffi::CString;
use std::os::raw::c_char;

#[cfg(windows)]
use libc::c_void;

#[cfg(windows)]
use libloading::{Library, Symbol};


/// 酷狗KGM文件解密器
pub struct KuGouDecoder<'a> {
    origin: Box<dyn Read + 'a>,
    own_key: [u8; KuGouDecoder::OWN_KEY_LEN as usize],
    pos: u64,
}

impl<'a> KuGouDecoder<'a> {
    const HEADER_LEN: u64 = 1024;
    const OWN_KEY_LEN: u64 = 17;
    const PUB_KEY_LEN: u64 = 1170494464;
    const PUB_KEY_LEN_MAGNIFICATION: u64 = 16;
    const MAGIC_HEADER: [u8; 28] = [
        0x7c, 0xd5, 0x32, 0xeb, 0x86, 0x02, 0x7f, 0x4b, 0xa8, 0xaf, 0xa6, 0x8e, 0x0f, 0xff, 0x99,
        0x14, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    ];

    /// 获取公钥数据
    fn get_pub_key(index: Range<u64>) -> &'static [u8] {
        // 嵌入的酷狗密钥数据
        static KGM_KEY_XZ: &[u8] = include_bytes!("../assets/kugou_key.xz");
        static KEYS: LazyLock<Vec<u8>> = LazyLock::new(|| {
            let mut xz_decoder = XzDecoder::new(Bytes::new(KGM_KEY_XZ));
            let mut key = vec![0; (KuGouDecoder::PUB_KEY_LEN / KuGouDecoder::PUB_KEY_LEN_MAGNIFICATION) as usize];
            match xz_decoder.read_exact(&mut key) {
                Ok(_) => key,
                _ => {
                    panic!("Failed to decode the KuGou key")
                }
            }
        });

        &KEYS[(index.start / KuGouDecoder::PUB_KEY_LEN_MAGNIFICATION) as usize
            ..(index.end / KuGouDecoder::PUB_KEY_LEN_MAGNIFICATION + 1) as usize]
    }

    /// 尝试创建解密器
    pub fn try_new(mut origin: impl Read + 'a) -> Result<Self> {
        let mut buf = [0; KuGouDecoder::HEADER_LEN as usize];
        match origin.read(&mut buf) {
            Ok(len) if len == buf.len() && buf.starts_with(&KuGouDecoder::MAGIC_HEADER) => {
                let mut own_key = [0; KuGouDecoder::OWN_KEY_LEN as usize];
                own_key[..16].copy_from_slice(&buf[0x1c..0x2c]);
                Ok(KuGouDecoder {
                    origin: Box::new(origin),
                    own_key,
                    pos: 0,
                })
            }
            _ => Err(anyhow!("Invalid KGM file format"))
        }
    }

    /// 解密文件到指定路径
    pub fn decrypt_to_file(&mut self, output_path: &Path) -> Result<String> {
        let mut output_file = std::fs::File::create(output_path)?;
        let mut buf = [0; 16 * 1024];
        
        // 读取文件头用于格式检测
        let mut head_buffer = [0; 128];
        self.read(&mut head_buffer)?;
        
        // 检测音频格式
        let info: Infer = Infer::new();
        let ext = if let Some(kind) = info.get(&head_buffer) {
            match kind.mime_type() {
                "audio/midi" => "midi",
                "audio/opus" => "opus", 
                "audio/flac" => "flac",
                "audio/webm" => "weba",
                "audio/wav" => "wav",
                "audio/ogg" => "ogg",
                "audio/aac" => "aac",
                _ => "mp3",
            }
        } else {
            "mp3"
        };
        
        // 写入文件头
        output_file.write_all(&head_buffer)?;
        
        // 解密并写入剩余数据
        while let Ok(len) = self.read(&mut buf) {
            if len == 0 {
                break;
            }
            output_file.write_all(&buf[..len])?;
        }
        
        Ok(ext.to_string())
    }
}

impl<'a> Read for KuGouDecoder<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        const PUB_KEY_MEND: [u8; 272] = [
            0xB8, 0xD5, 0x3D, 0xB2, 0xE9, 0xAF, 0x78, 0x8C, 0x83, 0x33, 0x71, 0x51, 0x76, 0xA0,
            0xCD, 0x37, 0x2F, 0x3E, 0x35, 0x8D, 0xA9, 0xBE, 0x98, 0xB7, 0xE7, 0x8C, 0x22, 0xCE,
            0x5A, 0x61, 0xDF, 0x68, 0x69, 0x89, 0xFE, 0xA5, 0xB6, 0xDE, 0xA9, 0x77, 0xFC, 0xC8,
            0xBD, 0xBD, 0xE5, 0x6D, 0x3E, 0x5A, 0x36, 0xEF, 0x69, 0x4E, 0xBE, 0xE1, 0xE9, 0x66,
            0x1C, 0xF3, 0xD9, 0x02, 0xB6, 0xF2, 0x12, 0x9B, 0x44, 0xD0, 0x6F, 0xB9, 0x35, 0x89,
            0xB6, 0x46, 0x6D, 0x73, 0x82, 0x06, 0x69, 0xC1, 0xED, 0xD7, 0x85, 0xC2, 0x30, 0xDF,
            0xA2, 0x62, 0xBE, 0x79, 0x2D, 0x62, 0x62, 0x3D, 0x0D, 0x7E, 0xBE, 0x48, 0x89, 0x23,
            0x02, 0xA0, 0xE4, 0xD5, 0x75, 0x51, 0x32, 0x02, 0x53, 0xFD, 0x16, 0x3A, 0x21, 0x3B,
            0x16, 0x0F, 0xC3, 0xB2, 0xBB, 0xB3, 0xE2, 0xBA, 0x3A, 0x3D, 0x13, 0xEC, 0xF6, 0x01,
            0x45, 0x84, 0xA5, 0x70, 0x0F, 0x93, 0x49, 0x0C, 0x64, 0xCD, 0x31, 0xD5, 0xCC, 0x4C,
            0x07, 0x01, 0x9E, 0x00, 0x1A, 0x23, 0x90, 0xBF, 0x88, 0x1E, 0x3B, 0xAB, 0xA6, 0x3E,
            0xC4, 0x73, 0x47, 0x10, 0x7E, 0x3B, 0x5E, 0xBC, 0xE3, 0x00, 0x84, 0xFF, 0x09, 0xD4,
            0xE0, 0x89, 0x0F, 0x5B, 0x58, 0x70, 0x4F, 0xFB, 0x65, 0xD8, 0x5C, 0x53, 0x1B, 0xD3,
            0xC8, 0xC6, 0xBF, 0xEF, 0x98, 0xB0, 0x50, 0x4F, 0x0F, 0xEA, 0xE5, 0x83, 0x58, 0x8C,
            0x28, 0x2C, 0x84, 0x67, 0xCD, 0xD0, 0x9E, 0x47, 0xDB, 0x27, 0x50, 0xCA, 0xF4, 0x63,
            0x63, 0xE8, 0x97, 0x7F, 0x1B, 0x4B, 0x0C, 0xC2, 0xC1, 0x21, 0x4C, 0xCC, 0x58, 0xF5,
            0x94, 0x52, 0xA3, 0xF3, 0xD3, 0xE0, 0x68, 0xF4, 0x00, 0x23, 0xF3, 0x5E, 0x0A, 0x7B,
            0x93, 0xDD, 0xAB, 0x12, 0xB2, 0x13, 0xE8, 0x84, 0xD7, 0xA7, 0x9F, 0x0F, 0x32, 0x4C,
            0x55, 0x1D, 0x04, 0x36, 0x52, 0xDC, 0x03, 0xF3, 0xF9, 0x4E, 0x42, 0xE9, 0x3D, 0x61,
            0xEF, 0x7C, 0xB6, 0xB3, 0x93, 0x50,
        ];

        let len = self.origin.read(buf)?;
        let audio = &mut buf[..len];

        let pub_key = KuGouDecoder::get_pub_key(self.pos..self.pos + len as u64);

        for (byte, i) in audio.iter_mut().zip(self.pos..self.pos + len as u64) {
            let own_key = self.own_key[(i % self.own_key.len() as u64) as usize] ^ *byte;
            let own_key = own_key ^ (own_key & 0x0f) << 4;

            let pub_key = PUB_KEY_MEND[(i % PUB_KEY_MEND.len() as u64) as usize]
                ^ pub_key[(i / KuGouDecoder::PUB_KEY_LEN_MAGNIFICATION) as usize
                    - (self.pos / KuGouDecoder::PUB_KEY_LEN_MAGNIFICATION) as usize];
            let pub_key = pub_key ^ (pub_key & 0xf) << 4;
            *byte = own_key ^ pub_key;
        }

        self.pos += len as u64;
        Ok(len)
    }
}

/// 字节流读取器
struct Bytes<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Bytes<'a> {
    fn new(data: &'a [u8]) -> Self {
        Bytes { data, pos: 0 }
    }
}

impl Read for Bytes<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = std::cmp::min(buf.len(), self.data.len() - self.pos);

        buf[..len].copy_from_slice(&self.data[self.pos..self.pos + len]);
        self.pos += len;

        Ok(len)
    }
}

/// 音频解密管理器
pub struct AudioDecryptManager;

impl AudioDecryptManager {
    /// 解密酷狗KGM文件
    pub fn decrypt_kugou_file(input_path: &Path, output_dir: &Path) -> Result<String> {
        let input_file = std::fs::File::open(input_path)?;
        let mut decoder = KuGouDecoder::try_new(input_file)?;
        
        // 生成输出文件名
        let file_stem = input_path.file_stem()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_string_lossy();
        
        let output_path = output_dir.join(format!("{}.mp3", file_stem));
        let detected_format = decoder.decrypt_to_file(&output_path)?;
        
        // 如果检测到的格式不是mp3，重命名文件
        if detected_format != "mp3" {
            let final_path = output_dir.join(format!("{}.{}", file_stem, detected_format));
            std::fs::rename(&output_path, &final_path)?;
            Ok(final_path.to_string_lossy().to_string())
        } else {
            Ok(output_path.to_string_lossy().to_string())
        }
    }
    
    
    /// 解密网易云NCM文件
    pub fn decrypt_netease_file(input_path: &Path, output_dir: &Path) -> Result<String> {
        #[cfg(windows)]
        {
            // 生成输出文件名
            let file_stem = input_path.file_stem()
                .ok_or_else(|| anyhow!("Invalid file name"))?
                .to_string_lossy();
            
            // 使用libncmdump DLL解密（DLL会自动输出到源文件位置）
            Self::decrypt_ncm_with_dll(input_path)?;
            
            // 检查源文件目录中是否生成了mp3文件
            let input_dir = input_path.parent().unwrap();
            let possible_output_paths = vec![
                input_dir.join(format!("{}.mp3", file_stem)),
                input_dir.join(format!("{}.flac", file_stem)),
            ];
            
            let mut found_path = None;
            for path in &possible_output_paths {
                if path.exists() {
                    found_path = Some(path.clone());
                    break;
                }
            }
            
            let output_path = found_path.ok_or_else(|| {
                anyhow!("解密完成但未找到输出文件。检查目录: {}", input_dir.display())
            })?;
            
            // 如果输出目录不是源文件目录，移动文件到指定目录
            if output_dir != input_dir {
                let final_output_path = output_dir.join(output_path.file_name().unwrap());
                std::fs::create_dir_all(output_dir)?;
                
                // 使用复制+删除的方式处理跨磁盘移动
                std::fs::copy(&output_path, &final_output_path)?;
                std::fs::remove_file(&output_path)?;
                
                Ok(final_output_path.to_string_lossy().to_string())
            } else {
                Ok(output_path.to_string_lossy().to_string())
            }
        }
        
        #[cfg(not(windows))]
        {
            Err(anyhow!("NCM解密仅在Windows平台支持"))
        }
    }
    
    /// 使用libncmdump DLL解密NCM文件
    #[cfg(windows)]
    fn decrypt_ncm_with_dll(input_path: &Path) -> Result<()> {
        // 尝试多个DLL路径
        let dll_paths = vec![
            "libncmdump.dll",  // 当前目录
            "lib/libncmdump.dll",  // lib文件夹
            "libncmdump-1.5.0-windows-amd64-msvc/libncmdump.dll",  // 原始文件夹
        ];
        
        let mut lib = None;
        let mut last_error = None;
        
        for dll_path in &dll_paths {
            match unsafe { Library::new(dll_path) } {
                Ok(l) => {
                    lib = Some(l);
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        
        let lib = lib.ok_or_else(|| {
            anyhow!("Failed to load libncmdump.dll from any path. Last error: {:?}", last_error)
        })?;
        
        // 获取函数指针
        let create_netease_crypt: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = 
            unsafe { lib.get(b"CreateNeteaseCrypt") }
                .map_err(|e| anyhow!("Failed to get CreateNeteaseCrypt function: {}", e))?;
        
        let dump: Symbol<unsafe extern "C" fn(*mut c_void, *const c_char) -> i32> = 
            unsafe { lib.get(b"Dump") }
                .map_err(|e| anyhow!("Failed to get Dump function: {}", e))?;
        
        let fix_metadata: Symbol<unsafe extern "C" fn(*mut c_void)> = 
            unsafe { lib.get(b"FixMetadata") }
                .map_err(|e| anyhow!("Failed to get FixMetadata function: {}", e))?;
        
        let destroy_netease_crypt: Symbol<unsafe extern "C" fn(*mut c_void)> = 
            unsafe { lib.get(b"DestroyNeteaseCrypt") }
                .map_err(|e| anyhow!("Failed to get DestroyNeteaseCrypt function: {}", e))?;
        
        // 将路径转换为UTF-8编码的C字符串
        let input_cstr = CString::new(input_path.to_string_lossy().as_bytes())?;
        
        // 根据ncmdump文档，传递空字符串让DLL自动决定输出路径
        let output_cstr = CString::new("")?;
        
        unsafe {
            // 创建NeteaseCrypt实例
            let netease_crypt = create_netease_crypt(input_cstr.as_ptr());
            if netease_crypt.is_null() {
                return Err(anyhow!("Failed to create NeteaseCrypt instance"));
            }
            
            // 执行解密
            let result = dump(netease_crypt, output_cstr.as_ptr());
            
            // 修复元数据
            fix_metadata(netease_crypt);
            
            // 销毁实例
            destroy_netease_crypt(netease_crypt);
            
            if result == 0 {
                Ok(())
            } else {
                Err(anyhow!("NCM解密失败，返回码: {}", result))
            }
        }
    }
    

    /// 检查文件是否为酷狗KGM格式
    pub fn is_kugou_file(path: &Path) -> bool {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut header = [0; 28];
            if let Ok(_) = std::io::Read::read_exact(&mut file, &mut header) {
                return header.starts_with(&KuGouDecoder::MAGIC_HEADER);
            }
        }
        false
    }

    /// 检查文件是否为网易云NCM格式
    pub fn is_netease_file(path: &Path) -> bool {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut header = [0; 8];
            if let Ok(_) = std::io::Read::read_exact(&mut file, &mut header) {
                // NCM文件头是两个4字节整数：
                // 第一个4字节：0x4e455443 (对应字符串"CTEN")
                // 第二个4字节：0x4d414446 (对应字符串"FDAM")
                let first_word = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
                let second_word = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
                
                return first_word == 0x4e455443 && second_word == 0x4d414446;
            }
        }
        false
    }
}

use std::ops::Range;
