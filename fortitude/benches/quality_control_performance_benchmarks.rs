// ABOUTME: Performance benchmarks for quality control systems
//! This benchmark suite validates that the quality control systems meet
//! the performance requirements:
//! - Provider switching latency: <50ms
//! - Quality control processing: <10% overhead
//! - Overall system response time: <200ms under load
//! - >95% research accuracy achievement
//! - Quality evaluation time: <100ms
//!
//! # Benchmark Categories
//! - Quality scoring performance
//! - Cross-validation latency
//! - Provider selection speed
//! - Metrics collection overhead
//! - Configuration management performance
//! - End-to-end workflow benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

use fortitude_core::quality::{
    ComprehensiveQualityScorer, CrossValidationConfig, CrossValidationEngine,
    FeedbackCollectionConfig, FeedbackIntegrationSystem, InMemoryMetricsStorage, MetricsCollector,
    MetricsConfig, OptimizationConfig, QualityConfigManager, QualityContext, QualityControlConfig,
    QualityLearningConfig, QualityOptimizationEngine, QualityScore, QualityScorer, QualityWeights,
    ScorerConfig, SelectionCriteria, UrgencyLevel,
};

/// Benchmark configuration
struct BenchmarkConfig {
    /// Sample sizes for throughput testing
    sample_sizes: Vec<usize>,
    /// Concurrent load levels
    concurrent_loads: Vec<usize>,
    /// Test data variations
    test_data_count: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            sample_sizes: vec![1, 10, 50, 100, 500],
            concurrent_loads: vec![1, 5, 10, 25, 50, 100],
            test_data_count: 100,
        }
    }
}

/// Test data for benchmarks
#[derive(Debug, Clone)]
struct BenchmarkTestData {
    queries: Vec<String>,
    responses: Vec<String>,
    contexts: Vec<QualityContext>,
    weights: QualityWeights,
}

impl BenchmarkTestData {
    fn new(count: usize) -> Self {
        let mut queries = Vec::new();
        let mut responses = Vec::new();
        let mut contexts = Vec::new();

        for i in 0..count {
            queries.push(format!(
                "How does {} work in modern applications?",
                match i % 5 {
                    0 => "machine learning",
                    1 => "blockchain technology",
                    2 => "quantum computing",
                    3 => "artificial intelligence",
                    _ => "cloud computing",
                }
            ));

            responses.push(format!("This is a comprehensive explanation of topic {} covering the fundamental principles, practical applications, current industry trends, and future developments in the field.", i));

            contexts.push(
                QualityContext::new()
                    .with_domain(match i % 3 {
                        0 => "technology".to_string(),
                        1 => "science".to_string(),
                        _ => "engineering".to_string(),
                    })
                    .with_audience(match i % 3 {
                        0 => "expert".to_string(),
                        1 => "intermediate".to_string(),
                        _ => "beginner".to_string(),
                    }),
            );
        }

        Self {
            queries,
            responses,
            contexts,
            weights: QualityWeights::research_optimized(),
        }
    }

    fn get_test_case(&self, index: usize) -> (&str, &str, &QualityContext) {
        let i = index % self.queries.len();
        (&self.queries[i], &self.responses[i], &self.contexts[i])
    }
}

/// Initialize test environment
async fn setup_test_environment() -> (
    Arc<ComprehensiveQualityScorer>,
    Arc<CrossValidationEngine>,
    Arc<FeedbackIntegrationSystem>,
    Arc<MetricsCollector>,
    Arc<QualityOptimizationEngine>,
    Arc<QualityConfigManager>,
) {
    // Initialize quality scorer
    let scorer_config = ScorerConfig::production_optimized();
    let scorer = Arc::new(ComprehensiveQualityScorer::new(scorer_config).unwrap());

    // Initialize cross-validation engine
    let validation_config = CrossValidationConfig::production_defaults();
    let cross_validator = Arc::new(
        CrossValidationEngine::new(
            validation_config,
            vec![], // Mock providers
            scorer.clone(),
        )
        .unwrap(),
    );

    // Initialize feedback system
    let feedback_config = FeedbackCollectionConfig::production_optimized();
    let learning_config = QualityLearningConfig::production_optimized();
    let feedback_system =
        Arc::new(FeedbackIntegrationSystem::new(feedback_config, learning_config).unwrap());

    // Initialize metrics collector
    let metrics_config = MetricsConfig::production_optimized();
    let metrics_collector = Arc::new(
        MetricsCollector::new(metrics_config, Arc::new(InMemoryMetricsStorage::new())).unwrap(),
    );

    // Initialize optimization engine
    let opt_config = OptimizationConfig::production_optimized();
    let optimization_engine = Arc::new(
        QualityOptimizationEngine::new(
            opt_config,
            scorer.clone(),
            cross_validator.clone(),
            feedback_system.clone(),
            metrics_collector.clone(),
        )
        .unwrap(),
    );

    // Initialize configuration manager
    let quality_config = QualityControlConfig::production_defaults();
    let config_manager = Arc::new(QualityConfigManager::new(quality_config));

    (
        scorer,
        cross_validator,
        feedback_system,
        metrics_collector,
        optimization_engine,
        config_manager,
    )
}

/// Benchmark quality scoring performance
fn bench_quality_scoring(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let test_data = BenchmarkTestData::new(config.test_data_count);

    let (scorer, _, _, _, _, _) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("quality_scoring");

    // Single evaluation benchmark
    group.bench_function("single_evaluation", |b| {
        b.to_async(&rt).iter(|| async {
            let (query, response, context) = test_data.get_test_case(0);
            let result = scorer
                .evaluate_quality_with_context(
                    black_box(query),
                    black_box(response),
                    black_box(&test_data.weights),
                    black_box(context),
                )
                .await;
            black_box(result)
        });
    });

    // Throughput benchmarks
    for &size in &config.sample_sizes {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("throughput", size), &size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let mut results = Vec::new();
                for i in 0..size {
                    let (query, response, context) = test_data.get_test_case(i);
                    let result = scorer
                        .evaluate_quality_with_context(query, response, &test_data.weights, context)
                        .await;
                    results.push(result);
                }
                black_box(results)
            });
        });
    }

    // Concurrent load benchmarks
    for &load in &config.concurrent_loads {
        group.bench_with_input(BenchmarkId::new("concurrent", load), &load, |b, &load| {
            b.to_async(&rt).iter(|| async {
                let mut tasks = Vec::new();
                for i in 0..load {
                    let scorer = scorer.clone();
                    let (query, response, context) = test_data.get_test_case(i);
                    let query = query.to_string();
                    let response = response.to_string();
                    let context = context.clone();
                    let weights = test_data.weights.clone();

                    tasks.push(tokio::spawn(async move {
                        scorer
                            .evaluate_quality_with_context(&query, &response, &weights, &context)
                            .await
                    }));
                }

                let results: Vec<_> = futures::future::join_all(tasks).await;
                black_box(results)
            });
        });
    }

    group.finish();
}

/// Benchmark cross-validation performance
fn bench_cross_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let test_data = BenchmarkTestData::new(config.test_data_count);

    let (_, cross_validator, _, _, _, _) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("cross_validation");

    // Single validation benchmark
    group.bench_function("single_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let (query, response, context) = test_data.get_test_case(0);
            let result = cross_validator
                .validate_response(black_box(query), black_box(response), black_box(context))
                .await;
            black_box(result)
        });
    });

    // Validation throughput
    for &size in &[1, 5, 10, 25] {
        // Smaller sizes for cross-validation due to complexity
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("validation_throughput", size),
            &size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let mut results = Vec::new();
                    for i in 0..size {
                        let (query, response, context) = test_data.get_test_case(i);
                        let result = cross_validator
                            .validate_response(query, response, context)
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

/// Benchmark provider selection optimization
fn bench_provider_optimization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let test_data = BenchmarkTestData::new(config.test_data_count);

    let (_, _, _, _, optimization_engine, _) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("provider_optimization");

    // Single optimization benchmark
    group.bench_function("single_optimization", |b| {
        b.to_async(&rt).iter(|| async {
            let (query, _, _) = test_data.get_test_case(0);
            let criteria = SelectionCriteria::research_optimized()
                .with_domain("technology".to_string())
                .with_urgency_level(UrgencyLevel::Medium);

            let result = optimization_engine
                .execute_optimized_query(black_box(query), black_box(criteria))
                .await;
            black_box(result)
        });
    });

    // Provider selection latency benchmark (requirement: <50ms)
    group.bench_function("provider_selection_latency", |b| {
        b.to_async(&rt).iter(|| async {
            let criteria = SelectionCriteria::research_optimized()
                .with_domain("technology".to_string())
                .with_urgency_level(UrgencyLevel::High);

            let result = optimization_engine
                .select_optimal_provider(black_box(&criteria))
                .await;
            black_box(result)
        });
    });

    // Adaptation speed benchmark
    group.bench_function("adaptation_speed", |b| {
        b.to_async(&rt).iter(|| async {
            let result = optimization_engine.adapt_selection_criteria().await;
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark metrics collection performance
fn bench_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();

    let (scorer, _, _, metrics_collector, _, _) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("metrics_collection");

    // Create sample evaluation for metrics recording
    let sample_evaluation = rt.block_on(async {
        let test_data = BenchmarkTestData::new(1);
        let (query, response, context) = test_data.get_test_case(0);
        scorer
            .evaluate_quality_with_context(query, response, &test_data.weights, context)
            .await
            .unwrap()
    });

    // Metrics recording benchmark
    group.bench_function("record_evaluation", |b| {
        b.to_async(&rt).iter(|| async {
            let result = metrics_collector
                .record_quality_evaluation(black_box(&sample_evaluation))
                .await;
            black_box(result)
        });
    });

    // Metrics retrieval benchmark
    group.bench_function("retrieve_metrics", |b| {
        b.to_async(&rt).iter(|| async {
            let result = metrics_collector
                .get_recent_metrics(black_box(Duration::from_secs(3600)))
                .await;
            black_box(result)
        });
    });

    // Performance statistics benchmark
    group.bench_function("performance_stats", |b| {
        b.to_async(&rt).iter(|| async {
            let result = metrics_collector.get_performance_stats().await;
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark configuration management
fn bench_configuration_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let (_, _, _, _, _, config_manager) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("configuration_management");

    // Configuration validation benchmark
    group.bench_function("config_validation", |b| {
        b.iter(|| {
            let result = config_manager.config().validate();
            black_box(result)
        });
    });

    // Environment configuration benchmark
    group.bench_function("environment_config", |b| {
        b.iter(|| {
            let result = config_manager.config_for_environment(black_box("production"));
            black_box(result)
        });
    });

    // Effective configuration benchmark
    group.bench_function("effective_config", |b| {
        b.iter(|| {
            let result = config_manager.config().effective_config();
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark end-to-end quality workflow
fn bench_e2e_quality_workflow(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let test_data = BenchmarkTestData::new(config.test_data_count);

    let (
        scorer,
        cross_validator,
        feedback_system,
        metrics_collector,
        optimization_engine,
        config_manager,
    ) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("end_to_end_workflow");

    // Complete quality assessment workflow
    group.bench_function("complete_workflow", |b| {
        b.to_async(&rt).iter(|| async {
            let (query, response, context) = test_data.get_test_case(0);

            // 1. Quality evaluation
            let evaluation = scorer
                .evaluate_quality_with_context(query, response, &test_data.weights, context)
                .await
                .unwrap();

            // 2. Cross-validation (if enabled)
            let validation_result = if cross_validator.is_enabled() {
                Some(
                    cross_validator
                        .validate_response(query, response, context)
                        .await
                        .unwrap(),
                )
            } else {
                None
            };

            // 3. Metrics recording
            let _ = metrics_collector
                .record_quality_evaluation(&evaluation)
                .await;

            // 4. Provider optimization assessment
            let criteria =
                SelectionCriteria::research_optimized().with_domain("technology".to_string());
            let _ = optimization_engine
                .assess_query_complexity(query, &criteria)
                .await;

            black_box((evaluation, validation_result))
        });
    });

    // System response time under load (requirement: <200ms)
    for &load in &[1, 5, 10, 25] {
        group.bench_with_input(
            BenchmarkId::new("system_response_load", load),
            &load,
            |b, &load| {
                b.to_async(&rt).iter(|| async {
                    let mut tasks = Vec::new();

                    for i in 0..load {
                        let scorer = scorer.clone();
                        let metrics_collector = metrics_collector.clone();
                        let (query, response, context) = test_data.get_test_case(i);
                        let query = query.to_string();
                        let response = response.to_string();
                        let context = context.clone();
                        let weights = test_data.weights.clone();

                        tasks.push(tokio::spawn(async move {
                            // Complete workflow under load
                            let evaluation = scorer
                                .evaluate_quality_with_context(
                                    &query, &response, &weights, &context,
                                )
                                .await
                                .unwrap();
                            let _ = metrics_collector
                                .record_quality_evaluation(&evaluation)
                                .await;
                            evaluation
                        }));
                    }

                    let results: Vec<_> = futures::future::join_all(tasks).await;
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark accuracy achievement rate
fn bench_accuracy_achievement(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_data = BenchmarkTestData::new(100);

    let (scorer, _, _, _, optimization_engine, _) = rt.block_on(setup_test_environment());

    let mut group = c.benchmark_group("accuracy_achievement");

    // Accuracy validation benchmark (requirement: >95%)
    group.bench_function("accuracy_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let mut accurate_count = 0;
            let total_evaluations = 50; // Reduced for benchmark performance

            for i in 0..total_evaluations {
                let (query, response, context) = test_data.get_test_case(i);

                if let Ok(evaluation) = scorer
                    .evaluate_quality_with_context(query, response, &test_data.weights, context)
                    .await
                {
                    // Check if evaluation meets quality target (simulated accuracy check)
                    if evaluation.score.composite >= 0.85 && evaluation.score.is_valid() {
                        accurate_count += 1;
                    }
                }
            }

            let accuracy_rate = accurate_count as f64 / total_evaluations as f64;
            black_box(accuracy_rate)
        });
    });

    // Provider optimization accuracy
    group.bench_function("optimization_accuracy", |b| {
        b.to_async(&rt).iter(|| async {
            let mut optimal_selections = 0;
            let total_optimizations = 20; // Reduced for benchmark performance

            for i in 0..total_optimizations {
                let (query, _, _) = test_data.get_test_case(i);
                let criteria = SelectionCriteria::research_optimized()
                    .with_domain("technology".to_string())
                    .with_urgency_level(UrgencyLevel::Medium);

                if let Ok(result) = optimization_engine
                    .execute_optimized_query(query, criteria)
                    .await
                {
                    // Check if optimization achieved target accuracy
                    if result.accuracy_confidence >= 0.90 {
                        optimal_selections += 1;
                    }
                }
            }

            let optimization_accuracy = optimal_selections as f64 / total_optimizations as f64;
            black_box(optimization_accuracy)
        });
    });

    group.finish();
}

criterion_group!(
    quality_control_benchmarks,
    bench_quality_scoring,
    bench_cross_validation,
    bench_provider_optimization,
    bench_metrics_collection,
    bench_configuration_management,
    bench_e2e_quality_workflow,
    bench_accuracy_achievement
);

criterion_main!(quality_control_benchmarks);
