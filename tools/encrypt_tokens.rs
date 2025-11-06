#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! aes-gcm = "0.10"
//! argon2 = "0.5"
//! base64 = "0.21"
//! rand = "0.8"
//! anyhow = "1.0"
//! ```

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use std::io::{self, Write};
use anyhow::{Result, Context};

const NONCE_SIZE: usize = 12;

fn main() -> Result<()> {
    println!("\n╔════════════════════════════════════════╗");
    println!("║  System Reporter - Token Encryptor    ║");
    println!("╚════════════════════════════════════════╝\n");

    println!("Этот инструмент поможет зашифровать ваши Telegram учетные данные.");
    println!("Зашифрованные токены можно безопасно хранить в переменных окружения.\n");

    // Get bot token
    print!("Введите Telegram Bot Token: ");
    io::stdout().flush()?;
    let mut bot_token = String::new();
    io::stdin().read_line(&mut bot_token)?;
    let bot_token = bot_token.trim();

    // Get chat ID
    print!("Введите Chat ID: ");
    io::stdout().flush()?;
    let mut chat_id = String::new();
    io::stdin().read_line(&mut chat_id)?;
    let chat_id = chat_id.trim();

    // Get password
    print!("Введите пароль для шифрования (запомните его!): ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    if password.len() < 8 {
        println!("\n⚠ ВНИМАНИЕ: Используйте пароль длиной минимум 8 символов!");
    }

    // Confirm password
    print!("Подтвердите пароль: ");
    io::stdout().flush()?;
    let mut password_confirm = String::new();
    io::stdin().read_line(&mut password_confirm)?;
    let password_confirm = password_confirm.trim();

    if password != password_confirm {
        println!("\n❌ Пароли не совпадают!");
        return Ok(());
    }

    println!("\n🔐 Шифрование...");

    // Encrypt tokens
    let encrypted_bot_token = encrypt_token(bot_token, password)?;
    let encrypted_chat_id = encrypt_token(chat_id, password)?;

    println!("\n✓ Токены успешно зашифрованы!\n");
    println!("════════════════════════════════════════════════════════════════");
    println!("Установите следующие переменные окружения:\n");

    println!("Windows (PowerShell):");
    println!("─────────────────────");
    println!("$env:ENCRYPTED_BOT_TOKEN=\"{}\"", encrypted_bot_token);
    println!("$env:ENCRYPTED_CHAT_ID=\"{}\"", encrypted_chat_id);
    println!("$env:ENCRYPTION_PASSWORD=\"{}\"", password);
    println!();

    println!("Windows (CMD):");
    println!("──────────────");
    println!("set ENCRYPTED_BOT_TOKEN={}", encrypted_bot_token);
    println!("set ENCRYPTED_CHAT_ID={}", encrypted_chat_id);
    println!("set ENCRYPTION_PASSWORD={}", password);
    println!();

    println!("Linux/macOS:");
    println!("────────────");
    println!("export ENCRYPTED_BOT_TOKEN=\"{}\"", encrypted_bot_token);
    println!("export ENCRYPTED_CHAT_ID=\"{}\"", encrypted_chat_id);
    println!("export ENCRYPTION_PASSWORD=\"{}\"", password);
    println!();

    println!("════════════════════════════════════════════════════════════════");
    println!("\n⚠ ВАЖНО:");
    println!("  • Сохраните эти команды в безопасном месте");
    println!("  • НЕ коммитьте их в Git");
    println!("  • Пароль необходим для расшифровки токенов");
    println!("  • Если потеряете пароль, придется создать новые токены\n");

    Ok(())
}

fn encrypt_token(token: &str, password: &str) -> Result<String> {
    // Генерируем соль для Argon2
    let salt = SaltString::generate(&mut OsRng);

    // Создаем ключ из пароля через Argon2
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .context("Ошибка хэширования пароля")?;

    // Извлекаем первые 32 байта для AES-256
    let key_bytes = password_hash.hash.unwrap().as_bytes();
    let key = &key_bytes[..32];

    // Создаем шифр
    let cipher = Aes256Gcm::new_from_slice(key)
        .context("Ошибка создания шифра")?;

    // Генерируем nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Шифруем
    let ciphertext = cipher
        .encrypt(nonce, token.as_bytes())
        .map_err(|e| anyhow::anyhow!("Ошибка шифрования: {}", e))?;

    // Объединяем: nonce + salt + ciphertext
    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(salt.as_str().as_bytes());
    result.extend_from_slice(&ciphertext);

    // Кодируем в base64
    Ok(general_purpose::STANDARD.encode(&result))
}
