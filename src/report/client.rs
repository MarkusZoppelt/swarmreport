use crate::report::system::get_system_report;
use crate::swarmreport::SwarmReportRequest;
/// Client functions for communicating with the SwarmReport Sentinel
///
/// These functions handle sending system reports to the sentinel server
/// and retrieving aggregated swarm information.
use crate::swarmreport::swarm_report_service_client::SwarmReportServiceClient;

/// Sends the current system report to the sentinel server
pub async fn send_system_report() -> Result<(), Box<dyn std::error::Error>> {
    let report = get_system_report();

    let server_addr = std::env::var("SWARM_SENTINEL_ADDR")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());

    let mut client = SwarmReportServiceClient::connect(server_addr).await?;
    let response = client
        .send_system_report(tonic::Request::new(report))
        .await?;

    println!("Response from server: {:?}", response.into_inner());
    Ok(())
}

/// Retrieves aggregated swarm information from the sentinel server
pub async fn get_swarm_report() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr =
        std::env::var("SWARM_SENTINEL_ADDR").unwrap_or_else(|_| "http://gordon:50051".to_string());

    let mut client = SwarmReportServiceClient::connect(server_addr).await?;
    let response = client
        .get_swarm_report(tonic::Request::new(SwarmReportRequest {}))
        .await?;

    println!("Swarm report: {:?}", response.into_inner());
    Ok(())
}
