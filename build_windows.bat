@echo off
REM ============================================
REM System Reporter - Windows Build Script
REM ============================================

echo.
echo ╔════════════════════════════════════════╗
echo ║   System Reporter - Windows Build     ║
echo ╚════════════════════════════════════════╝
echo.

REM Check if Rust is installed
where rustc >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Rust не установлен!
    echo Установите Rust с https://rustup.rs/
    exit /b 1
)

echo [1/4] Проверка версии Rust...
rustc --version
cargo --version
echo.

echo [2/5] Проверка .env файла...
if exist .env (
    echo   [OK] .env файл найден - токены будут встроены в exe
) else (
    echo   [INFO] .env файл не найден
)
echo.

echo [3/5] Очистка предыдущих сборок...
cargo clean
echo.

echo [4/5] Сборка Release версии для Windows...
echo Это может занять несколько минут...
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Ошибка сборки!
    exit /b 1
)
echo.

echo [5/5] Готово!
echo.
echo ════════════════════════════════════════
echo Исполняемый файл:
echo   target\release\system_reporter.exe
echo.
echo Размер:
dir target\release\system_reporter.exe | find "system_reporter.exe"
echo ════════════════════════════════════════
echo.

echo Запустите: target\release\system_reporter.exe
echo.
echo Примечание:
echo   - Эта версия показывает консольное окно
echo   - Для скрытого запуска см. build_windows_silent.ps1
echo.

pause
