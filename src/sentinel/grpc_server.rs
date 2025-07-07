/// gRPC server implementation for SwarmReport Sentinel
///
/// Handles incoming system reports from client machines and forwards them
/// to the state manager via a broadcast channel.
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tonic::transport::Server;

use super::types::ReportEntry;
use crate::swarmreport::swarm_report_service_server::SwarmReportService;
use crate::swarmreport::{ReportResponse, SwarmReportRequest, SwarmReportResponse, SystemReport};

pub mod swarmreport {
    tonic::include_proto!("swarmreport");
}

/// The main Sentinel service that receives reports from clients
#[derive(Debug)]
pub struct Sentinel {
    pub report_sender: broadcast::Sender<ReportEntry>,
}

#[tonic::async_trait]
impl SwarmReportService for Sentinel {
    /// Receives a system report from a client and forwards it to the state manager
    async fn send_system_report(
        &self,
        request: tonic::Request<SystemReport>,
    ) -> Result<tonic::Response<ReportResponse>, tonic::Status> {
        let report = request.into_inner();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = ReportEntry {
            report,
            last_updated: timestamp,
        };
        let _ = self.report_sender.send(entry); // Ignore if no receivers

        Ok(tonic::Response::new(ReportResponse {
            message: "System report received successfully".to_string(),
            success: true,
        }))
    }

    /// Returns aggregated swarm information (currently just a placeholder)
    async fn get_swarm_report(
        &self,
        _request: tonic::Request<SwarmReportRequest>,
    ) -> Result<tonic::Response<SwarmReportResponse>, tonic::Status> {
        Ok(tonic::Response::new(SwarmReportResponse {
            reports: vec![], // TODO: Implement actual swarm report aggregation
            message: "Swarm report generated successfully".to_string(),
        }))
    }
}

/// Starts the gRPC server on port 50051
pub async fn run_grpc_server(
    sentinel: Sentinel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:50051".parse()?;

    Server::builder()
        .add_service(
            crate::swarmreport::swarm_report_service_server::SwarmReportServiceServer::new(
                sentinel,
            ),
        )
        .serve(addr)
        .await?;

    Ok(())
}
