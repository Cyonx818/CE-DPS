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

// ABOUTME: Mock provider implementation for testing
//! Mock provider implementation that can be used for testing and development.
//! Provides configurable responses, delays, and failure modes.

use super::{HealthStatus, Provider, ProviderError, ProviderMetadata, ProviderResult};
use async_trait::async_trait;
use std::time::Duration;

/// Mock provider for testing trait implementations
#[derive(Debug, Clone)]
pub struct MockProvider {
    name: String,
    healthy: bool,
    should_fail: bool,
    response_delay: Duration,
    custom_response: Option<String>,
}

impl MockProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            healthy: true,
            should_fail: false,
            response_delay: Duration::from_millis(10),
            custom_response: None,
        }
    }

    /// Create a new MockProvider with specified name and health status (used in tests)
    pub fn new_with_health(name: &str, healthy: bool) -> Self {
        Self {
            name: name.to_string(),
            healthy,
            should_fail: false,
            response_delay: Duration::from_millis(10),
            custom_response: None,
        }
    }

    pub fn with_health(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }

    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    pub fn with_response(mut self, response: &str) -> Self {
        self.custom_response = Some(response.to_string());
        self
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        // Simulate response delay
        tokio::time::sleep(self.response_delay).await;

        if self.should_fail {
            return Err(ProviderError::QueryFailed {
                message: "Mock provider configured to fail".to_string(),
                provider: self.name.clone(),
                error_code: Some("MOCK_FAILURE".to_string()),
            });
        }

        if !self.healthy {
            return Err(ProviderError::Unhealthy {
                provider: self.name.clone(),
                message: "Provider is not healthy".to_string(),
            });
        }

        Ok(self
            .custom_response
            .clone()
            .unwrap_or_else(|| format!("Mock response for query: {query}")))
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(self.name.clone(), "1.0.0".to_string())
            .with_capabilities(vec![
                "research".to_string(),
                "async".to_string(),
                "mock".to_string(),
            ])
            .with_models(vec!["mock-model-v1".to_string()])
    }

    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        if self.healthy {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy(
                "Mock provider unhealthy".to_string(),
            ))
        }
    }
}
