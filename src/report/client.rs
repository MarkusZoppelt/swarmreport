use crate::swarmreport::swarm_report_service_client::SwarmReportServiceClient;
use crate::swarmreport::SwarmReportRequest;
use crate::report::system::get_system_report;

pub async fn send_system_report() -> Result<(), Box<dyn std::error::Error>> {
    let report = get_system_report();

    let mut client = SwarmReportServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(report);
    let response = client.send_system_report(request).await?;
    println!("Response from server: {:?}", response.into_inner());

    Ok(())
}

pub async fn get_swarm_report() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SwarmReportServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(SwarmReportRequest {});
    let response = client.get_swarm_report(request).await?;
    println!("Swarm report: {:?}", response.into_inner());

    Ok(())
}
