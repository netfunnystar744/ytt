use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// --- Главная структура отчета ---

#[derive(Serialize, Debug)]
struct SystemReport {
    // Модуль 1: Базовая информация
    username: String,
    hostname: String,
    os_name: String,
    os_version: String,
    system_uptime_seconds: u64,

    // Модуль 2: Оборудование
    cpu_model: String,
    cpu_cores: usize,
    total_memory_gb: f64,
    used_memory_gb: f64,
    disks: Vec<String>,

    // Модуль 3: Сеть
    local_ip: String,
    mac_addresses: HashMap<String, String>,

    // Модуль 4: Программное обеспечение
    process_count: usize,
    suspicious_processes: Vec<String>,
    all_processes: Vec<String>,
}


// --- Точка входа ---

fn main() {
    println!("=== System Reporter ===");
    println!("1. Запуск сборщика системной информации...\n");

    let report = collect_system_info();
    let filename = save_report(&report);

    println!("\n=== Результат ===");
    println!("✓ Отчет успешно создан: {}", filename);
    println!("✓ Собрано данных: {} процессов, {} дисков",
             report.process_count, report.disks.len());
    println!("\nИнформация сохранена в файл '{}'", filename);
}

fn collect_system_info() -> SystemReport {
    use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt};

    println!("2. Инициализация системных библиотек...");
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("3. Сбор системной информации...");
    println!("   - Базовая информация о системе");
    println!("   - Информация об оборудовании");
    println!("   - Сетевая информация");
    println!("   - Список процессов");

    SystemReport {
        // Модуль 1: Базовая информация
        username: whoami::username(),
        hostname: sys.host_name().unwrap_or_else(|| "N/A".to_string()),
        os_name: sys.name().unwrap_or_else(|| "N/A".to_string()),
        os_version: sys.os_version().unwrap_or_else(|| "N/A".to_string()),
        system_uptime_seconds: System::uptime(),

        // Модуль 2: Оборудование
        cpu_model: sys.global_cpu_info().brand().to_string(),
        cpu_cores: sys.cpus().len(),
        total_memory_gb: (sys.total_memory() as f64) / (1024.0 * 1024.0 * 1024.0),
        used_memory_gb: (sys.used_memory() as f64) / (1024.0 * 1024.0 * 1024.0),
        disks: sys.disks().iter().map(|d|
            format!("{} - {} ({:?})",
                d.name().to_string_lossy(),
                format_bytes(d.total_space()),
                d.kind())
        ).collect(),

        // Модуль 3: Сеть
        local_ip: get_local_ip(),
        mac_addresses: get_mac_addresses(),

        // Модуль 4: Программное обеспечение
        process_count: sys.processes().len(),
        suspicious_processes: find_suspicious_processes(&sys),
        all_processes: sys.processes().values().map(|p| p.name().to_string()).collect(),
    }
}

fn save_report(report: &SystemReport) -> String {
    let report_json = serde_json::to_string_pretty(report)
        .expect("Ошибка сериализации отчета в JSON");

    let filename = "report.json";
    fs::write(filename, report_json)
        .expect("Ошибка записи отчета в файл");

    filename.to_string()
}


// --- Вспомогательные функции ---

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn get_local_ip() -> String {
    use std::net::TcpStream;

    // Пытаемся определить локальный IP через соединение с внешним сервером
    // (соединение не устанавливается реально, просто определяется интерфейс)
    if let Ok(stream) = TcpStream::connect("8.8.8.8:80") {
        if let Ok(local_addr) = stream.local_addr() {
            return local_addr.ip().to_string();
        }
    }

    "N/A".to_string()
}

fn get_mac_addresses() -> HashMap<String, String> {
    use sysinfo::{Networks, NetworksExt, NetworkExt};
    let mut macs = HashMap::new();
    let networks = Networks::new_with_refreshed_list();

    for (interface_name, data) in &networks {
        let mac = data.mac_address();
        let mac_str = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac.0[0], mac.0[1], mac.0[2], mac.0[3], mac.0[4], mac.0[5]);

        // Пропускаем нулевые MAC-адреса
        if mac_str != "00:00:00:00:00:00" {
            macs.insert(interface_name.clone(), mac_str);
        }
    }

    macs
}

fn find_suspicious_processes(sys: &sysinfo::System) -> Vec<String> {
    use sysinfo::{ProcessExt, SystemExt};

    let vm_processes = ["vmtoolsd.exe", "VBoxService.exe", "VBoxTray.exe",
                        "vmtoolsd", "VBoxService", "VBoxTray"];
    let mut found = Vec::new();

    for (pid, process) in sys.processes() {
        if vm_processes.contains(&process.name()) {
            found.push(format!("{} (PID: {})", process.name(), pid));
        }
    }

    found
}
