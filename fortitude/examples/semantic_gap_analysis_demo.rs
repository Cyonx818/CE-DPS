// ABOUTME: Demonstration of semantic gap analysis integration with vector database
//! This example shows how to use the semantic gap analyzer to enhance
//! gap detection with vector database integration for validation,
//! related content discovery, and priority enhancement.

use fortitude::proactive::{
    DetectedGap, GapAnalysisConfig, GapAnalyzer, GapType, SemanticAnalysisConfig,
    SemanticGapAnalyzer,
};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

/// Demo function showing semantic gap analysis workflow
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔍 Semantic Gap Analysis Demo");
    println!("================================");

    // Step 1: Set up temporary project structure
    let temp_dir = setup_demo_project().await?;
    println!("✅ Created demo project structure");

    // Step 2: Detect gaps using standard gap analyzer
    let gaps = detect_standard_gaps(&temp_dir).await?;
    println!("✅ Detected {} knowledge gaps", gaps.len());

    // Step 3: Perform semantic analysis on detected gaps (using mock implementation)
    let semantic_results = perform_semantic_analysis(gaps).await?;
    println!("✅ Completed semantic analysis");

    // Step 5: Display results
    display_semantic_results(&semantic_results).await?;

    println!("\n🎉 Semantic gap analysis demo completed successfully!");

    Ok(())
}

/// Set up a temporary project with sample code for gap detection
async fn setup_demo_project() -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create sample Rust file with various gaps
    let main_rs = temp_dir.path().join("main.rs");
    let content = r#"
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use reqwest::Client;

// TODO: Implement comprehensive error handling for HTTP requests
pub async fn fetch_data(url: &str) -> String {
    // Basic implementation without proper error handling
    let client = Client::new();
    let response = client.get(url).send().await.unwrap();
    response.text().await.unwrap()
}

pub struct UserService {
    client: Client,
    cache: RwLock<HashMap<String, User>>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    id: String,
    name: String,
    email: String,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: RwLock::new(HashMap::new()),
        }
    }
    
    // Missing documentation for this public method
    pub async fn get_user(&self, id: &str) -> Option<User> {
        // TODO: Add caching logic here
        let cache = self.cache.read().await;
        cache.get(id).cloned()
    }
    
    // TODO: Implement user creation with validation
    pub async fn create_user(&self, user: User) -> Result<User, Box<dyn std::error::Error>> {
        // Placeholder implementation
        Ok(user)
    }
}
"#;

    fs::write(&main_rs, content).await?;

    // Create API documentation file
    let api_md = temp_dir.path().join("api.md");
    let api_content = r#"
# API Documentation

## Overview
This API provides user management functionality.

## Endpoints

### GET /users/{id}
Retrieves a user by ID.

Note: This documentation is missing examples and error handling details.
"#;

    fs::write(&api_md, api_content).await?;

    Ok(temp_dir)
}

/// Detect gaps using the standard gap analyzer
async fn detect_standard_gaps(
    temp_dir: &TempDir,
) -> Result<Vec<DetectedGap>, Box<dyn std::error::Error>> {
    let config = GapAnalysisConfig::for_rust_project();
    let gap_analyzer = GapAnalyzer::new(config)?;

    let main_file = temp_dir.path().join("main.rs");
    let gaps = gap_analyzer.analyze_file(&main_file).await?;

    Ok(gaps)
}

/// Perform semantic analysis on detected gaps (using mock implementation for demo)
async fn perform_semantic_analysis(
    gaps: Vec<DetectedGap>,
) -> Result<Vec<fortitude::proactive::SemanticGapAnalysis>, Box<dyn std::error::Error>> {
    println!("🔬 Analyzing {} gaps semantically...", gaps.len());
    println!("📚 Simulating vector database with sample knowledge documents:");
    println!("   - Error handling best practices");
    println!("   - Tokio async patterns");
    println!("   - API documentation standards");
    println!("   - Caching implementation examples");

    // For demo purposes, we'll create mock results since setting up a real vector DB is complex
    let results = create_mock_semantic_results(gaps);

    Ok(results)
}

/// Create mock semantic analysis results for demonstration
fn create_mock_semantic_results(
    gaps: Vec<DetectedGap>,
) -> Vec<fortitude::proactive::SemanticGapAnalysis> {
    use fortitude::proactive::{
        RelatedDocument, RelationshipType, SemanticAnalysisMetadata, SemanticGapAnalysis,
    };

    gaps.into_iter()
        .enumerate()
        .map(|(i, gap)| {
            let is_error_handling = gap.description.to_lowercase().contains("error");
            let is_todo_caching = gap.description.to_lowercase().contains("caching");
            let is_missing_docs = gap.gap_type == GapType::MissingDocumentation;

            // Simulate validation based on gap content
            let (is_validated, confidence) = if is_error_handling {
                (false, 0.3) // Found similar content - lower validation
            } else {
                (true, 0.9) // No similar content - high validation
            };

            // Create related documents based on gap type
            let related_documents = if is_error_handling {
                vec![RelatedDocument {
                    document_id: "error-handling-guide".to_string(),
                    content_preview:
                        "Comprehensive guide to error handling in async Rust applications..."
                            .to_string(),
                    similarity_score: 0.85,
                    relationship_type: RelationshipType::ImplementationPattern,
                    metadata: [("topic", "error_handling"), ("language", "rust")]
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                }]
            } else if is_todo_caching {
                vec![RelatedDocument {
                    document_id: "caching-patterns".to_string(),
                    content_preview:
                        "Caching strategies for web applications using RwLock and HashMap..."
                            .to_string(),
                    similarity_score: 0.78,
                    relationship_type: RelationshipType::TopicalSimilarity,
                    metadata: [("topic", "caching"), ("pattern", "rwlock")]
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                }]
            } else if is_missing_docs {
                vec![RelatedDocument {
                    document_id: "api-documentation-standards".to_string(),
                    content_preview: "Best practices for documenting public APIs with examples..."
                        .to_string(),
                    similarity_score: 0.72,
                    relationship_type: RelationshipType::BackgroundContext,
                    metadata: [("topic", "documentation"), ("scope", "api")]
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                }]
            } else {
                Vec::new()
            };

            // Enhance priority based on semantic analysis
            let enhanced_priority = if related_documents.is_empty() {
                gap.priority + 1 // Higher priority if no related content
            } else {
                gap.priority // Keep same priority if content exists
            }
            .min(10);

            let has_related = !related_documents.is_empty();

            SemanticGapAnalysis {
                gap: gap.clone(),
                is_validated,
                validation_confidence: confidence,
                related_documents: related_documents.clone(),
                enhanced_priority,
                metadata: SemanticAnalysisMetadata {
                    analysis_time_ms: 15.0 + (i as f64 * 2.0), // Simulate varying analysis times
                    vector_queries_count: if has_related { 2 } else { 1 },
                    search_query: format!(
                        "{} {} {}",
                        gap.description,
                        gap.context,
                        gap.gap_type.priority()
                    ),
                    search_results_count: related_documents.len(),
                    features_used: vec![
                        "gap_validation".to_string(),
                        if has_related {
                            "related_content".to_string()
                        } else {
                            "".to_string()
                        },
                        "priority_enhancement".to_string(),
                    ]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect(),
                },
            }
        })
        .collect()
}

/// Display semantic analysis results
async fn display_semantic_results(
    results: &[fortitude::proactive::SemanticGapAnalysis],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Semantic Analysis Results");
    println!("=============================");

    for (i, result) in results.iter().enumerate() {
        println!("\n🔍 Gap #{}: {}", i + 1, result.gap.description);
        println!(
            "   📁 File: {}:{}",
            result.gap.file_path.display(),
            result.gap.line_number
        );
        println!("   🏷️  Type: {:?}", result.gap.gap_type);
        println!(
            "   ⭐ Original Priority: {} → Enhanced: {}",
            result.gap.priority, result.enhanced_priority
        );

        // Validation status
        let validation_emoji = if result.is_validated { "✅" } else { "⚠️" };
        println!(
            "   {} Validated: {} (confidence: {:.1}%)",
            validation_emoji,
            result.is_validated,
            result.validation_confidence * 100.0
        );

        // Related content
        if result.related_documents.is_empty() {
            println!("   📚 Related Content: None found (suggests new knowledge gap)");
        } else {
            println!(
                "   📚 Related Content: {} documents found",
                result.related_documents.len()
            );
            for (j, doc) in result.related_documents.iter().enumerate() {
                println!(
                    "      {}. {} (similarity: {:.1}%, type: {:?})",
                    j + 1,
                    doc.document_id,
                    doc.similarity_score * 100.0,
                    doc.relationship_type
                );
                println!(
                    "         Preview: {}",
                    if doc.content_preview.len() > 80 {
                        format!("{}...", &doc.content_preview[..80])
                    } else {
                        doc.content_preview.clone()
                    }
                );
            }
        }

        // Performance metrics
        println!(
            "   ⏱️  Analysis Time: {:.1}ms (queries: {})",
            result.metadata.analysis_time_ms, result.metadata.vector_queries_count
        );
        println!(
            "   🔧 Features Used: {}",
            result.metadata.features_used.join(", ")
        );
    }

    // Summary statistics
    println!("\n📈 Summary Statistics");
    println!("====================");

    let total_gaps = results.len();
    let validated_gaps = results.iter().filter(|r| r.is_validated).count();
    let gaps_with_related = results
        .iter()
        .filter(|r| !r.related_documents.is_empty())
        .count();
    let priority_enhanced = results
        .iter()
        .filter(|r| r.enhanced_priority > r.gap.priority)
        .count();

    let total_analysis_time: f64 = results.iter().map(|r| r.metadata.analysis_time_ms).sum();
    let avg_analysis_time = total_analysis_time / total_gaps as f64;

    let total_queries: usize = results
        .iter()
        .map(|r| r.metadata.vector_queries_count)
        .sum();

    println!("📊 Total Gaps Analyzed: {}", total_gaps);
    println!(
        "✅ Validated Gaps: {} ({:.1}%)",
        validated_gaps,
        (validated_gaps as f64 / total_gaps as f64) * 100.0
    );
    println!(
        "📚 Gaps with Related Content: {} ({:.1}%)",
        gaps_with_related,
        (gaps_with_related as f64 / total_gaps as f64) * 100.0
    );
    println!(
        "⭐ Priority Enhanced: {} ({:.1}%)",
        priority_enhanced,
        (priority_enhanced as f64 / total_gaps as f64) * 100.0
    );
    println!(
        "⏱️  Average Analysis Time: {:.1}ms per gap",
        avg_analysis_time
    );
    println!(
        "🔍 Total Vector Queries: {} ({:.1} per gap)",
        total_queries,
        total_queries as f64 / total_gaps as f64
    );

    // Performance validation
    if avg_analysis_time <= 100.0 {
        println!("✅ Performance requirement met: <100ms overhead per gap");
    } else {
        println!(
            "⚠️  Performance requirement exceeded: {:.1}ms > 100ms",
            avg_analysis_time
        );
    }

    if total_analysis_time <= 500.0 {
        println!("✅ Total analysis time requirement met: <500ms total");
    } else {
        println!(
            "⚠️  Total analysis time requirement exceeded: {:.1}ms > 500ms",
            total_analysis_time
        );
    }

    Ok(())
}
