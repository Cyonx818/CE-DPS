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

// ABOUTME: MCP server library for Fortitude AI research assistant integration
// Provides Model Context Protocol server implementation for seamless Claude Code integration
// Exposes research tools and resources via MCP protocol for external AI model access

pub mod auth;
pub mod config;
pub mod monitoring;
pub mod pattern_tracking;
pub mod proactive_tools;
pub mod quality_tools;
pub mod resources;
pub mod server;
pub mod tools;

pub use auth::{AuthManager, AuthMiddleware, Claims, Permission, RateLimitConfig};
pub use config::ServerConfig;
pub use monitoring::{
    McpMetrics, McpMonitoringService, McpOperationContext, McpPerformanceSummary,
};
pub use pattern_tracking::{
    McpPatternAggregationService, McpPatternTracker, McpPatternTrackingConfig,
};
pub use proactive_tools::ProactiveTools;
pub use quality_tools::QualityTools;
pub use resources::ResourceProvider;
pub use server::McpServer;
pub use tools::FortitudeTools;

// Re-export key types for consumers
pub use rmcp::Error as McpError;
pub use rust_mcp_schema::*;
