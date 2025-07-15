//! Test utilities for multi-LLM provider testing

use fortitude::providers::{HealthStatus, Provider, ProviderError, ProviderResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Test result collector for analyzing test outcomes
#[derive(Debug, Clone, Default)]
pub struct TestResultCollector {
    pub successes: Vec<TestResult>,
    pub failures: Vec<TestResult>,
    pub timeouts: Vec<TestResult>,
    pub errors: Vec<TestResult>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub provider_name: String,
    pub query: String,
    pub duration: Duration,
    pub error: Option<String>,
    pub timestamp: Instant,
}

impl TestResultCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_success(&mut self, provider_name: String, query: String, duration: Duration) {
        self.successes.push(TestResult {
            provider_name,
            query,
            duration,
            error: None,
            timestamp: Instant::now(),
        });
    }

    pub fn add_failure(
        &mut self,
        provider_name: String,
        query: String,
        duration: Duration,
        error: String,
    ) {
        self.failures.push(TestResult {
            provider_name,
            query,
            duration,
            error: Some(error),
            timestamp: Instant::now(),
        });
    }

    pub fn add_timeout(&mut self, provider_name: String, query: String, duration: Duration) {
        self.timeouts.push(TestResult {
            provider_name,
            query,
            duration,
            error: Some("Timeout".to_string()),
            timestamp: Instant::now(),
        });
    }

    pub fn add_error(
        &mut self,
        provider_name: String,
        query: String,
        duration: Duration,
        error: String,
    ) {
        self.errors.push(TestResult {
            provider_name,
            query,
            duration,
            error: Some(error),
            timestamp: Instant::now(),
        });
    }

    pub fn total_tests(&self) -> usize {
        self.successes.len() + self.failures.len() + self.timeouts.len() + self.errors.len()
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_tests();
        if total == 0 {
            return 0.0;
        }
        self.successes.len() as f64 / total as f64
    }

    pub fn average_success_duration(&self) -> Duration {
        if self.successes.is_empty() {
            return Duration::ZERO;
        }
        let total_millis: u64 = self
            .successes
            .iter()
            .map(|r| r.duration.as_millis() as u64)
            .sum();
        Duration::from_millis(total_millis / self.successes.len() as u64)
    }

    pub fn max_duration(&self) -> Duration {
        let all_results = self
            .successes
            .iter()
            .chain(self.failures.iter())
            .chain(self.timeouts.iter())
            .chain(self.errors.iter());

        all_results
            .map(|r| r.duration)
            .max()
            .unwrap_or(Duration::ZERO)
    }

    pub fn min_duration(&self) -> Duration {
        let all_results = self
            .successes
            .iter()
            .chain(self.failures.iter())
            .chain(self.timeouts.iter())
            .chain(self.errors.iter());

        all_results
            .map(|r| r.duration)
            .min()
            .unwrap_or(Duration::ZERO)
    }

    pub fn throughput(&self, test_duration: Duration) -> f64 {
        if test_duration.is_zero() {
            return 0.0;
        }
        self.total_tests() as f64 / test_duration.as_secs_f64()
    }

    pub fn error_types(&self) -> HashMap<String, usize> {
        let mut error_counts = HashMap::new();

        for failure in &self.failures {
            if let Some(error) = &failure.error {
                *error_counts.entry(error.clone()).or_insert(0) += 1;
            }
        }

        for error in &self.errors {
            if let Some(error_msg) = &error.error {
                *error_counts.entry(error_msg.clone()).or_insert(0) += 1;
            }
        }

        if !self.timeouts.is_empty() {
            *error_counts.entry("Timeout".to_string()).or_insert(0) += self.timeouts.len();
        }

        error_counts
    }

    pub fn print_summary(&self) {
        println!("=== Test Result Summary ===");
        println!("Total tests: {}", self.total_tests());
        println!(
            "Successes: {} ({:.1}%)",
            self.successes.len(),
            self.success_rate() * 100.0
        );
        println!("Failures: {}", self.failures.len());
        println!("Timeouts: {}", self.timeouts.len());
        println!("Errors: {}", self.errors.len());
        println!(
            "Average success duration: {:?}",
            self.average_success_duration()
        );
        println!("Max duration: {:?}", self.max_duration());
        println!("Min duration: {:?}", self.min_duration());

        let error_types = self.error_types();
        if !error_types.is_empty() {
            println!("Error breakdown:");
            for (error_type, count) in error_types {
                println!("  {}: {}", error_type, count);
            }
        }
    }
}

/// Utility for running concurrent provider tests
pub struct ConcurrentTester {
    results: Arc<Mutex<TestResultCollector>>,
}

impl ConcurrentTester {
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(TestResultCollector::new())),
        }
    }

    /// Run a single provider test with timeout
    pub async fn test_provider_query<P: Provider>(
        &self,
        provider: &P,
        query: String,
        test_timeout: Duration,
    ) -> ProviderResult<String> {
        let provider_name = provider.metadata().name().to_string();
        let start_time = Instant::now();

        let result = timeout(test_timeout, provider.research_query(query.clone())).await;
        let duration = start_time.elapsed();

        match result {
            Ok(Ok(response)) => {
                let mut results = self.results.lock().unwrap();
                results.add_success(provider_name, query, duration);
                Ok(response)
            }
            Ok(Err(provider_error)) => {
                let mut results = self.results.lock().unwrap();
                results.add_failure(provider_name, query, duration, provider_error.to_string());
                Err(provider_error)
            }
            Err(_timeout_error) => {
                let mut results = self.results.lock().unwrap();
                results.add_timeout(provider_name.clone(), query, duration);
                Err(ProviderError::Timeout {
                    provider: provider_name,
                    duration,
                })
            }
        }
    }

    /// Run multiple concurrent tests against a provider
    pub async fn test_provider_concurrent<P: Provider + Send + Sync + 'static>(
        &self,
        provider: Arc<P>,
        queries: Vec<String>,
        concurrent_limit: usize,
        test_timeout: Duration,
    ) -> Vec<ProviderResult<String>> {
        use futures::stream::{self, StreamExt};

        let results = stream::iter(queries.into_iter().enumerate())
            .map(|(i, query)| {
                let provider_clone = Arc::clone(&provider);
                let tester = self.clone();
                async move {
                    tokio::time::sleep(Duration::from_millis(i as u64 * 10)).await; // Stagger requests slightly
                    tester
                        .test_provider_query(provider_clone.as_ref(), query, test_timeout)
                        .await
                }
            })
            .buffer_unordered(concurrent_limit)
            .collect::<Vec<_>>()
            .await;

        results
    }

    /// Get test results
    pub fn get_results(&self) -> TestResultCollector {
        self.results.lock().unwrap().clone()
    }

    /// Clear test results
    pub fn clear_results(&self) {
        let mut results = self.results.lock().unwrap();
        *results = TestResultCollector::new();
    }
}

impl Clone for ConcurrentTester {
    fn clone(&self) -> Self {
        Self {
            results: Arc::clone(&self.results),
        }
    }
}

/// Utility for health check monitoring
pub struct HealthMonitor {
    checks: Arc<Mutex<Vec<HealthCheckResult>>>,
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub provider_name: String,
    pub status: HealthStatus,
    pub duration: Duration,
    pub timestamp: Instant,
    pub error: Option<String>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Perform health check on a provider
    pub async fn check_provider_health<P: Provider>(
        &self,
        provider: &P,
    ) -> ProviderResult<HealthStatus> {
        let provider_name = provider.metadata().name().to_string();
        let start_time = Instant::now();

        let result = provider.health_check().await;
        let duration = start_time.elapsed();

        let health_result = match &result {
            Ok(status) => HealthCheckResult {
                provider_name: provider_name.clone(),
                status: status.clone(),
                duration,
                timestamp: Instant::now(),
                error: None,
            },
            Err(error) => HealthCheckResult {
                provider_name: provider_name.clone(),
                status: HealthStatus::Unhealthy("Health check failed".to_string()),
                duration,
                timestamp: Instant::now(),
                error: Some(error.to_string()),
            },
        };

        let mut checks = self.checks.lock().unwrap();
        checks.push(health_result);

        result
    }

    /// Monitor provider health over time
    pub async fn monitor_provider_health<P: Provider + Send + Sync>(
        &self,
        provider: Arc<P>,
        interval: Duration,
        duration: Duration,
    ) {
        let end_time = Instant::now() + duration;

        while Instant::now() < end_time {
            let _ = self.check_provider_health(provider.as_ref()).await;
            tokio::time::sleep(interval).await;
        }
    }

    /// Get health check history
    pub fn get_health_history(&self) -> Vec<HealthCheckResult> {
        self.checks.lock().unwrap().clone()
    }

    /// Get latest health status for a provider
    pub fn get_latest_health(&self, provider_name: &str) -> Option<HealthCheckResult> {
        let checks = self.checks.lock().unwrap();
        checks
            .iter()
            .filter(|check| check.provider_name == provider_name)
            .max_by_key(|check| check.timestamp)
            .cloned()
    }

    /// Calculate health uptime percentage
    pub fn calculate_uptime(&self, provider_name: &str) -> f64 {
        let checks = self.checks.lock().unwrap();
        let provider_checks: Vec<_> = checks
            .iter()
            .filter(|check| check.provider_name == provider_name)
            .collect();

        if provider_checks.is_empty() {
            return 0.0;
        }

        let healthy_count = provider_checks
            .iter()
            .filter(|check| matches!(check.status, HealthStatus::Healthy))
            .count();

        healthy_count as f64 / provider_checks.len() as f64
    }

    /// Clear health check history
    pub fn clear_history(&self) {
        let mut checks = self.checks.lock().unwrap();
        checks.clear();
    }
}

impl Clone for HealthMonitor {
    fn clone(&self) -> Self {
        Self {
            checks: Arc::clone(&self.checks),
        }
    }
}

/// Utility for performance testing
pub struct PerformanceTester {
    pub max_latency_threshold: Duration,
    pub min_throughput_threshold: f64,
    pub success_rate_threshold: f64,
}

impl PerformanceTester {
    pub fn new() -> Self {
        Self {
            max_latency_threshold: Duration::from_secs(5),
            min_throughput_threshold: 1.0, // requests per second
            success_rate_threshold: 0.95,  // 95%
        }
    }

    pub fn with_thresholds(max_latency: Duration, min_throughput: f64, success_rate: f64) -> Self {
        Self {
            max_latency_threshold: max_latency,
            min_throughput_threshold: min_throughput,
            success_rate_threshold: success_rate,
        }
    }

    /// Run performance test against a provider
    pub async fn run_performance_test<P: Provider + Send + Sync + 'static>(
        &self,
        provider: Arc<P>,
        queries: Vec<String>,
        concurrent_requests: usize,
        test_duration: Duration,
    ) -> PerformanceTestResult {
        let tester = ConcurrentTester::new();
        let start_time = Instant::now();

        // Run all queries with concurrency limit
        let results = tester
            .test_provider_concurrent(
                provider,
                queries,
                concurrent_requests,
                Duration::from_secs(30), // Individual request timeout
            )
            .await;

        let actual_duration = start_time.elapsed();
        let test_results = tester.get_results();

        PerformanceTestResult {
            total_requests: test_results.total_tests(),
            successful_requests: test_results.successes.len(),
            failed_requests: test_results.failures.len() + test_results.errors.len(),
            timeout_requests: test_results.timeouts.len(),
            success_rate: test_results.success_rate(),
            average_latency: test_results.average_success_duration(),
            max_latency: test_results.max_duration(),
            min_latency: test_results.min_duration(),
            throughput: test_results.throughput(actual_duration),
            test_duration: actual_duration,
            meets_latency_threshold: test_results.max_duration() <= self.max_latency_threshold,
            meets_throughput_threshold: test_results.throughput(actual_duration)
                >= self.min_throughput_threshold,
            meets_success_rate_threshold: test_results.success_rate()
                >= self.success_rate_threshold,
            error_breakdown: test_results.error_types(),
        }
    }

    /// Validate performance test results
    pub fn validate_performance(&self, result: &PerformanceTestResult) -> bool {
        result.meets_latency_threshold
            && result.meets_throughput_threshold
            && result.meets_success_rate_threshold
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub timeout_requests: usize,
    pub success_rate: f64,
    pub average_latency: Duration,
    pub max_latency: Duration,
    pub min_latency: Duration,
    pub throughput: f64, // requests per second
    pub test_duration: Duration,
    pub meets_latency_threshold: bool,
    pub meets_throughput_threshold: bool,
    pub meets_success_rate_threshold: bool,
    pub error_breakdown: HashMap<String, usize>,
}

impl PerformanceTestResult {
    pub fn print_summary(&self) {
        println!("=== Performance Test Results ===");
        println!("Total requests: {}", self.total_requests);
        println!(
            "Successful: {} ({:.1}%)",
            self.successful_requests,
            self.success_rate * 100.0
        );
        println!("Failed: {}", self.failed_requests);
        println!("Timeouts: {}", self.timeout_requests);
        println!("Average latency: {:?}", self.average_latency);
        println!("Max latency: {:?}", self.max_latency);
        println!("Min latency: {:?}", self.min_latency);
        println!("Throughput: {:.2} req/sec", self.throughput);
        println!("Test duration: {:?}", self.test_duration);
        println!("Meets latency threshold: {}", self.meets_latency_threshold);
        println!(
            "Meets throughput threshold: {}",
            self.meets_throughput_threshold
        );
        println!(
            "Meets success rate threshold: {}",
            self.meets_success_rate_threshold
        );

        if !self.error_breakdown.is_empty() {
            println!("Error breakdown:");
            for (error_type, count) in &self.error_breakdown {
                println!("  {}: {}", error_type, count);
            }
        }
    }
}

/// Utility for environment variable testing
pub fn setup_test_environment() {
    // Set up test environment variables if not already set
    let test_vars = [
        ("OPENAI_API_KEY", "sk-test1234567890abcdef1234567890abcdef"),
        (
            "ANTHROPIC_API_KEY",
            "sk-ant-test1234567890abcdef1234567890abcdef",
        ),
        (
            "GOOGLE_API_KEY",
            "AIzaSyTest1234567890abcdef1234567890abcdef",
        ),
        ("TEST_MODE", "true"),
    ];

    for (key, value) in test_vars.iter() {
        if std::env::var(key).is_err() {
            std::env::set_var(key, value);
        }
    }
}

/// Utility for cleaning up test environment
pub fn cleanup_test_environment() {
    let test_vars = [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "GOOGLE_API_KEY",
        "TEST_MODE",
    ];

    for key in test_vars.iter() {
        // Only remove if it's a test value
        if let Ok(value) = std::env::var(key) {
            if value.contains("test") || value.contains("Test") {
                std::env::remove_var(key);
            }
        }
    }
}

/// Test guard for automatic cleanup
pub struct TestEnvironmentGuard {
    original_vars: HashMap<String, Option<String>>,
}

impl TestEnvironmentGuard {
    pub fn new() -> Self {
        let test_vars = [
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "GOOGLE_API_KEY",
            "TEST_MODE",
        ];

        let mut original_vars = HashMap::new();
        for var in test_vars.iter() {
            original_vars.insert(var.to_string(), std::env::var(var).ok());
        }

        setup_test_environment();

        Self { original_vars }
    }
}

impl Drop for TestEnvironmentGuard {
    fn drop(&mut self) {
        // Restore original environment variables
        for (key, original_value) in &self.original_vars {
            match original_value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::mock_providers::MockProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_result_collector() {
        let mut collector = TestResultCollector::new();

        collector.add_success(
            "test-provider".to_string(),
            "query1".to_string(),
            Duration::from_millis(100),
        );
        collector.add_failure(
            "test-provider".to_string(),
            "query2".to_string(),
            Duration::from_millis(200),
            "Error".to_string(),
        );

        assert_eq!(collector.total_tests(), 2);
        assert_eq!(collector.success_rate(), 0.5);
        assert_eq!(
            collector.average_success_duration(),
            Duration::from_millis(100)
        );
    }

    #[tokio::test]
    async fn test_concurrent_tester() {
        let provider = Arc::new(MockProvider::new("test-provider"));
        let tester = ConcurrentTester::new();

        let queries = vec!["query1".to_string(), "query2".to_string()];
        let results = tester
            .test_provider_concurrent(provider, queries, 2, Duration::from_secs(5))
            .await;

        assert_eq!(results.len(), 2);
        let test_results = tester.get_results();
        assert_eq!(test_results.total_tests(), 2);
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let provider = MockProvider::new("health-test");
        let monitor = HealthMonitor::new();

        let status = monitor.check_provider_health(&provider).await;
        assert!(status.is_ok());

        let history = monitor.get_health_history();
        assert_eq!(history.len(), 1);

        let uptime = monitor.calculate_uptime("health-test");
        assert_eq!(uptime, 1.0);
    }

    #[tokio::test]
    async fn test_performance_tester() {
        let provider = Arc::new(MockProvider::new("perf-test"));
        let tester = PerformanceTester::new();

        let queries = vec!["query1".to_string(), "query2".to_string()];
        let result = tester
            .run_performance_test(provider, queries, 2, Duration::from_secs(1))
            .await;

        assert_eq!(result.total_requests, 2);
        assert!(result.success_rate > 0.0);
    }

    #[test]
    fn test_environment_guard() {
        {
            let _guard = TestEnvironmentGuard::new();
            assert!(std::env::var("TEST_MODE").is_ok());
        }
        // After guard is dropped, if TEST_MODE wasn't originally set, it should be removed
    }
}
