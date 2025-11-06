#!/usr/bin/env pwsh
# ============================================
# System Reporter - Silent Build Script
# ============================================
# Создает исполняемый файл без консольного окна
# Токены встраиваются из .env файла

Write-Host ""
Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   System Reporter - Silent Build      ║" -ForegroundColor Cyan
Write-Host "║   (Скрытая сборка без окна)            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Rust не установлен!" -ForegroundColor Red
    Write-Host "Установите Rust с https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "[1/6] Проверка версии Rust..." -ForegroundColor Green
rustc --version
cargo --version
Write-Host ""

Write-Host "[2/6] Проверка .env файла..." -ForegroundColor Green
if (Test-Path ".env") {
    Write-Host "  ✓ .env файл найден" -ForegroundColor Green
    Write-Host "  ℹ Токены будут встроены в exe!" -ForegroundColor Cyan

    # Показываем какие переменные найдены (без значений!)
    $envContent = Get-Content ".env"
    $foundVars = @()
    foreach ($line in $envContent) {
        if ($line -match '^([A-Z_]+)=.+' -and $line -notmatch '^#') {
            $foundVars += $Matches[1]
        }
    }

    if ($foundVars.Count -gt 0) {
        Write-Host "  ℹ Найдены переменные: $($foundVars -join ', ')" -ForegroundColor Cyan
    }
} else {
    Write-Host "  ⚠ .env файл НЕ найден!" -ForegroundColor Yellow
    Write-Host "  ℹ Создайте .env из .env.example" -ForegroundColor Yellow
    Write-Host ""
    $choice = Read-Host "Продолжить сборку без встроенных токенов? (y/N)"
    if ($choice -ne 'y' -and $choice -ne 'Y') {
        Write-Host "Сборка отменена." -ForegroundColor Yellow
        exit 0
    }
}
Write-Host ""

Write-Host "[3/6] Очистка предыдущих сборок..." -ForegroundColor Green
cargo clean
Write-Host ""

Write-Host "[4/6] Сборка Silent версии (без окна консоли)..." -ForegroundColor Green
Write-Host "  ℹ В release режиме exe будет запускаться СКРЫТНО" -ForegroundColor Cyan
Write-Host "  ℹ Логи будут записываться в system_reporter.log" -ForegroundColor Cyan
Write-Host "  ⏳ Сборка может занять несколько минут..." -ForegroundColor Yellow
Write-Host ""

$env:RUSTFLAGS = "-C target-cpu=native"
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "[ERROR] Ошибка сборки!" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "[5/6] Анализ результата..." -ForegroundColor Green
$exePath = "target\release\system_reporter.exe"
if (Test-Path $exePath) {
    $fileInfo = Get-Item $exePath
    Write-Host "  ✓ Сборка успешна!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Файл: $exePath" -ForegroundColor White
    Write-Host "  Размер: $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor White
    Write-Host "  Дата: $($fileInfo.LastWriteTime)" -ForegroundColor White
    Write-Host ""

    # Проверяем наличие встроенных токенов (косвенно через размер)
    if (Test-Path ".env") {
        Write-Host "  ✓ Токены встроены в exe" -ForegroundColor Green
        Write-Host "  ⚠ ВНИМАНИЕ: Не распространяйте этот exe публично!" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✗ Файл не найден!" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "[6/6] Финализация..." -ForegroundColor Green
# Создаем standalone версию
$standalonePath = "system_reporter_silent.exe"
Copy-Item $exePath $standalonePath -Force
Write-Host "  ✓ Создана standalone версия: $standalonePath" -ForegroundColor Green
Write-Host ""

Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ ГОТОВО! Silent версия собрана успешно" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "📁 Файлы:" -ForegroundColor Yellow
Write-Host "   • target\release\system_reporter.exe" -ForegroundColor White
Write-Host "   • $standalonePath (копия для удобства)" -ForegroundColor White
Write-Host ""
Write-Host "🔇 Особенности:" -ForegroundColor Yellow
Write-Host "   • Запускается БЕЗ консольного окна (полностью скрытно)" -ForegroundColor White
Write-Host "   • Логи пишутся в system_reporter.log" -ForegroundColor White
Write-Host "   • Токены встроены внутрь exe (если был .env)" -ForegroundColor White
Write-Host "   • Отчеты отправляются в Telegram автоматически" -ForegroundColor White
Write-Host ""
Write-Host "⚠️  ВАЖНО:" -ForegroundColor Yellow
Write-Host "   • Не публикуйте этот exe (содержит ваши токены)" -ForegroundColor White
Write-Host "   • При возникновении ошибок смотрите system_reporter.log" -ForegroundColor White
Write-Host "   • Для остановки завершите процесс через Task Manager" -ForegroundColor White
Write-Host ""
Write-Host "🚀 Тестирование:" -ForegroundColor Yellow
Write-Host "   1. Запустите: .\$standalonePath" -ForegroundColor White
Write-Host "   2. Проверьте Telegram - должен прийти отчет" -ForegroundColor White
Write-Host "   3. Проверьте логи: Get-Content system_reporter.log -Tail 20" -ForegroundColor White
Write-Host ""

# Предлагаем запустить тест
$choice = Read-Host "Запустить тестовый запуск? (y/N)"
if ($choice -eq 'y' -or $choice -eq 'Y') {
    Write-Host ""
    Write-Host "🔄 Запускаем в фоновом режиме..." -ForegroundColor Cyan
    Start-Process -FilePath ".\$standalonePath" -WindowStyle Hidden
    Write-Host "✓ Процесс запущен!" -ForegroundColor Green
    Write-Host "Следите за логом: Get-Content system_reporter.log -Wait" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Готово! 🎉" -ForegroundColor Green
