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

// ABOUTME: Provider configuration structures with validation and environment support
//! This module provides configuration structures for multi-LLM providers including
//! API credentials, rate limiting, retry policies, and timeout settings.
//! Supports environment variable substitution and comprehensive validation.
//!
//! # Example Usage
//!
//! ```rust
//! use fortitude::providers::config::{ProviderSettings, RateLimitConfig, RetryConfig};
//! use std::time::Duration;
//!
//! let settings = ProviderSettings {
//!     api_key: std::env::var("OPENAI_API_KEY").expect("API key required"),
//!     model: "gpt-4".to_string(),
//!     endpoint: None, // Use default
//!     timeout: Duration::from_secs(30),
//!     rate_limits: RateLimitConfig::default(),
//!     retry: RetryConfig::default(),
//! };
//!
//! // Validate configuration before use
//! settings.validate().expect("Configuration should be valid");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

/// Configuration validation errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Environment variable not found: {0}")]
    EnvironmentVariable(String),

    #[error("Invalid duration format: {0}")]
    InvalidDuration(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("Configuration conflict: {0}")]
    Conflict(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Rate limiting configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateLimitConfig {
    #[validate(range(
        min = 1,
        max = 10000,
        message = "Requests per minute must be between 1 and 10000"
    ))]
    pub requests_per_minute: u32,

    #[validate(range(
        min = 100,
        max = 1000000,
        message = "Input tokens per minute must be between 100 and 1,000,000"
    ))]
    pub input_tokens_per_minute: u32,

    #[validate(range(
        min = 100,
        max = 1000000,
        message = "Output tokens per minute must be between 100 and 1,000,000"
    ))]
    pub output_tokens_per_minute: u32,

    #[validate(range(
        min = 1,
        max = 100,
        message = "Max concurrent requests must be between 1 and 100"
    ))]
    pub max_concurrent_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            input_tokens_per_minute: 50_000,
            output_tokens_per_minute: 10_000,
            max_concurrent_requests: 5,
        }
    }
}

impl RateLimitConfig {
    /// Create a conservative rate limit configuration
    pub fn conservative() -> Self {
        Self {
            requests_per_minute: 30,
            input_tokens_per_minute: 20_000,
            output_tokens_per_minute: 5_000,
            max_concurrent_requests: 2,
        }
    }

    /// Create an aggressive rate limit configuration (for premium plans)
    pub fn aggressive() -> Self {
        Self {
            requests_per_minute: 300,
            input_tokens_per_minute: 200_000,
            output_tokens_per_minute: 50_000,
            max_concurrent_requests: 20,
        }
    }

    pub fn validate(&self) -> ConfigResult<()> {
        match Validate::validate(self) {
            Ok(_) => Ok(()),
            Err(e) => Err(ConfigError::Validation(e)),
        }
    }
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RetryConfig {
    #[validate(range(min = 0, max = 10, message = "Max retries must be between 0 and 10"))]
    pub max_retries: u32,

    #[serde(with = "duration_serde")]
    pub initial_delay: Duration,

    #[serde(with = "duration_serde")]
    pub max_delay: Duration,

    #[validate(range(
        min = 1.0,
        max = 10.0,
        message = "Backoff multiplier must be between 1.0 and 10.0"
    ))]
    pub backoff_multiplier: f64,

    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a fast retry configuration for low-latency scenarios
    pub fn fast() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    /// Create a conservative retry configuration for stable operations
    pub fn conservative() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }

    pub fn validate(&self) -> ConfigResult<()> {
        if let Err(e) = Validate::validate(self) {
            return Err(ConfigError::Validation(e));
        }

        if self.initial_delay >= self.max_delay {
            return Err(ConfigError::Conflict(
                "Initial delay must be less than max delay".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate the delay for a specific retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }

        let base_delay =
            self.initial_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);

        let delay = if self.jitter {
            let jitter_factor = 0.1; // 10% jitter
            let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_factor;
            base_delay * (1.0 + jitter)
        } else {
            base_delay
        };

        let clamped_delay = delay.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(clamped_delay as u64)
    }
}

/// Provider-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub api_key: String,
    pub model: String,
    pub endpoint: Option<String>,

    #[serde(with = "duration_serde")]
    pub timeout: Duration,

    pub rate_limits: RateLimitConfig,
    pub retry: RetryConfig,
}

impl ProviderSettings {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            endpoint: None,
            timeout: Duration::from_secs(30),
            rate_limits: RateLimitConfig::default(),
            retry: RetryConfig::default(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_rate_limits(mut self, rate_limits: RateLimitConfig) -> Self {
        self.rate_limits = rate_limits;
        self
    }

    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    pub fn validate(&self) -> ConfigResult<()> {
        // Validate API key
        if self.api_key.trim().is_empty() {
            return Err(ConfigError::Conflict("API key cannot be empty".to_string()));
        }

        // Validate model name
        if self.model.trim().is_empty() {
            return Err(ConfigError::Conflict(
                "Model name cannot be empty".to_string(),
            ));
        }

        // Validate endpoint URL if provided
        if let Some(ref endpoint) = self.endpoint {
            if endpoint.trim().is_empty() {
                return Err(ConfigError::InvalidUrl(
                    "Endpoint URL cannot be empty".to_string(),
                ));
            }
            // Basic URL validation
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                return Err(ConfigError::InvalidUrl(format!(
                    "Invalid endpoint URL format: {endpoint}"
                )));
            }
        }

        // Validate timeout
        if self.timeout < Duration::from_secs(1) {
            return Err(ConfigError::Conflict(
                "Timeout must be at least 1 second".to_string(),
            ));
        }

        if self.timeout > Duration::from_secs(300) {
            return Err(ConfigError::Conflict(
                "Timeout must not exceed 5 minutes".to_string(),
            ));
        }

        // Validate nested configurations
        self.rate_limits.validate()?;
        self.retry.validate()?;

        Ok(())
    }

    /// Load API key from environment variable
    pub fn from_env(env_var: &str, model: String) -> ConfigResult<Self> {
        let api_key = std::env::var(env_var)
            .map_err(|_| ConfigError::EnvironmentVariable(env_var.to_string()))?;

        Ok(Self::new(api_key, model))
    }
}

/// Configuration for a specific provider type (OpenAI, Claude, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub settings: ProviderSettings,
    pub enabled: bool,
    pub priority: u32,
    pub custom_headers: HashMap<String, String>,
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

impl ProviderConfig {
    pub fn new(name: String, provider_type: String, settings: ProviderSettings) -> Self {
        Self {
            name,
            provider_type,
            settings,
            enabled: true,
            priority: 100,
            custom_headers: HashMap::new(),
            custom_parameters: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.custom_headers.insert(key, value);
        self
    }

    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_parameters.insert(key, value);
        self
    }

    pub fn validate(&self) -> ConfigResult<()> {
        if self.name.trim().is_empty() {
            return Err(ConfigError::Conflict(
                "Provider name cannot be empty".to_string(),
            ));
        }

        if self.provider_type.trim().is_empty() {
            return Err(ConfigError::Conflict(
                "Provider type cannot be empty".to_string(),
            ));
        }

        // Validate nested settings
        self.settings.validate()?;

        Ok(())
    }
}

/// Multi-provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProviderConfig {
    pub providers: Vec<ProviderConfig>,

    pub fallback_strategy: FallbackStrategy,
    pub load_balancing: LoadBalancingStrategy,
    pub health_check_interval: Duration,
}

/// Fallback strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FallbackStrategy {
    /// Use providers in priority order
    #[default]
    Priority,
    /// Round-robin through healthy providers
    RoundRobin,
    /// Use the provider with lowest current load
    LeastLoaded,
    /// Use the fastest responding provider
    FastestResponse,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoadBalancingStrategy {
    /// No load balancing - use primary provider
    #[default]
    None,
    /// Distribute requests evenly
    RoundRobin,
    /// Send to provider with least current requests
    LeastConnections,
    /// Weight-based distribution
    Weighted(HashMap<String, u32>),
}

impl MultiProviderConfig {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            fallback_strategy: FallbackStrategy::default(),
            load_balancing: LoadBalancingStrategy::default(),
            health_check_interval: Duration::from_secs(30),
        }
    }

    pub fn add_provider(mut self, provider: ProviderConfig) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn with_fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    pub fn with_load_balancing(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.load_balancing = strategy;
        self
    }

    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    pub fn validate(&self) -> ConfigResult<()> {
        if self.providers.is_empty() {
            return Err(ConfigError::Conflict(
                "At least one provider must be configured".to_string(),
            ));
        }

        // Check for duplicate provider names
        let mut names = std::collections::HashSet::new();
        for provider in &self.providers {
            if !names.insert(&provider.name) {
                return Err(ConfigError::Conflict(format!(
                    "Duplicate provider name: {}",
                    provider.name
                )));
            }
        }

        // Validate each provider
        for provider in &self.providers {
            provider.validate()?;
        }

        Ok(())
    }

    /// Get enabled providers sorted by priority
    pub fn enabled_providers(&self) -> Vec<&ProviderConfig> {
        let mut enabled: Vec<_> = self.providers.iter().filter(|p| p.enabled).collect();
        enabled.sort_by(|a, b| a.priority.cmp(&b.priority));
        enabled
    }

    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }
}

impl Default for MultiProviderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialization helper for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = duration.as_millis() as u64;
        serializer.serialize_u64(millis)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_validation() {
        let valid_config = RateLimitConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = RateLimitConfig {
            requests_per_minute: 0, // Invalid: too low
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_rate_limit_config_presets() {
        let conservative = RateLimitConfig::conservative();
        assert!(conservative.validate().is_ok());
        assert!(conservative.requests_per_minute < RateLimitConfig::default().requests_per_minute);

        let aggressive = RateLimitConfig::aggressive();
        assert!(aggressive.validate().is_ok());
        assert!(aggressive.requests_per_minute > RateLimitConfig::default().requests_per_minute);
    }

    #[test]
    fn test_retry_config_validation() {
        let valid_config = RetryConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = RetryConfig {
            initial_delay: Duration::from_secs(60),
            max_delay: Duration::from_secs(30), // Invalid: initial > max
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };

        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);

        assert_eq!(delay0, Duration::from_millis(100));
        assert_eq!(delay1, Duration::from_millis(200));
        assert_eq!(delay2, Duration::from_millis(400));

        // Test clamping to max delay
        let delay_large = config.calculate_delay(10);
        assert_eq!(delay_large, Duration::from_secs(10));
    }

    #[test]
    fn test_provider_settings_validation() {
        let valid_settings =
            ProviderSettings::new("valid-api-key".to_string(), "gpt-4".to_string());
        assert!(valid_settings.validate().is_ok());

        let invalid_settings = ProviderSettings::new(
            "".to_string(), // Invalid: empty API key
            "gpt-4".to_string(),
        );
        assert!(invalid_settings.validate().is_err());

        let timeout_too_short = ProviderSettings {
            timeout: Duration::from_millis(500), // Invalid: too short
            ..valid_settings.clone()
        };
        assert!(timeout_too_short.validate().is_err());
    }

    #[test]
    fn test_provider_settings_builder() {
        let settings = ProviderSettings::new("api-key".to_string(), "model".to_string())
            .with_endpoint("https://api.example.com".to_string())
            .with_timeout(Duration::from_secs(60))
            .with_rate_limits(RateLimitConfig::conservative())
            .with_retry(RetryConfig::fast());

        assert!(settings.validate().is_ok());
        assert_eq!(
            settings.endpoint,
            Some("https://api.example.com".to_string())
        );
        assert_eq!(settings.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_provider_config_validation() {
        let settings = ProviderSettings::new("key".to_string(), "model".to_string());
        let valid_config = ProviderConfig::new(
            "test-provider".to_string(),
            "openai".to_string(),
            settings.clone(),
        );
        assert!(valid_config.validate().is_ok());

        let invalid_config = ProviderConfig::new(
            "".to_string(), // Invalid: empty name
            "openai".to_string(),
            settings,
        );
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_multi_provider_config() {
        let settings1 = ProviderSettings::new("key1".to_string(), "model1".to_string());
        let settings2 = ProviderSettings::new("key2".to_string(), "model2".to_string());

        let provider1 =
            ProviderConfig::new("provider1".to_string(), "openai".to_string(), settings1)
                .with_priority(100);
        let provider2 =
            ProviderConfig::new("provider2".to_string(), "claude".to_string(), settings2)
                .with_priority(200);

        let multi_config = MultiProviderConfig::new()
            .add_provider(provider1)
            .add_provider(provider2)
            .with_fallback_strategy(FallbackStrategy::Priority);

        assert!(multi_config.validate().is_ok());

        let enabled = multi_config.enabled_providers();
        assert_eq!(enabled.len(), 2);
        assert_eq!(enabled[0].name, "provider1"); // Lower priority = higher precedence
        assert_eq!(enabled[1].name, "provider2");
    }

    #[test]
    fn test_multi_provider_duplicate_names() {
        let settings = ProviderSettings::new("key".to_string(), "model".to_string());
        let provider1 = ProviderConfig::new(
            "duplicate".to_string(),
            "openai".to_string(),
            settings.clone(),
        );
        let provider2 =
            ProviderConfig::new("duplicate".to_string(), "claude".to_string(), settings);

        let multi_config = MultiProviderConfig::new()
            .add_provider(provider1)
            .add_provider(provider2);

        assert!(multi_config.validate().is_err());
    }

    #[test]
    fn test_provider_lookup() {
        let settings = ProviderSettings::new("key".to_string(), "model".to_string());
        let provider =
            ProviderConfig::new("test-provider".to_string(), "openai".to_string(), settings);

        let multi_config = MultiProviderConfig::new().add_provider(provider);

        assert!(multi_config.get_provider("test-provider").is_some());
        assert!(multi_config.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let settings = ProviderSettings::new("key".to_string(), "model".to_string());
        let provider = ProviderConfig::new("test".to_string(), "openai".to_string(), settings);
        let multi_config = MultiProviderConfig::new().add_provider(provider);

        let serialized = serde_json::to_string(&multi_config).unwrap();
        let deserialized: MultiProviderConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(multi_config.providers.len(), deserialized.providers.len());
        assert_eq!(
            multi_config.providers[0].name,
            deserialized.providers[0].name
        );
    }
}
