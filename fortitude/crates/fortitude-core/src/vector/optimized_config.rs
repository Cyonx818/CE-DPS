// ABOUTME: Comprehensive performance-optimized configuration for vector operations
//! This module provides a unified configuration system that optimizes all vector operations
//! for <200ms response times with intelligent defaults and tuning parameters.

use crate::vector::{
    cache::CacheConfig,
    error::{VectorError, VectorResult},
    optimized_embeddings::OptimizedEmbeddingConfig,
    performance::PerformanceConfig,
    VectorConfig,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

/// Master configuration for optimized vector operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedVectorConfig {
    /// Core vector database configuration
    pub vector_db: VectorConfig,
    /// Embedding generation configuration
    pub embeddings: OptimizedEmbeddingConfig,
    /// Performance optimization settings
    pub performance: PerformanceConfig,
    /// Caching configuration
    pub caching: CacheConfig,
    /// Environment-specific optimizations
    pub environment: EnvironmentConfig,
    /// Automatic tuning settings
    pub auto_tuning: AutoTuningConfig,
}

/// Environment-specific optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Deployment environment
    pub environment_type: EnvironmentType,
    /// Available CPU cores
    pub cpu_cores: usize,
    /// Available memory (MB)
    pub memory_mb: usize,
    /// Network latency to vector DB (ms)
    pub network_latency_ms: f64,
    /// Storage type (SSD/HDD/Network)
    pub storage_type: StorageType,
    /// Expected load characteristics
    pub load_profile: LoadProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentType {
    Development,
    Testing,
    Staging,
    Production,
    HighPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    SSD,
    HDD,
    NetworkAttached,
    InMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProfile {
    /// Expected requests per second
    pub requests_per_second: f64,
    /// Peak request multiplier
    pub peak_multiplier: f64,
    /// Average text length
    pub avg_text_length: usize,
    /// Percentage of repeated queries
    pub query_repetition_rate: f64,
    /// Batch request percentage
    pub batch_request_percentage: f64,
}

/// Automatic tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTuningConfig {
    /// Enable automatic performance tuning
    pub enabled: bool,
    /// Tuning sensitivity (0.0-1.0)
    pub sensitivity: f64,
    /// Minimum observation period before adjustments
    pub observation_period: Duration,
    /// Maximum adjustment magnitude per iteration
    pub max_adjustment_factor: f64,
    /// Performance targets for tuning
    pub targets: PerformanceTargets,
    /// Tuning strategies to enable
    pub strategies: TuningStrategies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Target response time (ms)
    pub target_response_time_ms: f64,
    /// Target throughput (ops/sec)
    pub target_throughput_ops_sec: f64,
    /// Target cache hit rate
    pub target_cache_hit_rate: f64,
    /// Target memory utilization
    pub target_memory_utilization: f64,
    /// Target CPU utilization
    pub target_cpu_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningStrategies {
    /// Auto-adjust cache sizes
    pub cache_size_tuning: bool,
    /// Auto-adjust batch sizes
    pub batch_size_tuning: bool,
    /// Auto-adjust connection pool sizes
    pub connection_pool_tuning: bool,
    /// Auto-adjust query timeouts
    pub timeout_tuning: bool,
    /// Auto-adjust resource limits
    pub resource_limit_tuning: bool,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment_type: EnvironmentType::Development,
            cpu_cores: num_cpus::get(),
            memory_mb: 4096, // 4GB default
            network_latency_ms: 10.0,
            storage_type: StorageType::SSD,
            load_profile: LoadProfile {
                requests_per_second: 100.0,
                peak_multiplier: 3.0,
                avg_text_length: 500,
                query_repetition_rate: 0.3,
                batch_request_percentage: 0.2,
            },
        }
    }
}

impl Default for AutoTuningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.5,
            observation_period: Duration::from_secs(300), // 5 minutes
            max_adjustment_factor: 0.2,                   // 20% max change
            targets: PerformanceTargets {
                target_response_time_ms: 200.0,
                target_throughput_ops_sec: 100.0,
                target_cache_hit_rate: 0.8,
                target_memory_utilization: 0.8,
                target_cpu_utilization: 0.7,
            },
            strategies: TuningStrategies {
                cache_size_tuning: true,
                batch_size_tuning: true,
                connection_pool_tuning: true,
                timeout_tuning: true,
                resource_limit_tuning: true,
            },
        }
    }
}

impl Default for OptimizedVectorConfig {
    fn default() -> Self {
        let environment = EnvironmentConfig::default();
        let auto_tuning = AutoTuningConfig::default();

        Self {
            vector_db: VectorConfig::default(),
            embeddings: OptimizedEmbeddingConfig::default(),
            performance: PerformanceConfig::default(),
            caching: CacheConfig::default(),
            environment: environment.clone(),
            auto_tuning,
        }
    }
}

impl OptimizedVectorConfig {
    /// Create optimized configuration for specific environment
    pub fn for_environment(env_type: EnvironmentType) -> Self {
        let mut config = Self::default();
        config.environment.environment_type = env_type.clone();

        match env_type {
            EnvironmentType::Development => {
                config.performance.target_response_time_ms = 500; // More relaxed for dev
                config.performance.connection_pool.max_connections = 5;
                config.caching.l1_config.max_memory_bytes = 50 * 1024 * 1024; // 50MB
                config.embeddings.performance.max_concurrent_requests = 10;
            }
            EnvironmentType::Testing => {
                config.performance.target_response_time_ms = 300;
                config.performance.connection_pool.max_connections = 10;
                config.caching.l1_config.max_memory_bytes = 100 * 1024 * 1024; // 100MB
                config.embeddings.performance.max_concurrent_requests = 20;
            }
            EnvironmentType::Staging => {
                config.performance.target_response_time_ms = 250;
                config.performance.connection_pool.max_connections = 15;
                config.caching.l1_config.max_memory_bytes = 200 * 1024 * 1024; // 200MB
                config.embeddings.performance.max_concurrent_requests = 30;
            }
            EnvironmentType::Production => {
                config.performance.target_response_time_ms = 200;
                config.performance.connection_pool.max_connections = 20;
                config.caching.l1_config.max_memory_bytes = 512 * 1024 * 1024; // 512MB
                config.embeddings.performance.max_concurrent_requests = 50;
                config.auto_tuning.enabled = true;
            }
            EnvironmentType::HighPerformance => {
                config.performance.target_response_time_ms = 100;
                config.performance.connection_pool.max_connections = 50;
                config.caching.l1_config.max_memory_bytes = 1024 * 1024 * 1024; // 1GB
                config.embeddings.performance.max_concurrent_requests = 100;
                config.auto_tuning.enabled = true;
                config.auto_tuning.sensitivity = 0.8; // More aggressive tuning
            }
        }

        config
    }

    /// Auto-optimize configuration based on system capabilities
    pub fn auto_optimize(&mut self) -> VectorResult<()> {
        info!(
            "Auto-optimizing configuration for system with {} cores, {}MB memory",
            self.environment.cpu_cores, self.environment.memory_mb
        );

        // Optimize based on CPU cores
        let cpu_factor = (self.environment.cpu_cores as f64 / 4.0).clamp(1.0, 4.0);

        // Optimize connection pool
        self.performance.connection_pool.max_connections =
            (self.performance.connection_pool.max_connections as f64 * cpu_factor) as usize;

        // Optimize concurrent requests
        self.embeddings.performance.max_concurrent_requests =
            (self.embeddings.performance.max_concurrent_requests as f64 * cpu_factor) as usize;

        // Optimize batch sizes based on CPU
        self.performance
            .batch_processing
            .optimal_embedding_batch_size = (16.0 * cpu_factor) as usize;

        // Optimize memory usage
        let memory_factor = (self.environment.memory_mb as f64 / 4096.0).clamp(0.5, 4.0);
        self.caching.l1_config.max_memory_bytes =
            (self.caching.l1_config.max_memory_bytes as f64 * memory_factor) as usize;

        // Optimize for storage type
        match self.environment.storage_type {
            StorageType::SSD => {
                // Faster cache eviction for SSD
                self.caching.l1_config.ttl = Duration::from_secs(1800); // 30 minutes
            }
            StorageType::HDD => {
                // Longer cache retention for HDD
                self.caching.l1_config.ttl = Duration::from_secs(7200); // 2 hours
            }
            StorageType::NetworkAttached => {
                // Aggressive caching for network storage
                self.caching.l1_config.ttl = Duration::from_secs(3600); // 1 hour
                self.caching.l1_config.max_memory_bytes *= 2; // Double cache size
            }
            StorageType::InMemory => {
                // Minimal caching for in-memory storage
                self.caching.l1_config.ttl = Duration::from_secs(300); // 5 minutes
            }
        }

        // Optimize for network latency
        if self.environment.network_latency_ms > 50.0 {
            // High latency - optimize for fewer round trips
            self.performance.connection_pool.min_connections *= 2;
            self.performance.batch_processing.optimal_search_batch_size *= 2;
            self.performance.query_optimization.query_timeout =
                Duration::from_millis((self.environment.network_latency_ms * 10.0) as u64);
        }

        // Optimize for load profile
        let load = &self.environment.load_profile;

        // High repetition rate - larger caches
        if load.query_repetition_rate > 0.5 {
            self.caching.l1_config.max_entries *= 2;
            self.caching.l1_config.max_memory_bytes =
                (self.caching.l1_config.max_memory_bytes as f64 * 1.5) as usize;
        }

        // High batch percentage - optimize batch processing
        if load.batch_request_percentage > 0.3 {
            self.performance
                .batch_processing
                .optimal_embedding_batch_size *= 2;
            self.embeddings.batching.max_batch_size *= 2;
        }

        info!("Configuration auto-optimization completed");
        Ok(())
    }

    /// Validate configuration for consistency and performance
    pub fn validate(&self) -> VectorResult<()> {
        // Validate target response time is achievable
        if self.performance.target_response_time_ms < 50 {
            return Err(VectorError::ConfigurationError(
                "Target response time too aggressive (< 50ms)".to_string(),
            ));
        }

        // Validate memory allocation doesn't exceed system memory
        let cache_memory_mb = self.caching.l1_config.max_memory_bytes / (1024 * 1024);
        if cache_memory_mb > self.environment.memory_mb / 2 {
            warn!(
                "Cache memory allocation ({}MB) is more than 50% of system memory ({}MB)",
                cache_memory_mb, self.environment.memory_mb
            );
        }

        // Validate connection pool settings
        if self.performance.connection_pool.min_connections
            > self.performance.connection_pool.max_connections
        {
            return Err(VectorError::ConfigurationError(
                "Minimum connections cannot exceed maximum connections".to_string(),
            ));
        }

        // Validate batch sizes
        if self
            .performance
            .batch_processing
            .optimal_embedding_batch_size
            == 0
        {
            return Err(VectorError::ConfigurationError(
                "Embedding batch size must be greater than 0".to_string(),
            ));
        }

        // Validate timeouts
        if self.performance.connection_pool.connect_timeout.as_secs() == 0 {
            return Err(VectorError::ConfigurationError(
                "Connection timeout must be greater than 0".to_string(),
            ));
        }

        info!("Configuration validation passed");
        Ok(())
    }

    /// Generate configuration recommendations based on usage patterns
    pub fn get_recommendations(
        &self,
        usage_stats: &UsageStatistics,
    ) -> ConfigurationRecommendations {
        let mut recommendations = ConfigurationRecommendations::new();

        // Cache hit rate recommendations
        if usage_stats.avg_cache_hit_rate < 0.6 {
            recommendations.add_recommendation(
                RecommendationType::CacheOptimization,
                "Consider increasing cache size - low hit rate detected".to_string(),
                Priority::High,
            );
        }

        // Response time recommendations
        if usage_stats.avg_response_time_ms > self.performance.target_response_time_ms as f64 * 1.2
        {
            recommendations.add_recommendation(
                RecommendationType::PerformanceOptimization,
                "Response time exceeds target by >20% - consider optimizing".to_string(),
                Priority::High,
            );
        }

        // Throughput recommendations
        if usage_stats.throughput_ops_sec < self.auto_tuning.targets.target_throughput_ops_sec * 0.8
        {
            recommendations.add_recommendation(
                RecommendationType::ThroughputOptimization,
                "Throughput below target - consider increasing concurrency".to_string(),
                Priority::Medium,
            );
        }

        // Memory utilization recommendations
        if usage_stats.memory_utilization > 0.9 {
            recommendations.add_recommendation(
                RecommendationType::ResourceOptimization,
                "High memory utilization - consider increasing cache eviction".to_string(),
                Priority::High,
            );
        }

        recommendations
    }

    /// Apply automatic tuning adjustments
    pub fn apply_auto_tuning(&mut self, metrics: &PerformanceMetrics) -> VectorResult<()> {
        if !self.auto_tuning.enabled {
            return Ok(());
        }

        let targets = &self.auto_tuning.targets;
        let max_adjustment = self.auto_tuning.max_adjustment_factor;

        // Adjust cache size based on hit rate
        if self.auto_tuning.strategies.cache_size_tuning
            && metrics.cache_hit_rate < targets.target_cache_hit_rate
        {
            let increase_factor = 1.0 + (max_adjustment * 0.5);
            self.caching.l1_config.max_entries =
                (self.caching.l1_config.max_entries as f64 * increase_factor) as usize;

            info!(
                "Auto-tuning: Increased cache size by {:.1}%",
                (increase_factor - 1.0) * 100.0
            );
        }

        // Adjust batch sizes based on throughput
        if self.auto_tuning.strategies.batch_size_tuning
            && metrics.throughput_ops_sec < targets.target_throughput_ops_sec
        {
            let increase_factor = 1.0 + (max_adjustment * 0.3);
            self.performance
                .batch_processing
                .optimal_embedding_batch_size = (self
                .performance
                .batch_processing
                .optimal_embedding_batch_size
                as f64
                * increase_factor) as usize;

            info!(
                "Auto-tuning: Increased batch size by {:.1}%",
                (increase_factor - 1.0) * 100.0
            );
        }

        // Adjust connection pool based on response time
        if self.auto_tuning.strategies.connection_pool_tuning
            && metrics.avg_latency_ms > targets.target_response_time_ms
        {
            let current_max = self.performance.connection_pool.max_connections;
            let increase = (current_max as f64 * max_adjustment * 0.2) as usize;
            self.performance.connection_pool.max_connections = current_max + increase.max(1);

            info!(
                "Auto-tuning: Increased connection pool to {}",
                self.performance.connection_pool.max_connections
            );
        }

        Ok(())
    }
}

/// Performance metrics for auto-tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_latency_ms: f64,
    pub throughput_ops_sec: f64,
    pub cache_hit_rate: f64,
    pub memory_utilization: f64,
    pub cpu_utilization: f64,
    pub error_rate: f64,
}

/// Usage statistics for generating recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub avg_response_time_ms: f64,
    pub avg_cache_hit_rate: f64,
    pub throughput_ops_sec: f64,
    pub memory_utilization: f64,
    pub cpu_utilization: f64,
    pub error_rate: f64,
    pub peak_concurrent_requests: usize,
    pub avg_batch_size: f64,
}

/// Configuration recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationRecommendations {
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: Priority,
    pub estimated_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    CacheOptimization,
    PerformanceOptimization,
    ThroughputOptimization,
    ResourceOptimization,
    ConfigurationTuning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for ConfigurationRecommendations {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationRecommendations {
    pub fn new() -> Self {
        Self {
            recommendations: Vec::new(),
        }
    }

    pub fn add_recommendation(
        &mut self,
        rec_type: RecommendationType,
        description: String,
        priority: Priority,
    ) {
        self.recommendations.push(Recommendation {
            recommendation_type: rec_type,
            description,
            priority,
            estimated_impact: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_config_creation() {
        let config = OptimizedVectorConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_environment_specific_config() {
        let prod_config = OptimizedVectorConfig::for_environment(EnvironmentType::Production);
        let dev_config = OptimizedVectorConfig::for_environment(EnvironmentType::Development);

        assert!(
            prod_config.performance.target_response_time_ms
                < dev_config.performance.target_response_time_ms
        );
        assert!(
            prod_config.performance.connection_pool.max_connections
                > dev_config.performance.connection_pool.max_connections
        );
    }

    #[test]
    fn test_auto_optimization() {
        let mut config = OptimizedVectorConfig::default();
        config.environment.cpu_cores = 8;
        config.environment.memory_mb = 8192;

        let original_connections = config.performance.connection_pool.max_connections;
        config.auto_optimize().unwrap();

        assert!(config.performance.connection_pool.max_connections >= original_connections);
    }

    #[test]
    fn test_config_validation() {
        let mut config = OptimizedVectorConfig::default();

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Invalid configuration should fail
        config.performance.target_response_time_ms = 10; // Too aggressive
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_recommendations() {
        let config = OptimizedVectorConfig::default();
        let usage_stats = UsageStatistics {
            avg_response_time_ms: 500.0, // High response time
            avg_cache_hit_rate: 0.3,     // Low hit rate
            throughput_ops_sec: 50.0,
            memory_utilization: 0.95, // High memory usage
            cpu_utilization: 0.6,
            error_rate: 0.01,
            peak_concurrent_requests: 20,
            avg_batch_size: 10.0,
        };

        let recommendations = config.get_recommendations(&usage_stats);
        assert!(!recommendations.recommendations.is_empty());

        // Should have high priority recommendations
        let high_priority_count = recommendations
            .recommendations
            .iter()
            .filter(|r| matches!(r.priority, Priority::High))
            .count();
        assert!(high_priority_count > 0);
    }
}
