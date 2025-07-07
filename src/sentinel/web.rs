use super::types::SharedState;
/// Web server for SwarmReport Sentinel
///
/// Provides a REST API and web dashboard for viewing system reports from
/// connected clients. Includes endpoints for retrieving client data and
/// adding test clients for demonstration purposes.
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

/// API endpoint to get all connected clients and their current status
async fn get_clients_api(state: SharedState) -> Result<impl warp::Reply, warp::Rejection> {
    let app = state.lock().unwrap();
    Ok(warp::reply::json(&app.get_web_clients()))
}

/// API endpoint to add a test client for demonstration purposes
async fn add_test_client(state: SharedState) -> Result<impl warp::Reply, warp::Rejection> {
    use super::types::ReportEntry;
    use crate::swarmreport::{Service, SystemReport};

    let test_report = SystemReport {
        node_id: "test-node-123".to_string(),
        hostname: "test-host".to_string(),
        ip_address: "192.168.1.100".to_string(),
        cpu_usage: "45.5%".to_string(),
        memory_usage: "8.5/16 GB".to_string(),
        disk_usage: "120.5 GB / 500.2 GB".to_string(),
        services: vec![
            Service {
                name: "nginx".to_string(),
                status: "running".to_string(),
                needs_update: false,
            },
            Service {
                name: "postgres".to_string(),
                status: "stopped".to_string(),
                needs_update: true,
            },
        ],
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let entry = ReportEntry {
        report: test_report,
        last_updated: timestamp,
    };

    state.lock().unwrap().update_report(entry);
    Ok(warp::reply::with_status(
        "Test client added",
        warp::http::StatusCode::OK,
    ))
}

/// Serves the HTML dashboard page
async fn serve_dashboard() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::html(include_str!(
        "../../static/dashboard.html"
    )))
}

/// Starts the web server with API endpoints and dashboard
pub async fn run_web_server(
    state: SharedState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state_filter = warp::any().map(move || state.clone());

    // Define API routes
    let api_clients = warp::path!("api" / "clients")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(get_clients_api);

    let test_client = warp::path!("api" / "test")
        .and(warp::post())
        .and(state_filter)
        .and_then(add_test_client);

    let dashboard = warp::path::end().and(warp::get()).and_then(serve_dashboard);

    let routes = dashboard.or(api_clients).or(test_client);

    println!("Web dashboard available at http://localhost:6969");
    warp::serve(routes).run(([0, 0, 0, 0], 6969)).await;
    Ok(())
}
