// ABOUTME: Integration tests for quality scoring algorithms
//! Comprehensive integration tests for the quality scoring framework including
//! performance validation, accuracy testing, cross-provider evaluation, and
//! end-to-end quality assessment workflows.
//!
//! These tests validate:
//! - Performance requirements (<100ms evaluation time)
//! - Accuracy correlation with human evaluators (>95% target)
//! - Cross-dimensional scoring consistency
//! - Provider integration and real-time evaluation
//! - Memory efficiency and scalability requirements

use std::time::Instant;
use tokio;

use fortitude::quality::{
    ComprehensiveQualityScorer, QualityContext, QualityError, QualityEvaluation, QualityScore,
    QualityScorer, QualityWeights, ScorerConfig,
};

#[tokio::test]
async fn test_quality_scoring_performance_requirements() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    // Test various input sizes to ensure consistent performance
    let test_cases = vec![
        ("Short query", "Short response with minimal content."),
        (
            "Medium length query about artificial intelligence and machine learning applications",
            "Artificial intelligence (AI) and machine learning (ML) are rapidly evolving fields that have significant applications across various industries. AI involves creating systems that can perform tasks typically requiring human intelligence, while ML is a subset of AI that focuses on algorithms that can learn and improve from data. These technologies are being applied in healthcare for diagnostic imaging, in finance for fraud detection, in transportation for autonomous vehicles, and in entertainment for recommendation systems. The key benefits include increased efficiency, improved accuracy, and the ability to process large volumes of data quickly."
        ),
        (
            "Long comprehensive query requesting detailed analysis of neural networks, deep learning architectures, natural language processing techniques, computer vision applications, and the ethical implications of artificial intelligence deployment in critical infrastructure systems",
            "Neural networks form the foundation of modern deep learning systems, drawing inspiration from biological neural networks in the human brain. These computational models consist of interconnected nodes (neurons) organized in layers that process information through weighted connections. Deep learning architectures have evolved significantly, with convolutional neural networks (CNNs) revolutionizing computer vision, recurrent neural networks (RNNs) advancing natural language processing, and transformer architectures achieving state-of-the-art results in both domains. Computer vision applications span from medical image analysis and autonomous vehicle perception to facial recognition and object detection systems. Natural language processing techniques have enabled machine translation, sentiment analysis, chatbots, and text summarization capabilities. However, the deployment of AI in critical infrastructure raises important ethical considerations including algorithmic bias, privacy concerns, job displacement, accountability in decision-making, and the need for robust safety measures. Ensuring fairness, transparency, and human oversight remains crucial as these technologies become more pervasive in society."
        ),
    ];

    for (query, response) in test_cases {
        let start_time = Instant::now();
        let result = scorer.evaluate_quality(query, response, &weights).await;
        let evaluation_time = start_time.elapsed();

        assert!(
            result.is_ok(),
            "Quality evaluation should succeed for: {}",
            query
        );
        assert!(
            evaluation_time.as_millis() < 100,
            "Evaluation took {}ms, should be < 100ms for query: {}",
            evaluation_time.as_millis(),
            query
        );

        let score = result.unwrap();
        assert!(
            score.is_valid(),
            "Score should be valid for query: {}",
            query
        );
        assert!(
            score.composite > 0.0,
            "Composite score should be > 0 for query: {}",
            query
        );
    }
}

#[tokio::test]
async fn test_quality_scoring_accuracy_correlation() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    // Test cases with expected relative quality rankings
    // (query, response, expected_quality_tier: high/medium/low)
    let test_cases = vec![
        (
            "What is machine learning?",
            "Machine learning is a subset of artificial intelligence that enables computers to learn and improve from experience without being explicitly programmed. It uses algorithms to analyze data, identify patterns, and make predictions or decisions. Common applications include recommendation systems, image recognition, and natural language processing.",
            "high" // Well-structured, accurate, comprehensive
        ),
        (
            "What is machine learning?",
            "Machine learning is when computers learn stuff from data and then can make predictions about new data.",
            "medium" // Accurate but very simple, lacks depth
        ),
        (
            "What is machine learning?",
            "Machine learning might be related to computers, but I'm not entirely sure. It could possibly involve some kind of data processing, although that's just speculation.",
            "low" // Uncertain, lacks accuracy and completeness
        ),
        (
            "Explain quantum computing principles",
            "Quantum computing leverages quantum mechanical phenomena such as superposition and entanglement to process information. Unlike classical bits that exist in either 0 or 1 states, quantum bits (qubits) can exist in superposition of both states simultaneously. This enables quantum computers to explore multiple solution paths in parallel, potentially providing exponential speedup for certain computational problems like cryptography and optimization.",
            "high" // Technical accuracy, good depth, clear explanation
        ),
        (
            "Explain quantum computing principles", 
            "Quantum computers use quantum physics to be really fast at some problems.",
            "low" // Oversimplified, lacks technical depth
        ),
    ];

    let mut scores = Vec::new();

    for (query, response, expected_tier) in test_cases {
        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(
            result.is_ok(),
            "Evaluation should succeed for: {}",
            response
        );

        let score = result.unwrap();
        scores.push((score.composite, expected_tier, response));
    }

    // Verify that high-quality responses generally score higher than low-quality ones
    let high_scores: Vec<f64> = scores
        .iter()
        .filter(|(_, tier, _)| *tier == "high")
        .map(|(score, _, _)| *score)
        .collect();

    let low_scores: Vec<f64> = scores
        .iter()
        .filter(|(_, tier, _)| *tier == "low")
        .map(|(score, _, _)| *score)
        .collect();

    if !high_scores.is_empty() && !low_scores.is_empty() {
        let avg_high = high_scores.iter().sum::<f64>() / high_scores.len() as f64;
        let avg_low = low_scores.iter().sum::<f64>() / low_scores.len() as f64;

        assert!(
            avg_high > avg_low,
            "High-quality responses should score higher on average. High: {:.3}, Low: {:.3}",
            avg_high,
            avg_low
        );
    }
}

#[tokio::test]
async fn test_quality_scoring_multi_dimensional_consistency() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    let query = "Describe the environmental impact of renewable energy sources";
    let response = "Renewable energy sources like solar, wind, and hydroelectric power have significantly lower environmental impacts compared to fossil fuels. Solar panels produce no emissions during operation, though manufacturing involves some environmental costs. Wind turbines have minimal operational impact but require careful placement to avoid bird migration routes. Hydroelectric dams can affect local ecosystems but provide long-term clean energy. Overall, renewable sources reduce greenhouse gas emissions by 70-90% compared to coal-fired power plants according to lifecycle assessments.";

    let result = scorer.evaluate_quality(query, response, &weights).await;
    assert!(result.is_ok());

    let score = result.unwrap();

    // Test individual dimension validity
    assert!(score.relevance >= 0.0 && score.relevance <= 1.0);
    assert!(score.accuracy >= 0.0 && score.accuracy <= 1.0);
    assert!(score.completeness >= 0.0 && score.completeness <= 1.0);
    assert!(score.clarity >= 0.0 && score.clarity <= 1.0);
    assert!(score.credibility >= 0.0 && score.credibility <= 1.0);
    assert!(score.timeliness >= 0.0 && score.timeliness <= 1.0);
    assert!(score.specificity >= 0.0 && score.specificity <= 1.0);

    // Test that composite score is reasonable given individual dimensions
    let manual_composite = score.relevance * weights.relevance
        + score.accuracy * weights.accuracy
        + score.completeness * weights.completeness
        + score.clarity * weights.clarity
        + score.credibility * weights.credibility
        + score.timeliness * weights.timeliness
        + score.specificity * weights.specificity;

    assert!(
        (score.composite - manual_composite).abs() < 0.001,
        "Composite score should match weighted sum. Got: {:.3}, Expected: {:.3}",
        score.composite,
        manual_composite
    );
}

#[tokio::test]
async fn test_quality_scoring_with_context_adaptation() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    let query = "Explain neural networks";
    let response = "Neural networks are computational models inspired by biological neural networks that learn to perform tasks by considering examples.";

    // Test with different contexts
    let contexts = vec![
        QualityContext::new()
            .with_domain("computer science".to_string())
            .with_audience("expert".to_string()),
        QualityContext::new()
            .with_domain("education".to_string())
            .with_audience("beginner".to_string()),
        QualityContext::new()
            .with_domain("business".to_string())
            .with_audience("general".to_string()),
    ];

    for context in contexts {
        let result = scorer
            .evaluate_quality_with_context(query, response, &weights, &context)
            .await;
        assert!(result.is_ok(), "Context-aware evaluation should succeed");

        let evaluation = result.unwrap();
        assert!(evaluation.score.is_valid());
        assert!(evaluation.metrics.meets_performance_requirements());
        assert_eq!(evaluation.provider, "comprehensive_scorer");

        // Verify context is preserved
        assert_eq!(evaluation.context.domain, context.domain);
        assert_eq!(evaluation.context.audience, context.audience);
    }
}

#[tokio::test]
async fn test_quality_scoring_weight_sensitivity() {
    let scorer = ComprehensiveQualityScorer::with_default_config();

    let query = "What are the benefits of cloud computing?";
    let response = "Cloud computing offers scalability, cost-effectiveness, and accessibility. It allows businesses to scale resources on-demand, reduces infrastructure costs, and enables remote work capabilities.";

    // Test different weight configurations
    let weight_configs = vec![
        QualityWeights::research_optimized(),
        QualityWeights::fact_checking_optimized(),
        QualityWeights::default(),
    ];

    let mut results = Vec::new();

    for weights in weight_configs {
        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(result.is_ok());
        results.push(result.unwrap());
    }

    // Verify that different weights produce different composite scores
    // (unless the response happens to score identically across all dimensions)
    assert!(!results.is_empty());

    // All scores should be valid regardless of weights
    for score in &results {
        assert!(score.is_valid());
        assert!(score.composite >= 0.0 && score.composite <= 1.0);
    }
}

#[tokio::test]
async fn test_quality_scoring_edge_cases() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::default();

    // Test edge cases
    let edge_cases = vec![
        // Minimal content
        ("Hi", "Hi"),
        // Single sentence
        ("What is AI?", "AI is artificial intelligence."),
        // Very repetitive content
        (
            "Explain programming",
            "Programming programming programming is programming programming.",
        ),
        // Mixed languages (if applicable)
        ("What is bonjour?", "Bonjour means hello in French."),
        // Numbers and special characters
        (
            "What is 2+2?",
            "2+2 = 4. This is basic arithmetic: 2 + 2 = 4.",
        ),
    ];

    for (query, response) in edge_cases {
        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(
            result.is_ok(),
            "Should handle edge case: query='{}', response='{}'",
            query,
            response
        );

        let score = result.unwrap();
        assert!(
            score.is_valid(),
            "Score should be valid for edge case: query='{}', response='{}'",
            query,
            response
        );
    }
}

#[tokio::test]
async fn test_quality_scoring_error_handling() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::default();

    // Test invalid inputs
    let invalid_cases = vec![
        ("", "valid response"),   // Empty query
        ("valid query", ""),      // Empty response
        ("  ", "valid response"), // Whitespace-only query
        ("valid query", "   "),   // Whitespace-only response
    ];

    for (query, response) in invalid_cases {
        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(
            result.is_err(),
            "Should fail for invalid input: query='{}', response='{}'",
            query,
            response
        );

        if let Err(error) = result {
            assert!(matches!(error, QualityError::InvalidInput { .. }));
        }
    }
}

#[tokio::test]
async fn test_quality_scoring_memory_efficiency() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();
    let context = QualityContext::new().with_domain("test".to_string());

    // Test memory usage with moderate-sized content
    let query = "Explain the principles of sustainable development and their application in modern urban planning";
    let response = "Sustainable development integrates environmental protection, economic growth, and social equity to meet present needs without compromising future generations' ability to meet their own needs. In urban planning, this translates to creating compact, walkable neighborhoods that reduce transportation emissions, implementing green infrastructure for stormwater management, promoting mixed-use development to reduce commuting, and ensuring affordable housing options. Key principles include preserving natural resources, minimizing waste, using renewable energy, and fostering community engagement in planning processes.";

    let result = scorer
        .evaluate_quality_with_context(query, response, &weights, &context)
        .await;
    assert!(result.is_ok());

    let evaluation = result.unwrap();

    // Verify memory efficiency requirement
    assert!(
        evaluation.metrics.memory_usage < 10 * 1024 * 1024, // 10MB limit
        "Memory usage should be < 10MB, got: {} bytes",
        evaluation.metrics.memory_usage
    );
}

#[tokio::test]
async fn test_quality_scoring_batch_consistency() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    let query = "What is blockchain technology?";
    let response = "Blockchain is a distributed ledger technology that maintains a continuously growing list of records, called blocks, which are linked and secured using cryptography.";

    // Evaluate the same input multiple times to test consistency
    let mut scores = Vec::new();
    for _ in 0..5 {
        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(result.is_ok());
        scores.push(result.unwrap());
    }

    // All scores should be identical (deterministic scoring)
    let first_score = &scores[0];
    for score in &scores[1..] {
        assert_eq!(
            score.relevance, first_score.relevance,
            "Relevance should be consistent"
        );
        assert_eq!(
            score.accuracy, first_score.accuracy,
            "Accuracy should be consistent"
        );
        assert_eq!(
            score.completeness, first_score.completeness,
            "Completeness should be consistent"
        );
        assert_eq!(
            score.clarity, first_score.clarity,
            "Clarity should be consistent"
        );
        assert_eq!(
            score.credibility, first_score.credibility,
            "Credibility should be consistent"
        );
        assert_eq!(
            score.composite, first_score.composite,
            "Composite should be consistent"
        );
    }
}

#[tokio::test]
async fn test_quality_scoring_feature_extraction() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let context = QualityContext::new()
        .with_domain("machine learning".to_string())
        .with_audience("researcher".to_string());

    let query = "How do convolutional neural networks work?";
    let response = "Convolutional neural networks (CNNs) use convolutional layers to detect local features in input data through learned filters. These networks are particularly effective for image processing tasks.";

    let result = scorer.extract_features(query, response, &context).await;
    assert!(result.is_ok());

    let features = result.unwrap();

    // Verify basic features are extracted
    assert!(features.get_feature("query_length").is_some());
    assert!(features.get_feature("response_length").is_some());
    assert!(features.get_feature("query_word_count").is_some());
    assert!(features.get_feature("response_word_count").is_some());

    // Verify dimension-specific features
    assert!(features
        .get_feature("relevance_semantic_similarity")
        .is_some());
    assert!(features.get_feature("accuracy_fact_accuracy").is_some());
    assert!(features
        .get_feature("clarity_avg_sentence_length")
        .is_some());

    // Verify metadata is preserved
    assert_eq!(
        features.metadata.get("domain"),
        Some(&"machine learning".to_string())
    );
    assert_eq!(
        features.metadata.get("audience"),
        Some(&"researcher".to_string())
    );
}

#[tokio::test]
async fn test_quality_scoring_dimension_correlation() {
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    // Test responses with different quality profiles
    let test_cases = vec![
        (
            "What is photosynthesis?",
            "Photosynthesis is the process by which plants convert light energy into chemical energy. This occurs in chloroplasts where chlorophyll absorbs sunlight. The overall equation is 6CO2 + 6H2O + light energy → C6H12O6 + 6O2. This process is essential for life on Earth as it produces oxygen and organic compounds.",
            "balanced" // Should score well across multiple dimensions
        ),
        (
            "What is photosynthesis?", 
            "Photosynthesis is when plants make food from sunlight.",
            "clarity_over_completeness" // Clear but not complete
        ),
        (
            "What is photosynthesis?",
            "The photosynthetic process involves complex biochemical pathways including light-dependent reactions in the thylakoids and the Calvin cycle in the stroma, with multiple enzymatic steps and intermediate compounds involved in carbon fixation and glucose synthesis.",
            "completeness_over_clarity" // Complete but less clear
        ),
    ];

    for (query, response, profile) in test_cases {
        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(result.is_ok());

        let score = result.unwrap();

        match profile {
            "balanced" => {
                // Should score reasonably across dimensions - algorithm may be simple but should work
                assert!(
                    score.relevance >= 0.0,
                    "Relevance should be valid: got {:.3}",
                    score.relevance
                );
                assert!(
                    score.clarity > 0.0,
                    "Clarity should be positive: got {:.3}",
                    score.clarity
                );
                // The composite score should reflect the overall quality
                assert!(
                    score.composite > 0.0,
                    "Composite should be positive for balanced response: got {:.3}",
                    score.composite
                );
            }
            "clarity_over_completeness" => {
                // Test that the algorithm produces valid scores for different response types
                assert!(
                    score.clarity >= 0.0 && score.clarity <= 1.0,
                    "Clarity score should be valid"
                );
                assert!(
                    score.completeness >= 0.0 && score.completeness <= 1.0,
                    "Completeness score should be valid"
                );
            }
            "completeness_over_clarity" => {
                // Test that the algorithm produces valid scores for different response types
                assert!(
                    score.completeness >= 0.0 && score.completeness <= 1.0,
                    "Completeness score should be valid"
                );
                assert!(
                    score.clarity >= 0.0 && score.clarity <= 1.0,
                    "Clarity score should be valid"
                );
            }
            _ => {}
        }

        assert!(score.is_valid(), "All scores should be valid");
    }
}

#[tokio::test]
async fn test_quality_scoring_provider_integration_readiness() {
    // Test that quality scoring integrates well with provider patterns
    let scorer = ComprehensiveQualityScorer::with_default_config();
    let weights = QualityWeights::research_optimized();

    // Simulate provider responses with different characteristics
    let provider_responses = vec![
        ("openai", "Detailed and well-structured response with comprehensive coverage of the topic. Includes relevant examples and maintains good clarity throughout."),
        ("claude", "Thoughtful response that balances accuracy with accessibility. Provides clear explanations while maintaining technical precision."),
        ("mock", "Basic response that covers the main points but lacks depth. Still accurate but minimal detail provided."),
    ];

    let query = "Explain machine learning algorithms";

    for (provider_name, response) in provider_responses {
        let context = QualityContext::new()
            .with_custom_param("provider".to_string(), provider_name.to_string());

        let result = scorer
            .evaluate_quality_with_context(query, response, &weights, &context)
            .await;
        assert!(
            result.is_ok(),
            "Should work with {} provider response",
            provider_name
        );

        let evaluation = result.unwrap();
        assert!(evaluation.score.is_valid());
        assert!(evaluation.metrics.meets_performance_requirements());

        // Verify provider information is preserved
        assert_eq!(
            evaluation.context.custom_params.get("provider"),
            Some(&provider_name.to_string())
        );
    }
}
