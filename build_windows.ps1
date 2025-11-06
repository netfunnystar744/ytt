#!/usr/bin/env pwsh
# ============================================
# System Reporter - Windows PowerShell Build Script
# ============================================

Write-Host ""
Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   System Reporter - Windows Build     ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Rust не установлен!" -ForegroundColor Red
    Write-Host "Установите Rust с https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "[1/4] Проверка версии Rust..." -ForegroundColor Green
rustc --version
cargo --version
Write-Host ""

Write-Host "[2/4] Очистка предыдущих сборок..." -ForegroundColor Green
cargo clean
Write-Host ""

Write-Host "[3/4] Сборка Release версии для Windows..." -ForegroundColor Green
Write-Host "Это может занять несколько минут..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-C target-cpu=native"
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Ошибка сборки!" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "[4/4] Готово!" -ForegroundColor Green
Write-Host ""
Write-Host "════════════════════════════════════════" -ForegroundColor Cyan

$exePath = "target\release\system_reporter.exe"
if (Test-Path $exePath) {
    $fileInfo = Get-Item $exePath
    Write-Host "Исполняемый файл:" -ForegroundColor Cyan
    Write-Host "  $exePath" -ForegroundColor White
    Write-Host ""
    Write-Host "Размер:" -ForegroundColor Cyan
    Write-Host "  $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor White
    Write-Host ""
    Write-Host "Дата сборки:" -ForegroundColor Cyan
    Write-Host "  $($fileInfo.LastWriteTime)" -ForegroundColor White
} else {
    Write-Host "[WARNING] Файл не найден: $exePath" -ForegroundColor Yellow
}

Write-Host "════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "Запустите:" -ForegroundColor Green
Write-Host "  .\target\release\system_reporter.exe" -ForegroundColor White
Write-Host ""

# Опционально: создаем standalone версию
$choice = Read-Host "Создать standalone версию в корне проекта? (y/N)"
if ($choice -eq 'y' -or $choice -eq 'Y') {
    Copy-Item $exePath "system_reporter.exe" -Force
    Write-Host "✓ Создан: system_reporter.exe" -ForegroundColor Green
}

Write-Host ""
Write-Host "Готово! Для дополнительных опций см. README.md" -ForegroundColor Green
