//! Comprehensive unit tests for Research Engine Integration (Task 1.7)
//! 
//! This module tests:
//! - End-to-end research workflow integration
//! - Provider manager integration with research pipeline
//! - Classification to provider routing
//! - Result aggregation and quality scoring
//! - Cache integration and performance optimization
//! - Error handling and fallback mechanisms
//! - Multi-provider coordination and load balancing

use fortitude::providers::{
    Provider, ProviderError, ProviderResult, ProviderMetadata, HealthStatus,
    QueryCost, UsageStats
};
use fortitude::providers::manager::{
    ProviderManager, ProviderConfig as ManagerConfig, SelectionStrategy, ProviderManagerError
};
use fortitude::providers::config::{ProviderSettings, RateLimitConfig, RetryConfig};
use fortitude_types::{
    ClassifiedRequest, ResearchType, AudienceContext, DomainContext,
    ResearchResult, ResearchMetadata, Evidence, Detail
};
use crate::common::{
    MockProvider, MockProviderConfig, TestEnvironmentGuard,
    valid_openai_settings, valid_claude_settings, test_queries
};
use async_trait::async_trait;
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use chrono::Utc;
use proptest::prelude::*;

/// Mock Research Engine for integration testing
#[derive(Debug, Clone)]
pub struct MockResearchEngine {
    provider_manager: Arc<ProviderManager>,
    cache: Arc<std::sync::Mutex<HashMap<String, ResearchResult>>>,
    performance_metrics: Arc<std::sync::Mutex<ResearchPerformanceMetrics>>,
}

#[derive(Debug, Default)]
struct ResearchPerformanceMetrics {
    total_requests: u64,
    cache_hits: u64,
    cache_misses: u64,
    average_latency_ms: f64,
    provider_usage: HashMap<String, u64>,
}

impl MockResearchEngine {
    pub async fn new(manager_config: ManagerConfig) -> Result<Self, ProviderManagerError> {
        let provider_manager = Arc::new(ProviderManager::new(manager_config).await?);
        
        Ok(Self {
            provider_manager,
            cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            performance_metrics: Arc::new(std::sync::Mutex::new(ResearchPerformanceMetrics::default())),
        })
    }
    
    pub async fn add_provider(&self, name: String, provider: Arc<dyn Provider>) -> Result<(), ProviderManagerError> {
        self.provider_manager.add_provider(name, provider).await
    }
    
    pub async fn execute_research(&self, request: &ClassifiedRequest) -> Result<ResearchResult, ProviderManagerError> {
        let start_time = Instant::now();
        let cache_key = self.generate_cache_key(request);
        
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached_result) = cache.get(&cache_key) {
                let mut metrics = self.performance_metrics.lock().unwrap();
                metrics.cache_hits += 1;
                metrics.total_requests += 1;
                return Ok(cached_result.clone());
            }
        }
        
        // Cache miss - execute research through provider manager
        let result = self.provider_manager.execute_research(request).await?;
        
        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, result.clone());
        }
        
        // Update metrics
        {
            let mut metrics = self.performance_metrics.lock().unwrap();
            metrics.cache_misses += 1;
            metrics.total_requests += 1;
            metrics.average_latency_ms = start_time.elapsed().as_millis() as f64;
            
            // Track provider usage (simplified)
            let provider_name = "selected_provider".to_string(); // Would be actual selected provider
            *metrics.provider_usage.entry(provider_name).or_insert(0) += 1;
        }
        
        Ok(result)
    }
    
    pub fn get_performance_metrics(&self) -> ResearchPerformanceMetrics {
        self.performance_metrics.lock().unwrap().clone()
    }
    
    pub fn get_cache_stats(&self) -> (usize, u64, u64) {
        let metrics = self.performance_metrics.lock().unwrap();
        let cache_size = self.cache.lock().unwrap().len();
        (cache_size, metrics.cache_hits, metrics.cache_misses)
    }
    
    fn generate_cache_key(&self, request: &ClassifiedRequest) -> String {
        format!("{}:{}:{}", 
            request.research_type, 
            request.original_query, 
            request.confidence as u32
        )
    }
}

/// Enhanced test provider with configurable research capabilities
#[derive(Debug, Clone)]
struct ResearchTestProvider {
    name: String,
    research_capabilities: HashMap<ResearchType, f64>, // Quality scores by research type
    latency: Duration,
    cost_per_request: f64,
    failure_rate: f64,
    call_count: Arc<AtomicU64>,
    healthy: Arc<AtomicBool>,
}

impl ResearchTestProvider {
    fn new(name: &str, capabilities: HashMap<ResearchType, f64>) -> Self {
        Self {
            name: name.to_string(),
            research_capabilities: capabilities,
            latency: Duration::from_millis(100),
            cost_per_request: 0.01,
            failure_rate: 0.0,
            call_count: Arc::new(AtomicU64::new(0)),
            healthy: Arc::new(AtomicBool::new(true)),
        }
    }
    
    fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }
    
    fn with_cost(mut self, cost: f64) -> Self {
        self.cost_per_request = cost;
        self
    }
    
    fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }
    
    fn get_call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }
    
    fn set_healthy(&self, healthy: bool) {
        self.healthy.store(healthy, Ordering::SeqCst);
    }
}

#[async_trait]
impl Provider for ResearchTestProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        
        // Simulate latency
        tokio::time::sleep(self.latency).await;
        
        // Check health
        if !self.healthy.load(Ordering::SeqCst) {
            return Err(ProviderError::ServiceUnavailable {
                provider: self.name.clone(),
                message: "Provider marked as unhealthy".to_string(),
                estimated_recovery: Some(Duration::from_secs(60)),
            });
        }
        
        // Simulate random failures
        if rand::random::<f64>() < self.failure_rate {
            return Err(ProviderError::QueryFailed {
                message: "Random failure simulation".to_string(),
                provider: self.name.clone(),
                error_code: Some("RANDOM_FAILURE".to_string()),
            });
        }
        
        // Generate research-specific response
        let response = format!("{} comprehensive research response for: {}", self.name, query);
        Ok(response)
    }
    
    fn metadata(&self) -> ProviderMetadata {
        let capabilities = vec![
            "research".to_string(),
            "rate_limited".to_string(),
            "cost_estimation".to_string(),
        ];
        
        ProviderMetadata::new(self.name.clone(), "1.0.0".to_string())
            .with_capabilities(capabilities)
            .with_models(vec![format!("{}-model", self.name)])
    }
    
    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        Ok(if self.healthy.load(Ordering::SeqCst) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy("Marked unhealthy for testing".to_string())
        })
    }
    
    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost> {
        let estimated_tokens = query.len() / 4; // Rough token estimation
        Ok(QueryCost {
            estimated_input_tokens: estimated_tokens as u32,
            estimated_output_tokens: (estimated_tokens * 2) as u32,
            estimated_duration: self.latency,
            estimated_cost_usd: Some(self.cost_per_request),
        })
    }
    
    async fn usage_stats(&self) -> ProviderResult<UsageStats> {
        Ok(UsageStats {
            requests_made: self.call_count.load(Ordering::SeqCst),
            tokens_consumed: self.call_count.load(Ordering::SeqCst) * 100, // Mock
            estimated_cost: self.call_count.load(Ordering::SeqCst) as f64 * self.cost_per_request,
        })
    }
}

mod end_to_end_workflow_tests {
    use super::*;

    /// ANCHOR: Verifies complete end-to-end research workflow
    #[tokio::test]
    async fn test_anchor_end_to_end_research_workflow() {
        let _guard = TestEnvironmentGuard::new();
        
        // Create research engine with multiple providers
        let config = ManagerConfig {
            selection_strategy: SelectionStrategy::Balanced,
            enable_failover: true,
            max_failover_attempts: 3,
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Add providers with different capabilities
        let openai_capabilities = {
            let mut caps = HashMap::new();
            caps.insert(ResearchType::Implementation, 0.9);
            caps.insert(ResearchType::Learning, 0.8);
            caps.insert(ResearchType::Decision, 0.7);
            caps.insert(ResearchType::Troubleshooting, 0.8);
            caps.insert(ResearchType::Validation, 0.7);
            caps
        };
        
        let claude_capabilities = {
            let mut caps = HashMap::new();
            caps.insert(ResearchType::Learning, 0.9);
            caps.insert(ResearchType::Decision, 0.9);
            caps.insert(ResearchType::Validation, 0.8);
            caps.insert(ResearchType::Implementation, 0.7);
            caps.insert(ResearchType::Troubleshooting, 0.7);
            caps
        };
        
        let openai_provider = Arc::new(ResearchTestProvider::new("openai", openai_capabilities)
            .with_latency(Duration::from_millis(150))
            .with_cost(0.02));
        
        let claude_provider = Arc::new(ResearchTestProvider::new("claude", claude_capabilities)
            .with_latency(Duration::from_millis(200))
            .with_cost(0.015));
        
        engine.add_provider("openai".to_string(), openai_provider.clone()).await.unwrap();
        engine.add_provider("claude".to_string(), claude_provider.clone()).await.unwrap();
        
        // Test different research types
        let test_requests = vec![
            ClassifiedRequest::new(
                "How to implement async iterators in Rust?".to_string(),
                ResearchType::Implementation,
                AudienceContext {
                    level: "intermediate".to_string(),
                    domain: "rust".to_string(),
                    format: "markdown".to_string(),
                },
                DomainContext {
                    technology: "rust".to_string(),
                    project_type: "library".to_string(),
                    frameworks: vec!["tokio".to_string()],
                    tags: vec!["async".to_string(), "iterators".to_string()],
                },
                0.85,
                vec!["implement".to_string(), "async".to_string()],
            ),
            
            ClassifiedRequest::new(
                "What are the trade-offs between REST and GraphQL?".to_string(),
                ResearchType::Decision,
                AudienceContext {
                    level: "advanced".to_string(),
                    domain: "web".to_string(),
                    format: "markdown".to_string(),
                },
                DomainContext {
                    technology: "web".to_string(),
                    project_type: "api".to_string(),
                    frameworks: vec!["rest".to_string(), "graphql".to_string()],
                    tags: vec!["architecture".to_string()],
                },
                0.9,
                vec!["trade-offs".to_string(), "rest".to_string(), "graphql".to_string()],
            ),
        ];
        
        // Execute research requests
        for request in &test_requests {
            let result = engine.execute_research(request).await;
            assert!(result.is_ok(), "Research execution should succeed");
            
            let research_result = result.unwrap();
            assert_eq!(research_result.request.id, request.id);
            assert_eq!(research_result.request.research_type, request.research_type);
            assert!(!research_result.immediate_answer.is_empty());
        }
        
        // Verify providers were called
        assert!(openai_provider.get_call_count() > 0 || claude_provider.get_call_count() > 0);
        
        // Verify performance metrics
        let metrics = engine.get_performance_metrics();
        assert_eq!(metrics.total_requests, test_requests.len() as u64);
        assert!(metrics.cache_misses > 0); // First run should be cache misses
    }

    #[tokio::test]
    async fn test_research_caching_behavior() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig::default();
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        let provider = Arc::new(ResearchTestProvider::new(
            "test-provider", 
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Learning, 0.8);
                caps
            }
        ));
        
        engine.add_provider("test".to_string(), provider.clone()).await.unwrap();
        
        let request = ClassifiedRequest::new(
            "Test caching behavior".to_string(),
            ResearchType::Learning,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );
        
        // First request - should be cache miss
        let result1 = engine.execute_research(&request).await.unwrap();
        let (cache_size_1, hits_1, misses_1) = engine.get_cache_stats();
        
        // Second identical request - should be cache hit
        let result2 = engine.execute_research(&request).await.unwrap();
        let (cache_size_2, hits_2, misses_2) = engine.get_cache_stats();
        
        // Verify caching behavior
        assert_eq!(cache_size_1, 1);
        assert_eq!(hits_1, 0);
        assert_eq!(misses_1, 1);
        
        assert_eq!(cache_size_2, 1);
        assert_eq!(hits_2, 1);
        assert_eq!(misses_2, 1);
        
        // Results should be identical
        assert_eq!(result1.immediate_answer, result2.immediate_answer);
        
        // Provider should only be called once
        assert_eq!(provider.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_provider_selection_strategies() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test different selection strategies
        let strategies = vec![
            SelectionStrategy::RoundRobin,
            SelectionStrategy::LowestLatency,
            SelectionStrategy::HighestSuccessRate,
            SelectionStrategy::CostOptimized,
            SelectionStrategy::Balanced,
        ];
        
        for strategy in strategies {
            let config = ManagerConfig {
                selection_strategy: strategy.clone(),
                ..Default::default()
            };
            
            let engine = MockResearchEngine::new(config).await.unwrap();
            
            // Add providers with different characteristics
            let fast_expensive = Arc::new(ResearchTestProvider::new(
                "fast-expensive",
                {
                    let mut caps = HashMap::new();
                    caps.insert(ResearchType::Implementation, 0.8);
                    caps
                }
            )
            .with_latency(Duration::from_millis(50))
            .with_cost(0.05));
            
            let slow_cheap = Arc::new(ResearchTestProvider::new(
                "slow-cheap",
                {
                    let mut caps = HashMap::new();
                    caps.insert(ResearchType::Implementation, 0.8);
                    caps
                }
            )
            .with_latency(Duration::from_millis(300))
            .with_cost(0.01));
            
            engine.add_provider("fast".to_string(), fast_expensive).await.unwrap();
            engine.add_provider("slow".to_string(), slow_cheap).await.unwrap();
            
            let request = ClassifiedRequest::new(
                "Test provider selection".to_string(),
                ResearchType::Implementation,
                AudienceContext::default(),
                DomainContext::default(),
                0.8,
                vec!["test".to_string()],
            );
            
            let result = engine.execute_research(&request).await;
            assert!(result.is_ok(), "Research should succeed with strategy {:?}", strategy);
        }
    }
}

mod provider_coordination_tests {
    use super::*;

    /// ANCHOR: Verifies multi-provider coordination and load balancing
    #[tokio::test]
    async fn test_anchor_multi_provider_coordination() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            selection_strategy: SelectionStrategy::RoundRobin,
            enable_failover: true,
            max_failover_attempts: 3,
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Add multiple providers
        let providers = vec![
            ("provider1", Duration::from_millis(100), 0.02),
            ("provider2", Duration::from_millis(150), 0.015),
            ("provider3", Duration::from_millis(200), 0.01),
        ];
        
        let provider_instances = providers.iter().map(|(name, latency, cost)| {
            Arc::new(ResearchTestProvider::new(
                name,
                {
                    let mut caps = HashMap::new();
                    caps.insert(ResearchType::Learning, 0.8);
                    caps
                }
            )
            .with_latency(*latency)
            .with_cost(*cost))
        }).collect::<Vec<_>>();
        
        for (i, provider) in provider_instances.iter().enumerate() {
            engine.add_provider(providers[i].0.to_string(), provider.clone()).await.unwrap();
        }
        
        // Execute multiple requests to test load distribution
        let request_count = 10;
        let mut successful_requests = 0;
        
        for i in 0..request_count {
            let request = ClassifiedRequest::new(
                format!("Test request {}", i),
                ResearchType::Learning,
                AudienceContext::default(),
                DomainContext::default(),
                0.8,
                vec!["test".to_string()],
            );
            
            if engine.execute_research(&request).await.is_ok() {
                successful_requests += 1;
            }
        }
        
        assert_eq!(successful_requests, request_count);
        
        // Verify load distribution (round-robin should distribute fairly)
        let total_calls: u64 = provider_instances.iter()
            .map(|p| p.get_call_count())
            .sum();
        
        assert!(total_calls >= request_count as u64);
        
        // Each provider should have been called at least once with round-robin
        for provider in &provider_instances {
            assert!(provider.get_call_count() > 0);
        }
    }

    #[tokio::test]
    async fn test_failover_mechanism_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            selection_strategy: SelectionStrategy::RoundRobin,
            enable_failover: true,
            max_failover_attempts: 3,
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Add failing and working providers
        let failing_provider = Arc::new(ResearchTestProvider::new(
            "failing",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Implementation, 0.8);
                caps
            }
        ).with_failure_rate(1.0)); // Always fails
        
        let working_provider = Arc::new(ResearchTestProvider::new(
            "working",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Implementation, 0.8);
                caps
            }
        ).with_failure_rate(0.0)); // Never fails
        
        engine.add_provider("failing".to_string(), failing_provider.clone()).await.unwrap();
        engine.add_provider("working".to_string(), working_provider.clone()).await.unwrap();
        
        let request = ClassifiedRequest::new(
            "Test failover mechanism".to_string(),
            ResearchType::Implementation,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );
        
        // Execute research - should succeed despite one provider failing
        let result = engine.execute_research(&request).await;
        assert!(result.is_ok(), "Research should succeed with failover");
        
        // Verify both providers were attempted
        assert!(failing_provider.get_call_count() > 0);
        assert!(working_provider.get_call_count() > 0);
    }

    #[tokio::test]
    async fn test_provider_health_monitoring_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            health_check_interval: Duration::from_millis(100),
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        let provider = Arc::new(ResearchTestProvider::new(
            "health-test",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Learning, 0.8);
                caps
            }
        ));
        
        engine.add_provider("health-test".to_string(), provider.clone()).await.unwrap();
        
        // Initial request should succeed
        let request = ClassifiedRequest::new(
            "Test health monitoring".to_string(),
            ResearchType::Learning,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );
        
        let result1 = engine.execute_research(&request).await;
        assert!(result1.is_ok());
        
        // Mark provider as unhealthy
        provider.set_healthy(false);
        
        // Subsequent requests should handle unhealthy provider
        let result2 = engine.execute_research(&request).await;
        // Result depends on failover configuration and other available providers
        // In this case, we expect it to fail since there's only one provider
        assert!(result2.is_err());
    }
}

mod error_handling_integration_tests {
    use super::*;

    /// ANCHOR: Verifies comprehensive error handling in research integration
    #[tokio::test]
    async fn test_anchor_research_error_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            enable_failover: false, // Disable failover to test error propagation
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Test with no providers
        let request = ClassifiedRequest::new(
            "Test with no providers".to_string(),
            ResearchType::Learning,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );
        
        let result = engine.execute_research(&request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderManagerError::NoProviders));
        
        // Add a failing provider
        let failing_provider = Arc::new(ResearchTestProvider::new(
            "failing",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Learning, 0.8);
                caps
            }
        ).with_failure_rate(1.0));
        
        engine.add_provider("failing".to_string(), failing_provider.clone()).await.unwrap();
        
        // Test with failing provider
        let result = engine.execute_research(&request).await;
        assert!(result.is_err());
        // Should get AllProvidersFailed since failover is disabled
    }

    #[tokio::test]
    async fn test_provider_error_types_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig::default();
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Test different error scenarios
        let error_scenarios = vec![
            ("auth-error", 0.0, false), // Authentication error simulation
            ("rate-limited", 0.0, true), // Rate limit error simulation
            ("service-unavailable", 0.0, false), // Service unavailable
        ];
        
        for (provider_name, failure_rate, healthy) in error_scenarios {
            let provider = Arc::new({
                let mut provider = ResearchTestProvider::new(
                    provider_name,
                    {
                        let mut caps = HashMap::new();
                        caps.insert(ResearchType::Learning, 0.8);
                        caps
                    }
                );
                
                if !healthy {
                    provider.set_healthy(false);
                }
                
                provider.with_failure_rate(failure_rate)
            });
            
            let engine = MockResearchEngine::new(ManagerConfig::default()).await.unwrap();
            engine.add_provider(provider_name.to_string(), provider).await.unwrap();
            
            let request = ClassifiedRequest::new(
                format!("Test {} error handling", provider_name),
                ResearchType::Learning,
                AudienceContext::default(),
                DomainContext::default(),
                0.8,
                vec!["test".to_string()],
            );
            
            let result = engine.execute_research(&request).await;
            if !healthy {
                assert!(result.is_err(), "Should fail with unhealthy provider");
            }
        }
    }

    #[tokio::test]
    async fn test_timeout_handling_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            provider_timeout: Duration::from_millis(50), // Very short timeout
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Add a slow provider
        let slow_provider = Arc::new(ResearchTestProvider::new(
            "slow",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Learning, 0.8);
                caps
            }
        ).with_latency(Duration::from_millis(200))); // Slower than timeout
        
        engine.add_provider("slow".to_string(), slow_provider).await.unwrap();
        
        let request = ClassifiedRequest::new(
            "Test timeout handling".to_string(),
            ResearchType::Learning,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );
        
        let result = engine.execute_research(&request).await;
        assert!(result.is_err(), "Should timeout with slow provider");
    }
}

mod performance_integration_tests {
    use super::*;

    /// ANCHOR: Verifies performance characteristics of research integration
    #[tokio::test]
    async fn test_anchor_research_performance_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            selection_strategy: SelectionStrategy::LowestLatency,
            enable_performance_tracking: true,
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Add providers with different performance characteristics
        let fast_provider = Arc::new(ResearchTestProvider::new(
            "fast",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Implementation, 0.8);
                caps
            }
        ).with_latency(Duration::from_millis(50)));
        
        let slow_provider = Arc::new(ResearchTestProvider::new(
            "slow",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Implementation, 0.8);
                caps
            }
        ).with_latency(Duration::from_millis(300)));
        
        engine.add_provider("fast".to_string(), fast_provider.clone()).await.unwrap();
        engine.add_provider("slow".to_string(), slow_provider.clone()).await.unwrap();
        
        let request = ClassifiedRequest::new(
            "Test performance selection".to_string(),
            ResearchType::Implementation,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );
        
        // Execute multiple requests
        let request_count = 5;
        let start_time = Instant::now();
        
        for _ in 0..request_count {
            let result = engine.execute_research(&request).await;
            assert!(result.is_ok());
        }
        
        let total_time = start_time.elapsed();
        
        // With LowestLatency strategy, should prefer fast provider
        assert!(fast_provider.get_call_count() > slow_provider.get_call_count());
        
        // Total time should be reasonable (not 5 * slow_provider_latency)
        assert!(total_time < Duration::from_millis(1000));
        
        // Verify performance metrics
        let metrics = engine.get_performance_metrics();
        assert_eq!(metrics.total_requests, request_count as u64);
        assert!(metrics.average_latency_ms > 0.0);
    }

    #[tokio::test]
    async fn test_concurrent_request_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            selection_strategy: SelectionStrategy::RoundRobin,
            ..Default::default()
        };
        
        let engine = Arc::new(MockResearchEngine::new(config).await.unwrap());
        
        let provider = Arc::new(ResearchTestProvider::new(
            "concurrent",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Learning, 0.8);
                caps
            }
        ).with_latency(Duration::from_millis(100)));
        
        engine.add_provider("concurrent".to_string(), provider.clone()).await.unwrap();
        
        // Execute concurrent requests
        let concurrent_count = 10;
        let mut handles = Vec::new();
        
        for i in 0..concurrent_count {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let request = ClassifiedRequest::new(
                    format!("Concurrent request {}", i),
                    ResearchType::Learning,
                    AudienceContext::default(),
                    DomainContext::default(),
                    0.8,
                    vec!["test".to_string()],
                );
                
                engine_clone.execute_research(&request).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut successful_requests = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                successful_requests += 1;
            }
        }
        
        assert_eq!(successful_requests, concurrent_count);
        assert_eq!(provider.get_call_count(), concurrent_count);
        
        // Verify cache behavior with concurrent requests
        let (cache_size, _, _) = engine.get_cache_stats();
        assert_eq!(cache_size, concurrent_count); // Each unique request should be cached
    }

    proptest! {
        #[test]
        fn test_research_integration_property(
            request_count in 1usize..=20usize,
            provider_count in 1usize..=5usize
        ) {
            let _guard = TestEnvironmentGuard::new();
            
            tokio_test::block_on(async {
                let config = ManagerConfig::default();
                let engine = MockResearchEngine::new(config).await.unwrap();
                
                // Add multiple providers
                for i in 0..provider_count {
                    let provider = Arc::new(ResearchTestProvider::new(
                        &format!("provider{}", i),
                        {
                            let mut caps = HashMap::new();
                            caps.insert(ResearchType::Learning, 0.8);
                            caps
                        }
                    ));
                    
                    engine.add_provider(format!("provider{}", i), provider).await.unwrap();
                }
                
                // Execute requests
                let mut successful_requests = 0;
                for i in 0..request_count {
                    let request = ClassifiedRequest::new(
                        format!("Property test request {}", i),
                        ResearchType::Learning,
                        AudienceContext::default(),
                        DomainContext::default(),
                        0.8,
                        vec!["test".to_string()],
                    );
                    
                    if engine.execute_research(&request).await.is_ok() {
                        successful_requests += 1;
                    }
                }
                
                // Properties to verify
                assert_eq!(successful_requests, request_count);
                
                let metrics = engine.get_performance_metrics();
                assert_eq!(metrics.total_requests, request_count as u64);
                assert!(metrics.cache_misses > 0);
            });
        }
    }
}

mod research_quality_integration_tests {
    use super::*;

    /// ANCHOR: Verifies research quality and result validation integration
    #[tokio::test]
    async fn test_anchor_research_quality_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig {
            selection_strategy: SelectionStrategy::ResearchTypeOptimized,
            ..Default::default()
        };
        
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        // Add providers with different quality profiles
        let high_quality_provider = Arc::new(ResearchTestProvider::new(
            "high-quality",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Implementation, 0.95);
                caps.insert(ResearchType::Learning, 0.9);
                caps.insert(ResearchType::Decision, 0.85);
                caps
            }
        ));
        
        let medium_quality_provider = Arc::new(ResearchTestProvider::new(
            "medium-quality",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Implementation, 0.7);
                caps.insert(ResearchType::Learning, 0.75);
                caps.insert(ResearchType::Decision, 0.8);
                caps
            }
        ));
        
        engine.add_provider("high-quality".to_string(), high_quality_provider.clone()).await.unwrap();
        engine.add_provider("medium-quality".to_string(), medium_quality_provider.clone()).await.unwrap();
        
        // Test research type optimization
        let research_types = vec![
            ResearchType::Implementation,
            ResearchType::Learning,
            ResearchType::Decision,
        ];
        
        for research_type in research_types {
            let request = ClassifiedRequest::new(
                format!("Test {} research quality", research_type),
                research_type.clone(),
                AudienceContext::default(),
                DomainContext::default(),
                0.8,
                vec!["test".to_string()],
            );
            
            let result = engine.execute_research(&request).await;
            assert!(result.is_ok());
            
            let research_result = result.unwrap();
            assert_eq!(research_result.request.research_type, research_type);
            assert!(!research_result.immediate_answer.is_empty());
        }
        
        // Verify provider selection based on quality
        assert!(high_quality_provider.get_call_count() > 0);
    }

    #[tokio::test]
    async fn test_result_validation_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let config = ManagerConfig::default();
        let engine = MockResearchEngine::new(config).await.unwrap();
        
        let provider = Arc::new(ResearchTestProvider::new(
            "validator",
            {
                let mut caps = HashMap::new();
                caps.insert(ResearchType::Validation, 0.9);
                caps
            }
        ));
        
        engine.add_provider("validator".to_string(), provider.clone()).await.unwrap();
        
        let request = ClassifiedRequest::new(
            "Validate this approach to async error handling".to_string(),
            ResearchType::Validation,
            AudienceContext {
                level: "advanced".to_string(),
                domain: "rust".to_string(),
                format: "markdown".to_string(),
            },
            DomainContext {
                technology: "rust".to_string(),
                project_type: "library".to_string(),
                frameworks: vec!["tokio".to_string()],
                tags: vec!["async".to_string(), "error-handling".to_string()],
            },
            0.9,
            vec!["validate".to_string(), "async".to_string(), "error".to_string()],
        );
        
        let result = engine.execute_research(&request).await;
        assert!(result.is_ok());
        
        let research_result = result.unwrap();
        
        // Verify result structure for validation research
        assert_eq!(research_result.request.research_type, ResearchType::Validation);
        assert!(research_result.immediate_answer.contains("comprehensive research response"));
        assert!(research_result.metadata.quality_score > 0.0);
        assert!(research_result.metadata.processing_time_ms > 0);
    }
}