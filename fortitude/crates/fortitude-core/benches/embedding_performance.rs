// ABOUTME: Embedding generation performance benchmarks
//! This benchmark suite measures the performance of embedding generation,
//! caching strategies, batching, and text preprocessing operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude_core::vector::{
    CacheKeyStrategy, DeviceType, EmbeddingCacheConfig, EmbeddingConfig, EmbeddingGenerator,
    LocalEmbeddingService, ModelDownloadConfig, PreprocessingConfig,
};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Helper function to create embedding config with different cache settings
fn create_embedding_config_with_cache(cache_enabled: bool, max_entries: usize) -> EmbeddingConfig {
    EmbeddingConfig {
        model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        max_sequence_length: 512,
        batch_size: 32,
        device: DeviceType::Cpu,
        cache_config: EmbeddingCacheConfig {
            enabled: cache_enabled,
            max_entries,
            ttl: Duration::from_secs(3600),
            key_strategy: CacheKeyStrategy::Hash,
        },
        download_config: ModelDownloadConfig::default(),
        preprocessing: PreprocessingConfig::default(),
    }
}

/// Helper function to create config with different batch sizes
fn create_config_with_batch_size(batch_size: usize) -> EmbeddingConfig {
    EmbeddingConfig {
        batch_size,
        ..EmbeddingConfig::default()
    }
}

/// Helper function to create config with different preprocessing options
fn create_config_with_preprocessing(
    lowercase: bool,
    normalize_whitespace: bool,
    remove_special_chars: bool,
) -> EmbeddingConfig {
    EmbeddingConfig {
        preprocessing: PreprocessingConfig {
            lowercase,
            normalize_whitespace,
            remove_special_chars,
            max_text_length: 8192,
        },
        ..EmbeddingConfig::default()
    }
}

/// Generate test texts of different lengths
fn generate_test_texts(count: usize, length: usize) -> Vec<String> {
    let base_text =
        "This is a sample text that will be repeated to create texts of different lengths. ";
    let repeat_count = (length / base_text.len()).max(1);

    (0..count)
        .map(|i| {
            let mut text = base_text.repeat(repeat_count);
            text.push_str(&format!(" unique_id_{}", i));
            text.truncate(length);
            text
        })
        .collect()
}

/// Benchmark single embedding generation
fn bench_single_embedding_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("single_embedding_generation");

    // Test different text lengths
    for length in [50, 100, 500, 1000, 2000].iter() {
        group.throughput(Throughput::Bytes(*length as u64));
        group.bench_with_input(
            BenchmarkId::new("generate", length),
            length,
            |b, &length| {
                b.to_async(&rt).iter(|| async {
                    let config = EmbeddingConfig::default();
                    let service = LocalEmbeddingService::new(config);
                    let _ = service.initialize().await;

                    let text = "a".repeat(length);
                    let _embedding = service.generate_embedding(black_box(&text)).await;
                });
            },
        );
    }
    group.finish();
}

/// Benchmark batch embedding generation
fn bench_batch_embedding_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_embedding_generation");

    // Test different batch sizes
    for batch_size in [1, 5, 10, 20, 50, 100].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let config = create_config_with_batch_size(32); // Keep internal batch size constant
                    let service = LocalEmbeddingService::new(config);
                    let _ = service.initialize().await;

                    let texts = generate_test_texts(batch_size, 200);
                    let _embeddings = service.generate_embeddings(black_box(&texts)).await;
                });
            },
        );
    }
    group.finish();
}

/// Benchmark cache performance
fn bench_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_performance");

    // Test cache hit scenarios
    group.bench_function("cache_miss", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_embedding_config_with_cache(true, 1000);
            let service = LocalEmbeddingService::new(config);
            let _ = service.initialize().await;

            // Generate unique texts to ensure cache misses
            let text = format!(
                "unique_text_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let _embedding = service.generate_embedding(black_box(&text)).await;
        });
    });

    group.bench_function("cache_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_embedding_config_with_cache(true, 1000);
            let service = LocalEmbeddingService::new(config);
            let _ = service.initialize().await;

            // Use same text to ensure cache hits after first generation
            let text = "cached_text_for_hit_testing";
            let _embedding1 = service.generate_embedding(text).await;
            let _embedding2 = service.generate_embedding(black_box(text)).await;
            // This should be a cache hit
        });
    });

    group.bench_function("cache_disabled", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_embedding_config_with_cache(false, 0);
            let service = LocalEmbeddingService::new(config);
            let _ = service.initialize().await;

            let text = "text_without_caching";
            let _embedding = service.generate_embedding(black_box(text)).await;
        });
    });

    group.finish();
}

/// Benchmark different cache key strategies
fn bench_cache_key_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_key_strategies");

    for strategy in [
        CacheKeyStrategy::Hash,
        CacheKeyStrategy::LengthHash,
        CacheKeyStrategy::PrefixHash(10),
        CacheKeyStrategy::PrefixHash(50),
    ]
    .iter()
    {
        let strategy_name = match strategy {
            CacheKeyStrategy::Hash => "hash",
            CacheKeyStrategy::LengthHash => "length_hash",
            CacheKeyStrategy::PrefixHash(n) => &format!("prefix_hash_{}", n),
        };

        group.bench_with_input(
            BenchmarkId::new("key_generation", strategy_name),
            strategy,
            |b, strategy| {
                b.to_async(&rt).iter(|| async {
                    let config = EmbeddingConfig {
                        cache_config: EmbeddingCacheConfig {
                            enabled: true,
                            max_entries: 1000,
                            ttl: Duration::from_secs(3600),
                            key_strategy: strategy.clone(),
                        },
                        ..EmbeddingConfig::default()
                    };

                    let service = LocalEmbeddingService::new(config);
                    let text =
                        "test text for cache key generation benchmarking with various strategies";

                    // Generate cache key (this is internal, so we simulate the operation)
                    let _key = service.generate_cache_key(black_box(text));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark text preprocessing
fn bench_text_preprocessing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_preprocessing");

    let test_texts = vec![
        "Simple text",
        "Text with    extra   whitespace",
        "Text WITH Mixed Case LETTERS",
        "Text with special chars!@#$%^&*()_+-=[]{}|;':\",./<>?",
        "Very long text that needs to be truncated because it exceeds the maximum length limit and should be cut off at some point to test the truncation functionality of the preprocessing pipeline".repeat(10),
    ];

    for (i, text) in test_texts.iter().enumerate() {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("preprocess", i), text, |b, text| {
            b.iter(|| {
                let config = create_config_with_preprocessing(true, true, true);
                let service = LocalEmbeddingService::new(config);
                let _processed = service.preprocess_text(black_box(text));
            });
        });
    }
    group.finish();
}

/// Benchmark cache cleanup operations
fn bench_cache_cleanup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_cleanup");

    // Test cleanup with different cache sizes
    for cache_size in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*cache_size as u64));
        group.bench_with_input(
            BenchmarkId::new("cleanup", cache_size),
            cache_size,
            |b, &cache_size| {
                b.to_async(&rt).iter(|| async {
                    let config = create_embedding_config_with_cache(true, cache_size);
                    let service = LocalEmbeddingService::new(config);
                    let _ = service.initialize().await;

                    // Fill cache with entries
                    for i in 0..cache_size.min(100) {
                        // Don't generate too many embeddings for speed
                        let text = format!("cache_text_{}", i);
                        let _ = service.generate_embedding(&text).await;
                    }

                    // Benchmark cleanup operation
                    service.cleanup_cache().await;
                });
            },
        );
    }
    group.finish();
}

/// Benchmark concurrent embedding generation
fn bench_concurrent_embeddings(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_embeddings");

    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let config = create_embedding_config_with_cache(true, 1000);
                    let service = std::sync::Arc::new(LocalEmbeddingService::new(config));
                    let _ = service.initialize().await;

                    let mut handles = Vec::new();
                    for i in 0..concurrency {
                        let service_clone = service.clone();
                        let handle = tokio::spawn(async move {
                            let text = format!("concurrent_text_{}", i);
                            let _embedding = service_clone.generate_embedding(&text).await;
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        let _ = handle.await;
                    }
                });
            },
        );
    }
    group.finish();
}

/// Benchmark embedding statistics collection
fn bench_embedding_stats(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("stats_collection", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_embedding_config_with_cache(true, 1000);
            let service = LocalEmbeddingService::new(config);
            let _ = service.initialize().await;

            // Generate some embeddings to create stats
            for i in 0..10 {
                let text = format!("stats_text_{}", i);
                let _ = service.generate_embedding(&text).await;
            }

            let _stats = service.get_stats().await;
        });
    });
}

/// Benchmark memory usage patterns for embeddings
fn bench_embedding_memory(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("embedding_memory");

    // Test memory usage with different numbers of cached embeddings
    for count in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("memory_usage", count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let config = create_embedding_config_with_cache(true, count);
                    let service = LocalEmbeddingService::new(config);
                    let _ = service.initialize().await;

                    // Generate embeddings to fill memory
                    for i in 0..count.min(50) {
                        // Limit actual generations for speed
                        let text = format!("memory_test_{}", i);
                        let _embedding = service.generate_embedding(&text).await;
                    }

                    let _stats = service.get_stats().await;
                });
            },
        );
    }
    group.finish();
}

/// Performance regression detection for embeddings
fn bench_embedding_regression_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("embedding_performance_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            // Baseline embedding performance test
            let config = EmbeddingConfig::default();
            let service = LocalEmbeddingService::new(config);
            let start = std::time::Instant::now();
            // Initialize service
            let _ = service.initialize().await;
            // Generate embeddings for different text lengths
            let texts = vec![
                "Short text",
                "Medium length text that contains more words and characters for testing",
                "Very long text that simulates real-world usage with multiple sentences and complex content that might be typical of research documents or technical documentation".repeat(5),
            ];
            for text in texts {
                let _embedding = service.generate_embedding(black_box(&text)).await;
            }
            // Test batch processing
            let batch_texts = generate_test_texts(10, 200);
            let _batch_embeddings = service.generate_embeddings(black_box(&batch_texts)).await;
            // Test cache operations
            let _stats = service.get_stats().await;
            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
}

criterion_group!(
    benches,
    bench_single_embedding_generation,
    bench_batch_embedding_generation,
    bench_cache_performance,
    bench_cache_key_strategies,
    bench_text_preprocessing,
    bench_cache_cleanup,
    bench_concurrent_embeddings,
    bench_embedding_stats,
    bench_embedding_memory,
    bench_embedding_regression_detection
);

criterion_main!(benches);
