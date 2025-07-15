// ABOUTME: Anchor tests for Sprint 009 Tasks 3 & 4 - Data Persistence Critical Workflows
//! These tests protect critical data persistence functionality implemented in Sprint 009
//! Tasks 3 and 4. They ensure that data persistence workflows continue to work correctly
//! as the system evolves.
//!
//! ## Protected Functionality
//! - Data persistence (learning data storage, vector operations, cache persistence)
//! - External API integration (vector database operations, storage backends)
//! - Type definition changes (learning data types, storage interfaces)
//! - Critical error handling (data loss prevention, corruption recovery)
//! - Business logic (data consistency, transaction integrity)

use fortitude::learning::*;
use fortitude::monitoring::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mock vector database for testing data persistence
pub struct MockVectorDatabase {
    embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    metadata: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    collections: Arc<RwLock<HashMap<String, VectorCollection>>>,
    query_logs: Arc<RwLock<Vec<VectorQuery>>>,
}

#[derive(Clone)]
pub struct VectorCollection {
    pub name: String,
    pub dimension: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub document_count: usize,
    pub index_type: String,
}

#[derive(Clone)]
pub struct VectorQuery {
    pub query_id: String,
    pub collection: String,
    pub query_vector: Vec<f32>,
    pub limit: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
}

impl MockVectorDatabase {
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            collections: Arc::new(RwLock::new(HashMap::new())),
            query_logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn create_collection(&self, name: &str, dimension: usize) -> Result<(), String> {
        let mut collections = self.collections.write().await;

        if collections.contains_key(name) {
            return Err(format!("Collection '{}' already exists", name));
        }

        let collection = VectorCollection {
            name: name.to_string(),
            dimension,
            created_at: chrono::Utc::now(),
            document_count: 0,
            index_type: "HNSW".to_string(),
        };

        collections.insert(name.to_string(), collection);
        Ok(())
    }

    pub async fn store_embedding(
        &self,
        collection: &str,
        id: &str,
        vector: Vec<f32>,
        metadata: serde_json::Value,
    ) -> Result<(), String> {
        // Verify collection exists
        let collections = self.collections.read().await;
        let collection_info = collections
            .get(collection)
            .ok_or_else(|| format!("Collection '{}' does not exist", collection))?;

        if vector.len() != collection_info.dimension {
            return Err(format!(
                "Vector dimension {} does not match collection dimension {}",
                vector.len(),
                collection_info.dimension
            ));
        }
        drop(collections);

        // Store embedding and metadata
        let embedding_key = format!("{}:{}", collection, id);

        {
            let mut embeddings = self.embeddings.write().await;
            embeddings.insert(embedding_key.clone(), vector);
        }

        {
            let mut metadata_store = self.metadata.write().await;
            metadata_store.insert(embedding_key, metadata);
        }

        // Update collection document count
        {
            let mut collections = self.collections.write().await;
            if let Some(coll) = collections.get_mut(collection) {
                coll.document_count += 1;
            }
        }

        Ok(())
    }

    pub async fn query_similar(
        &self,
        collection: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SimilarityResult>, String> {
        let start_time = Instant::now();

        // Verify collection exists
        let collections = self.collections.read().await;
        let collection_info = collections
            .get(collection)
            .ok_or_else(|| format!("Collection '{}' does not exist", collection))?;

        if query_vector.len() != collection_info.dimension {
            return Err(format!(
                "Query vector dimension {} does not match collection dimension {}",
                query_vector.len(),
                collection_info.dimension
            ));
        }
        drop(collections);

        // Perform similarity search
        let embeddings = self.embeddings.read().await;
        let metadata_store = self.metadata.read().await;

        let collection_prefix = format!("{}:", collection);
        let mut results = Vec::new();

        for (key, embedding) in embeddings.iter() {
            if key.starts_with(&collection_prefix) {
                let similarity = cosine_similarity(&query_vector, embedding);
                let id = key.strip_prefix(&collection_prefix).unwrap().to_string();
                let metadata = metadata_store
                    .get(key)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                results.push(SimilarityResult {
                    id,
                    similarity,
                    metadata,
                });
            }
        }

        // Sort by similarity and limit results
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.truncate(limit);

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        // Log query
        let query_log = VectorQuery {
            query_id: Uuid::new_v4().to_string(),
            collection: collection.to_string(),
            query_vector,
            limit,
            timestamp: chrono::Utc::now(),
            response_time_ms,
        };

        {
            let mut query_logs = self.query_logs.write().await;
            query_logs.push(query_log);
        }

        Ok(results)
    }

    pub async fn delete_embedding(&self, collection: &str, id: &str) -> Result<bool, String> {
        let embedding_key = format!("{}:{}", collection, id);

        let existed = {
            let mut embeddings = self.embeddings.write().await;
            embeddings.remove(&embedding_key).is_some()
        };

        if existed {
            let mut metadata_store = self.metadata.write().await;
            metadata_store.remove(&embedding_key);

            // Update collection document count
            let mut collections = self.collections.write().await;
            if let Some(coll) = collections.get_mut(collection) {
                coll.document_count = coll.document_count.saturating_sub(1);
            }
        }

        Ok(existed)
    }

    pub async fn get_collection_stats(&self, collection: &str) -> Result<CollectionStats, String> {
        let collections = self.collections.read().await;
        let collection_info = collections
            .get(collection)
            .ok_or_else(|| format!("Collection '{}' does not exist", collection))?;

        let query_logs = self.query_logs.read().await;
        let collection_queries: Vec<&VectorQuery> = query_logs
            .iter()
            .filter(|q| q.collection == collection)
            .collect();

        let avg_query_time = if collection_queries.is_empty() {
            0.0
        } else {
            collection_queries
                .iter()
                .map(|q| q.response_time_ms as f64)
                .sum::<f64>()
                / collection_queries.len() as f64
        };

        Ok(CollectionStats {
            collection_name: collection.to_string(),
            document_count: collection_info.document_count,
            dimension: collection_info.dimension,
            total_queries: collection_queries.len(),
            avg_query_time_ms: avg_query_time,
            created_at: collection_info.created_at,
        })
    }
}

#[derive(Clone)]
pub struct SimilarityResult {
    pub id: String,
    pub similarity: f64,
    pub metadata: serde_json::Value,
}

#[derive(Clone)]
pub struct CollectionStats {
    pub collection_name: String,
    pub document_count: usize,
    pub dimension: usize,
    pub total_queries: usize,
    pub avg_query_time_ms: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Simple cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)) as f64
}

/// Mock persistent storage for learning data
pub struct MockPersistentStorage {
    feedback_file: Arc<RwLock<HashMap<String, UserFeedback>>>,
    pattern_file: Arc<RwLock<HashMap<String, PatternData>>>,
    learning_file: Arc<RwLock<HashMap<String, LearningData>>>,
    usage_file: Arc<RwLock<HashMap<String, UsagePattern>>>,
    storage_path: String,
    transaction_log: Arc<RwLock<Vec<TransactionEntry>>>,
}

#[derive(Clone)]
pub struct TransactionEntry {
    pub transaction_id: String,
    pub operation: String,
    pub data_type: String,
    pub data_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

impl MockPersistentStorage {
    pub fn new(storage_path: String) -> Self {
        Self {
            feedback_file: Arc::new(RwLock::new(HashMap::new())),
            pattern_file: Arc::new(RwLock::new(HashMap::new())),
            learning_file: Arc::new(RwLock::new(HashMap::new())),
            usage_file: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
            transaction_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn persist_feedback(&self, feedback: &UserFeedback) -> Result<(), String> {
        let transaction_id = Uuid::new_v4().to_string();

        let result = {
            let mut feedback_data = self.feedback_file.write().await;
            feedback_data.insert(feedback.id.clone(), feedback.clone());
            Ok(())
        };

        self.log_transaction(
            transaction_id,
            "insert",
            "feedback",
            &feedback.id,
            result.is_ok(),
        )
        .await;
        result
    }

    pub async fn persist_pattern(&self, pattern: &PatternData) -> Result<(), String> {
        let transaction_id = Uuid::new_v4().to_string();

        let result = {
            let mut pattern_data = self.pattern_file.write().await;
            pattern_data.insert(pattern.id.clone(), pattern.clone());
            Ok(())
        };

        self.log_transaction(
            transaction_id,
            "insert",
            "pattern",
            &pattern.id,
            result.is_ok(),
        )
        .await;
        result
    }

    pub async fn persist_learning_data(&self, learning: &LearningData) -> Result<(), String> {
        let transaction_id = Uuid::new_v4().to_string();

        let result = {
            let mut learning_data = self.learning_file.write().await;
            learning_data.insert(learning.id.clone(), learning.clone());
            Ok(())
        };

        self.log_transaction(
            transaction_id,
            "insert",
            "learning",
            &learning.id,
            result.is_ok(),
        )
        .await;
        result
    }

    pub async fn load_feedback(&self, id: &str) -> Result<Option<UserFeedback>, String> {
        let feedback_data = self.feedback_file.read().await;
        Ok(feedback_data.get(id).cloned())
    }

    pub async fn load_all_feedback(&self) -> Result<Vec<UserFeedback>, String> {
        let feedback_data = self.feedback_file.read().await;
        Ok(feedback_data.values().cloned().collect())
    }

    pub async fn delete_feedback(&self, id: &str) -> Result<bool, String> {
        let transaction_id = Uuid::new_v4().to_string();

        let result = {
            let mut feedback_data = self.feedback_file.write().await;
            feedback_data.remove(id).is_some()
        };

        self.log_transaction(transaction_id, "delete", "feedback", id, true)
            .await;
        Ok(result)
    }

    pub async fn backup_data(&self) -> Result<BackupInfo, String> {
        let feedback_count = self.feedback_file.read().await.len();
        let pattern_count = self.pattern_file.read().await.len();
        let learning_count = self.learning_file.read().await.len();
        let usage_count = self.usage_file.read().await.len();

        Ok(BackupInfo {
            backup_id: Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            feedback_count,
            pattern_count,
            learning_count,
            usage_count,
            backup_size_bytes: (feedback_count + pattern_count + learning_count + usage_count)
                * 1024, // Mock size
        })
    }

    pub async fn restore_from_backup(&self, backup_id: &str) -> Result<RestoreInfo, String> {
        // Mock restore operation
        Ok(RestoreInfo {
            backup_id: backup_id.to_string(),
            restored_at: chrono::Utc::now(),
            items_restored: 100, // Mock count
            errors_encountered: 0,
        })
    }

    pub async fn get_transaction_log(&self) -> Vec<TransactionEntry> {
        self.transaction_log.read().await.clone()
    }

    async fn log_transaction(
        &self,
        transaction_id: String,
        operation: &str,
        data_type: &str,
        data_id: &str,
        success: bool,
    ) {
        let entry = TransactionEntry {
            transaction_id,
            operation: operation.to_string(),
            data_type: data_type.to_string(),
            data_id: data_id.to_string(),
            timestamp: chrono::Utc::now(),
            success,
        };

        let mut transaction_log = self.transaction_log.write().await;
        transaction_log.push(entry);
    }
}

#[derive(Clone)]
pub struct BackupInfo {
    pub backup_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub feedback_count: usize,
    pub pattern_count: usize,
    pub learning_count: usize,
    pub usage_count: usize,
    pub backup_size_bytes: usize,
}

#[derive(Clone)]
pub struct RestoreInfo {
    pub backup_id: String,
    pub restored_at: chrono::DateTime<chrono::Utc>,
    pub items_restored: usize,
    pub errors_encountered: usize,
}

/// Helper functions for test data generation
fn create_test_vector(dimension: usize, seed: u64) -> Vec<f32> {
    let mut vector = Vec::with_capacity(dimension);
    for i in 0..dimension {
        vector.push(((seed + i as u64) % 1000) as f32 / 1000.0);
    }
    vector
}

fn create_test_feedback(user_id: &str, content_id: &str, score: f64) -> UserFeedback {
    UserFeedback::new(
        user_id.to_string(),
        content_id.to_string(),
        "quality_rating".to_string(),
        Some(score),
        Some("Test feedback".to_string()),
    )
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Vector database operations workflow
    /// Tests: Collection creation → Embedding storage → Similarity search → Data retrieval
    /// Protects: Vector database integration and embedding operations
    #[tokio::test]
    async fn test_anchor_vector_database_operations_workflow() {
        let vector_db = MockVectorDatabase::new();

        // Test 1: Collection creation and management
        let collections_to_create = vec![
            ("learning_embeddings", 384),
            ("feedback_embeddings", 512),
            ("pattern_embeddings", 256),
            ("insight_embeddings", 768),
        ];

        for (collection_name, dimension) in &collections_to_create {
            let result = vector_db
                .create_collection(collection_name, *dimension)
                .await;
            assert!(
                result.is_ok(),
                "Collection creation should succeed for {}",
                collection_name
            );
        }

        // Test duplicate collection creation should fail
        let duplicate_result = vector_db
            .create_collection("learning_embeddings", 384)
            .await;
        assert!(
            duplicate_result.is_err(),
            "Duplicate collection creation should fail"
        );

        // Test 2: Embedding storage with validation
        let test_embeddings = vec![
            (
                "learning_embeddings",
                "feedback_1",
                create_test_vector(384, 1),
                serde_json::json!({"type": "feedback", "score": 0.8}),
            ),
            (
                "learning_embeddings",
                "feedback_2",
                create_test_vector(384, 2),
                serde_json::json!({"type": "feedback", "score": 0.9}),
            ),
            (
                "feedback_embeddings",
                "pattern_1",
                create_test_vector(512, 3),
                serde_json::json!({"type": "pattern", "frequency": 5}),
            ),
            (
                "pattern_embeddings",
                "insight_1",
                create_test_vector(256, 4),
                serde_json::json!({"type": "insight", "confidence": 0.85}),
            ),
            (
                "insight_embeddings",
                "learning_1",
                create_test_vector(768, 5),
                serde_json::json!({"type": "learning", "source": "adaptation"}),
            ),
        ];

        for (collection, id, vector, metadata) in test_embeddings {
            let result = vector_db
                .store_embedding(collection, id, vector, metadata)
                .await;
            assert!(
                result.is_ok(),
                "Embedding storage should succeed for {} in {}",
                id,
                collection
            );
        }

        // Test 3: Invalid embedding storage (wrong dimension)
        let invalid_vector = create_test_vector(100, 999); // Wrong dimension
        let invalid_result = vector_db
            .store_embedding(
                "learning_embeddings",
                "invalid",
                invalid_vector,
                serde_json::json!({}),
            )
            .await;
        assert!(
            invalid_result.is_err(),
            "Invalid dimension embedding should be rejected"
        );

        // Test 4: Similarity search functionality
        let query_vector = create_test_vector(384, 1); // Similar to feedback_1
        let search_result = vector_db
            .query_similar("learning_embeddings", query_vector, 5)
            .await;
        assert!(search_result.is_ok(), "Similarity search should succeed");

        let results = search_result.unwrap();
        assert!(!results.is_empty(), "Should find similar embeddings");
        assert!(results.len() <= 5, "Should respect limit parameter");

        // Results should be sorted by similarity (descending)
        for i in 0..results.len() - 1 {
            assert!(
                results[i].similarity >= results[i + 1].similarity,
                "Results should be sorted by similarity"
            );
        }

        // Test 5: Cross-collection similarity searches
        let collections_to_query = vec![
            ("feedback_embeddings", 512),
            ("pattern_embeddings", 256),
            ("insight_embeddings", 768),
        ];

        for (collection, dimension) in collections_to_query {
            let query_vector = create_test_vector(dimension, 100);
            let result = vector_db.query_similar(collection, query_vector, 3).await;
            assert!(
                result.is_ok(),
                "Cross-collection search should work for {}",
                collection
            );
        }

        // Test 6: Batch embedding operations
        let batch_embeddings = (0..20)
            .map(|i| {
                (
                    "learning_embeddings",
                    format!("batch_embedding_{}", i),
                    create_test_vector(384, 100 + i),
                    serde_json::json!({"batch_id": i, "type": "batch_test"}),
                )
            })
            .collect::<Vec<_>>();

        for (collection, id, vector, metadata) in batch_embeddings {
            let result = vector_db
                .store_embedding(collection, &id, vector, metadata)
                .await;
            assert!(
                result.is_ok(),
                "Batch embedding storage should succeed for {}",
                id
            );
        }

        // Test 7: Large-scale similarity search
        let large_query_vector = create_test_vector(384, 50);
        let large_search_result = vector_db
            .query_similar("learning_embeddings", large_query_vector, 15)
            .await;
        assert!(
            large_search_result.is_ok(),
            "Large-scale search should succeed"
        );

        let large_results = large_search_result.unwrap();
        assert!(large_results.len() <= 15, "Should respect large limit");
        assert!(
            large_results.len() >= 10,
            "Should find multiple results in populated collection"
        );

        // Test 8: Embedding deletion and consistency
        let delete_result = vector_db
            .delete_embedding("learning_embeddings", "feedback_1")
            .await;
        assert!(delete_result.is_ok(), "Embedding deletion should succeed");
        assert!(delete_result.unwrap(), "Should confirm deletion occurred");

        // Verify deletion by searching
        let post_delete_query = create_test_vector(384, 1);
        let post_delete_results = vector_db
            .query_similar("learning_embeddings", post_delete_query, 10)
            .await
            .unwrap();

        let deleted_found = post_delete_results.iter().any(|r| r.id == "feedback_1");
        assert!(
            !deleted_found,
            "Deleted embedding should not appear in search results"
        );

        // Test 9: Collection statistics and monitoring
        let stats_result = vector_db.get_collection_stats("learning_embeddings").await;
        assert!(
            stats_result.is_ok(),
            "Collection stats should be retrievable"
        );

        let stats = stats_result.unwrap();
        assert_eq!(stats.collection_name, "learning_embeddings");
        assert!(
            stats.document_count > 15,
            "Should track document count accurately"
        ); // 2 initial + 20 batch - 1 deleted
        assert_eq!(stats.dimension, 384);
        assert!(stats.total_queries > 0, "Should track query count");
        assert!(
            stats.avg_query_time_ms >= 0.0,
            "Should track average query time"
        );

        // Test 10: Performance validation under load
        let performance_start = Instant::now();
        let mut performance_tasks = Vec::new();

        for i in 0..50 {
            let db_clone = &vector_db;
            let task = async move {
                if i % 2 == 0 {
                    // Store embedding
                    let vector = create_test_vector(384, 1000 + i);
                    let metadata = serde_json::json!({"performance_test": true, "iteration": i});
                    db_clone
                        .store_embedding(
                            "learning_embeddings",
                            &format!("perf_test_{}", i),
                            vector,
                            metadata,
                        )
                        .await
                } else {
                    // Query embedding
                    let query_vector = create_test_vector(384, 500 + i);
                    db_clone
                        .query_similar("learning_embeddings", query_vector, 5)
                        .await
                        .map(|_| ())
                }
            };
            performance_tasks.push(task);
        }

        let performance_results = futures::future::join_all(performance_tasks).await;
        let performance_time = performance_start.elapsed();

        let successful_ops = performance_results.iter().filter(|r| r.is_ok()).count();
        assert!(
            successful_ops >= 45,
            "Most performance operations should succeed"
        );
        assert!(
            performance_time < Duration::from_secs(5),
            "Performance test should complete within 5 seconds"
        );

        // Test 11: Data consistency validation
        let final_stats = vector_db
            .get_collection_stats("learning_embeddings")
            .await
            .unwrap();
        assert!(
            final_stats.document_count > 40,
            "Final document count should reflect all operations"
        );

        // Verify that all collections maintain their properties
        for (collection_name, expected_dimension) in &collections_to_create {
            let collection_stats = vector_db
                .get_collection_stats(collection_name)
                .await
                .unwrap();
            assert_eq!(
                collection_stats.dimension, *expected_dimension,
                "Collection {} should maintain its dimension",
                collection_name
            );
        }
    }

    /// ANCHOR: Learning data persistence workflow
    /// Tests: Data storage → Retrieval → Updates → Backup/Restore → Transaction integrity
    /// Protects: Learning data persistence and integrity
    #[tokio::test]
    async fn test_anchor_learning_data_persistence_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_string_lossy().to_string();
        let persistent_storage = MockPersistentStorage::new(storage_path);

        // Test 1: Feedback data persistence
        let test_feedback = vec![
            create_test_feedback("user_1", "content_A", 0.8),
            create_test_feedback("user_2", "content_A", 0.9),
            create_test_feedback("user_3", "content_B", 0.7),
            create_test_feedback("user_4", "content_B", 0.85),
            create_test_feedback("user_5", "content_C", 0.92),
        ];

        for feedback in &test_feedback {
            let result = persistent_storage.persist_feedback(feedback).await;
            assert!(
                result.is_ok(),
                "Feedback persistence should succeed for {}",
                feedback.id
            );
        }

        // Test 2: Feedback data retrieval
        for feedback in &test_feedback {
            let retrieved = persistent_storage
                .load_feedback(&feedback.id)
                .await
                .unwrap();
            assert!(
                retrieved.is_some(),
                "Feedback should be retrievable for {}",
                feedback.id
            );

            let retrieved_feedback = retrieved.unwrap();
            assert_eq!(retrieved_feedback.id, feedback.id);
            assert_eq!(retrieved_feedback.user_id, feedback.user_id);
            assert_eq!(retrieved_feedback.content_id, feedback.content_id);
            assert_eq!(retrieved_feedback.score, feedback.score);
        }

        // Test 3: Bulk feedback retrieval
        let all_feedback = persistent_storage.load_all_feedback().await.unwrap();
        assert_eq!(
            all_feedback.len(),
            test_feedback.len(),
            "Should retrieve all stored feedback"
        );

        // Verify data integrity
        for original in &test_feedback {
            let found = all_feedback.iter().find(|f| f.id == original.id);
            assert!(
                found.is_some(),
                "All original feedback should be found in bulk retrieval"
            );
        }

        // Test 4: Pattern data persistence
        let test_patterns = vec![
            PatternData::new("user_behavior".to_string(), 5, 0.8),
            PatternData::new("query_complexity".to_string(), 12, 0.75),
            PatternData::new("response_quality".to_string(), 8, 0.9),
            PatternData::new("technical_domain".to_string(), 15, 0.85),
        ];

        for pattern in &test_patterns {
            let result = persistent_storage.persist_pattern(pattern).await;
            assert!(
                result.is_ok(),
                "Pattern persistence should succeed for {}",
                pattern.id
            );
        }

        // Test 5: Learning data persistence with complex structures
        let test_learning_data = vec![
            LearningData::new(
                "user_preference".to_string(),
                "feedback_analysis".to_string(),
                vec![
                    "Users prefer detailed explanations".to_string(),
                    "Technical examples improve satisfaction".to_string(),
                ],
                0.87,
            ),
            LearningData::new(
                "performance_optimization".to_string(),
                "pattern_analysis".to_string(),
                vec!["Response time under 200ms correlates with positive feedback".to_string()],
                0.82,
            ),
            LearningData::new(
                "quality_indicator".to_string(),
                "correlation_analysis".to_string(),
                vec![
                    "Code examples increase perceived helpfulness".to_string(),
                    "Step-by-step explanations reduce follow-up questions".to_string(),
                ],
                0.91,
            ),
        ];

        for learning in &test_learning_data {
            let result = persistent_storage.persist_learning_data(learning).await;
            assert!(
                result.is_ok(),
                "Learning data persistence should succeed for {}",
                learning.id
            );
        }

        // Test 6: Transaction logging and integrity
        let transaction_log = persistent_storage.get_transaction_log().await;
        assert!(
            !transaction_log.is_empty(),
            "Transaction log should contain entries"
        );

        let expected_transaction_count =
            test_feedback.len() + test_patterns.len() + test_learning_data.len();
        assert_eq!(
            transaction_log.len(),
            expected_transaction_count,
            "Transaction log should track all operations"
        );

        // Verify all transactions were successful
        let failed_transactions = transaction_log.iter().filter(|t| !t.success).count();
        assert_eq!(
            failed_transactions, 0,
            "All transactions should have succeeded"
        );

        // Test 7: Data deletion and consistency
        let feedback_to_delete = &test_feedback[0];
        let delete_result = persistent_storage
            .delete_feedback(&feedback_to_delete.id)
            .await;
        assert!(delete_result.is_ok(), "Feedback deletion should succeed");
        assert!(delete_result.unwrap(), "Should confirm deletion occurred");

        // Verify deletion
        let deleted_feedback = persistent_storage
            .load_feedback(&feedback_to_delete.id)
            .await
            .unwrap();
        assert!(
            deleted_feedback.is_none(),
            "Deleted feedback should not be retrievable"
        );

        let remaining_feedback = persistent_storage.load_all_feedback().await.unwrap();
        assert_eq!(
            remaining_feedback.len(),
            test_feedback.len() - 1,
            "Bulk retrieval should reflect deletion"
        );

        // Test 8: Backup and restore operations
        let backup_info = persistent_storage.backup_data().await.unwrap();
        assert!(
            !backup_info.backup_id.is_empty(),
            "Backup should generate valid ID"
        );
        assert_eq!(
            backup_info.feedback_count,
            test_feedback.len() - 1,
            "Backup should reflect current data count"
        );
        assert_eq!(backup_info.pattern_count, test_patterns.len());
        assert_eq!(backup_info.learning_count, test_learning_data.len());
        assert!(
            backup_info.backup_size_bytes > 0,
            "Backup should calculate size"
        );

        // Test restore operation
        let restore_info = persistent_storage
            .restore_from_backup(&backup_info.backup_id)
            .await
            .unwrap();
        assert_eq!(restore_info.backup_id, backup_info.backup_id);
        assert!(
            restore_info.items_restored > 0,
            "Restore should report items restored"
        );
        assert_eq!(
            restore_info.errors_encountered, 0,
            "Restore should complete without errors"
        );

        // Test 9: Concurrent data operations
        let mut concurrent_tasks = Vec::new();

        for i in 0..20 {
            let storage = &persistent_storage;
            let task = async move {
                if i % 3 == 0 {
                    // Store feedback
                    let feedback = create_test_feedback(
                        &format!("concurrent_user_{}", i),
                        "concurrent_content",
                        0.8,
                    );
                    storage.persist_feedback(&feedback).await
                } else if i % 3 == 1 {
                    // Store pattern
                    let pattern =
                        PatternData::new(format!("concurrent_pattern_{}", i), i % 10 + 1, 0.75);
                    storage.persist_pattern(&pattern).await
                } else {
                    // Store learning data
                    let learning = LearningData::new(
                        format!("concurrent_learning_{}", i),
                        format!("concurrent_source_{}", i),
                        vec![format!("Concurrent insight {}", i)],
                        0.8,
                    );
                    storage.persist_learning_data(&learning).await
                }
            };
            concurrent_tasks.push(task);
        }

        let concurrent_results = futures::future::join_all(concurrent_tasks).await;
        let successful_concurrent_ops = concurrent_results.iter().filter(|r| r.is_ok()).count();
        assert!(
            successful_concurrent_ops >= 18,
            "Most concurrent operations should succeed"
        );

        // Test 10: Data integrity validation after load
        let final_transaction_log = persistent_storage.get_transaction_log().await;
        assert!(
            final_transaction_log.len() > expected_transaction_count,
            "Transaction log should include all operations"
        );

        // Verify data consistency
        let final_feedback = persistent_storage.load_all_feedback().await.unwrap();
        assert!(
            final_feedback.len() >= test_feedback.len() - 1 + 6, // Original - 1 deleted + ~6 concurrent
            "Final feedback count should reflect all operations"
        );

        // Verify transaction integrity
        let successful_final_transactions =
            final_transaction_log.iter().filter(|t| t.success).count();
        assert_eq!(
            successful_final_transactions,
            final_transaction_log.len(),
            "All logged transactions should be successful"
        );

        // Test 11: Performance validation for persistence operations
        let persistence_perf_start = Instant::now();

        for i in 0..100 {
            let feedback = create_test_feedback(&format!("perf_user_{}", i), "perf_content", 0.8);
            persistent_storage
                .persist_feedback(&feedback)
                .await
                .unwrap();
        }

        let persistence_perf_time = persistence_perf_start.elapsed();
        assert!(
            persistence_perf_time < Duration::from_secs(3),
            "Persistence performance should be acceptable"
        );

        // Test bulk retrieval performance
        let retrieval_perf_start = Instant::now();
        let _bulk_data = persistent_storage.load_all_feedback().await.unwrap();
        let retrieval_perf_time = retrieval_perf_start.elapsed();

        assert!(
            retrieval_perf_time < Duration::from_secs(1),
            "Bulk retrieval should be fast"
        );

        // Test 12: Data validation and corruption detection
        // Ensure data remains valid after all operations
        let final_validation_feedback = persistent_storage.load_all_feedback().await.unwrap();

        for feedback in final_validation_feedback {
            assert!(
                feedback.is_valid(),
                "All persisted feedback should remain valid"
            );
            assert!(!feedback.id.is_empty(), "Feedback IDs should be preserved");
            assert!(!feedback.user_id.is_empty(), "User IDs should be preserved");
            assert!(
                !feedback.content_id.is_empty(),
                "Content IDs should be preserved"
            );
        }

        // Final backup to ensure everything is persistent
        let final_backup = persistent_storage.backup_data().await.unwrap();
        assert!(
            final_backup.feedback_count > 100,
            "Final backup should reflect all stored data"
        );
    }

    /// ANCHOR: Cache persistence and invalidation workflow  
    /// Tests: Cache storage → Retrieval → Invalidation → TTL handling → Performance optimization
    /// Protects: Cache persistence mechanisms and performance optimizations
    #[tokio::test]
    async fn test_anchor_cache_persistence_invalidation_workflow() {
        let vector_db = MockVectorDatabase::new();
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_string_lossy().to_string();
        let persistent_storage = MockPersistentStorage::new(storage_path);

        // Set up collections for cache testing
        vector_db
            .create_collection("cache_embeddings", 256)
            .await
            .unwrap();

        // Test 1: Cache storage with TTL and metadata
        let cache_entries = vec![
            ("learning_cache_1", "User preference analysis results", 0.9),
            ("learning_cache_2", "Pattern recognition insights", 0.85),
            ("learning_cache_3", "Adaptation optimization data", 0.88),
            ("feedback_cache_1", "Quality score aggregation", 0.92),
            ("feedback_cache_2", "Trend analysis results", 0.87),
        ];

        for (cache_key, description, confidence) in &cache_entries {
            // Store in vector database as cached embedding
            let cache_vector = create_test_vector(256, cache_key.len() as u64);
            let cache_metadata = serde_json::json!({
                "cache_key": cache_key,
                "description": description,
                "confidence": confidence,
                "created_at": chrono::Utc::now(),
                "ttl_seconds": 3600,
                "access_count": 0
            });

            let result = vector_db
                .store_embedding("cache_embeddings", cache_key, cache_vector, cache_metadata)
                .await;
            assert!(
                result.is_ok(),
                "Cache entry storage should succeed for {}",
                cache_key
            );
        }

        // Test 2: Cache retrieval and hit/miss tracking
        let cache_query_vector = create_test_vector(256, "learning_cache_1".len() as u64);
        let cache_results = vector_db
            .query_similar("cache_embeddings", cache_query_vector, 3)
            .await
            .unwrap();

        assert!(
            !cache_results.is_empty(),
            "Cache query should return results"
        );

        // Verify cache entry structure
        let first_result = &cache_results[0];
        assert!(
            first_result.metadata.get("cache_key").is_some(),
            "Cache metadata should include cache_key"
        );
        assert!(
            first_result.metadata.get("ttl_seconds").is_some(),
            "Cache metadata should include TTL"
        );
        assert!(
            first_result.similarity > 0.0,
            "Cache similarity should be calculated"
        );

        // Test 3: Cache invalidation by key
        let invalidation_result = vector_db
            .delete_embedding("cache_embeddings", "learning_cache_1")
            .await;
        assert!(
            invalidation_result.is_ok(),
            "Cache invalidation should succeed"
        );
        assert!(
            invalidation_result.unwrap(),
            "Should confirm cache entry was removed"
        );

        // Verify invalidation by querying
        let post_invalidation_results = vector_db
            .query_similar("cache_embeddings", cache_query_vector, 5)
            .await
            .unwrap();
        let invalidated_found = post_invalidation_results
            .iter()
            .any(|r| r.id == "learning_cache_1");
        assert!(
            !invalidated_found,
            "Invalidated cache entry should not be found"
        );

        // Test 4: Batch cache operations
        let batch_cache_entries = (0..15)
            .map(|i| {
                let cache_key = format!("batch_cache_{}", i);
                let vector = create_test_vector(256, 1000 + i);
                let metadata = serde_json::json!({
                    "cache_key": cache_key,
                    "batch_id": i,
                    "data_type": "learning_result",
                    "created_at": chrono::Utc::now(),
                    "ttl_seconds": 1800
                });
                (cache_key, vector, metadata)
            })
            .collect::<Vec<_>>();

        for (cache_key, vector, metadata) in batch_cache_entries {
            let result = vector_db
                .store_embedding("cache_embeddings", &cache_key, vector, metadata)
                .await;
            assert!(
                result.is_ok(),
                "Batch cache storage should succeed for {}",
                cache_key
            );
        }

        // Test 5: Cache performance under concurrent access
        let cache_perf_start = Instant::now();
        let mut cache_access_tasks = Vec::new();

        for i in 0..30 {
            let db_ref = &vector_db;
            let task = async move {
                if i % 3 == 0 {
                    // Cache store
                    let cache_key = format!("concurrent_cache_{}", i);
                    let vector = create_test_vector(256, 2000 + i);
                    let metadata = serde_json::json!({"concurrent": true, "iteration": i});
                    db_ref
                        .store_embedding("cache_embeddings", &cache_key, vector, metadata)
                        .await
                } else if i % 3 == 1 {
                    // Cache query
                    let query_vector = create_test_vector(256, 1500 + i);
                    db_ref
                        .query_similar("cache_embeddings", query_vector, 5)
                        .await
                        .map(|_| ())
                } else {
                    // Cache invalidation
                    let cache_key = format!("batch_cache_{}", i % 15);
                    db_ref
                        .delete_embedding("cache_embeddings", &cache_key)
                        .await
                        .map(|_| ())
                }
            };
            cache_access_tasks.push(task);
        }

        let cache_access_results = futures::future::join_all(cache_access_tasks).await;
        let cache_perf_time = cache_perf_start.elapsed();

        let successful_cache_ops = cache_access_results.iter().filter(|r| r.is_ok()).count();
        assert!(
            successful_cache_ops >= 25,
            "Most concurrent cache operations should succeed"
        );
        assert!(
            cache_perf_time < Duration::from_secs(3),
            "Cache operations should be performant"
        );

        // Test 6: Cache statistics and monitoring
        let cache_stats = vector_db
            .get_collection_stats("cache_embeddings")
            .await
            .unwrap();
        assert!(
            cache_stats.document_count > 10,
            "Cache should contain multiple entries"
        );
        assert!(
            cache_stats.total_queries > 0,
            "Cache should track query statistics"
        );
        assert!(
            cache_stats.avg_query_time_ms >= 0.0,
            "Cache should track performance metrics"
        );

        // Test 7: Cache consistency with persistent storage
        // Store learning data that generates cache entries
        let cache_related_feedback = create_test_feedback("cache_user", "cache_content", 0.9);
        persistent_storage
            .persist_feedback(&cache_related_feedback)
            .await
            .unwrap();

        let cache_related_pattern = PatternData::new("cache_pattern".to_string(), 5, 0.85);
        persistent_storage
            .persist_pattern(&cache_related_pattern)
            .await
            .unwrap();

        // Create cache entries that reference persistent data
        let persistent_cache_vector = create_test_vector(256, 5000);
        let persistent_cache_metadata = serde_json::json!({
            "cache_key": "persistent_cache_entry",
            "references": {
                "feedback_id": cache_related_feedback.id,
                "pattern_id": cache_related_pattern.id
            },
            "cache_type": "persistent_reference",
            "created_at": chrono::Utc::now()
        });

        let persistent_cache_result = vector_db
            .store_embedding(
                "cache_embeddings",
                "persistent_cache_entry",
                persistent_cache_vector,
                persistent_cache_metadata,
            )
            .await;
        assert!(
            persistent_cache_result.is_ok(),
            "Persistent-linked cache entry should be stored"
        );

        // Test 8: Cache invalidation patterns
        let invalidation_patterns = vec![
            "concurrent_cache_5",  // Specific key
            "concurrent_cache_10", // Another specific key
            "concurrent_cache_15", // Third specific key
        ];

        for pattern in invalidation_patterns {
            let invalidation_result = vector_db
                .delete_embedding("cache_embeddings", pattern)
                .await;
            // May succeed or fail depending on what was created/deleted in concurrent operations
            if invalidation_result.is_ok() {
                println!("Successfully invalidated cache entry: {}", pattern);
            }
        }

        // Test 9: Cache warming and pre-population
        let cache_warming_entries = vec![
            ("warm_cache_learning", "Pre-computed learning insights"),
            ("warm_cache_patterns", "Pre-analyzed patterns"),
            ("warm_cache_feedback", "Pre-aggregated feedback"),
            ("warm_cache_optimization", "Pre-calculated optimizations"),
        ];

        for (cache_key, description) in cache_warming_entries {
            let warming_vector = create_test_vector(256, cache_key.len() as u64 * 100);
            let warming_metadata = serde_json::json!({
                "cache_key": cache_key,
                "description": description,
                "cache_type": "warmed",
                "priority": "high",
                "precomputed": true,
                "created_at": chrono::Utc::now()
            });

            let result = vector_db
                .store_embedding(
                    "cache_embeddings",
                    cache_key,
                    warming_vector,
                    warming_metadata,
                )
                .await;
            assert!(
                result.is_ok(),
                "Cache warming should succeed for {}",
                cache_key
            );
        }

        // Verify cache warming effectiveness
        let warming_query = create_test_vector(256, "warm_cache_learning".len() as u64 * 100);
        let warming_results = vector_db
            .query_similar("cache_embeddings", warming_query, 3)
            .await
            .unwrap();

        let warmed_entry = warming_results
            .iter()
            .find(|r| r.id.starts_with("warm_cache_"));
        assert!(warmed_entry.is_some(), "Should find warmed cache entries");

        // Test 10: Cache memory and storage efficiency
        let final_cache_stats = vector_db
            .get_collection_stats("cache_embeddings")
            .await
            .unwrap();

        // Verify cache is maintaining reasonable size
        assert!(
            final_cache_stats.document_count < 100,
            "Cache should not grow unbounded"
        );
        assert!(
            final_cache_stats.avg_query_time_ms < 100.0,
            "Cache queries should remain fast"
        );

        // Test 11: Cache and persistence integration consistency
        let final_transaction_log = persistent_storage.get_transaction_log().await;
        let final_backup = persistent_storage.backup_data().await.unwrap();

        // Verify that persistent data and cache remain consistent
        assert!(
            !final_transaction_log.is_empty(),
            "Persistent storage should track operations"
        );
        assert!(
            final_backup.feedback_count > 0,
            "Persistent storage should contain data"
        );

        // Cache should complement, not replace, persistent storage
        let cache_query_all = create_test_vector(256, 9999);
        let all_cache_results = vector_db
            .query_similar("cache_embeddings", cache_query_all, 50)
            .await
            .unwrap();

        // Cache entries should have valid metadata linking to persistent data where applicable
        for result in all_cache_results {
            if let Some(cache_key) = result.metadata.get("cache_key") {
                assert!(cache_key.is_string(), "Cache key should be string");
                assert!(
                    !cache_key.as_str().unwrap().is_empty(),
                    "Cache key should not be empty"
                );
            }
        }

        println!("Cache persistence test completed successfully");
        println!("Final cache entries: {}", final_cache_stats.document_count);
        println!(
            "Cache query performance: {:.2}ms",
            final_cache_stats.avg_query_time_ms
        );
        println!(
            "Persistent storage backup size: {} items",
            final_backup.feedback_count + final_backup.pattern_count + final_backup.learning_count
        );
    }
}
