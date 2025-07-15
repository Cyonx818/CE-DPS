//! Comprehensive unit tests for Configuration Management (Task 1.6)
//! 
//! This module tests:
//! - Environment variable detection and loading
//! - YAML configuration file loading and parsing
//! - Provider auto-detection and registration
//! - Configuration validation and error handling
//! - Hot reloading and configuration updates
//! - Multi-provider configuration management
//! - Rate limit and retry configuration validation

use fortitude::providers::config::{
    ProviderSettings, ProviderConfig, MultiProviderConfig, RateLimitConfig, RetryConfig,
    FallbackStrategy, LoadBalancingStrategy, ConfigError, ConfigResult
};
use crate::common::{
    valid_openai_settings, valid_claude_settings, valid_gemini_settings,
    invalid_provider_settings, conservative_rate_limits, aggressive_rate_limits,
    TestEnvironmentGuard
};
use std::time::Duration;
use std::collections::HashMap;
use proptest::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;

mod provider_settings_tests {
    use super::*;

    /// ANCHOR: Verifies provider settings creation and validation
    #[test]
    fn test_anchor_provider_settings_creation() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = ProviderSettings::new(
            "test-api-key".to_string(),
            "gpt-4".to_string(),
        );
        
        assert_eq!(settings.api_key, "test-api-key");
        assert_eq!(settings.model, "gpt-4");
        assert_eq!(settings.timeout, Duration::from_secs(30));
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_provider_settings_builder_pattern() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = ProviderSettings::new("api-key".to_string(), "model".to_string())
            .with_endpoint("https://api.custom.com".to_string())
            .with_timeout(Duration::from_secs(60))
            .with_rate_limits(conservative_rate_limits())
            .with_retry(RetryConfig::fast());
        
        assert_eq!(settings.endpoint, Some("https://api.custom.com".to_string()));
        assert_eq!(settings.timeout, Duration::from_secs(60));
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_provider_settings_validation_success() {
        let _guard = TestEnvironmentGuard::new();
        
        let test_cases = vec![
            ProviderSettings::new("valid-key".to_string(), "gpt-4".to_string()),
            ProviderSettings::new("sk-test123".to_string(), "claude-3-sonnet".to_string())
                .with_endpoint("https://api.anthropic.com".to_string()),
            ProviderSettings::new("AIzaSy123".to_string(), "gemini-pro".to_string())
                .with_timeout(Duration::from_secs(45))
                .with_rate_limits(aggressive_rate_limits()),
        ];
        
        for (i, settings) in test_cases.into_iter().enumerate() {
            assert!(settings.validate().is_ok(), "Test case {} should be valid", i);
        }
    }

    #[test]
    fn test_provider_settings_validation_failures() {
        let _guard = TestEnvironmentGuard::new();
        
        let test_cases = vec![
            // Empty API key
            ProviderSettings::new("".to_string(), "gpt-4".to_string()),
            
            // Empty model
            ProviderSettings::new("valid-key".to_string(), "".to_string()),
            
            // Timeout too short
            ProviderSettings::new("valid-key".to_string(), "gpt-4".to_string())
                .with_timeout(Duration::from_millis(500)),
            
            // Timeout too long
            ProviderSettings::new("valid-key".to_string(), "gpt-4".to_string())
                .with_timeout(Duration::from_secs(400)),
            
            // Invalid endpoint URL
            ProviderSettings::new("valid-key".to_string(), "gpt-4".to_string())
                .with_endpoint("not-a-url".to_string()),
            
            // Empty endpoint
            ProviderSettings::new("valid-key".to_string(), "gpt-4".to_string())
                .with_endpoint("".to_string()),
        ];
        
        for (i, settings) in test_cases.into_iter().enumerate() {
            assert!(settings.validate().is_err(), "Test case {} should be invalid", i);
        }
    }

    #[test]
    fn test_provider_settings_from_env() {
        let _guard = TestEnvironmentGuard::new();
        
        // Set environment variable
        std::env::set_var("TEST_API_KEY", "test-key-from-env");
        
        let settings = ProviderSettings::from_env("TEST_API_KEY", "gpt-4".to_string());
        assert!(settings.is_ok());
        
        let settings = settings.unwrap();
        assert_eq!(settings.api_key, "test-key-from-env");
        assert_eq!(settings.model, "gpt-4");
        
        // Clean up
        std::env::remove_var("TEST_API_KEY");
        
        // Test missing environment variable
        let result = ProviderSettings::from_env("NONEXISTENT_VAR", "model".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::EnvironmentVariable(_)));
    }

    #[test]
    fn test_provider_settings_env_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test multiple environment variables
        let env_vars = vec![
            ("OPENAI_API_KEY", "sk-test123"),
            ("CLAUDE_API_KEY", "sk-ant-test456"),
            ("GEMINI_API_KEY", "AIzaSy789"),
        ];
        
        for (var_name, var_value) in &env_vars {
            std::env::set_var(var_name, var_value);
        }
        
        let openai_settings = ProviderSettings::from_env("OPENAI_API_KEY", "gpt-4".to_string());
        let claude_settings = ProviderSettings::from_env("CLAUDE_API_KEY", "claude-3-sonnet".to_string());
        let gemini_settings = ProviderSettings::from_env("GEMINI_API_KEY", "gemini-pro".to_string());
        
        assert!(openai_settings.is_ok());
        assert!(claude_settings.is_ok());
        assert!(gemini_settings.is_ok());
        
        // Clean up
        for (var_name, _) in &env_vars {
            std::env::remove_var(var_name);
        }
    }

    proptest! {
        #[test]
        fn test_provider_settings_timeout_property(
            timeout_seconds in 1u64..=300u64
        ) {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = ProviderSettings::new("api-key".to_string(), "model".to_string())
                .with_timeout(Duration::from_secs(timeout_seconds));
            
            assert!(settings.validate().is_ok());
            assert_eq!(settings.timeout, Duration::from_secs(timeout_seconds));
        }
        
        #[test]
        fn test_provider_settings_api_key_property(
            api_key in "[a-zA-Z0-9\\-_]{10,100}"
        ) {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = ProviderSettings::new(api_key.clone(), "model".to_string());
            
            if !api_key.trim().is_empty() {
                assert!(settings.validate().is_ok());
                assert_eq!(settings.api_key, api_key);
            }
        }
    }
}

mod rate_limit_config_tests {
    use super::*;

    /// ANCHOR: Verifies rate limit configuration validation
    #[test]
    fn test_anchor_rate_limit_config_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = RateLimitConfig::default();
        assert!(config.validate().is_ok());
        
        let conservative = RateLimitConfig::conservative();
        assert!(conservative.validate().is_ok());
        assert!(conservative.requests_per_minute < config.requests_per_minute);
        
        let aggressive = RateLimitConfig::aggressive();
        assert!(aggressive.validate().is_ok());
        assert!(aggressive.requests_per_minute > config.requests_per_minute);
    }

    #[test]
    fn test_rate_limit_config_validation_boundaries() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test valid boundary values
        let min_valid = RateLimitConfig {
            requests_per_minute: 1,
            input_tokens_per_minute: 100,
            output_tokens_per_minute: 100,
            max_concurrent_requests: 1,
        };
        assert!(min_valid.validate().is_ok());
        
        let max_valid = RateLimitConfig {
            requests_per_minute: 10000,
            input_tokens_per_minute: 1000000,
            output_tokens_per_minute: 1000000,
            max_concurrent_requests: 100,
        };
        assert!(max_valid.validate().is_ok());
        
        // Test invalid boundary values
        let below_min = RateLimitConfig {
            requests_per_minute: 0, // Below minimum
            ..Default::default()
        };
        assert!(below_min.validate().is_err());
        
        let above_max = RateLimitConfig {
            requests_per_minute: 10001, // Above maximum
            ..Default::default()
        };
        assert!(above_max.validate().is_err());
    }

    #[test]
    fn test_rate_limit_config_presets() {
        let _guard = TestEnvironmentGuard::new();
        
        let default = RateLimitConfig::default();
        let conservative = RateLimitConfig::conservative();
        let aggressive = RateLimitConfig::aggressive();
        
        // Verify preset relationships
        assert!(conservative.requests_per_minute < default.requests_per_minute);
        assert!(conservative.input_tokens_per_minute < default.input_tokens_per_minute);
        assert!(conservative.max_concurrent_requests < default.max_concurrent_requests);
        
        assert!(aggressive.requests_per_minute > default.requests_per_minute);
        assert!(aggressive.input_tokens_per_minute > default.input_tokens_per_minute);
        assert!(aggressive.max_concurrent_requests > default.max_concurrent_requests);
        
        // Verify all presets are valid
        assert!(default.validate().is_ok());
        assert!(conservative.validate().is_ok());
        assert!(aggressive.validate().is_ok());
    }

    proptest! {
        #[test]
        fn test_rate_limit_config_property(
            requests_per_minute in 1u32..=10000u32,
            input_tokens_per_minute in 100u32..=1000000u32,
            output_tokens_per_minute in 100u32..=1000000u32,
            max_concurrent_requests in 1u32..=100u32
        ) {
            let _guard = TestEnvironmentGuard::new();
            
            let config = RateLimitConfig {
                requests_per_minute,
                input_tokens_per_minute,
                output_tokens_per_minute,
                max_concurrent_requests,
            };
            
            assert!(config.validate().is_ok());
        }
    }
}

mod retry_config_tests {
    use super::*;

    /// ANCHOR: Verifies retry configuration validation and delay calculation
    #[test]
    fn test_anchor_retry_config_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = RetryConfig::default();
        assert!(config.validate().is_ok());
        
        let fast = RetryConfig::fast();
        assert!(fast.validate().is_ok());
        assert!(fast.initial_delay < config.initial_delay);
        
        let conservative = RetryConfig::conservative();
        assert!(conservative.validate().is_ok());
        assert!(conservative.max_retries < config.max_retries);
    }

    #[test]
    fn test_retry_config_delay_calculation() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
            max_retries: 5,
        };
        
        let delays = (0..6).map(|i| config.calculate_delay(i)).collect::<Vec<_>>();
        
        assert_eq!(delays[0], Duration::from_millis(100));
        assert_eq!(delays[1], Duration::from_millis(200));
        assert_eq!(delays[2], Duration::from_millis(400));
        assert_eq!(delays[3], Duration::from_millis(800));
        assert_eq!(delays[4], Duration::from_millis(1600));
        assert_eq!(delays[5], Duration::from_millis(3200));
        
        // Test clamping to max delay
        let config_with_low_max = RetryConfig {
            max_delay: Duration::from_millis(500),
            ..config
        };
        
        let clamped_delay = config_with_low_max.calculate_delay(10);
        assert_eq!(clamped_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_retry_config_jitter() {
        let _guard = TestEnvironmentGuard::new();
        
        let config_with_jitter = RetryConfig {
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
            max_retries: 3,
        };
        
        let config_without_jitter = RetryConfig {
            jitter: false,
            ..config_with_jitter.clone()
        };
        
        // Multiple runs to test jitter variation
        let mut jitter_delays = Vec::new();
        let mut no_jitter_delays = Vec::new();
        
        for _ in 0..10 {
            jitter_delays.push(config_with_jitter.calculate_delay(2));
            no_jitter_delays.push(config_without_jitter.calculate_delay(2));
        }
        
        // Without jitter, all delays should be identical
        assert!(no_jitter_delays.iter().all(|&d| d == no_jitter_delays[0]));
        
        // With jitter, there should be some variation (though this is probabilistic)
        let base_delay = config_without_jitter.calculate_delay(2);
        let jitter_range = jitter_delays.iter().any(|&d| d != base_delay);
        
        // Note: This test might occasionally fail due to randomness, but it's very unlikely
        assert!(jitter_range, "Expected some variation in jittered delays");
    }

    #[test]
    fn test_retry_config_validation_errors() {
        let _guard = TestEnvironmentGuard::new();
        
        // Initial delay >= max delay
        let invalid_delay_config = RetryConfig {
            initial_delay: Duration::from_secs(60),
            max_delay: Duration::from_secs(30),
            ..Default::default()
        };
        assert!(invalid_delay_config.validate().is_err());
        
        // Max retries too high
        let invalid_retries_config = RetryConfig {
            max_retries: 15, // Above maximum of 10
            ..Default::default()
        };
        assert!(invalid_retries_config.validate().is_err());
        
        // Backoff multiplier too high
        let invalid_multiplier_config = RetryConfig {
            backoff_multiplier: 15.0, // Above maximum of 10.0
            ..Default::default()
        };
        assert!(invalid_multiplier_config.validate().is_err());
    }

    proptest! {
        #[test]
        fn test_retry_config_delay_monotonic(
            initial_delay_ms in 100u64..=1000u64,
            backoff_multiplier in 1.1f64..=5.0f64,
            attempt in 0u32..=5u32
        ) {
            let _guard = TestEnvironmentGuard::new();
            
            let config = RetryConfig {
                initial_delay: Duration::from_millis(initial_delay_ms),
                max_delay: Duration::from_secs(60),
                backoff_multiplier,
                jitter: false,
                max_retries: 10,
            };
            
            if attempt > 0 {
                let current_delay = config.calculate_delay(attempt);
                let previous_delay = config.calculate_delay(attempt - 1);
                
                // Delays should be non-decreasing (monotonic)
                assert!(current_delay >= previous_delay);
            }
        }
    }
}

mod provider_config_tests {
    use super::*;

    /// ANCHOR: Verifies provider configuration creation and validation
    #[test]
    fn test_anchor_provider_config_creation() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings();
        let config = ProviderConfig::new(
            "openai-primary".to_string(),
            "openai".to_string(),
            settings,
        );
        
        assert_eq!(config.name, "openai-primary");
        assert_eq!(config.provider_type, "openai");
        assert!(config.enabled);
        assert_eq!(config.priority, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_builder_pattern() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_claude_settings();
        let config = ProviderConfig::new(
            "claude-secondary".to_string(),
            "claude".to_string(),
            settings,
        )
        .with_priority(200)
        .with_enabled(false)
        .with_header("X-Custom-Header".to_string(), "custom-value".to_string())
        .with_parameter("temperature".to_string(), serde_json::json!(0.7));
        
        assert_eq!(config.priority, 200);
        assert!(!config.enabled);
        assert_eq!(config.custom_headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));
        assert_eq!(config.custom_parameters.get("temperature"), Some(&serde_json::json!(0.7)));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_validation_errors() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings();
        
        // Empty name
        let empty_name_config = ProviderConfig::new(
            "".to_string(),
            "openai".to_string(),
            settings.clone(),
        );
        assert!(empty_name_config.validate().is_err());
        
        // Empty provider type
        let empty_type_config = ProviderConfig::new(
            "provider".to_string(),
            "".to_string(),
            settings.clone(),
        );
        assert!(empty_type_config.validate().is_err());
        
        // Invalid settings
        let invalid_settings = invalid_provider_settings();
        let invalid_settings_config = ProviderConfig::new(
            "provider".to_string(),
            "openai".to_string(),
            invalid_settings,
        );
        assert!(invalid_settings_config.validate().is_err());
    }

    #[test]
    fn test_provider_config_custom_attributes() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_gemini_settings();
        let mut config = ProviderConfig::new(
            "gemini-test".to_string(),
            "gemini".to_string(),
            settings,
        );
        
        // Add multiple custom headers
        config = config
            .with_header("Authorization".to_string(), "Bearer token".to_string())
            .with_header("X-API-Version".to_string(), "v1beta".to_string())
            .with_header("Content-Type".to_string(), "application/json".to_string());
        
        // Add multiple custom parameters
        config = config
            .with_parameter("temperature".to_string(), serde_json::json!(0.8))
            .with_parameter("top_p".to_string(), serde_json::json!(0.9))
            .with_parameter("max_tokens".to_string(), serde_json::json!(1000))
            .with_parameter("safety_settings".to_string(), serde_json::json!({
                "category": "HARM_CATEGORY_HATE_SPEECH",
                "threshold": "BLOCK_MEDIUM_AND_ABOVE"
            }));
        
        assert_eq!(config.custom_headers.len(), 3);
        assert_eq!(config.custom_parameters.len(), 4);
        assert!(config.validate().is_ok());
        
        // Verify specific values
        assert_eq!(config.custom_headers.get("X-API-Version"), Some(&"v1beta".to_string()));
        assert_eq!(config.custom_parameters.get("temperature"), Some(&serde_json::json!(0.8)));
    }
}

mod multi_provider_config_tests {
    use super::*;

    /// ANCHOR: Verifies multi-provider configuration management
    #[test]
    fn test_anchor_multi_provider_config_creation() {
        let _guard = TestEnvironmentGuard::new();
        
        let openai_config = ProviderConfig::new(
            "openai".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        ).with_priority(100);
        
        let claude_config = ProviderConfig::new(
            "claude".to_string(),
            "claude".to_string(),
            valid_claude_settings(),
        ).with_priority(200);
        
        let multi_config = MultiProviderConfig::new()
            .add_provider(openai_config)
            .add_provider(claude_config)
            .with_fallback_strategy(FallbackStrategy::Priority)
            .with_load_balancing(LoadBalancingStrategy::RoundRobin);
        
        assert_eq!(multi_config.providers.len(), 2);
        assert!(matches!(multi_config.fallback_strategy, FallbackStrategy::Priority));
        assert!(matches!(multi_config.load_balancing, LoadBalancingStrategy::RoundRobin));
        assert!(multi_config.validate().is_ok());
    }

    #[test]
    fn test_multi_provider_config_enabled_providers() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider1 = ProviderConfig::new(
            "provider1".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        ).with_priority(300).with_enabled(true);
        
        let provider2 = ProviderConfig::new(
            "provider2".to_string(),
            "claude".to_string(),
            valid_claude_settings(),
        ).with_priority(100).with_enabled(true);
        
        let provider3 = ProviderConfig::new(
            "provider3".to_string(),
            "gemini".to_string(),
            valid_gemini_settings(),
        ).with_priority(200).with_enabled(false); // Disabled
        
        let multi_config = MultiProviderConfig::new()
            .add_provider(provider1)
            .add_provider(provider2)
            .add_provider(provider3);
        
        let enabled = multi_config.enabled_providers();
        assert_eq!(enabled.len(), 2); // Only enabled providers
        
        // Should be sorted by priority (lower number = higher priority)
        assert_eq!(enabled[0].name, "provider2"); // Priority 100
        assert_eq!(enabled[1].name, "provider1"); // Priority 300
    }

    #[test]
    fn test_multi_provider_config_provider_lookup() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ProviderConfig::new(
            "test-provider".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        );
        
        let multi_config = MultiProviderConfig::new().add_provider(provider);
        
        assert!(multi_config.get_provider("test-provider").is_some());
        assert!(multi_config.get_provider("nonexistent").is_none());
        
        let found_provider = multi_config.get_provider("test-provider").unwrap();
        assert_eq!(found_provider.name, "test-provider");
        assert_eq!(found_provider.provider_type, "openai");
    }

    #[test]
    fn test_multi_provider_config_validation_errors() {
        let _guard = TestEnvironmentGuard::new();
        
        // Empty providers list
        let empty_config = MultiProviderConfig::new();
        assert!(empty_config.validate().is_err());
        
        // Duplicate provider names
        let provider1 = ProviderConfig::new(
            "duplicate".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        );
        let provider2 = ProviderConfig::new(
            "duplicate".to_string(),
            "claude".to_string(),
            valid_claude_settings(),
        );
        
        let duplicate_config = MultiProviderConfig::new()
            .add_provider(provider1)
            .add_provider(provider2);
        assert!(duplicate_config.validate().is_err());
        
        // Invalid provider configuration
        let invalid_provider = ProviderConfig::new(
            "".to_string(), // Invalid: empty name
            "openai".to_string(),
            valid_openai_settings(),
        );
        
        let invalid_config = MultiProviderConfig::new().add_provider(invalid_provider);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_multi_provider_config_strategies() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ProviderConfig::new(
            "test".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        );
        
        // Test all fallback strategies
        let fallback_strategies = vec![
            FallbackStrategy::Priority,
            FallbackStrategy::RoundRobin,
            FallbackStrategy::LeastLoaded,
            FallbackStrategy::FastestResponse,
        ];
        
        for strategy in fallback_strategies {
            let config = MultiProviderConfig::new()
                .add_provider(provider.clone())
                .with_fallback_strategy(strategy.clone());
            
            assert!(config.validate().is_ok());
            assert!(matches!(config.fallback_strategy, strategy));
        }
        
        // Test all load balancing strategies
        let load_balancing_strategies = vec![
            LoadBalancingStrategy::None,
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::Weighted({
                let mut weights = HashMap::new();
                weights.insert("test".to_string(), 100);
                weights
            }),
        ];
        
        for strategy in load_balancing_strategies {
            let config = MultiProviderConfig::new()
                .add_provider(provider.clone())
                .with_load_balancing(strategy.clone());
            
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn test_multi_provider_config_health_check_interval() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ProviderConfig::new(
            "test".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        );
        
        let intervals = vec![
            Duration::from_secs(10),
            Duration::from_secs(30),
            Duration::from_secs(60),
            Duration::from_secs(300),
        ];
        
        for interval in intervals {
            let config = MultiProviderConfig::new()
                .add_provider(provider.clone())
                .with_health_check_interval(interval);
            
            assert_eq!(config.health_check_interval, interval);
            assert!(config.validate().is_ok());
        }
    }
}

mod configuration_serialization_tests {
    use super::*;

    /// ANCHOR: Verifies configuration serialization and deserialization
    #[test]
    fn test_anchor_configuration_serialization_roundtrip() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider_config = ProviderConfig::new(
            "test-provider".to_string(),
            "openai".to_string(),
            valid_openai_settings(),
        )
        .with_priority(150)
        .with_header("X-Custom".to_string(), "value".to_string())
        .with_parameter("temperature".to_string(), serde_json::json!(0.7));
        
        let multi_config = MultiProviderConfig::new()
            .add_provider(provider_config)
            .with_fallback_strategy(FallbackStrategy::RoundRobin)
            .with_health_check_interval(Duration::from_secs(45));
        
        // Serialize to JSON
        let json_str = serde_json::to_string(&multi_config).unwrap();
        assert!(!json_str.is_empty());
        
        // Deserialize from JSON
        let deserialized: MultiProviderConfig = serde_json::from_str(&json_str).unwrap();
        
        // Verify structure is preserved
        assert_eq!(deserialized.providers.len(), multi_config.providers.len());
        assert_eq!(deserialized.providers[0].name, multi_config.providers[0].name);
        assert_eq!(deserialized.providers[0].priority, multi_config.providers[0].priority);
        assert_eq!(deserialized.health_check_interval, multi_config.health_check_interval);
        
        // Verify custom attributes are preserved
        assert_eq!(
            deserialized.providers[0].custom_headers.get("X-Custom"),
            multi_config.providers[0].custom_headers.get("X-Custom")
        );
        assert_eq!(
            deserialized.providers[0].custom_parameters.get("temperature"),
            multi_config.providers[0].custom_parameters.get("temperature")
        );
    }

    #[test]
    fn test_yaml_configuration_file_parsing() {
        let _guard = TestEnvironmentGuard::new();
        
        let yaml_content = r#"
providers:
  - name: "openai-primary"
    provider_type: "openai"
    enabled: true
    priority: 100
    settings:
      api_key: "sk-test123"
      model: "gpt-4"
      timeout: 30000
      rate_limits:
        requests_per_minute: 60
        input_tokens_per_minute: 50000
        output_tokens_per_minute: 10000
        max_concurrent_requests: 5
      retry:
        max_retries: 3
        initial_delay: 1000
        max_delay: 60000
        backoff_multiplier: 2.0
        jitter: true
    custom_headers: {}
    custom_parameters: {}
  - name: "claude-secondary"
    provider_type: "claude"
    enabled: true
    priority: 200
    settings:
      api_key: "sk-ant-test456"
      model: "claude-3-sonnet"
      timeout: 45000
      rate_limits:
        requests_per_minute: 30
        input_tokens_per_minute: 20000
        output_tokens_per_minute: 5000
        max_concurrent_requests: 2
      retry:
        max_retries: 2
        initial_delay: 2000
        max_delay: 30000
        backoff_multiplier: 2.0
        jitter: false
    custom_headers: {}
    custom_parameters: {}
fallback_strategy: "Priority"
load_balancing: "None"
health_check_interval: 30000
"#;
        
        // Parse YAML
        let config: MultiProviderConfig = serde_yaml::from_str(yaml_content).unwrap();
        
        // Verify parsing
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.providers[0].name, "openai-primary");
        assert_eq!(config.providers[1].name, "claude-secondary");
        assert!(matches!(config.fallback_strategy, FallbackStrategy::Priority));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
        
        // Verify validation passes
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_configuration_file_hot_reload_simulation() {
        let _guard = TestEnvironmentGuard::new();
        
        // Create temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        
        // Initial configuration
        let initial_yaml = r#"
providers:
  - name: "provider1"
    provider_type: "openai"
    enabled: true
    priority: 100
    settings:
      api_key: "key1"
      model: "gpt-4"
      timeout: 30000
      rate_limits:
        requests_per_minute: 60
        input_tokens_per_minute: 50000
        output_tokens_per_minute: 10000
        max_concurrent_requests: 5
      retry:
        max_retries: 3
        initial_delay: 1000
        max_delay: 60000
        backoff_multiplier: 2.0
        jitter: true
    custom_headers: {}
    custom_parameters: {}
fallback_strategy: "Priority"
load_balancing: "None"
health_check_interval: 30000
"#;
        
        write!(temp_file, "{}", initial_yaml).unwrap();
        temp_file.flush().unwrap();
        
        // Load initial configuration
        let initial_content = std::fs::read_to_string(temp_file.path()).unwrap();
        let initial_config: MultiProviderConfig = serde_yaml::from_str(&initial_content).unwrap();
        assert_eq!(initial_config.providers.len(), 1);
        assert_eq!(initial_config.providers[0].name, "provider1");
        
        // Simulate configuration update
        let updated_yaml = r#"
providers:
  - name: "provider1"
    provider_type: "openai"
    enabled: true
    priority: 100
    settings:
      api_key: "key1"
      model: "gpt-4"
      timeout: 30000
      rate_limits:
        requests_per_minute: 60
        input_tokens_per_minute: 50000
        output_tokens_per_minute: 10000
        max_concurrent_requests: 5
      retry:
        max_retries: 3
        initial_delay: 1000
        max_delay: 60000
        backoff_multiplier: 2.0
        jitter: true
    custom_headers: {}
    custom_parameters: {}
  - name: "provider2"
    provider_type: "claude"
    enabled: true
    priority: 200
    settings:
      api_key: "key2"
      model: "claude-3-sonnet"
      timeout: 30000
      rate_limits:
        requests_per_minute: 30
        input_tokens_per_minute: 20000
        output_tokens_per_minute: 5000
        max_concurrent_requests: 2
      retry:
        max_retries: 2
        initial_delay: 2000
        max_delay: 30000
        backoff_multiplier: 2.0
        jitter: false
    custom_headers: {}
    custom_parameters: {}
fallback_strategy: "RoundRobin"
load_balancing: "RoundRobin"
health_check_interval: 60000
"#;
        
        // Overwrite file with updated configuration
        std::fs::write(temp_file.path(), updated_yaml).unwrap();
        
        // Load updated configuration
        let updated_content = std::fs::read_to_string(temp_file.path()).unwrap();
        let updated_config: MultiProviderConfig = serde_yaml::from_str(&updated_content).unwrap();
        
        // Verify updates
        assert_eq!(updated_config.providers.len(), 2);
        assert_eq!(updated_config.providers[0].name, "provider1");
        assert_eq!(updated_config.providers[1].name, "provider2");
        assert!(matches!(updated_config.fallback_strategy, FallbackStrategy::RoundRobin));
        assert!(matches!(updated_config.load_balancing, LoadBalancingStrategy::RoundRobin));
        assert_eq!(updated_config.health_check_interval, Duration::from_secs(60));
        
        // Verify validation passes
        assert!(updated_config.validate().is_ok());
    }

    proptest! {
        #[test]
        fn test_configuration_serialization_property(
            provider_count in 1usize..=5usize,
            health_check_seconds in 10u64..=300u64
        ) {
            let _guard = TestEnvironmentGuard::new();
            
            let mut multi_config = MultiProviderConfig::new()
                .with_health_check_interval(Duration::from_secs(health_check_seconds));
            
            for i in 0..provider_count {
                let provider = ProviderConfig::new(
                    format!("provider{}", i),
                    "openai".to_string(),
                    valid_openai_settings(),
                ).with_priority(100 + i as u32 * 10);
                
                multi_config = multi_config.add_provider(provider);
            }
            
            // Serialize and deserialize
            let json_str = serde_json::to_string(&multi_config).unwrap();
            let deserialized: MultiProviderConfig = serde_json::from_str(&json_str).unwrap();
            
            // Verify properties are preserved
            assert_eq!(deserialized.providers.len(), provider_count);
            assert_eq!(deserialized.health_check_interval, Duration::from_secs(health_check_seconds));
            assert!(deserialized.validate().is_ok());
        }
    }
}

mod environment_integration_tests {
    use super::*;

    /// ANCHOR: Verifies environment variable integration
    #[test]
    fn test_anchor_environment_variable_detection() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test common environment variable patterns
        let env_test_cases = vec![
            ("OPENAI_API_KEY", "sk-test-openai-key"),
            ("CLAUDE_API_KEY", "sk-ant-test-claude-key"),
            ("GEMINI_API_KEY", "AIzaSy-test-gemini-key"),
            ("ANTHROPIC_API_KEY", "sk-ant-test-anthropic-key"),
        ];
        
        for (env_var, test_value) in &env_test_cases {
            // Set environment variable
            std::env::set_var(env_var, test_value);
            
            // Test retrieval
            let retrieved = std::env::var(env_var).unwrap();
            assert_eq!(&retrieved, test_value);
            
            // Test with ProviderSettings
            let settings = ProviderSettings::from_env(env_var, "test-model".to_string());
            assert!(settings.is_ok());
            assert_eq!(settings.unwrap().api_key, *test_value);
            
            // Clean up
            std::env::remove_var(env_var);
        }
    }

    #[test]
    fn test_provider_auto_detection_from_environment() {
        let _guard = TestEnvironmentGuard::new();
        
        // Set up environment variables for multiple providers
        std::env::set_var("OPENAI_API_KEY", "sk-test-openai");
        std::env::set_var("CLAUDE_API_KEY", "sk-ant-test-claude");
        std::env::set_var("GEMINI_API_KEY", "AIzaSy-test-gemini");
        
        // Simulate auto-detection logic
        let mut detected_providers = Vec::new();
        
        let provider_env_mapping = vec![
            ("OPENAI_API_KEY", "openai", "gpt-4"),
            ("CLAUDE_API_KEY", "claude", "claude-3-sonnet"),
            ("GEMINI_API_KEY", "gemini", "gemini-pro"),
        ];
        
        for (env_var, provider_type, default_model) in provider_env_mapping {
            if std::env::var(env_var).is_ok() {
                let settings = ProviderSettings::from_env(env_var, default_model.to_string()).unwrap();
                let config = ProviderConfig::new(
                    provider_type.to_string(),
                    provider_type.to_string(),
                    settings,
                );
                detected_providers.push(config);
            }
        }
        
        assert_eq!(detected_providers.len(), 3);
        assert!(detected_providers.iter().any(|p| p.provider_type == "openai"));
        assert!(detected_providers.iter().any(|p| p.provider_type == "claude"));
        assert!(detected_providers.iter().any(|p| p.provider_type == "gemini"));
        
        // Create multi-provider configuration
        let mut multi_config = MultiProviderConfig::new();
        for provider in detected_providers {
            multi_config = multi_config.add_provider(provider);
        }
        
        assert!(multi_config.validate().is_ok());
        assert_eq!(multi_config.providers.len(), 3);
        
        // Clean up
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("CLAUDE_API_KEY");
        std::env::remove_var("GEMINI_API_KEY");
    }

    #[test]
    fn test_environment_variable_override_priority() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test override priority: env var > config file > defaults
        
        // Set environment variable
        std::env::set_var("TEST_API_KEY", "env-key-value");
        
        // Create settings from environment
        let env_settings = ProviderSettings::from_env("TEST_API_KEY", "model".to_string()).unwrap();
        assert_eq!(env_settings.api_key, "env-key-value");
        
        // Create settings with explicit value (should take precedence over defaults but not env)
        let explicit_settings = ProviderSettings::new("explicit-key-value".to_string(), "model".to_string());
        assert_eq!(explicit_settings.api_key, "explicit-key-value");
        
        // Environment should override when using from_env
        let env_override = ProviderSettings::from_env("TEST_API_KEY", "model".to_string()).unwrap();
        assert_eq!(env_override.api_key, "env-key-value");
        assert_ne!(env_override.api_key, "explicit-key-value");
        
        // Clean up
        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_missing_environment_variables_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        // Ensure test environment variables don't exist
        std::env::remove_var("NONEXISTENT_API_KEY");
        std::env::remove_var("MISSING_KEY");
        std::env::remove_var("UNDEFINED_VAR");
        
        // Test missing environment variables
        let missing_vars = vec![
            "NONEXISTENT_API_KEY",
            "MISSING_KEY",
            "UNDEFINED_VAR",
        ];
        
        for var_name in missing_vars {
            let result = ProviderSettings::from_env(var_name, "model".to_string());
            assert!(result.is_err());
            
            match result.unwrap_err() {
                ConfigError::EnvironmentVariable(missing_var) => {
                    assert_eq!(missing_var, var_name);
                }
                _ => panic!("Expected EnvironmentVariable error"),
            }
        }
    }
}