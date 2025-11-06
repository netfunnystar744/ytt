# System Reporter 🦀

**Профессиональный сборщик системной информации на Rust**

Мощное кроссплатформенное приложение для полного сбора информации о системе с расширенными возможностями для Windows через WMI и опциональной отправкой отчетов в Telegram.

## ✨ Ключевые возможности

- 🖥️ **Полный сбор системной информации** - CPU, RAM, диски, сеть, процессы
- 🪟 **Глубокая интеграция Windows** - WMI для сбора информации о GPU, BIOS, материнской плате
- 🤖 **Telegram интеграция** - автоматическая отправка отчетов в Telegram
- 🔍 **VM/Sandbox детектор** - обнаружение подозрительных процессов и драйверов
- 📊 **TOP процессы** - анализ процессов по использованию CPU и памяти
- 🎯 **Простота использования** - один бинарник, никаких внешних зависимостей
- 🌍 **Кроссплатформенность** - Windows, Linux, macOS

## 📋 Собираемая информация

### 🔧 Модуль 1: Базовая информация системы
- Имя пользователя и хоста
- Операционная система, версия и архитектура
- Время работы системы (uptime) в секундах и читаемом формате

### 💻 Модуль 2: Оборудование
- **Процессор:**
  - Модель и бренд
  - Количество физических и логических ядер
  - Частота работы (MHz)
- **Память (RAM):**
  - Общий объем
  - Используемая память
  - Доступная память
  - Процент использования

### 💾 Модуль 3: Диски
- Список всех дисков с:
  - Именем/буквой
  - Размером
  - Типом (HDD/SSD)
  - Интерфейсом (для Windows)
- Общее и использованное дисковое пространство

### 🌐 Модуль 4: Сетевая информация
- Локальный IP-адрес
- MAC-адреса всех сетевых адаптеров
- Список всех сетевых интерфейсов

### 🪟 Модуль 5: Глубокая информация (Windows + WMI)
- **GPU (Видеокарты):**
  - Название
  - Объем видеопамяти (VRAM)
- **Системная плата:**
  - Производитель компьютера
  - Модель компьютера
- **BIOS:**
  - Производитель
  - Версия
  - Серийный номер
- **Материнская плата:**
  - Производитель
  - Модель
  - Серийный номер

### 📊 Модуль 6: Программное обеспечение
- Общее количество запущенных процессов
- TOP-10 процессов по использованию памяти
- TOP-10 процессов по использованию CPU
- **VM/Sandbox детектор:**
  - Подозрительные процессы (VMware, VirtualBox, QEMU, Xen, Parallels)
  - Подозрительные драйверы (только Windows)

### 📝 Метаинформация
- Дата и время генерации отчета (RFC3339)
- Версия программы

## 🚀 Установка и запуск

### Предварительные требования

1. **Rust** (версия 1.70+):
```bash
# Установка через rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Для Windows: скачайте с https://rustup.rs/
```

2. **(Опционально) Telegram Bot** для отправки отчетов:
   - Создайте бота через [@BotFather](https://t.me/BotFather)
   - Получите токен бота
   - Узнайте свой Chat ID через [@userinfobot](https://t.me/userinfobot)

### Быстрый старт

```bash
# 1. Клонирование репозитория
git clone https://github.com/yourusername/ytt.git
cd ytt

# 2. Сборка release версии
cargo build --release

# 3. Запуск
cargo run --release
```

### Режим с Telegram

Для отправки отчетов в Telegram установите переменные окружения:

**Windows (PowerShell):**
```powershell
$env:TELEGRAM_BOT_TOKEN="123456:ABC-DEF..."
$env:TELEGRAM_CHAT_ID="987654321"
cargo run --release
```

**Windows (CMD):**
```cmd
set TELEGRAM_BOT_TOKEN=123456:ABC-DEF...
set TELEGRAM_CHAT_ID=987654321
cargo run --release
```

**Linux/macOS:**
```bash
export TELEGRAM_BOT_TOKEN="123456:ABC-DEF..."
export TELEGRAM_CHAT_ID="987654321"
cargo run --release
```

### Бинарный файл

После сборки исполняемый файл находится в:
- Windows: `target\release\system_reporter.exe`
- Linux/macOS: `target/release/system_reporter`

Скопируйте его куда угодно и запускайте без зависимостей!

## 📺 Пример вывода

```
╔════════════════════════════════════════╗
║     System Reporter v0.2.0             ║
║     Полный сбор системной информации   ║
╚════════════════════════════════════════╝

✓ Режим Telegram активирован

[1/5] Инициализация системных библиотек...
   → Сбор базовой информации...
   → WMI инициализирован (глубокий сбор для Windows)
   → Сбор информации об оборудовании...
   → Сбор сетевой информации...
   → Анализ процессов...
[2/5] Сериализация данных в JSON...
[3/5] Отчет сохранен: system_report_20250106_143025.json
[4/5] Отправка отчета в Telegram...
[5/5] ✓ Отчет успешно отправлен в Telegram!

╔════════════════════════════════════════╗
║           Статистика отчета             ║
╠════════════════════════════════════════╣
║ Процессов:                         245 ║
║ Дисков:                              3 ║
║ Сетевых интерфейсов:                 2 ║
║ GPU:                                 1 ║
╚════════════════════════════════════════╝
```

## 📊 Пример JSON отчета

```json
{
  "username": "admin",
  "hostname": "DESKTOP-PC",
  "os_name": "Windows",
  "os_version": "Windows 11 Pro",
  "os_architecture": "x86_64",
  "system_uptime_seconds": 145230,
  "system_uptime_readable": "1d 16h 20m",

  "cpu_model": "AMD Ryzen 7 5800X 8-Core Processor",
  "cpu_cores_physical": 8,
  "cpu_cores_logical": 16,
  "cpu_frequency_mhz": 3800,
  "total_memory_gb": 31.93,
  "used_memory_gb": 16.42,
  "available_memory_gb": 15.51,
  "memory_usage_percent": 51.4,

  "disks": [
    {
      "name": "C:\\",
      "size": "931.51 GB",
      "disk_type": "SSD",
      "interface": "NVMe"
    }
  ],

  "local_ip": "192.168.1.100",
  "mac_addresses": {
    "Ethernet": "AA:BB:CC:DD:EE:FF"
  },

  "gpus": [
    {
      "name": "NVIDIA GeForce RTX 3080",
      "vram_mb": 10240
    }
  ],

  "computer_manufacturer": "Gigabyte Technology Co., Ltd.",
  "computer_model": "X570 AORUS ELITE",

  "bios_info": {
    "manufacturer": "American Megatrends Inc.",
    "version": "F36",
    "serial_number": "Default string"
  },

  "motherboard_info": {
    "manufacturer": "Gigabyte Technology Co., Ltd.",
    "model": "X570 AORUS ELITE",
    "serial_number": "Default string"
  },

  "process_count": 245,
  "suspicious_processes": [],
  "suspicious_drivers": [],

  "top_memory_processes": [
    "chrome.exe (PID: 12345, RAM: 1024.50 MB)",
    "rust-analyzer.exe (PID: 23456, RAM: 512.30 MB)"
  ],

  "top_cpu_processes": [
    "cargo.exe (PID: 34567, CPU: 25.3%)",
    "rustc.exe (PID: 45678, CPU: 18.7%)"
  ],

  "report_generated_at": "2025-01-06T14:30:25+00:00",
  "reporter_version": "0.2.0"
}
```

## 🛠️ Архитектура

### Структура проекта

```
ytt/
├── Cargo.toml              # Манифест и зависимости
├── src/
│   └── main.rs             # Вся логика приложения (590 строк)
├── README.md               # Документация
├── .gitignore              # Игнорируемые файлы
└── target/                 # Скомпилированные файлы
```

### Основные зависимости

| Крейт | Версия | Назначение |
|-------|--------|------------|
| `sysinfo` | 0.30 | Кроссплатформенная системная информация |
| `whoami` | 1.5 | Информация о пользователе |
| `wmi` | 0.15 | Windows Management Instrumentation |
| `teloxide` | 0.12 | Telegram Bot API |
| `tokio` | 1.x | Асинхронный runtime |
| `serde` + `serde_json` | 1.0 | Сериализация в JSON |
| `anyhow` | 1.0 | Обработка ошибок |
| `chrono` | 0.4 | Работа с датой и временем |
| `local-ip-address` | 0.6 | Определение локального IP |

### Условная компиляция

Программа использует условную компиляцию для поддержки разных платформ:

- **Windows-специфичный код** (`#[cfg(target_os = "windows")]`):
  - WMI интеграция
  - Сбор информации о GPU, BIOS, материнской плате
  - Поиск подозрительных драйверов

- **Кроссплатформенный код**:
  - Базовая информация о системе
  - Сбор информации о CPU, памяти, дисках
  - Сетевая информация
  - Анализ процессов

## 🔧 Расширение функциона ла

### Добавление новых WMI-запросов (Windows)

```rust
#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_YourClass")]
#[serde(rename_all = "PascalCase")]
struct YourClass {
    property_name: String,
}

#[cfg(target_os = "windows")]
fn get_your_data(wmi_con: &Option<WMIConnection>) -> Vec<String> {
    let data: Vec<YourClass> = get_wmi_data(wmi_con,
        "SELECT PropertyName FROM Win32_YourClass");

    data.iter().map(|d| d.property_name.clone()).collect()
}
```

### Добавление новых модулей сбора

Просто добавьте новую функцию и вызовите её в `collect_system_info()`:

```rust
fn collect_custom_info() -> Vec<String> {
    // Ваша логика
    vec!["data1".to_string(), "data2".to_string()]
}
```

## 🐛 Устранение неполадок

### Ошибки компиляции

```bash
# Обновите Rust
rustup update

# Очистите кэш
cargo clean

# Пересоберите
cargo build --release
```

### WMI не работает (Windows)

1. Проверьте, что служба WMI запущена:
   ```powershell
   Get-Service Winmgmt
   ```

2. Запустите от имени администратора для полного доступа к WMI

3. Если WMI не инициализируется, программа продолжит работу без WMI-данных

### Telegram не отправляет

1. Проверьте токен и Chat ID:
   ```bash
   echo $TELEGRAM_BOT_TOKEN
   echo $TELEGRAM_CHAT_ID
   ```

2. Проверьте интернет-соединение

3. Убедитесь, что бот не заблокирован и вы написали ему `/start`

### Проблемы с правами доступа (Linux/macOS)

Некоторые системные данные требуют sudo:

```bash
sudo ./target/release/system_reporter
```

## 📐 Производительность

- **Размер бинарника:** ~8-12 MB (с зависимостями)
- **Время выполнения:** 2-5 секунд
- **Использование памяти:** ~20-50 MB
- **Поддерживаемые ОС:** Windows 7+, Linux (любой дистрибутив), macOS 10.12+

## 🔐 Безопасность

### Лучшие практики

- ✅ Никогда не вшивайте токены в код
- ✅ Используйте переменные окружения
- ✅ Не коммитьте отчеты в репозиторий
- ✅ Добавьте `*.json` в `.gitignore`
- ✅ Ограничьте доступ к отчетам (могут содержать серийные номера)

### Что собирается

Программа собирает **только** системную информацию:
- ❌ НЕ собирает пароли
- ❌ НЕ собирает файлы пользователя
- ❌ НЕ собирает историю браузера
- ❌ НЕ собирает ключи шифрования
- ✅ Только публичная системная информация

## 📄 Лицензия

MIT License - используйте как хотите!

## 🤝 Вклад

Pull requests приветствуются! Для серьезных изменений сначала откройте issue.

## 📞 Поддержка

- GitHub Issues: [Создать issue](https://github.com/yourusername/ytt/issues)
- Telegram: @yourusername

## 🎯 Roadmap

- [ ] GUI версия
- [ ] Отправка в Discord
- [ ] Экспорт в HTML/PDF
- [ ] Историческое отслеживание изменений
- [ ] Веб-панель для просмотра отчетов
- [ ] Docker образ

---

Сделано с ❤️ на Rust 🦀
