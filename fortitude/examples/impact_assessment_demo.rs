// Example demonstrating the impact assessment feature for knowledge gap prioritization

use fortitude::proactive::{
    DetectedGap, DevelopmentContext, DevelopmentPhase, GapType, ImpactAssessor,
    PrioritizationConfig, PriorityScorer,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🎯 Impact Assessment Demo - Analyzing Code Usage Patterns for Priority Scoring\n");

    // Create various types of gaps to demonstrate impact assessment
    let gaps = create_demo_gaps();

    // Demo 1: Basic impact assessment
    println!("📊 Demo 1: Basic Impact Assessment");
    let impact_assessor = ImpactAssessor::with_defaults().await?;

    for gap in &gaps[0..3] {
        let assessment = impact_assessor.assess_gap_impact(gap).await?;
        print_impact_assessment(&gap, &assessment);
    }

    // Demo 2: Priority scoring with impact assessment enabled
    println!("\n🎯 Demo 2: Priority Scoring with Impact Assessment");
    let config = PrioritizationConfig::with_impact_assessment();
    let context = create_demo_context();
    let priority_scorer = PriorityScorer::new(config, context).await?;

    for gap in &gaps {
        let breakdown = priority_scorer.score_gap_priority(gap).await?;
        print_priority_breakdown(&gap, &breakdown);
    }

    // Demo 3: Full enhancement with both context-aware scoring and impact assessment
    println!("\n🚀 Demo 3: Full Enhancement - Context-Aware + Impact Assessment");
    let enhanced_config = PrioritizationConfig::with_full_enhancement();
    let enhanced_context = create_enhanced_context();
    let enhanced_scorer = PriorityScorer::new(enhanced_config, enhanced_context).await?;

    for gap in &gaps[0..2] {
        let breakdown = enhanced_scorer.score_gap_priority_enhanced(gap).await?;
        print_enhanced_breakdown(&gap, &breakdown);
    }

    // Demo 4: Batch processing with impact assessment
    println!("\n⚡ Demo 4: Batch Impact Assessment");
    let start = std::time::Instant::now();
    let batch_results = impact_assessor.assess_gaps_batch(&gaps).await?;
    let duration = start.elapsed();

    println!("Processed {} gaps in {:?}", batch_results.len(), duration);
    for (gap, result) in gaps.iter().zip(batch_results.iter()) {
        println!(
            "  {} -> Impact: {:.2} (Primary factor: {})",
            gap_type_to_string(&gap.gap_type),
            result.final_impact_score,
            result.primary_impact_factor()
        );
    }

    // Demo 5: Performance comparison
    println!("\n⏱️  Demo 5: Performance Comparison");
    compare_performance().await?;

    println!("\n✅ Impact Assessment Demo Complete!");
    println!("Key benefits demonstrated:");
    println!("  • Sophisticated code usage pattern analysis");
    println!("  • Dependency impact and API visibility assessment");
    println!("  • Development activity and team impact analysis");
    println!("  • Enhanced priority scoring accuracy");
    println!("  • Efficient batch processing with caching");

    Ok(())
}

fn create_demo_gaps() -> Vec<DetectedGap> {
    vec![
        DetectedGap {
            gap_type: GapType::ApiDocumentationGap,
            file_path: PathBuf::from("src/lib.rs"),
            line_number: 42,
            column_number: Some(1),
            context: "pub fn critical_api_function() -> Result<Data, Error>".to_string(),
            description: "Public API function missing documentation".to_string(),
            confidence: 0.95,
            priority: 9,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("item_type".to_string(), "function".to_string());
                meta.insert("visibility".to_string(), "public".to_string());
                meta
            },
        },
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/main.rs"),
            line_number: 156,
            column_number: Some(8),
            context: "// TODO: Implement proper error handling for network requests".to_string(),
            description: "Error handling implementation needed".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::UndocumentedTechnology,
            file_path: PathBuf::from("src/database/mod.rs"),
            line_number: 23,
            column_number: Some(1),
            context: "use advanced_orm::query_builder::ComplexQuery;".to_string(),
            description: "Advanced ORM usage not documented".to_string(),
            confidence: 0.8,
            priority: 8,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("technology".to_string(), "advanced_orm".to_string());
                meta.insert("complexity".to_string(), "high".to_string());
                meta
            },
        },
        DetectedGap {
            gap_type: GapType::ConfigurationGap,
            file_path: PathBuf::from("config/production.yaml"),
            line_number: 67,
            column_number: None,
            context: "database_pool_size: 20".to_string(),
            description: "Database configuration not documented".to_string(),
            confidence: 0.85,
            priority: 6,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::MissingDocumentation,
            file_path: PathBuf::from("tests/integration_tests.rs"),
            line_number: 89,
            column_number: Some(1),
            context: "fn test_complex_scenario()".to_string(),
            description: "Test function missing documentation".to_string(),
            confidence: 0.7,
            priority: 4,
            metadata: HashMap::new(),
        },
    ]
}

fn create_demo_context() -> DevelopmentContext {
    DevelopmentContext {
        phase: DevelopmentPhase::Development,
        has_urgent_deadlines: false,
        team_size: 5,
        is_public_api: true,
        performance_critical: true,
        custom_boosts: HashMap::new(),
    }
}

fn create_enhanced_context() -> DevelopmentContext {
    let mut custom_boosts = HashMap::new();
    custom_boosts.insert(GapType::ApiDocumentationGap, 1.5);
    custom_boosts.insert(GapType::UndocumentedTechnology, 1.3);

    DevelopmentContext {
        phase: DevelopmentPhase::PreProduction,
        has_urgent_deadlines: true,
        team_size: 8,
        is_public_api: true,
        performance_critical: true,
        custom_boosts,
    }
}

fn print_impact_assessment(
    gap: &DetectedGap,
    assessment: &fortitude::proactive::ImpactAssessmentResult,
) {
    println!(
        "Gap: {} ({}:{})",
        gap_type_to_string(&gap.gap_type),
        gap.file_path.display(),
        gap.line_number
    );
    println!(
        "  Final Impact Score: {:.2}/10.0",
        assessment.final_impact_score
    );
    println!(
        "  Primary Impact Factor: {}",
        assessment.primary_impact_factor()
    );
    println!("  Overall Confidence: {:.2}", assessment.overall_confidence);
    println!("  Component Scores:");
    println!(
        "    Usage Frequency: {:.2}",
        assessment.usage_analysis.usage_frequency_score
    );
    println!(
        "    Dependency Impact: {:.2}",
        assessment.dependency_analysis.dependency_impact_score
    );
    println!(
        "    API Visibility: {:.2}",
        assessment.api_visibility_analysis.api_visibility_score
    );
    println!(
        "    Development Activity: {:.2}",
        assessment.activity_analysis.activity_impact_score
    );
    println!(
        "    Team Impact: {:.2}",
        assessment.team_analysis.team_impact_score
    );
    println!("  Analysis Duration: {:?}", assessment.analysis_duration);
    println!();
}

fn print_priority_breakdown(
    gap: &DetectedGap,
    breakdown: &fortitude::proactive::PriorityScoreBreakdown,
) {
    println!(
        "Gap: {} -> Priority: {:.2} ({})",
        gap_type_to_string(&gap.gap_type),
        breakdown.final_score,
        priority_to_string(&breakdown.priority_level)
    );

    if let Some(impact_assessment) = &breakdown.impact_assessment {
        println!("  Enhanced with Impact Assessment:");
        println!(
            "    Impact Score: {:.2} (Primary: {})",
            impact_assessment.final_impact_score,
            impact_assessment.primary_impact_factor()
        );
    } else {
        println!("  Basic Impact Score: {:.2}", breakdown.impact_score);
    }

    println!("  Confidence: {:.2}", breakdown.confidence);
    println!();
}

fn print_enhanced_breakdown(
    gap: &DetectedGap,
    breakdown: &fortitude::proactive::ContextAwarePriorityBreakdown,
) {
    println!(
        "Gap: {} -> Enhanced Priority: {:.2}",
        gap_type_to_string(&gap.gap_type),
        breakdown.context_enhanced_score
    );

    println!("  Base Score: {:.2}", breakdown.base_breakdown.final_score);
    println!("  Context Adjustments:");
    println!("    Domain: {:.2}x", breakdown.domain_adjustment);
    println!("    Audience: {:.2}x", breakdown.audience_adjustment);
    println!("    Urgency: {:.2}x", breakdown.urgency_adjustment);

    if let Some(impact_assessment) = &breakdown.base_breakdown.impact_assessment {
        println!(
            "  Impact Assessment: {:.2} ({})",
            impact_assessment.final_impact_score,
            impact_assessment.primary_impact_factor()
        );
    }

    println!(
        "  Classification Available: {}",
        breakdown.classification_available
    );
    println!("  Processing Time: {:?}", breakdown.context_processing_time);
    println!();
}

async fn compare_performance() -> Result<(), Box<dyn std::error::Error>> {
    let gaps = create_demo_gaps();

    // Basic prioritization
    let basic_config = PrioritizationConfig::default();
    let context = create_demo_context();
    let basic_scorer = PriorityScorer::new(basic_config, context.clone()).await?;

    let start = std::time::Instant::now();
    let _basic_results = basic_scorer.score_gaps_batch(&gaps).await?;
    let basic_duration = start.elapsed();

    // With impact assessment
    let impact_config = PrioritizationConfig::with_impact_assessment();
    let impact_scorer = PriorityScorer::new(impact_config, context).await?;

    let start = std::time::Instant::now();
    let _impact_results = impact_scorer.score_gaps_batch(&gaps).await?;
    let impact_duration = start.elapsed();

    println!("Performance Comparison for {} gaps:", gaps.len());
    println!("  Basic Prioritization: {:?}", basic_duration);
    println!("  With Impact Assessment: {:?}", impact_duration);
    println!(
        "  Overhead: {:?} ({:.1}% increase)",
        impact_duration - basic_duration,
        ((impact_duration.as_nanos() as f64 / basic_duration.as_nanos() as f64) - 1.0) * 100.0
    );

    Ok(())
}

// Helper functions to convert enums to strings (avoiding orphan rule issues)
fn gap_type_to_string(gap_type: &GapType) -> String {
    match gap_type {
        GapType::TodoComment => "TODO Comment".to_string(),
        GapType::MissingDocumentation => "Missing Documentation".to_string(),
        GapType::UndocumentedTechnology => "Undocumented Technology".to_string(),
        GapType::ApiDocumentationGap => "API Documentation Gap".to_string(),
        GapType::ConfigurationGap => "Configuration Gap".to_string(),
    }
}

fn priority_to_string(priority: &fortitude::proactive::TaskPriority) -> String {
    match priority {
        fortitude::proactive::TaskPriority::Low => "Low".to_string(),
        fortitude::proactive::TaskPriority::Medium => "Medium".to_string(),
        fortitude::proactive::TaskPriority::High => "High".to_string(),
        fortitude::proactive::TaskPriority::Critical => "Critical".to_string(),
    }
}
