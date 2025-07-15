// ABOUTME: Demonstration of the comprehensive performance optimization system
//! This example shows how the performance optimizations work together to achieve
//! the <200ms target response time for vector operations.

use fortitude_core::vector::{
    cache::CacheManager,
    optimized_config::{EnvironmentType, OptimizedVectorConfig},
    optimized_embeddings::OptimizedEmbeddingService,
    performance::PerformanceManager,
    regression_detection::{RegressionConfig, RegressionDetector},
    EmbeddingGenerator,
};
use std::time::Instant;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Performance Optimization Demo");

    // 1. Create optimized configuration for production environment
    let mut config = OptimizedVectorConfig::for_environment(EnvironmentType::Production);
    config.auto_optimize()?;
    config.validate()?;

    info!("Configuration optimized for production environment");
    info!(
        "Target response time: {}ms",
        config.performance.target_response_time_ms
    );
    info!(
        "Max connections: {}",
        config.performance.connection_pool.max_connections
    );
    info!(
        "Cache memory: {}MB",
        config.caching.l1_config.max_memory_bytes / (1024 * 1024)
    );

    // 2. Initialize performance management system
    let performance_manager = PerformanceManager::new(config.performance.clone());
    performance_manager.initialize().await?;

    info!("Performance management system initialized");

    // 3. Initialize advanced caching system
    let cache_manager = CacheManager::new(config.caching.clone());

    info!("Advanced caching system initialized");

    // 4. Initialize optimized embedding service
    let embedding_service =
        OptimizedEmbeddingService::new(config.embeddings.clone(), cache_manager);
    embedding_service.initialize().await?;

    info!("Optimized embedding service initialized");

    // 5. Initialize regression detection system
    let regression_detector = RegressionDetector::new(
        RegressionConfig::default(),
        performance_manager.monitor.clone(),
    );
    regression_detector.start_monitoring().await?;

    info!("Regression detection system started");

    // 6. Demonstrate performance optimizations
    info!("Starting performance demonstration...");

    // Test 1: Single embedding generation with caching
    let test_texts = vec![
        "performance optimization",
        "vector database operations",
        "semantic search capabilities",
        "cache efficiency improvements",
        "performance optimization", // Repeat to test cache hit
    ];

    let mut total_time = std::time::Duration::ZERO;
    let mut cache_hits = 0;

    for (i, text) in test_texts.iter().enumerate() {
        let start = Instant::now();
        let _embedding = embedding_service.generate_embedding_optimized(text).await?;
        let duration = start.elapsed();
        total_time += duration;

        info!("Text {}: '{}' - {}ms", i + 1, text, duration.as_millis());

        if i == 4 {
            // The repeated text should be a cache hit
            if duration.as_millis() < 50 {
                // Should be much faster due to cache
                cache_hits += 1;
                info!("  ✓ Cache hit detected (fast response)");
            }
        }
    }

    let avg_time = total_time.as_millis() / test_texts.len() as u128;
    info!("Average response time: {}ms", avg_time);

    if avg_time < 200 {
        info!("✓ SUCCESS: Average response time under 200ms target!");
    } else {
        info!("⚠ WARNING: Average response time exceeds 200ms target");
    }

    // Test 2: Batch processing optimization
    info!("Testing batch processing optimization...");

    let batch_texts: Vec<String> = (0..20)
        .map(|i| format!("batch test text number {}", i))
        .collect();

    let batch_start = Instant::now();
    let _batch_embeddings = embedding_service.generate_embeddings(&batch_texts).await?;
    let batch_duration = batch_start.elapsed();
    let batch_avg = batch_duration.as_millis() / batch_texts.len() as u128;

    info!(
        "Batch processing: {} texts in {}ms",
        batch_texts.len(),
        batch_duration.as_millis()
    );
    info!("Batch average per text: {}ms", batch_avg);

    if batch_avg < 100 {
        info!("✓ SUCCESS: Batch processing highly optimized!");
    }

    // 7. Get performance statistics
    let embedding_stats = embedding_service.get_optimized_stats().await;
    let performance_report = performance_manager.get_performance_report().await;

    info!("=== Performance Statistics ===");
    info!(
        "Embeddings generated: {}",
        embedding_stats.base_stats.total_generated
    );
    info!(
        "Cache hit rate: {:.1}%",
        embedding_stats.base_stats.cache_hit_rate * 100.0
    );
    info!(
        "Average response time: {:.1}ms",
        embedding_stats.performance.avg_response_time_ms
    );

    // 8. Demonstrate auto-tuning (simulation)
    info!("=== Auto-Tuning Demonstration ===");

    // Simulate metrics for auto-tuning
    let simulated_metrics = fortitude_core::vector::optimized_config::PerformanceMetrics {
        avg_latency_ms: 150.0,
        throughput_ops_sec: 120.0,
        cache_hit_rate: 0.85,
        memory_utilization: 0.8, // 80% memory utilization
        cpu_utilization: 0.6,    // 60% CPU utilization
        error_rate: 0.01,        // 1% error rate
    };

    // Apply auto-tuning based on metrics
    let mut tuning_config = config.clone();
    tuning_config.apply_auto_tuning(&simulated_metrics)?;

    info!("Auto-tuning applied based on simulated metrics");
    info!(
        "Cache size adjusted: {} -> {} entries",
        config.caching.l1_config.max_entries, tuning_config.caching.l1_config.max_entries
    );

    // 9. Performance recommendations
    let usage_stats = fortitude_core::vector::optimized_config::UsageStatistics {
        avg_response_time_ms: avg_time as f64,
        avg_cache_hit_rate: embedding_stats.base_stats.cache_hit_rate,
        throughput_ops_sec: 100.0,
        memory_utilization: 0.7,
        cpu_utilization: 0.6,
        error_rate: 0.01,
        peak_concurrent_requests: 25,
        avg_batch_size: 15.0,
    };

    let recommendations = config.get_recommendations(&usage_stats);

    info!("=== Performance Recommendations ===");
    for rec in &recommendations.recommendations {
        info!("• {:?}: {}", rec.priority, rec.description);
    }

    info!("=== Demo Summary ===");
    info!("✓ Multi-level caching system operational");
    info!("✓ Advanced embedding optimization active");
    info!("✓ Performance monitoring enabled");
    info!("✓ Regression detection running");
    info!("✓ Auto-tuning capabilities demonstrated");

    if avg_time < 200 {
        info!("🎯 TARGET ACHIEVED: <200ms response time goal met!");
    }

    if cache_hits > 0 {
        info!("🚀 OPTIMIZATION WORKING: Cache hits detected!");
    }

    info!("Performance optimization demo completed successfully");

    Ok(())
}
