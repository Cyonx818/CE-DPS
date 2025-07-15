use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use tracing::{info, warn, error, debug, span, Level};
use prometheus::{Counter, Histogram, Gauge, Registry, TextEncoder, Encoder};

// Core observability framework for research systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMetrics {
    pub request_latency: f64,
    pub tokens_used: u64,
    pub cost_usd: f64,
    pub quality_score: f64,
    pub error_category: Option<ErrorCategory>,
    pub timestamp: u64,
    pub operation_type: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    RateLimited,
    ModelOverload,
    NetworkTimeout,
    AuthenticationFailure,
    InvalidInput,
    InternalError,
    CostThresholdExceeded,
}

#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_io: f64,
    pub disk_io: f64,
    pub bottleneck: Option<BottleneckType>,
}

#[derive(Debug, Clone)]
pub enum BottleneckType {
    CPU,
    Memory,
    Network,
    Disk,
    ExternalAPI,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub threshold: f64,
    pub comparison: AlertComparison,
    pub metric_name: String,
    pub severity: AlertSeverity,
    pub cooldown: Duration,
    pub last_triggered: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum AlertComparison {
    GreaterThan,
    LessThan,
    Equals,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug)]
pub struct Alert {
    pub rule_name: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: Instant,
    pub value: f64,
}

// Main observability system
pub struct ObservabilitySystem {
    metrics: Arc<Mutex<Vec<ResearchMetrics>>>,
    performance_profiles: Arc<Mutex<Vec<PerformanceProfile>>>,
    alert_rules: Arc<Mutex<Vec<AlertRule>>>,
    alert_sender: broadcast::Sender<Alert>,
    prometheus_registry: Registry,
    
    // Prometheus metrics
    request_duration: Histogram,
    token_usage: Counter,
    cost_tracker: Gauge,
    error_counter: Counter,
    quality_gauge: Gauge,
}

impl ObservabilitySystem {
    pub fn new() -> Self {
        let registry = Registry::new();
        let (alert_sender, _) = broadcast::channel(1000);
        
        // Initialize Prometheus metrics
        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "research_request_duration_seconds",
                "Duration of research requests in seconds"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
        ).unwrap();
        
        let token_usage = Counter::with_opts(
            prometheus::CounterOpts::new(
                "research_tokens_total",
                "Total tokens used in research operations"
            )
        ).unwrap();
        
        let cost_tracker = Gauge::with_opts(
            prometheus::GaugeOpts::new(
                "research_cost_usd",
                "Current research operation cost in USD"
            )
        ).unwrap();
        
        let error_counter = Counter::with_opts(
            prometheus::CounterOpts::new(
                "research_errors_total",
                "Total research operation errors"
            )
        ).unwrap();
        
        let quality_gauge = Gauge::with_opts(
            prometheus::GaugeOpts::new(
                "research_quality_score",
                "Quality score of research results"
            )
        ).unwrap();
        
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(token_usage.clone())).unwrap();
        registry.register(Box::new(cost_tracker.clone())).unwrap();
        registry.register(Box::new(error_counter.clone())).unwrap();
        registry.register(Box::new(quality_gauge.clone())).unwrap();
        
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            performance_profiles: Arc::new(Mutex::new(Vec::new())),
            alert_rules: Arc::new(Mutex::new(Vec::new())),
            alert_sender,
            prometheus_registry: registry,
            request_duration,
            token_usage,
            cost_tracker,
            error_counter,
            quality_gauge,
        }
    }
    
    // Record research operation metrics
    pub fn record_metrics(&self, metrics: ResearchMetrics) -> Result<(), Box<dyn std::error::Error>> {
        let _span = span!(Level::DEBUG, "record_metrics").entered();
        
        // Update Prometheus metrics
        self.request_duration.observe(metrics.request_latency);
        self.token_usage.inc_by(metrics.tokens_used as f64);
        self.cost_tracker.set(metrics.cost_usd);
        self.quality_gauge.set(metrics.quality_score);
        
        if metrics.error_category.is_some() {
            self.error_counter.inc();
        }
        
        // Store metrics for analysis
        let mut metrics_store = self.metrics.lock().unwrap();
        metrics_store.push(metrics.clone());
        
        // Keep only last 10,000 metrics to prevent memory bloat
        if metrics_store.len() > 10_000 {
            metrics_store.drain(0..1000);
        }
        
        // Check alert rules
        self.check_alerts(&metrics)?;
        
        info!(
            "Recorded metrics: latency={:.2}s, tokens={}, cost=${:.4}, quality={:.2}",
            metrics.request_latency, metrics.tokens_used, metrics.cost_usd, metrics.quality_score
        );
        
        Ok(())
    }
    
    // Performance profiling
    pub fn record_performance_profile(&self, profile: PerformanceProfile) {
        let _span = span!(Level::DEBUG, "record_performance").entered();
        
        let mut profiles = self.performance_profiles.lock().unwrap();
        profiles.push(profile.clone());
        
        // Keep only last 1,000 profiles
        if profiles.len() > 1000 {
            profiles.drain(0..100);
        }
        
        debug!(
            "Performance profile: CPU={:.1}%, Memory={:.1}%, Network={:.1}MB/s, Disk={:.1}MB/s",
            profile.cpu_usage, profile.memory_usage, profile.network_io, profile.disk_io
        );
        
        if let Some(bottleneck) = &profile.bottleneck {
            warn!("Performance bottleneck detected: {:?}", bottleneck);
        }
    }
    
    // Intelligent alerting system
    pub fn add_alert_rule(&self, rule: AlertRule) {
        let mut rules = self.alert_rules.lock().unwrap();
        rules.push(rule);
    }
    
    fn check_alerts(&self, metrics: &ResearchMetrics) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.alert_rules.lock().unwrap();
        
        for rule in rules.iter_mut() {
            // Check cooldown
            if let Some(last_triggered) = rule.last_triggered {
                if last_triggered.elapsed() < rule.cooldown {
                    continue;
                }
            }
            
            let metric_value = match rule.metric_name.as_str() {
                "latency" => metrics.request_latency,
                "cost" => metrics.cost_usd,
                "quality" => metrics.quality_score,
                "tokens" => metrics.tokens_used as f64,
                _ => continue,
            };
            
            let should_alert = match rule.comparison {
                AlertComparison::GreaterThan => metric_value > rule.threshold,
                AlertComparison::LessThan => metric_value < rule.threshold,
                AlertComparison::Equals => (metric_value - rule.threshold).abs() < 0.001,
            };
            
            if should_alert {
                let alert = Alert {
                    rule_name: rule.name.clone(),
                    message: format!(
                        "Alert: {} {} {:.2} (threshold: {:.2})",
                        rule.metric_name, 
                        match rule.comparison {
                            AlertComparison::GreaterThan => ">",
                            AlertComparison::LessThan => "<",
                            AlertComparison::Equals => "=",
                        },
                        metric_value, 
                        rule.threshold
                    ),
                    severity: rule.severity.clone(),
                    timestamp: Instant::now(),
                    value: metric_value,
                };
                
                match alert.severity {
                    AlertSeverity::Info => info!("{}", alert.message),
                    AlertSeverity::Warning => warn!("{}", alert.message),
                    AlertSeverity::Critical => error!("{}", alert.message),
                }
                
                let _ = self.alert_sender.send(alert);
                rule.last_triggered = Some(Instant::now());
            }
        }
        
        Ok(())
    }
    
    // Cost optimization based on metrics
    pub fn optimize_costs(&self) -> CostOptimization {
        let metrics = self.metrics.lock().unwrap();
        let recent_metrics: Vec<&ResearchMetrics> = metrics
            .iter()
            .filter(|m| {
                let age = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - m.timestamp;
                age < 3600 // Last hour
            })
            .collect();
        
        if recent_metrics.is_empty() {
            return CostOptimization::default();
        }
        
        let avg_cost = recent_metrics.iter().map(|m| m.cost_usd).sum::<f64>() / recent_metrics.len() as f64;
        let avg_quality = recent_metrics.iter().map(|m| m.quality_score).sum::<f64>() / recent_metrics.len() as f64;
        let total_tokens = recent_metrics.iter().map(|m| m.tokens_used).sum::<u64>();
        
        let mut recommendations = Vec::new();
        
        // Cost per quality point analysis
        let cost_efficiency = avg_cost / avg_quality.max(0.1);
        
        if cost_efficiency > 0.1 {
            recommendations.push("Consider using a more cost-effective model for lower-priority queries".to_string());
        }
        
        if avg_quality > 0.9 && avg_cost > 0.05 {
            recommendations.push("Quality is very high - consider using a less expensive model".to_string());
        }
        
        if total_tokens > 100_000 {
            recommendations.push("High token usage detected - implement request batching or caching".to_string());
        }
        
        CostOptimization {
            current_hourly_cost: avg_cost * recent_metrics.len() as f64,
            cost_efficiency,
            potential_savings: if cost_efficiency > 0.1 { avg_cost * 0.3 } else { 0.0 },
            recommendations,
        }
    }
    
    // Performance bottleneck identification
    pub fn identify_bottlenecks(&self) -> Vec<BottleneckAnalysis> {
        let profiles = self.performance_profiles.lock().unwrap();
        let recent_profiles: Vec<&PerformanceProfile> = profiles
            .iter()
            .rev()
            .take(100)
            .collect();
        
        if recent_profiles.is_empty() {
            return Vec::new();
        }
        
        let mut bottlenecks = Vec::new();
        
        // CPU bottleneck analysis
        let avg_cpu = recent_profiles.iter().map(|p| p.cpu_usage).sum::<f64>() / recent_profiles.len() as f64;
        if avg_cpu > 80.0 {
            bottlenecks.push(BottleneckAnalysis {
                bottleneck_type: BottleneckType::CPU,
                severity: if avg_cpu > 95.0 { "Critical" } else { "Warning" }.to_string(),
                impact: "High CPU usage may slow down request processing".to_string(),
                recommendation: "Consider scaling horizontally or optimizing CPU-intensive operations".to_string(),
            });
        }
        
        // Memory bottleneck analysis
        let avg_memory = recent_profiles.iter().map(|p| p.memory_usage).sum::<f64>() / recent_profiles.len() as f64;
        if avg_memory > 85.0 {
            bottlenecks.push(BottleneckAnalysis {
                bottleneck_type: BottleneckType::Memory,
                severity: if avg_memory > 95.0 { "Critical" } else { "Warning" }.to_string(),
                impact: "High memory usage may cause out-of-memory errors".to_string(),
                recommendation: "Implement memory caching strategies or increase available memory".to_string(),
            });
        }
        
        bottlenecks
    }
    
    // Export metrics for Prometheus
    pub fn export_prometheus_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.prometheus_registry.gather();
        let mut output = Vec::new();
        encoder.encode(&metric_families, &mut output)?;
        Ok(String::from_utf8(output)?)
    }
    
    // Generate dashboard data
    pub fn generate_dashboard_data(&self) -> DashboardData {
        let metrics = self.metrics.lock().unwrap();
        let recent_metrics: Vec<&ResearchMetrics> = metrics
            .iter()
            .rev()
            .take(1000)
            .collect();
        
        if recent_metrics.is_empty() {
            return DashboardData::default();
        }
        
        DashboardData {
            total_requests: recent_metrics.len() as u64,
            avg_latency: recent_metrics.iter().map(|m| m.request_latency).sum::<f64>() / recent_metrics.len() as f64,
            total_cost: recent_metrics.iter().map(|m| m.cost_usd).sum::<f64>(),
            avg_quality: recent_metrics.iter().map(|m| m.quality_score).sum::<f64>() / recent_metrics.len() as f64,
            error_rate: recent_metrics.iter().filter(|m| m.error_category.is_some()).count() as f64 / recent_metrics.len() as f64,
            top_errors: self.get_top_errors(&recent_metrics),
        }
    }
    
    fn get_top_errors(&self, metrics: &[&ResearchMetrics]) -> Vec<(String, u64)> {
        let mut error_counts: HashMap<String, u64> = HashMap::new();
        
        for metric in metrics {
            if let Some(error) = &metric.error_category {
                let error_name = format!("{:?}", error);
                *error_counts.entry(error_name).or_insert(0) += 1;
            }
        }
        
        let mut errors: Vec<(String, u64)> = error_counts.into_iter().collect();
        errors.sort_by(|a, b| b.1.cmp(&a.1));
        errors.into_iter().take(5).collect()
    }
    
    // Subscribe to alerts
    pub fn subscribe_to_alerts(&self) -> broadcast::Receiver<Alert> {
        self.alert_sender.subscribe()
    }
}

#[derive(Debug, Default)]
pub struct CostOptimization {
    pub current_hourly_cost: f64,
    pub cost_efficiency: f64,
    pub potential_savings: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug)]
pub struct BottleneckAnalysis {
    pub bottleneck_type: BottleneckType,
    pub severity: String,
    pub impact: String,
    pub recommendation: String,
}

#[derive(Debug, Default)]
pub struct DashboardData {
    pub total_requests: u64,
    pub avg_latency: f64,
    pub total_cost: f64,
    pub avg_quality: f64,
    pub error_rate: f64,
    pub top_errors: Vec<(String, u64)>,
}

// Testing utilities and examples
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_metrics_recording() {
        let observability = ObservabilitySystem::new();
        
        let metrics = ResearchMetrics {
            request_latency: 1.5,
            tokens_used: 1000,
            cost_usd: 0.02,
            quality_score: 0.85,
            error_category: None,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            operation_type: "search".to_string(),
            model_name: "gpt-4".to_string(),
        };
        
        assert!(observability.record_metrics(metrics).is_ok());
    }
    
    #[tokio::test]
    async fn test_alerting_system() {
        let observability = ObservabilitySystem::new();
        
        // Add alert rule for high latency
        let alert_rule = AlertRule {
            name: "High Latency".to_string(),
            threshold: 2.0,
            comparison: AlertComparison::GreaterThan,
            metric_name: "latency".to_string(),
            severity: AlertSeverity::Warning,
            cooldown: Duration::from_secs(60),
            last_triggered: None,
        };
        
        observability.add_alert_rule(alert_rule);
        
        let mut alert_receiver = observability.subscribe_to_alerts();
        
        // Record high latency metric
        let metrics = ResearchMetrics {
            request_latency: 3.0, // Above threshold
            tokens_used: 500,
            cost_usd: 0.01,
            quality_score: 0.7,
            error_category: None,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            operation_type: "search".to_string(),
            model_name: "gpt-4".to_string(),
        };
        
        observability.record_metrics(metrics).unwrap();
        
        // Should receive an alert
        let alert = alert_receiver.recv().await.unwrap();
        assert_eq!(alert.rule_name, "High Latency");
    }
    
    #[test]
    fn test_cost_optimization() {
        let observability = ObservabilitySystem::new();
        
        // Add some sample metrics
        for i in 0..10 {
            let metrics = ResearchMetrics {
                request_latency: 1.0 + i as f64 * 0.1,
                tokens_used: 1000 + i * 100,
                cost_usd: 0.02 + i as f64 * 0.01,
                quality_score: 0.8 + i as f64 * 0.01,
                error_category: None,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                operation_type: "search".to_string(),
                model_name: "gpt-4".to_string(),
            };
            observability.record_metrics(metrics).unwrap();
        }
        
        let optimization = observability.optimize_costs();
        assert!(optimization.current_hourly_cost > 0.0);
    }
}

// Usage example and configuration
impl ObservabilitySystem {
    pub fn example_setup() -> Self {
        let observability = ObservabilitySystem::new();
        
        // Add common alert rules
        observability.add_alert_rule(AlertRule {
            name: "High Latency Alert".to_string(),
            threshold: 5.0,
            comparison: AlertComparison::GreaterThan,
            metric_name: "latency".to_string(),
            severity: AlertSeverity::Warning,
            cooldown: Duration::from_secs(300),
            last_triggered: None,
        });
        
        observability.add_alert_rule(AlertRule {
            name: "High Cost Alert".to_string(),
            threshold: 0.50,
            comparison: AlertComparison::GreaterThan,
            metric_name: "cost".to_string(),
            severity: AlertSeverity::Critical,
            cooldown: Duration::from_secs(600),
            last_triggered: None,
        });
        
        observability.add_alert_rule(AlertRule {
            name: "Low Quality Alert".to_string(),
            threshold: 0.6,
            comparison: AlertComparison::LessThan,
            metric_name: "quality".to_string(),
            severity: AlertSeverity::Warning,
            cooldown: Duration::from_secs(300),
            last_triggered: None,
        });
        
        observability
    }
}