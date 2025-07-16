// ABOUTME: Demonstration of semantic search functionality
//! This example shows how to use the SemanticSearchService for vector-based
//! content discovery and similarity search operations.

use fortitude_core::vector::{
    client::QdrantClient,
    config::VectorConfig,
    embeddings::{EmbeddingConfig, LocalEmbeddingService},
    search::{SearchOptions, SemanticSearchConfig, SemanticSearchService, SuggestionRequest},
    storage::{DocumentMetadata, VectorStorage},
};
use std::sync::Arc;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> Result<(), fortitude_core::vector::VectorError> {
    // Initialize tracing for better debugging
    init();

    println!("🔍 Semantic Search Demo");
    println!("=====================");

    // Configuration for vector database
    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        default_collection: "fortitude_demo".to_string(),
        vector_dimensions: 384,
        ..VectorConfig::default()
    };

    // Configuration for embeddings
    let embedding_config = EmbeddingConfig::default();

    // Configuration for semantic search
    let search_config = SemanticSearchConfig {
        default_limit: 5,
        default_threshold: 0.7,
        max_limit: 50,
        enable_analytics: true,
        cache_results: true,
        cache_ttl_seconds: 300,
        enable_query_optimization: true,
        max_query_length: 8192,
    };

    println!("📡 Connecting to Qdrant at {}", vector_config.url);

    // Initialize components
    let qdrant_client = match QdrantClient::new(vector_config).await {
        Ok(client) => {
            println!("✅ Connected to Qdrant successfully");
            Arc::new(client)
        }
        Err(e) => {
            println!("❌ Failed to connect to Qdrant: {e}");
            println!("💡 Make sure Qdrant is running on localhost:6334");
            println!("   You can start it with: docker run -p 6334:6334 qdrant/qdrant");
            return Err(e);
        }
    };

    let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));
    let vector_storage = Arc::new(VectorStorage::new(qdrant_client, embedding_service));
    let search_service = SemanticSearchService::new(vector_storage.clone(), search_config);

    println!("🚀 Initializing search service...");
    search_service.initialize().await?;

    // Store some sample documents for demonstration
    println!("📝 Storing sample documents...");

    let sample_docs = vec![
        (
            "Rust async programming with tokio provides powerful concurrency primitives for building scalable applications. The async/await syntax makes asynchronous code more readable and maintainable.",
            DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Learning),
                content_type: "tutorial".to_string(),
                quality_score: Some(0.9),
                source: Some("rust-async-guide".to_string()),
                tags: vec!["rust".to_string(), "async".to_string(), "tokio".to_string()],
                custom_fields: std::collections::HashMap::new(),
            }
        ),
        (
            "Vector databases like Qdrant enable semantic search by storing high-dimensional embeddings that capture the meaning of text content. This allows for similarity-based retrieval.",
            DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Implementation),
                content_type: "concept".to_string(),
                quality_score: Some(0.85),
                source: Some("vector-db-guide".to_string()),
                tags: vec!["vector".to_string(), "database".to_string(), "embeddings".to_string()],
                custom_fields: std::collections::HashMap::new(),
            }
        ),
        (
            "Error handling in Rust uses Result types and the ? operator for propagating errors. This approach ensures that errors are handled explicitly and safely.",
            DocumentMetadata {
                research_type: Some(fortitude_types::research::ResearchType::Troubleshooting),
                content_type: "guide".to_string(),
                quality_score: Some(0.8),
                source: Some("rust-error-handling".to_string()),
                tags: vec!["rust".to_string(), "errors".to_string(), "safety".to_string()],
                custom_fields: std::collections::HashMap::new(),
            }
        ),
    ];

    let mut stored_docs = Vec::new();
    for (content, metadata) in sample_docs {
        match vector_storage.store_document(content, metadata).await {
            Ok(doc) => {
                println!("  ✅ Stored document: {}", &doc.id[..8]);
                stored_docs.push(doc);
            }
            Err(e) => {
                println!("  ❌ Failed to store document: {e}");
            }
        }
    }

    println!("\n🔍 Performing semantic searches...");

    // Example 1: Basic semantic search
    println!("\n1️⃣ Basic Search: 'async programming'");
    let search_options = SearchOptions {
        limit: 3,
        threshold: Some(0.5),
        include_explanations: true,
        ..SearchOptions::default()
    };

    match search_service
        .search_similar("async programming", search_options)
        .await
    {
        Ok(results) => {
            println!("   Found {} results:", results.results.len());
            for (i, result) in results.results.iter().enumerate() {
                println!(
                    "   {}. Score: {:.3} - {}",
                    i + 1,
                    result.relevance_score,
                    &result.document.content[..60]
                );
                if let Some(explanation) = &result.explanation {
                    println!("      📊 {}", explanation.calculation);
                }
            }
            println!(
                "   ⏱️  Search took: {:.2}ms",
                results.execution_stats.total_time_ms
            );
        }
        Err(e) => println!("   ❌ Search failed: {e}"),
    }

    // Example 2: Search with quality boost
    println!("\n2️⃣ Search with Quality Boost: 'vector embeddings'");
    let boosted_options = SearchOptions {
        limit: 3,
        quality_boost: Some(0.2),
        temporal_boost: Some(0.1),
        include_explanations: true,
        ..SearchOptions::default()
    };

    match search_service
        .search_similar("vector embeddings", boosted_options)
        .await
    {
        Ok(results) => {
            println!("   Found {} results with boosting:", results.results.len());
            for (i, result) in results.results.iter().enumerate() {
                println!(
                    "   {}. Score: {:.3} (sim: {:.3}) - {}",
                    i + 1,
                    result.relevance_score,
                    result.similarity_score,
                    &result.document.content[..60]
                );
            }
        }
        Err(e) => println!("   ❌ Search failed: {e}"),
    }

    // Example 3: Suggest related content
    if let Some(first_doc) = stored_docs.first() {
        println!(
            "\n3️⃣ Related Content Suggestions for document: {}",
            &first_doc.id[..8]
        );
        let suggestion_request = SuggestionRequest {
            document_id: first_doc.id.clone(),
            limit: 2,
            threshold: Some(0.3),
            exclude_source: true,
            filters: vec![],
        };

        match search_service.suggest_related(suggestion_request).await {
            Ok(results) => {
                println!("   Found {} related documents:", results.results.len());
                for (i, result) in results.results.iter().enumerate() {
                    println!(
                        "   {}. Score: {:.3} - {}",
                        i + 1,
                        result.relevance_score,
                        &result.document.content[..60]
                    );
                }
            }
            Err(e) => println!("   ❌ Suggestion failed: {e}"),
        }
    }

    // Example 4: Search analytics
    println!("\n📊 Search Analytics:");
    let analytics = search_service.get_analytics().await;
    println!("   Total searches: {}", analytics.total_searches);
    println!("   Avg search time: {:.2}ms", analytics.avg_search_time_ms);
    println!(
        "   Avg results per search: {:.1}",
        analytics.avg_results_per_search
    );
    if !analytics.popular_terms.is_empty() {
        println!("   Popular search terms:");
        for (term, count) in &analytics.popular_terms {
            println!("     - {term}: {count} times");
        }
    }

    println!("\n🎉 Demo completed successfully!");
    println!("💡 This demo shows basic semantic search capabilities.");
    println!("   In production, you would integrate this with your research pipeline.");

    Ok(())
}
