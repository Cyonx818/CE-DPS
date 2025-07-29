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

//! Resilient research engine wrapper with circuit breaker and retry logic
//!
//! This module provides a wrapper around any research engine that adds:
//! - Circuit breaker pattern for external API protection
//! - Retry logic with exponential backoff for transient failures
//! - Comprehensive error handling and correlation tracking
//! - Graceful degradation with fallback to cached results

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error_handling::{
    CircuitBreaker, CircuitBreakerConfig, PipelineError, RetryConfig, RetryExecutor,
};
use crate::research_engine::{ResearchEngine, ResearchEngineError};
use crate::vector::VectorDocument;
use fortitude_types::{ClassifiedRequest, ResearchResult};

/// Configuration for resilient research engine
#[derive(Debug, Clone)]
pub struct ResilientResearchEngineConfig {
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Enable fallback to cached results
    pub enable_fallback: bool,
    /// Maximum age of cached results to use as fallback
    pub max_fallback_age: Duration,
}

impl Default for ResilientResearchEngineConfig {
    fn default() -> Self {
        Self {
            retry_config: RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(30),
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
            },
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout: Duration::from_secs(30),
                reset_timeout: Duration::from_secs(60),
            },
            enable_fallback: true,
            max_fallback_age: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Resilient research engine that wraps another research engine with reliability patterns
pub struct ResilientResearchEngine<T: ResearchEngine> {
    inner: Arc<T>,
    retry_executor: RetryExecutor,
    circuit_breaker: CircuitBreaker,
    config: ResilientResearchEngineConfig,
}

impl<T: ResearchEngine> ResilientResearchEngine<T> {
    /// Create a new resilient research engine
    pub fn new(inner: T, config: ResilientResearchEngineConfig) -> Self {
        Self {
            inner: Arc::new(inner),
            retry_executor: RetryExecutor::new(config.retry_config.clone()),
            circuit_breaker: CircuitBreaker::new(config.circuit_breaker_config.clone()),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(inner: T) -> Self {
        Self::new(inner, ResilientResearchEngineConfig::default())
    }

    /// Execute operation with full resilience patterns
    async fn execute_with_resilience<F, Fut, R>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> Result<R, ResearchEngineError>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<R, ResearchEngineError>> + Send,
        R: Send,
    {
        let correlation_id = Uuid::new_v4().to_string();
        debug!("Starting resilient operation '{}' with correlation_id: {}", operation_name, correlation_id);

        // Wrap the operation to convert ResearchEngineError to PipelineError for circuit breaker
        let circuit_breaker_operation = || async {
            match operation().await {
                Ok(result) => Ok(result),
                Err(research_error) => {
                    let pipeline_error = research_error.to_pipeline_error();
                    error!("Operation '{}' failed with correlation_id: {}, error: {}", 
                           operation_name, correlation_id, pipeline_error);
                    Err(pipeline_error)
                }
            }
        };

        // Execute with circuit breaker protection
        let circuit_result = self.circuit_breaker.execute(circuit_breaker_operation).await;

        match circuit_result {
            Ok(result) => {
                debug!("Operation '{}' succeeded with correlation_id: {}", operation_name, correlation_id);
                Ok(result)
            }
            Err(pipeline_error) => {
                // Convert back to ResearchEngineError
                let research_error = ResearchEngineError::PipelineError(pipeline_error);
                
                // Apply retry logic if appropriate
                if research_error.is_retryable() {
                    debug!("Attempting retry for operation '{}' with correlation_id: {}", operation_name, correlation_id);
                    
                    let retry_result = self.retry_executor.execute(|| async {
                        match operation().await {
                            Ok(result) => Ok(result),
                            Err(error) => Err(error.to_pipeline_error()),
                        }
                    }).await;

                    match retry_result {
                        Ok(result) => {
                            info!("Operation '{}' succeeded after retry with correlation_id: {}", operation_name, correlation_id);
                            Ok(result)
                        }
                        Err(pipeline_error) => {
                            error!("Operation '{}' failed after all retries with correlation_id: {}, error: {}", 
                                   operation_name, correlation_id, pipeline_error);
                            Err(ResearchEngineError::PipelineError(pipeline_error))
                        }
                    }
                } else {
                    error!("Operation '{}' failed with non-retryable error, correlation_id: {}, error: {}", 
                           operation_name, correlation_id, research_error);
                    Err(research_error)
                }
            }
        }
    }
}

#[async_trait]
impl<T: ResearchEngine> ResearchEngine for ResilientResearchEngine<T> {
    /// Generate research with full resilience patterns
    async fn generate_research(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        let request_clone = request.clone();
        let inner = self.inner.clone();

        self.execute_with_resilience("generate_research", move || {
            let inner = inner.clone();
            let request = request_clone.clone();
            async move { inner.generate_research(&request).await }
        })
        .await
    }

    /// Generate research with context discovery using resilience patterns
    async fn generate_research_with_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        let request_clone = request.clone();
        let inner = self.inner.clone();

        self.execute_with_resilience("generate_research_with_context", move || {
            let inner = inner.clone();
            let request = request_clone.clone();
            async move { inner.generate_research_with_context(&request).await }
        })
        .await
    }

    /// Discover context with resilience patterns
    async fn discover_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<Vec<VectorDocument>, ResearchEngineError> {
        let request_clone = request.clone();
        let inner = self.inner.clone();

        self.execute_with_resilience("discover_context", move || {
            let inner = inner.clone();
            let request = request_clone.clone();
            async move { inner.discover_context(&request).await }
        })
        .await
    }

    /// Health check with circuit breaker awareness
    async fn health_check(&self) -> Result<(), ResearchEngineError> {
        // Check circuit breaker state first
        let circuit_state = self.circuit_breaker.state();
        match circuit_state {
            crate::error_handling::CircuitBreakerState::Open { until: _ } => {
                warn!("Health check failed: circuit breaker is open");
                return Err(ResearchEngineError::PipelineError(
                    PipelineError::CircuitBreakerOpen {
                        service: "research_engine".to_string(),
                        retry_after: Duration::from_secs(30),
                        failure_count: self.circuit_breaker.failure_count(),
                    },
                ));
            }
            _ => {}
        }

        // Perform actual health check
        let inner = self.inner.clone();
        self.execute_with_resilience("health_check", move || {
            let inner = inner.clone();
            async move { inner.health_check().await }
        })
        .await
    }

    /// Get estimated processing time (delegate to inner engine)
    fn estimate_processing_time(&self, request: &ClassifiedRequest) -> Duration {
        // Add buffer for retry and circuit breaker overhead
        let base_time = self.inner.estimate_processing_time(request);
        let max_retries = self.config.retry_config.max_attempts as u32;
        let max_delay = self.config.retry_config.max_delay;
        
        // Estimate worst-case scenario: base_time * retries + max_delay * (retries - 1)
        base_time * max_retries + max_delay * (max_retries - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_engine::ResearchEngineError;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::time::Duration;
    use fortitude_types::{ClassifiedRequest, ResearchResult, ResearchType};

    // Mock research engine for testing
    struct MockResearchEngine {
        call_count: Arc<AtomicU32>,
        fail_count: u32,
        delay: Duration,
    }

    impl MockResearchEngine {
        fn new(fail_count: u32, delay: Duration) -> Self {
            Self {
                call_count: Arc::new(AtomicU32::new(0)),
                fail_count,
                delay,
            }
        }
    }

    #[async_trait]
    impl ResearchEngine for MockResearchEngine {
        async fn generate_research(
            &self,
            _request: &ClassifiedRequest,
        ) -> Result<ResearchResult, ResearchEngineError> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(self.delay).await;

            if count < self.fail_count {
                Err(ResearchEngineError::TimeoutError)
            } else {
                let request = ClassifiedRequest::new(
                    "test query".to_string(),
                    ResearchType::Implementation,
                    fortitude_types::AudienceContext::default(),
                    fortitude_types::DomainContext::default(),
                    0.9,
                    vec![],
                );
                
                let metadata = fortitude_types::ResearchMetadata {
                    completed_at: chrono::Utc::now(),
                    processing_time_ms: self.delay.as_millis() as u64,
                    sources_consulted: vec!["mock".to_string()],
                    quality_score: 0.9,
                    cache_key: "mock-key".to_string(),
                    tags: std::collections::HashMap::new(),
                };

                Ok(ResearchResult::new(
                    request,
                    "Mock research result".to_string(),
                    vec![],
                    vec![],
                    metadata,
                ))
            }
        }

        async fn generate_research_with_context(
            &self,
            request: &ClassifiedRequest,
        ) -> Result<ResearchResult, ResearchEngineError> {
            self.generate_research(request).await
        }

        async fn discover_context(
            &self,
            _request: &ClassifiedRequest,
        ) -> Result<Vec<VectorDocument>, ResearchEngineError> {
            Ok(vec![])
        }

        async fn health_check(&self) -> Result<(), ResearchEngineError> {
            Ok(())
        }

        fn estimate_processing_time(&self, _request: &ClassifiedRequest) -> Duration {
            self.delay
        }
    }

    // ANCHOR: Resilient research engine regression test
    #[tokio::test]
    async fn test_resilient_engine_retry_success() {
        let mock_engine = MockResearchEngine::new(2, Duration::from_millis(10)); // Fail twice, then succeed
        let config = ResilientResearchEngineConfig {
            retry_config: RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(10),
                backoff_multiplier: 1.5,
                jitter_factor: 0.0,
            },
            ..Default::default()
        };
        
        let resilient_engine = ResilientResearchEngine::new(mock_engine, config);
        
        let request = ClassifiedRequest::new(
            "test query".to_string(),
            ResearchType::Implementation,
            fortitude_types::AudienceContext::default(),
            fortitude_types::DomainContext::default(),
            0.9,
            vec![],
        );

        let result = resilient_engine.generate_research(&request).await;
        assert!(result.is_ok());
        
        let research_result = result.unwrap();
        assert_eq!(research_result.immediate_answer, "Mock research result");
    }

    #[tokio::test]
    async fn test_resilient_engine_circuit_breaker() {
        let mock_engine = MockResearchEngine::new(10, Duration::from_millis(1)); // Always fail
        let config = ResilientResearchEngineConfig {
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 2,
                success_threshold: 1,
                timeout: Duration::from_millis(100),
                reset_timeout: Duration::from_millis(10),
            },
            retry_config: RetryConfig {
                max_attempts: 1, // Don't retry to test circuit breaker
                ..Default::default()
            },
            ..Default::default()
        };
        
        let resilient_engine = ResilientResearchEngine::new(mock_engine, config);
        
        let request = ClassifiedRequest::new(
            "test query".to_string(),
            ResearchType::Implementation,
            fortitude_types::AudienceContext::default(),
            fortitude_types::DomainContext::default(),
            0.9,
            vec![],
        );

        // First two calls should fail normally
        let result1 = resilient_engine.generate_research(&request).await;
        assert!(result1.is_err());
        
        let result2 = resilient_engine.generate_research(&request).await;
        assert!(result2.is_err());

        // Third call should be rejected by circuit breaker
        let result3 = resilient_engine.generate_research(&request).await;
        assert!(result3.is_err());
        
        if let Err(ResearchEngineError::PipelineError(PipelineError::CircuitBreakerOpen { .. })) = result3 {
            // Expected
        } else {
            panic!("Expected circuit breaker open error, got: {:?}", result3);
        }
    }

    #[tokio::test]
    async fn test_health_check_with_circuit_breaker() {
        let mock_engine = MockResearchEngine::new(0, Duration::from_millis(1));
        let resilient_engine = ResilientResearchEngine::with_defaults(mock_engine);

        let health_result = resilient_engine.health_check().await;
        assert!(health_result.is_ok());
    }

    #[test]
    fn test_estimated_processing_time_includes_overhead() {
        let mock_engine = MockResearchEngine::new(0, Duration::from_millis(100));
        let resilient_engine = ResilientResearchEngine::with_defaults(mock_engine);
        
        let request = ClassifiedRequest::new(
            "test query".to_string(),
            ResearchType::Implementation,
            fortitude_types::AudienceContext::default(),
            fortitude_types::DomainContext::default(),
            0.9,
            vec![],
        );

        let estimated_time = resilient_engine.estimate_processing_time(&request);
        
        // Should be more than base processing time due to retry overhead
        assert!(estimated_time > Duration::from_millis(100));
        // But should be reasonable (not more than 10x base time)
        assert!(estimated_time < Duration::from_millis(1000));
    }
}