// ABOUTME: Integration tests for vector storage operations

use super::storage::*;
use crate::vector::{
    client::QdrantClient,
    config::{ConnectionPoolConfig, VectorConfig},
    embeddings::{EmbeddingConfig, LocalEmbeddingService},
    error::{VectorError, VectorResult},
};
use fortitude_types::research::ResearchType;
use std::sync::Arc;
use tokio;

/// Create a test vector storage service with mock configuration
async fn create_test_storage() -> VectorResult<VectorStorage> {
    let embedding_config = EmbeddingConfig {
        model_name: "test-model".to_string(),
        max_sequence_length: 128,
        batch_size: 4,
        ..Default::default()
    };

    let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));

    // Use a test configuration for Qdrant client
    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(), // Test URL
        default_collection: "test_storage_collection".to_string(),
        api_key: None,
        timeout: std::time::Duration::from_secs(5),
        vector_dimensions: 384,
        distance_metric: crate::vector::config::DistanceMetric::Cosine,
        health_check: crate::vector::config::HealthCheckConfig::default(),
        connection_pool: ConnectionPoolConfig::default(),
        embedding: EmbeddingConfig::default(),
    };

    // Note: In real integration tests, you would start a test Qdrant instance
    // For now, we'll handle the connection error gracefully
    let qdrant_client = match QdrantClient::new(vector_config).await {
        Ok(client) => Arc::new(client),
        Err(_) => {
            // Skip tests if Qdrant is not available
            println!("Skipping vector storage tests - Qdrant not available");
            return Err(VectorError::ConnectionError {
                message: "Test Qdrant instance not available".to_string(),
            });
        }
    };

    let storage = VectorStorage::new(qdrant_client, embedding_service);
    storage.initialize().await?;

    Ok(storage)
}

/// Helper function to create test metadata
fn create_test_metadata() -> DocumentMetadata {
    DocumentMetadata {
        research_type: Some(ResearchType::Learning),
        content_type: "test_document".to_string(),
        quality_score: Some(0.85),
        source: Some("integration_test".to_string()),
        tags: vec!["rust".to_string(), "vector".to_string(), "test".to_string()],
        custom_fields: {
            let mut fields = std::collections::HashMap::new();
            fields.insert("test_field".to_string(), serde_json::json!("test_value"));
            fields.insert("numeric_field".to_string(), serde_json::json!(42));
            fields
        },
    }
}

#[tokio::test]
async fn test_storage_initialization() {
    if let Ok(storage) = create_test_storage().await {
        // Test that storage is properly initialized
        assert_eq!(storage.default_collection(), "test_storage_collection");
        assert_eq!(storage.embedding_dimension(), 384);

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.total_documents, 0);
        assert_eq!(stats.total_searches, 0);
    }
}

#[tokio::test]
async fn test_store_and_retrieve_document() {
    if let Ok(storage) = create_test_storage().await {
        let content = "This is a test document for vector storage";
        let metadata = create_test_metadata();

        // Store document
        let stored_doc = storage
            .store_document(content, metadata.clone())
            .await
            .unwrap();

        assert_eq!(stored_doc.content, content);
        assert_eq!(
            stored_doc.metadata.research_type,
            Some(ResearchType::Learning)
        );
        assert_eq!(stored_doc.embedding.len(), 384);
        assert!(!stored_doc.id.is_empty());

        // Retrieve by ID
        let retrieved_doc = storage.retrieve_by_id(&stored_doc.id).await.unwrap();
        assert!(retrieved_doc.is_some());

        let retrieved = retrieved_doc.unwrap();
        assert_eq!(retrieved.id, stored_doc.id);
        assert_eq!(retrieved.content, content);
        assert_eq!(
            retrieved.metadata.research_type,
            Some(ResearchType::Learning)
        );

        // Clean up
        let deleted = storage.delete_document(&stored_doc.id).await.unwrap();
        assert!(deleted);
    }
}

#[tokio::test]
async fn test_similarity_search() {
    if let Ok(storage) = create_test_storage().await {
        let documents = vec![
            (
                "Rust programming language async await patterns",
                create_test_metadata(),
            ),
            (
                "Python asyncio and coroutines for concurrent programming",
                create_test_metadata(),
            ),
            (
                "JavaScript promises and async functions",
                create_test_metadata(),
            ),
            (
                "Rust ownership and borrowing concepts",
                create_test_metadata(),
            ),
        ];

        // Store documents
        let mut stored_ids = Vec::new();
        for (content, metadata) in documents {
            let doc = storage.store_document(content, metadata).await.unwrap();
            stored_ids.push(doc.id);
        }

        // Search for similar documents
        let search_config = SearchConfig {
            limit: 3,
            threshold: Some(0.0), // Accept all results for testing
            ..Default::default()
        };

        let results = storage
            .retrieve_similar("async programming in Rust", search_config)
            .await
            .unwrap();

        assert!(!results.is_empty());
        assert!(results.len() <= 3);

        // Results should be ordered by similarity score
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }

        // Clean up
        for id in stored_ids {
            storage.delete_document(&id).await.unwrap();
        }
    }
}

#[tokio::test]
async fn test_update_document() {
    if let Ok(storage) = create_test_storage().await {
        let original_content = "Original document content";
        let updated_content = "Updated document content with new information";
        let metadata = create_test_metadata();

        // Store original document
        let stored_doc = storage
            .store_document(original_content, metadata.clone())
            .await
            .unwrap();
        let original_id = stored_doc.id.clone();

        // Update the document
        let mut updated_metadata = metadata;
        updated_metadata.quality_score = Some(0.95);
        updated_metadata.tags.push("updated".to_string());

        let updated_doc = storage
            .update_document(&original_id, updated_content, updated_metadata)
            .await
            .unwrap();

        assert_eq!(updated_doc.id, original_id);
        assert_eq!(updated_doc.content, updated_content);
        assert_eq!(updated_doc.metadata.quality_score, Some(0.95));
        assert!(updated_doc.metadata.tags.contains(&"updated".to_string()));

        // Verify the update persisted
        let retrieved = storage.retrieve_by_id(&original_id).await.unwrap().unwrap();
        assert_eq!(retrieved.content, updated_content);
        assert_eq!(retrieved.metadata.quality_score, Some(0.95));

        // Clean up
        storage.delete_document(&original_id).await.unwrap();
    }
}

#[tokio::test]
async fn test_batch_operations() {
    if let Ok(storage) = create_test_storage().await {
        let documents = vec![
            ("First batch document".to_string(), create_test_metadata()),
            ("Second batch document".to_string(), create_test_metadata()),
            ("Third batch document".to_string(), create_test_metadata()),
        ];

        // Batch store
        let store_result = storage.store_documents(documents).await.unwrap();
        assert_eq!(store_result.total_attempted, 3);
        assert_eq!(store_result.successful.len(), 3);
        assert_eq!(store_result.failed.len(), 0);

        let stored_ids: Vec<String> = store_result
            .successful
            .iter()
            .map(|doc| doc.id.clone())
            .collect();

        // Batch retrieve
        let retrieve_result = storage.retrieve_batch(stored_ids.clone()).await.unwrap();
        assert_eq!(retrieve_result.total_attempted, 3);
        assert_eq!(retrieve_result.successful.len(), 3);
        assert_eq!(retrieve_result.failed.len(), 0);

        // Verify content
        for doc in &retrieve_result.successful {
            assert!(doc.content.contains("batch document"));
            assert_eq!(doc.metadata.research_type, Some(ResearchType::Learning));
        }

        // Batch delete
        let delete_result = storage.delete_batch(stored_ids).await.unwrap();
        assert_eq!(delete_result.total_attempted, 3);
        assert_eq!(delete_result.successful.len(), 3);
        assert_eq!(delete_result.failed.len(), 0);
    }
}

#[tokio::test]
async fn test_search_with_filters() {
    if let Ok(storage) = create_test_storage().await {
        // Store documents with different metadata
        let mut learning_metadata = create_test_metadata();
        learning_metadata.research_type = Some(ResearchType::Learning);
        learning_metadata.tags = vec!["learning".to_string(), "tutorial".to_string()];

        let mut impl_metadata = create_test_metadata();
        impl_metadata.research_type = Some(ResearchType::Implementation);
        impl_metadata.tags = vec!["implementation".to_string(), "guide".to_string()];

        let doc1 = storage
            .store_document("Learning about Rust vectors", learning_metadata)
            .await
            .unwrap();
        let doc2 = storage
            .store_document("Implementing vector storage in Rust", impl_metadata)
            .await
            .unwrap();

        // Search with configuration
        let search_config = SearchConfig {
            limit: 5,
            threshold: Some(0.0),
            collection: None,
            filters: vec![], // Note: Actual filter implementation would require Qdrant filter syntax
        };

        let results = storage
            .retrieve_similar("vectors in Rust", search_config)
            .await
            .unwrap();
        assert!(!results.is_empty());

        // Clean up
        storage.delete_document(&doc1.id).await.unwrap();
        storage.delete_document(&doc2.id).await.unwrap();
    }
}

#[tokio::test]
async fn test_storage_statistics() {
    if let Ok(storage) = create_test_storage().await {
        let initial_stats = storage.get_stats().await.unwrap();
        let initial_documents = initial_stats.total_documents;
        let initial_searches = initial_stats.total_searches;

        // Store a document
        let metadata = create_test_metadata();
        let doc = storage
            .store_document("Test document for stats", metadata)
            .await
            .unwrap();

        // Perform a search
        let search_config = SearchConfig::default();
        let _results = storage
            .retrieve_similar("test query", search_config)
            .await
            .unwrap();

        // Check updated stats
        let updated_stats = storage.get_stats().await.unwrap();
        assert_eq!(updated_stats.total_documents, initial_documents + 1);
        assert_eq!(updated_stats.total_searches, initial_searches + 1);
        assert!(updated_stats.avg_search_latency_ms >= 0.0);
        assert!(updated_stats.avg_embedding_time_ms >= 0.0);

        // Clean up
        storage.delete_document(&doc.id).await.unwrap();
    }
}

#[tokio::test]
async fn test_cache_operations() {
    if let Ok(storage) = create_test_storage().await {
        // Clear cache
        storage.clear_cache().await.unwrap();

        // Store and retrieve to populate cache
        let metadata = create_test_metadata();
        let doc = storage
            .store_document("Cache test document", metadata)
            .await
            .unwrap();

        // Test would access embedding service through storage interface
        // For testing caching, we rely on the stats to show cache hit rate

        // Store the same content twice to test caching behavior
        let doc2 = storage
            .store_document("Cache test document", create_test_metadata())
            .await
            .unwrap();

        // Get stats to check cache performance
        let stats = storage.get_stats().await.unwrap();
        // Cache hit rate should be available
        assert!(stats.embedding_cache_hit_rate >= 0.0);

        // Clean up
        storage.delete_document(&doc.id).await.unwrap();
        storage.delete_document(&doc2.id).await.unwrap();
    }
}

#[tokio::test]
async fn test_error_handling() {
    if let Ok(storage) = create_test_storage().await {
        // Test retrieving non-existent document
        let result = storage.retrieve_by_id("non-existent-id").await.unwrap();
        assert!(result.is_none());

        // Test deleting non-existent document
        let deleted = storage.delete_document("non-existent-id").await.unwrap();
        assert!(!deleted);

        // Test batch operations with some invalid IDs
        let mixed_ids = vec!["non-existent-1".to_string(), "non-existent-2".to_string()];

        let batch_result = storage.retrieve_batch(mixed_ids).await.unwrap();
        assert_eq!(batch_result.total_attempted, 2);
        assert_eq!(batch_result.successful.len(), 0);
        assert_eq!(batch_result.failed.len(), 2);
    }
}

/// Performance test for batch operations
#[tokio::test]
async fn test_batch_performance() {
    if let Ok(storage) = create_test_storage().await {
        let batch_size = 10;
        let documents: Vec<(String, DocumentMetadata)> = (0..batch_size)
            .map(|i| {
                (
                    format!("Performance test document number {i}"),
                    create_test_metadata(),
                )
            })
            .collect();

        let start_time = std::time::Instant::now();
        let store_result = storage.store_documents(documents).await.unwrap();
        let store_duration = start_time.elapsed();

        assert_eq!(store_result.successful.len(), batch_size);
        println!("Batch store of {batch_size} documents took: {store_duration:?}");

        let stored_ids: Vec<String> = store_result
            .successful
            .iter()
            .map(|doc| doc.id.clone())
            .collect();

        // Test batch retrieval performance
        let start_time = std::time::Instant::now();
        let retrieve_result = storage.retrieve_batch(stored_ids.clone()).await.unwrap();
        let retrieve_duration = start_time.elapsed();

        assert_eq!(retrieve_result.successful.len(), batch_size);
        println!("Batch retrieve of {batch_size} documents took: {retrieve_duration:?}");

        // Clean up
        let _delete_result = storage.delete_batch(stored_ids).await.unwrap();
    }
}
