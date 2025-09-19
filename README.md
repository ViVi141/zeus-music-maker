# 宙斯音乐制作器 (Zeus Music Maker)

<div align="center">

![Version](https://img.shields.io/badge/version-v1.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

**一个专为 Arma 3 设计的专业音乐模组制作工具**

[功能特性](#-功能特性) • [快速开始](#-快速开始) • [使用指南](#-使用指南) • [开发计划](#-开发计划) • [贡献指南](#-贡献指南)

</div>

---

## 📖 项目简介

宙斯音乐制作器是一个功能强大的 Arma 3 音乐模组制作工具，旨在简化音乐模组的创建和管理过程。该工具集成了多种音频解密功能，支持酷狗音乐和网易云音乐的加密文件解密，并提供完整的 Arma 3 模组生成功能。

### 🎯 设计理念

- **简单易用**: 直观的图形界面，无需复杂配置
- **功能完整**: 从音频解密到模组打包，一站式解决方案
- **专业品质**: 专为 Arma 3 社区设计，符合模组制作标准
- **持续更新**: 积极维护，不断添加新功能

## ✨ 功能特性

### 🎵 音频处理
- **多格式解密**: 支持酷狗音乐 KGM 和网易云音乐 NCM 格式
- **自动识别**: 智能检测文件类型，无需手动选择
- **批量处理**: 同时处理多个音频文件
- **格式转换**: 自动转换为 Arma 3 兼容的 OGG 格式

### 🎮 Arma 3 模组制作
- **自动配置**: 智能生成 `config.cpp` 和 `mod.cpp` 文件
- **模组信息**: 支持自定义模组名称、作者、版本等信息
- **音轨管理**: 可视化音轨列表，支持拖拽排序
- **PBO 打包**: 一键生成可发布的 PBO 文件

### 🖼️ 图片处理
- **PAA 转换**: 将常见图片格式转换为 Arma 3 专用的 PAA 格式
- **智能裁剪**: 支持居中裁剪和自定义裁剪区域
- **尺寸优化**: 自动调整到 2 的次方尺寸（256x256, 512x512 等）
- **批量处理**: 同时处理多个图片文件

### 🎨 用户界面
- **现代化设计**: 基于 egui 的跨平台 GUI
- **中文支持**: 完整的中文界面和帮助文档
- **响应式布局**: 自适应窗口大小，支持高分辨率显示
- **实时反馈**: 操作进度和结果实时显示

## 🚀 快速开始

### 系统要求

| 项目 | 要求 |
|------|------|
| 操作系统 | Windows 10/11 (64位) |
| 内存 | 至少 4GB RAM |
| 存储空间 | 100MB 可用空间 |
| 显卡 | 支持 OpenGL 3.3+ |

### 安装方式

#### 方式一：直接下载（推荐）
1. 访问 [Releases](https://github.com/ViVi141/zeus-music-maker/releases) 页面
2. 下载最新版本的 `zeus-music-maker.exe`
3. 双击运行即可使用

#### 方式二：从源码编译
```bash
# 克隆仓库
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker

# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 编译项目
cargo build --release

# 运行程序
cargo run --release
```

## 📋 使用指南

### 1. 音频解密

#### 酷狗音乐解密
1. 点击菜单栏 **"工具"** → **"音频解密"**
2. 选择 `.kgm` 格式的酷狗音乐文件
3. 选择输出目录
4. 点击 **"开始解密"**

#### 网易云音乐解密
1. 在音频解密界面选择 `.ncm` 格式文件
2. 程序会自动识别并调用相应的解密算法
3. 解密完成后文件会保存为 `.mp3` 格式

### 2. 创建音乐模组

#### 新建项目
1. 点击 **"文件"** → **"新建项目"**
2. 填写模组信息：
   - 模组名称
   - 作者信息
   - 版本号
   - 描述信息

#### 添加音频文件
1. 点击 **"添加音频文件"** 按钮
2. 选择 OGG 格式的音频文件
3. 程序会自动提取音频信息并添加到音轨列表

#### 配置音轨信息
- **音轨名称**: 在游戏中显示的名称
- **音量**: 音轨播放音量（0-100）
- **时长**: 自动检测或手动设置
- **描述**: 音轨的详细描述

#### 导出模组
1. 点击 **"导出"** → **"导出模组"**
2. 选择导出目录
3. 程序会自动生成完整的模组结构
4. 生成的文件可以直接打包为 PBO

### 3. PAA 图片转换

#### 基本转换
1. 点击 **"工具"** → **"PAA 转换器"**
2. 选择要转换的图片文件（支持 PNG, JPG, BMP 等）
3. 配置转换选项：
   - 目标尺寸
   - 裁剪方式
   - 质量设置

#### 高级选项
- **居中裁剪**: 自动裁剪到指定尺寸
- **保持比例**: 保持原始宽高比
- **2的次方**: 自动调整到 2 的次方尺寸

## 🛠️ 技术架构

### 核心技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 编程语言 | Rust | 1.70+ |
| GUI 框架 | egui | 0.26.2 |
| 音频处理 | symphonia | 0.5.4 |
| 图片处理 | image | 0.24.9 |
| 模板引擎 | handlebars | 5.1.2 |
| 文件操作 | rfd, fs_extra | 最新 |

### 项目结构

```
zeus-music-maker/
├── assets/                 # 资源文件
│   ├── fonts/             # 字体文件
│   ├── kugou_key.xz       # 酷狗解密密钥
│   ├── logo.paa           # 默认模组Logo
│   ├── zeus_music_maker.png  # 应用图标
│   └── zeus_steam_logo.png   # Steam Logo
├── lib/                   # 外部库
│   └── libncmdump.dll     # 网易云解密库
├── src/                   # 源代码
│   ├── app.rs             # 应用主逻辑
│   ├── audio_decrypt.rs   # 音频解密模块
│   ├── paa_converter.rs   # PAA 转换模块
│   ├── ui.rs              # 用户界面
│   ├── models.rs          # 数据模型
│   ├── file_ops.rs        # 文件操作
│   └── templates.rs       # 模板处理
├── templates/             # 配置文件模板
│   ├── config.txt         # 主配置文件模板
│   ├── mod.txt            # 模组描述模板
│   └── FileListWithMusicTracks.txt  # 音轨列表模板
└── README.md              # 项目文档
```

## 🔧 开发计划

### v2.0 (计划中)
- [ ] **FFmpeg 集成**: 支持更多音频格式转换
- [ ] **OGG 优化**: 音频质量优化和压缩
- [ ] **批量处理**: 改进批量操作的用户体验

### v3.0 (长期目标)
- [ ] **视频模组**: 支持视频文件处理和模组制作

## 🤝 贡献指南

我们欢迎任何形式的贡献！无论是代码、文档、测试还是反馈，都对项目发展非常重要。

### 如何贡献

1. **Fork 项目**: 点击右上角的 Fork 按钮
2. **创建分支**: `git checkout -b feature/AmazingFeature`
3. **提交更改**: `git commit -m 'Add some AmazingFeature'`
4. **推送分支**: `git push origin feature/AmazingFeature`
5. **创建 PR**: 在 GitHub 上创建 Pull Request

### 贡献类型

- 🐛 **Bug 修复**: 报告和修复程序错误
- ✨ **新功能**: 添加新的功能特性
- 📚 **文档**: 改进文档和帮助信息
- 🎨 **界面**: 优化用户界面和体验
- ⚡ **性能**: 提升程序性能和稳定性

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

```
MIT License

Copyright (c) 2025 ViVi141

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 🙏 致谢

本项目借鉴或直接引用了以下优秀的开源项目：

- **[kugou-kgm-decoder](https://github.com/ghtz08/kugou-kgm-decoder)** - 酷狗音乐解密功能
- **[ncmdump](https://github.com/taurusxin/ncmdump)** - 网易云音乐解密功能  
- **[Zeus-Music-Mod-Generator](https://github.com/bijx/Zeus-Music-Mod-Generator)** - 原始 Java 版本参考

感谢这些开源项目为我们的开发提供了重要参考和基础！

## 📞 联系我们

<div align="center">

**作者**: ViVi141  
**邮箱**: 747384120@qq.com  
**项目地址**: [https://github.com/ViVi141/zeus-music-maker](https://github.com/ViVi141/zeus-music-maker)

[![GitHub stars](https://img.shields.io/github/stars/ViVi141/zeus-music-maker?style=social)](https://github.com/ViVi141/zeus-music-maker)
[![GitHub forks](https://img.shields.io/github/forks/ViVi141/zeus-music-maker?style=social)](https://github.com/ViVi141/zeus-music-maker)
[![GitHub issues](https://img.shields.io/github/issues/ViVi141/zeus-music-maker)](https://github.com/ViVi141/zeus-music-maker/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/ViVi141/zeus-music-maker)](https://github.com/ViVi141/zeus-music-maker/pulls)

</div>

---

<div align="center">

**最后更新**: 2025年9月20日  
**版本**: v1.0  
**状态**: 积极开发中 🚀

*让音乐模组制作变得简单而专业*

</div>