# 🎵 宙斯音乐制作器

<div align="center">

![Version](https://img.shields.io/badge/version-v2.1.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-yellow.svg)

**专为 Arma 3 设计的专业音乐/视频模组制作工具**

*基于 Rust 构建的高性能多媒体处理平台*

[功能特性](#-功能特性) • [快速开始](#-快速开始) • [使用指南](#-使用指南) • [技术栈](#-技术栈) • [贡献](#-贡献)

</div>

---

## 📖 简介

宙斯音乐制作器是一个功能强大的 Arma 3 模组制作工具，集成了音频解密、格式转换、PAA图片处理和完整的模组生成功能。无论是音乐模组还是视频模组，都能轻松制作出专业的 Arma 3 模组。

### 主要特点

- 🎵 **音频解密**: 支持酷狗KGM和网易云NCM格式解密
- 🎼 **智能转换**: 基于FFmpeg的多格式音频/视频转换（自动下载FFmpeg）
- 🎬 **视频支持**: 支持视频模组制作，智能分片并行处理
- 🖼️ **PAA转换**: 智能图片格式转换，支持Arma 3专用PAA格式
- 🎮 **模组生成**: 完整的Arma 3音乐/视频模组制作
- 🌍 **多语言支持**: 通过Stringtable本地化支持中文/日语/韩语/俄语等多语言曲目名
- ⚡ **高性能**: 多线程处理、并行转换、分片处理
- 🚀 **智能管理**: 自动检测重复文件，避免重复处理
- 🌐 **多镜像源**: 支持多个FFmpeg下载镜像，提升下载体验
- 🎨 **现代界面**: 基于egui的响应式GUI界面
- 📊 **实时监控**: 详细的进度显示和性能统计

---

## ✨ 功能特性

### 核心功能

- **音频解密**
  - 支持酷狗KGM格式解密
  - 支持网易云NCM格式解密（Windows平台）
  - 自动检测输出格式
  - 文件名自动转换为ASCII安全格式

- **音频转换**
  - 支持多种输入格式：MP3, WAV, FLAC, M4A, OGG, AAC等
  - 输出为Arma 3标准OGG格式
  - 并行转换优化，提升处理速度
  - 支持多语言文件名处理

- **视频转换**
  - 支持多种输入格式：MP4, AVI, MOV, MKV, WMV, FLV, WEBM, M4V, 3GP等
  - 输出为Arma 3标准OGV格式
  - 大文件自动分片并行处理
  - 智能质量调整

- **PAA转换**
  - 支持PNG, JPG, JPEG, BMP, TGA格式
  - 输出为Arma 3专用PAA格式
  - 自动裁剪到2的次方尺寸
  - 居中裁剪处理

- **模组生成**
  - 完整的Arma 3音乐模组制作
  - 完整的Arma 3视频模组制作
  - 自动生成模组配置文件
  - 智能文件管理
  - **Stringtable多语言本地化**
    - 🔹 自动生成 `stringtable.xml` 配置文件
    - 🔹 智能语言检测：自动识别中文/日语/韩语/俄语等语言
    - 🔹 全面支持 Arma 3 的 13 种官方语言
    - 🔹 原生显示：玩家游戏语言与曲目语言匹配时显示原文
    - 🔹 友好回退：不匹配时自动使用拼音或英文回退

### 高级特性

- **多线程处理**: 基于crossbeam-channel的高效任务调度
- **并行转换**: 支持多文件同时处理，速度提升2-4倍
- **分片处理**: 大文件自动分片并行处理，避免内存溢出
- **智能缓存**: 文件路径缓存，避免重复处理
- **文件名安全**: 自动处理多语言文件名，过滤Windows保留字符
- **冲突处理**: 自动检测和处理输出文件冲突
- **FFmpeg管理**: 自动下载、安装、配置FFmpeg
- **格式检测**: 智能识别文件格式，自动选择最佳处理方式

---

## 🚀 快速开始

### 系统要求

#### 最低要求
- **操作系统**: Windows 10 (64位) 版本 1607 或更高
- **处理器**: x64 (AMD64/Intel 64) 架构
- **内存**: 4GB RAM
- **存储空间**: 200MB 可用空间
- **网络**: 首次运行需要网络连接（下载FFmpeg）

#### 推荐配置
- **操作系统**: Windows 11 (64位)
- **内存**: 8GB RAM 或更多
- **处理器**: 多核心处理器（4核或更多）
- **存储空间**: 500MB 或更多可用空间

> 📋 **详细系统要求**: 查看 [系统要求.md](系统要求.md) 获取完整的系统要求、兼容性信息和故障排除指南

### 安装

#### 方式一：直接使用（推荐）

1. 从 [Releases](https://github.com/ViVi141/zeus-music-maker/releases) 下载最新版本的 `zeus-music-maker.exe`
2. 双击运行即可使用
3. 首次启动会显示新用户指导

#### 方式二：从源码构建

```bash
# 克隆仓库
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker

# 构建项目
cargo build --release

# 或使用构建脚本（Windows）
build_portable.bat
```

构建完成后，可执行文件位于 `target/release/zeus-music-maker.exe`

---

## 📋 使用指南

> 📖 **详细教程**: 查看 [模组制作完整指南.md](模组制作完整指南.md) 获取从零到完成模组的详细步骤

### 快速开始

#### 音乐模组制作

1. **启动程序**: 双击运行 `zeus-music-maker.exe`
2. **选择模组类型**: 在工具栏选择"音乐模组"
3. **添加音频文件**: 点击底部"添加音频文件"按钮，选择要转换的音频文件
4. **配置项目**: 点击"文件" → "项目设置"修改模组信息（名称、作者等）
5. **导出模组**: 点击"导出" → "导出模组"生成Arma 3模组

#### 视频模组制作

1. **启动程序**: 双击运行 `zeus-music-maker.exe`
2. **选择模组类型**: 在工具栏选择"视频模组"
3. **添加视频文件**: 点击底部"添加视频文件"按钮，选择要转换的视频文件
4. **配置项目**: 点击"文件" → "项目设置"修改模组信息
5. **导出模组**: 点击"导出" → "导出模组"生成Arma 3模组

### 工具功能

#### 音频解密

- **位置**: 工具 → 音频解密
- **支持格式**: 酷狗KGM、网易云NCM
- **使用方法**: 
  1. 选择要解密的文件
  2. 选择输出目录
  3. 点击"开始解密"
- **输出格式**: 自动检测并输出为原始格式（MP3/FLAC等）
- **核心算法**: 
  - 酷狗KGM解密算法来自 [kugou-kgm-decoder](https://github.com/ghtz08/kugou-kgm-decoder)
  - 网易云NCM解密使用 [ncmdump](https://github.com/taurusxin/ncmdump) 动态库

#### 音频转换

- **输入格式**: MP3, WAV, FLAC, M4A, OGG, AAC等
- **输出格式**: OGG (Arma 3标准格式)
- **特性**: 
  - 自动下载FFmpeg（首次使用）
  - 并行转换优化
  - 支持多语言文件名

#### 视频转换

- **输入格式**: MP4, AVI, MOV, MKV, WMV, FLV, WEBM, M4V, 3GP, OGV
- **输出格式**: OGV (Arma 3标准格式)
- **特性**: 
  - 分片并行处理（大文件）
  - 智能质量调整
  - 自动处理特殊字符

#### PAA转换

- **输入格式**: PNG, JPG, JPEG, BMP, TGA
- **输出格式**: PAA (Arma 3专用格式)
- **特性**: 
  - 自动裁剪到2的次方尺寸
  - 居中裁剪处理
  - 自动转换为ASCII安全文件名

---

## 🎮 支持的格式

### 音频格式

| 类型 | 输入格式 | 输出格式 | 说明 |
|------|----------|----------|------|
| 标准音频 | MP3, WAV, FLAC, M4A, OGG, AAC | OGG | Arma 3标准格式 |
| 加密音频 | KGM (酷狗), NCM (网易云) | 原始格式 | 自动解密 |

### 视频格式

| 类型 | 输入格式 | 输出格式 | 说明 |
|------|----------|----------|------|
| 标准视频 | MP4, AVI, MOV, MKV, WMV, FLV, WEBM, M4V, 3GP | OGV | Arma 3标准格式 |
| 特殊处理 | 大文件自动分片 | OGV | 分片并行处理 |

### 图片格式

| 类型 | 输入格式 | 输出格式 | 说明 |
|------|----------|----------|------|
| 标准图片 | PNG, JPG, JPEG, BMP, TGA | PAA | Arma 3专用格式 |
| 智能处理 | 任意尺寸 | 2的次方尺寸 | 自动裁剪和缩放 |

---

## 🔧 技术栈

### 核心技术

- **编程语言**: Rust 1.70+ (高性能、内存安全)
- **GUI框架**: egui 0.27.2 (现代、响应式界面)
- **音频处理**: symphonia 0.5.4 (专业音频解码)
- **视频处理**: FFmpeg (业界标准视频处理)
- **图片处理**: image 0.25 (高质量图片处理)

### 高级特性

- **多线程**: crossbeam-channel 0.5 (高效任务调度)
- **并行处理**: rayon 1.10 (数据并行计算)
- **异步IO**: tokio 1.40 (异步网络和文件操作)
- **配置管理**: serde_json (结构化配置存储)
- **模板引擎**: handlebars 5.1 (动态模组生成)

### 系统集成

- **Windows支持**: winapi 0.3 (原生Windows API)
- **DLL加载**: libloading 0.8 (动态库加载)
- **字体支持**: 自动加载中文字体
- **资源嵌入**: rust-embed 8.4 (内置资源文件)

### 优化特性

- **依赖精简**: 移除未使用依赖，构建时间减少30%
- **内存优化**: 智能内存管理，减少内存占用
- **启动优化**: 快速启动和关闭，提升用户体验
- **Unicode处理**: 优化的多语言文件名处理，支持中文、日语、俄语、西班牙语
- **文件名安全**: 自动处理Windows保留字符，确保文件名兼容性
- **冲突处理**: 智能检测和处理文件名冲突

---

## 📁 项目结构

本仓库为 **Rust 单栈** 项目（egui GUI），**不包含** `python/` 或其它脚本语言实现。

```
zeus-music-maker/
├── src/                                # Rust 源码
│   ├── main.rs                         # 程序入口
│   ├── app.rs                          # 应用状态与 eframe 集成
│   ├── app/lifecycle.rs                # 启动/关闭生命周期
│   ├── ui.rs                           # 界面与对话框
│   ├── models.rs                       # 项目与轨道数据模型
│   ├── file_ops.rs                     # 导入/导出文件操作
│   ├── audio.rs                        # 音频通用逻辑
│   ├── audio_decrypt.rs                # KGM / NCM 解密
│   ├── audio_converter.rs              # 音频转 OGG
│   ├── video_converter.rs              # 视频转 OGV
│   ├── video_chunk_converter.rs          # 大文件分片
│   ├── video_chunk_parallel_processor.rs # 分片并行处理
│   ├── paa_converter.rs                # 图片转 PAA
│   ├── parallel_converter.rs           # 并行转换引擎
│   ├── threading.rs                    # 后台任务与进度
│   ├── ffmpeg_downloader.rs            # FFmpeg 下载
│   ├── ffmpeg_plugin.rs                # FFmpeg 调用封装
│   ├── templates.rs                    # Handlebars 模组配置生成
│   ├── resource_manager.rs             # 资源路径管理
│   ├── embedded.rs                     # 内嵌模板与资源
│   └── utils/                          # 常量、文件、字符串工具
├── templates/                          # Arma 3 配置模板（Handlebars）
├── assets/                             # logo、KGM 密钥等内置资源
├── bin/                                # ncmdump.exe 等可选二进制
├── lib/                                # NCM 解密等外部 DLL（可选）
├── build.rs                            # Windows 资源/manifest 构建脚本
├── build_portable.bat                  # 便携版发布构建
├── Cargo.toml                          # Rust 依赖与发布配置
├── README.md                           # 本文件
├── 模组制作完整指南.md                  # 用户教程（含完整目录）
├── 系统要求.md
├── 文件名ASCII转换说明.md
└── LICENSE
```

### 文档索引

| 文档 | 用途 |
|------|------|
| [README.md](README.md) | 功能概览、构建与开发 |
| [模组制作完整指南.md](模组制作完整指南.md) | 从安装到发布模组的完整步骤 |
| [多语言支持说明.md](多语言支持说明.md) | Stringtable 本地化与 13 种语言支持详解 |
| [系统要求.md](系统要求.md) | 系统兼容性与排错 |
| [文件名ASCII转换说明.md](文件名ASCII转换说明.md) | 多语言文件名处理说明 |

---

## 📊 性能指标

### 构建性能

- **构建时间**: 约50秒（优化前1分10秒）
- **文件大小**: 约8.2MB便携版单文件
- **依赖数量**: 精简至必要依赖，减少构建复杂度
- **内存使用**: 构建时内存占用减少约20%

### 处理速度

- **音频转换**: 约10-50MB/秒（取决于格式复杂度）
- **视频转换**: 约5-20MB/秒（取决于分辨率和编码）
- **并行处理**: 多文件同时处理，速度提升2-4倍
- **分片处理**: 大文件处理速度提升3-5倍

### 资源使用

- **内存占用**: 基础约50MB，处理时动态增长
- **CPU使用**: 多核心并行，充分利用系统资源
- **磁盘空间**: 临时文件自动清理，最小化占用

---

## 🛠️ 开发指南

### 环境要求

- Rust 1.70 或更高版本
- Windows 10/11 (64位)
- Git（用于克隆仓库）

### 开发环境设置

```bash
# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker

# 运行开发版本
cargo run

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 代码风格

- 遵循Google Rust代码风格
- 使用 `rustfmt` 自动格式化
- 使用 `clippy` 进行代码检查
- 详细的文档注释
- 依赖优化和清理

### 构建发布版本

```bash
# 构建发布版本
cargo build --release

# 或使用构建脚本（Windows）
build_portable.bat
```

---

## 🤝 贡献

我们欢迎所有形式的贡献！

### 贡献方式

1. **报告问题**: 在 [Issues](https://github.com/ViVi141/zeus-music-maker/issues) 中报告bug或提出功能建议
2. **提交代码**: 
   - Fork项目仓库
   - 创建功能分支 (`git checkout -b feature/AmazingFeature`)
   - 提交更改 (`git commit -m 'Add some AmazingFeature'`)
   - 推送到分支 (`git push origin feature/AmazingFeature`)
   - 创建Pull Request

### 贡献指南

- 遵循项目的代码风格
- 添加适当的注释和文档
- 确保代码通过 `cargo clippy` 检查
- 更新相关文档

---

## 📄 许可证

本项目基于 [AGPL-3.0](LICENSE) 许可证开源。

---

## 🙏 致谢

本项目使用了以下开源项目的核心算法和库：

- **[kugou-kgm-decoder](https://github.com/ghtz08/kugou-kgm-decoder)** - 酷狗KGM格式解密的核心算法
- **[ncmdump](https://github.com/taurusxin/ncmdump)** - 网易云NCM格式解密的动态库

感谢这些优秀的开源项目为本项目提供的技术支持！

---

## 🔮 未来功能（不一定达成）

以下功能正在考虑中，但不保证会实现：

- **内嵌 PBO 打包功能**: 集成 PBO 打包功能，无需使用 Arma 3 Tools 的 Addon Builder，实现一键打包
  - 自动配置 Options（`*.ogg` / `*.ogv`）
  - 简化模组制作流程
  - 减少外部工具依赖

> ⚠️ **注意**: 这些功能目前仅为计划，不保证会在未来版本中实现。如有建议或需求，欢迎在 [Issues](https://github.com/ViVi141/zeus-music-maker/issues) 中提出。

---

## 📞 联系方式

- **作者**: ViVi141
- **邮箱**: 747384120@qq.com
- **项目地址**: [https://github.com/ViVi141/zeus-music-maker](https://github.com/ViVi141/zeus-music-maker)
- **问题反馈**: [Issues](https://github.com/ViVi141/zeus-music-maker/issues)

---

## 📚 相关文档

详见上方 [文档索引](#文档索引)。用户教程目录已按章节展开，见 [模组制作完整指南 · 目录](模组制作完整指南.md#-目录)。

---

<div align="center">

**版本**: v2.1.0  
**状态**: 稳定发布 🎉  
**发布日期**: 2026年1月  
**最新更新**: 文件名处理优化、BUG修复、系统要求完善

*让音乐模组制作变得简单而专业*

**⭐ 如果这个项目对你有帮助，请给个Star支持一下！**

Made with ❤️ by ViVi141

</div>
