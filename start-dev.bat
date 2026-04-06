@echo off
echo ========================================
echo   Kiro One - 启动脚本
echo ========================================
echo.
echo 正在启动开发模式...
echo.
cd /d %~dp0
npx tauri dev
pause
