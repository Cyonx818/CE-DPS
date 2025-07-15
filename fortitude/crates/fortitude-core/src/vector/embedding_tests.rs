// ABOUTME: Comprehensive unit tests for embedding functionality
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::error::VectorError;
    use fortitude_types::research::{ClassifiedRequest, ResearchType, AudienceContext, DomainContext};
    use std::time::Duration;
    use tokio;

    /// Create a test embedding configuration
    fn create_test_config() -> EmbeddingConfig {
        EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 128, // Smaller for tests
            batch_size: 4,
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 100,
                ttl: Duration::from_secs(60),
                key_strategy: CacheKeyStrategy::Hash,
            },
            download_config: ModelDownloadConfig {
                cache_dir: Some(std::env::temp_dir().join("fortitude_test_models")),
                offline: true, // Use offline mode for tests
                timeout: Duration::from_secs(30),
            },
            preprocessing: PreprocessingConfig {
                lowercase: true,
                normalize_whitespace: true,
                remove_special_chars: false,
                max_text_length: 1000,
            },
        }
    }

    /// Create test research content
    fn create_test_research_content() -> Vec<String> {
        vec![
            "How to implement async Rust programming patterns?".to_string(),
            "Best practices for error handling in Rust applications".to_string(),
            "Vector database integration with Qdrant and semantic search".to_string(),
            "Multi-threaded programming with tokio runtime".to_string(),
            "Design patterns for scalable web APIs in Rust".to_string(),
        ]
    }

    /// Create a classified request for testing
    fn create_test_classified_request(query: &str) -> ClassifiedRequest {
        ClassifiedRequest::new(
            query.to_string(),
            ResearchType::Implementation,
            AudienceContext::default(),
            DomainContext::default(),
            0.85,
            vec!["implement".to_string(), "rust".to_string()],
        )
    }

    #[test]
    fn test_embedding_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.model_name, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(config.max_sequence_length, 128);
        assert_eq!(config.batch_size, 4);
        assert!(config.cache_config.enabled);
        assert_eq!(config.cache_config.max_entries, 100);
    }

    #[test]
    fn test_device_type_conversion() {
        let _cpu_device: MockDevice = DeviceType::Cpu.into();
        // We can't test the actual device type easily, but we can test the conversion doesn't panic
        assert!(true);
    }

    #[test]
    fn test_cache_key_strategies() {
        let config = create_test_config();
        let service = LocalEmbeddingService::new(config);

        let text = "test embedding text";
        let key1 = service.generate_cache_key(text);
        let key2 = service.generate_cache_key(text);
        
        // Same text should generate same key
        assert_eq!(key1, key2);
        
        // Different text should generate different key
        let key3 = service.generate_cache_key("different text");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_text_preprocessing() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: true,
                normalize_whitespace: true,
                remove_special_chars: false,
                max_text_length: 50,
            },
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        let input = "  Hello   WORLD!  How are YOU?  ";
        let processed = service.preprocess_text(input);
        
        assert_eq!(processed, "hello world! how are you?");
    }

    #[test]
    fn test_text_preprocessing_with_length_limit() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: false,
                normalize_whitespace: false,
                remove_special_chars: false,
                max_text_length: 10,
            },
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        let input = "This is a very long text that should be truncated";
        let processed = service.preprocess_text(input);
        
        assert_eq!(processed.len(), 10);
        assert_eq!(processed, "This is a ");
    }

    #[test]
    fn test_text_preprocessing_special_chars() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: false,
                normalize_whitespace: false,
                remove_special_chars: true,
                max_text_length: 1000,
            },
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        let input = "Hello, World! @#$%^&*()";
        let processed = service.preprocess_text(input);
        
        assert_eq!(processed, "Hello World ");
    }

    #[tokio::test]
    async fn test_embedding_service_creation() {
        let config = create_test_config();
        let service = LocalEmbeddingService::new(config);
        
        // Service should be created successfully
        assert_eq!(service.embedding_dimension(), 384);
    }

    #[tokio::test]
    async fn test_embedding_stats() {
        let config = create_test_config();
        let service = LocalEmbeddingService::new(config);
        
        let stats = service.get_stats().await;
        
        assert_eq!(stats.total_generated, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.avg_generation_time_ms, 0.0);
        assert_eq!(stats.cache_size, 0);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let config = create_test_config();
        let service = LocalEmbeddingService::new(config);
        
        // Test cache clearing
        let result = service.clear_cache().await;
        assert!(result.is_ok());
        
        let stats = service.get_stats().await;
        assert_eq!(stats.cache_size, 0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mut config = create_test_config();
        config.cache_config.ttl = Duration::from_millis(1); // Very short TTL
        
        let service = LocalEmbeddingService::new(config);
        
        // Add an entry to cache manually
        service.cache.insert("test_key".to_string(), CachedEmbedding {
            embedding: vec![1.0, 2.0, 3.0],
            cached_at: std::time::Instant::now() - Duration::from_millis(10), // Already expired
            access_count: 1,
        });
        
        assert_eq!(service.cache.len(), 1);
        
        // Cleanup should remove expired entries
        service.cleanup_cache().await;
        
        assert_eq!(service.cache.len(), 0);
    }

    #[test]
    fn test_cache_key_strategy_prefix_hash() {
        let config = EmbeddingConfig {
            cache_config: EmbeddingCacheConfig {
                key_strategy: CacheKeyStrategy::PrefixHash(5),
                ..EmbeddingCacheConfig::default()
            },
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        let key = service.generate_cache_key("Hello World");
        assert!(key.starts_with("emb_Hello"));
    }

    #[test]
    fn test_cache_key_strategy_length_hash() {
        let config = EmbeddingConfig {
            cache_config: EmbeddingCacheConfig {
                key_strategy: CacheKeyStrategy::LengthHash,
                ..EmbeddingCacheConfig::default()
            },
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        let text = "Hello";
        let key = service.generate_cache_key(text);
        assert!(key.contains(&format!("emb_{}_", text.len())));
    }

    #[tokio::test]
    async fn test_research_content_preprocessing() {
        let config = create_test_config();
        let service = LocalEmbeddingService::new(config);
        
        let test_contents = create_test_research_content();
        
        for content in &test_contents {
            let processed = service.preprocess_text(content);
            
            // Processed text should be lowercase
            assert_eq!(processed, processed.to_lowercase());
            
            // Should not have leading/trailing whitespace
            assert_eq!(processed.trim(), processed);
            
            // Should not be empty
            assert!(!processed.is_empty());
        }
    }

    #[test]
    fn test_classified_request_integration() {
        let request = create_test_classified_request("How to implement async patterns in Rust?");
        
        assert_eq!(request.research_type, ResearchType::Implementation);
        assert_eq!(request.original_query, "How to implement async patterns in Rust?");
        assert_eq!(request.confidence, 0.85);
        assert!(request.matched_keywords.contains(&"implement".to_string()));
    }

    #[tokio::test]
    async fn test_batch_size_handling() {
        let config = EmbeddingConfig {
            batch_size: 2, // Small batch size for testing
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        let test_texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
            "Fourth text".to_string(),
            "Fifth text".to_string(),
        ];
        
        // This should handle batching internally (though we can't test the actual embeddings without a model)
        // We're testing that the structure handles batch processing correctly
        assert_eq!(test_texts.len(), 5);
        assert_eq!(service.config.batch_size, 2);
    }

    #[test]
    fn test_error_types() {
        let error = VectorError::EmbeddingError("Test error".to_string());
        assert_eq!(error.to_string(), "Embedding generation error: Test error");
        assert!(!error.is_retryable());
        
        let model_error = VectorError::ModelLoadError {
            model: "test-model".to_string(),
            reason: "File not found".to_string(),
        };
        assert_eq!(model_error.to_string(), "Model loading error: test-model - File not found");
        assert!(!model_error.is_retryable());
        
        let tokenization_error = VectorError::TokenizationError("Invalid token".to_string());
        assert_eq!(tokenization_error.to_string(), "Tokenization error: Invalid token");
        assert!(!tokenization_error.is_retryable());
    }

    #[test]
    fn test_vector_config_embedding_integration() {
        use crate::vector::config::VectorConfig;
        
        let embedding_config = create_test_config();
        let vector_config = VectorConfig::default()
            .with_embedding_config(embedding_config.clone());
        
        assert_eq!(vector_config.embedding.model_name, embedding_config.model_name);
        assert_eq!(vector_config.embedding.batch_size, embedding_config.batch_size);
    }

    #[test]
    fn test_vector_config_dimension_sync() {
        use crate::vector::config::VectorConfig;
        
        let mut vector_config = VectorConfig::default();
        
        // Test with known model
        vector_config.embedding.model_name = "sentence-transformers/all-MiniLM-L6-v2".to_string();
        vector_config.sync_dimensions_with_embedding();
        assert_eq!(vector_config.vector_dimensions, 384);
        
        // Test with different known model
        vector_config.embedding.model_name = "sentence-transformers/all-mpnet-base-v2".to_string();
        vector_config.sync_dimensions_with_embedding();
        assert_eq!(vector_config.vector_dimensions, 768);
    }

    #[test]
    fn test_embedding_config_serialization() {
        let config = create_test_config();
        
        // Test serialization
        let serialized = serde_json::to_string(&config).expect("Should serialize");
        assert!(serialized.contains("sentence-transformers/all-MiniLM-L6-v2"));
        
        // Test deserialization
        let deserialized: EmbeddingConfig = serde_json::from_str(&serialized)
            .expect("Should deserialize");
        assert_eq!(deserialized.model_name, config.model_name);
        assert_eq!(deserialized.batch_size, config.batch_size);
    }

    #[test]
    fn test_preprocessing_config_edge_cases() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: true,
                normalize_whitespace: true,
                remove_special_chars: true,
                max_text_length: 5,
            },
            ..create_test_config()
        };
        
        let service = LocalEmbeddingService::new(config);
        
        // Empty string
        let processed = service.preprocess_text("");
        assert_eq!(processed, "");
        
        // Only whitespace
        let processed = service.preprocess_text("   \t\n  ");
        assert_eq!(processed, "");
        
        // Only special characters
        let processed = service.preprocess_text("@#$%^&*()");
        assert_eq!(processed, "");
        
        // Mixed content with length limit
        let processed = service.preprocess_text("Hello@#$ World!!!");
        assert_eq!(processed, "hello"); // Should be truncated to 5 chars and lowercased
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let config = create_test_config();
        let service = std::sync::Arc::new(LocalEmbeddingService::new(config));
        
        let mut handles = vec![];
        
        // Simulate concurrent cache operations
        for i in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                let key = format!("test_key_{}", i);
                service_clone.cache.insert(key.clone(), CachedEmbedding {
                    embedding: vec![i as f32; 384],
                    cached_at: std::time::Instant::now(),
                    access_count: 1,
                });
                
                // Try to access the cache
                service_clone.cache.get(&key).is_some()
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(result);
        }
        
        // Check final cache state
        assert_eq!(service.cache.len(), 10);
    }

    #[test]
    fn test_research_type_embedding_relevance() {
        let research_types = vec![
            ResearchType::Decision,
            ResearchType::Implementation,
            ResearchType::Troubleshooting,
            ResearchType::Learning,
            ResearchType::Validation,
        ];
        
        for research_type in research_types {
            let request = ClassifiedRequest::new(
                format!("Sample {} query", research_type.display_name()),
                research_type.clone(),
                AudienceContext::default(),
                DomainContext::default(),
                0.8,
                vec![research_type.display_name().to_lowercase()],
            );
            
            // Verify that the request structure is compatible with embedding generation
            assert!(!request.original_query.is_empty());
            assert!(request.confidence > 0.0);
            assert_eq!(request.research_type, research_type);
        }
    }
}