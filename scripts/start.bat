@echo off
REM Z-CPP 启动脚本 (Windows)
REM 用法: start.bat [port]

set PORT=%1
if "%PORT%"=="" set PORT=3000

cd /d "%~dp0"

if not exist workspace mkdir workspace

set ZCPP_MODE=production

echo ========================================
echo   Z-CPP 轻量级 C/C++ IDE
echo   端口: %PORT%
echo   工作目录: %CD%\workspace
echo   浏览器打开: http://localhost:%PORT%
echo ========================================
echo.

.\z-cpp-backend.exe --port %PORT%

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo 程序异常退出，错误码: %ERRORLEVEL%
    pause
)
