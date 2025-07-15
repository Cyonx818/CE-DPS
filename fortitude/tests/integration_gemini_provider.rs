//! Integration test for Gemini provider cross-component compatibility
//!
//! This test verifies that the Gemini provider works correctly with other system components
//! including the provider manager, configuration system, and common interfaces.

use fortitude::providers::config::ProviderSettings;
use fortitude::providers::{
    GeminiProvider, Provider, ProviderConfig, ProviderManager, SelectionStrategy,
};
use std::sync::Arc;
use std::time::Duration;

mod common;
use common::{valid_gemini_settings, TestEnvironmentGuard};

/// Test that Gemini provider can be used with ProviderManager
#[tokio::test]
async fn test_gemini_provider_manager_integration() {
    let _guard = TestEnvironmentGuard::new();

    // Create provider manager
    let config = fortitude::providers::manager::ProviderConfig::default();
    let manager = ProviderManager::new(config)
        .await
        .expect("Manager creation should succeed");

    // Create Gemini provider
    let settings = valid_gemini_settings();
    let provider = GeminiProvider::new(settings)
        .await
        .expect("Gemini provider creation should succeed");

    // Add provider to manager
    let result = manager
        .add_provider("gemini-test".to_string(), Arc::new(provider))
        .await;
    assert!(
        result.is_ok(),
        "Adding Gemini provider to manager should succeed"
    );

    // Verify provider is listed
    let providers = manager.list_providers().await;
    assert!(
        providers.contains(&"gemini-test".to_string()),
        "Gemini provider should be listed"
    );

    // Test health check through manager
    let health_results = manager
        .health_check_all()
        .await
        .expect("Health check should succeed");
    assert!(
        health_results.contains_key("gemini-test"),
        "Health check should include Gemini provider"
    );

    println!("✅ Gemini provider manager integration test passed");
}

/// Test that Gemini provider works with different selection strategies
#[tokio::test]
async fn test_gemini_provider_selection_strategies() {
    let _guard = TestEnvironmentGuard::new();

    let strategies = vec![
        SelectionStrategy::RoundRobin,
        SelectionStrategy::LowestLatency,
        SelectionStrategy::HighestSuccessRate,
        SelectionStrategy::CostOptimized,
        SelectionStrategy::Balanced,
    ];

    for strategy in strategies {
        let mut config = fortitude::providers::manager::ProviderConfig::default();
        config.selection_strategy = strategy.clone();

        let manager = ProviderManager::new(config)
            .await
            .expect("Manager creation should succeed");

        // Add Gemini provider
        let settings = valid_gemini_settings();
        let provider = GeminiProvider::new(settings)
            .await
            .expect("Gemini provider creation should succeed");
        let result = manager
            .add_provider("gemini-strategy-test".to_string(), Arc::new(provider))
            .await;
        assert!(
            result.is_ok(),
            "Adding Gemini provider should succeed for strategy: {:?}",
            strategy
        );

        println!(
            "✅ Gemini provider works with selection strategy: {:?}",
            strategy
        );
    }
}

/// Test that Gemini provider configuration integrates with the config system
#[tokio::test]
async fn test_gemini_provider_configuration_integration() {
    let _guard = TestEnvironmentGuard::new();

    // Test with different configuration options
    let test_configs = vec![
        // Basic configuration
        valid_gemini_settings(),
        // With custom timeout
        valid_gemini_settings().with_timeout(Duration::from_secs(60)),
        // With custom endpoint (though Gemini uses fixed endpoint)
        valid_gemini_settings()
            .with_endpoint("https://generativelanguage.googleapis.com/v1beta".to_string()),
        // With rate limits
        valid_gemini_settings()
            .with_rate_limits(fortitude::providers::config::RateLimitConfig::conservative()),
    ];

    for (i, config) in test_configs.into_iter().enumerate() {
        // Validate configuration
        assert!(
            config.validate().is_ok(),
            "Configuration {} should be valid",
            i
        );

        // Create provider with configuration
        let provider_result = GeminiProvider::new(config).await;
        assert!(
            provider_result.is_ok(),
            "Provider creation with config {} should succeed",
            i
        );

        let provider = provider_result.unwrap();

        // Test basic functionality
        let metadata = provider.metadata();
        assert_eq!(
            metadata.name(),
            "gemini",
            "Provider name should be consistent for config {}",
            i
        );

        let health_result = provider.health_check().await;
        assert!(
            health_result.is_ok(),
            "Health check should work for config {}",
            i
        );

        println!("✅ Configuration {} passed integration test", i);
    }
}

/// Test that Gemini provider can be used alongside other providers
#[tokio::test]
async fn test_gemini_provider_multi_provider_setup() {
    let _guard = TestEnvironmentGuard::new();

    let config = fortitude::providers::manager::ProviderConfig::default();
    let manager = ProviderManager::new(config)
        .await
        .expect("Manager creation should succeed");

    // Add Gemini provider
    let gemini_settings = valid_gemini_settings();
    let gemini_provider = GeminiProvider::new(gemini_settings)
        .await
        .expect("Gemini provider creation should succeed");
    let result = manager
        .add_provider("gemini".to_string(), Arc::new(gemini_provider))
        .await;
    assert!(result.is_ok(), "Adding Gemini provider should succeed");

    // Add mock providers to test multi-provider scenario
    let mock_provider1 = Arc::new(common::mock_providers::MockProvider::new("mock-openai"));
    let result = manager
        .add_provider("mock-openai".to_string(), mock_provider1)
        .await;
    assert!(result.is_ok(), "Adding mock OpenAI provider should succeed");

    let mock_provider2 = Arc::new(common::mock_providers::MockProvider::new("mock-claude"));
    let result = manager
        .add_provider("mock-claude".to_string(), mock_provider2)
        .await;
    assert!(result.is_ok(), "Adding mock Claude provider should succeed");

    // Verify all providers are listed
    let providers = manager.list_providers().await;
    assert_eq!(providers.len(), 3, "Should have 3 providers");
    assert!(
        providers.contains(&"gemini".to_string()),
        "Should include Gemini provider"
    );
    assert!(
        providers.contains(&"mock-openai".to_string()),
        "Should include mock OpenAI provider"
    );
    assert!(
        providers.contains(&"mock-claude".to_string()),
        "Should include mock Claude provider"
    );

    // Test health check for all providers
    let health_results = manager
        .health_check_all()
        .await
        .expect("Health check should succeed");
    assert_eq!(
        health_results.len(),
        3,
        "Should have health results for all providers"
    );

    // Test performance stats
    let performance_stats = manager.get_performance_stats().await;
    assert_eq!(
        performance_stats.len(),
        3,
        "Should have performance stats for all providers"
    );

    // Test getting healthy providers
    let healthy_providers = manager.get_healthy_providers().await;
    assert!(
        !healthy_providers.is_empty(),
        "Should have at least some healthy providers"
    );

    // Verify Gemini provider is included in healthy providers list
    let gemini_in_healthy = healthy_providers.iter().any(|(name, _)| name == "gemini");
    // Note: This might be false if health check fails due to network/auth, which is expected in test environment
    println!("Gemini provider in healthy list: {}", gemini_in_healthy);

    println!("✅ Multi-provider setup test passed");
}

/// Test provider metadata consistency across the system
#[tokio::test]
async fn test_gemini_provider_metadata_consistency() {
    let _guard = TestEnvironmentGuard::new();

    let settings = valid_gemini_settings();
    let provider = GeminiProvider::new(settings)
        .await
        .expect("Provider creation should succeed");

    let metadata = provider.metadata();

    // Test metadata consistency
    assert_eq!(
        metadata.name(),
        "gemini",
        "Provider name should be 'gemini'"
    );
    assert!(
        !metadata.version().is_empty(),
        "Version should not be empty"
    );

    // Test capabilities are properly set
    let capabilities = metadata.capabilities();
    let expected_capabilities = vec![
        "research",
        "rate_limited",
        "cost_estimation",
        "safety_settings",
        "multimodal",
    ];

    for expected in expected_capabilities {
        assert!(
            capabilities.contains(&expected.to_string()),
            "Should have capability: {}",
            expected
        );
    }

    // Test supported models
    let models = metadata.supported_models();
    assert!(!models.is_empty(), "Should have supported models");
    assert!(
        models.contains(&"gemini-1.5-pro".to_string()),
        "Should support gemini-1.5-pro"
    );

    // Test context length
    assert!(
        metadata.max_context_length() > 0,
        "Context length should be positive"
    );
    assert!(
        metadata.max_context_length() >= 30720,
        "Gemini should support at least 30K tokens"
    );

    // Test rate limits
    let rate_limits = metadata.rate_limits();
    assert!(
        rate_limits.requests_per_minute > 0,
        "Requests per minute should be positive"
    );
    assert!(
        rate_limits.input_tokens_per_minute > 0,
        "Input tokens per minute should be positive"
    );
    assert!(
        rate_limits.output_tokens_per_minute > 0,
        "Output tokens per minute should be positive"
    );
    assert!(
        rate_limits.max_concurrent_requests > 0,
        "Max concurrent requests should be positive"
    );

    println!("✅ Metadata consistency test passed");
}

/// Test error handling consistency across components
#[tokio::test]
async fn test_gemini_provider_error_handling_consistency() {
    let _guard = TestEnvironmentGuard::new();

    // Test with invalid configuration
    let invalid_settings =
        ProviderSettings::new("invalid-key".to_string(), "gemini-1.5-pro".to_string());
    let provider_result = GeminiProvider::new(invalid_settings).await;

    // Should either fail creation or fail on first operation with consistent error structure
    if provider_result.is_err() {
        let error = provider_result.unwrap_err();
        let error_str = error.to_string();
        assert!(!error_str.is_empty(), "Error message should not be empty");
        assert!(
            error_str.contains("API key") || error_str.contains("configuration"),
            "Error should mention API key or configuration issue"
        );
    } else {
        // If creation succeeded, test that operations fail consistently
        let provider = provider_result.unwrap();

        let query_result = provider.research_query("test query".to_string()).await;
        assert!(
            query_result.is_err(),
            "Query should fail with invalid configuration"
        );

        let error = query_result.unwrap_err();

        // Test error properties
        assert!(
            !error.provider().is_empty(),
            "Error should have provider name"
        );
        assert_eq!(
            error.provider(),
            "gemini",
            "Error provider should be 'gemini'"
        );

        // Test error type consistency
        let error_display = format!("{}", error);
        assert!(
            !error_display.is_empty(),
            "Error display should not be empty"
        );
    }

    println!("✅ Error handling consistency test passed");
}
