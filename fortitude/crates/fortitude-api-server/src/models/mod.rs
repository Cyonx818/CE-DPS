// ABOUTME: HTTP-specific models for Fortitude API server
// Defines request/response schemas and error handling for REST API

pub mod errors;
pub mod requests;
pub mod responses;

// Re-export common types for easier access
pub use responses::{HealthCheckRequest, LearningInsight, MonitoringMetricsQuery};
