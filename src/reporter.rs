/// SwarmReport Reporter - Sends system metrics to the sentinel server
///
/// This binary runs on client machines and periodically sends system information
/// (CPU, memory, disk usage, running services) to a central sentinel server.
mod report;

pub mod swarmreport {
    tonic::include_proto!("swarmreport");
}

use report::{get_swarm_report, send_system_report};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SwarmReport Reporter starting...");

    loop {
        // Send reports every 500ms
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Continue the loop even if operations fail
        let _ = send_system_report().await;
        let _ = get_swarm_report().await;
    }
}
