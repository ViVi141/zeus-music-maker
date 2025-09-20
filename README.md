# 🎵 宙斯音乐制作器

<div align="center">

![Version](https://img.shields.io/badge/version-v1.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

**专为 Arma 3 设计的音乐模组制作工具**

[快速开始](#-快速开始) • [功能特性](#-功能特性) • [使用指南](#-使用指南) • [构建说明](#-构建说明)

</div>

---

## 📖 项目简介

宙斯音乐制作器是一个功能强大的 Arma 3 音乐模组制作工具，支持音频解密、PAA图片转换和完整的模组生成功能。

### ✨ 核心功能
- 🎵 **音频解密**: 支持酷狗KGM和网易云NCM格式
- 🖼️ **PAA转换**: 智能图片格式转换
- 🎮 **模组生成**: 完整的Arma 3音乐模组制作
- ⚡ **多线程处理**: 后台并行处理，UI保持响应
- 🎨 **现代化界面**: 基于egui的跨平台GUI

## 🚀 快速开始

### 系统要求
- **操作系统**: Windows 10/11 (64位)
- **内存**: 4GB RAM
- **存储**: 100MB 可用空间

### 安装使用

#### 方式一：直接运行（推荐）
1. 下载 `zeus-music-maker.exe`
2. 双击运行即可使用

#### 方式二：从源码构建
```bash
# 克隆项目
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker

# 构建项目
cargo build --release

# 运行程序
cargo run --release
```

## ✨ 功能特性

### 🎵 音频解密
- **酷狗音乐**: 支持 `.kgm` 格式解密
- **网易云音乐**: 支持 `.ncm` 格式解密
- **自动识别**: 智能检测文件类型
- **批量处理**: 同时处理多个文件
- **实时进度**: 详细的进度显示和状态更新

### 🖼️ PAA图片转换
- **多格式支持**: PNG, JPG, BMP, TGA, TIFF, WEBP
- **智能裁剪**: 居中裁剪和自定义裁剪
- **尺寸优化**: 自动调整到2的次方尺寸
- **批量转换**: 多线程并行处理

### 🎮 Arma 3 模组制作
- **自动配置**: 生成 `config.cpp` 和 `mod.cpp`
- **音轨管理**: 可视化音轨列表
- **模组信息**: 自定义名称、作者、版本
- **一键导出**: 生成完整的模组结构

## 📋 使用指南

### 音频解密
1. 点击 **"工具"** → **"音频解密"**
2. 选择加密的音频文件（.kgm 或 .ncm）
3. 选择输出目录
4. 点击 **"开始解密"**
5. 查看进度条和结果

### PAA转换
1. 点击 **"工具"** → **"转换图片为PAA"**
2. 选择图片文件
3. 配置转换选项：
   - 目标尺寸（256x256, 512x512等）
   - 裁剪方式（居中裁剪或保持比例）
4. 点击 **"开始转换"**

### 创建音乐模组
1. 点击 **"文件"** → **"新建项目"**
2. 填写模组信息（名称、作者、版本）
3. 点击 **"添加OGG歌曲"** 添加音频文件
4. 配置音轨信息（名称、音量、描述）
5. 点击 **"导出"** → **"导出模组"**
6. 选择导出目录完成

## 🛠️ 构建说明

### 开发环境
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker
```

### 构建命令
```bash
# 调试构建
cargo build

# 发布构建
cargo build --release

# 运行程序
cargo run --release
```

### 单文件打包
```bash
# Windows
build_release.bat

# Linux/macOS
./build_release.sh
```

## 📁 项目结构

```
zeus-music-maker/
├── src/                    # 源代码
│   ├── main.rs            # 程序入口
│   ├── app.rs             # 应用主逻辑
│   ├── audio_decrypt.rs   # 音频解密
│   ├── paa_converter.rs   # PAA转换
│   ├── threading.rs       # 多线程处理
│   ├── ui.rs              # 用户界面
│   ├── models.rs          # 数据模型
│   └── embedded.rs        # 嵌入资源
├── templates/             # 配置文件模板
├── assets/               # 资源文件
├── lib/                  # 外部库
├── build.rs              # 构建脚本
└── Cargo.toml           # 项目配置
```

## 🔧 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 编程语言 | Rust | 1.70+ |
| GUI框架 | egui | 0.26.2 |
| 音频处理 | symphonia | 0.5.4 |
| 图片处理 | image | 0.24.9 |
| 多线程 | crossbeam-channel | 0.5 |
| 模板引擎 | handlebars | 5.1.2 |

## 🚧 开发状态

### ✅ 已完成
- [x] 音频解密功能（酷狗KGM、网易云NCM）
- [x] PAA图片转换
- [x] Arma 3模组生成
- [x] 多线程处理
- [x] 现代化GUI界面
- [x] 单文件打包

### 🔄 进行中
- [ ] 性能优化
- [ ] 错误处理改进
- [ ] 用户体验优化

### 📋 计划中
- [ ] 更多音频格式支持
- [ ] 批量处理优化
- [ ] 插件系统

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献方式
1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

## 🙏 致谢

感谢以下开源项目：
- [kugou-kgm-decoder](https://github.com/ghtz08/kugou-kgm-decoder) - 酷狗音乐解密
- [ncmdump](https://github.com/taurusxin/ncmdump) - 网易云音乐解密
- [Zeus-Music-Mod-Generator](https://github.com/bijx/Zeus-Music-Mod-Generator) - 原始Java版本

## 📞 联系方式

**作者**: ViVi141  
**邮箱**: 747384120@qq.com  
**项目地址**: [https://github.com/ViVi141/zeus-music-maker](https://github.com/ViVi141/zeus-music-maker)

---

<div align="center">

**版本**: v1.0  
**状态**: 稳定发布 🎉

*让音乐模组制作变得简单而专业*

</div>