/// System information collection for SwarmReport
///
/// This module gathers system metrics including CPU, memory, disk usage,
/// IP address, and running Docker services.
use crate::swarmreport::{Service, SystemReport};
use local_ip_address::local_ip;
use sysinfo::{Disks, System};

/// Gets the system's IP address, preferring Tailscale if available
fn get_ip_address() -> String {
    // Try to get Tailscale IP first
    if let Ok(output) = std::process::Command::new("tailscale").arg("ip").output() {
        if output.status.success() {
            if let Ok(ip) = String::from_utf8(output.stdout) {
                return ip.lines().next().unwrap_or("unknown").to_string();
            }
        }
    }

    // Fall back to local IP
    local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Formats bytes into human-readable storage units (GB or TB)
fn format_storage_size(bytes: f64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1000.0;
    const TB: f64 = GB * 1000.0;

    if bytes > TB {
        format!("{:.2} TB", bytes / TB)
    } else {
        format!("{:.2} GB", bytes / GB)
    }
}

/// Gets total disk usage across all mounted disks
fn get_disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();
    if disks.is_empty() {
        return "unknown".to_string();
    }

    let (total_space, available_space): (f64, f64) = disks
        .list()
        .iter()
        .map(|disk| (disk.total_space() as f64, disk.available_space() as f64))
        .fold(
            (0.0, 0.0),
            |(total, available), (disk_total, disk_available)| {
                (total + disk_total, available + disk_available)
            },
        );

    let used_space = total_space - available_space;
    format!(
        "{} / {}",
        format_storage_size(used_space),
        format_storage_size(total_space)
    )
}

/// Formats memory usage as "used/total GB"
fn get_memory_usage(sys: &System) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1000.0;
    let used_gb = sys.used_memory() as f64 / GB;
    let total_gb = (sys.total_memory() as f64 / GB).round();
    format!("{used_gb:.1}/{total_gb} GB")
}

/// Gets running Docker containers as services
/// Returns empty vector if Docker is not available or no containers are running
fn get_services() -> Vec<Service> {
    let output = match std::process::Command::new("docker").arg("ps").output() {
        Ok(output) if output.status.success() => output,
        _ => return vec![], // Docker not available or command failed
    };

    let stdout = String::from_utf8(output.stdout).unwrap_or_default();

    stdout
        .lines()
        .skip(1) // Skip header line
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(Service {
                    name: parts[1].to_string(),
                    status: "running".to_string(),
                    needs_update: false, // TODO: Implement update checking
                })
            } else {
                None
            }
        })
        .collect()
}

/// Creates a unique node identifier from system information
fn create_node_id() -> String {
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
    format!("{hostname}-{os_name}-{os_version}")
}

/// Collects and returns a complete system report
pub fn get_system_report() -> SystemReport {
    let mut sys = System::new();

    // Refresh only the data we need
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    SystemReport {
        node_id: create_node_id(),
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        ip_address: get_ip_address(),
        cpu_usage: format!("{:.1}%", sys.global_cpu_usage()),
        memory_usage: get_memory_usage(&sys),
        disk_usage: get_disk_usage(),
        services: get_services(),
    }
}
