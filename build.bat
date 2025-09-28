@echo off
chcp 65001 >nul
echo 宙斯音乐制作器 - 构建脚本
echo.

echo 清理之前的构建...
if exist target rmdir /s /q target

echo 更新依赖...
cargo update

echo 检查代码...
cargo check

echo 构建发布版本...
cargo build --release

echo.
echo 构建完成！可执行文件位于: target\release\zeus-music-maker.exe
echo.
pause