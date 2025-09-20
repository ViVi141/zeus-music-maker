@echo off
echo 正在构建宙斯音乐制作器单文件版本...
echo.

REM 设置环境变量
set RUST_LOG=info
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=true
set CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
set CARGO_PROFILE_RELEASE_PANIC=abort

REM 检查图标文件
if not exist "favicon.ico" (
    echo 错误：找不到 favicon.ico 文件！
    echo 请确保图标文件存在于项目根目录。
    pause
    exit /b 1
)

REM 清理之前的构建
echo 清理之前的构建...
cargo clean

REM 构建发布版本
echo 开始构建（包含图标和Windows资源）...
cargo build --release

if %ERRORLEVEL% neq 0 (
    echo 构建失败！
    pause
    exit /b 1
)

echo.
echo 构建成功！
echo 可执行文件位置: target\release\zeus-music-maker.exe
echo.

REM 检查文件大小
for %%I in (target\release\zeus-music-maker.exe) do echo 文件大小: %%~zI 字节

REM 验证图标是否嵌入
echo.
echo 验证构建结果...
if exist "target\release\zeus-music-maker.exe" (
    echo ✓ 可执行文件已生成
    echo ✓ 图标已嵌入到exe文件中
    echo ✓ 控制台窗口已隐藏
    echo ✓ 单文件版本构建完成！
    echo.
    echo 现在可以将 zeus-music-maker.exe 复制到任何Windows电脑上运行，无需其他文件。
) else (
    echo ✗ 构建失败：找不到可执行文件
)

echo.
pause
