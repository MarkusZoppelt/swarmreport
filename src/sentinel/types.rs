use serde::{Deserialize, Serialize};
/// Data types and state management for the SwarmReport sentinel
///
/// This module defines the core data structures used to store and manage
/// system reports from multiple client machines.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::swarmreport::SystemReport;

#[derive(Clone, Debug)]
pub struct ReportEntry {
    pub report: SystemReport,
    pub last_updated: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebClient {
    pub hostname: String,
    pub ip_address: String,
    pub node_id: String,
    pub cpu_usage: f64,
    pub memory_usage: String,
    pub disk_usage: String,
    pub last_updated: u64,
    pub seconds_since_update: u64,
    pub status: String,
    pub services: Vec<WebService>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebService {
    pub name: String,
    pub status: String,
    pub needs_update: bool,
}

pub struct App {
    pub reports: HashMap<String, ReportEntry>,
    pub report_order: Vec<String>,
}

/// Thread-safe shared state for the application
pub type SharedState = Arc<Mutex<App>>;

/// Parses CPU usage string (e.g., "45.2%") into a float
pub fn parse_cpu_usage(cpu_str: &str) -> f64 {
    cpu_str.trim_end_matches('%').parse::<f64>().unwrap_or(0.0)
}

/// Gets current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl App {
    pub fn new() -> Self {
        Self {
            reports: HashMap::new(),
            report_order: Vec::new(),
        }
    }

    pub fn update_report(&mut self, report: ReportEntry) {
        let key = format!("{}:{}", report.report.hostname, report.report.ip_address);

        if let Some(existing) = self.reports.get_mut(&key) {
            existing.report = report.report;
            existing.last_updated = report.last_updated;
        } else {
            self.report_order.push(key.clone());
            self.reports.insert(key, report);
        }
    }

    /// Removes clients that haven't reported in for the specified timeout
    pub fn remove_offline_clients(&mut self, timeout_seconds: u64) {
        let current_time = current_timestamp();

        let keys_to_remove: Vec<String> = self
            .reports
            .iter()
            .filter(|(_, entry)| current_time.saturating_sub(entry.last_updated) > timeout_seconds)
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            self.reports.remove(&key);
            self.report_order.retain(|k| k != &key);
        }
    }

    pub fn get_ordered_reports(&self) -> Vec<&ReportEntry> {
        self.report_order
            .iter()
            .filter_map(|key| self.reports.get(key))
            .collect()
    }

    /// Converts internal reports to web-friendly format with status indicators
    pub fn get_web_clients(&self) -> Vec<WebClient> {
        let current_time = current_timestamp();

        self.get_ordered_reports()
            .iter()
            .map(|entry| {
                let seconds_since_update = current_time.saturating_sub(entry.last_updated);

                let status = match seconds_since_update {
                    0..=4 => "recent",
                    5..=30 => "normal",
                    _ => "stale",
                }
                .to_string();

                let services = entry
                    .report
                    .services
                    .iter()
                    .map(|s| WebService {
                        name: s.name.clone(),
                        status: s.status.clone(),
                        needs_update: s.needs_update,
                    })
                    .collect();

                WebClient {
                    hostname: entry.report.hostname.clone(),
                    ip_address: entry.report.ip_address.clone(),
                    node_id: entry.report.node_id.clone(),
                    cpu_usage: parse_cpu_usage(&entry.report.cpu_usage),
                    memory_usage: entry.report.memory_usage.clone(),
                    disk_usage: entry.report.disk_usage.clone(),
                    last_updated: entry.last_updated,
                    seconds_since_update,
                    status,
                    services,
                }
            })
            .collect()
    }
}
