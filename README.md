# 🎵 宙斯音乐制作器

<div align="center">

![Version](https://img.shields.io/badge/version-v1.2-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

**专为 Arma 3 设计的音乐模组制作工具**

</div>

---

## 📖 简介

宙斯音乐制作器是一个功能强大的 Arma 3 音乐模组制作工具，支持音频解密、PAA图片转换和完整的模组生成功能。

### ✨ 核心功能
- 🎵 **音频解密**: 支持酷狗KGM和网易云NCM格式
- 🖼️ **PAA转换**: 智能图片格式转换
- 🎮 **模组生成**: 完整的Arma 3音乐模组制作
- ⚡ **多线程处理**: 后台并行处理，UI保持响应
- 🚀 **性能优化**: 快速启动和关闭，响应更流畅

## 🚀 快速开始

### 系统要求
- Windows 10/11 (64位)
- 4GB RAM
- 100MB 可用空间

### 安装使用
1. 下载 `zeus-music-maker.exe`
2. 双击运行即可使用

### 构建源码
```bash
git clone https://github.com/ViVi141/zeus-music-maker.git
cd zeus-music-maker
cargo build --release
```

## 📋 使用指南

### 音频解密
1. 点击 **"工具"** → **"音频解密"**
2. 选择加密的音频文件（.kgm 或 .ncm）
3. 选择输出目录
4. 点击 **"开始解密"**

### PAA转换
1. 点击 **"工具"** → **"转换图片为PAA"**
2. 选择图片文件
3. 配置转换选项
4. 点击 **"开始转换"**

### 创建音乐模组
1. 点击 **"文件"** → **"新建项目"**
2. 填写模组信息
3. 添加OGG歌曲文件
4. 配置音轨信息
5. 点击 **"导出"** → **"导出模组"**

## 🔧 技术栈

- **编程语言**: Rust 1.70+
- **GUI框架**: egui 0.26.2
- **音频处理**: symphonia 0.5.4
- **图片处理**: image 0.24.9
- **多线程**: crossbeam-channel 0.5

## 📋 更新日志

### v1.2 (2025年9月20日)
- 🚀 **生产环境优化**: 添加了Windows API级别的强制进程终止
- 🔧 **技术改进**: 创建了专门的production构建配置文件
- 🐛 **问题修复**: 解决了生产环境关闭卡顿问题

### v1.1 (2025年9月20日)
- 🚀 **性能优化**: 优化了启动和关闭速度
- 🔧 **技术改进**: 改进了多线程任务处理机制
- 🐛 **问题修复**: 修复了程序关闭时的延迟问题

### v1.0 (初始版本)
- 音频解密功能（酷狗KGM、网易云NCM）
- PAA图片转换
- Arma 3模组生成
- 多线程处理
- 现代化GUI界面

## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

## 📞 联系方式

**作者**: ViVi141  
**邮箱**: 747384120@qq.com  
**项目地址**: [https://github.com/ViVi141/zeus-music-maker](https://github.com/ViVi141/zeus-music-maker)

---

<div align="center">

**版本**: v1.2  
**状态**: 稳定发布 🎉  
**发布日期**: 2025年9月20日

*让音乐模组制作变得简单而专业*

</div>