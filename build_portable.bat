@echo off
chcp 65001 >nul 2>&1
echo ========================================
echo 宙斯音乐制作器 - 便携版构建脚本 v2.0
echo ========================================
echo.

echo [1/6] 清理之前的构建...
if exist target rmdir /s /q target 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✓ 清理完成
) else (
    echo ✓ 无需清理
)

echo.
echo [2/6] 检查必要文件...
if not exist "favicon.ico" (
    echo ✗ 未找到图标文件 favicon.ico！
    pause
    exit /b 1
)
if not exist "app.manifest" (
    echo ✗ 未找到应用程序清单文件 app.manifest！
    pause
    exit /b 1
)
echo ✓ 必要文件检查完成

echo.
echo [3/6] 更新依赖...
cargo update
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 依赖更新失败！
    pause
    exit /b 1
)
echo ✓ 依赖更新完成

echo.
echo [4/6] 检查代码...
cargo check
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 代码检查失败！
    pause
    exit /b 1
)
echo ✓ 代码检查通过

echo.
echo [5/6] 构建便携版发布版本...
echo 这可能需要几分钟时间，请耐心等待...
echo 正在应用便携版优化设置...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 构建失败！
    pause
    exit /b 1
)

echo.
echo [6/6] 创建便携版发布包...
set RELEASE_DIR=release_portable
set EXE_NAME=zeus-music-maker.exe

if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

echo 复制可执行文件...
copy "target\release\%EXE_NAME%" "%RELEASE_DIR%\%EXE_NAME%"
if %ERRORLEVEL% NEQ 0 (
    echo ✗ 复制可执行文件失败！
    pause
    exit /b 1
)

echo 复制必要资源文件...
if exist "templates" (
    xcopy "templates" "%RELEASE_DIR%\templates\" /E /I /Q
)
if exist "lib" (
    xcopy "lib" "%RELEASE_DIR%\lib\" /E /I /Q
)
if exist "assets" (
    xcopy "assets" "%RELEASE_DIR%\assets\" /E /I /Q
)

echo 创建便携版说明文件...
(
echo Zeus Music Maker - Portable Edition
echo ===================================
echo.
echo This is a portable application that requires no installation.
echo.
echo Usage:
echo 1. Double-click zeus-music-maker.exe to run
echo 2. Configuration files will be created automatically on first run
echo 3. All settings and project files are saved in the program directory
echo.
echo Notes:
echo - Ensure the program has read/write permissions
echo - It is recommended to run the program from a non-system drive
echo - To uninstall, simply delete the entire folder
echo.
echo Version: 2.0.0
echo Author: ViVi141
echo License: CC-BY-SA-4.0
) > "%RELEASE_DIR%\README.txt"

echo.
echo ========================================
echo ✓ 便携版构建成功！
echo 📁 发布目录: %RELEASE_DIR%\
echo 📊 可执行文件大小: 
for %%I in ("%RELEASE_DIR%\%EXE_NAME%") do echo    %%~zI 字节
echo ========================================
echo.
echo 便携版已准备就绪，可以分发给用户使用！
echo.
echo 按任意键退出...
pause >nul
