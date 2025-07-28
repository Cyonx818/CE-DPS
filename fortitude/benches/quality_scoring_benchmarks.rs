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

// ABOUTME: Performance benchmarks for quality scoring algorithms
//! Comprehensive benchmarks to validate that the quality scoring system
//! meets performance requirements (<100ms evaluation time) and scales
//! efficiently with different input sizes and complexity levels.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fortitude::quality::{
    ComprehensiveQualityScorer, QualityContext, QualityScorer, QualityWeights,
};
use std::time::Duration;

/// Benchmark basic quality evaluation performance
fn bench_quality_evaluation(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    let test_cases = vec![
        ("Short", "What is AI?", "AI is artificial intelligence."),
        ("Medium", "Explain machine learning algorithms", "Machine learning algorithms are computational methods that enable systems to learn patterns from data and make predictions. Common types include supervised learning (like linear regression and decision trees), unsupervised learning (like clustering), and reinforcement learning."),
        ("Long", "Provide a comprehensive analysis of deep learning architectures and their applications", "Deep learning architectures represent a sophisticated subset of machine learning that has revolutionized artificial intelligence applications across multiple domains. These neural network-based systems are characterized by their multi-layered structure, enabling them to automatically extract hierarchical features from raw data without extensive manual feature engineering. Convolutional Neural Networks (CNNs) have particularly excelled in computer vision tasks, utilizing spatial hierarchies through convolution operations, pooling layers, and feature maps to identify patterns in images. Recurrent Neural Networks (RNNs), including Long Short-Term Memory (LSTM) and Gated Recurrent Unit (GRU) variants, have proven invaluable for sequential data processing, particularly in natural language processing and time series analysis. Transformer architectures, introduced through attention mechanisms, have achieved state-of-the-art results in language understanding and generation tasks, forming the foundation for models like BERT, GPT, and T5."),
    ];

    let mut group = c.benchmark_group("quality_evaluation");

    for (size, query, response) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("basic_evaluation", size),
            &(query, response),
            |b, (q, r)| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let result = scorer
                            .evaluate_quality(black_box(q), black_box(r), black_box(&weights))
                            .await;
                        black_box(result)
                    });
            },
        );
    }

    group.finish();
}

/// Benchmark quality evaluation with context
fn bench_quality_evaluation_with_context(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();
    let context = QualityContext::new()
        .with_domain("artificial intelligence".to_string())
        .with_audience("researcher".to_string());

    let query = "Analyze the current state of neural network architectures";
    let response = "Neural network architectures have evolved significantly, with transformer models achieving breakthrough performance in natural language processing and computer vision tasks through attention mechanisms and parallel processing capabilities.";

    c.bench_function("quality_evaluation_with_context", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let result = scorer
                    .evaluate_quality_with_context(
                        black_box(query),
                        black_box(response),
                        black_box(&weights),
                        black_box(&context),
                    )
                    .await;
                black_box(result)
            });
    });
}

/// Benchmark feature extraction performance
fn bench_feature_extraction(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let context = QualityContext::new().with_domain("machine learning".to_string());

    let query = "Explain the principles of reinforcement learning";
    let response = "Reinforcement learning is a machine learning paradigm where agents learn to make decisions by interacting with an environment to maximize cumulative reward.";

    c.bench_function("feature_extraction", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let result = scorer
                    .extract_features(black_box(query), black_box(response), black_box(&context))
                    .await;
                black_box(result)
            });
    });
}

/// Benchmark different weight configurations
fn bench_weight_configurations(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let query = "What are the applications of computer vision?";
    let response = "Computer vision applications include autonomous vehicles, medical imaging, facial recognition, object detection, and augmented reality systems.";

    let weight_configs = vec![
        ("default", QualityWeights::default()),
        ("research_optimized", QualityWeights::research_optimized()),
        ("fact_checking", QualityWeights::fact_checking_optimized()),
    ];

    let mut group = c.benchmark_group("weight_configurations");

    for (name, weights) in weight_configs {
        group.bench_with_input(BenchmarkId::new("evaluation", name), &weights, |b, w| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let result = scorer
                        .evaluate_quality(black_box(query), black_box(response), black_box(w))
                        .await;
                    black_box(result)
                });
        });
    }

    group.finish();
}

/// Benchmark batch processing simulation
fn bench_batch_processing(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    let queries_responses = vec![
        (
            "What is machine learning?",
            "Machine learning enables computers to learn from data.",
        ),
        (
            "Explain neural networks",
            "Neural networks are computational models inspired by biological neurons.",
        ),
        (
            "Define artificial intelligence",
            "Artificial intelligence is the simulation of human intelligence in machines.",
        ),
        (
            "What is deep learning?",
            "Deep learning uses multi-layered neural networks for complex pattern recognition.",
        ),
        (
            "Describe computer vision",
            "Computer vision enables machines to interpret and understand visual information.",
        ),
    ];

    let batch_sizes = vec![1, 5, 10];

    let mut group = c.benchmark_group("batch_processing");

    for batch_size in batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &batch_size,
            |b, &size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async {
                        let mut results = Vec::new();
                        for i in 0..size {
                            let idx = i % queries_responses.len();
                            let (query, response) = &queries_responses[idx];
                            let result = scorer
                                .evaluate_quality(
                                    black_box(query),
                                    black_box(response),
                                    black_box(&weights),
                                )
                                .await;
                            results.push(result);
                        }
                        black_box(results)
                    });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage with different input sizes
fn bench_memory_efficiency(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::default();

    // Generate responses of different sizes
    let base_response = "Machine learning is a powerful technology that enables computers to learn patterns from data and make intelligent decisions without explicit programming. ";
    let responses = vec![
        ("small", base_response.repeat(1)),
        ("medium", base_response.repeat(5)),
        ("large", base_response.repeat(20)),
    ];

    let mut group = c.benchmark_group("memory_efficiency");

    for (size, response) in responses {
        group.bench_with_input(BenchmarkId::new("memory", size), &response, |b, r| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let result = scorer
                        .evaluate_quality(
                            black_box("Explain machine learning"),
                            black_box(r),
                            black_box(&weights),
                        )
                        .await;
                    black_box(result)
                });
        });
    }

    group.finish();
}

/// Benchmark performance threshold validation
fn bench_performance_threshold(c: &mut Criterion) {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    let query = "Analyze the impact of artificial intelligence on modern society";
    let response = "Artificial intelligence has transformed modern society across multiple domains including healthcare diagnostics, autonomous transportation, financial services, entertainment recommendation systems, and scientific research acceleration.";

    c.bench_function("performance_threshold_validation", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter_custom(|iters| {
                let scorer_clone = scorer.clone();
                let weights_clone = weights.clone();
                async move {
                    let start = std::time::Instant::now();

                    for _ in 0..iters {
                        let eval_start = std::time::Instant::now();
                        let result = scorer_clone
                            .evaluate_quality(
                                black_box(query),
                                black_box(response),
                                black_box(&weights_clone),
                            )
                            .await;
                        let eval_time = eval_start.elapsed();

                        // Assert performance requirement
                        assert!(
                            eval_time < Duration::from_millis(100),
                            "Evaluation took {:?}, should be < 100ms",
                            eval_time
                        );

                        black_box(result);
                    }

                    start.elapsed()
                }
            });
    });
}

criterion_group!(
    benches,
    bench_quality_evaluation,
    bench_quality_evaluation_with_context,
    bench_feature_extraction,
    bench_weight_configurations,
    bench_batch_processing,
    bench_memory_efficiency,
    bench_performance_threshold
);

criterion_main!(benches);
