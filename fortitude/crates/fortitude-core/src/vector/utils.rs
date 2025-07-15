// ABOUTME: Utility functions for vector database configuration conversion
use crate::vector::{
    ConnectionPoolConfig, DistanceMetric, EmbeddingConfig, HealthCheckConfig, VectorConfig,
};
use std::time::Duration;

/// Configuration parameters for CLI to vector config conversion
#[derive(Debug, Clone)]
pub struct CliConfigParams {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub default_collection: String,
    pub vector_dimensions: usize,
    pub distance_metric: String,
    pub health_check_enabled: bool,
    pub health_check_interval_seconds: u64,
    pub health_check_max_failures: u32,
    pub health_check_timeout_seconds: u64,
    pub connection_pool_max_connections: usize,
    pub connection_pool_idle_timeout_seconds: u64,
    pub connection_pool_connection_timeout_seconds: u64,
}

/// Convert CLI configuration to core vector configuration
pub fn cli_config_to_vector_config(params: CliConfigParams) -> Result<VectorConfig, String> {
    let distance_metric = match params.distance_metric.to_lowercase().as_str() {
        "cosine" => DistanceMetric::Cosine,
        "euclidean" => DistanceMetric::Euclidean,
        "dot" => DistanceMetric::Dot,
        _ => {
            return Err(format!(
                "Invalid distance metric: {}",
                params.distance_metric
            ))
        }
    };

    Ok(VectorConfig {
        url: params.url,
        api_key: params.api_key,
        timeout: Duration::from_secs(params.timeout_seconds),
        default_collection: params.default_collection,
        vector_dimensions: params.vector_dimensions,
        distance_metric,
        health_check: HealthCheckConfig {
            enabled: params.health_check_enabled,
            interval: Duration::from_secs(params.health_check_interval_seconds),
            max_failures: params.health_check_max_failures,
            timeout: Duration::from_secs(params.health_check_timeout_seconds),
        },
        connection_pool: ConnectionPoolConfig {
            max_connections: params.connection_pool_max_connections,
            idle_timeout: Duration::from_secs(params.connection_pool_idle_timeout_seconds),
            connection_timeout: Duration::from_secs(
                params.connection_pool_connection_timeout_seconds,
            ),
        },
        embedding: EmbeddingConfig::default(),
    })
}

// Note: CLI config conversion can be added when needed
// The conversion function cli_config_to_vector_config is available for manual use
