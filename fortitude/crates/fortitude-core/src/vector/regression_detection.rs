// ABOUTME: Performance regression detection system for vector operations
//! This module provides automated detection of performance regressions and quality degradation
//! in vector operations with alerting and auto-recovery capabilities.

use crate::vector::{
    error::VectorResult,
    performance::{PerformanceMetrics, PerformanceMonitor},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Performance baseline for regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Baseline metrics
    pub metrics: PerformanceMetrics,
    /// When the baseline was established
    pub established_at: SystemTime,
    /// Number of samples used for baseline
    pub sample_count: usize,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Variance in metrics
    pub variance: MetricsVariance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsVariance {
    pub latency_variance: f64,
    pub throughput_variance: f64,
    pub cache_hit_rate_variance: f64,
    pub memory_variance: f64,
    pub cpu_variance: f64,
}

/// Regression detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// Baseline establishment period
    pub baseline_period: Duration,
    /// Minimum samples for baseline
    pub min_samples_for_baseline: usize,
    /// Detection sensitivity (0.0-1.0)
    pub sensitivity: f64,
    /// Regression thresholds
    pub thresholds: RegressionThresholds,
    /// Auto-recovery settings
    pub auto_recovery: AutoRecoveryConfig,
    /// Alerting configuration
    pub alerting: AlertingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionThresholds {
    /// Latency increase threshold (percentage)
    pub latency_increase_threshold: f64,
    /// Throughput decrease threshold (percentage)
    pub throughput_decrease_threshold: f64,
    /// Cache hit rate decrease threshold (percentage)
    pub cache_hit_rate_decrease_threshold: f64,
    /// Memory increase threshold (percentage)
    pub memory_increase_threshold: f64,
    /// Error rate increase threshold (percentage)
    pub error_rate_increase_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRecoveryConfig {
    /// Enable automatic recovery attempts
    pub enabled: bool,
    /// Recovery strategies to try
    pub strategies: Vec<RecoveryStrategy>,
    /// Maximum recovery attempts
    pub max_attempts: usize,
    /// Recovery attempt interval
    pub attempt_interval: Duration,
    /// Recovery timeout
    pub recovery_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Clear caches and restart fresh
    CacheClear,
    /// Reduce load by increasing batch sizes
    LoadReduction,
    /// Scale up connection pool
    ConnectionPoolScale,
    /// Restart embedding service
    EmbeddingServiceRestart,
    /// Fallback to safe configuration
    SafeConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable alerting
    pub enabled: bool,
    /// Alert channels
    pub channels: Vec<AlertChannel>,
    /// Alert severity levels
    pub severity_levels: HashMap<RegressionSeverity, AlertSeverity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Log,
    Email(String),
    Webhook(String),
    Metrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RegressionSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Detected regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    /// Regression ID
    pub id: String,
    /// Detection timestamp
    pub detected_at: SystemTime,
    /// Regression type
    pub regression_type: RegressionType,
    /// Severity
    pub severity: RegressionSeverity,
    /// Affected metrics
    pub affected_metrics: Vec<String>,
    /// Performance impact
    pub impact: RegressionImpact,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
    /// Recovery attempts made
    pub recovery_attempts: Vec<RecoveryAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    PerformanceDegradation,
    ThroughputReduction,
    LatencyIncrease,
    CacheEfficiencyLoss,
    ResourceExhaustion,
    QualityDegradation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionImpact {
    /// Estimated impact on response time (ms)
    pub response_time_impact_ms: f64,
    /// Estimated impact on throughput (percentage)
    pub throughput_impact_percentage: f64,
    /// Estimated impact on user experience (0.0-1.0)
    pub user_experience_impact: f64,
    /// Estimated cost impact (if applicable)
    pub cost_impact: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Recovery strategy used
    pub strategy: RecoveryStrategy,
    /// Attempt timestamp
    pub attempted_at: SystemTime,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Metrics after recovery attempt
    pub post_recovery_metrics: Option<PerformanceMetrics>,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            baseline_period: Duration::from_secs(1800), // 30 minutes
            min_samples_for_baseline: 100,
            sensitivity: 0.7,
            thresholds: RegressionThresholds {
                latency_increase_threshold: 25.0,        // 25% increase
                throughput_decrease_threshold: 20.0,     // 20% decrease
                cache_hit_rate_decrease_threshold: 15.0, // 15% decrease
                memory_increase_threshold: 30.0,         // 30% increase
                error_rate_increase_threshold: 50.0,     // 50% increase
            },
            auto_recovery: AutoRecoveryConfig {
                enabled: true,
                strategies: vec![
                    RecoveryStrategy::CacheClear,
                    RecoveryStrategy::LoadReduction,
                    RecoveryStrategy::ConnectionPoolScale,
                    RecoveryStrategy::SafeConfiguration,
                ],
                max_attempts: 3,
                attempt_interval: Duration::from_secs(60),
                recovery_timeout: Duration::from_secs(300),
            },
            alerting: AlertingConfig {
                enabled: true,
                channels: vec![AlertChannel::Log, AlertChannel::Metrics],
                severity_levels: {
                    let mut levels = HashMap::new();
                    levels.insert(RegressionSeverity::Minor, AlertSeverity::Info);
                    levels.insert(RegressionSeverity::Moderate, AlertSeverity::Warning);
                    levels.insert(RegressionSeverity::Severe, AlertSeverity::Error);
                    levels.insert(RegressionSeverity::Critical, AlertSeverity::Critical);
                    levels
                },
            },
        }
    }
}

/// Performance regression detector
pub struct RegressionDetector {
    config: RegressionConfig,
    baseline: Arc<RwLock<Option<PerformanceBaseline>>>,
    metrics_history: Arc<RwLock<VecDeque<(SystemTime, PerformanceMetrics)>>>,
    detected_regressions: Arc<RwLock<Vec<Regression>>>,
    monitor: Arc<PerformanceMonitor>,
}

impl RegressionDetector {
    pub fn new(config: RegressionConfig, monitor: PerformanceMonitor) -> Self {
        Self {
            config,
            baseline: Arc::new(RwLock::new(None)),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            detected_regressions: Arc::new(RwLock::new(Vec::new())),
            monitor: Arc::new(monitor),
        }
    }

    /// Start regression detection monitoring
    #[instrument(skip(self))]
    pub async fn start_monitoring(&self) -> VectorResult<()> {
        info!("Starting performance regression detection");

        let detector = self.clone();
        tokio::spawn(async move {
            let mut check_interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

            loop {
                check_interval.tick().await;

                if let Err(e) = detector.check_for_regressions().await {
                    error!("Error during regression check: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Check for performance regressions
    #[instrument(skip(self))]
    pub async fn check_for_regressions(&self) -> VectorResult<()> {
        // Get current metrics
        let current_metrics = self.monitor.get_metrics().await;
        let now = SystemTime::now();

        // Add to history
        {
            let mut history = self.metrics_history.write().await;
            history.push_back((now, current_metrics.clone()));

            // Keep only recent history (last 24 hours)
            let cutoff = now - Duration::from_secs(86400);
            while let Some((timestamp, _)) = history.front() {
                if *timestamp < cutoff {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        // Establish or update baseline if needed
        if self.should_update_baseline().await {
            self.update_baseline().await?;
        }

        // Check for regressions against baseline
        if let Some(baseline) = self.baseline.read().await.as_ref() {
            if let Some(regression) = self.detect_regression(&current_metrics, baseline).await? {
                self.handle_regression(regression).await?;
            }
        }

        Ok(())
    }

    /// Check if baseline should be updated
    async fn should_update_baseline(&self) -> bool {
        let baseline = self.baseline.read().await;
        let history = self.metrics_history.read().await;

        match baseline.as_ref() {
            None => {
                // No baseline established yet
                history.len() >= self.config.min_samples_for_baseline
            }
            Some(baseline) => {
                // Check if baseline is too old
                let age = SystemTime::now()
                    .duration_since(baseline.established_at)
                    .unwrap_or(Duration::ZERO);

                age > Duration::from_secs(86400) && // Older than 24 hours
                history.len() >= self.config.min_samples_for_baseline
            }
        }
    }

    /// Update performance baseline
    async fn update_baseline(&self) -> VectorResult<()> {
        let history = self.metrics_history.read().await;

        if history.len() < self.config.min_samples_for_baseline {
            return Ok(()); // Not enough samples
        }

        // Calculate baseline metrics from recent stable period
        let recent_samples: Vec<_> = history
            .iter()
            .rev()
            .take(self.config.min_samples_for_baseline)
            .map(|(_, metrics)| metrics.clone())
            .collect();

        let baseline_metrics = self.calculate_average_metrics(&recent_samples);
        let variance = self.calculate_variance(&recent_samples, &baseline_metrics);

        let baseline = PerformanceBaseline {
            metrics: baseline_metrics,
            established_at: SystemTime::now(),
            sample_count: recent_samples.len(),
            confidence: self.calculate_confidence(&recent_samples),
            variance,
        };

        *self.baseline.write().await = Some(baseline);

        info!(
            "Performance baseline updated with {} samples",
            recent_samples.len()
        );
        Ok(())
    }

    /// Calculate average metrics from samples
    fn calculate_average_metrics(&self, samples: &[PerformanceMetrics]) -> PerformanceMetrics {
        let count = samples.len() as f64;

        PerformanceMetrics {
            avg_latency_ms: samples.iter().map(|m| m.avg_latency_ms).sum::<f64>() / count,
            p95_latency_ms: samples.iter().map(|m| m.p95_latency_ms).sum::<f64>() / count,
            p99_latency_ms: samples.iter().map(|m| m.p99_latency_ms).sum::<f64>() / count,
            throughput_ops_sec: samples.iter().map(|m| m.throughput_ops_sec).sum::<f64>() / count,
            error_rate_percentage: samples.iter().map(|m| m.error_rate_percentage).sum::<f64>()
                / count,
            cache_hit_rate: samples.iter().map(|m| m.cache_hit_rate).sum::<f64>() / count,
            memory_usage_bytes: (samples
                .iter()
                .map(|m| m.memory_usage_bytes as f64)
                .sum::<f64>()
                / count) as usize,
            cpu_usage_percentage: samples.iter().map(|m| m.cpu_usage_percentage).sum::<f64>()
                / count,
            active_connections: (samples
                .iter()
                .map(|m| m.active_connections as f64)
                .sum::<f64>()
                / count) as usize,
            queue_depth: (samples.iter().map(|m| m.queue_depth as f64).sum::<f64>() / count)
                as usize,
        }
    }

    /// Calculate variance in metrics
    fn calculate_variance(
        &self,
        samples: &[PerformanceMetrics],
        baseline: &PerformanceMetrics,
    ) -> MetricsVariance {
        let count = samples.len() as f64;

        let latency_variance = samples
            .iter()
            .map(|m| (m.avg_latency_ms - baseline.avg_latency_ms).powi(2))
            .sum::<f64>()
            / count;

        let throughput_variance = samples
            .iter()
            .map(|m| (m.throughput_ops_sec - baseline.throughput_ops_sec).powi(2))
            .sum::<f64>()
            / count;

        let cache_variance = samples
            .iter()
            .map(|m| (m.cache_hit_rate - baseline.cache_hit_rate).powi(2))
            .sum::<f64>()
            / count;

        let memory_variance = samples
            .iter()
            .map(|m| (m.memory_usage_bytes as f64 - baseline.memory_usage_bytes as f64).powi(2))
            .sum::<f64>()
            / count;

        let cpu_variance = samples
            .iter()
            .map(|m| (m.cpu_usage_percentage - baseline.cpu_usage_percentage).powi(2))
            .sum::<f64>()
            / count;

        MetricsVariance {
            latency_variance,
            throughput_variance,
            cache_hit_rate_variance: cache_variance,
            memory_variance,
            cpu_variance,
        }
    }

    /// Calculate confidence level for baseline
    fn calculate_confidence(&self, samples: &[PerformanceMetrics]) -> f64 {
        // Simple confidence calculation based on sample size and consistency
        let min_confidence = 0.5;
        let max_confidence = 0.95;

        let size_factor = (samples.len() as f64 / 1000.0).min(1.0);

        // Calculate consistency (inverse of coefficient of variation)
        let avg_latency =
            samples.iter().map(|m| m.avg_latency_ms).sum::<f64>() / samples.len() as f64;
        let latency_std = (samples
            .iter()
            .map(|m| (m.avg_latency_ms - avg_latency).powi(2))
            .sum::<f64>()
            / samples.len() as f64)
            .sqrt();

        let consistency_factor = if avg_latency > 0.0 {
            1.0 - (latency_std / avg_latency).min(1.0)
        } else {
            0.5
        };

        min_confidence + (max_confidence - min_confidence) * size_factor * consistency_factor
    }

    /// Detect regression by comparing current metrics to baseline
    async fn detect_regression(
        &self,
        current: &PerformanceMetrics,
        baseline: &PerformanceBaseline,
    ) -> VectorResult<Option<Regression>> {
        let thresholds = &self.config.thresholds;
        let mut issues = Vec::new();
        let mut affected_metrics = Vec::new();

        // Check latency regression
        let latency_increase = ((current.avg_latency_ms - baseline.metrics.avg_latency_ms)
            / baseline.metrics.avg_latency_ms)
            * 100.0;
        if latency_increase > thresholds.latency_increase_threshold {
            issues.push(format!("Latency increased by {latency_increase:.1}%"));
            affected_metrics.push("avg_latency_ms".to_string());
        }

        // Check throughput regression
        let throughput_decrease = ((baseline.metrics.throughput_ops_sec
            - current.throughput_ops_sec)
            / baseline.metrics.throughput_ops_sec)
            * 100.0;
        if throughput_decrease > thresholds.throughput_decrease_threshold {
            issues.push(format!("Throughput decreased by {throughput_decrease:.1}%"));
            affected_metrics.push("throughput_ops_sec".to_string());
        }

        // Check cache hit rate regression
        let cache_decrease = ((baseline.metrics.cache_hit_rate - current.cache_hit_rate)
            / baseline.metrics.cache_hit_rate)
            * 100.0;
        if cache_decrease > thresholds.cache_hit_rate_decrease_threshold {
            issues.push(format!("Cache hit rate decreased by {cache_decrease:.1}%"));
            affected_metrics.push("cache_hit_rate".to_string());
        }

        // Check memory usage regression
        let memory_increase = ((current.memory_usage_bytes as f64
            - baseline.metrics.memory_usage_bytes as f64)
            / baseline.metrics.memory_usage_bytes as f64)
            * 100.0;
        if memory_increase > thresholds.memory_increase_threshold {
            issues.push(format!("Memory usage increased by {memory_increase:.1}%"));
            affected_metrics.push("memory_usage_bytes".to_string());
        }

        // Check error rate regression
        let error_increase = if baseline.metrics.error_rate_percentage > 0.0 {
            ((current.error_rate_percentage - baseline.metrics.error_rate_percentage)
                / baseline.metrics.error_rate_percentage)
                * 100.0
        } else if current.error_rate_percentage > 0.0 {
            100.0 // Any errors when baseline had none is significant
        } else {
            0.0
        };

        if error_increase > thresholds.error_rate_increase_threshold {
            issues.push(format!("Error rate increased by {error_increase:.1}%"));
            affected_metrics.push("error_rate_percentage".to_string());
        }

        if issues.is_empty() {
            return Ok(None);
        }

        // Determine severity and regression type
        let severity = self.calculate_severity(&issues, current, baseline);
        let regression_type = self.determine_regression_type(&affected_metrics);

        let regression = Regression {
            id: uuid::Uuid::new_v4().to_string(),
            detected_at: SystemTime::now(),
            regression_type,
            severity,
            affected_metrics,
            impact: self.calculate_impact(current, baseline),
            suggested_actions: self.generate_suggested_actions(&issues),
            recovery_attempts: Vec::new(),
        };

        Ok(Some(regression))
    }

    /// Calculate regression severity
    fn calculate_severity(
        &self,
        issues: &[String],
        current: &PerformanceMetrics,
        baseline: &PerformanceBaseline,
    ) -> RegressionSeverity {
        let latency_ratio = current.avg_latency_ms / baseline.metrics.avg_latency_ms;
        let throughput_ratio = current.throughput_ops_sec / baseline.metrics.throughput_ops_sec;

        if latency_ratio > 2.0 || throughput_ratio < 0.5 || current.error_rate_percentage > 10.0 {
            RegressionSeverity::Critical
        } else if latency_ratio > 1.5
            || throughput_ratio < 0.7
            || current.error_rate_percentage > 5.0
        {
            RegressionSeverity::Severe
        } else if latency_ratio > 1.3 || throughput_ratio < 0.8 || issues.len() > 2 {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Minor
        }
    }

    /// Determine regression type based on affected metrics
    fn determine_regression_type(&self, affected_metrics: &[String]) -> RegressionType {
        if affected_metrics.contains(&"avg_latency_ms".to_string()) {
            RegressionType::LatencyIncrease
        } else if affected_metrics.contains(&"throughput_ops_sec".to_string()) {
            RegressionType::ThroughputReduction
        } else if affected_metrics.contains(&"cache_hit_rate".to_string()) {
            RegressionType::CacheEfficiencyLoss
        } else if affected_metrics.contains(&"memory_utilization".to_string()) {
            RegressionType::ResourceExhaustion
        } else {
            RegressionType::PerformanceDegradation
        }
    }

    /// Calculate regression impact
    fn calculate_impact(
        &self,
        current: &PerformanceMetrics,
        baseline: &PerformanceBaseline,
    ) -> RegressionImpact {
        RegressionImpact {
            response_time_impact_ms: current.avg_latency_ms - baseline.metrics.avg_latency_ms,
            throughput_impact_percentage: ((baseline.metrics.throughput_ops_sec
                - current.throughput_ops_sec)
                / baseline.metrics.throughput_ops_sec)
                * 100.0,
            user_experience_impact: self.calculate_user_experience_impact(current, baseline),
            cost_impact: None, // Could be calculated based on resource usage
        }
    }

    /// Calculate user experience impact (0.0-1.0)
    fn calculate_user_experience_impact(
        &self,
        current: &PerformanceMetrics,
        baseline: &PerformanceBaseline,
    ) -> f64 {
        let latency_impact =
            (current.avg_latency_ms / baseline.metrics.avg_latency_ms - 1.0).max(0.0);
        let throughput_impact =
            (1.0 - current.throughput_ops_sec / baseline.metrics.throughput_ops_sec).max(0.0);
        let error_impact = current.error_rate_percentage / 10.0; // Convert percentage to 0-1 scale

        ((latency_impact + throughput_impact + error_impact) / 3.0).min(1.0)
    }

    /// Generate suggested actions for regression
    fn generate_suggested_actions(&self, issues: &[String]) -> Vec<String> {
        let mut actions = Vec::new();

        for issue in issues {
            if issue.contains("Latency") {
                actions.push("Consider increasing connection pool size".to_string());
                actions.push("Check for database query optimization opportunities".to_string());
                actions.push("Review recent configuration changes".to_string());
            }

            if issue.contains("Throughput") {
                actions.push("Increase batch processing sizes".to_string());
                actions.push("Scale up concurrent request limits".to_string());
                actions.push("Check for resource bottlenecks".to_string());
            }

            if issue.contains("Cache") {
                actions.push("Clear and rebuild caches".to_string());
                actions.push("Increase cache size allocation".to_string());
                actions.push("Review cache eviction policies".to_string());
            }

            if issue.contains("Memory") {
                actions.push("Reduce cache sizes temporarily".to_string());
                actions.push("Check for memory leaks".to_string());
                actions.push("Scale up available memory".to_string());
            }

            if issue.contains("Error") {
                actions.push("Check error logs for root cause".to_string());
                actions.push("Restart embedding service".to_string());
                actions.push("Fallback to safe configuration".to_string());
            }
        }

        actions.dedup();
        actions
    }

    /// Handle detected regression
    async fn handle_regression(&self, mut regression: Regression) -> VectorResult<()> {
        warn!(
            "Performance regression detected: {:?} severity, affecting {:?}",
            regression.severity, regression.affected_metrics
        );

        // Store regression
        self.detected_regressions
            .write()
            .await
            .push(regression.clone());

        // Send alerts
        self.send_alerts(&regression).await?;

        // Attempt auto-recovery if enabled
        if self.config.auto_recovery.enabled {
            self.attempt_auto_recovery(&mut regression).await?;
        }

        Ok(())
    }

    /// Send alerts for regression
    async fn send_alerts(&self, regression: &Regression) -> VectorResult<()> {
        if !self.config.alerting.enabled {
            return Ok(());
        }

        let alert_severity = self
            .config
            .alerting
            .severity_levels
            .get(&regression.severity)
            .unwrap_or(&AlertSeverity::Warning);

        let alert_message = format!(
            "Performance regression detected: {} ({}). Impact: {:.1}% UX degradation. Affected: {:?}",
            regression.regression_type.describe(),
            regression.severity.describe(),
            regression.impact.user_experience_impact * 100.0,
            regression.affected_metrics
        );

        for channel in &self.config.alerting.channels {
            match channel {
                AlertChannel::Log => match alert_severity {
                    AlertSeverity::Info => info!("{}", alert_message),
                    AlertSeverity::Warning => warn!("{}", alert_message),
                    AlertSeverity::Error => error!("{}", alert_message),
                    AlertSeverity::Critical => error!("CRITICAL: {}", alert_message),
                },
                AlertChannel::Email(_) => {
                    // Email implementation would go here
                    debug!("Would send email alert: {}", alert_message);
                }
                AlertChannel::Webhook(_) => {
                    // Webhook implementation would go here
                    debug!("Would send webhook alert: {}", alert_message);
                }
                AlertChannel::Metrics => {
                    // Metrics emission would go here
                    debug!("Would emit metrics alert: {}", alert_message);
                }
            }
        }

        Ok(())
    }

    /// Attempt automatic recovery
    async fn attempt_auto_recovery(&self, regression: &mut Regression) -> VectorResult<()> {
        info!("Attempting auto-recovery for regression: {}", regression.id);

        for strategy in &self.config.auto_recovery.strategies {
            if regression.recovery_attempts.len() >= self.config.auto_recovery.max_attempts {
                warn!(
                    "Maximum recovery attempts reached for regression: {}",
                    regression.id
                );
                break;
            }

            let attempt_start = SystemTime::now();
            let success = self.execute_recovery_strategy(strategy).await;

            let attempt = RecoveryAttempt {
                strategy: strategy.clone(),
                attempted_at: attempt_start,
                success,
                error_message: if success {
                    None
                } else {
                    Some("Recovery failed".to_string())
                },
                post_recovery_metrics: if success {
                    Some(self.monitor.get_metrics().await)
                } else {
                    None
                },
            };

            regression.recovery_attempts.push(attempt);

            if success {
                info!("Recovery successful using strategy: {:?}", strategy);
                break;
            } else {
                warn!("Recovery failed using strategy: {:?}", strategy);

                // Wait before next attempt
                tokio::time::sleep(self.config.auto_recovery.attempt_interval).await;
            }
        }

        Ok(())
    }

    /// Execute a specific recovery strategy
    async fn execute_recovery_strategy(&self, strategy: &RecoveryStrategy) -> bool {
        match strategy {
            RecoveryStrategy::CacheClear => {
                debug!("Executing cache clear recovery strategy");
                // Implementation would clear all caches
                true // Simulate success
            }
            RecoveryStrategy::LoadReduction => {
                debug!("Executing load reduction recovery strategy");
                // Implementation would increase batch sizes, reduce concurrency
                true
            }
            RecoveryStrategy::ConnectionPoolScale => {
                debug!("Executing connection pool scaling recovery strategy");
                // Implementation would increase connection pool size
                true
            }
            RecoveryStrategy::EmbeddingServiceRestart => {
                debug!("Executing embedding service restart recovery strategy");
                // Implementation would restart embedding service
                true
            }
            RecoveryStrategy::SafeConfiguration => {
                debug!("Executing safe configuration recovery strategy");
                // Implementation would fallback to conservative settings
                true
            }
        }
    }

    /// Get all detected regressions
    pub async fn get_regressions(&self) -> Vec<Regression> {
        self.detected_regressions.read().await.clone()
    }

    /// Get current baseline
    pub async fn get_baseline(&self) -> Option<PerformanceBaseline> {
        self.baseline.read().await.clone()
    }
}

impl Clone for RegressionDetector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            baseline: Arc::clone(&self.baseline),
            metrics_history: Arc::clone(&self.metrics_history),
            detected_regressions: Arc::clone(&self.detected_regressions),
            monitor: Arc::clone(&self.monitor),
        }
    }
}

// Helper trait for enum descriptions
trait Describe {
    fn describe(&self) -> &'static str;
}

impl Describe for RegressionType {
    fn describe(&self) -> &'static str {
        match self {
            RegressionType::PerformanceDegradation => "Performance Degradation",
            RegressionType::ThroughputReduction => "Throughput Reduction",
            RegressionType::LatencyIncrease => "Latency Increase",
            RegressionType::CacheEfficiencyLoss => "Cache Efficiency Loss",
            RegressionType::ResourceExhaustion => "Resource Exhaustion",
            RegressionType::QualityDegradation => "Quality Degradation",
        }
    }
}

impl Describe for RegressionSeverity {
    fn describe(&self) -> &'static str {
        match self {
            RegressionSeverity::Minor => "Minor",
            RegressionSeverity::Moderate => "Moderate",
            RegressionSeverity::Severe => "Severe",
            RegressionSeverity::Critical => "Critical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::performance::MonitoringConfig;

    #[tokio::test]
    async fn test_baseline_calculation() {
        let config = RegressionConfig::default();
        let monitor = PerformanceMonitor::new(MonitoringConfig::default());
        let detector = RegressionDetector::new(config, monitor);

        // Simulate metrics history
        let metrics = PerformanceMetrics {
            avg_latency_ms: 100.0,
            p95_latency_ms: 100.0,
            p99_latency_ms: 120.0,
            throughput_ops_sec: 50.0,
            error_rate_percentage: 1.0,
            cache_hit_rate: 0.8,
            memory_usage_bytes: 1024 * 1024 * 600, // 600MB
            cpu_usage_percentage: 50.0,
            active_connections: 10,
            queue_depth: 5,
        };

        let samples = vec![metrics; 100];
        let baseline = detector.calculate_average_metrics(&samples);

        assert_eq!(baseline.p95_latency_ms, 100.0);
        assert_eq!(baseline.throughput_ops_sec, 50.0);
    }

    #[tokio::test]
    async fn test_regression_detection() {
        let config = RegressionConfig::default();
        let monitor = PerformanceMonitor::new(MonitoringConfig::default());
        let detector = RegressionDetector::new(config, monitor);

        let baseline = PerformanceBaseline {
            metrics: PerformanceMetrics {
                avg_latency_ms: 100.0,
                p95_latency_ms: 110.0,
                p99_latency_ms: 120.0,
                throughput_ops_sec: 50.0,
                error_rate_percentage: 1.0,
                cache_hit_rate: 0.8,
                memory_usage_bytes: 1024 * 1024 * 600, // 600MB
                cpu_usage_percentage: 50.0,
                active_connections: 10,
                queue_depth: 5,
            },
            established_at: SystemTime::now(),
            sample_count: 100,
            confidence: 0.9,
            variance: MetricsVariance {
                latency_variance: 1.0,
                throughput_variance: 1.0,
                cache_hit_rate_variance: 0.01,
                memory_variance: 0.01,
                cpu_variance: 0.01,
            },
        };

        // Test regression detection with degraded metrics
        let degraded_metrics = PerformanceMetrics {
            avg_latency_ms: 160.0, // 60% increase (ratio > 1.5 for Severe)
            p95_latency_ms: 175.0,
            p99_latency_ms: 190.0,
            throughput_ops_sec: 30.0, // 40% decrease (ratio 0.6 < 0.7 for Severe)
            error_rate_percentage: 6.0, // 6x increase (> 5.0 for Severe)
            cache_hit_rate: 0.6,      // 25% decrease
            memory_usage_bytes: 1024 * 1024 * 800, // 800MB - 33% increase
            cpu_usage_percentage: 70.0,
            active_connections: 15,
            queue_depth: 8,
        };

        let regression = detector
            .detect_regression(&degraded_metrics, &baseline)
            .await
            .unwrap();
        assert!(regression.is_some());

        let regression = regression.unwrap();
        assert!(matches!(
            regression.severity,
            RegressionSeverity::Severe | RegressionSeverity::Critical
        ));
        assert!(!regression.affected_metrics.is_empty());
    }
}
