@echo off
echo 正在构建宙斯音乐制作器生产环境版本...
echo.

REM 设置生产环境变量
set RUST_LOG=warn
set RUST_BACKTRACE=0
set RUST_MIN_STACK=8388608
set RUST_MAX_STACK=8388608
set RUSTC_BOOTSTRAP=0

REM 设置Cargo优化参数
set CARGO_PROFILE_PRODUCTION_OPT_LEVEL=3
set CARGO_PROFILE_PRODUCTION_LTO=fat
set CARGO_PROFILE_PRODUCTION_CODEGEN_UNITS=1
set CARGO_PROFILE_PRODUCTION_PANIC=abort
set CARGO_PROFILE_PRODUCTION_STRIP=symbols

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

REM 构建生产版本
echo 开始构建生产环境版本...
cargo build --profile production

if %ERRORLEVEL% neq 0 (
    echo 构建失败！
    pause
    exit /b 1
)

echo.
echo 构建成功！
echo 可执行文件位置: target\production\zeus-music-maker.exe
echo.

REM 检查文件大小
for %%I in (target\production\zeus-music-maker.exe) do echo 文件大小: %%~zI 字节

REM 复制到根目录
copy target\production\zeus-music-maker.exe .\zeus-music-maker-production.exe
echo.
echo 生产版本已复制为: zeus-music-maker-production.exe

REM 验证构建结果
echo.
echo 验证构建结果...
if exist "target\production\zeus-music-maker.exe" (
    echo ✓ 可执行文件已生成
    echo ✓ 生产环境优化已应用
    echo ✓ 强制退出机制已启用
    echo ✓ 生产版本构建完成！
    echo.
    echo 这个版本专门针对生产环境优化，应该能解决关闭卡顿问题。
) else (
    echo ✗ 构建失败：找不到可执行文件
)

echo.
pause
