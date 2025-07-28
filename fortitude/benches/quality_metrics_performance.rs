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

// Performance benchmarks for the automated quality metrics collection system
// Validates that performance requirements are met:
// - Collection overhead: <5ms per research operation
// - Storage throughput: Support 1000+ metrics/second ingestion
// - Query performance: <100ms for standard analytics queries

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude::quality::{
    InMemoryMetricsStorage, MetricFilters, MetricType, MetricValue, MetricsCollector,
    MetricsConfig, MetricsStorage, QualityMetric, QualityScore,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_test_quality_score(value: f64) -> QualityScore {
    QualityScore {
        relevance: value,
        accuracy: value,
        completeness: value,
        clarity: value,
        credibility: value,
        timeliness: value,
        specificity: value,
        composite: value,
        confidence: value,
    }
}

fn create_test_metric(provider: &str, quality_value: f64) -> QualityMetric {
    QualityMetric::new(
        MetricType::ResearchQuality,
        MetricValue::QualityScore(create_test_quality_score(quality_value)),
        Some(provider.to_string()),
    )
    .with_tag("benchmark".to_string(), "true".to_string())
}

// Benchmark single metric collection performance
fn bench_single_metric_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("single_metric_collection");
    group.throughput(Throughput::Elements(1));

    // Test different collection modes
    for realtime_enabled in [true, false] {
        let config = MetricsConfig {
            enable_realtime: realtime_enabled,
            enable_batch: !realtime_enabled,
            buffer_size: 1000,
            batch_size: 100,
            ..Default::default()
        };

        let storage = Arc::new(InMemoryMetricsStorage::new());
        let collector = MetricsCollector::new(config, storage);

        let mode = if realtime_enabled {
            "realtime"
        } else {
            "batch"
        };

        group.bench_with_input(
            BenchmarkId::new("collect_metric", mode),
            &collector,
            |b, collector| {
                b.to_async(&rt).iter(|| async {
                    let metric = create_test_metric("claude", 0.85);
                    collector.collect(metric).await.unwrap();
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

// Benchmark batch collection performance
fn bench_batch_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_collection");

    let config = MetricsConfig {
        enable_realtime: false,
        enable_batch: true,
        buffer_size: 10000,
        batch_size: 1000,
        ..Default::default()
    };

    let storage = Arc::new(InMemoryMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage);

    for batch_size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*batch_size));

        group.bench_with_input(
            BenchmarkId::new("collect_batch", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let metrics: Vec<QualityMetric> = (0..size)
                        .map(|i| create_test_metric("provider", 0.7 + (i as f64 * 0.01)))
                        .collect();

                    collector.collect_batch(metrics).await.unwrap();
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

// Benchmark storage throughput
fn bench_storage_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("storage_throughput");

    let storage = InMemoryMetricsStorage::new();

    for metrics_count in [100, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*metrics_count));

        group.bench_with_input(
            BenchmarkId::new("store_metrics", metrics_count),
            metrics_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let metrics: Vec<QualityMetric> = (0..count)
                        .map(|i| {
                            create_test_metric(
                                &format!("provider_{}", i % 3),
                                0.5 + (i as f64 * 0.0001),
                            )
                        })
                        .collect();

                    storage.store_metrics(&metrics).await.unwrap();
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

// Benchmark query performance
fn bench_query_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("query_performance");

    // Pre-populate storage with test data
    let storage = InMemoryMetricsStorage::new();

    rt.block_on(async {
        let metrics: Vec<QualityMetric> = (0..10000)
            .map(|i| create_test_metric(&format!("provider_{}", i % 5), 0.5 + (i as f64 * 0.00005)))
            .collect();
        storage.store_metrics(&metrics).await.unwrap();
    });

    // Benchmark different query patterns
    group.bench_function("query_by_provider", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = MetricFilters::new().with_provider("provider_0".to_string());
            black_box(storage.query_metrics(&filters).await.unwrap());
        });
    });

    group.bench_function("query_by_type", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);
            black_box(storage.query_metrics(&filters).await.unwrap());
        });
    });

    group.bench_function("query_with_limit", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = MetricFilters::new()
                .with_metric_type(MetricType::ResearchQuality)
                .with_limit(100);
            black_box(storage.query_metrics(&filters).await.unwrap());
        });
    });

    group.bench_function("query_aggregated", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);
            black_box(
                storage
                    .get_aggregated_metrics(&filters, Duration::from_secs(3600))
                    .await
                    .unwrap(),
            );
        });
    });

    group.finish();
}

// Benchmark memory efficiency
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_usage");

    // Test memory usage with different metric counts
    for metric_count in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_footprint", metric_count),
            metric_count,
            |b, &count| {
                b.iter(|| {
                    let storage = InMemoryMetricsStorage::new();

                    rt.block_on(async {
                        let metrics: Vec<QualityMetric> = (0..count)
                            .map(|i| create_test_metric("provider", 0.7 + (i as f64 * 0.0001)))
                            .collect();

                        storage.store_metrics(&metrics).await.unwrap();

                        // Force evaluation to ensure memory is actually allocated
                        let stats = storage.get_storage_stats().await.unwrap();
                        black_box(stats.total_metrics);
                    });
                });
            },
        );
    }

    group.finish();
}

// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_operations");

    let storage = Arc::new(InMemoryMetricsStorage::new());
    let config = MetricsConfig::default();
    let collector = Arc::new(MetricsCollector::new(config, storage.clone()));

    // Start collector
    rt.block_on(async {
        collector.start().await.unwrap();
    });

    group.bench_function("concurrent_collection", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks: Vec<_> = (0..10)
                .map(|i| {
                    let collector = Arc::clone(&collector);
                    tokio::spawn(async move {
                        for j in 0..10 {
                            let metric = create_test_metric(
                                &format!("provider_{i}"),
                                0.5 + ((i * 10 + j) as f64 * 0.001),
                            );
                            collector.collect(metric).await.unwrap();
                        }
                    })
                })
                .collect();

            for task in tasks {
                task.await.unwrap();
            }
        });
    });

    // Stop collector
    rt.block_on(async {
        collector.stop().await.unwrap();
    });

    group.finish();
}

// Custom performance validation
fn performance_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("performance_validation");
    group.sample_size(100);

    // Validate collection overhead requirement: <5ms per operation
    let config = MetricsConfig {
        enable_realtime: true,
        enable_batch: false,
        ..Default::default()
    };
    let storage = Arc::new(InMemoryMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage);

    group.bench_function("validate_collection_overhead", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            let metric = create_test_metric("claude", 0.85);
            collector.collect(metric).await.unwrap();

            let duration = start.elapsed();

            // Assert performance requirement: should be well under 5ms
            assert!(
                duration < Duration::from_millis(5),
                "Collection took {duration:?}, requirement is <5ms"
            );

            black_box(duration);
        });
    });

    // Validate query performance requirement: <100ms for standard queries
    let storage = InMemoryMetricsStorage::new();
    rt.block_on(async {
        let metrics: Vec<QualityMetric> = (0..1000)
            .map(|i| create_test_metric("provider", 0.7 + (i as f64 * 0.0001)))
            .collect();
        storage.store_metrics(&metrics).await.unwrap();
    });

    group.bench_function("validate_query_performance", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            let filters = MetricFilters::new()
                .with_metric_type(MetricType::ResearchQuality)
                .with_limit(100);
            let results = storage.query_metrics(&filters).await.unwrap();

            let duration = start.elapsed();

            // Assert performance requirement: should be well under 100ms
            assert!(
                duration < Duration::from_millis(100),
                "Query took {duration:?}, requirement is <100ms"
            );

            black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_metric_collection,
    bench_batch_collection,
    bench_storage_throughput,
    bench_query_performance,
    bench_memory_usage,
    bench_concurrent_operations,
    performance_validation
);
criterion_main!(benches);
