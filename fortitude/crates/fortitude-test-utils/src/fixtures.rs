// ABOUTME: Test fixtures for the Fortitude research system
use chrono::Utc;
use fortitude_types::*;
use std::collections::HashMap;

/// Create a sample classification result for testing
pub fn sample_classification_result(
    research_type: ResearchType,
    confidence: f64,
) -> ClassificationResult {
    let candidates = vec![ClassificationCandidate::new(
        research_type.clone(),
        confidence,
        vec!["test".to_string(), "sample".to_string()],
        1,
    )];

    ClassificationResult::new(
        research_type,
        confidence,
        vec!["test".to_string(), "sample".to_string()],
        1,
        candidates,
    )
}

/// Create a sample classified request for testing
pub fn sample_classified_request(query: &str, research_type: ResearchType) -> ClassifiedRequest {
    ClassifiedRequest::new(
        query.to_string(),
        research_type,
        sample_audience_context(),
        sample_domain_context(),
        0.85,
        vec!["test".to_string(), "sample".to_string()],
    )
}

/// Create a sample audience context for testing
pub fn sample_audience_context() -> AudienceContext {
    AudienceContext {
        level: "intermediate".to_string(),
        domain: "rust".to_string(),
        format: "markdown".to_string(),
    }
}

/// Create a sample domain context for testing
pub fn sample_domain_context() -> DomainContext {
    DomainContext {
        technology: "rust".to_string(),
        project_type: "cli".to_string(),
        frameworks: vec!["clap".to_string(), "tokio".to_string()],
        tags: vec!["async".to_string(), "testing".to_string()],
    }
}

/// Create a sample research metadata for testing
pub fn sample_research_metadata() -> ResearchMetadata {
    ResearchMetadata {
        completed_at: Utc::now(),
        processing_time_ms: 1500,
        sources_consulted: vec![
            "documentation".to_string(),
            "examples".to_string(),
            "reference".to_string(),
        ],
        quality_score: 0.9,
        cache_key: "sample-cache-key".to_string(),
        tags: HashMap::from([
            ("complexity".to_string(), "medium".to_string()),
            ("language".to_string(), "rust".to_string()),
        ]),
    }
}

/// Create a sample evidence for testing
pub fn sample_evidence() -> Evidence {
    Evidence {
        source: "Official Documentation".to_string(),
        content: "This is sample evidence content that supports the research result.".to_string(),
        relevance: 0.9,
        evidence_type: "documentation".to_string(),
    }
}

/// Create a sample detail for testing
pub fn sample_detail() -> Detail {
    Detail {
        category: "implementation".to_string(),
        content: "This is a sample implementation detail with step-by-step instructions."
            .to_string(),
        priority: "high".to_string(),
        prerequisites: vec!["rust".to_string(), "cargo".to_string()],
    }
}

/// Create a complete sample research result for testing
pub fn sample_research_result(query: &str, research_type: ResearchType) -> ResearchResult {
    let request = sample_classified_request(query, research_type);
    let metadata = sample_research_metadata();

    let immediate_answer = match request.research_type {
        ResearchType::Decision => format!("Decision guidance for: {query}"),
        ResearchType::Implementation => format!("Implementation guide for: {query}"),
        ResearchType::Troubleshooting => format!("Troubleshooting steps for: {query}"),
        ResearchType::Learning => format!("Learning material for: {query}"),
        ResearchType::Validation => format!("Validation approach for: {query}"),
    };

    ResearchResult::new(
        request,
        immediate_answer,
        vec![sample_evidence()],
        vec![sample_detail()],
        metadata,
    )
}

/// Create a sample cache entry for testing
pub fn sample_cache_entry() -> CacheEntry {
    CacheEntry::new(
        "sample-cache-key".to_string(),
        "/tmp/sample.json".into(),
        ResearchType::Learning,
        "Sample query".to_string(),
        1024,
        "hash123".to_string(),
        3600,
    )
}

/// Create a sample storage config for testing
pub fn sample_storage_config() -> StorageConfig {
    StorageConfig {
        base_path: "/tmp/fortitude-test".into(),
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 10 * 1024 * 1024, // 10MB
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    }
}

/// Create a sample classification config for testing
pub fn sample_classification_config() -> ClassificationConfig {
    ClassificationConfig {
        default_threshold: 0.7,
        fallback_type: ResearchType::Learning,
        enable_fuzzy_matching: false,
        max_candidates: 5,
    }
}

/// Create a sample search query for testing
pub fn sample_search_query() -> SearchQuery {
    SearchQuery::new("rust async programming".to_string())
        .with_research_type(ResearchType::Implementation)
        .with_tags(vec!["rust".to_string(), "async".to_string()])
        .with_min_quality(0.8)
        .with_limit(10)
        .with_offset(0)
}

/// Create a sample index entry for testing
pub fn sample_index_entry() -> IndexEntry {
    IndexEntry::new(
        "sample-cache-key".to_string(),
        ResearchType::Implementation,
        "How to implement async in Rust?".to_string(),
        "Async programming in Rust involves using async/await syntax...".to_string(),
        vec!["async".to_string(), "rust".to_string(), "tokio".to_string()],
        vec!["programming".to_string(), "concurrency".to_string()],
        0.9,
    )
}

/// Create a sample search result for testing
pub fn sample_search_result() -> SearchResult {
    SearchResult::new(
        sample_index_entry(),
        0.95,
        vec!["async".to_string(), "rust".to_string()],
        "...async programming in Rust involves...".to_string(),
    )
}

/// Create multiple sample research results for different research types
pub fn sample_research_results() -> Vec<ResearchResult> {
    vec![
        sample_research_result("How to choose a web framework?", ResearchType::Decision),
        sample_research_result("How to implement OAuth?", ResearchType::Implementation),
        sample_research_result("Why is my code crashing?", ResearchType::Troubleshooting),
        sample_research_result("What is async programming?", ResearchType::Learning),
        sample_research_result("How to test async code?", ResearchType::Validation),
    ]
}

/// Create a set of sample classification rules
pub fn sample_classification_rules() -> Vec<ClassificationRule> {
    vec![
        ClassificationRule::new(
            ResearchType::Decision,
            HashMap::from([
                ("choose".to_string(), 1.0),
                ("decide".to_string(), 0.9),
                ("select".to_string(), 0.8),
            ]),
            0.6,
            1,
        ),
        ClassificationRule::new(
            ResearchType::Implementation,
            HashMap::from([
                ("implement".to_string(), 1.0),
                ("build".to_string(), 0.9),
                ("create".to_string(), 0.8),
            ]),
            0.6,
            1,
        ),
        ClassificationRule::new(
            ResearchType::Troubleshooting,
            HashMap::from([
                ("error".to_string(), 1.0),
                ("bug".to_string(), 0.9),
                ("fix".to_string(), 0.8),
            ]),
            0.6,
            2,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_classification_result() {
        let result = sample_classification_result(ResearchType::Learning, 0.8);
        assert_eq!(result.research_type, ResearchType::Learning);
        assert_eq!(result.confidence, 0.8);
        assert!(!result.matched_keywords.is_empty());
    }

    #[test]
    fn test_sample_classified_request() {
        let request = sample_classified_request("test query", ResearchType::Implementation);
        assert_eq!(request.original_query, "test query");
        assert_eq!(request.research_type, ResearchType::Implementation);
        assert!(request.confidence > 0.0);
    }

    #[test]
    fn test_sample_research_result() {
        let result = sample_research_result("test query", ResearchType::Validation);
        assert_eq!(result.request.original_query, "test query");
        assert_eq!(result.request.research_type, ResearchType::Validation);
        assert!(result.immediate_answer.contains("test query"));
        assert_eq!(result.supporting_evidence.len(), 1);
        assert_eq!(result.implementation_details.len(), 1);
    }

    #[test]
    fn test_sample_cache_entry() {
        let entry = sample_cache_entry();
        assert_eq!(entry.key, "sample-cache-key");
        assert_eq!(entry.research_type, ResearchType::Learning);
        assert_eq!(entry.original_query, "Sample query");
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_sample_search_query() {
        let query = sample_search_query();
        assert_eq!(query.query, "rust async programming");
        assert_eq!(query.research_type, Some(ResearchType::Implementation));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(0));
    }

    #[test]
    fn test_sample_research_results() {
        let results = sample_research_results();
        assert_eq!(results.len(), 5);

        // Verify we have one of each research type
        let types: Vec<_> = results.iter().map(|r| &r.request.research_type).collect();
        assert!(types.contains(&&ResearchType::Decision));
        assert!(types.contains(&&ResearchType::Implementation));
        assert!(types.contains(&&ResearchType::Troubleshooting));
        assert!(types.contains(&&ResearchType::Learning));
        assert!(types.contains(&&ResearchType::Validation));
    }

    #[test]
    fn test_sample_classification_rules() {
        let rules = sample_classification_rules();
        assert_eq!(rules.len(), 3);

        // Verify troubleshooting has higher priority
        let troubleshooting_rule = rules
            .iter()
            .find(|r| r.research_type == ResearchType::Troubleshooting)
            .unwrap();
        assert_eq!(troubleshooting_rule.priority, 2);
    }
}
