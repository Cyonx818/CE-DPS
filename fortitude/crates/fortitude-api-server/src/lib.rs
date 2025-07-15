// ABOUTME: Fortitude API Server library providing production-ready JSON REST API
// Includes authentication, caching, research endpoints, and comprehensive testing

pub mod config;
pub mod extractors;
pub mod middleware;
pub mod models;
pub mod monitoring_types;
pub mod routes;
pub mod server;

pub use config::ApiServerConfig;
pub use server::ApiServer;
pub use models::{LearningInsight, MonitoringMetricsQuery, HealthCheckRequest};
