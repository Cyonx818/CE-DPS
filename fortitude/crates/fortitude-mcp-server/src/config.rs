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

// ABOUTME: Configuration management for Fortitude MCP server
// Handles server configuration loading from files and environment variables
// Supports validation and default values for production deployment

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use validator::Validate;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    /// Server port
    #[validate(range(min = 1024, max = 65535))]
    pub port: u16,

    /// Server host address
    pub host: String,

    /// Maximum number of concurrent connections
    #[validate(range(min = 1, max = 10000))]
    pub max_connections: u32,

    /// Request timeout in seconds
    #[validate(range(min = 1, max = 300))]
    pub request_timeout: u64,

    /// Authentication configuration
    #[validate(nested)]
    pub auth: AuthConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Performance settings
    #[validate(nested)]
    pub performance: PerformanceConfig,

    /// Security settings
    #[validate(nested)]
    pub security: SecurityConfig,

    /// Integration settings for fortitude-core
    #[validate(nested)]
    pub integration: IntegrationConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthConfig {
    /// JWT secret key
    #[validate(length(min = 32))]
    pub jwt_secret: String,

    /// Token expiration time in hours
    #[validate(range(min = 1, max = 168))] // 1 hour to 1 week
    pub token_expiration_hours: u64,

    /// Enable authentication (disable for development)
    pub enabled: bool,

    /// Rate limiting configuration
    #[validate(nested)]
    pub rate_limit: RateLimitConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    #[validate(range(min = 1, max = 10000))]
    pub max_requests_per_minute: u32,

    /// Time window in seconds
    #[validate(range(min = 1, max = 3600))]
    pub window_seconds: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,

    /// Enable structured logging
    pub structured: bool,

    /// Log file path (optional)
    pub file_path: Option<String>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceConfig {
    /// Cache size for responses
    #[validate(range(min = 100, max = 100000))]
    pub cache_size: u32,

    /// Cache TTL in seconds
    #[validate(range(min = 60, max = 3600))]
    pub cache_ttl: u64,

    /// Enable request deduplication
    pub enable_deduplication: bool,

    /// Maximum concurrent connections
    #[validate(range(min = 1, max = 10000))]
    pub max_concurrent_connections: u32,

    /// Connection timeout in seconds
    #[validate(range(min = 1, max = 300))]
    pub connection_timeout_seconds: u64,

    /// Enable HTTP/2 support
    pub enable_http2: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,

    /// Enable HTTPS redirect
    pub force_https: bool,

    /// Enable request validation
    pub enable_request_validation: bool,

    /// Maximum request size in bytes
    #[validate(range(min = 1024, max = 10_485_760))] // 1KB to 10MB
    pub max_request_size: u64,

    /// Security headers configuration
    pub security_headers: SecurityHeaders,

    /// IP whitelist (empty means allow all)
    pub ip_whitelist: Vec<String>,

    /// Enable intrusion detection
    pub enable_intrusion_detection: bool,
}

/// Security headers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeaders {
    /// Enable X-Frame-Options header
    pub x_frame_options: bool,

    /// Enable X-Content-Type-Options header
    pub x_content_type_options: bool,

    /// Enable X-XSS-Protection header
    pub x_xss_protection: bool,

    /// Enable Strict-Transport-Security header
    pub strict_transport_security: bool,

    /// Content Security Policy
    pub content_security_policy: Option<String>,
}

/// Integration configuration with fortitude-core
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct IntegrationConfig {
    /// Fortitude core data directory
    pub fortitude_data_dir: PathBuf,

    /// Enable research pipeline integration
    pub enable_research_pipeline: bool,

    /// Enable reference library access
    pub enable_reference_library: bool,

    /// Enable classification system integration
    pub enable_classification: bool,

    /// Classification confidence threshold
    #[validate(range(min = 0.0, max = 1.0))]
    pub classification_threshold: f64,

    /// Enable context detection
    pub enable_context_detection: bool,

    /// Context detection timeout in milliseconds
    #[validate(range(min = 100, max = 10000))]
    pub context_detection_timeout_ms: u64,

    /// Enable caching for research queries
    pub enable_research_caching: bool,

    /// Research cache TTL in seconds
    #[validate(range(min = 60, max = 86400))]
    pub research_cache_ttl: u64,

    /// Enable pattern tracking for MCP interactions
    pub enable_pattern_tracking: Option<bool>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_connections: 1000,
            request_timeout: 30,
            auth: AuthConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            window_seconds: 60,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: generate_default_secret(),
            token_expiration_hours: 24,
            enabled: true,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: true,
            file_path: None,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            cache_ttl: 300,
            enable_deduplication: true,
            max_concurrent_connections: 1000,
            connection_timeout_seconds: 30,
            enable_http2: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            force_https: false,
            enable_request_validation: true,
            max_request_size: 1_048_576, // 1MB
            security_headers: SecurityHeaders::default(),
            ip_whitelist: Vec::new(),
            enable_intrusion_detection: false,
        }
    }
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            x_frame_options: true,
            x_content_type_options: true,
            x_xss_protection: true,
            strict_transport_security: false,
            content_security_policy: None,
        }
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            fortitude_data_dir: PathBuf::from("./reference_library"),
            enable_research_pipeline: true,
            enable_reference_library: true,
            enable_classification: true,
            classification_threshold: 0.7,
            enable_context_detection: true,
            context_detection_timeout_ms: 1000,
            enable_research_caching: true,
            research_cache_ttl: 3600,
            enable_pattern_tracking: Some(true),
        }
    }
}

impl ServerConfig {
    /// Load configuration from file (legacy JSON support)
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file_with_format(path).await
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Override with environment variables
        if let Ok(port) = std::env::var("MCP_SERVER_PORT") {
            config.port = port
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SERVER_PORT: {}", e))?;
        }

        if let Ok(host) = std::env::var("MCP_SERVER_HOST") {
            config.host = host;
        }

        if let Ok(max_conn) = std::env::var("MCP_MAX_CONNECTIONS") {
            config.max_connections = max_conn
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_MAX_CONNECTIONS: {}", e))?;
        }

        if let Ok(timeout) = std::env::var("MCP_REQUEST_TIMEOUT") {
            config.request_timeout = timeout
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_REQUEST_TIMEOUT: {}", e))?;
        }

        if let Ok(secret) = std::env::var("MCP_JWT_SECRET") {
            config.auth.jwt_secret = secret;
        }

        if let Ok(auth_enabled) = std::env::var("MCP_AUTH_ENABLED") {
            config.auth.enabled = auth_enabled
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_AUTH_ENABLED: {}", e))?;
        }

        if let Ok(log_level) = std::env::var("MCP_LOG_LEVEL") {
            config.logging.level = log_level;
        }

        if let Ok(rate_limit) = std::env::var("MCP_RATE_LIMIT_MAX_REQUESTS") {
            config.auth.rate_limit.max_requests_per_minute = rate_limit
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_RATE_LIMIT_MAX_REQUESTS: {}", e))?;
        }

        if let Ok(rate_limit_window) = std::env::var("MCP_RATE_LIMIT_WINDOW_SECONDS") {
            config.auth.rate_limit.window_seconds = rate_limit_window
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_RATE_LIMIT_WINDOW_SECONDS: {}", e))?;
        }

        // Performance configuration
        if let Ok(cache_size) = std::env::var("MCP_PERFORMANCE_CACHE_SIZE") {
            config.performance.cache_size = cache_size
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_PERFORMANCE_CACHE_SIZE: {}", e))?;
        }

        if let Ok(cache_ttl) = std::env::var("MCP_PERFORMANCE_CACHE_TTL") {
            config.performance.cache_ttl = cache_ttl
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_PERFORMANCE_CACHE_TTL: {}", e))?;
        }

        if let Ok(enable_dedup) = std::env::var("MCP_PERFORMANCE_ENABLE_DEDUPLICATION") {
            config.performance.enable_deduplication = enable_dedup
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_PERFORMANCE_ENABLE_DEDUPLICATION: {}", e))?;
        }

        if let Ok(max_concurrent) = std::env::var("MCP_PERFORMANCE_MAX_CONCURRENT_CONNECTIONS") {
            config.performance.max_concurrent_connections =
                max_concurrent.parse().map_err(|e| {
                    anyhow!("Invalid MCP_PERFORMANCE_MAX_CONCURRENT_CONNECTIONS: {}", e)
                })?;
        }

        if let Ok(conn_timeout) = std::env::var("MCP_PERFORMANCE_CONNECTION_TIMEOUT_SECONDS") {
            config.performance.connection_timeout_seconds = conn_timeout.parse().map_err(|e| {
                anyhow!("Invalid MCP_PERFORMANCE_CONNECTION_TIMEOUT_SECONDS: {}", e)
            })?;
        }

        if let Ok(enable_http2) = std::env::var("MCP_PERFORMANCE_ENABLE_HTTP2") {
            config.performance.enable_http2 = enable_http2
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_PERFORMANCE_ENABLE_HTTP2: {}", e))?;
        }

        // Security configuration
        if let Ok(allowed_origins) = std::env::var("MCP_SECURITY_ALLOWED_ORIGINS") {
            config.security.allowed_origins = allowed_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        if let Ok(force_https) = std::env::var("MCP_SECURITY_FORCE_HTTPS") {
            config.security.force_https = force_https
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_FORCE_HTTPS: {}", e))?;
        }

        if let Ok(enable_validation) = std::env::var("MCP_SECURITY_ENABLE_REQUEST_VALIDATION") {
            config.security.enable_request_validation = enable_validation
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_ENABLE_REQUEST_VALIDATION: {}", e))?;
        }

        if let Ok(max_request_size) = std::env::var("MCP_SECURITY_MAX_REQUEST_SIZE") {
            config.security.max_request_size = max_request_size
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_MAX_REQUEST_SIZE: {}", e))?;
        }

        if let Ok(ip_whitelist) = std::env::var("MCP_SECURITY_IP_WHITELIST") {
            config.security.ip_whitelist = ip_whitelist
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        if let Ok(enable_intrusion) = std::env::var("MCP_SECURITY_ENABLE_INTRUSION_DETECTION") {
            config.security.enable_intrusion_detection = enable_intrusion
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_ENABLE_INTRUSION_DETECTION: {}", e))?;
        }

        // Security headers
        if let Ok(x_frame_options) = std::env::var("MCP_SECURITY_X_FRAME_OPTIONS") {
            config.security.security_headers.x_frame_options = x_frame_options
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_X_FRAME_OPTIONS: {}", e))?;
        }

        if let Ok(x_content_type) = std::env::var("MCP_SECURITY_X_CONTENT_TYPE_OPTIONS") {
            config.security.security_headers.x_content_type_options = x_content_type
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_X_CONTENT_TYPE_OPTIONS: {}", e))?;
        }

        if let Ok(x_xss_protection) = std::env::var("MCP_SECURITY_X_XSS_PROTECTION") {
            config.security.security_headers.x_xss_protection = x_xss_protection
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_SECURITY_X_XSS_PROTECTION: {}", e))?;
        }

        if let Ok(strict_transport) = std::env::var("MCP_SECURITY_STRICT_TRANSPORT_SECURITY") {
            config.security.security_headers.strict_transport_security =
                strict_transport.parse().map_err(|e| {
                    anyhow!("Invalid MCP_SECURITY_STRICT_TRANSPORT_SECURITY: {}", e)
                })?;
        }

        if let Ok(csp) = std::env::var("MCP_SECURITY_CONTENT_SECURITY_POLICY") {
            config.security.security_headers.content_security_policy =
                if csp.is_empty() { None } else { Some(csp) };
        }

        // Integration configuration
        if let Ok(data_dir) = std::env::var("MCP_INTEGRATION_FORTITUDE_DATA_DIR") {
            config.integration.fortitude_data_dir = PathBuf::from(data_dir);
        }

        if let Ok(enable_research) = std::env::var("MCP_INTEGRATION_ENABLE_RESEARCH_PIPELINE") {
            config.integration.enable_research_pipeline = enable_research
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_ENABLE_RESEARCH_PIPELINE: {}", e))?;
        }

        if let Ok(enable_ref_lib) = std::env::var("MCP_INTEGRATION_ENABLE_REFERENCE_LIBRARY") {
            config.integration.enable_reference_library = enable_ref_lib
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_ENABLE_REFERENCE_LIBRARY: {}", e))?;
        }

        if let Ok(enable_classification) = std::env::var("MCP_INTEGRATION_ENABLE_CLASSIFICATION") {
            config.integration.enable_classification = enable_classification
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_ENABLE_CLASSIFICATION: {}", e))?;
        }

        if let Ok(classification_threshold) =
            std::env::var("MCP_INTEGRATION_CLASSIFICATION_THRESHOLD")
        {
            config.integration.classification_threshold = classification_threshold
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_CLASSIFICATION_THRESHOLD: {}", e))?;
        }

        if let Ok(enable_context) = std::env::var("MCP_INTEGRATION_ENABLE_CONTEXT_DETECTION") {
            config.integration.enable_context_detection = enable_context
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_ENABLE_CONTEXT_DETECTION: {}", e))?;
        }

        if let Ok(context_timeout) = std::env::var("MCP_INTEGRATION_CONTEXT_DETECTION_TIMEOUT_MS") {
            config.integration.context_detection_timeout_ms =
                context_timeout.parse().map_err(|e| {
                    anyhow!(
                        "Invalid MCP_INTEGRATION_CONTEXT_DETECTION_TIMEOUT_MS: {}",
                        e
                    )
                })?;
        }

        if let Ok(enable_caching) = std::env::var("MCP_INTEGRATION_ENABLE_RESEARCH_CACHING") {
            config.integration.enable_research_caching = enable_caching
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_ENABLE_RESEARCH_CACHING: {}", e))?;
        }

        if let Ok(cache_ttl) = std::env::var("MCP_INTEGRATION_RESEARCH_CACHE_TTL") {
            config.integration.research_cache_ttl = cache_ttl
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_INTEGRATION_RESEARCH_CACHE_TTL: {}", e))?;
        }

        if let Ok(enable_pattern_tracking) =
            std::env::var("MCP_INTEGRATION_ENABLE_PATTERN_TRACKING")
        {
            config.integration.enable_pattern_tracking =
                Some(enable_pattern_tracking.parse().map_err(|e| {
                    anyhow!("Invalid MCP_INTEGRATION_ENABLE_PATTERN_TRACKING: {}", e)
                })?);
        }

        // Logging configuration extensions
        if let Ok(structured) = std::env::var("MCP_LOG_STRUCTURED") {
            config.logging.structured = structured
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_LOG_STRUCTURED: {}", e))?;
        }

        if let Ok(file_path) = std::env::var("MCP_LOG_FILE_PATH") {
            config.logging.file_path = if file_path.is_empty() {
                None
            } else {
                Some(file_path)
            };
        }

        // Token expiration configuration
        if let Ok(token_expiry) = std::env::var("MCP_AUTH_TOKEN_EXPIRATION_HOURS") {
            config.auth.token_expiration_hours = token_expiry
                .parse()
                .map_err(|e| anyhow!("Invalid MCP_AUTH_TOKEN_EXPIRATION_HOURS: {}", e))?;
        }

        config
            .validate()
            .map_err(|e| anyhow!("Config validation failed: {}", e))?;

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        Validate::validate(self).map_err(|e| anyhow!("Config validation failed: {}", e))?;

        // Additional custom validations
        if self.auth.enabled && self.auth.jwt_secret.len() < 32 {
            return Err(anyhow!(
                "JWT secret must be at least 32 characters when auth is enabled"
            ));
        }

        // Validate security configuration
        if self.security.max_request_size == 0 {
            return Err(anyhow!("Security max_request_size must be greater than 0"));
        }

        if self.security.max_request_size > 100 * 1024 * 1024 {
            return Err(anyhow!("Security max_request_size should not exceed 100MB"));
        }

        // Validate performance configuration
        if self.performance.max_concurrent_connections == 0 {
            return Err(anyhow!(
                "Performance max_concurrent_connections must be greater than 0"
            ));
        }

        if self.performance.connection_timeout_seconds == 0 {
            return Err(anyhow!(
                "Performance connection_timeout_seconds must be greater than 0"
            ));
        }

        // Validate integration configuration
        if self.integration.classification_threshold < 0.0
            || self.integration.classification_threshold > 1.0
        {
            return Err(anyhow!(
                "Integration classification_threshold must be between 0.0 and 1.0"
            ));
        }

        if self.integration.context_detection_timeout_ms == 0 {
            return Err(anyhow!(
                "Integration context_detection_timeout_ms must be greater than 0"
            ));
        }

        if self.integration.research_cache_ttl == 0 {
            return Err(anyhow!(
                "Integration research_cache_ttl must be greater than 0"
            ));
        }

        // Validate data directory exists or can be created
        if !self.integration.fortitude_data_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&self.integration.fortitude_data_dir) {
                return Err(anyhow!("Failed to create fortitude data directory: {}", e));
            }
        }

        // Validate IP whitelist format
        for ip in &self.security.ip_whitelist {
            if !ip.is_empty() && !is_valid_ip_or_cidr(ip) {
                return Err(anyhow!("Invalid IP address or CIDR in whitelist: {}", ip));
            }
        }

        Ok(())
    }

    /// Save configuration to file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(self)
                .map_err(|e| anyhow!("Failed to serialize config to TOML: {}", e))?
        } else {
            serde_json::to_string_pretty(self)
                .map_err(|e| anyhow!("Failed to serialize config to JSON: {}", e))?
        };

        fs::write(path, content)
            .await
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load configuration from file (supports both JSON and TOML)
    pub async fn from_file_with_format<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Self = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse TOML config file: {}", e))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse JSON config file: {}", e))?
        };

        config
            .validate()
            .map_err(|e| anyhow!("Config validation failed: {}", e))?;

        Ok(config)
    }

    /// Merge configuration from environment variables and file
    pub async fn load_with_env_override<P: AsRef<Path>>(config_path: Option<P>) -> Result<Self> {
        let mut config = if let Some(path) = config_path {
            Self::from_file_with_format(path).await?
        } else {
            Self::default()
        };

        // Override with environment variables
        let env_config = Self::from_env()?;
        config.merge_with(env_config);

        Ok(config)
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge_with(&mut self, other: Self) {
        // Basic server settings
        if other.port != 8080 {
            self.port = other.port;
        }
        if other.host != "127.0.0.1" {
            self.host = other.host;
        }
        if other.max_connections != 1000 {
            self.max_connections = other.max_connections;
        }
        if other.request_timeout != 30 {
            self.request_timeout = other.request_timeout;
        }

        // Merge auth configuration
        self.auth.merge_with(other.auth);

        // Merge logging configuration
        self.logging.merge_with(other.logging);

        // Merge performance configuration
        self.performance.merge_with(other.performance);

        // Merge security configuration
        self.security.merge_with(other.security);

        // Merge integration configuration
        self.integration.merge_with(other.integration);
    }

    /// Get environment variable documentation
    pub fn get_env_var_documentation() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "MCP_SERVER_HOST",
                "Server host address (default: 127.0.0.1)",
            ),
            ("MCP_SERVER_PORT", "Server port (default: 8080)"),
            (
                "MCP_MAX_CONNECTIONS",
                "Maximum concurrent connections (default: 1000)",
            ),
            (
                "MCP_REQUEST_TIMEOUT",
                "Request timeout in seconds (default: 30)",
            ),
            ("MCP_JWT_SECRET", "JWT secret key (minimum 32 characters)"),
            ("MCP_AUTH_ENABLED", "Enable authentication (default: true)"),
            (
                "MCP_AUTH_TOKEN_EXPIRATION_HOURS",
                "Token expiration in hours (default: 24)",
            ),
            (
                "MCP_RATE_LIMIT_MAX_REQUESTS",
                "Max requests per minute (default: 60)",
            ),
            (
                "MCP_RATE_LIMIT_WINDOW_SECONDS",
                "Rate limit window in seconds (default: 60)",
            ),
            ("MCP_LOG_LEVEL", "Log level (default: info)"),
            (
                "MCP_LOG_STRUCTURED",
                "Enable structured logging (default: true)",
            ),
            ("MCP_LOG_FILE_PATH", "Log file path (optional)"),
            (
                "MCP_PERFORMANCE_CACHE_SIZE",
                "Cache size for responses (default: 1000)",
            ),
            (
                "MCP_PERFORMANCE_CACHE_TTL",
                "Cache TTL in seconds (default: 300)",
            ),
            (
                "MCP_PERFORMANCE_ENABLE_DEDUPLICATION",
                "Enable request deduplication (default: true)",
            ),
            (
                "MCP_PERFORMANCE_MAX_CONCURRENT_CONNECTIONS",
                "Max concurrent connections (default: 1000)",
            ),
            (
                "MCP_PERFORMANCE_CONNECTION_TIMEOUT_SECONDS",
                "Connection timeout in seconds (default: 30)",
            ),
            (
                "MCP_PERFORMANCE_ENABLE_HTTP2",
                "Enable HTTP/2 support (default: true)",
            ),
            (
                "MCP_SECURITY_ALLOWED_ORIGINS",
                "Allowed origins for CORS (comma-separated, default: *)",
            ),
            (
                "MCP_SECURITY_FORCE_HTTPS",
                "Force HTTPS redirect (default: false)",
            ),
            (
                "MCP_SECURITY_ENABLE_REQUEST_VALIDATION",
                "Enable request validation (default: true)",
            ),
            (
                "MCP_SECURITY_MAX_REQUEST_SIZE",
                "Max request size in bytes (default: 1048576)",
            ),
            (
                "MCP_SECURITY_IP_WHITELIST",
                "IP whitelist (comma-separated, empty means allow all)",
            ),
            (
                "MCP_SECURITY_ENABLE_INTRUSION_DETECTION",
                "Enable intrusion detection (default: false)",
            ),
            (
                "MCP_SECURITY_X_FRAME_OPTIONS",
                "Enable X-Frame-Options header (default: true)",
            ),
            (
                "MCP_SECURITY_X_CONTENT_TYPE_OPTIONS",
                "Enable X-Content-Type-Options header (default: true)",
            ),
            (
                "MCP_SECURITY_X_XSS_PROTECTION",
                "Enable X-XSS-Protection header (default: true)",
            ),
            (
                "MCP_SECURITY_STRICT_TRANSPORT_SECURITY",
                "Enable Strict-Transport-Security header (default: false)",
            ),
            (
                "MCP_SECURITY_CONTENT_SECURITY_POLICY",
                "Content Security Policy (optional)",
            ),
            (
                "MCP_INTEGRATION_FORTITUDE_DATA_DIR",
                "Fortitude data directory (default: ./reference_library)",
            ),
            (
                "MCP_INTEGRATION_ENABLE_RESEARCH_PIPELINE",
                "Enable research pipeline (default: true)",
            ),
            (
                "MCP_INTEGRATION_ENABLE_REFERENCE_LIBRARY",
                "Enable reference library (default: true)",
            ),
            (
                "MCP_INTEGRATION_ENABLE_CLASSIFICATION",
                "Enable classification system (default: true)",
            ),
            (
                "MCP_INTEGRATION_CLASSIFICATION_THRESHOLD",
                "Classification threshold (default: 0.7)",
            ),
            (
                "MCP_INTEGRATION_ENABLE_CONTEXT_DETECTION",
                "Enable context detection (default: true)",
            ),
            (
                "MCP_INTEGRATION_CONTEXT_DETECTION_TIMEOUT_MS",
                "Context detection timeout in ms (default: 1000)",
            ),
            (
                "MCP_INTEGRATION_ENABLE_RESEARCH_CACHING",
                "Enable research caching (default: true)",
            ),
            (
                "MCP_INTEGRATION_RESEARCH_CACHE_TTL",
                "Research cache TTL in seconds (default: 3600)",
            ),
            (
                "MCP_INTEGRATION_ENABLE_PATTERN_TRACKING",
                "Enable pattern tracking for MCP interactions (default: true)",
            ),
        ]
    }
}

/// Generate a default JWT secret (for development only)
fn generate_default_secret() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    format!("fortitude-mcp-server-default-secret-{timestamp}")
}

/// Validate IP address or CIDR notation
fn is_valid_ip_or_cidr(ip: &str) -> bool {
    // Simple validation - in production, use a proper IP parsing library
    if ip.contains('/') {
        // CIDR notation
        let parts: Vec<&str> = ip.split('/').collect();
        if parts.len() == 2 {
            if let Ok(prefix) = parts[1].parse::<u8>() {
                return prefix <= 32 && is_valid_ip(parts[0]);
            }
        }
        false
    } else {
        is_valid_ip(ip)
    }
}

/// Simple IP address validation
fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}

// Merge implementations for nested configurations
impl AuthConfig {
    pub fn merge_with(&mut self, other: Self) {
        if other.jwt_secret != generate_default_secret() {
            self.jwt_secret = other.jwt_secret;
        }
        if other.token_expiration_hours != 24 {
            self.token_expiration_hours = other.token_expiration_hours;
        }
        if !other.enabled {
            self.enabled = other.enabled;
        }
        self.rate_limit.merge_with(other.rate_limit);
    }
}

impl RateLimitConfig {
    pub fn merge_with(&mut self, other: Self) {
        if other.max_requests_per_minute != 60 {
            self.max_requests_per_minute = other.max_requests_per_minute;
        }
        if other.window_seconds != 60 {
            self.window_seconds = other.window_seconds;
        }
    }
}

impl LoggingConfig {
    pub fn merge_with(&mut self, other: Self) {
        if other.level != "info" {
            self.level = other.level;
        }
        if !other.structured {
            self.structured = other.structured;
        }
        if other.file_path.is_some() {
            self.file_path = other.file_path;
        }
    }
}

impl PerformanceConfig {
    pub fn merge_with(&mut self, other: Self) {
        if other.cache_size != 1000 {
            self.cache_size = other.cache_size;
        }
        if other.cache_ttl != 300 {
            self.cache_ttl = other.cache_ttl;
        }
        if !other.enable_deduplication {
            self.enable_deduplication = other.enable_deduplication;
        }
        if other.max_concurrent_connections != 1000 {
            self.max_concurrent_connections = other.max_concurrent_connections;
        }
        if other.connection_timeout_seconds != 30 {
            self.connection_timeout_seconds = other.connection_timeout_seconds;
        }
        if !other.enable_http2 {
            self.enable_http2 = other.enable_http2;
        }
    }
}

impl SecurityConfig {
    pub fn merge_with(&mut self, other: Self) {
        if other.allowed_origins != vec!["*".to_string()] {
            self.allowed_origins = other.allowed_origins;
        }
        if other.force_https {
            self.force_https = other.force_https;
        }
        if !other.enable_request_validation {
            self.enable_request_validation = other.enable_request_validation;
        }
        if other.max_request_size != 1_048_576 {
            self.max_request_size = other.max_request_size;
        }
        if !other.ip_whitelist.is_empty() {
            self.ip_whitelist = other.ip_whitelist;
        }
        if other.enable_intrusion_detection {
            self.enable_intrusion_detection = other.enable_intrusion_detection;
        }
        self.security_headers.merge_with(other.security_headers);
    }
}

impl SecurityHeaders {
    pub fn merge_with(&mut self, other: Self) {
        if !other.x_frame_options {
            self.x_frame_options = other.x_frame_options;
        }
        if !other.x_content_type_options {
            self.x_content_type_options = other.x_content_type_options;
        }
        if !other.x_xss_protection {
            self.x_xss_protection = other.x_xss_protection;
        }
        if other.strict_transport_security {
            self.strict_transport_security = other.strict_transport_security;
        }
        if other.content_security_policy.is_some() {
            self.content_security_policy = other.content_security_policy;
        }
    }
}

impl IntegrationConfig {
    pub fn merge_with(&mut self, other: Self) {
        if other.fortitude_data_dir != PathBuf::from("./reference_library") {
            self.fortitude_data_dir = other.fortitude_data_dir;
        }
        if !other.enable_research_pipeline {
            self.enable_research_pipeline = other.enable_research_pipeline;
        }
        if !other.enable_reference_library {
            self.enable_reference_library = other.enable_reference_library;
        }
        if !other.enable_classification {
            self.enable_classification = other.enable_classification;
        }
        if other.classification_threshold != 0.7 {
            self.classification_threshold = other.classification_threshold;
        }
        if !other.enable_context_detection {
            self.enable_context_detection = other.enable_context_detection;
        }
        if other.context_detection_timeout_ms != 1000 {
            self.context_detection_timeout_ms = other.context_detection_timeout_ms;
        }
        if !other.enable_research_caching {
            self.enable_research_caching = other.enable_research_caching;
        }
        if other.research_cache_ttl != 3600 {
            self.research_cache_ttl = other.research_cache_ttl;
        }
        if other.enable_pattern_tracking.is_some() && other.enable_pattern_tracking != Some(true) {
            self.enable_pattern_tracking = other.enable_pattern_tracking;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_is_valid() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());

        // Test all sections have proper defaults
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.request_timeout, 30);
        assert!(config.auth.enabled);
        assert_eq!(config.auth.rate_limit.max_requests_per_minute, 60);
        assert_eq!(config.performance.cache_size, 1000);
        assert_eq!(config.performance.cache_ttl, 300);
        assert!(config.performance.enable_deduplication);
        assert_eq!(config.security.allowed_origins, vec!["*".to_string()]);
        assert!(!config.security.force_https);
        assert!(config.security.enable_request_validation);
        assert_eq!(config.integration.classification_threshold, 0.7);
        assert!(config.integration.enable_research_pipeline);
        assert!(config.integration.enable_reference_library);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ServerConfig {
            port: 80, // Too low
            ..Default::default()
        };
        assert!(config.validate().is_err());

        config.port = 8080; // Valid
        assert!(config.validate().is_ok());

        // Test invalid max connections
        config.max_connections = 0; // Too low
        assert!(config.validate().is_err());

        config.max_connections = 1000; // Valid
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_config_file_roundtrip() {
        let original_config = ServerConfig::default();
        let temp_file = NamedTempFile::new().unwrap();

        // Save config
        original_config
            .save_to_file(temp_file.path())
            .await
            .unwrap();

        // Load config
        let loaded_config = ServerConfig::from_file(temp_file.path()).await.unwrap();

        // Compare (excluding JWT secret which is generated)
        assert_eq!(original_config.port, loaded_config.port);
        assert_eq!(original_config.host, loaded_config.host);
        assert_eq!(
            original_config.max_connections,
            loaded_config.max_connections
        );
    }

    #[test]
    fn test_env_config_loading() {
        std::env::set_var("MCP_SERVER_PORT", "9090");
        std::env::set_var("MCP_SERVER_HOST", "0.0.0.0");
        std::env::set_var("MCP_AUTH_ENABLED", "false");
        std::env::set_var("MCP_RATE_LIMIT_MAX_REQUESTS", "120");
        std::env::set_var("MCP_RATE_LIMIT_WINDOW_SECONDS", "30");

        let config = ServerConfig::from_env().unwrap();

        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "0.0.0.0");
        assert!(!config.auth.enabled);
        assert_eq!(config.auth.rate_limit.max_requests_per_minute, 120);
        assert_eq!(config.auth.rate_limit.window_seconds, 30);

        // Clean up
        std::env::remove_var("MCP_SERVER_PORT");
        std::env::remove_var("MCP_SERVER_HOST");
        std::env::remove_var("MCP_AUTH_ENABLED");
        std::env::remove_var("MCP_RATE_LIMIT_MAX_REQUESTS");
        std::env::remove_var("MCP_RATE_LIMIT_WINDOW_SECONDS");
    }

    #[test]
    fn test_rate_limit_config_validation() {
        let mut config = ServerConfig::default();

        // Test invalid max requests (too low)
        config.auth.rate_limit.max_requests_per_minute = 0;
        assert!(config.validate().is_err());

        // Test invalid window seconds (too low)
        config.auth.rate_limit.max_requests_per_minute = 60;
        config.auth.rate_limit.window_seconds = 0;
        assert!(config.validate().is_err());

        // Test valid config
        config.auth.rate_limit.window_seconds = 60;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_security_config_validation() {
        let mut config = ServerConfig::default();

        // Test invalid max request size (too low)
        config.security.max_request_size = 0;
        assert!(config.validate().is_err());

        // Test invalid max request size (too high)
        config.security.max_request_size = 200 * 1024 * 1024; // 200MB
        assert!(config.validate().is_err());

        // Test valid config
        config.security.max_request_size = 5 * 1024 * 1024; // 5MB
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_integration_config_validation() {
        let mut config = ServerConfig::default();

        // Test invalid classification threshold (too low)
        config.integration.classification_threshold = -0.1;
        assert!(config.validate().is_err());

        // Test invalid classification threshold (too high)
        config.integration.classification_threshold = 1.1;
        assert!(config.validate().is_err());

        // Test invalid context detection timeout
        config.integration.classification_threshold = 0.5;
        config.integration.context_detection_timeout_ms = 0;
        assert!(config.validate().is_err());

        // Test valid config
        config.integration.context_detection_timeout_ms = 1000;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ip_whitelist_validation() {
        let mut config = ServerConfig::default();

        // Test valid IP addresses
        config.security.ip_whitelist = vec![
            "127.0.0.1".to_string(),
            "192.168.1.0/24".to_string(),
            "10.0.0.1".to_string(),
        ];
        assert!(config.validate().is_ok());

        // Test invalid IP address
        config.security.ip_whitelist = vec!["127.0.0.1".to_string(), "invalid-ip".to_string()];
        assert!(config.validate().is_err());

        // Test invalid CIDR
        config.security.ip_whitelist = vec![
            "127.0.0.1".to_string(),
            "192.168.1.0/33".to_string(), // Invalid CIDR
        ];
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_toml_config_roundtrip() {
        let original_config = ServerConfig::default();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().with_extension("toml");

        // Save config as TOML
        original_config.save_to_file(&temp_path).await.unwrap();

        // Load config from TOML
        let loaded_config = ServerConfig::from_file_with_format(&temp_path)
            .await
            .unwrap();

        // Compare configurations
        assert_eq!(original_config.port, loaded_config.port);
        assert_eq!(original_config.host, loaded_config.host);
        assert_eq!(
            original_config.max_connections,
            loaded_config.max_connections
        );
        assert_eq!(
            original_config.performance.cache_size,
            loaded_config.performance.cache_size
        );
        assert_eq!(
            original_config.security.max_request_size,
            loaded_config.security.max_request_size
        );
        assert_eq!(
            original_config.integration.classification_threshold,
            loaded_config.integration.classification_threshold
        );

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_comprehensive_env_loading() {
        // Set all environment variables
        std::env::set_var("MCP_SERVER_PORT", "9090");
        std::env::set_var("MCP_SERVER_HOST", "0.0.0.0");
        std::env::set_var("MCP_AUTH_ENABLED", "false");
        std::env::set_var("MCP_PERFORMANCE_CACHE_SIZE", "2000");
        std::env::set_var("MCP_SECURITY_MAX_REQUEST_SIZE", "2097152");
        std::env::set_var("MCP_INTEGRATION_CLASSIFICATION_THRESHOLD", "0.8");
        std::env::set_var(
            "MCP_SECURITY_ALLOWED_ORIGINS",
            "https://example.com,https://test.com",
        );
        std::env::set_var("MCP_SECURITY_IP_WHITELIST", "127.0.0.1,192.168.1.0/24");
        std::env::set_var("MCP_INTEGRATION_FORTITUDE_DATA_DIR", "/tmp/test_data");

        let config = ServerConfig::from_env().unwrap();

        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "0.0.0.0");
        assert!(!config.auth.enabled);
        assert_eq!(config.performance.cache_size, 2000);
        assert_eq!(config.security.max_request_size, 2097152);
        assert_eq!(config.integration.classification_threshold, 0.8);
        assert_eq!(
            config.security.allowed_origins,
            vec!["https://example.com", "https://test.com"]
        );
        assert_eq!(
            config.security.ip_whitelist,
            vec!["127.0.0.1", "192.168.1.0/24"]
        );
        assert_eq!(
            config.integration.fortitude_data_dir,
            std::path::PathBuf::from("/tmp/test_data")
        );

        // Clean up
        std::env::remove_var("MCP_SERVER_PORT");
        std::env::remove_var("MCP_SERVER_HOST");
        std::env::remove_var("MCP_AUTH_ENABLED");
        std::env::remove_var("MCP_PERFORMANCE_CACHE_SIZE");
        std::env::remove_var("MCP_SECURITY_MAX_REQUEST_SIZE");
        std::env::remove_var("MCP_INTEGRATION_CLASSIFICATION_THRESHOLD");
        std::env::remove_var("MCP_SECURITY_ALLOWED_ORIGINS");
        std::env::remove_var("MCP_SECURITY_IP_WHITELIST");
        std::env::remove_var("MCP_INTEGRATION_FORTITUDE_DATA_DIR");
    }

    #[test]
    fn test_config_merge() {
        let mut base_config = ServerConfig::default();
        let mut other_config = ServerConfig {
            port: 9090,
            host: "0.0.0.0".to_string(),
            ..Default::default()
        };
        other_config.performance.cache_size = 2000;
        other_config.security.max_request_size = 2097152;

        // Merge configurations
        base_config.merge_with(other_config);

        // Check that values were merged
        assert_eq!(base_config.port, 9090);
        assert_eq!(base_config.host, "0.0.0.0");
        assert_eq!(base_config.performance.cache_size, 2000);
        assert_eq!(base_config.security.max_request_size, 2097152);
    }

    #[test]
    fn test_env_var_documentation() {
        let docs = ServerConfig::get_env_var_documentation();
        assert!(!docs.is_empty());

        // Check that some key variables are documented
        let var_names: Vec<&str> = docs.iter().map(|(name, _)| *name).collect();
        assert!(var_names.contains(&"MCP_SERVER_HOST"));
        assert!(var_names.contains(&"MCP_SERVER_PORT"));
        assert!(var_names.contains(&"MCP_JWT_SECRET"));
        assert!(var_names.contains(&"MCP_INTEGRATION_CLASSIFICATION_THRESHOLD"));
    }
}
