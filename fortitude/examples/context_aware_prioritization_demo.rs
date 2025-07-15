// ABOUTME: Demonstration of context-aware priority scoring using classification system
//! This example demonstrates the enhanced priority scoring system with context-aware features.
//! It shows how the prioritization system integrates with the classification system to provide
//! intelligent priority adjustments based on research context, domain, and classification metadata.

use fortitude::proactive::{
    AudiencePriorityAdjustments, ContextAwareScoringConfig, DetectedGap, DevelopmentContext,
    DevelopmentPhase, DomainPriorityWeights, GapType, PrioritizationConfig, PriorityScorer,
    UrgencyPriorityScaling,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🎯 Context-Aware Priority Scoring Demonstration");
    println!("=====================================================");
    println!();

    // Demonstrate different scoring configurations
    let configurations = vec![
        ("Basic Prioritization", create_basic_config()),
        ("Context-Aware Default", create_context_aware_config()),
        ("Security-Focused", create_security_focused_config()),
        ("Beginner-Friendly", create_beginner_friendly_config()),
    ];

    // Create test gaps representing different scenarios
    let test_gaps = create_test_gaps();

    for (config_name, config) in configurations {
        println!("📊 Configuration: {}", config_name);
        println!("-------------------");

        let context = create_development_context();
        let scorer = match PriorityScorer::new(config, context).await {
            Ok(scorer) => scorer,
            Err(e) => {
                println!("❌ Failed to create scorer: {}", e);
                continue;
            }
        };

        // Show configuration details
        println!(
            "Context-aware enabled: {}",
            scorer.is_context_aware_enabled()
        );
        if let Some(ca_config) = scorer.get_context_aware_config() {
            println!(
                "Domain weights: Rust={:.1}, Security={:.1}, AI={:.1}",
                ca_config.domain_weights.rust_multiplier,
                ca_config.domain_weights.security_multiplier,
                ca_config.domain_weights.ai_multiplier
            );
            println!(
                "Audience adjustments: Beginner={:.1}, Advanced={:.1}",
                ca_config.audience_adjustments.beginner_boost,
                ca_config.audience_adjustments.advanced_adjustment
            );
        }
        println!();

        // Score gaps and show results
        let start_time = Instant::now();

        if scorer.is_context_aware_enabled() {
            // Use enhanced scoring
            let mut enhanced_results = Vec::new();
            for gap in &test_gaps {
                match scorer.score_gap_priority_enhanced(gap).await {
                    Ok(breakdown) => enhanced_results.push((gap, breakdown)),
                    Err(e) => println!("⚠️  Failed to score gap: {}", e),
                }
            }

            // Sort by enhanced score and display
            enhanced_results.sort_by(|a, b| {
                b.1.context_enhanced_score
                    .partial_cmp(&a.1.context_enhanced_score)
                    .unwrap()
            });

            println!("Top 5 Priority Gaps (Enhanced):");
            for (i, (gap, breakdown)) in enhanced_results.iter().take(5).enumerate() {
                let priority_level = score_to_priority_level(breakdown.context_enhanced_score);
                println!("  {}. {:?} | Enhanced Score: {:.2} | Base Score: {:.2} | Priority: {} | File: {}:{}",
                       i + 1,
                       gap.gap_type,
                       breakdown.context_enhanced_score,
                       breakdown.base_breakdown.final_score,
                       priority_level,
                       gap.file_path.file_name().unwrap_or_default().to_string_lossy(),
                       gap.line_number);

                if breakdown.classification_available {
                    if let Some(context) = &breakdown.extracted_context {
                        println!(
                            "    📋 Context: Domain={}, Audience={}, Urgency={}",
                            context.technical_domain.display_name(),
                            context.audience_level.display_name(),
                            context.urgency_level.display_name()
                        );
                    }
                    println!("    🔧 Adjustments: Domain={:.1}x, Audience={:.1}x, Urgency={:.1}x, Confidence={:.2}",
                           breakdown.domain_adjustment,
                           breakdown.audience_adjustment,
                           breakdown.urgency_adjustment,
                           breakdown.confidence_weighting);
                }

                if breakdown.used_graceful_degradation {
                    println!("    ⚠️  Used graceful degradation");
                }
            }
        } else {
            // Use basic scoring
            let mut basic_results = Vec::new();
            for gap in &test_gaps {
                match scorer.score_gap_priority(gap).await {
                    Ok(breakdown) => basic_results.push((gap, breakdown)),
                    Err(e) => println!("⚠️  Failed to score gap: {}", e),
                }
            }

            // Sort by score and display
            basic_results.sort_by(|a, b| b.1.final_score.partial_cmp(&a.1.final_score).unwrap());

            println!("Top 5 Priority Gaps (Basic):");
            for (i, (gap, breakdown)) in basic_results.iter().take(5).enumerate() {
                println!(
                    "  {}. {:?} | Score: {:.2} | Priority: {} | File: {}:{}",
                    i + 1,
                    gap.gap_type,
                    breakdown.final_score,
                    breakdown.priority_level.to_string(),
                    gap.file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    gap.line_number
                );

                println!(
                    "    📊 Breakdown: Type={:.1}, Recency={:.1}, Impact={:.1}, Context={:.1}",
                    breakdown.gap_type_score,
                    breakdown.recency_score,
                    breakdown.impact_score,
                    breakdown.context_score
                );
            }
        }

        let duration = start_time.elapsed();
        println!("⏱️  Scoring completed in {:?}", duration);
        println!();
    }

    // Performance comparison
    println!("🏃 Performance Comparison");
    println!("=========================");

    let basic_scorer = PriorityScorer::new(
        PrioritizationConfig::default(),
        DevelopmentContext::default(),
    )
    .await?;

    let context_aware_scorer = PriorityScorer::new(
        PrioritizationConfig::with_context_aware_scoring(),
        DevelopmentContext::default(),
    )
    .await?;

    // Benchmark basic scoring
    let start = Instant::now();
    let basic_results = basic_scorer.score_gaps_batch(&test_gaps).await?;
    let basic_duration = start.elapsed();

    // Benchmark enhanced scoring
    let start = Instant::now();
    let enhanced_results = context_aware_scorer
        .score_gaps_batch_enhanced(&test_gaps)
        .await?;
    let enhanced_duration = start.elapsed();

    println!(
        "Basic scoring: {} gaps in {:?} ({:.1}µs per gap)",
        basic_results.len(),
        basic_duration,
        basic_duration.as_micros() as f64 / basic_results.len() as f64
    );

    println!(
        "Enhanced scoring: {} gaps in {:?} ({:.1}µs per gap)",
        enhanced_results.len(),
        enhanced_duration,
        enhanced_duration.as_micros() as f64 / enhanced_results.len() as f64
    );

    let overhead = enhanced_duration.saturating_sub(basic_duration);
    println!(
        "Context-aware overhead: {:?} ({:.1}% increase)",
        overhead,
        (overhead.as_micros() as f64 / basic_duration.as_micros() as f64) * 100.0
    );

    println!();
    println!("✅ Context-aware priority scoring demonstration completed!");

    Ok(())
}

fn create_basic_config() -> PrioritizationConfig {
    PrioritizationConfig::default()
}

fn create_context_aware_config() -> PrioritizationConfig {
    PrioritizationConfig::with_context_aware_scoring()
}

fn create_security_focused_config() -> PrioritizationConfig {
    let mut context_config = ContextAwareScoringConfig::default();

    // Boost security domain significantly
    context_config.domain_weights.security_multiplier = 2.0;
    context_config.domain_weights.rust_multiplier = 1.3;

    // Higher urgency scaling for immediate needs
    context_config.urgency_scaling.immediate_multiplier = 2.5;

    // Increased confidence weighting
    context_config.confidence_weight = 0.3;

    PrioritizationConfig::with_custom_context_aware(context_config)
}

fn create_beginner_friendly_config() -> PrioritizationConfig {
    let mut context_config = ContextAwareScoringConfig::default();

    // Significantly boost beginner content
    context_config.audience_adjustments.beginner_boost = 1.5;
    context_config.audience_adjustments.advanced_adjustment = 0.7;

    // Boost general domain (more foundational)
    context_config.domain_weights.general_multiplier = 1.2;

    // Lower confidence threshold to include more marginal content
    context_config.min_classification_confidence = 0.2;

    PrioritizationConfig::with_custom_context_aware(context_config)
}

fn create_development_context() -> DevelopmentContext {
    let mut context = DevelopmentContext::default();
    context.phase = DevelopmentPhase::Development;
    context.is_public_api = true;
    context.team_size = 5;
    context.performance_critical = true;

    // Add custom boosts for certain gap types
    context
        .custom_boosts
        .insert(GapType::ApiDocumentationGap, 1.3);
    context
        .custom_boosts
        .insert(GapType::UndocumentedTechnology, 1.2);

    context
}

fn create_test_gaps() -> Vec<DetectedGap> {
    vec![
        // High-priority security gap
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/security/auth.rs"),
            line_number: 45,
            column_number: Some(8),
            context: "// TODO: URGENT - Implement proper password hashing with bcrypt".to_string(),
            description: "Critical security vulnerability: passwords stored in plaintext. Immediate implementation of secure hashing required.".to_string(),
            confidence: 0.95,
            priority: 9,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "function".to_string());
                map.insert("security_critical".to_string(), "true".to_string());
                map
            },
        },

        // API documentation gap
        DetectedGap {
            gap_type: GapType::ApiDocumentationGap,
            file_path: PathBuf::from("src/api/endpoints.rs"),
            line_number: 120,
            column_number: Some(5),
            context: "pub async fn create_user(user_data: UserData) -> Result<User, ApiError>".to_string(),
            description: "Public API endpoint lacks documentation. Users need clear examples and parameter descriptions.".to_string(),
            confidence: 0.88,
            priority: 8,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "function".to_string());
                map.insert("visibility".to_string(), "public".to_string());
                map.insert("api_endpoint".to_string(), "true".to_string());
                map
            },
        },

        // Beginner-level Rust concept
        DetectedGap {
            gap_type: GapType::MissingDocumentation,
            file_path: PathBuf::from("src/core/ownership.rs"),
            line_number: 30,
            column_number: Some(1),
            context: "struct OwnershipExample { data: Vec<String> }".to_string(),
            description: "Complex ownership pattern needs beginner-friendly explanation with examples of borrowing and moving.".to_string(),
            confidence: 0.75,
            priority: 6,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "struct".to_string());
                map.insert("difficulty".to_string(), "beginner".to_string());
                map.insert("concept".to_string(), "ownership".to_string());
                map
            },
        },

        // AI/ML undocumented technology
        DetectedGap {
            gap_type: GapType::UndocumentedTechnology,
            file_path: PathBuf::from("src/ml/neural_network.rs"),
            line_number: 88,
            column_number: Some(12),
            context: "impl BackpropagationAlgorithm for CustomNN".to_string(),
            description: "Advanced machine learning implementation using custom backpropagation algorithm lacks technical documentation.".to_string(),
            confidence: 0.92,
            priority: 7,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "impl".to_string());
                map.insert("domain".to_string(), "machine_learning".to_string());
                map.insert("complexity".to_string(), "advanced".to_string());
                map
            },
        },

        // Configuration gap for DevOps
        DetectedGap {
            gap_type: GapType::ConfigurationGap,
            file_path: PathBuf::from("deployment/docker-compose.yml"),
            line_number: 25,
            column_number: None,
            context: "environment:\n  - DATABASE_URL=\n  - API_KEY=".to_string(),
            description: "Docker configuration missing environment variable documentation and default values.".to_string(),
            confidence: 0.82,
            priority: 5,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "configuration".to_string());
                map.insert("environment".to_string(), "docker".to_string());
                map.insert("deployment".to_string(), "true".to_string());
                map
            },
        },

        // Web development TODO
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/web/handlers.rs"),
            line_number: 156,
            column_number: Some(4),
            context: "// TODO: Add rate limiting to prevent API abuse".to_string(),
            description: "Web API needs rate limiting implementation to prevent abuse and ensure service stability.".to_string(),
            confidence: 0.87,
            priority: 7,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "function".to_string());
                map.insert("web_security".to_string(), "true".to_string());
                map.insert("performance".to_string(), "true".to_string());
                map
            },
        },

        // Database documentation gap
        DetectedGap {
            gap_type: GapType::MissingDocumentation,
            file_path: PathBuf::from("src/database/migrations.rs"),
            line_number: 67,
            column_number: Some(1),
            context: "fn migrate_user_table_v2() -> Result<(), MigrationError>".to_string(),
            description: "Database migration function needs documentation explaining schema changes and rollback procedures.".to_string(),
            confidence: 0.79,
            priority: 6,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "function".to_string());
                map.insert("database".to_string(), "true".to_string());
                map.insert("migration".to_string(), "true".to_string());
                map
            },
        },

        // General systems programming
        DetectedGap {
            gap_type: GapType::UndocumentedTechnology,
            file_path: PathBuf::from("src/memory/allocator.rs"),
            line_number: 203,
            column_number: Some(8),
            context: "unsafe fn custom_alloc(size: usize) -> *mut u8".to_string(),
            description: "Custom memory allocator implementation needs detailed safety documentation and usage guidelines.".to_string(),
            confidence: 0.91,
            priority: 8,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "function".to_string());
                map.insert("unsafe".to_string(), "true".to_string());
                map.insert("systems".to_string(), "true".to_string());
                map.insert("complexity".to_string(), "expert".to_string());
                map
            },
        },
    ]
}

fn score_to_priority_level(score: f64) -> &'static str {
    match score {
        s if s >= 8.5 => "CRITICAL",
        s if s >= 7.0 => "HIGH",
        s if s >= 4.0 => "MEDIUM",
        _ => "LOW",
    }
}
