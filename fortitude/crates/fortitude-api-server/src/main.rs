// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
