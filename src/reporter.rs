mod report;

pub mod swarmreport {
    tonic::include_proto!("swarmreport");
}

use report::{get_swarm_report, send_system_report};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    loop {
        // every second, send a system report
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        send_system_report().await?;
        get_swarm_report().await?;
    }
}
