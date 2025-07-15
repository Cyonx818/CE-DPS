// ABOUTME: HTTP API server entry point for Fortitude research system
// Provides production-ready JSON REST API with authentication, caching, and comprehensive endpoints

use anyhow::Result;
use fortitude_api_server::config::ApiServerConfig;
use fortitude_api_server::server::ApiServer;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Fortitude API Server");

    // Load configuration
    let config = ApiServerConfig::from_env()?;

    // Create and run server
    let server = ApiServer::new(config).await?;

    if let Err(e) = server.run().await {
        error!("Server error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}
