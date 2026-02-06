use alloy_mcp::server::AlloyMcpServer;
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting alloy-mcp server");

    let server = AlloyMcpServer::new();
    let transport = (stdin(), stdout());

    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
