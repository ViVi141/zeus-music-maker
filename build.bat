@echo off
echo 正在构建宙斯音乐制作器版本...
echo.

REM 设置生产环境变量
set RUST_LOG=warn
set RUST_BACKTRACE=0
set RUST_MIN_STACK=8388608
set RUST_MAX_STACK=8388608
set RUSTC_BOOTSTRAP=0

REM 检查必要文件
if not exist "favicon.ico" (
    echo 错误：找不到 favicon.ico 文件！
    echo 请确保图标文件存在于项目根目录。
    pause
    exit /b 1
)

REM 检查 FFmpeg（可选）
if not exist "ffmpeg\ffmpeg.exe" (
    echo 警告：找不到 FFmpeg 可执行文件！
    echo 音频格式转换功能可能不可用。
    echo 请下载 FFmpeg 并放置到 ffmpeg\ 目录中。
    echo 详情请查看 ffmpeg\README.txt
    echo.
)

REM 清理之前的构建
echo 清理之前的构建...
cargo clean

REM 运行测试
echo 运行测试...
cargo test --quiet
if %ERRORLEVEL% neq 0 (
    echo 测试失败！请修复错误后重试。
    pause
    exit /b 1
)

REM 检查代码质量
echo 检查代码质量...
cargo check --quiet
if %ERRORLEVEL% neq 0 (
    echo 代码检查失败！请修复错误后重试。
    pause
    exit /b 1
)

REM 构建优化版本
echo 开始构建版本...
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

REM 复制到根目录
copy target\release\zeus-music-maker.exe .\zeus-music-maker-optimized.exe
echo.
echo 版本已复制为: zeus-music-maker-optimized.exe

REM 验证构建结果
echo.
echo 验证构建结果...
if exist "target\release\zeus-music-maker.exe" (
    echo ✓ 可执行文件已生成
    echo ✓ 代码质量检查通过
    echo ✓ 测试全部通过
    echo ✓ 版本构建完成！
) else (
    echo ✗ 构建失败：找不到可执行文件
)

echo.
pause
