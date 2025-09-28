# 🎵 宙斯音乐制作器

<div align="center">

![Version](https://img.shields.io/badge/version-v2.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

**专为 Arma 3 设计的专业音乐模组制作工具**

</div>

---

## 📖 简介

宙斯音乐制作器是一个功能强大的 Arma 3 音乐模组制作工具，集成了音频解密、格式转换、PAA图片处理和完整的模组生成功能。无论是音乐模组还是视频模组，都能轻松制作出专业的 Arma 3 模组。

### ✨ 核心功能

- 🎵 **音频解密**: 支持酷狗KGM和网易云NCM格式解密
- 🎼 **音频转换**: 基于FFmpeg的多格式音频转换（自动下载FFmpeg）
- 🎬 **视频支持**: 支持视频模组制作，自动转换为OGV格式
- 🖼️ **PAA转换**: 智能图片格式转换，支持Arma 3专用PAA格式
- 🎮 **模组生成**: 完整的Arma 3音乐/视频模组制作
- ⚡ **多线程处理**: 后台并行处理，UI保持响应
- 🚀 **智能管理**: 自动检测重复文件，避免重复处理
- 🌐 **多镜像源**: 支持多个FFmpeg下载镜像，提升下载体验

## 🚀 快速开始

### 系统要求
- Windows 10/11 (64位)
- 4GB RAM
- 100MB 可用空间

### 安装使用
1. 下载 `zeus-music-maker.exe`
2. 双击运行即可使用
3. 首次启动会显示新用户指导

### 构建源码
```bash
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker
cargo build
```

或者使用构建脚本：
```bash
# Windows
build.bat
```

## 📋 使用指南

### 🎵 音乐模组制作

1. **选择模组类型**: 在工具栏选择"音乐模组"
2. **添加音频文件**: 点击底部"添加音频文件"按钮
3. **配置项目**: 点击"文件" → "项目设置"修改模组信息
4. **导出模组**: 点击"导出" → "导出模组"生成Arma 3模组

### 🎬 视频模组制作

1. **选择模组类型**: 在工具栏选择"视频模组"
2. **添加视频文件**: 点击底部"添加视频文件"按钮
3. **配置项目**: 点击"文件" → "项目设置"修改模组信息
4. **导出模组**: 点击"导出" → "导出模组"生成Arma 3模组

### 🔧 工具功能

#### 音频解密
1. 点击 **"工具"** → **"音频解密"**
2. 选择加密的音频文件（.kgm 或 .ncm）
3. 选择输出目录
4. 点击 **"开始解密"**

#### 音频转换
1. 点击 **"工具"** → **"音频格式转换"**
2. 选择音频文件（支持多选）
3. 选择输出目录
4. 点击 **"开始转换"**
5. 如未安装FFmpeg，软件会自动下载

#### PAA转换
1. 点击 **"工具"** → **"转换图片为PAA"**
2. 选择图片文件（支持多选）
3. 配置转换选项
4. 点击 **"开始转换"**

## 🎯 主要特性

### 智能文件管理
- **重复检测**: 自动检测重复文件，避免重复处理
- **批量操作**: 支持多文件同时处理
- **拖拽支持**: 支持文件拖拽操作

### 自动化处理
- **FFmpeg自动下载**: 首次使用时自动下载和配置FFmpeg
- **多镜像源**: 支持GitHub代理镜像，提升中国用户体验
- **格式检测**: 自动检测文件格式并选择合适的处理方式

### 用户体验
- **新用户指导**: 首次启动显示详细的使用指导
- **响应式界面**: 多线程处理，UI始终保持响应
- **错误处理**: 完善的错误提示和日志记录

## 🔧 技术栈

- **编程语言**: Rust 1.70+
- **GUI框架**: egui 0.26.2
- **音频处理**: symphonia 0.5.4
- **视频处理**: FFmpeg
- **图片处理**: image 0.24.9
- **多线程**: crossbeam-channel 0.5
- **配置管理**: serde_json

## 📁 项目结构

```
zeus-music-maker/
├── src/
│   ├── app.rs              # 主应用程序逻辑
│   ├── main.rs             # 程序入口
│   ├── models.rs           # 数据模型定义
│   ├── ui.rs               # 用户界面
│   ├── audio_decrypt.rs    # 音频解密功能
│   ├── audio_converter.rs  # 音频转换功能
│   ├── video_converter.rs  # 视频转换功能
│   ├── paa_converter.rs    # PAA转换功能
│   ├── ffmpeg_downloader.rs # FFmpeg下载管理
│   └── utils/              # 工具函数
├── templates/              # 模组模板文件
├── assets/                 # 资源文件
└── lib/                    # 外部库文件
```

## 🎮 支持的格式

### 音频格式
- **输入**: MP3, WAV, FLAC, M4A, KGM, NCM
- **输出**: OGG (Arma 3标准格式)

### 视频格式
- **输入**: MP4, AVI, MOV, MKV, WMV, FLV, WEBM, M4V, 3GP, OGV
- **输出**: OGV (Arma 3标准格式)

### 图片格式
- **输入**: PNG, JPG, JPEG, BMP, TGA
- **输出**: PAA (Arma 3专用格式)

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

## 📞 联系方式

**作者**: ViVi141  
**邮箱**: 747384120@qq.com  
**项目地址**: [https://github.com/ViVi141/zeus-music-maker](https://github.com/ViVi141/zeus-music-maker)

---

<div align="center">

**版本**: v2.0  
**状态**: 稳定发布 🎉  
**发布日期**: 2025年9月

*让音乐模组制作变得简单而专业*

</div>