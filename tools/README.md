# 🔐 System Reporter - Encryption Utility

Утилита для безопасного шифрования Telegram учетных данных.

## Использование

### Вариант 1: Через Cargo

```bash
cd tools
cargo run --release
```

### Вариант 2: Компиляция отдельного исполняемого файла

```bash
# Из корня проекта
cargo build --release --bin encrypt_tokens

# Запуск
./target/release/encrypt_tokens
```

### Вариант 3: Rust-script (если установлен cargo-script)

```bash
cargo install cargo-script
cargo script tools/encrypt_tokens.rs
```

## Что она делает?

1. Запрашивает ваши Telegram Bot Token и Chat ID
2. Запрашивает пароль для шифрования (минимум 8 символов)
3. Шифрует данные с использованием AES-256-GCM + Argon2
4. Выводит готовые команды для установки переменных окружения

## Примеры вывода

После запуска вы получите готовые команды:

```powershell
# Windows PowerShell
$env:ENCRYPTED_BOT_TOKEN="AbCdEf123..."
$env:ENCRYPTED_CHAT_ID="XyZ789..."
$env:ENCRYPTION_PASSWORD="your_password"
```

```bash
# Linux/macOS
export ENCRYPTED_BOT_TOKEN="AbCdEf123..."
export ENCRYPTED_CHAT_ID="XyZ789..."
export ENCRYPTION_PASSWORD="your_password"
```

## Безопасность

✅ **Что безопасно:**
- Хранить ENCRYPTED_BOT_TOKEN в переменных окружения
- Хранить ENCRYPTED_CHAT_ID в переменных окружения
- Использовать в CI/CD пайплайнах

⚠️ **Что НЕ безопасно:**
- Хранить ENCRYPTION_PASSWORD в Git
- Использовать слабые пароли (меньше 8 символов)
- Передавать пароль через незащищенные каналы

## Технические детали

- **Алгоритм шифрования:** AES-256-GCM
- **Ключ деривация:** Argon2 (с random salt)
- **Формат:** base64(nonce + salt + ciphertext)
- **Безопасность:** Military-grade encryption

## Альтернатива: Использовать незашифрованные токены

Если шифрование не требуется, можно использовать обычные переменные:

```bash
export TELEGRAM_BOT_TOKEN="123456:ABC-DEF..."
export TELEGRAM_CHAT_ID="987654321"
```

⚠️ При этом программа выведет предупреждение о незащищенных данных.
