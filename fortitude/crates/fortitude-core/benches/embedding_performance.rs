use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude_core::vector::{VectorConfig, EmbeddingService, MockEmbeddingService};
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_test_vector_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        collection_name: "test_collection".to_string(),
        embedding_dimension: 384,
        ..Default::default()
    }
}

fn benchmark_embedding_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_vector_config();
    let service = MockEmbeddingService::new(config);

    let mut group = c.benchmark_group("embedding_generation");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(30));

    // Test different text lengths
    let test_cases = vec![
        ("short", "Simple test text"),
        ("medium", &"word ".repeat(50)),
        ("long", &"word ".repeat(200)),
    ];

    for (name, text) in test_cases {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("generate", name), text, |b, text| {
            b.to_async(&rt).iter(|| async {
                let _embedding = service.generate_embedding(black_box(text)).await;
                black_box(_embedding);
            });
        });
    }

    group.finish();
}

fn benchmark_batch_embedding_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_vector_config();
    let service = MockEmbeddingService::new(config);

    let mut group = c.benchmark_group("batch_embedding_generation");
    group.sample_size(30);

    let batch_sizes = [1, 5, 10, 20, 50];
    
    for &batch_size in &batch_sizes {
        let texts: Vec<String> = (0..batch_size)
            .map(|i| format!("test_text_{i}"))
            .collect();
        
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &texts,
            |b, texts| {
                b.to_async(&rt).iter(|| async {
                    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                    let _embeddings = service.generate_batch_embeddings(black_box(text_refs)).await;
                    black_box(_embeddings);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_text_preprocessing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_vector_config();
    let service = MockEmbeddingService::new(config);

    let mut group = c.benchmark_group("text_preprocessing");
    
    let test_texts = [
        "Simple text",
        "Text with    extra   whitespace",
        "Text WITH Mixed Case LETTERS",
        "Text with special chars!@#$%^&*()_+-=[]{}|;':\",./<>?",
        &"a".repeat(10000), // Long text for truncation testing
    ];

    for (i, text) in test_texts.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("preprocess", i),
            text,
            |b, text| {
                b.to_async(&rt).iter(|| async {
                    let _embedding = service.generate_embedding(black_box(text)).await;
                    black_box(_embedding);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_embedding_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_vector_config();
    let service = MockEmbeddingService::new(config);

    let mut group = c.benchmark_group("concurrent_embedding");
    
    let concurrency_levels = [1, 2, 4, 8, 16];
    
    for &concurrency in &concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let tasks: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let service = &service;
                            async move {
                                let text = format!("stats_text_{i}");
                                let _embedding = service.generate_embedding(black_box(&text)).await;
                                black_box(_embedding);
                            }
                        })
                        .collect();
                    
                    futures::future::join_all(tasks).await;
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_embedding_generation,
    benchmark_batch_embedding_generation,
    benchmark_text_preprocessing,
    benchmark_concurrent_embedding_generation
);
criterion_main!(benches);