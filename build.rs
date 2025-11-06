use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=.env");

    // Пытаемся загрузить .env файл
    let env_path = Path::new(".env");

    if env_path.exists() {
        println!("cargo:warning=🔧 Обнаружен .env файл - встраивание токенов в сборку...");

        match fs::read_to_string(env_path) {
            Ok(content) => {
                let mut found_tokens = false;

                for line in content.lines() {
                    let line = line.trim();

                    // Пропускаем комментарии и пустые строки
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Парсим переменные вида KEY=VALUE
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();

                        // Убираем кавычки если есть
                        let value = value.trim_matches('"').trim_matches('\'');

                        // Встраиваем только Telegram-связанные переменные
                        match key {
                            "TELEGRAM_BOT_TOKEN" |
                            "TELEGRAM_CHAT_ID" |
                            "ENCRYPTED_BOT_TOKEN" |
                            "ENCRYPTED_CHAT_ID" |
                            "ENCRYPTION_PASSWORD" => {
                                if !value.is_empty() {
                                    println!("cargo:rustc-env=EMBEDDED_{}={}", key, value);
                                    println!("cargo:warning=  ✓ Встроено: {}", key);
                                    found_tokens = true;
                                }
                            }
                            _ => {} // Игнорируем остальные переменные
                        }
                    }
                }

                if found_tokens {
                    println!("cargo:warning=✅ Токены успешно встроены в исполняемый файл!");
                    println!("cargo:warning=⚠️  ВАЖНО: Не распространяйте этот .exe публично!");
                } else {
                    println!("cargo:warning=ℹ️  .env файл пуст или не содержит токенов");
                }
            }
            Err(e) => {
                println!("cargo:warning=⚠️  Не удалось прочитать .env: {}", e);
            }
        }
    } else {
        println!("cargo:warning=ℹ️  .env файл не найден - сборка без встроенных токенов");
        println!("cargo:warning=   Токены нужно будет передавать через переменные окружения");
    }

    // Устанавливаем флаг что build script выполнен
    println!("cargo:rustc-env=BUILD_SCRIPT_RAN=1");
}
