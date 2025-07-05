use tonic::{Request, Response, Status, transport::Server};

use crate::swarmreport::swarm_report_service_server::SwarmReportService;
use crate::swarmreport::{ReportResponse, SwarmReportRequest, SwarmReportResponse, SystemReport};

pub mod swarmreport {
    tonic::include_proto!("swarmreport");
}

#[derive(Debug, Default)]
struct Sentinel {}

#[tonic::async_trait]
impl SwarmReportService for Sentinel {
    async fn send_system_report(
        &self,
        request: tonic::Request<swarmreport::SystemReport>,
    ) -> Result<tonic::Response<swarmreport::ReportResponse>, tonic::Status> {
        println!("Received system report: {:?}", request.get_ref());
        Ok(tonic::Response::new(swarmreport::ReportResponse {
            message: "System report received successfully".to_string(),
            success: true,
        }))
    }

    async fn get_swarm_report(
        &self,
        request: tonic::Request<swarmreport::SwarmReportRequest>,
    ) -> Result<tonic::Response<swarmreport::SwarmReportResponse>, tonic::Status> {
        println!("Received swarm report request: {:?}", request.get_ref());
        let report = swarmreport::SwarmReportResponse {
            reports: vec![],
            message: "Swarm report generated successfully".to_string(),
        };
        Ok(tonic::Response::new(report))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let sentinel = Sentinel::default();

    Server::builder()
        .add_service(
            swarmreport::swarm_report_service_server::SwarmReportServiceServer::new(sentinel),
        )
        .serve(addr)
        .await?;

    println!("This is SwarmReport Sentinel 0.1. Listening for reports...");

    Ok(())
}
