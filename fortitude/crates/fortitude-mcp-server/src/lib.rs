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
