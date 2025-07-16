// ABOUTME: Demonstration of hybrid search capabilities combining vector and keyword search
//! This example showcases the hybrid search functionality that combines semantic vector
//! search with traditional keyword search for improved relevance and precision.

use fortitude_core::vector::{
    client::QdrantClient,
    config::VectorConfig,
    embeddings::{EmbeddingConfig, LocalEmbeddingService},
    hybrid::{
        FusionMethod, HybridSearchConfig, HybridSearchOperations, HybridSearchService,
        KeywordSearcher, SearchStrategy,
    },
    search::{SearchOptions, SemanticSearchService},
    storage::{DocumentMetadata, VectorDocument, VectorStorage},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🔍 Hybrid Search Demo - Combining Vector and Keyword Search");
    println!("============================================================");

    // Create mock documents for demonstration
    let documents = create_sample_documents();
    println!("📚 Created {} sample documents", documents.len());

    // Initialize components
    let vector_config = VectorConfig::default();
    let qdrant_client = Arc::new(QdrantClient::new(vector_config.clone()).await?);
    let embedding_config = EmbeddingConfig::default();
    let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));
    let vector_storage = Arc::new(VectorStorage::new(qdrant_client, embedding_service));

    // Initialize semantic search service
    let semantic_service = Arc::new(SemanticSearchService::with_defaults(vector_storage.clone()));

    // Initialize keyword searcher and index documents
    let mut keyword_searcher = KeywordSearcher::new();
    keyword_searcher.index_documents(documents.clone()).await?;
    let keyword_searcher = Arc::new(keyword_searcher);

    // Create hybrid search service
    let hybrid_config = HybridSearchConfig::default();
    let hybrid_service =
        HybridSearchService::new(semantic_service, keyword_searcher, hybrid_config);

    // Initialize the hybrid search service
    hybrid_service.initialize().await?;
    println!("✅ Hybrid search service initialized");

    // Demonstrate different search strategies
    println!("\n🎯 Testing Different Search Strategies");
    println!("======================================");

    let query = "async programming patterns in Rust";
    let search_options = SearchOptions::default();

    // Test different strategies
    let strategies = vec![
        SearchStrategy::Balanced,
        SearchStrategy::SemanticFocus,
        SearchStrategy::KeywordFocus,
        SearchStrategy::VectorOnly,
        SearchStrategy::KeywordOnly,
    ];

    println!("\n📋 Comparing strategies for query: '{query}'");
    let comparison_results = hybrid_service
        .compare_strategies(query, strategies, search_options.clone())
        .await?;

    for result_set in comparison_results {
        if let Some(strategy) = &result_set.request.strategy {
            println!("\n🔧 Strategy: {strategy:?}");
            println!("   Results: {} found", result_set.results.len());
            println!(
                "   Execution time: {:.2}ms",
                result_set.execution_stats.total_time_ms
            );
            println!(
                "   Vector time: {:.2}ms",
                result_set.execution_stats.vector_search_time_ms
            );
            println!(
                "   Keyword time: {:.2}ms",
                result_set.execution_stats.keyword_search_time_ms
            );

            // Show top result if available
            if let Some(top_result) = result_set.results.first() {
                println!(
                    "   Top result: {} (score: {:.3})",
                    top_result.document.id, top_result.hybrid_score
                );
            }
        }
    }

    // Demonstrate adaptive search
    println!("\n🧠 Adaptive Search");
    println!("==================");

    let adaptive_result = hybrid_service
        .adaptive_search(query, search_options.clone())
        .await?;
    println!("Query: '{query}'");
    println!(
        "Recommended strategy: {:?}",
        adaptive_result.query_analysis.recommended_strategy
    );
    println!(
        "Query type: {:?}",
        adaptive_result.query_analysis.query_type
    );
    println!(
        "Complexity: {:.2}",
        adaptive_result.query_analysis.complexity
    );
    println!(
        "Technical terms: {:?}",
        adaptive_result.query_analysis.technical_terms
    );

    // Demonstrate fusion methods
    println!("\n🔀 Testing Different Fusion Methods");
    println!("===================================");

    let fusion_methods = vec![
        FusionMethod::ReciprocalRankFusion,
        FusionMethod::WeightedScoring,
        FusionMethod::RankFusion,
    ];

    for fusion_method in fusion_methods {
        let result = hybrid_service
            .search_with_fusion(query, fusion_method.clone(), search_options.clone())
            .await?;

        println!("\n🔗 Fusion method: {fusion_method:?}");
        println!("   Results: {} found", result.results.len());
        println!(
            "   Fusion time: {:.2}ms",
            result.execution_stats.fusion_time_ms
        );

        if let Some(top_result) = result.results.first() {
            println!(
                "   Top result: {} (score: {:.3})",
                top_result.document.id, top_result.hybrid_score
            );
        }
    }

    // Demonstrate custom weights
    println!("\n⚖️  Custom Weight Testing");
    println!("========================");

    let weight_combinations = vec![
        (0.8, 0.2), // Vector-heavy
        (0.5, 0.5), // Balanced
        (0.2, 0.8), // Keyword-heavy
    ];

    for (vector_weight, keyword_weight) in weight_combinations {
        let result = hybrid_service
            .search_with_strategy(
                query,
                SearchStrategy::Custom {
                    vector_weight,
                    keyword_weight,
                },
                None,
                search_options.clone(),
            )
            .await?;

        println!("\n⚖️  Weights: Vector {vector_weight:.1}, Keyword {keyword_weight:.1}");
        println!("   Results: {} found", result.results.len());

        if let Some(top_result) = result.results.first() {
            println!(
                "   Top result: {} (score: {:.3})",
                top_result.document.id, top_result.hybrid_score
            );

            if let Some(explanation) = &top_result.explanation {
                println!(
                    "   Vector contribution: {:?}",
                    explanation.vector_contribution
                );
                println!(
                    "   Keyword contribution: {:?}",
                    explanation.keyword_contribution
                );
            }
        }
    }

    // Show performance metrics
    println!("\n📊 Performance Metrics");
    println!("=====================");

    let metrics = hybrid_service.get_performance_metrics().await?;
    println!("Total searches: {}", metrics.total_searches);
    println!(
        "Average execution time: {:.2}ms",
        metrics.avg_execution_time_ms
    );
    println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);
    println!("Average result count: {:.1}", metrics.avg_result_count);

    println!("\nStrategy usage:");
    for (strategy, count) in &metrics.strategy_distribution {
        println!("  {strategy}: {count} times");
    }

    println!("\n✅ Hybrid Search Demo Complete!");
    Ok(())
}

fn create_sample_documents() -> Vec<VectorDocument> {
    vec![
        VectorDocument {
            id: "doc1".to_string(),
            content: "Rust async programming with tokio runtime for high-performance applications"
                .to_string(),
            embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5], // Mock embedding
            metadata: DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Implementation),
                content_type: "guide".to_string(),
                quality_score: Some(0.9),
                source: Some("rust-docs".to_string()),
                tags: vec!["rust".to_string(), "async".to_string(), "tokio".to_string()],
                custom_fields: std::collections::HashMap::new(),
            },
            stored_at: chrono::Utc::now(),
        },
        VectorDocument {
            id: "doc2".to_string(),
            content:
                "JavaScript async/await patterns and Promise handling in modern web development"
                    .to_string(),
            embedding: vec![0.2, 0.3, 0.4, 0.5, 0.6], // Mock embedding
            metadata: DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Learning),
                content_type: "tutorial".to_string(),
                quality_score: Some(0.8),
                source: Some("mdn-docs".to_string()),
                tags: vec![
                    "javascript".to_string(),
                    "async".to_string(),
                    "promises".to_string(),
                ],
                custom_fields: std::collections::HashMap::new(),
            },
            stored_at: chrono::Utc::now(),
        },
        VectorDocument {
            id: "doc3".to_string(),
            content:
                "Rust systems programming fundamentals: memory safety and zero-cost abstractions"
                    .to_string(),
            embedding: vec![0.3, 0.4, 0.5, 0.6, 0.7], // Mock embedding
            metadata: DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Learning),
                content_type: "reference".to_string(),
                quality_score: Some(0.95),
                source: Some("rust-book".to_string()),
                tags: vec![
                    "rust".to_string(),
                    "systems".to_string(),
                    "memory".to_string(),
                ],
                custom_fields: std::collections::HashMap::new(),
            },
            stored_at: chrono::Utc::now(),
        },
        VectorDocument {
            id: "doc4".to_string(),
            content:
                "Python asyncio library usage patterns for concurrent programming in data science"
                    .to_string(),
            embedding: vec![0.4, 0.5, 0.6, 0.7, 0.8], // Mock embedding
            metadata: DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Implementation),
                content_type: "example".to_string(),
                quality_score: Some(0.7),
                source: Some("python-docs".to_string()),
                tags: vec![
                    "python".to_string(),
                    "asyncio".to_string(),
                    "concurrent".to_string(),
                ],
                custom_fields: std::collections::HashMap::new(),
            },
            stored_at: chrono::Utc::now(),
        },
        VectorDocument {
            id: "doc5".to_string(),
            content: "Database query optimization techniques for high-performance web applications"
                .to_string(),
            embedding: vec![0.5, 0.6, 0.7, 0.8, 0.9], // Mock embedding
            metadata: DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Validation),
                content_type: "optimization".to_string(),
                quality_score: Some(0.85),
                source: Some("db-guide".to_string()),
                tags: vec![
                    "database".to_string(),
                    "optimization".to_string(),
                    "performance".to_string(),
                ],
                custom_fields: std::collections::HashMap::new(),
            },
            stored_at: chrono::Utc::now(),
        },
    ]
}
