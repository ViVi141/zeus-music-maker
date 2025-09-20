#!/bin/bash

echo "正在构建宙斯音乐制作器单文件版本..."
echo

# 设置环境变量
export RUST_LOG=info
export CARGO_PROFILE_RELEASE_OPT_LEVEL=3
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=abort

# 清理之前的构建
echo "清理之前的构建..."
cargo clean

# 构建发布版本
echo "开始构建..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "构建失败！"
    exit 1
fi

echo
echo "构建成功！"

# 检查操作系统并显示可执行文件路径
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    EXECUTABLE="target/release/zeus-music-maker"
    echo "可执行文件位置: $EXECUTABLE"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    EXECUTABLE="target/release/zeus-music-maker"
    echo "可执行文件位置: $EXECUTABLE"
else
    EXECUTABLE="target/release/zeus-music-maker.exe"
    echo "可执行文件位置: $EXECUTABLE"
fi

echo

# 检查文件大小
if [ -f "$EXECUTABLE" ]; then
    SIZE=$(stat -c%s "$EXECUTABLE" 2>/dev/null || stat -f%z "$EXECUTABLE" 2>/dev/null || echo "未知")
    echo "文件大小: $SIZE 字节"
fi

echo
echo "单文件版本构建完成！"
echo "现在可以将可执行文件复制到任何电脑上运行，无需其他文件。"
echo
