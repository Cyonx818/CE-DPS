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

// ABOUTME: Configuration management for Fortitude API server
// Handles HTTP server configuration with authentication, performance, and security settings
// Reuses patterns from MCP server with HTTP-specific extensions

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;
use validator::Validate;

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApiServerConfig {
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

    /// CORS configuration
    pub cors: CorsConfig,

    /// Rate limiting configuration
    #[validate(nested)]
    pub rate_limit: RateLimitConfig,

    /// Performance settings
    #[validate(nested)]
    pub performance: PerformanceConfig,

    /// Security settings
    #[validate(nested)]
    pub security: SecurityConfig,

    /// Feature flags
    pub features: std::collections::HashMap<String, bool>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthConfig {
    /// Enable JWT authentication
    pub enabled: bool,

    /// JWT secret key (must be at least 32 characters)
    #[validate(length(min = 32))]
    pub jwt_secret: String,

    /// Token expiration in hours
    #[validate(range(min = 1, max = 168))] // 1 hour to 1 week
    pub token_expiration_hours: u32,

    /// Rate limiting per client
    #[validate(nested)]
    pub rate_limit: RateLimitConfig,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins (empty means all)
    pub allowed_origins: Vec<String>,

    /// Allow credentials
    pub allow_credentials: bool,

    /// Maximum age for preflight requests
    pub max_age: u32,
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

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceConfig {
    /// Request buffer size
    #[validate(range(min = 1024, max = 1048576))] // 1KB to 1MB
    pub request_buffer_size: usize,

    /// Response compression threshold
    #[validate(range(min = 256, max = 65536))] // 256B to 64KB
    pub compression_threshold: usize,

    /// Keep-alive timeout in seconds
    #[validate(range(min = 1, max = 300))]
    pub keep_alive_timeout: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    /// Enable request ID header
    pub enable_request_id: bool,

    /// Maximum request body size in bytes
    #[validate(range(min = 1024, max = 104857600))] // 1KB to 100MB
    pub max_request_body_size: usize,

    /// Enable security headers
    pub enable_security_headers: bool,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        let mut features = std::collections::HashMap::new();
        features.insert("pattern_tracking".to_string(), true);

        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            max_connections: 1000,
            request_timeout: 30,
            auth: AuthConfig::default(),
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            features,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_secret: "your-secret-key-must-be-at-least-32-characters-long-for-security"
                .to_string(),
            token_expiration_hours: 24,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allow_credentials: true,
            max_age: 86400, // 24 hours
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

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            request_buffer_size: 8192,   // 8KB
            compression_threshold: 1024, // 1KB
            keep_alive_timeout: 75,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_request_id: true,
            max_request_body_size: 10485760, // 10MB
            enable_security_headers: true,
        }
    }
}

impl ApiServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Server settings
        if let Ok(port) = env::var("FORTITUDE_API_PORT") {
            config.port = port
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_PORT"))?;
        }

        if let Ok(host) = env::var("FORTITUDE_API_HOST") {
            config.host = host;
        }

        if let Ok(max_conn) = env::var("FORTITUDE_API_MAX_CONNECTIONS") {
            config.max_connections = max_conn
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_MAX_CONNECTIONS"))?;
        }

        if let Ok(timeout) = env::var("FORTITUDE_API_REQUEST_TIMEOUT") {
            config.request_timeout = timeout
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_REQUEST_TIMEOUT"))?;
        }

        // Authentication settings
        if let Ok(auth_enabled) = env::var("FORTITUDE_API_AUTH_ENABLED") {
            config.auth.enabled = auth_enabled.to_lowercase() == "true";
        }

        if let Ok(jwt_secret) = env::var("FORTITUDE_API_JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }

        if let Ok(token_exp) = env::var("FORTITUDE_API_TOKEN_EXPIRATION_HOURS") {
            config.auth.token_expiration_hours = token_exp
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_TOKEN_EXPIRATION_HOURS"))?;
        }

        // Rate limiting
        if let Ok(rate_limit) = env::var("FORTITUDE_API_RATE_LIMIT_PER_MINUTE") {
            let limit = rate_limit
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_RATE_LIMIT_PER_MINUTE"))?;
            config.rate_limit.max_requests_per_minute = limit;
            config.auth.rate_limit.max_requests_per_minute = limit;
        }

        // CORS settings
        if let Ok(origins) = env::var("FORTITUDE_API_CORS_ORIGINS") {
            config.cors.allowed_origins =
                origins.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(credentials) = env::var("FORTITUDE_API_CORS_CREDENTIALS") {
            config.cors.allow_credentials = credentials.to_lowercase() == "true";
        }

        // Performance settings
        if let Ok(buffer_size) = env::var("FORTITUDE_API_REQUEST_BUFFER_SIZE") {
            config.performance.request_buffer_size = buffer_size
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_REQUEST_BUFFER_SIZE"))?;
        }

        if let Ok(compression) = env::var("FORTITUDE_API_COMPRESSION_THRESHOLD") {
            config.performance.compression_threshold = compression
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_COMPRESSION_THRESHOLD"))?;
        }

        // Security settings
        if let Ok(max_body_size) = env::var("FORTITUDE_API_MAX_REQUEST_BODY_SIZE") {
            config.security.max_request_body_size = max_body_size
                .parse()
                .map_err(|_| anyhow!("Invalid FORTITUDE_API_MAX_REQUEST_BODY_SIZE"))?;
        }

        // Validate configuration
        config
            .validate()
            .map_err(|e| anyhow!("Configuration validation failed: {}", e))?;

        Ok(config)
    }

    /// Get bind address for server
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config_validation() {
        let config = ApiServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_from_env() {
        // Set test environment variables
        env::set_var("FORTITUDE_API_PORT", "8080");
        env::set_var("FORTITUDE_API_HOST", "0.0.0.0");
        env::set_var("FORTITUDE_API_AUTH_ENABLED", "false");

        let config = ApiServerConfig::from_env().unwrap();

        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(!config.auth.enabled);

        // Clean up
        env::remove_var("FORTITUDE_API_PORT");
        env::remove_var("FORTITUDE_API_HOST");
        env::remove_var("FORTITUDE_API_AUTH_ENABLED");
    }

    #[test]
    fn test_invalid_port_range() {
        let mut config = ApiServerConfig {
            port: 80,
            ..Default::default()
        }; // Too low (< 1024)
        assert!(config.validate().is_err());

        // Note: 65535 is actually the max valid port, so it should pass validation
        config.port = 65535; // Max valid port
        assert!(config.validate().is_ok());

        // Test minimum valid port
        config.port = 1024; // Min valid port
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_jwt_secret_length() {
        let mut config = ApiServerConfig::default();
        config.auth.jwt_secret = "short".to_string(); // Too short
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_bind_address() {
        let config = ApiServerConfig::default();
        assert_eq!(config.bind_address(), "127.0.0.1:3000");
    }
}
