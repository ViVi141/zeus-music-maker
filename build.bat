@echo off
chcp 65001 >nul 2>&1
echo ========================================
echo 宙斯音乐制作器 - 构建脚本 v2.0
echo ========================================
echo.

echo [1/5] 清理之前的构建...
if exist target rmdir /s /q target 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✓ 清理完成
) else (
    echo ✓ 无需清理
)

echo.
echo [2/5] 更新依赖...
cargo update
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 依赖更新失败！
    pause
    exit /b 1
)
echo ✓ 依赖更新完成

echo.
echo [3/5] 检查代码...
cargo check
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 代码检查失败！
    pause
    exit /b 1
)
echo ✓ 代码检查通过

echo.
echo [4/5] 构建发布版本...
echo 这可能需要几分钟时间，请耐心等待...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 构建失败！
    pause
    exit /b 1
)

echo.
echo [5/5] 构建完成！
echo ========================================
echo ✓ 构建成功！
echo 📁 可执行文件位置: target\release\zeus-music-maker.exe
echo 📊 文件大小: 
for %%I in (target\release\zeus-music-maker.exe) do echo    %%~zI 字节
echo ========================================
echo.
echo 按任意键退出...
pause >nul