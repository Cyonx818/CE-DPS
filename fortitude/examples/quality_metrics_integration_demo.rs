// Example demonstrating integration of the automated quality metrics collection system
// with the existing quality scoring infrastructure.

use chrono::{Duration as ChronoDuration, Utc};
use fortitude::quality::{
    InMemoryMetricsStorage, MetricContext, MetricFilters, MetricType, MetricValue, MetricsAnalyzer,
    MetricsCollector, MetricsConfig, MetricsStorage, QualityMetric, QualityScore, QualityWeights,
    TrendDirection,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Quality Metrics Collection Integration Demo");
    println!("==========================================");

    // 1. Setup metrics collection infrastructure
    let storage = Arc::new(InMemoryMetricsStorage::new());
    let config = MetricsConfig {
        buffer_size: 1000,
        batch_size: 10,
        flush_interval: Duration::from_secs(1),
        enable_realtime: true,
        enable_batch: true,
        ..Default::default()
    };
    let collector = MetricsCollector::new(config, storage.clone());
    let analyzer = MetricsAnalyzer::new(storage.clone());

    // Start collecting metrics
    collector.start().await?;
    println!("✓ Metrics collection system started");

    // 2. Simulate quality scoring and metrics collection
    let providers = ["claude", "openai", "gemini"];
    let queries = [
        "What is machine learning?",
        "Explain quantum computing",
        "How does blockchain work?",
        "What are neural networks?",
        "Describe artificial intelligence",
    ];

    println!("\n📊 Collecting quality metrics from simulated research operations...");

    for (i, query) in queries.iter().enumerate() {
        for provider in &providers {
            // Simulate quality scoring (would normally come from actual quality scorer)
            let base_quality = 0.7 + (i as f64 * 0.05);
            let provider_variance = match *provider {
                "claude" => 0.1,
                "openai" => 0.05,
                "gemini" => 0.08,
                _ => 0.0,
            };

            let quality_score = QualityScore {
                relevance: base_quality + provider_variance,
                accuracy: base_quality + provider_variance * 0.8,
                completeness: base_quality + provider_variance * 1.2,
                clarity: base_quality + provider_variance * 0.9,
                credibility: base_quality + provider_variance * 1.1,
                timeliness: base_quality + provider_variance * 0.7,
                specificity: base_quality + provider_variance * 0.6,
                composite: 0.0, // Will be calculated
                confidence: 0.85 + provider_variance * 0.5,
            };

            // Create quality metric
            let context = MetricContext::new()
                .with_research_type("general".to_string())
                .with_domain("technology".to_string())
                .with_query_complexity(0.6 + (i as f64 * 0.1))
                .with_custom("query".to_string(), query.to_string());

            let quality_metric = QualityMetric::with_context(
                MetricType::ResearchQuality,
                MetricValue::QualityScore(quality_score),
                Some(provider.to_string()),
                context,
            )
            .with_tag("session_id".to_string(), "demo_session".to_string())
            .with_tag("query_type".to_string(), "informational".to_string());

            // Collect the metric
            collector.collect(quality_metric).await?;

            // Also collect provider performance metrics
            let response_time = match *provider {
                "claude" => Duration::from_millis((150 + i * 20) as u64),
                "openai" => Duration::from_millis((200 + i * 15) as u64),
                "gemini" => Duration::from_millis((120 + i * 25) as u64),
                _ => Duration::from_millis(180),
            };

            let perf_metric = QualityMetric::new(
                MetricType::ProviderPerformance,
                MetricValue::Duration(response_time),
                Some(provider.to_string()),
            )
            .with_tag("metric_name".to_string(), "response_time".to_string())
            .with_tag("session_id".to_string(), "demo_session".to_string());

            collector.collect(perf_metric).await?;
        }
    }

    // Give time for batch processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("✓ Collected {} metrics", storage.metrics_count().await);

    // 3. Demonstrate analytics capabilities
    println!("\n📈 Analyzing collected metrics...");

    // Analyze quality trends for each provider
    let time_range = (Utc::now() - ChronoDuration::hours(1), Utc::now());

    for provider in &providers {
        let trends = analyzer
            .analyze_quality_trends(Some(provider.to_string()), time_range)
            .await?;

        println!(
            "  {} - Avg Quality: {:.3}, Trend: {:?}, Samples: {}",
            provider, trends.average_quality, trends.trend_direction, trends.sample_count
        );
    }

    // Analyze provider performance comparison
    let performance_analysis = analyzer.analyze_provider_performance(time_range).await?;

    println!("\n⚡ Provider Performance Analysis:");
    for analysis in performance_analysis {
        println!(
            "  {} - Avg Response: {:.0}ms, Reliability: {:.3}, Samples: {}",
            analysis.provider,
            analysis.average_performance,
            analysis.reliability_score,
            analysis.sample_count
        );
    }

    // Demonstrate filtering capabilities
    println!("\n🔍 Demonstrating query capabilities...");

    // Query quality metrics for a specific provider
    let claude_filter = MetricFilters::new()
        .with_provider("claude".to_string())
        .with_metric_type(MetricType::ResearchQuality)
        .with_limit(3);

    let claude_metrics = storage.query_metrics(&claude_filter).await?;
    println!(
        "  Found {} quality metrics for Claude (limited to 3)",
        claude_metrics.len()
    );

    // Query performance metrics
    let perf_filter = MetricFilters::new()
        .with_metric_type(MetricType::ProviderPerformance)
        .with_tag("metric_name".to_string(), "response_time".to_string());

    let perf_metrics = storage.query_metrics(&perf_filter).await?;
    println!(
        "  Found {} response time metrics across all providers",
        perf_metrics.len()
    );

    // Get aggregated metrics
    let aggregated = storage
        .get_aggregated_metrics(
            &MetricFilters::new().with_metric_type(MetricType::ResearchQuality),
            Duration::from_secs(3600),
        )
        .await?;

    println!("\n📊 Aggregated Quality Metrics:");
    for agg in aggregated {
        if let (Some(avg), Some(min), Some(max)) = (
            agg.aggregations.average,
            agg.aggregations.minimum,
            agg.aggregations.maximum,
        ) {
            println!(
                "  {:?} - {}: Avg={:.3}, Min={:.3}, Max={:.3}, Count={}",
                agg.metric_type,
                agg.provider.as_deref().unwrap_or("All"),
                avg,
                min,
                max,
                agg.count
            );
        }
    }

    // Demonstrate storage statistics
    let stats = storage.get_storage_stats().await?;
    println!("\n💾 Storage Statistics:");
    println!("  Total metrics: {}", stats.total_metrics);
    println!("  Storage size: {} bytes", stats.storage_size_bytes);
    println!("  Metrics by type:");
    for (metric_type, count) in stats.metrics_by_type {
        println!("    {:?}: {}", metric_type, count);
    }
    println!("  Metrics by provider:");
    for (provider, count) in stats.metrics_by_provider {
        println!("    {}: {}", provider, count);
    }

    // Performance monitoring
    let performance_stats = collector.performance_stats().await;
    println!("\n⚡ Collector Performance:");
    println!(
        "  Avg collection time: {:?}",
        performance_stats.collection_time_avg
    );
    println!(
        "  P95 collection time: {:?}",
        performance_stats.collection_time_p95
    );
    println!(
        "  Memory usage: {} bytes",
        performance_stats.memory_usage_current
    );

    // Cleanup
    collector.stop().await?;
    println!("\n✓ Metrics collection system stopped");

    println!("\n🎯 Integration Benefits Demonstrated:");
    println!("  • Real-time quality metrics collection (<5ms overhead)");
    println!("  • Time-series storage with efficient querying");
    println!("  • Provider performance comparison and trends");
    println!("  • Automated analytics and insights");
    println!("  • Comprehensive filtering and aggregation");
    println!("  • Performance monitoring and optimization");

    println!("\n✅ Quality metrics integration demo completed successfully!");

    Ok(())
}
