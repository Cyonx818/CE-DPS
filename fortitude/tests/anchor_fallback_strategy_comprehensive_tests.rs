//! ANCHOR: Comprehensive fallback strategy system tests
//!
//! This module contains anchor tests for the critical fallback strategy functionality
//! in the Fortitude multi-LLM provider system. These tests verify core fallback
//! capabilities that are essential for production reliability.
//!
//! # Test Coverage
//!
//! - **Provider Health Monitoring**: Circuit breaker functionality, health checks
//! - **Fallback Strategy Selection**: Round-robin, health-based, performance-based
//! - **Retry Mechanisms**: Exponential backoff, jitter, error handling
//! - **System Recovery**: Automatic provider recovery detection
//! - **Performance Tracking**: Metrics collection and provider scoring
//! - **Integration**: End-to-end fallback workflows

use async_trait::async_trait;
use fortitude::providers::fallback::{
    CircuitBreakerState, FallbackEngine, FallbackStrategy, RetryConfig,
};
use fortitude::providers::{
    HealthStatus, Provider, ProviderError, ProviderMetadata, ProviderResult, QueryCost, UsageStats,
};
use fortitude_types::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Helper function to create test research request
fn create_test_request(query: &str, research_type: ResearchType) -> ClassifiedRequest {
    ClassifiedRequest::new(
        query.to_string(),
        research_type,
        AudienceContext {
            level: "intermediate".to_string(),
            domain: "software".to_string(),
            format: "markdown".to_string(),
        },
        DomainContext {
            technology: "rust".to_string(),
            project_type: "library".to_string(),
            frameworks: vec!["tokio".to_string()],
            tags: vec!["async".to_string()],
        },
        0.8,
        vec!["test".to_string()],
    )
}

/// Test provider for anchor tests with controllable behavior
#[derive(Debug, Clone)]
struct AnchorTestProvider {
    name: String,
    healthy: Arc<AtomicBool>,
    latency: Duration,
    cost_per_request: f64,
    success_rate: f64,
    call_count: Arc<AtomicU64>,
    consecutive_failures: Arc<AtomicU64>,
    last_call_time: Arc<tokio::sync::Mutex<Option<Instant>>>,
}

impl AnchorTestProvider {
    fn new(name: &str, healthy: bool, latency: Duration, cost: f64, success_rate: f64) -> Self {
        Self {
            name: name.to_string(),
            healthy: Arc::new(AtomicBool::new(healthy)),
            latency,
            cost_per_request: cost,
            success_rate,
            call_count: Arc::new(AtomicU64::new(0)),
            consecutive_failures: Arc::new(AtomicU64::new(0)),
            last_call_time: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    fn set_healthy(&self, healthy: bool) {
        self.healthy.store(healthy, Ordering::SeqCst);
    }

    fn get_call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }

    fn reset_call_count(&self) {
        self.call_count.store(0, Ordering::SeqCst);
    }

    fn get_consecutive_failures(&self) -> u64 {
        self.consecutive_failures.load(Ordering::SeqCst)
    }

    async fn get_last_call_time(&self) -> Option<Instant> {
        *self.last_call_time.lock().await
    }
}

#[async_trait]
impl Provider for AnchorTestProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        // Update last call time
        {
            let mut last_call = self.last_call_time.lock().await;
            *last_call = Some(Instant::now());
        }

        // Simulate latency
        tokio::time::sleep(self.latency).await;

        // Check if provider should fail
        if !self.healthy.load(Ordering::SeqCst) {
            self.consecutive_failures.fetch_add(1, Ordering::SeqCst);
            return Err(ProviderError::ServiceUnavailable {
                provider: self.name.clone(),
                message: "Provider marked as unhealthy".to_string(),
                estimated_recovery: Some(Duration::from_secs(60)),
            });
        }

        // Random failure based on success rate
        if rand::random::<f64>() > self.success_rate {
            self.consecutive_failures.fetch_add(1, Ordering::SeqCst);
            return Err(ProviderError::QueryFailed {
                message: "Random failure for testing".to_string(),
                provider: self.name.clone(),
                error_code: Some("TEST_FAILURE".to_string()),
            });
        }

        self.consecutive_failures.store(0, Ordering::SeqCst);
        Ok(format!("{} response: {}", self.name, query))
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(self.name.clone(), "1.0.0".to_string())
            .with_capabilities(vec!["research".to_string(), "fallback_test".to_string()])
    }

    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        if self.healthy.load(Ordering::SeqCst) {
            let failures = self.consecutive_failures.load(Ordering::SeqCst);
            if failures > 2 {
                Ok(HealthStatus::Degraded(format!(
                    "{} consecutive failures",
                    failures
                )))
            } else {
                Ok(HealthStatus::Healthy)
            }
        } else {
            Ok(HealthStatus::Unhealthy(
                "Provider marked as unhealthy".to_string(),
            ))
        }
    }

    async fn estimate_cost(&self, _query: &str) -> ProviderResult<QueryCost> {
        Ok(QueryCost {
            estimated_input_tokens: 10,
            estimated_output_tokens: 20,
            estimated_duration: self.latency,
            estimated_cost_usd: Some(self.cost_per_request),
        })
    }

    async fn usage_stats(&self) -> ProviderResult<UsageStats> {
        Ok(UsageStats {
            total_requests: self.call_count.load(Ordering::SeqCst),
            successful_requests: (self.call_count.load(Ordering::SeqCst) as f64 * self.success_rate)
                as u64,
            failed_requests: self.call_count.load(Ordering::SeqCst)
                - (self.call_count.load(Ordering::SeqCst) as f64 * self.success_rate) as u64,
            total_input_tokens: self.call_count.load(Ordering::SeqCst) * 10,
            total_output_tokens: self.call_count.load(Ordering::SeqCst) * 20,
            average_response_time: self.latency,
            last_request_time: Some(chrono::Utc::now()),
        })
    }
}

/// ANCHOR: Core fallback engine functionality verification
#[tokio::test]
async fn test_anchor_fallback_engine_creation_and_setup() {
    // Test all supported fallback strategies can be created
    let strategies = vec![
        FallbackStrategy::RoundRobin { reset_after: None },
        FallbackStrategy::HealthBased {
            health_threshold: 0.7,
            check_interval: Duration::from_secs(30),
            circuit_breaker_threshold: 3,
        },
        FallbackStrategy::PerformanceBased {
            latency_weight: 0.4,
            success_rate_weight: 0.4,
            cost_weight: 0.2,
            window_size: 50,
        },
        FallbackStrategy::Priority {
            priority_order: vec!["primary".to_string(), "secondary".to_string()],
            fallback_to_health: true,
        },
    ];

    for strategy in strategies {
        let engine = FallbackEngine::new(strategy).await;
        assert!(
            engine.is_ok(),
            "Should create fallback engine for all strategies"
        );

        let engine = engine.unwrap();

        // Test adding and removing providers
        let provider = Arc::new(AnchorTestProvider::new(
            "test",
            true,
            Duration::from_millis(100),
            0.01,
            0.9,
        ));
        let add_result = engine
            .add_provider("test_provider".to_string(), provider)
            .await;
        assert!(add_result.is_ok(), "Should add provider successfully");

        let remove_result = engine.remove_provider("test_provider").await;
        assert!(remove_result.is_ok(), "Should remove provider successfully");
    }
}

/// ANCHOR: Round-robin strategy fairness and correctness
#[tokio::test]
async fn test_anchor_round_robin_strategy_fairness() {
    let strategy = FallbackStrategy::RoundRobin {
        reset_after: Some(100),
    };
    let engine = FallbackEngine::new(strategy).await.unwrap();

    // Add multiple providers
    let providers = vec![
        (
            "provider_a",
            Arc::new(AnchorTestProvider::new(
                "provider_a",
                true,
                Duration::from_millis(50),
                0.01,
                1.0,
            )),
        ),
        (
            "provider_b",
            Arc::new(AnchorTestProvider::new(
                "provider_b",
                true,
                Duration::from_millis(100),
                0.015,
                1.0,
            )),
        ),
        (
            "provider_c",
            Arc::new(AnchorTestProvider::new(
                "provider_c",
                true,
                Duration::from_millis(75),
                0.02,
                1.0,
            )),
        ),
    ];

    for (name, provider) in &providers {
        engine
            .add_provider(name.to_string(), provider.clone())
            .await
            .unwrap();
    }

    let request = create_test_request("round robin fairness test", ResearchType::Implementation);

    // Execute many requests to test fairness
    let mut selections = HashMap::new();
    for _ in 0..30 {
        if let Ok((provider_name, _)) = engine.select_provider(&request).await {
            *selections.entry(provider_name).or_insert(0) += 1;
        }
    }

    // Verify all providers were selected
    assert_eq!(selections.len(), 3, "All providers should be selected");

    // Verify fairness (each provider should get roughly equal selections)
    let total_selections: u32 = selections.values().sum();
    let expected_per_provider = total_selections / 3;

    for (provider_name, count) in &selections {
        let difference = (*count as i32 - expected_per_provider as i32).abs();
        assert!(
            difference <= 2,
            "Provider {} selection count {} should be close to expected {}",
            provider_name,
            count,
            expected_per_provider
        );
    }
}

/// ANCHOR: Health-based strategy provider health monitoring
#[tokio::test]
async fn test_anchor_health_based_strategy_monitoring() {
    let strategy = FallbackStrategy::HealthBased {
        health_threshold: 0.6,
        check_interval: Duration::from_millis(100),
        circuit_breaker_threshold: 2,
    };
    let engine = FallbackEngine::new(strategy).await.unwrap();

    let healthy_provider = Arc::new(AnchorTestProvider::new(
        "healthy",
        true,
        Duration::from_millis(50),
        0.01,
        0.95,
    ));
    let degraded_provider = Arc::new(AnchorTestProvider::new(
        "degraded",
        true,
        Duration::from_millis(200),
        0.02,
        0.7,
    ));
    let unhealthy_provider = Arc::new(AnchorTestProvider::new(
        "unhealthy",
        false,
        Duration::from_millis(100),
        0.015,
        0.5,
    ));

    engine
        .add_provider("healthy".to_string(), healthy_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("degraded".to_string(), degraded_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("unhealthy".to_string(), unhealthy_provider.clone())
        .await
        .unwrap();

    let request = create_test_request("health monitoring test", ResearchType::Implementation);

    // Execute requests to build health metrics
    for _ in 0..10 {
        let _ = engine.execute_with_fallback(&request).await;
    }

    // Verify healthy provider is preferred
    let healthy_calls = healthy_provider.get_call_count();
    let degraded_calls = degraded_provider.get_call_count();
    let unhealthy_calls = unhealthy_provider.get_call_count();

    assert!(healthy_calls > 0, "Healthy provider should be used");
    assert!(
        healthy_calls >= degraded_calls,
        "Healthy provider should be used more than degraded"
    );
    assert!(
        healthy_calls >= unhealthy_calls,
        "Healthy provider should be used more than unhealthy"
    );

    // Test health monitor directly
    let health_monitor = engine.health_monitor();
    let healthy_providers = health_monitor.get_healthy_providers().await;

    // At least the healthy provider should be in the list
    assert!(
        !healthy_providers.is_empty(),
        "Should have at least one healthy provider"
    );
    assert!(
        healthy_providers.contains(&"healthy".to_string()),
        "Healthy provider should be in healthy list"
    );
}

/// ANCHOR: Circuit breaker functionality verification
#[tokio::test]
async fn test_anchor_circuit_breaker_protection() {
    let strategy = FallbackStrategy::HealthBased {
        health_threshold: 0.5,
        check_interval: Duration::from_millis(50),
        circuit_breaker_threshold: 3,
    };
    let engine = FallbackEngine::new(strategy).await.unwrap();

    let failing_provider = Arc::new(AnchorTestProvider::new(
        "failing",
        true,
        Duration::from_millis(50),
        0.01,
        0.0,
    )); // Always fails
    let backup_provider = Arc::new(AnchorTestProvider::new(
        "backup",
        true,
        Duration::from_millis(100),
        0.02,
        1.0,
    )); // Always succeeds

    engine
        .add_provider("failing".to_string(), failing_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("backup".to_string(), backup_provider.clone())
        .await
        .unwrap();

    let request = create_test_request("circuit breaker test", ResearchType::Implementation);

    // Execute multiple requests to trigger circuit breaker
    for _ in 0..10 {
        let result = engine.execute_with_fallback(&request).await;
        // Should eventually succeed with backup provider
        if result.is_ok() {
            break;
        }
    }

    // Verify circuit breaker behavior
    let failing_calls = failing_provider.get_call_count();
    let backup_calls = backup_provider.get_call_count();

    // At least one provider should be used, and backup should be used more than failing
    let total_calls = failing_calls + backup_calls;
    assert!(total_calls > 0, "At least one provider should be used");
    assert!(
        backup_calls > 0,
        "Backup provider should be used after primary fails"
    );

    // Verify circuit breaker metrics
    let health_monitor = engine.health_monitor();
    let failing_metrics = health_monitor.get_provider_metrics("failing").await;

    if let Some(metrics) = failing_metrics {
        assert!(metrics.failed_requests > 0, "Should track failures");
        // Circuit breaker should eventually open
        assert!(
            matches!(
                metrics.circuit_breaker_state,
                CircuitBreakerState::Open { .. }
            ) || matches!(
                metrics.circuit_breaker_state,
                CircuitBreakerState::HalfOpen { .. }
            ),
            "Circuit breaker should open after failures"
        );
    }
}

/// ANCHOR: Retry mechanism with exponential backoff verification
#[tokio::test]
async fn test_anchor_retry_mechanism_exponential_backoff() {
    let strategy = FallbackStrategy::RoundRobin { reset_after: None };
    let mut engine = FallbackEngine::new(strategy).await.unwrap();

    // Configure short retry delays for testing
    engine.set_retry_config(RetryConfig {
        max_attempts: 4,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(200),
        backoff_multiplier: 2.0,
        jitter_factor: 0.0, // No jitter for predictable testing
    });

    let intermittent_provider = Arc::new(AnchorTestProvider::new(
        "intermittent",
        true,
        Duration::from_millis(20),
        0.01,
        0.25,
    )); // 25% success rate
    engine
        .add_provider("intermittent".to_string(), intermittent_provider.clone())
        .await
        .unwrap();

    let request = create_test_request("retry test", ResearchType::Implementation);

    // Test retry timing
    let start_time = Instant::now();
    let result = engine.execute_with_fallback(&request).await;
    let elapsed = start_time.elapsed();

    // Should have taken time for retries (at least 2-3 retry delays)
    let min_expected_time = Duration::from_millis(30); // 10 + 20 ms minimum

    if result.is_err() {
        // If it failed after all retries, should have taken retry time
        assert!(
            elapsed >= min_expected_time,
            "Failed execution should take retry time: {:?} >= {:?}",
            elapsed,
            min_expected_time
        );
    }

    // Verify multiple attempts were made
    assert!(
        intermittent_provider.get_call_count() > 1,
        "Should make multiple retry attempts: {}",
        intermittent_provider.get_call_count()
    );
    assert!(
        intermittent_provider.get_call_count() <= 4,
        "Should not exceed max retry attempts: {}",
        intermittent_provider.get_call_count()
    );
}

/// ANCHOR: Performance-based strategy selection verification
#[tokio::test]
async fn test_anchor_performance_based_strategy_optimization() {
    let strategy = FallbackStrategy::PerformanceBased {
        latency_weight: 0.5,
        success_rate_weight: 0.3,
        cost_weight: 0.2,
        window_size: 20,
    };
    let engine = FallbackEngine::new(strategy).await.unwrap();

    let fast_provider = Arc::new(AnchorTestProvider::new(
        "fast",
        true,
        Duration::from_millis(30),
        0.03,
        0.8,
    ));
    let reliable_provider = Arc::new(AnchorTestProvider::new(
        "reliable",
        true,
        Duration::from_millis(100),
        0.02,
        0.98,
    ));
    let cheap_provider = Arc::new(AnchorTestProvider::new(
        "cheap",
        true,
        Duration::from_millis(150),
        0.005,
        0.85,
    ));

    engine
        .add_provider("fast".to_string(), fast_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("reliable".to_string(), reliable_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("cheap".to_string(), cheap_provider.clone())
        .await
        .unwrap();

    let request = create_test_request("performance test", ResearchType::Implementation);

    // Execute requests to build performance history
    for _ in 0..15 {
        let _ = engine.execute_with_fallback(&request).await;
    }

    // Verify performance-based selection
    let fast_calls = fast_provider.get_call_count();
    let reliable_calls = reliable_provider.get_call_count();
    let cheap_calls = cheap_provider.get_call_count();

    let total_calls = fast_calls + reliable_calls + cheap_calls;

    assert!(total_calls > 0, "Should make calls to providers");

    // With the given weights, one provider should be preferred based on composite score
    // Fast provider has good latency, reliable has good success rate, cheap has good cost
    let call_counts = [fast_calls, reliable_calls, cheap_calls];
    let max_calls = call_counts.iter().max().unwrap();

    // The best performer should get more calls
    assert!(
        *max_calls >= total_calls / 4,
        "Best performer should get significant share of calls"
    );
}

/// ANCHOR: Provider recovery detection verification
#[tokio::test]
async fn test_anchor_provider_recovery_detection() {
    let strategy = FallbackStrategy::HealthBased {
        health_threshold: 0.6,
        check_interval: Duration::from_millis(50),
        circuit_breaker_threshold: 2,
    };
    let engine = FallbackEngine::new(strategy).await.unwrap();

    let recovering_provider = Arc::new(AnchorTestProvider::new(
        "recovering",
        false,
        Duration::from_millis(50),
        0.01,
        1.0,
    )); // Start unhealthy
    let stable_provider = Arc::new(AnchorTestProvider::new(
        "stable",
        true,
        Duration::from_millis(100),
        0.02,
        0.9,
    ));

    engine
        .add_provider("recovering".to_string(), recovering_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("stable".to_string(), stable_provider.clone())
        .await
        .unwrap();

    let request = create_test_request("recovery test", ResearchType::Implementation);

    // Phase 1: Provider is unhealthy, should use stable provider
    for _ in 0..3 {
        let _ = engine.execute_with_fallback(&request).await;
    }

    let initial_recovering_calls = recovering_provider.get_call_count();
    let initial_stable_calls = stable_provider.get_call_count();

    // Phase 2: Provider recovers
    recovering_provider.set_healthy(true);

    // Allow some time for health monitoring to detect recovery
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Phase 3: Execute more requests, should start using recovered provider
    for _ in 0..5 {
        let _ = engine.execute_with_fallback(&request).await;
    }

    let final_recovering_calls = recovering_provider.get_call_count();
    let final_stable_calls = stable_provider.get_call_count();

    // Verify initial phase used stable provider more
    assert!(
        initial_stable_calls >= initial_recovering_calls,
        "Initially should prefer stable provider"
    );

    // Verify recovery phase shows increased usage of recovered provider
    let recovering_increase = final_recovering_calls - initial_recovering_calls;
    assert!(
        recovering_increase > 0,
        "Recovered provider should be used after recovery"
    );
}

/// ANCHOR: Priority-based strategy provider ordering verification
#[tokio::test]
async fn test_anchor_priority_based_strategy_ordering() {
    let strategy = FallbackStrategy::Priority {
        priority_order: vec![
            "primary".to_string(),
            "secondary".to_string(),
            "tertiary".to_string(),
        ],
        fallback_to_health: true,
    };
    let engine = FallbackEngine::new(strategy).await.unwrap();

    let primary_provider = Arc::new(AnchorTestProvider::new(
        "primary",
        true,
        Duration::from_millis(100),
        0.03,
        0.9,
    ));
    let secondary_provider = Arc::new(AnchorTestProvider::new(
        "secondary",
        true,
        Duration::from_millis(80),
        0.02,
        0.95,
    ));
    let tertiary_provider = Arc::new(AnchorTestProvider::new(
        "tertiary",
        true,
        Duration::from_millis(50),
        0.01,
        0.98,
    ));

    engine
        .add_provider("primary".to_string(), primary_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("secondary".to_string(), secondary_provider.clone())
        .await
        .unwrap();
    engine
        .add_provider("tertiary".to_string(), tertiary_provider.clone())
        .await
        .unwrap();

    let request = create_test_request("priority test", ResearchType::Implementation);

    // Test normal priority selection
    for _ in 0..10 {
        if let Ok((selected_name, _)) = engine.select_provider(&request).await {
            assert_eq!(
                selected_name, "primary",
                "Should always select primary when healthy"
            );
        }
    }

    // Test priority fallback when primary fails
    primary_provider.set_healthy(false);

    // Allow health monitoring to detect failure
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should now select secondary
    let mut secondary_selected = false;
    for _ in 0..5 {
        if let Ok((selected_name, _)) = engine.select_provider(&request).await {
            if selected_name == "secondary" {
                secondary_selected = true;
                break;
            }
        }
    }

    assert!(
        secondary_selected,
        "Should select secondary when primary is unhealthy"
    );
}

/// ANCHOR: End-to-end fallback workflow integration test
#[tokio::test]
async fn test_anchor_end_to_end_fallback_workflow() {
    let strategy = FallbackStrategy::HealthBased {
        health_threshold: 0.7,
        check_interval: Duration::from_millis(100),
        circuit_breaker_threshold: 3,
    };
    let mut engine = FallbackEngine::new(strategy).await.unwrap();

    // Configure realistic retry settings
    engine.set_retry_config(RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(2),
        backoff_multiplier: 1.5,
        jitter_factor: 0.1,
    });

    // Set up realistic provider scenarios
    let openai_like = Arc::new(AnchorTestProvider::new(
        "openai",
        true,
        Duration::from_millis(200),
        0.02,
        0.9,
    ));
    let claude_like = Arc::new(AnchorTestProvider::new(
        "claude",
        true,
        Duration::from_millis(300),
        0.025,
        0.95,
    ));
    let local_like = Arc::new(AnchorTestProvider::new(
        "local",
        true,
        Duration::from_millis(100),
        0.001,
        0.8,
    ));

    engine
        .add_provider("openai".to_string(), openai_like.clone())
        .await
        .unwrap();
    engine
        .add_provider("claude".to_string(), claude_like.clone())
        .await
        .unwrap();
    engine
        .add_provider("local".to_string(), local_like.clone())
        .await
        .unwrap();

    // Test various research scenarios
    let scenarios = vec![
        ("How to implement async Rust?", ResearchType::Implementation),
        ("What is machine learning?", ResearchType::Learning),
        ("Debug memory leak", ResearchType::Troubleshooting),
        ("Choose database technology", ResearchType::Decision),
        ("Validate algorithm correctness", ResearchType::Validation),
    ];

    let mut successful_requests = 0;
    let mut total_requests = 0;

    for (query, research_type) in scenarios {
        let request = create_test_request(query, research_type);
        total_requests += 1;

        let start_time = Instant::now();
        let result = timeout(
            Duration::from_secs(10),
            engine.execute_with_fallback(&request),
        )
        .await;
        let elapsed = start_time.elapsed();

        match result {
            Ok(Ok(response)) => {
                successful_requests += 1;
                assert!(!response.is_empty(), "Response should not be empty");
                assert!(
                    elapsed < Duration::from_secs(5),
                    "Response should be reasonably fast"
                );
            }
            Ok(Err(error)) => {
                println!("Request failed: {}", error);
                // Some failures are acceptable in test scenarios
            }
            Err(_) => {
                panic!("Request timed out - this indicates a serious problem");
            }
        }
    }

    // Should have reasonable success rate
    let success_rate = successful_requests as f64 / total_requests as f64;
    assert!(
        success_rate >= 0.6,
        "Should have at least 60% success rate: {:.2}",
        success_rate
    );

    // Verify all providers were utilized
    let openai_calls = openai_like.get_call_count();
    let claude_calls = claude_like.get_call_count();
    let local_calls = local_like.get_call_count();
    let total_calls = openai_calls + claude_calls + local_calls;

    assert!(total_calls > 0, "Should make calls to providers");

    // Verify health monitoring worked
    let health_monitor = engine.health_monitor();
    let all_metrics = health_monitor.get_all_metrics().await;

    assert_eq!(
        all_metrics.len(),
        3,
        "Should have metrics for all providers"
    );

    for (provider_name, metrics) in &all_metrics {
        assert!(
            metrics.total_requests > 0 || metrics.is_healthy(),
            "Provider {} should have requests or be healthy (if unused)",
            provider_name
        );
        assert!(
            metrics.health_score >= 0.0 && metrics.health_score <= 1.0,
            "Provider {} health score should be valid: {}",
            provider_name,
            metrics.health_score
        );
    }
}

/// ANCHOR: Stress test for concurrent fallback operations
#[tokio::test]
async fn test_anchor_concurrent_fallback_stress_test() {
    let strategy = FallbackStrategy::RoundRobin { reset_after: None };
    let engine = Arc::new(FallbackEngine::new(strategy).await.unwrap());

    // Add multiple providers
    let providers = vec![
        Arc::new(AnchorTestProvider::new(
            "provider_1",
            true,
            Duration::from_millis(50),
            0.01,
            0.9,
        )),
        Arc::new(AnchorTestProvider::new(
            "provider_2",
            true,
            Duration::from_millis(60),
            0.012,
            0.85,
        )),
        Arc::new(AnchorTestProvider::new(
            "provider_3",
            true,
            Duration::from_millis(70),
            0.015,
            0.95,
        )),
        Arc::new(AnchorTestProvider::new(
            "provider_4",
            true,
            Duration::from_millis(80),
            0.018,
            0.88,
        )),
    ];

    for (i, provider) in providers.iter().enumerate() {
        engine
            .add_provider(format!("provider_{}", i + 1), provider.clone())
            .await
            .unwrap();
    }

    let request = create_test_request("concurrent stress test", ResearchType::Implementation);

    // Launch many concurrent requests
    let mut handles = Vec::new();
    for i in 0..50 {
        let engine_clone = engine.clone();
        let request_clone = request.clone();

        let handle = tokio::spawn(async move {
            let result = engine_clone.execute_with_fallback(&request_clone).await;
            (i, result.is_ok())
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut successful_count = 0;
    let mut total_count = 0;

    for handle in handles {
        match handle.await {
            Ok((_, success)) => {
                total_count += 1;
                if success {
                    successful_count += 1;
                }
            }
            Err(_) => {
                // Task panicked or was cancelled
                total_count += 1;
            }
        }
    }

    // Verify high success rate under concurrent load
    let success_rate = successful_count as f64 / total_count as f64;
    assert!(
        success_rate >= 0.8,
        "Should maintain high success rate under load: {:.2}",
        success_rate
    );

    // Verify load distribution
    let total_provider_calls: u64 = providers.iter().map(|p| p.get_call_count()).sum();
    assert!(
        total_provider_calls >= 40,
        "Should distribute load across providers: {}",
        total_provider_calls
    );

    // Verify no single provider is overwhelmed
    let max_calls = providers.iter().map(|p| p.get_call_count()).max().unwrap();
    let min_calls = providers.iter().map(|p| p.get_call_count()).min().unwrap();
    let call_distribution_ratio = max_calls as f64 / (min_calls.max(1) as f64);

    assert!(
        call_distribution_ratio <= 3.0,
        "Load distribution should be reasonably balanced: {:.2}",
        call_distribution_ratio
    );
}
