// ABOUTME: Integration test for embedding generation functionality
//! This test demonstrates the complete embedding generation pipeline
//! from text input to vector output, including caching and batch processing.

use fortitude_core::vector::{
    CacheKeyStrategy, DeviceType, EmbeddingCacheConfig, EmbeddingConfig, EmbeddingGenerator,
    LocalEmbeddingService,
};
use fortitude_types::research::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};
use std::time::Duration;

#[tokio::test]
async fn test_embedding_generation_pipeline() {
    // Create a test configuration
    let config = EmbeddingConfig {
        model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        max_sequence_length: 128,
        batch_size: 4,
        device: DeviceType::Cpu,
        cache_config: EmbeddingCacheConfig {
            enabled: true,
            max_entries: 100,
            ttl: Duration::from_secs(60),
            key_strategy: CacheKeyStrategy::Hash,
        },
        ..Default::default()
    };

    // Create the embedding service
    let service = LocalEmbeddingService::new(config);

    // Initialize the service (loads model and tokenizer)
    service
        .initialize()
        .await
        .expect("Service should initialize");

    // Test single embedding generation
    let text = "How to implement async Rust programming patterns?";
    let embedding = service
        .generate_embedding(text)
        .await
        .expect("Should generate embedding");

    // Verify embedding properties
    assert_eq!(embedding.len(), 384); // Standard dimension for all-MiniLM-L6-v2
    assert!(embedding.iter().all(|&x| x.is_finite())); // All values should be finite

    // Verify embedding is normalized (common for sentence transformers)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01); // Should be approximately unit length

    // Test caching - generate the same embedding again
    let embedding_cached = service
        .generate_embedding(text)
        .await
        .expect("Should generate cached embedding");

    // Should be identical due to caching
    assert_eq!(embedding, embedding_cached);

    // Test batch generation
    let texts = vec![
        "Vector database integration with Qdrant".to_string(),
        "Multi-threaded programming with tokio runtime".to_string(),
        "Design patterns for scalable web APIs".to_string(),
    ];

    let batch_embeddings = service
        .generate_embeddings(&texts)
        .await
        .expect("Should generate batch embeddings");

    assert_eq!(batch_embeddings.len(), 3);
    for embedding in &batch_embeddings {
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().all(|&x| x.is_finite()));
    }

    // Different texts should produce different embeddings
    assert_ne!(batch_embeddings[0], batch_embeddings[1]);
    assert_ne!(batch_embeddings[1], batch_embeddings[2]);

    // Test statistics
    let stats = service.get_stats().await;
    assert!(stats.total_generated >= 4); // At least 4 embeddings generated
    assert!(stats.avg_generation_time_ms > 0.0);

    // Test with research content types
    let research_request = ClassifiedRequest::new(
        "How to optimize vector database queries?".to_string(),
        ResearchType::Implementation,
        AudienceContext::default(),
        DomainContext::default(),
        0.85,
        vec!["optimize".to_string(), "vector".to_string()],
    );

    let research_embedding = service
        .generate_embedding(&research_request.original_query)
        .await
        .expect("Should generate research embedding");

    assert_eq!(research_embedding.len(), 384);
    assert!(research_embedding.iter().all(|&x| x.is_finite()));
}

#[tokio::test]
async fn test_embedding_determinism() {
    // Test that the same text always produces the same embedding
    let config = EmbeddingConfig::default();
    let service = LocalEmbeddingService::new(config);
    service
        .initialize()
        .await
        .expect("Service should initialize");

    let text = "Test text for determinism";

    let embedding1 = service.generate_embedding(text).await.unwrap();
    let embedding2 = service.generate_embedding(text).await.unwrap();

    // Since we're using caching, this should be identical
    assert_eq!(embedding1, embedding2);

    // Test without cache by clearing it
    service.clear_cache().await.unwrap();
    let embedding3 = service.generate_embedding(text).await.unwrap();

    // Even without cache, should be deterministic (due to our mock implementation)
    assert_eq!(embedding1, embedding3);
}

#[tokio::test]
async fn test_embedding_text_preprocessing() {
    let config = EmbeddingConfig::default();
    let service = LocalEmbeddingService::new(config);
    service
        .initialize()
        .await
        .expect("Service should initialize");

    // Test that preprocessing is applied consistently
    let original_text = "  How to IMPLEMENT   Async Patterns?  ";
    let normalized_text = "how to implement async patterns?";

    let embedding1 = service.generate_embedding(original_text).await.unwrap();
    let embedding2 = service.generate_embedding(normalized_text).await.unwrap();

    // Should be identical due to preprocessing normalization
    assert_eq!(embedding1, embedding2);
}

#[tokio::test]
async fn test_embedding_service_configuration() {
    // Test different configuration options
    let mut config = EmbeddingConfig::default();
    config.cache_config.enabled = false; // Disable caching

    let service = LocalEmbeddingService::new(config);
    service
        .initialize()
        .await
        .expect("Service should initialize");

    let text = "Test without caching";
    let embedding = service.generate_embedding(text).await.unwrap();

    assert_eq!(embedding.len(), 384);

    // Since caching is disabled, cache should remain empty
    let stats = service.get_stats().await;
    assert_eq!(stats.cache_size, 0);
}

#[tokio::test]
async fn test_embedding_error_handling() {
    let config = EmbeddingConfig::default();
    let service = LocalEmbeddingService::new(config);

    // Try to generate embedding without initialization
    let result = service.generate_embedding("test").await;
    assert!(result.is_err());

    // The error should indicate that the model is not initialized
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not initialized"));
}
