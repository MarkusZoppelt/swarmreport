/// SwarmReport Sentinel - Central monitoring server
///
/// This is the main sentinel server that receives system reports from multiple
/// client machines and displays them via both a terminal UI and web dashboard.
///
/// The server runs four concurrent tasks:
/// - gRPC server: Receives reports from clients
/// - State manager: Processes reports and cleans up offline clients  
/// - TUI: Terminal interface for real-time monitoring
/// - Web server: HTTP API and dashboard
mod sentinel;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

use sentinel::{
    grpc_server::{Sentinel, run_grpc_server},
    tui::run_tui_display_only,
    types::{App, ReportEntry, SharedState},
    web::run_web_server,
};

pub mod swarmreport {
    tonic::include_proto!("swarmreport");
}

/// Manages incoming reports and periodically cleans up offline clients
async fn run_state_manager(
    mut report_receiver: broadcast::Receiver<ReportEntry>,
    state: SharedState,
) {
    let mut last_cleanup = std::time::Instant::now();

    loop {
        // Process all pending reports
        while let Ok(report) = report_receiver.try_recv() {
            state.lock().unwrap().update_report(report);
        }

        // Clean up offline clients every 5 seconds
        if last_cleanup.elapsed() >= Duration::from_secs(5) {
            state.lock().unwrap().remove_offline_clients(60);
            last_cleanup = std::time::Instant::now();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create communication channel for reports
    let (report_sender, report_receiver) = broadcast::channel::<ReportEntry>(100);
    let sentinel = Sentinel { report_sender };

    // Create shared state for all components
    let shared_state = Arc::new(Mutex::new(App::new()));

    // Spawn all concurrent tasks
    let server_handle = tokio::spawn({
        let sentinel = sentinel;
        async move { run_grpc_server(sentinel).await }
    });

    let state_manager_handle = tokio::spawn({
        let state = shared_state.clone();
        async move { run_state_manager(report_receiver, state).await }
    });

    let tui_handle = tokio::spawn({
        let state = shared_state.clone();
        async move { run_tui_display_only(state).await }
    });

    let web_handle = tokio::spawn({
        let state = shared_state.clone();
        async move { run_web_server(state).await }
    });

    // Print startup information
    println!("SwarmReport Sentinel 0.1 - TUI + Web Mode");
    println!("Starting gRPC server on 0.0.0.0:50051");
    println!("Web dashboard available at http://localhost:6969");
    println!("Press 'q' in TUI to quit. Clients auto-remove after 60s offline.");

    // Wait for any task to complete (which means exit)
    tokio::select! {
        _ = server_handle => {},
        _ = state_manager_handle => {},
        _ = tui_handle => {},
        _ = web_handle => {},
    }

    Ok(())
}
