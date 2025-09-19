use anyhow::{Context, Result};
use image::{DynamicImage, RgbaImage, GenericImageView, imageops};
use log::{debug, info};
use std::path::Path;
use egui::TextureHandle;

/// 裁剪区域选择（相对于原始图片的比例，0.0-1.0）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CropSelection {
    /// 裁剪区域起始X坐标比例 (0.0-1.0)
    pub start_x_ratio: f32,
    /// 裁剪区域起始Y坐标比例 (0.0-1.0)
    pub start_y_ratio: f32,
    /// 裁剪区域宽度比例 (0.0-1.0)
    pub width_ratio: f32,
    /// 裁剪区域高度比例 (0.0-1.0)
    pub height_ratio: f32,
    /// 是否正在拖拽
    pub is_dragging: bool,
}

impl Default for CropSelection {
    fn default() -> Self {
        Self {
            start_x_ratio: 0.0,
            start_y_ratio: 0.0,
            width_ratio: 1.0,
            height_ratio: 1.0,
            is_dragging: false,
        }
    }
}

impl CropSelection {
    /// 获取裁剪区域在原始图片中的像素坐标
    pub fn get_pixel_coords(&self, original_width: u32, original_height: u32) -> (u32, u32, u32, u32) {
        let start_x = (self.start_x_ratio * original_width as f32) as u32;
        let start_y = (self.start_y_ratio * original_height as f32) as u32;
        let width = (self.width_ratio * original_width as f32) as u32;
        let height = (self.height_ratio * original_height as f32) as u32;
        
        // 确保不超出图片边界
        let start_x = start_x.min(original_width);
        let start_y = start_y.min(original_height);
        let width = width.min(original_width - start_x);
        let height = height.min(original_height - start_y);
        
        (start_x, start_y, width, height)
    }
    
}

/// PAA转换选项
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaaOptions {
    /// 是否裁剪到2的次方尺寸
    pub crop_to_power_of_two: bool,
    /// 目标尺寸（如果为None，则自动选择最接近的2的次方）
    pub target_size: Option<u32>,
    /// 是否居中裁剪
    pub center_crop: bool,
}

impl Default for PaaOptions {
    fn default() -> Self {
        Self {
            crop_to_power_of_two: true,
            target_size: None,
            center_crop: true,
        }
    }
}

/// 图片纹理管理器
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageTextureManager {
    /// 当前图片的原始尺寸
    pub original_size: (u32, u32),
    /// 当前图片的显示尺寸
    pub display_size: (f32, f32),
    /// 当前图片路径
    pub current_image_path: Option<std::path::PathBuf>,
}

/// 运行时图片纹理管理器（包含不可序列化的TextureHandle）
pub struct RuntimeImageTextureManager {
    /// 当前加载的图片纹理
    pub current_texture: Option<TextureHandle>,
    /// 基础管理器
    pub base: ImageTextureManager,
}

impl std::fmt::Debug for RuntimeImageTextureManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeImageTextureManager")
            .field("current_texture", &self.current_texture.is_some())
            .field("base", &self.base)
            .finish()
    }
}

impl Clone for RuntimeImageTextureManager {
    fn clone(&self) -> Self {
        Self {
            current_texture: None, // TextureHandle不能克隆，重新创建
            base: self.base.clone(),
        }
    }
}

impl Default for ImageTextureManager {
    fn default() -> Self {
        Self {
            original_size: (0, 0),
            display_size: (0.0, 0.0),
            current_image_path: None,
        }
    }
}

impl Default for RuntimeImageTextureManager {
    fn default() -> Self {
        Self {
            current_texture: None,
            base: ImageTextureManager::default(),
        }
    }
}

impl RuntimeImageTextureManager {
}

/// PAA转换器
pub struct PaaConverter;

impl PaaConverter {

    /// 将图片文件转换为PAA格式（带选项和裁剪）
    pub fn convert_image_to_paa_with_crop<P: AsRef<Path>>(
        input_path: P, 
        output_path: P, 
        options: PaaOptions,
        crop_selection: Option<&CropSelection>
    ) -> Result<()> {
        let input_path = input_path.as_ref();
        let output_path = output_path.as_ref();

        info!("开始转换图片到PAA: {:?} -> {:?}", input_path, output_path);

        // 检查输入文件是否存在
        if !input_path.exists() {
            return Err(anyhow::anyhow!("输入文件不存在: {:?}", input_path));
        }

        // 加载图片
        let img = image::open(input_path)
            .with_context(|| format!("无法加载图片: {:?}", input_path))?;

        // 处理图片（裁剪、调整尺寸等）
        let processed_img = if let Some(crop) = crop_selection {
            Self::crop_and_resize_image(img, crop, &options)?
        } else {
            Self::process_image(img, &options)?
        };

        // 转换为PAA格式
        let paa_data = Self::image_to_paa(&processed_img)?;

        // 写入PAA文件
        std::fs::write(output_path, &paa_data)
            .with_context(|| format!("无法写入PAA文件: {:?}", output_path))?;

        info!("PAA转换完成: {:?}", output_path);
        Ok(())
    }

    /// 裁剪图片并调整到2的次方尺寸
    fn crop_and_resize_image(img: DynamicImage, crop: &CropSelection, options: &PaaOptions) -> Result<RgbaImage> {
        let (original_width, original_height) = img.dimensions();
        
        // 获取裁剪区域的像素坐标
        let (crop_x, crop_y, crop_width, crop_height) = crop.get_pixel_coords(original_width, original_height);
        
        info!("裁剪区域: ({}, {}) - {}x{}", crop_x, crop_y, crop_width, crop_height);
        
        // 裁剪图片
        let cropped_img = imageops::crop_imm(&img, crop_x, crop_y, crop_width, crop_height).to_image();
        
        // 确定目标尺寸
        let target_size = if let Some(size) = options.target_size {
            size
        } else {
            // 自动选择最接近的2的次方
            let max_dim = crop_width.max(crop_height);
            Self::next_power_of_two(max_dim)
        };
        
        info!("目标尺寸: {}x{}", target_size, target_size);
        
        // 插值调整到目标尺寸
        let resized_img = if options.center_crop {
            // 居中裁剪到正方形
            let min_dim = crop_width.min(crop_height);
            let center_x = crop_width / 2;
            let center_y = crop_height / 2;
            let half_size = min_dim / 2;
            
            let square_crop = imageops::crop_imm(
                &cropped_img,
                center_x.saturating_sub(half_size),
                center_y.saturating_sub(half_size),
                min_dim,
                min_dim
            ).to_image();
            
            imageops::resize(&square_crop, target_size, target_size, imageops::FilterType::Lanczos3)
        } else {
            // 直接调整尺寸
            imageops::resize(&cropped_img, target_size, target_size, imageops::FilterType::Lanczos3)
        };
        
        Ok(resized_img)
    }

    /// 处理图片（裁剪、调整尺寸等）
    fn process_image(img: DynamicImage, options: &PaaOptions) -> Result<RgbaImage> {
        let mut rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        if options.crop_to_power_of_two {
            let target_size = options.target_size.unwrap_or_else(|| {
                // 自动选择最接近的2的次方尺寸
                let max_dim = width.max(height);
                Self::next_power_of_two(max_dim)
            });

            // 如果当前尺寸不是目标尺寸，进行裁剪或缩放
            if width != target_size || height != target_size {
                rgba_img = Self::resize_to_power_of_two(rgba_img, target_size, options.center_crop)?;
            }
        }

        Ok(rgba_img)
    }

    /// 调整图片到2的次方尺寸
    fn resize_to_power_of_two(
        img: RgbaImage, 
        target_size: u32, 
        center_crop: bool
    ) -> Result<RgbaImage> {
        let (width, height) = img.dimensions();
        
        if width == target_size && height == target_size {
            return Ok(img);
        }

        if center_crop {
            // 居中裁剪
            let crop_size = width.min(height).min(target_size);
            let start_x = (width - crop_size) / 2;
            let start_y = (height - crop_size) / 2;
            
            let cropped = image::imageops::crop_imm(&img, start_x, start_y, crop_size, crop_size).to_image();
            
            if crop_size == target_size {
                Ok(cropped)
            } else {
                // 缩放到目标尺寸
                Ok(image::imageops::resize(&cropped, target_size, target_size, image::imageops::FilterType::Lanczos3))
            }
        } else {
            // 直接缩放到目标尺寸
            Ok(image::imageops::resize(&img, target_size, target_size, image::imageops::FilterType::Lanczos3))
        }
    }

    /// 计算下一个2的次方
    pub fn next_power_of_two(n: u32) -> u32 {
        if n <= 1 {
            return 1;
        }
        
        let mut power = 1;
        while power < n {
            power <<= 1;
        }
        power
    }

    /// 将ImageData转换为PAA字节数据
    fn image_to_paa(img: &RgbaImage) -> Result<Vec<u8>> {
        let (width, height) = img.dimensions();
        
        // PAA文件头结构
        let mut paa_data = Vec::new();
        
        // PAA文件头 (基于Arma 3 PAA格式规范)
        // 文件头大小: 16字节
        paa_data.extend_from_slice(&(16u32).to_le_bytes()); // 头大小
        paa_data.extend_from_slice(&(width as u32).to_le_bytes()); // 宽度
        paa_data.extend_from_slice(&(height as u32).to_le_bytes()); // 高度
        paa_data.extend_from_slice(&(1u32).to_le_bytes()); // 格式标识
        
        // 添加像素数据
        for pixel in img.pixels() {
            // PAA使用BGRA格式
            paa_data.push(pixel[2]); // B
            paa_data.push(pixel[1]); // G
            paa_data.push(pixel[0]); // R
            paa_data.push(pixel[3]); // A
        }

        debug!("生成PAA数据: {}x{}, {}字节", width, height, paa_data.len());
        Ok(paa_data)
    }

}

