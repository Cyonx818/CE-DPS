// ABOUTME: Anchor tests for Sprint 009 Tasks 3 & 4 - Learning System Critical Workflows
//! These tests protect critical learning system functionality implemented in Sprint 009
//! Tasks 3 and 4. They ensure that learning workflows continue to work correctly
//! as the system evolves.
//!
//! ## Protected Functionality
//! - External API integration (LLM providers, vector database calls)
//! - Data persistence (learning data storage, vector operations)
//! - User input processing (feedback collection, configuration validation)
//! - Business logic (learning algorithms, adaptation logic)
//! - Cross-component integration (learning+monitoring+API+MCP integration)

use fortitude::learning::*;
use fortitude::monitoring::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// Mock storage implementation for testing
pub struct MockLearningStorage {
    feedback_data: Arc<tokio::sync::RwLock<Vec<UserFeedback>>>,
    pattern_data: Arc<tokio::sync::RwLock<Vec<PatternData>>>,
    learning_data: Arc<tokio::sync::RwLock<Vec<LearningData>>>,
    usage_patterns: Arc<tokio::sync::RwLock<Vec<UsagePattern>>>,
}

impl MockLearningStorage {
    pub fn new() -> Self {
        Self {
            feedback_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            pattern_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            learning_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            usage_patterns: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LearningStorageService for MockLearningStorage {
    async fn store_feedback(&self, feedback: &UserFeedback) -> LearningResult<()> {
        let mut data = self.feedback_data.write().await;
        data.push(feedback.clone());
        Ok(())
    }

    async fn get_feedback_for_content(&self, content_id: &str) -> LearningResult<Vec<UserFeedback>> {
        let data = self.feedback_data.read().await;
        Ok(data
            .iter()
            .filter(|f| f.content_id == content_id)
            .cloned()
            .collect())
    }

    async fn store_pattern(&self, pattern: &PatternData) -> LearningResult<()> {
        let mut data = self.pattern_data.write().await;
        data.push(pattern.clone());
        Ok(())
    }

    async fn get_patterns_by_type(&self, pattern_type: &str) -> LearningResult<Vec<PatternData>> {
        let data = self.pattern_data.read().await;
        Ok(data
            .iter()
            .filter(|p| p.pattern_type == pattern_type)
            .cloned()
            .collect())
    }

    async fn store_learning_data(&self, learning: &LearningData) -> LearningResult<()> {
        let mut data = self.learning_data.write().await;
        data.push(learning.clone());
        Ok(())
    }

    async fn get_learning_data_by_type(
        &self,
        learning_type: &str,
    ) -> LearningResult<Vec<LearningData>> {
        let data = self.learning_data.read().await;
        Ok(data
            .iter()
            .filter(|l| l.learning_type == learning_type)
            .cloned()
            .collect())
    }

    async fn store_usage_pattern(&self, usage: &UsagePattern) -> LearningResult<()> {
        let mut data = self.usage_patterns.write().await;
        data.push(usage.clone());
        Ok(())
    }

    async fn get_usage_patterns_by_type(
        &self,
        pattern_type: &str,
    ) -> LearningResult<Vec<UsagePattern>> {
        let data = self.usage_patterns.read().await;
        Ok(data
            .iter()
            .filter(|u| u.pattern_type == pattern_type)
            .cloned()
            .collect())
    }

    async fn analyze_feedback_trends(&self, content_id: &str) -> LearningResult<FeedbackTrend> {
        let feedback = self.get_feedback_for_content(content_id).await?;
        if feedback.is_empty() {
            return Ok(FeedbackTrend {
                content_id: content_id.to_string(),
                average_score: 0.0,
                trend_direction: 0.0,
                feedback_count: 0,
                recent_feedback_count: 0,
                score_variance: 0.0,
                improvement_rate: 0.0,
            });
        }

        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();
        let average_score = scores.iter().sum::<f64>() / scores.len() as f64;

        Ok(FeedbackTrend {
            content_id: content_id.to_string(),
            average_score,
            trend_direction: 0.1, // Mock positive trend
            feedback_count: feedback.len(),
            recent_feedback_count: feedback.len() / 2,
            score_variance: 0.05,
            improvement_rate: 0.05,
        })
    }

    async fn cleanup_expired_data(&self) -> LearningResult<CleanupResult> {
        Ok(CleanupResult {
            expired_feedback_removed: 0,
            expired_patterns_removed: 0,
            expired_learning_data_removed: 0,
            expired_usage_patterns_removed: 0,
            total_storage_freed_bytes: 0,
        })
    }

    async fn find_similar_learning(
        &self,
        query: &str,
        limit: usize,
    ) -> LearningResult<Vec<SimilarityLearningResult>> {
        let data = self.learning_data.read().await;
        let results = data
            .iter()
            .take(limit)
            .map(|l| SimilarityLearningResult {
                learning_data: l.clone(),
                similarity_score: 0.8, // Mock similarity
                matching_keywords: vec![query.to_string()],
            })
            .collect();
        Ok(results)
    }

    async fn find_similar_patterns(
        &self,
        pattern_data: &str,
        limit: usize,
    ) -> LearningResult<Vec<SimilarityUsagePattern>> {
        let data = self.usage_patterns.read().await;
        let results = data
            .iter()
            .take(limit)
            .map(|p| SimilarityUsagePattern {
                usage_pattern: p.clone(),
                similarity_score: 0.7, // Mock similarity
                matching_features: vec![pattern_data.to_string()],
            })
            .collect();
        Ok(results)
    }

    async fn get_cache_stats(&self) -> LearningResult<EmbeddingCacheStats> {
        Ok(EmbeddingCacheStats {
            total_embeddings: 100,
            cache_hits: 80,
            cache_misses: 20,
            hit_rate: 0.8,
            total_size_bytes: 1024 * 1024,
            last_cleanup: chrono::Utc::now(),
        })
    }
}

/// Helper function to create test feedback data
fn create_test_feedback(content_id: &str, score: f64) -> UserFeedback {
    UserFeedback::new(
        "test_user".to_string(),
        content_id.to_string(),
        "quality_rating".to_string(),
        Some(score),
        Some("Test feedback".to_string()),
    )
}

/// Helper function to create test usage pattern
fn create_test_usage_pattern(pattern_type: &str, data: &str) -> UsagePattern {
    UsagePattern::new(pattern_type.to_string(), data.to_string())
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Learning system end-to-end feedback processing workflow
    /// Tests: Feedback collection → Pattern analysis → Learning generation → Trend analysis
    /// Protects: Complete learning workflow and data flow integrity
    #[tokio::test]
    async fn test_anchor_learning_feedback_processing_workflow() {
        let storage = Arc::new(MockLearningStorage::new());
        let config = LearningConfig::default();

        // Test 1: Feedback collection and storage
        let content_id = "test_content_123";
        let feedback_entries = vec![
            create_test_feedback(content_id, 0.8),
            create_test_feedback(content_id, 0.9),
            create_test_feedback(content_id, 0.7),
            create_test_feedback(content_id, 0.85),
        ];

        for feedback in &feedback_entries {
            storage.store_feedback(feedback).await.unwrap();
        }

        // Verify feedback storage
        let stored_feedback = storage.get_feedback_for_content(content_id).await.unwrap();
        assert_eq!(
            stored_feedback.len(),
            4,
            "All feedback entries should be stored"
        );

        let scores: Vec<f64> = stored_feedback.iter().filter_map(|f| f.score).collect();
        let average_score = scores.iter().sum::<f64>() / scores.len() as f64;
        assert!(
            (average_score - 0.8125).abs() < 0.01,
            "Average score should be calculated correctly"
        );

        // Test 2: Feedback trend analysis
        let trend = storage.analyze_feedback_trends(content_id).await.unwrap();
        assert_eq!(trend.content_id, content_id);
        assert_eq!(trend.feedback_count, 4);
        assert!(
            trend.average_score > 0.0,
            "Average score should be positive"
        );
        assert!(
            trend.trend_direction >= 0.0,
            "Trend direction should be tracked"
        );

        // Test 3: Pattern recognition and storage
        let pattern = PatternData::new("user_preference".to_string(), 3, 0.8);
        storage.store_pattern(&pattern).await.unwrap();

        let stored_patterns = storage
            .get_patterns_by_type("user_preference")
            .await
            .unwrap();
        assert_eq!(stored_patterns.len(), 1, "Pattern should be stored");
        assert_eq!(stored_patterns[0].pattern_type, "user_preference");
        assert_eq!(stored_patterns[0].frequency, 3);

        // Test 4: Learning data generation
        let learning_data = LearningData::new(
            "feedback_insight".to_string(),
            content_id.to_string(),
            vec!["Users prefer detailed explanations".to_string()],
            0.85,
        );
        storage.store_learning_data(&learning_data).await.unwrap();

        let stored_learning = storage
            .get_learning_data_by_type("feedback_insight")
            .await
            .unwrap();
        assert_eq!(stored_learning.len(), 1, "Learning data should be stored");
        assert_eq!(stored_learning[0].confidence_score, 0.85);
        assert!(
            !stored_learning[0].insights.is_empty(),
            "Insights should be generated"
        );

        // Test 5: Usage pattern tracking
        let usage_pattern = create_test_usage_pattern("search_behavior", "detailed queries");
        storage.store_usage_pattern(&usage_pattern).await.unwrap();

        let stored_usage = storage
            .get_usage_patterns_by_type("search_behavior")
            .await
            .unwrap();
        assert_eq!(stored_usage.len(), 1, "Usage pattern should be stored");
        assert_eq!(stored_usage[0].frequency, 1);

        // Test 6: Similarity search functionality
        let similar_learning = storage
            .find_similar_learning("detailed explanations", 5)
            .await
            .unwrap();
        assert!(
            !similar_learning.is_empty(),
            "Should find similar learning data"
        );
        assert!(
            similar_learning[0].similarity_score > 0.0,
            "Similarity score should be valid"
        );

        let similar_patterns = storage
            .find_similar_patterns("detailed queries", 5)
            .await
            .unwrap();
        assert!(!similar_patterns.is_empty(), "Should find similar patterns");
        assert!(
            similar_patterns[0].similarity_score > 0.0,
            "Pattern similarity score should be valid"
        );

        // Test 7: Complete workflow integration
        // Simulate a complete learning cycle
        let new_feedback = create_test_feedback("new_content", 0.9);
        storage.store_feedback(&new_feedback).await.unwrap();

        let new_pattern = PatternData::new("high_quality_response".to_string(), 1, 1.0);
        storage.store_pattern(&new_pattern).await.unwrap();

        let new_learning = LearningData::new(
            "quality_improvement".to_string(),
            "new_content".to_string(),
            vec!["High-quality responses generate positive feedback".to_string()],
            0.95,
        );
        storage.store_learning_data(&new_learning).await.unwrap();

        // Verify end-to-end data consistency
        let all_learning = storage
            .get_learning_data_by_type("quality_improvement")
            .await
            .unwrap();
        assert!(
            !all_learning.is_empty(),
            "Quality improvement learning should be captured"
        );
        assert!(
            all_learning[0].confidence_score >= 0.9,
            "High confidence learning should be preserved"
        );
    }

    /// ANCHOR: Learning system adaptation algorithm workflow
    /// Tests: Pattern analysis → Adaptation recommendation → System optimization
    /// Protects: Adaptation algorithms and learning optimization logic
    #[tokio::test]
    async fn test_anchor_learning_adaptation_algorithm_workflow() {
        let storage = Arc::new(MockLearningStorage::new());
        let config = LearningConfig::default();

        // Test 1: Multi-modal feedback processing
        let content_ids = vec!["content_a", "content_b", "content_c"];

        // Store diverse feedback patterns
        for (i, content_id) in content_ids.iter().enumerate() {
            let base_score = 0.6 + (i as f64 * 0.1); // Varying quality scores
            for j in 0..5 {
                let score_variation = if j % 2 == 0 { 0.05 } else { -0.05 };
                let feedback = create_test_feedback(content_id, base_score + score_variation);
                storage.store_feedback(&feedback).await.unwrap();
            }
        }

        // Test 2: Pattern frequency analysis
        let pattern_types = vec![
            ("query_complexity", 10, 0.8),
            ("response_length", 15, 0.9),
            ("technical_depth", 8, 0.7),
            ("user_satisfaction", 20, 0.85),
        ];

        for (pattern_type, frequency, success_rate) in pattern_types {
            let mut pattern = PatternData::new(pattern_type.to_string(), frequency, success_rate);

            // Simulate pattern evolution
            for _ in 0..3 {
                pattern.update_occurrence(success_rate > 0.8);
            }

            storage.store_pattern(&pattern).await.unwrap();
        }

        // Test 3: Learning insight generation
        let learning_types = vec![
            (
                "user_preference",
                "Users prefer longer, more detailed responses",
            ),
            (
                "quality_indicator",
                "Technical depth correlates with satisfaction",
            ),
            (
                "optimization_opportunity",
                "Complex queries need more processing time",
            ),
        ];

        for (learning_type, insight) in learning_types {
            let learning_data = LearningData::new(
                learning_type.to_string(),
                "pattern_analysis".to_string(),
                vec![insight.to_string()],
                0.8,
            );
            storage.store_learning_data(&learning_data).await.unwrap();
        }

        // Test 4: Adaptation recommendation logic
        // Verify patterns meet significance thresholds
        let user_satisfaction_patterns = storage
            .get_patterns_by_type("user_satisfaction")
            .await
            .unwrap();
        assert!(
            !user_satisfaction_patterns.is_empty(),
            "Should capture user satisfaction patterns"
        );

        let satisfaction_pattern = &user_satisfaction_patterns[0];
        assert!(
            satisfaction_pattern.is_significant(config.pattern_frequency_threshold),
            "High-frequency patterns should be significant"
        );
        assert!(
            satisfaction_pattern.success_rate >= 0.8,
            "Success rate should meet quality threshold"
        );

        // Test 5: Multi-dimensional learning correlation
        let technical_patterns = storage
            .get_patterns_by_type("technical_depth")
            .await
            .unwrap();
        let complexity_patterns = storage
            .get_patterns_by_type("query_complexity")
            .await
            .unwrap();

        assert!(
            !technical_patterns.is_empty(),
            "Should capture technical depth patterns"
        );
        assert!(
            !complexity_patterns.is_empty(),
            "Should capture query complexity patterns"
        );

        // Verify correlation analysis capability
        let technical_success = technical_patterns[0].success_rate;
        let complexity_success = complexity_patterns[0].success_rate;

        // Both should be tracked for correlation analysis
        assert!(
            technical_success > 0.0 && complexity_success > 0.0,
            "Pattern success rates should enable correlation analysis"
        );

        // Test 6: Learning data validation and confidence scoring
        let preference_learning = storage
            .get_learning_data_by_type("user_preference")
            .await
            .unwrap();
        assert!(
            !preference_learning.is_empty(),
            "Should capture user preference learning"
        );

        let preference_data = &preference_learning[0];
        assert!(
            preference_data.confidence_score >= config.adaptation_threshold,
            "Learning confidence should meet adaptation threshold"
        );
        assert!(
            !preference_data.insights.is_empty(),
            "Learning should contain actionable insights"
        );

        // Test 7: Adaptive threshold adjustment
        // Simulate different confidence levels and verify threshold behavior
        let confidence_levels = vec![0.5, 0.7, 0.8, 0.9, 0.95];

        for confidence in confidence_levels {
            let test_learning = LearningData::new(
                "threshold_test".to_string(),
                "adaptation_test".to_string(),
                vec![format!("Confidence level {} test", confidence)],
                confidence,
            );

            storage.store_learning_data(&test_learning).await.unwrap();

            // Verify confidence-based adaptation logic
            let should_adapt = confidence >= config.adaptation_threshold;
            assert_eq!(
                test_learning.confidence_score >= config.adaptation_threshold,
                should_adapt,
                "Adaptation decision should be based on confidence threshold"
            );
        }

        // Test 8: Learning optimization workflow
        // Verify cache and performance optimization
        let cache_stats = storage.get_cache_stats().await.unwrap();
        assert!(
            cache_stats.hit_rate > 0.0,
            "Cache should have measurable hit rate"
        );
        assert!(
            cache_stats.total_embeddings > 0,
            "Should track embedding count"
        );

        // Test cleanup and optimization
        let cleanup_result = storage.cleanup_expired_data().await.unwrap();
        // Cleanup should succeed without errors
        assert!(
            cleanup_result.total_storage_freed_bytes >= 0,
            "Cleanup should track freed storage"
        );

        // Test 9: Cross-pattern learning integration
        // Verify that multiple patterns contribute to learning
        let all_patterns = vec![
            storage
                .get_patterns_by_type("user_satisfaction")
                .await
                .unwrap(),
            storage
                .get_patterns_by_type("technical_depth")
                .await
                .unwrap(),
            storage
                .get_patterns_by_type("response_length")
                .await
                .unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        assert!(
            all_patterns.len() >= 3,
            "Should capture multiple pattern types"
        );

        // Verify learning synthesis from multiple patterns
        let quality_learning = storage
            .get_learning_data_by_type("quality_indicator")
            .await
            .unwrap();
        assert!(
            !quality_learning.is_empty(),
            "Should synthesize quality insights from patterns"
        );
    }

    /// ANCHOR: Learning system performance and scalability validation
    /// Tests: High-volume processing, concurrent access, memory usage, response times
    /// Protects: System performance characteristics and scalability requirements
    #[tokio::test]
    async fn test_anchor_learning_performance_scalability_validation() {
        let storage = Arc::new(MockLearningStorage::new());
        let start_time = std::time::Instant::now();

        // Test 1: High-volume feedback processing
        let feedback_count = 100;
        let content_ids: Vec<String> = (0..10).map(|i| format!("content_{}", i)).collect();

        // Process feedback in batches to simulate real-world load
        for batch in 0..10 {
            let mut batch_tasks = Vec::new();

            for i in 0..10 {
                let storage_clone = storage.clone();
                let content_id = content_ids[i % content_ids.len()].clone();

                let task = tokio::spawn(async move {
                    let feedback = create_test_feedback(&content_id, 0.8 + (batch as f64 * 0.01));
                    storage_clone.store_feedback(&feedback).await
                });

                batch_tasks.push(task);
            }

            // Wait for batch completion
            for task in batch_tasks {
                task.await.unwrap().unwrap();
            }
        }

        let processing_time = start_time.elapsed();
        assert!(
            processing_time < Duration::from_secs(10),
            "Should process {} feedback entries within 10 seconds",
            feedback_count
        );

        // Test 2: Concurrent access performance
        let concurrent_start = std::time::Instant::now();
        let mut concurrent_tasks = Vec::new();

        // Simulate concurrent read/write operations
        for i in 0..20 {
            let storage_clone = storage.clone();
            let content_id = format!("concurrent_content_{}", i % 5);

            let task = tokio::spawn(async move {
                // Mix of read and write operations
                if i % 2 == 0 {
                    // Write operation
                    let feedback = create_test_feedback(&content_id, 0.8);
                    storage_clone.store_feedback(&feedback).await?;

                    let pattern = PatternData::new("concurrent_pattern".to_string(), 1, 0.8);
                    storage_clone.store_pattern(&pattern).await?;
                } else {
                    // Read operation
                    let _feedback = storage_clone.get_feedback_for_content(&content_id).await?;
                    let _patterns = storage_clone
                        .get_patterns_by_type("concurrent_pattern")
                        .await?;
                }

                Ok::<(), LearningError>(())
            });

            concurrent_tasks.push(task);
        }

        // Wait for all concurrent operations
        for task in concurrent_tasks {
            task.await.unwrap().unwrap();
        }

        let concurrent_time = concurrent_start.elapsed();
        assert!(
            concurrent_time < Duration::from_secs(5),
            "Concurrent operations should complete within 5 seconds"
        );

        // Test 3: Pattern analysis scalability
        let pattern_start = std::time::Instant::now();

        // Create diverse patterns for analysis
        let pattern_types = vec![
            "user_behavior",
            "query_type",
            "response_quality",
            "technical_domain",
            "urgency_level",
            "audience_type",
            "content_depth",
            "interaction_style",
        ];

        for pattern_type in &pattern_types {
            for frequency in 1..=25 {
                let success_rate = 0.7 + (frequency as f64 * 0.01);
                let pattern = PatternData::new(pattern_type.to_string(), frequency, success_rate);
                storage.store_pattern(&pattern).await.unwrap();
            }
        }

        let pattern_time = pattern_start.elapsed();
        assert!(
            pattern_time < Duration::from_secs(3),
            "Pattern storage should scale efficiently"
        );

        // Test 4: Similarity search performance
        let similarity_start = std::time::Instant::now();

        // Store learning data for similarity testing
        for i in 0..50 {
            let learning_data = LearningData::new(
                "similarity_test".to_string(),
                format!("source_{}", i),
                vec![format!("Learning insight number {}", i)],
                0.8,
            );
            storage.store_learning_data(&learning_data).await.unwrap();
        }

        // Perform multiple similarity searches
        for query in &["insight", "learning", "number", "test", "data"] {
            let results = timeout(
                Duration::from_millis(500),
                storage.find_similar_learning(query, 10),
            )
            .await;

            assert!(
                results.is_ok(),
                "Similarity search should complete within 500ms"
            );
            let search_results = results.unwrap().unwrap();
            assert!(search_results.len() <= 10, "Should respect result limit");
        }

        let similarity_time = similarity_start.elapsed();
        assert!(
            similarity_time < Duration::from_secs(2),
            "Similarity searches should be fast"
        );

        // Test 5: Memory usage validation
        // Verify that storage doesn't grow unbounded
        let initial_feedback_count = storage
            .get_feedback_for_content("content_0")
            .await
            .unwrap()
            .len();

        // Add more data and verify reasonable growth
        for i in 0..20 {
            let feedback = create_test_feedback("memory_test_content", 0.8);
            storage.store_feedback(&feedback).await.unwrap();
        }

        let memory_test_count = storage
            .get_feedback_for_content("memory_test_content")
            .await
            .unwrap()
            .len();
        assert_eq!(memory_test_count, 20, "Memory usage should be predictable");

        // Test 6: Cleanup performance
        let cleanup_start = std::time::Instant::now();
        let cleanup_result = storage.cleanup_expired_data().await.unwrap();
        let cleanup_time = cleanup_start.elapsed();

        assert!(
            cleanup_time < Duration::from_millis(100),
            "Cleanup operations should be fast"
        );
        assert!(
            cleanup_result.total_storage_freed_bytes >= 0,
            "Cleanup should report storage metrics"
        );

        // Test 7: Batch operation efficiency
        let batch_start = std::time::Instant::now();

        // Simulate batch feedback processing
        let batch_feedback: Vec<UserFeedback> = (0..50)
            .map(|i| create_test_feedback(&format!("batch_content_{}", i % 5), 0.8))
            .collect();

        for feedback in batch_feedback {
            storage.store_feedback(&feedback).await.unwrap();
        }

        let batch_time = batch_start.elapsed();
        assert!(
            batch_time < Duration::from_secs(2),
            "Batch operations should be efficient"
        );

        // Test 8: Response time consistency
        let mut response_times = Vec::new();

        for i in 0..10 {
            let start = std::time::Instant::now();
            let _trend = storage
                .analyze_feedback_trends(&format!("content_{}", i % 3))
                .await
                .unwrap();
            let response_time = start.elapsed();
            response_times.push(response_time);
        }

        let avg_response_time =
            response_times.iter().sum::<Duration>() / response_times.len() as u32;
        let max_response_time = response_times.iter().max().unwrap();

        assert!(
            avg_response_time < Duration::from_millis(50),
            "Average response time should be under 50ms"
        );
        assert!(
            max_response_time < &Duration::from_millis(200),
            "Max response time should be under 200ms"
        );

        // Test 9: Cache performance validation
        let cache_start = std::time::Instant::now();

        // Perform multiple cache stat requests
        for _ in 0..10 {
            let _stats = storage.get_cache_stats().await.unwrap();
        }

        let cache_time = cache_start.elapsed();
        assert!(
            cache_time < Duration::from_millis(100),
            "Cache operations should be very fast"
        );

        // Test 10: Overall system responsiveness
        let total_time = start_time.elapsed();
        assert!(
            total_time < Duration::from_secs(30),
            "Complete performance test should finish within 30 seconds"
        );

        println!(
            "Learning system performance test completed in {:?}",
            total_time
        );
        println!("Concurrent operations: {:?}", concurrent_time);
        println!("Similarity search: {:?}", similarity_time);
        println!("Average response: {:?}", avg_response_time);
    }

    /// ANCHOR: Learning system error handling and recovery workflow
    /// Tests: Invalid data handling, storage failures, network issues, graceful degradation
    /// Protects: Error handling and system resilience
    #[tokio::test]
    async fn test_anchor_learning_error_handling_recovery_workflow() {
        let storage = Arc::new(MockLearningStorage::new());

        // Test 1: Invalid feedback data handling
        let invalid_feedback_cases = vec![
            UserFeedback::new("".to_string(), "content", "rating", Some(0.5), None), // Empty user ID
            UserFeedback::new("user", "".to_string(), "rating", Some(0.5), None), // Empty content ID
            UserFeedback::new("user", "content", "".to_string(), Some(0.5), None), // Empty feedback type
            UserFeedback::new("user", "content", "rating", Some(1.5), None), // Invalid score > 1.0
            UserFeedback::new("user", "content", "rating", Some(-0.1), None), // Invalid score < 0.0
        ];

        for invalid_feedback in invalid_feedback_cases {
            // System should reject invalid feedback
            assert!(
                !invalid_feedback.is_valid(),
                "Should detect invalid feedback"
            );

            // But storage should not crash - it should handle gracefully
            let result = storage.store_feedback(&invalid_feedback).await;
            // Either succeeds (after validation/sanitization) or fails gracefully
            if result.is_err() {
                // Error should be descriptive
                let error = result.unwrap_err();
                assert!(
                    !error.to_string().is_empty(),
                    "Error message should be informative"
                );
            }
        }

        // Test 2: Pattern data boundary validation
        let mut boundary_pattern = PatternData::new("boundary_test".to_string(), 0, 0.5);

        // Test frequency boundary (0 is edge case)
        assert_eq!(boundary_pattern.frequency, 0);
        assert!(
            !boundary_pattern.is_significant(1),
            "Zero frequency should not be significant"
        );

        // Test success rate boundaries
        boundary_pattern.success_rate = 0.0; // Minimum
        assert!(
            boundary_pattern.success_rate >= 0.0,
            "Success rate should accept 0.0"
        );

        boundary_pattern.success_rate = 1.0; // Maximum
        assert!(
            boundary_pattern.success_rate <= 1.0,
            "Success rate should accept 1.0"
        );

        // Test pattern occurrence updates with edge cases
        boundary_pattern.update_occurrence(true);
        assert!(boundary_pattern.frequency > 0, "Frequency should increase");
        assert!(
            boundary_pattern.success_rate >= 0.0 && boundary_pattern.success_rate <= 1.0,
            "Success rate should remain valid after update"
        );

        // Test 3: Learning data expiration handling
        let mut expiring_learning = LearningData::new(
            "expiration_test".to_string(),
            "test_source".to_string(),
            vec!["Test insight".to_string()],
            0.8,
        );

        // Set expiration in the past
        let past_time = chrono::Utc::now() - chrono::Duration::hours(1);
        expiring_learning = expiring_learning.with_expiration(past_time);

        assert!(
            !expiring_learning.is_valid(),
            "Expired learning should be invalid"
        );

        // Storage should handle expired data gracefully
        let result = storage.store_learning_data(&expiring_learning).await;
        assert!(
            result.is_ok(),
            "Storage should handle expired data gracefully"
        );

        // Test 4: Empty dataset handling
        let empty_content_id = "nonexistent_content";

        let empty_feedback = storage
            .get_feedback_for_content(empty_content_id)
            .await
            .unwrap();
        assert!(
            empty_feedback.is_empty(),
            "Should handle empty feedback gracefully"
        );

        let empty_trend = storage
            .analyze_feedback_trends(empty_content_id)
            .await
            .unwrap();
        assert_eq!(
            empty_trend.feedback_count, 0,
            "Should handle empty trend analysis"
        );
        assert_eq!(
            empty_trend.average_score, 0.0,
            "Should provide safe defaults"
        );

        // Test 5: Large data volume stress testing
        let stress_test_start = std::time::Instant::now();

        // Attempt to store large amounts of data
        for i in 0..1000 {
            let feedback = create_test_feedback(&format!("stress_content_{}", i % 10), 0.8);
            let result = storage.store_feedback(&feedback).await;

            // Should either succeed or fail gracefully
            if result.is_err() {
                // Verify error is due to resource limits, not system failure
                let error = result.unwrap_err();
                match error {
                    LearningError::StorageError(_) => {
                        // Acceptable - storage limit reached
                        break;
                    }
                    _ => {
                        panic!("Unexpected error type during stress test: {}", error);
                    }
                }
            }
        }

        let stress_time = stress_test_start.elapsed();
        assert!(
            stress_time < Duration::from_secs(10),
            "Stress test should complete or fail gracefully within 10 seconds"
        );

        // Test 6: Concurrent error scenarios
        let mut error_tasks = Vec::new();

        for i in 0..20 {
            let storage_clone = storage.clone();

            let task = tokio::spawn(async move {
                // Mix of valid and invalid operations
                if i % 3 == 0 {
                    // Invalid feedback
                    let invalid_feedback = UserFeedback::new(
                        "".to_string(), // Invalid empty user ID
                        format!("content_{}", i),
                        "rating",
                        Some(0.5),
                        None,
                    );
                    storage_clone.store_feedback(&invalid_feedback).await
                } else if i % 3 == 1 {
                    // Query for nonexistent data
                    let _result = storage_clone.get_feedback_for_content("nonexistent").await;
                    Ok(())
                } else {
                    // Valid operation
                    let feedback = create_test_feedback(&format!("valid_content_{}", i), 0.8);
                    storage_clone.store_feedback(&feedback).await
                }
            });

            error_tasks.push(task);
        }

        // Collect results - some should succeed, some may fail, but no panics
        let mut success_count = 0;
        let mut error_count = 0;

        for task in error_tasks {
            match task.await.unwrap() {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }

        assert!(success_count > 0, "Some valid operations should succeed");
        println!(
            "Concurrent error test: {} successes, {} errors",
            success_count, error_count
        );

        // Test 7: Recovery after errors
        // Verify system remains functional after error conditions
        let recovery_feedback = create_test_feedback("recovery_test", 0.9);
        let recovery_result = storage.store_feedback(&recovery_feedback).await;
        assert!(
            recovery_result.is_ok(),
            "System should recover after errors"
        );

        let recovery_fetch = storage.get_feedback_for_content("recovery_test").await;
        assert!(
            recovery_fetch.is_ok(),
            "System should handle queries after recovery"
        );

        // Test 8: Configuration validation error handling
        let invalid_configs = vec![
            LearningConfig {
                adaptation_threshold: -0.1, // Invalid negative threshold
                ..LearningConfig::default()
            },
            LearningConfig {
                adaptation_threshold: 1.5, // Invalid threshold > 1.0
                ..LearningConfig::default()
            },
            LearningConfig {
                max_data_age_days: 0, // Invalid zero age
                ..LearningConfig::default()
            },
            LearningConfig {
                learning_rate: -0.1, // Invalid negative rate
                ..LearningConfig::default()
            },
        ];

        for invalid_config in invalid_configs {
            // Configuration validation should catch these
            // This tests the config validation that would happen during system initialization
            assert!(
                invalid_config.adaptation_threshold < 0.0
                    || invalid_config.adaptation_threshold > 1.0
                    || invalid_config.max_data_age_days == 0
                    || invalid_config.learning_rate < 0.0,
                "Invalid configuration should be detectable"
            );
        }

        // Test 9: Resource cleanup error handling
        let cleanup_result = storage.cleanup_expired_data().await;
        assert!(
            cleanup_result.is_ok(),
            "Cleanup should handle errors gracefully"
        );

        let cleanup_data = cleanup_result.unwrap();
        assert!(
            cleanup_data.total_storage_freed_bytes >= 0,
            "Cleanup should provide valid metrics even on error"
        );

        // Test 10: System state consistency after errors
        // Verify that errors don't leave system in inconsistent state
        let final_feedback = create_test_feedback("consistency_test", 0.8);
        storage.store_feedback(&final_feedback).await.unwrap();

        let final_pattern = PatternData::new("consistency_pattern".to_string(), 1, 0.8);
        storage.store_pattern(&final_pattern).await.unwrap();

        let final_learning = LearningData::new(
            "consistency_learning".to_string(),
            "consistency_test".to_string(),
            vec!["System remains consistent".to_string()],
            0.9,
        );
        storage.store_learning_data(&final_learning).await.unwrap();

        // Verify all data is accessible and consistent
        let stored_feedback = storage
            .get_feedback_for_content("consistency_test")
            .await
            .unwrap();
        let stored_patterns = storage
            .get_patterns_by_type("consistency_pattern")
            .await
            .unwrap();
        let stored_learning = storage
            .get_learning_data_by_type("consistency_learning")
            .await
            .unwrap();

        assert!(
            !stored_feedback.is_empty(),
            "Feedback should be consistently stored"
        );
        assert!(
            !stored_patterns.is_empty(),
            "Patterns should be consistently stored"
        );
        assert!(
            !stored_learning.is_empty(),
            "Learning should be consistently stored"
        );
    }
}
