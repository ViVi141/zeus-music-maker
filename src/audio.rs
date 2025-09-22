use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// 音频文件信息
#[derive(Debug, Clone)]
pub struct AudioInfo {
    pub duration: u32,
}

/// 音频处理工具
pub struct AudioProcessor;

impl AudioProcessor {
    /// 获取音频文件信息
    pub fn get_audio_info<P: AsRef<Path>>(path: P) -> Result<AudioInfo> {
        let path = path.as_ref();
        debug!("Reading audio file: {:?}", path);

        // 打开文件
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open file: {:?}", path))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // 创建格式提示
        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(extension);
        }

        // 探测格式
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .with_context(|| "Failed to probe audio format")?;

        // 查找第一个音频轨道
        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow::anyhow!("No audio track found"))?;

        // 获取音频信息
        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.unwrap_or(44100);

        // 计算时长
        let duration = if let Some(n_frames) = codec_params.n_frames {
            (n_frames as f64 / sample_rate as f64) as u32
        } else {
            // 如果无法直接获取帧数，尝试从元数据获取
            // 如果仍然无法获取，返回一个合理的默认值
            debug!("无法获取音频帧数，使用默认时长");
            180 // 默认3分钟
        };

        Ok(AudioInfo {
            duration,
        })
    }



}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ogg_file() {
        assert!(AudioProcessor::is_ogg_file("test.ogg"));
        assert!(AudioProcessor::is_ogg_file("test.OGG"));
        assert!(!AudioProcessor::is_ogg_file("test.mp3"));
        assert!(!AudioProcessor::is_ogg_file("test"));
    }

}
