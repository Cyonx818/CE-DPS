//! Test fixtures and data generators for multi-LLM provider testing

use fortitude::providers::config::{
    FallbackStrategy, LoadBalancingStrategy, MultiProviderConfig, ProviderConfig, ProviderSettings,
    RateLimitConfig, RetryConfig,
};
use std::collections::HashMap;
use std::time::Duration;

/// Generate valid provider settings for testing
pub fn valid_openai_settings() -> ProviderSettings {
    ProviderSettings::new(
        "sk-test1234567890abcdef1234567890abcdef".to_string(),
        "gpt-3.5-turbo".to_string(),
    )
    .with_timeout(Duration::from_secs(30))
    .with_rate_limits(RateLimitConfig::default())
    .with_retry(RetryConfig::default())
}

/// Generate valid Claude provider settings for testing
pub fn valid_claude_settings() -> ProviderSettings {
    ProviderSettings::new(
        "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    )
    .with_timeout(Duration::from_secs(30))
    .with_rate_limits(RateLimitConfig::default())
    .with_retry(RetryConfig::default())
}

/// Generate valid Gemini provider settings for testing
pub fn valid_gemini_settings() -> ProviderSettings {
    ProviderSettings::new(
        "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
        "gemini-1.5-pro".to_string(),
    )
    .with_timeout(Duration::from_secs(30))
    .with_rate_limits(RateLimitConfig::default())
    .with_retry(RetryConfig::default())
}

/// Generate invalid provider settings for testing validation
pub fn invalid_provider_settings() -> Vec<ProviderSettings> {
    vec![
        // Empty API key
        ProviderSettings::new("".to_string(), "gpt-3.5-turbo".to_string()),
        // Empty model
        ProviderSettings::new("sk-test123".to_string(), "".to_string()),
        // Timeout too short
        ProviderSettings::new("sk-test123".to_string(), "gpt-3.5-turbo".to_string())
            .with_timeout(Duration::from_millis(500)),
        // Timeout too long
        ProviderSettings::new("sk-test123".to_string(), "gpt-3.5-turbo".to_string())
            .with_timeout(Duration::from_secs(400)),
        // Invalid endpoint URL
        ProviderSettings::new("sk-test123".to_string(), "gpt-3.5-turbo".to_string())
            .with_endpoint("not-a-url".to_string()),
    ]
}

/// Generate conservative rate limit configuration for testing
pub fn conservative_rate_limits() -> RateLimitConfig {
    RateLimitConfig {
        requests_per_minute: 30,
        input_tokens_per_minute: 20_000,
        output_tokens_per_minute: 5_000,
        max_concurrent_requests: 2,
    }
}

/// Generate aggressive rate limit configuration for testing
pub fn aggressive_rate_limits() -> RateLimitConfig {
    RateLimitConfig {
        requests_per_minute: 300,
        input_tokens_per_minute: 200_000,
        output_tokens_per_minute: 50_000,
        max_concurrent_requests: 20,
    }
}

/// Generate invalid rate limit configurations for testing
pub fn invalid_rate_limit_configs() -> Vec<RateLimitConfig> {
    vec![
        // Requests per minute too low
        RateLimitConfig {
            requests_per_minute: 0,
            ..RateLimitConfig::default()
        },
        // Requests per minute too high
        RateLimitConfig {
            requests_per_minute: 20_000,
            ..RateLimitConfig::default()
        },
        // Input tokens too low
        RateLimitConfig {
            input_tokens_per_minute: 50,
            ..RateLimitConfig::default()
        },
        // Max concurrent requests too high
        RateLimitConfig {
            max_concurrent_requests: 200,
            ..RateLimitConfig::default()
        },
    ]
}

/// Generate fast retry configuration for testing
pub fn fast_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 1.5,
        jitter: true,
    }
}

/// Generate conservative retry configuration for testing
pub fn conservative_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 2,
        initial_delay: Duration::from_secs(2),
        max_delay: Duration::from_secs(30),
        backoff_multiplier: 2.0,
        jitter: false,
    }
}

/// Generate invalid retry configurations for testing
pub fn invalid_retry_configs() -> Vec<RetryConfig> {
    vec![
        // Max retries too high
        RetryConfig {
            max_retries: 15,
            ..RetryConfig::default()
        },
        // Backoff multiplier too low
        RetryConfig {
            backoff_multiplier: 0.5,
            ..RetryConfig::default()
        },
        // Backoff multiplier too high
        RetryConfig {
            backoff_multiplier: 15.0,
            ..RetryConfig::default()
        },
        // Initial delay >= max delay
        RetryConfig {
            initial_delay: Duration::from_secs(60),
            max_delay: Duration::from_secs(30),
            ..RetryConfig::default()
        },
    ]
}

/// Generate a complete provider configuration for testing
pub fn complete_openai_provider_config() -> ProviderConfig {
    let mut custom_headers = HashMap::new();
    custom_headers.insert("User-Agent".to_string(), "Fortitude-Test/1.0".to_string());

    let mut custom_parameters = HashMap::new();
    custom_parameters.insert(
        "temperature".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()),
    );
    custom_parameters.insert(
        "max_tokens".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1000)),
    );

    ProviderConfig::new(
        "primary-openai".to_string(),
        "openai".to_string(),
        valid_openai_settings(),
    )
    .with_priority(100)
    .with_enabled(true)
    .with_header("User-Agent".to_string(), "Fortitude-Test/1.0".to_string())
    .with_parameter(
        "temperature".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()),
    )
    .with_parameter(
        "max_tokens".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1000)),
    )
}

/// Generate a complete Claude provider configuration for testing
pub fn complete_claude_provider_config() -> ProviderConfig {
    ProviderConfig::new(
        "secondary-claude".to_string(),
        "claude".to_string(),
        valid_claude_settings(),
    )
    .with_priority(200)
    .with_enabled(true)
    .with_header("Anthropic-Version".to_string(), "2023-06-01".to_string())
    .with_parameter(
        "max_tokens".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1500)),
    )
}

/// Generate a complete Gemini provider configuration for testing
pub fn complete_gemini_provider_config() -> ProviderConfig {
    ProviderConfig::new(
        "backup-gemini".to_string(),
        "gemini".to_string(),
        valid_gemini_settings(),
    )
    .with_priority(300)
    .with_enabled(true)
    .with_parameter(
        "temperature".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.9).unwrap()),
    )
    .with_parameter(
        "top_p".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.8).unwrap()),
    )
}

/// Generate multi-provider configuration with all providers for testing
pub fn complete_multi_provider_config() -> MultiProviderConfig {
    MultiProviderConfig::new()
        .add_provider(complete_openai_provider_config())
        .add_provider(complete_claude_provider_config())
        .add_provider(complete_gemini_provider_config())
        .with_fallback_strategy(FallbackStrategy::Priority)
        .with_load_balancing(LoadBalancingStrategy::None)
        .with_health_check_interval(Duration::from_secs(30))
}

/// Generate multi-provider configuration with round-robin fallback
pub fn round_robin_multi_provider_config() -> MultiProviderConfig {
    MultiProviderConfig::new()
        .add_provider(complete_openai_provider_config())
        .add_provider(complete_claude_provider_config())
        .add_provider(complete_gemini_provider_config())
        .with_fallback_strategy(FallbackStrategy::RoundRobin)
        .with_load_balancing(LoadBalancingStrategy::RoundRobin)
        .with_health_check_interval(Duration::from_secs(15))
}

/// Generate multi-provider configuration with least loaded fallback
pub fn least_loaded_multi_provider_config() -> MultiProviderConfig {
    MultiProviderConfig::new()
        .add_provider(complete_openai_provider_config())
        .add_provider(complete_claude_provider_config())
        .add_provider(complete_gemini_provider_config())
        .with_fallback_strategy(FallbackStrategy::LeastLoaded)
        .with_load_balancing(LoadBalancingStrategy::LeastConnections)
        .with_health_check_interval(Duration::from_secs(10))
}

/// Generate multi-provider configuration with fastest response fallback
pub fn fastest_response_multi_provider_config() -> MultiProviderConfig {
    MultiProviderConfig::new()
        .add_provider(complete_openai_provider_config())
        .add_provider(complete_claude_provider_config())
        .add_provider(complete_gemini_provider_config())
        .with_fallback_strategy(FallbackStrategy::FastestResponse)
        .with_load_balancing(LoadBalancingStrategy::None)
        .with_health_check_interval(Duration::from_secs(5))
}

/// Generate weighted load balancing configuration for testing
pub fn weighted_load_balancing_multi_provider_config() -> MultiProviderConfig {
    let mut weights = HashMap::new();
    weights.insert("primary-openai".to_string(), 50);
    weights.insert("secondary-claude".to_string(), 30);
    weights.insert("backup-gemini".to_string(), 20);

    MultiProviderConfig::new()
        .add_provider(complete_openai_provider_config())
        .add_provider(complete_claude_provider_config())
        .add_provider(complete_gemini_provider_config())
        .with_fallback_strategy(FallbackStrategy::Priority)
        .with_load_balancing(LoadBalancingStrategy::Weighted(weights))
        .with_health_check_interval(Duration::from_secs(30))
}

/// Generate invalid multi-provider configurations for testing
pub fn invalid_multi_provider_configs() -> Vec<MultiProviderConfig> {
    vec![
        // No providers
        MultiProviderConfig::new(),
        // Duplicate provider names
        MultiProviderConfig::new()
            .add_provider(complete_openai_provider_config())
            .add_provider(ProviderConfig::new(
                "primary-openai".to_string(), // Duplicate name
                "claude".to_string(),
                valid_claude_settings(),
            )),
        // Invalid provider configuration
        MultiProviderConfig::new().add_provider(ProviderConfig::new(
            "invalid-provider".to_string(),
            "openai".to_string(),
            ProviderSettings::new("".to_string(), "gpt-3.5-turbo".to_string()), // Invalid settings
        )),
    ]
}

/// Generate test queries of various lengths and complexities
pub fn test_queries() -> Vec<String> {
    vec![
        // Short query
        "Hello".to_string(),
        
        // Medium query
        "What is the capital of France?".to_string(),
        
        // Long query
        "Explain the differences between synchronous and asynchronous programming, including the benefits and drawbacks of each approach, and provide examples of when you would use each.".to_string(),
        
        // Very long query (to test token limits)
        "Write a comprehensive analysis of the impact of artificial intelligence on modern society, discussing both the positive and negative aspects. Include sections on economic implications, ethical considerations, technological advancements, social changes, and future predictions. Make sure to provide specific examples and cite relevant research where applicable. The analysis should be thorough and well-structured, suitable for an academic audience.".to_string(),
        
        // Empty query (for validation testing)
        "".to_string(),
        
        // Whitespace-only query
        "   \t\n  ".to_string(),
        
        // Query with special characters
        "How do you handle UTF-8 encoding in Rust? 🦀 Discuss the use of String vs &str types.".to_string(),
        
        // Technical query
        "Implement a thread-safe LRU cache in Rust using Arc and Mutex. Explain the trade-offs between different approaches.".to_string(),
        
        // JSON-like query (to test escaping)
        r#"Parse this JSON: {"name": "test", "value": 42, "nested": {"array": [1, 2, 3]}}"#.to_string(),
        
        // Code query
        "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
    ]
}

/// Generate test error scenarios for provider testing
pub struct ErrorScenarios {
    pub auth_failure_api_key: String,
    pub invalid_model: String,
    pub malformed_endpoint: String,
    pub rate_limit_config: RateLimitConfig,
    pub timeout_settings: ProviderSettings,
}

impl ErrorScenarios {
    pub fn new() -> Self {
        Self {
            auth_failure_api_key: "invalid-key".to_string(),
            invalid_model: "non-existent-model".to_string(),
            malformed_endpoint: "not-a-url".to_string(),
            rate_limit_config: RateLimitConfig {
                requests_per_minute: 1, // Very low limit
                input_tokens_per_minute: 100,
                output_tokens_per_minute: 50,
                max_concurrent_requests: 1,
            },
            timeout_settings: ProviderSettings::new(
                "valid-key".to_string(),
                "gpt-3.5-turbo".to_string(),
            )
            .with_timeout(Duration::from_millis(100)), // Very short timeout
        }
    }
}

/// Generate environment variable test cases
pub fn environment_variable_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("OPENAI_API_KEY", "sk-test1234567890abcdef1234567890abcdef"),
        (
            "ANTHROPIC_API_KEY",
            "sk-ant-test1234567890abcdef1234567890abcdef",
        ),
        (
            "GOOGLE_API_KEY",
            "AIzaSyTest1234567890abcdef1234567890abcdef",
        ),
        ("OPENAI_MODEL", "gpt-4"),
        ("CLAUDE_MODEL", "claude-3-5-sonnet-20241022"),
        ("GEMINI_MODEL", "gemini-1.5-pro"),
        ("PROVIDER_TIMEOUT", "30"),
        ("PROVIDER_MAX_RETRIES", "3"),
        ("HEALTH_CHECK_INTERVAL", "30"),
    ]
}

/// Generate performance test configurations
pub struct PerformanceTestConfig {
    pub concurrent_requests: usize,
    pub total_requests: usize,
    pub request_timeout: Duration,
    pub expected_max_latency: Duration,
    pub expected_min_throughput: f64, // requests per second
}

impl PerformanceTestConfig {
    pub fn light_load() -> Self {
        Self {
            concurrent_requests: 5,
            total_requests: 50,
            request_timeout: Duration::from_secs(10),
            expected_max_latency: Duration::from_secs(2),
            expected_min_throughput: 5.0,
        }
    }

    pub fn medium_load() -> Self {
        Self {
            concurrent_requests: 20,
            total_requests: 200,
            request_timeout: Duration::from_secs(15),
            expected_max_latency: Duration::from_secs(5),
            expected_min_throughput: 10.0,
        }
    }

    pub fn heavy_load() -> Self {
        Self {
            concurrent_requests: 50,
            total_requests: 500,
            request_timeout: Duration::from_secs(30),
            expected_max_latency: Duration::from_secs(10),
            expected_min_throughput: 15.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_provider_settings() {
        let openai_settings = valid_openai_settings();
        assert!(openai_settings.validate().is_ok());

        let claude_settings = valid_claude_settings();
        assert!(claude_settings.validate().is_ok());

        let gemini_settings = valid_gemini_settings();
        assert!(gemini_settings.validate().is_ok());
    }

    #[test]
    fn test_invalid_provider_settings() {
        let invalid_settings = invalid_provider_settings();
        for settings in invalid_settings {
            assert!(settings.validate().is_err());
        }
    }

    #[test]
    fn test_rate_limit_configs() {
        let conservative = conservative_rate_limits();
        assert!(conservative.validate().is_ok());

        let aggressive = aggressive_rate_limits();
        assert!(aggressive.validate().is_ok());

        let invalid_configs = invalid_rate_limit_configs();
        for config in invalid_configs {
            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn test_retry_configs() {
        let fast = fast_retry_config();
        assert!(fast.validate().is_ok());

        let conservative = conservative_retry_config();
        assert!(conservative.validate().is_ok());

        let invalid_configs = invalid_retry_configs();
        for config in invalid_configs {
            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn test_provider_configs() {
        let openai_config = complete_openai_provider_config();
        assert!(openai_config.validate().is_ok());

        let claude_config = complete_claude_provider_config();
        assert!(claude_config.validate().is_ok());

        let gemini_config = complete_gemini_provider_config();
        assert!(gemini_config.validate().is_ok());
    }

    #[test]
    fn test_multi_provider_configs() {
        let complete_config = complete_multi_provider_config();
        assert!(complete_config.validate().is_ok());

        let round_robin_config = round_robin_multi_provider_config();
        assert!(round_robin_config.validate().is_ok());

        let least_loaded_config = least_loaded_multi_provider_config();
        assert!(least_loaded_config.validate().is_ok());

        let fastest_response_config = fastest_response_multi_provider_config();
        assert!(fastest_response_config.validate().is_ok());

        let weighted_config = weighted_load_balancing_multi_provider_config();
        assert!(weighted_config.validate().is_ok());

        let invalid_configs = invalid_multi_provider_configs();
        for config in invalid_configs {
            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn test_query_fixtures() {
        let queries = test_queries();
        assert!(!queries.is_empty());

        // Test that we have different query types
        let has_short = queries.iter().any(|q| q.len() < 50);
        let has_long = queries.iter().any(|q| q.len() > 500);
        let has_empty = queries.iter().any(|q| q.trim().is_empty());

        assert!(has_short, "Should have short queries");
        assert!(has_long, "Should have long queries");
        assert!(
            has_empty,
            "Should have empty/whitespace queries for validation testing"
        );
    }

    #[test]
    fn test_error_scenarios() {
        let scenarios = ErrorScenarios::new();
        assert!(!scenarios.auth_failure_api_key.is_empty());
        assert!(!scenarios.invalid_model.is_empty());
        assert!(!scenarios.malformed_endpoint.starts_with("http"));
        assert!(scenarios.rate_limit_config.validate().is_ok());
        assert!(scenarios.timeout_settings.validate().is_ok());
    }

    #[test]
    fn test_performance_configs() {
        let light = PerformanceTestConfig::light_load();
        assert!(light.concurrent_requests > 0);
        assert!(light.total_requests > light.concurrent_requests);

        let medium = PerformanceTestConfig::medium_load();
        assert!(medium.concurrent_requests > light.concurrent_requests);
        assert!(medium.total_requests > light.total_requests);

        let heavy = PerformanceTestConfig::heavy_load();
        assert!(heavy.concurrent_requests > medium.concurrent_requests);
        assert!(heavy.total_requests > medium.total_requests);
    }

    #[test]
    fn test_environment_variable_test_cases() {
        let test_cases = environment_variable_test_cases();
        assert!(!test_cases.is_empty());

        // Should have API keys for all major providers
        let has_openai = test_cases.iter().any(|(k, _)| k.contains("OPENAI"));
        let has_anthropic = test_cases.iter().any(|(k, _)| k.contains("ANTHROPIC"));
        let has_google = test_cases.iter().any(|(k, _)| k.contains("GOOGLE"));

        assert!(has_openai);
        assert!(has_anthropic);
        assert!(has_google);
    }
}
