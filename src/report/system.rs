use crate::swarmreport::{Service, SystemReport};

use local_ip_address::local_ip;
use sysinfo::{Disks, System};

/// Retrieves the IP address of the system, preferring Tailscale if available.
fn get_ip_address() -> String {
    // run tailscale ip and get the first line, if tailscale command is available
    // TODO: use a tailscale client library instead of running a command
    if let Ok(output) = std::process::Command::new("tailscale").arg("ip").output() {
        if output.status.success() {
            if let Some(ip) = String::from_utf8(output.stdout).ok() {
                return ip.lines().next().unwrap_or("unknown").to_string();
            }
        }
    }

    let Ok(local_ip) = local_ip() else {
        return "unknown".to_string();
    };
    local_ip.to_string()
}

fn get_disk_usage() -> String {
    // get the largest disk and return its usage
    let disks = Disks::new_with_refreshed_list();
    if disks.is_empty() {
        return "unknown".to_string();
    }

    let mut total_space: f64 = 0.0;
    let mut available_space: f64 = 0.0;

    for disk in disks.list() {
        total_space += disk.total_space() as f64;
        available_space += disk.available_space() as f64;
    }

    let used_str: String = if available_space > 1e12 {
        // If available space is greater than 1 TB, format in TB
        format!(
            "{:.2} TB",
            (total_space - available_space) / 1024.0 / 1024.0 / 1024.0 / 1000.0
        )
    } else {
        // Otherwise, format in GB
        format!(
            "{:.2} GB",
            (total_space - available_space) / 1024.0 / 1024.0 / 1000.0
        )
    };

    let total_str: String = if total_space > 1e12 {
        // If total space is greater than 1 TB, format in TB
        format!("{:.2} TB", total_space / 1024.0 / 1024.0 / 1024.0 / 1000.0)
    } else {
        // Otherwise, format in GB
        format!("{:.2} GB", total_space / 1024.0 / 1024.0 / 1000.0)
    };

    format!("{} / {}", used_str, total_str)
}

fn get_memory_usage(sys: &System) -> String {
    format!(
        "{}/{} GB",
        format!("{:.1}", sys.used_memory() as f64 / 1024.0 / 1024.0 / 1000.0),
        (sys.total_memory() as f64 / 1024.0 / 1024.0 / 1000.0).round()
    )
}

// Retrieves the list of services running on the system and their statuses.
// for demonstration purposes, we limit this to `docker ps` output.
fn get_services() -> Vec<Service> {
    // try running `docker ps` command to get the list of services
    // // if it fails, return an empty vector
    if let Ok(output) = std::process::Command::new("docker").arg("ps").output() {
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap_or_default();
            let mut services = Vec::new();
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    services.push(Service {
                        name: parts[1].to_string(),
                        status: "running".to_string(),
                        needs_update: false, // Placeholder, as we don't check for updates here
                    });
                }
            }
            return services;
        }
    }
    return vec![];
}

pub fn get_system_report() -> SystemReport {
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    SystemReport {
        node_id: format!(
            "{}-{}-{}",
            System::host_name().unwrap_or_else(|| "unknown".to_string()),
            System::name().unwrap_or_else(|| "unknown".to_string()),
            System::os_version().unwrap_or_else(|| "unknown".to_string())
        ),
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        ip_address: get_ip_address(),
        cpu_usage: format!("{:.1}%", sys.global_cpu_usage()),

        memory_usage: get_memory_usage(&sys),
        disk_usage: get_disk_usage(),
        services: get_services(),
    }
}
