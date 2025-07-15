// ABOUTME: Adapter to integrate provider system with research engine
//! This module provides adapters to integrate the multi-LLM provider system
//! with the research engine trait implementations, enabling seamless provider
//! switching and fallback strategies in research operations.

use crate::providers::{
    manager::{ProviderManager, ProviderPerformance},
    HealthStatus,
};
use fortitude_core::multi_provider_research_engine::{
    ProviderHealthStatus, ProviderManagerTrait, ProviderPerformanceStats,
};
use fortitude_types::ClassifiedRequest;

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Adapter that implements ProviderManagerTrait for ProviderManager
pub struct ProviderManagerAdapter {
    provider_manager: Arc<ProviderManager>,
}

impl ProviderManagerAdapter {
    /// Create a new adapter wrapping the provider manager
    pub fn new(provider_manager: Arc<ProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Get reference to the underlying provider manager
    pub fn inner(&self) -> &Arc<ProviderManager> {
        &self.provider_manager
    }

    /// Convert ProviderPerformance to ProviderPerformanceStats
    #[allow(dead_code)] // TODO: Will be used for performance metrics conversion
    fn convert_performance_stats(&self, perf: &ProviderPerformance) -> ProviderPerformanceStats {
        ProviderPerformanceStats {
            total_requests: perf.total_requests,
            successful_requests: perf.successful_requests,
            failed_requests: perf.failed_requests,
            average_latency: perf.average_latency(),
            average_quality: perf.average_quality(),
            success_rate: perf.success_rate(),
        }
    }

    /// Convert HealthStatus to ProviderHealthStatus
    #[allow(dead_code)] // TODO: Will be used for health status conversion
    fn convert_health_status(&self, status: &HealthStatus) -> ProviderHealthStatus {
        match status {
            HealthStatus::Healthy => ProviderHealthStatus::Healthy,
            HealthStatus::Degraded(reason) => ProviderHealthStatus::Degraded(reason.clone()),
            HealthStatus::Unhealthy(reason) => ProviderHealthStatus::Unhealthy(reason.clone()),
        }
    }
}

impl ProviderManagerTrait for ProviderManagerAdapter {
    fn execute_research(
        &self,
        request: &ClassifiedRequest,
    ) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send
    {
        let provider_manager = Arc::clone(&self.provider_manager);
        let request = request.clone();

        async move {
            debug!(
                "Executing research through provider manager for query: '{}'",
                request.original_query
            );

            // Execute research through the provider manager
            let result = provider_manager.execute_research(&request).await?;

            // The provider manager returns a ResearchResult, but we need just the response text
            // Extract the immediate answer for the research engine
            Ok(result.immediate_answer)
        }
    }

    fn get_performance_stats(
        &self,
    ) -> impl std::future::Future<Output = HashMap<String, ProviderPerformanceStats>> + Send {
        let provider_manager = Arc::clone(&self.provider_manager);

        async move {
            debug!("Retrieving performance statistics from provider manager");

            let provider_stats = provider_manager.get_performance_stats().await;
            let mut converted_stats = HashMap::new();

            for (provider_name, performance) in provider_stats {
                let converted_perf = ProviderPerformanceStats {
                    total_requests: performance.total_requests,
                    successful_requests: performance.successful_requests,
                    failed_requests: performance.failed_requests,
                    average_latency: performance.average_latency(),
                    average_quality: performance.average_quality(),
                    success_rate: performance.success_rate(),
                };
                converted_stats.insert(provider_name, converted_perf);
            }

            info!(
                "Retrieved performance stats for {} providers",
                converted_stats.len()
            );
            converted_stats
        }
    }

    fn health_check_all(
        &self,
    ) -> impl std::future::Future<
        Output = Result<
            HashMap<String, ProviderHealthStatus>,
            Box<dyn std::error::Error + Send + Sync>,
        >,
    > + Send {
        let provider_manager = Arc::clone(&self.provider_manager);

        async move {
            debug!("Performing health check on all providers");

            let health_results = provider_manager.health_check_all().await?;
            let mut converted_results = HashMap::new();

            for (provider_name, health_status) in health_results {
                let converted_status = match health_status {
                    HealthStatus::Healthy => ProviderHealthStatus::Healthy,
                    HealthStatus::Degraded(reason) => ProviderHealthStatus::Degraded(reason),
                    HealthStatus::Unhealthy(reason) => ProviderHealthStatus::Unhealthy(reason),
                };
                converted_results.insert(provider_name, converted_status);
            }

            info!(
                "Health check completed for {} providers",
                converted_results.len()
            );
            Ok(converted_results)
        }
    }
}

/// Factory for creating research engines with provider integration
pub struct ResearchEngineFactory;

impl ResearchEngineFactory {
    /// Create a multi-provider research engine with the given provider manager
    pub async fn create_multi_provider_engine(
        provider_manager: Arc<ProviderManager>,
        config: fortitude_core::multi_provider_research_engine::MultiProviderConfig,
    ) -> Result<
        fortitude_core::multi_provider_research_engine::MultiProviderResearchEngine<
            ProviderManagerAdapter,
        >,
        fortitude_core::multi_provider_research_engine::MultiProviderResearchError,
    > {
        let adapter = Arc::new(ProviderManagerAdapter::new(provider_manager));
        fortitude_core::multi_provider_research_engine::MultiProviderResearchEngine::new(
            adapter, config,
        )
        .await
    }

    /// Create a multi-provider research engine with vector search capabilities
    pub async fn create_multi_provider_engine_with_vector_search(
        provider_manager: Arc<ProviderManager>,
        config: fortitude_core::multi_provider_research_engine::MultiProviderConfig,
        vector_search: Arc<fortitude_core::vector::HybridSearchService>,
    ) -> Result<
        fortitude_core::multi_provider_research_engine::MultiProviderResearchEngine<
            ProviderManagerAdapter,
        >,
        fortitude_core::multi_provider_research_engine::MultiProviderResearchError,
    > {
        let adapter = Arc::new(ProviderManagerAdapter::new(provider_manager));
        fortitude_core::multi_provider_research_engine::MultiProviderResearchEngine::with_vector_search(
            adapter, config, vector_search,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{
        manager::{ProviderConfig, ProviderManager},
        mock::MockProvider,
    };
    use fortitude_types::{AudienceContext, DomainContext, ResearchType};

    #[allow(dead_code)]
    fn create_test_request() -> ClassifiedRequest {
        ClassifiedRequest::new(
            "Test adapter query".to_string(),
            ResearchType::Implementation,
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

    #[tokio::test]
    async fn test_provider_manager_adapter_creation() {
        let config = ProviderConfig::default();
        let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());
        let adapter = ProviderManagerAdapter::new(provider_manager);

        // Adapter should be created successfully
        assert!(adapter.inner().list_providers().await.is_empty());
    }

    #[tokio::test]
    async fn test_adapter_health_check() {
        let config = ProviderConfig::default();
        let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());

        // Add a mock provider
        let mock_provider = Arc::new(MockProvider::new_with_health("test-provider", true));
        provider_manager
            .add_provider("test".to_string(), mock_provider)
            .await
            .unwrap();

        let adapter = ProviderManagerAdapter::new(provider_manager);

        // Health check should succeed
        let health_results = adapter.health_check_all().await;
        assert!(health_results.is_ok());

        let results = health_results.unwrap();
        assert!(!results.is_empty());
        assert!(results.contains_key("test"));
    }

    #[tokio::test]
    async fn test_adapter_performance_stats() {
        let config = ProviderConfig::default();
        let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());

        // Add a mock provider
        let mock_provider = Arc::new(MockProvider::new("perf-test"));
        provider_manager
            .add_provider("perf".to_string(), mock_provider)
            .await
            .unwrap();

        let adapter = ProviderManagerAdapter::new(provider_manager);

        // Get performance stats
        let stats = adapter.get_performance_stats().await;
        assert!(stats.contains_key("perf"));

        let perf_stats = &stats["perf"];
        assert_eq!(perf_stats.total_requests, 0); // No requests yet
        assert_eq!(perf_stats.successful_requests, 0);
        assert_eq!(perf_stats.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_research_engine_factory() {
        let config = ProviderConfig::default();
        let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());

        // Add a mock provider
        let mock_provider = Arc::new(MockProvider::new("factory-test"));
        provider_manager
            .add_provider("factory".to_string(), mock_provider)
            .await
            .unwrap();

        // Create research engine through factory
        let engine_config =
            fortitude_core::multi_provider_research_engine::MultiProviderConfig::default();
        let result =
            ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
                .await;

        assert!(result.is_ok(), "Research engine creation should succeed");
    }

    #[tokio::test]
    async fn test_health_status_conversion() {
        let config = ProviderConfig::default();
        let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());
        let adapter = ProviderManagerAdapter::new(provider_manager);

        // Test all health status conversions
        let healthy = HealthStatus::Healthy;
        let degraded = HealthStatus::Degraded("Test degradation".to_string());
        let unhealthy = HealthStatus::Unhealthy("Test failure".to_string());

        let converted_healthy = adapter.convert_health_status(&healthy);
        let converted_degraded = adapter.convert_health_status(&degraded);
        let converted_unhealthy = adapter.convert_health_status(&unhealthy);

        assert!(matches!(converted_healthy, ProviderHealthStatus::Healthy));
        assert!(matches!(
            converted_degraded,
            ProviderHealthStatus::Degraded(_)
        ));
        assert!(matches!(
            converted_unhealthy,
            ProviderHealthStatus::Unhealthy(_)
        ));
    }
}
