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

// ABOUTME: Configuration management for the Fortitude CLI
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Environment variable error: {0}")]
    #[allow(dead_code)]
    EnvironmentError(String),

    #[error("Configuration file error: {0}")]
    #[allow(dead_code)]
    FileError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for the Fortitude CLI application
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Claude API configuration
    pub claude: Option<ClaudeConfig>,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Classification configuration
    pub classification: ClassificationConfig,

    /// Pipeline configuration
    pub pipeline: PipelineConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Vector database configuration
    pub vector: Option<VectorDatabaseConfig>,
}

/// Claude API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// API key for Claude API
    pub api_key: String,

    /// API base URL (optional, defaults to official API)
    pub base_url: Option<String>,

    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Model to use (optional, defaults to claude-3-sonnet-20240229)
    pub model: Option<String>,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Retry configuration
    pub retry: RetryConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,

    /// Input tokens per minute
    pub input_tokens_per_minute: u32,

    /// Output tokens per minute
    pub output_tokens_per_minute: u32,

    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Enable jitter
    pub jitter: bool,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for storage
    pub base_path: PathBuf,

    /// Cache expiration in seconds
    pub cache_expiration_seconds: u64,

    /// Maximum cache size in bytes
    pub max_cache_size_bytes: u64,

    /// Enable content addressing
    pub enable_content_addressing: bool,

    /// Index update interval in seconds
    pub index_update_interval_seconds: u64,
}

/// Classification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationConfig {
    /// Default confidence threshold
    pub default_threshold: f64,

    /// Enable advanced classification
    pub enable_advanced: bool,

    /// Enable context detection
    pub enable_context_detection: bool,

    /// Custom classification rules
    pub custom_rules: Vec<String>,

    /// Context detection configuration
    pub context_detection: ContextDetectionConfig,

    /// Advanced classification configuration
    pub advanced_classification: AdvancedClassificationConfig,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Enable caching
    pub enable_caching: bool,

    /// Maximum parallel requests
    pub max_parallel_requests: u32,

    /// Processing timeout in seconds
    pub processing_timeout_seconds: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Enable file logging
    pub enable_file: bool,

    /// Log file path
    pub file_path: Option<PathBuf>,

    /// Enable JSON format
    pub json_format: bool,
}

/// Vector database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDatabaseConfig {
    /// Qdrant server URL
    pub url: String,

    /// API key for authentication (optional)
    pub api_key: Option<String>,

    /// Connection timeout in seconds
    pub timeout_seconds: u64,

    /// Default collection name for research documents
    pub default_collection: String,

    /// Vector dimensions for embeddings
    pub vector_dimensions: usize,

    /// Distance metric for similarity search
    pub distance_metric: String,

    /// Health check configuration
    pub health_check: VectorHealthCheckConfig,

    /// Connection pool configuration
    pub connection_pool: VectorConnectionPoolConfig,
}

/// Vector database health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHealthCheckConfig {
    /// Enable periodic health checks
    pub enabled: bool,

    /// Interval between health checks in seconds
    pub interval_seconds: u64,

    /// Maximum consecutive failures before marking unhealthy
    pub max_failures: u32,

    /// Health check timeout in seconds
    pub timeout_seconds: u64,
}

/// Vector database connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,

    /// Connection idle timeout in seconds
    pub idle_timeout_seconds: u64,

    /// Maximum time to wait for a connection in seconds
    pub connection_timeout_seconds: u64,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: None,
            timeout_seconds: Some(300),
            model: None,
            rate_limit: RateLimitConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 50,
            input_tokens_per_minute: 40_000,
            output_tokens_per_minute: 8_000,
            max_concurrent_requests: 5,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./reference_library"),
            cache_expiration_seconds: 86400,         // 24 hours
            max_cache_size_bytes: 100 * 1024 * 1024, // 100MB
            enable_content_addressing: true,
            index_update_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Context detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDetectionConfig {
    /// Context detection confidence threshold
    pub confidence_threshold: f64,

    /// Enable fallback to basic detection
    pub enable_fallback: bool,

    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: u64,

    /// Enable debug logging
    pub debug_logging: bool,
}

/// Advanced classification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedClassificationConfig {
    /// Enable graceful degradation
    pub enable_graceful_degradation: bool,

    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: u64,

    /// Signal composition configuration
    pub signal_composition: SignalCompositionConfig,

    /// Contextual weights for different scenarios
    pub contextual_weights: ContextualWeights,
}

/// Signal composition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCompositionConfig {
    /// Signal composition confidence threshold
    pub confidence_threshold: f64,

    /// Enable fallback to basic composition
    pub enable_fallback: bool,

    /// Maximum number of signals to process
    pub max_signals: usize,
}

/// Contextual weighting factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualWeights {
    /// Weight boost for urgent queries
    pub urgency_boost: f64,

    /// Weight boost for beginner audiences
    pub beginner_boost: f64,

    /// Weight boost for specific technical domains
    pub domain_boost: f64,

    /// Weight penalty for low confidence context
    pub low_confidence_penalty: f64,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            default_threshold: 0.1,
            enable_advanced: false,
            enable_context_detection: true,
            custom_rules: Vec::new(),
            context_detection: ContextDetectionConfig::default(),
            advanced_classification: AdvancedClassificationConfig::default(),
        }
    }
}

impl Default for ContextDetectionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.6,
            enable_fallback: true,
            max_processing_time_ms: 1000,
            debug_logging: false,
        }
    }
}

impl Default for AdvancedClassificationConfig {
    fn default() -> Self {
        Self {
            enable_graceful_degradation: true,
            max_processing_time_ms: 1000,
            signal_composition: SignalCompositionConfig::default(),
            contextual_weights: ContextualWeights::default(),
        }
    }
}

impl Default for SignalCompositionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.5,
            enable_fallback: true,
            max_signals: 10,
        }
    }
}

impl Default for ContextualWeights {
    fn default() -> Self {
        Self {
            urgency_boost: 1.3,
            beginner_boost: 1.2,
            domain_boost: 1.1,
            low_confidence_penalty: 0.8,
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_parallel_requests: 4,
            processing_timeout_seconds: 300,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_file: false,
            file_path: None,
            json_format: false,
        }
    }
}

impl Default for VectorDatabaseConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout_seconds: 30,
            default_collection: "fortitude_research".to_string(),
            vector_dimensions: 384,
            distance_metric: "cosine".to_string(),
            health_check: VectorHealthCheckConfig::default(),
            connection_pool: VectorConnectionPoolConfig::default(),
        }
    }
}

impl Default for VectorHealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            max_failures: 3,
            timeout_seconds: 5,
        }
    }
}

impl Default for VectorConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            idle_timeout_seconds: 300,
            connection_timeout_seconds: 10,
        }
    }
}

impl Config {
    /// Load configuration from environment variables and config file
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::default();

        // Load from environment variables
        config.load_from_env()?;

        // Load from config file if it exists
        if let Ok(config_path) = env::var("FORTITUDE_CONFIG") {
            config.load_from_file(&PathBuf::from(config_path))?;
        } else {
            // Try default config locations
            let default_paths = [
                PathBuf::from("./fortitude.json"),
                PathBuf::from("~/.config/fortitude/config.json"),
                PathBuf::from("/etc/fortitude/config.json"),
            ];

            for path in &default_paths {
                if path.exists() {
                    config.load_from_file(path)?;
                    break;
                }
            }
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from environment variables
    fn load_from_env(&mut self) -> Result<(), ConfigError> {
        // Claude API configuration
        if let Ok(api_key) = env::var("CLAUDE_API_KEY") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.api_key = api_key;
        }

        if let Ok(base_url) = env::var("CLAUDE_BASE_URL") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.base_url = Some(base_url);
        }

        if let Ok(model) = env::var("CLAUDE_MODEL") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.model = Some(model);
        }

        if let Ok(timeout) = env::var("CLAUDE_TIMEOUT_SECONDS") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.timeout_seconds = Some(timeout.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid timeout value: {timeout}"))
            })?);
        }

        // Rate limiting
        if let Ok(requests_per_minute) = env::var("CLAUDE_REQUESTS_PER_MINUTE") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.rate_limit.requests_per_minute =
                requests_per_minute.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid requests_per_minute value: {requests_per_minute}"
                    ))
                })?;
        }

        if let Ok(input_tokens_per_minute) = env::var("CLAUDE_INPUT_TOKENS_PER_MINUTE") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.rate_limit.input_tokens_per_minute =
                input_tokens_per_minute.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid input_tokens_per_minute value: {input_tokens_per_minute}"
                    ))
                })?;
        }

        if let Ok(output_tokens_per_minute) = env::var("CLAUDE_OUTPUT_TOKENS_PER_MINUTE") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.rate_limit.output_tokens_per_minute =
                output_tokens_per_minute.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid output_tokens_per_minute value: {output_tokens_per_minute}"
                    ))
                })?;
        }

        if let Ok(max_concurrent) = env::var("CLAUDE_MAX_CONCURRENT_REQUESTS") {
            let claude_config = self.claude.get_or_insert_with(ClaudeConfig::default);
            claude_config.rate_limit.max_concurrent_requests =
                max_concurrent.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid max_concurrent_requests value: {max_concurrent}"
                    ))
                })?;
        }

        // Storage configuration
        if let Ok(data_dir) = env::var("FORTITUDE_DATA_DIR") {
            self.storage.base_path = PathBuf::from(data_dir);
        }

        if let Ok(cache_expiration) = env::var("FORTITUDE_CACHE_EXPIRATION_SECONDS") {
            self.storage.cache_expiration_seconds = cache_expiration.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid cache_expiration_seconds value: {cache_expiration}"
                ))
            })?;
        }

        if let Ok(max_cache_size) = env::var("FORTITUDE_MAX_CACHE_SIZE_BYTES") {
            self.storage.max_cache_size_bytes = max_cache_size.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid max_cache_size_bytes value: {max_cache_size}"
                ))
            })?;
        }

        // Classification configuration
        if let Ok(threshold) = env::var("FORTITUDE_CLASSIFICATION_THRESHOLD") {
            self.classification.default_threshold = threshold.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid classification threshold: {threshold}"))
            })?;
        }

        if let Ok(advanced) = env::var("FORTITUDE_CLASSIFICATION_ADVANCED") {
            self.classification.enable_advanced = advanced.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid advanced classification value: {advanced}"
                ))
            })?;
        }

        if let Ok(context_detection) = env::var("FORTITUDE_CLASSIFICATION_CONTEXT_DETECTION") {
            self.classification.enable_context_detection =
                context_detection.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid context detection value: {context_detection}"
                    ))
                })?;
        }

        // Context detection configuration
        if let Ok(confidence_threshold) = env::var("FORTITUDE_CONTEXT_CONFIDENCE_THRESHOLD") {
            self.classification.context_detection.confidence_threshold =
                confidence_threshold.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid context confidence threshold: {confidence_threshold}"
                    ))
                })?;
        }

        if let Ok(enable_fallback) = env::var("FORTITUDE_CONTEXT_ENABLE_FALLBACK") {
            self.classification.context_detection.enable_fallback =
                enable_fallback.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid context fallback value: {enable_fallback}"
                    ))
                })?;
        }

        if let Ok(max_processing_time) = env::var("FORTITUDE_CONTEXT_MAX_PROCESSING_TIME_MS") {
            self.classification.context_detection.max_processing_time_ms =
                max_processing_time.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid context max processing time: {max_processing_time}"
                    ))
                })?;
        }

        if let Ok(debug_logging) = env::var("FORTITUDE_CONTEXT_DEBUG_LOGGING") {
            self.classification.context_detection.debug_logging =
                debug_logging.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid context debug logging value: {debug_logging}"
                    ))
                })?;
        }

        // Advanced classification configuration
        if let Ok(graceful_degradation) = env::var("FORTITUDE_ADVANCED_GRACEFUL_DEGRADATION") {
            self.classification
                .advanced_classification
                .enable_graceful_degradation = graceful_degradation.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid graceful degradation value: {graceful_degradation}"
                ))
            })?;
        }

        if let Ok(max_processing_time) = env::var("FORTITUDE_ADVANCED_MAX_PROCESSING_TIME_MS") {
            self.classification
                .advanced_classification
                .max_processing_time_ms = max_processing_time.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid advanced max processing time: {max_processing_time}"
                ))
            })?;
        }

        // Signal composition configuration
        if let Ok(confidence_threshold) = env::var("FORTITUDE_SIGNAL_CONFIDENCE_THRESHOLD") {
            self.classification
                .advanced_classification
                .signal_composition
                .confidence_threshold = confidence_threshold.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid signal confidence threshold: {confidence_threshold}"
                ))
            })?;
        }

        if let Ok(enable_fallback) = env::var("FORTITUDE_SIGNAL_ENABLE_FALLBACK") {
            self.classification
                .advanced_classification
                .signal_composition
                .enable_fallback = enable_fallback.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid signal fallback value: {enable_fallback}"
                ))
            })?;
        }

        if let Ok(max_signals) = env::var("FORTITUDE_SIGNAL_MAX_SIGNALS") {
            self.classification
                .advanced_classification
                .signal_composition
                .max_signals = max_signals.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid signal max signals: {max_signals}"))
            })?;
        }

        // Contextual weights configuration
        if let Ok(urgency_boost) = env::var("FORTITUDE_WEIGHT_URGENCY_BOOST") {
            self.classification
                .advanced_classification
                .contextual_weights
                .urgency_boost = urgency_boost.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid urgency boost: {urgency_boost}"))
            })?;
        }

        if let Ok(beginner_boost) = env::var("FORTITUDE_WEIGHT_BEGINNER_BOOST") {
            self.classification
                .advanced_classification
                .contextual_weights
                .beginner_boost = beginner_boost.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid beginner boost: {beginner_boost}"))
            })?;
        }

        if let Ok(domain_boost) = env::var("FORTITUDE_WEIGHT_DOMAIN_BOOST") {
            self.classification
                .advanced_classification
                .contextual_weights
                .domain_boost = domain_boost.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid domain boost: {domain_boost}"))
            })?;
        }

        if let Ok(low_confidence_penalty) = env::var("FORTITUDE_WEIGHT_LOW_CONFIDENCE_PENALTY") {
            self.classification
                .advanced_classification
                .contextual_weights
                .low_confidence_penalty = low_confidence_penalty.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid low confidence penalty: {low_confidence_penalty}"
                ))
            })?;
        }

        // Logging configuration
        if let Ok(log_level) = env::var("FORTITUDE_LOG_LEVEL") {
            self.logging.level = log_level;
        }

        if let Ok(log_file) = env::var("FORTITUDE_LOG_FILE") {
            self.logging.enable_file = true;
            self.logging.file_path = Some(PathBuf::from(log_file));
        }

        if let Ok(json_format) = env::var("FORTITUDE_LOG_JSON") {
            self.logging.json_format = json_format.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid json format value: {json_format}"))
            })?;
        }

        // Vector database configuration
        if let Ok(vector_url) = env::var("QDRANT_URL") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.url = vector_url;
        }

        if let Ok(vector_api_key) = env::var("QDRANT_API_KEY") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.api_key = Some(vector_api_key);
        }

        if let Ok(timeout) = env::var("QDRANT_TIMEOUT_SECONDS") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.timeout_seconds = timeout.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid qdrant timeout value: {timeout}"))
            })?;
        }

        if let Ok(collection) = env::var("QDRANT_DEFAULT_COLLECTION") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.default_collection = collection;
        }

        if let Ok(dimensions) = env::var("QDRANT_VECTOR_DIMENSIONS") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.vector_dimensions = dimensions.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid vector dimensions value: {dimensions}"))
            })?;
        }

        if let Ok(distance_metric) = env::var("QDRANT_DISTANCE_METRIC") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.distance_metric = distance_metric;
        }

        // Vector health check configuration
        if let Ok(enabled) = env::var("QDRANT_HEALTH_CHECK_ENABLED") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.health_check.enabled = enabled.parse().map_err(|_| {
                ConfigError::InvalidValue(format!("Invalid health check enabled value: {enabled}"))
            })?;
        }

        if let Ok(interval) = env::var("QDRANT_HEALTH_CHECK_INTERVAL_SECONDS") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.health_check.interval_seconds = interval.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid health check interval value: {interval}"
                ))
            })?;
        }

        if let Ok(max_failures) = env::var("QDRANT_HEALTH_CHECK_MAX_FAILURES") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.health_check.max_failures = max_failures.parse().map_err(|_| {
                ConfigError::InvalidValue(format!(
                    "Invalid health check max failures value: {max_failures}"
                ))
            })?;
        }

        // Vector connection pool configuration
        if let Ok(max_connections) = env::var("QDRANT_MAX_CONNECTIONS") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.connection_pool.max_connections =
                max_connections.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid max connections value: {max_connections}"
                    ))
                })?;
        }

        if let Ok(idle_timeout) = env::var("QDRANT_IDLE_TIMEOUT_SECONDS") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.connection_pool.idle_timeout_seconds =
                idle_timeout.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!("Invalid idle timeout value: {idle_timeout}"))
                })?;
        }

        if let Ok(connection_timeout) = env::var("QDRANT_CONNECTION_TIMEOUT_SECONDS") {
            let vector_config = self
                .vector
                .get_or_insert_with(VectorDatabaseConfig::default);
            vector_config.connection_pool.connection_timeout_seconds =
                connection_timeout.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!(
                        "Invalid connection timeout value: {connection_timeout}"
                    ))
                })?;
        }

        Ok(())
    }

    /// Load configuration from a JSON file
    fn load_from_file(&mut self, path: &PathBuf) -> Result<(), ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let file_config: Config = serde_json::from_str(&contents)?;

        // Merge configurations (file config takes precedence)
        self.merge_with(file_config);

        Ok(())
    }

    /// Merge with another configuration
    fn merge_with(&mut self, other: Config) {
        // Merge Claude configuration
        if let Some(other_claude) = other.claude {
            if let Some(ref mut claude) = self.claude {
                // Merge existing Claude config
                if !other_claude.api_key.is_empty() {
                    claude.api_key = other_claude.api_key;
                }
                if other_claude.base_url.is_some() {
                    claude.base_url = other_claude.base_url;
                }
                if other_claude.timeout_seconds.is_some() {
                    claude.timeout_seconds = other_claude.timeout_seconds;
                }
                if other_claude.model.is_some() {
                    claude.model = other_claude.model;
                }
                claude.rate_limit = other_claude.rate_limit;
                claude.retry = other_claude.retry;
            } else {
                // Set Claude config
                self.claude = Some(other_claude);
            }
        }

        // Merge other configurations
        self.storage = other.storage;
        self.classification = other.classification;
        self.pipeline = other.pipeline;
        self.logging = other.logging;

        // Merge vector configuration
        if other.vector.is_some() {
            self.vector = other.vector;
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate Claude configuration if present
        if let Some(ref claude) = self.claude {
            if claude.api_key.is_empty() {
                return Err(ConfigError::MissingRequired("claude.api_key".to_string()));
            }

            if !claude.api_key.starts_with("sk-") {
                return Err(ConfigError::InvalidValue(
                    "claude.api_key must start with 'sk-'".to_string(),
                ));
            }

            if let Some(timeout) = claude.timeout_seconds {
                if timeout == 0 {
                    return Err(ConfigError::InvalidValue(
                        "claude.timeout_seconds must be greater than 0".to_string(),
                    ));
                }
            }

            // Validate rate limiting
            if claude.rate_limit.requests_per_minute == 0 {
                return Err(ConfigError::InvalidValue(
                    "claude.rate_limit.requests_per_minute must be greater than 0".to_string(),
                ));
            }

            if claude.rate_limit.input_tokens_per_minute == 0 {
                return Err(ConfigError::InvalidValue(
                    "claude.rate_limit.input_tokens_per_minute must be greater than 0".to_string(),
                ));
            }

            if claude.rate_limit.output_tokens_per_minute == 0 {
                return Err(ConfigError::InvalidValue(
                    "claude.rate_limit.output_tokens_per_minute must be greater than 0".to_string(),
                ));
            }

            if claude.rate_limit.max_concurrent_requests == 0 {
                return Err(ConfigError::InvalidValue(
                    "claude.rate_limit.max_concurrent_requests must be greater than 0".to_string(),
                ));
            }

            // Validate retry configuration
            if claude.retry.max_retries > 10 {
                return Err(ConfigError::InvalidValue(
                    "claude.retry.max_retries should not exceed 10".to_string(),
                ));
            }

            if claude.retry.initial_delay_ms == 0 {
                return Err(ConfigError::InvalidValue(
                    "claude.retry.initial_delay_ms must be greater than 0".to_string(),
                ));
            }

            if claude.retry.max_delay_ms <= claude.retry.initial_delay_ms {
                return Err(ConfigError::InvalidValue(
                    "claude.retry.max_delay_ms must be greater than initial_delay_ms".to_string(),
                ));
            }

            if claude.retry.backoff_multiplier <= 1.0 {
                return Err(ConfigError::InvalidValue(
                    "claude.retry.backoff_multiplier must be greater than 1.0".to_string(),
                ));
            }
        }

        // Validate storage configuration
        if self.storage.cache_expiration_seconds == 0 {
            return Err(ConfigError::InvalidValue(
                "storage.cache_expiration_seconds must be greater than 0".to_string(),
            ));
        }

        if self.storage.max_cache_size_bytes == 0 {
            return Err(ConfigError::InvalidValue(
                "storage.max_cache_size_bytes must be greater than 0".to_string(),
            ));
        }

        // Validate classification configuration
        if self.classification.default_threshold < 0.0
            || self.classification.default_threshold > 1.0
        {
            return Err(ConfigError::InvalidValue(
                "classification.default_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate context detection configuration
        if self.classification.context_detection.confidence_threshold < 0.0
            || self.classification.context_detection.confidence_threshold > 1.0
        {
            return Err(ConfigError::InvalidValue(
                "context_detection.confidence_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.classification.context_detection.max_processing_time_ms == 0 {
            return Err(ConfigError::InvalidValue(
                "context_detection.max_processing_time_ms must be greater than 0".to_string(),
            ));
        }

        // Validate advanced classification configuration
        if self
            .classification
            .advanced_classification
            .max_processing_time_ms
            == 0
        {
            return Err(ConfigError::InvalidValue(
                "advanced_classification.max_processing_time_ms must be greater than 0".to_string(),
            ));
        }

        // Validate signal composition configuration
        if self
            .classification
            .advanced_classification
            .signal_composition
            .confidence_threshold
            < 0.0
            || self
                .classification
                .advanced_classification
                .signal_composition
                .confidence_threshold
                > 1.0
        {
            return Err(ConfigError::InvalidValue(
                "signal_composition.confidence_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self
            .classification
            .advanced_classification
            .signal_composition
            .max_signals
            == 0
        {
            return Err(ConfigError::InvalidValue(
                "signal_composition.max_signals must be greater than 0".to_string(),
            ));
        }

        // Validate contextual weights configuration
        if self
            .classification
            .advanced_classification
            .contextual_weights
            .urgency_boost
            < 0.0
        {
            return Err(ConfigError::InvalidValue(
                "contextual_weights.urgency_boost must be greater than or equal to 0.0".to_string(),
            ));
        }

        if self
            .classification
            .advanced_classification
            .contextual_weights
            .beginner_boost
            < 0.0
        {
            return Err(ConfigError::InvalidValue(
                "contextual_weights.beginner_boost must be greater than or equal to 0.0"
                    .to_string(),
            ));
        }

        if self
            .classification
            .advanced_classification
            .contextual_weights
            .domain_boost
            < 0.0
        {
            return Err(ConfigError::InvalidValue(
                "contextual_weights.domain_boost must be greater than or equal to 0.0".to_string(),
            ));
        }

        if self
            .classification
            .advanced_classification
            .contextual_weights
            .low_confidence_penalty
            < 0.0
        {
            return Err(ConfigError::InvalidValue(
                "contextual_weights.low_confidence_penalty must be greater than or equal to 0.0"
                    .to_string(),
            ));
        }

        // Validate pipeline configuration
        if self.pipeline.max_parallel_requests == 0 {
            return Err(ConfigError::InvalidValue(
                "pipeline.max_parallel_requests must be greater than 0".to_string(),
            ));
        }

        if self.pipeline.processing_timeout_seconds == 0 {
            return Err(ConfigError::InvalidValue(
                "pipeline.processing_timeout_seconds must be greater than 0".to_string(),
            ));
        }

        // Validate logging configuration
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            let joined_levels = valid_levels.join(", ");
            return Err(ConfigError::InvalidValue(format!(
                "logging.level must be one of: {joined_levels}"
            )));
        }

        // Validate vector database configuration if present
        if let Some(ref vector) = self.vector {
            if vector.url.is_empty() {
                return Err(ConfigError::InvalidValue(
                    "vector.url cannot be empty".to_string(),
                ));
            }

            if vector.timeout_seconds == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.timeout_seconds must be greater than 0".to_string(),
                ));
            }

            if vector.default_collection.is_empty() {
                return Err(ConfigError::InvalidValue(
                    "vector.default_collection cannot be empty".to_string(),
                ));
            }

            if vector.vector_dimensions == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.vector_dimensions must be greater than 0".to_string(),
                ));
            }

            let valid_metrics = ["cosine", "euclidean", "dot"];
            if !valid_metrics.contains(&vector.distance_metric.as_str()) {
                let joined_metrics = valid_metrics.join(", ");
                return Err(ConfigError::InvalidValue(format!(
                    "vector.distance_metric must be one of: {joined_metrics}"
                )));
            }

            // Validate health check configuration
            if vector.health_check.interval_seconds == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.health_check.interval_seconds must be greater than 0".to_string(),
                ));
            }

            if vector.health_check.max_failures == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.health_check.max_failures must be greater than 0".to_string(),
                ));
            }

            if vector.health_check.timeout_seconds == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.health_check.timeout_seconds must be greater than 0".to_string(),
                ));
            }

            // Validate connection pool configuration
            if vector.connection_pool.max_connections == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.connection_pool.max_connections must be greater than 0".to_string(),
                ));
            }

            if vector.connection_pool.idle_timeout_seconds == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.connection_pool.idle_timeout_seconds must be greater than 0"
                        .to_string(),
                ));
            }

            if vector.connection_pool.connection_timeout_seconds == 0 {
                return Err(ConfigError::InvalidValue(
                    "vector.connection_pool.connection_timeout_seconds must be greater than 0"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get Claude configuration, creating a default if needed
    pub fn get_claude_config(&self) -> Result<ClaudeConfig, ConfigError> {
        self.claude.clone().ok_or_else(|| {
            ConfigError::MissingRequired("Claude API configuration is required".to_string())
        })
    }

    /// Check if Claude API is configured
    pub fn has_claude_config(&self) -> bool {
        self.claude.is_some() && !self.claude.as_ref().unwrap().api_key.is_empty()
    }

    /// Get vector database configuration, creating a default if needed
    #[allow(dead_code)] // TODO: Use in vector configuration management
    pub fn get_vector_config(&self) -> Result<VectorDatabaseConfig, ConfigError> {
        self.vector.clone().ok_or_else(|| {
            ConfigError::MissingRequired("Vector database configuration is required".to_string())
        })
    }

    /// Check if vector database is configured
    #[allow(dead_code)] // TODO: Use in vector configuration checks
    pub fn has_vector_config(&self) -> bool {
        self.vector.is_some()
    }

    /// Save configuration to a JSON file
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Generate a sample configuration file
    pub fn generate_sample() -> String {
        let sample_config = Config {
            claude: Some(ClaudeConfig {
                api_key: "sk-your-api-key-here".to_string(),
                base_url: None,
                timeout_seconds: Some(300),
                model: Some("claude-3-sonnet-20240229".to_string()),
                rate_limit: RateLimitConfig::default(),
                retry: RetryConfig::default(),
            }),
            storage: StorageConfig::default(),
            classification: ClassificationConfig {
                enable_advanced: true,
                enable_context_detection: true,
                context_detection: ContextDetectionConfig {
                    confidence_threshold: 0.6,
                    enable_fallback: true,
                    max_processing_time_ms: 1000,
                    debug_logging: false,
                },
                advanced_classification: AdvancedClassificationConfig {
                    enable_graceful_degradation: true,
                    max_processing_time_ms: 1000,
                    signal_composition: SignalCompositionConfig {
                        confidence_threshold: 0.5,
                        enable_fallback: true,
                        max_signals: 10,
                    },
                    contextual_weights: ContextualWeights {
                        urgency_boost: 1.3,
                        beginner_boost: 1.2,
                        domain_boost: 1.1,
                        low_confidence_penalty: 0.8,
                    },
                },
                ..ClassificationConfig::default()
            },
            pipeline: PipelineConfig::default(),
            logging: LoggingConfig::default(),
            vector: Some(VectorDatabaseConfig::default()),
        };

        serde_json::to_string_pretty(&sample_config).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.claude.is_none());
        assert_eq!(
            config.storage.base_path,
            PathBuf::from("./reference_library")
        );
        assert_eq!(config.classification.default_threshold, 0.1);
        assert!(config.pipeline.enable_caching);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Should pass validation with default values
        assert!(config.validate().is_ok());

        // Should fail with empty API key
        config.claude = Some(ClaudeConfig {
            api_key: "".to_string(),
            ..ClaudeConfig::default()
        });
        assert!(config.validate().is_err());

        // Should fail with invalid API key format
        config.claude = Some(ClaudeConfig {
            api_key: "invalid-key".to_string(),
            ..ClaudeConfig::default()
        });
        assert!(config.validate().is_err());

        // Should pass with valid API key
        config.claude = Some(ClaudeConfig {
            api_key: "sk-valid-key".to_string(),
            ..ClaudeConfig::default()
        });
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_env_var_loading() {
        // Set environment variables
        env::set_var("CLAUDE_API_KEY", "sk-test-key");
        env::set_var("CLAUDE_MODEL", "claude-3-sonnet-20240229");
        env::set_var("FORTITUDE_DATA_DIR", "/tmp/test");
        env::set_var("FORTITUDE_LOG_LEVEL", "debug");

        let mut config = Config::default();
        config.load_from_env().unwrap();

        assert!(config.claude.is_some());
        let claude = config.claude.unwrap();
        assert_eq!(claude.api_key, "sk-test-key");
        assert_eq!(claude.model, Some("claude-3-sonnet-20240229".to_string()));
        assert_eq!(config.storage.base_path, PathBuf::from("/tmp/test"));
        assert_eq!(config.logging.level, "debug");

        // Clean up
        env::remove_var("CLAUDE_API_KEY");
        env::remove_var("CLAUDE_MODEL");
        env::remove_var("FORTITUDE_DATA_DIR");
        env::remove_var("FORTITUDE_LOG_LEVEL");
    }

    #[test]
    fn test_has_claude_config() {
        let mut config = Config::default();
        assert!(!config.has_claude_config());

        config.claude = Some(ClaudeConfig {
            api_key: "sk-test-key".to_string(),
            ..ClaudeConfig::default()
        });
        assert!(config.has_claude_config());

        config.claude = Some(ClaudeConfig {
            api_key: "".to_string(),
            ..ClaudeConfig::default()
        });
        assert!(!config.has_claude_config());
    }

    #[test]
    fn test_sample_generation() {
        let sample = Config::generate_sample();
        assert!(sample.contains("sk-your-api-key-here"));
        assert!(sample.contains("claude-3-sonnet-20240229"));

        // Should be valid JSON
        let _: Config = serde_json::from_str(&sample).unwrap();
    }
}
