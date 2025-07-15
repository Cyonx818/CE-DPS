//! Demonstration of the prioritization algorithms with gap analysis integration
//!
//! This example shows how the priority scorer works with detected gaps from the gap analyzer
//! and integrates with the task scheduling system to create research tasks with intelligent
//! priority ordering.
//!
//! Usage: cargo run --example prioritization_demo

use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, DevelopmentContext,
    DevelopmentPhase, GapAnalysisConfig, GapAnalyzer, GapType, PrioritizationConfig,
    PriorityScorer, ResearchTask, TaskPriority,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use tracing::{info, warn, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 Starting prioritization algorithm demonstration");

    // Create temporary project structure for demo
    let temp_dir = create_demo_project().await?;
    let project_path = temp_dir.path();

    // Configure gap analyzer for the demo project
    let gap_config = GapAnalysisConfig::for_rust_project().with_confidence_threshold(0.5);
    let gap_analyzer = GapAnalyzer::new(gap_config)?;

    // Analyze project for gaps
    info!("📊 Analyzing project for knowledge gaps...");
    let mut all_gaps = Vec::new();

    for entry in std::fs::read_dir(project_path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Ok(gaps) = gap_analyzer.analyze_file(&entry.path()).await {
                all_gaps.extend(gaps);
            }
        }
    }

    info!("Found {} knowledge gaps to prioritize", all_gaps.len());

    // Demonstrate different development contexts
    let contexts = vec![
        ("Prototyping Phase", create_prototyping_context()),
        ("Development Phase", create_development_context()),
        ("Production Phase", create_production_context()),
    ];

    for (phase_name, context) in contexts {
        info!("\n🔍 Analyzing priorities for: {}", phase_name);

        // Create priority scorer for this context
        let config = PrioritizationConfig::default();
        let scorer = PriorityScorer::new(config, context).await?;

        // Score all gaps for priority
        let priority_breakdowns = scorer.score_gaps_batch(&all_gaps).await?;

        // Display top priority gaps
        let mut scored_gaps: Vec<_> = all_gaps.iter().zip(priority_breakdowns.iter()).collect();

        scored_gaps.sort_by(|a, b| b.1.final_score.partial_cmp(&a.1.final_score).unwrap());

        info!("  Top 5 priority gaps:");
        for (gap, breakdown) in scored_gaps.iter().take(5) {
            info!(
                "    {} | Score: {:.2} | Priority: {} | File: {}:{}",
                format_gap_type(&gap.gap_type),
                breakdown.final_score,
                format_task_priority(&breakdown.priority_level),
                gap.file_path.file_name().unwrap().to_string_lossy(),
                gap.line_number
            );

            info!(
                "      Breakdown - Type: {:.1}, Recency: {:.1}, Impact: {:.1}, Context: {:.1}",
                breakdown.gap_type_score,
                breakdown.recency_score,
                breakdown.impact_score,
                breakdown.context_score
            );
        }

        // Show metrics
        let metrics = scorer.get_metrics().await;
        info!(
            "  Scoring metrics - Total: {}, Avg time: {:?}, Cache hits: {}",
            metrics.total_scores_calculated, metrics.average_scoring_time, metrics.cache_hits
        );
    }

    // Demonstrate integration with task scheduler
    info!("\n🎯 Demonstrating task scheduling integration...");
    demonstrate_task_scheduling_integration(&all_gaps).await?;

    // Performance benchmark
    info!("\n⚡ Running performance benchmark...");
    run_performance_benchmark().await?;

    info!("✅ Prioritization demonstration completed successfully!");
    Ok(())
}

async fn create_demo_project() -> Result<TempDir, std::io::Error> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Create main.rs with various gaps
    let main_rs = base_path.join("main.rs");
    fs::write(
        &main_rs,
        r#"
// TODO: Add comprehensive error handling
use std::collections::HashMap;

pub fn main() {
    println!("Hello, world!");
    // FIXME: This function needs proper structure
    let data = process_data();
}

pub fn process_data() -> HashMap<String, i32> {
    HashMap::new()
}

pub struct ApiClient {
    pub url: String,
}

impl ApiClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
"#,
    )
    .await?;

    // Create lib.rs with missing documentation
    let lib_rs = base_path.join("lib.rs");
    fs::write(
        &lib_rs,
        r#"
pub fn undocumented_function() -> Result<String, Box<dyn std::error::Error>> {
    Ok("test".to_string())
}

/// This function has documentation
pub fn documented_function() -> String {
    "documented".to_string()
}

pub struct PublicStruct {
    pub field: String,
}
"#,
    )
    .await?;

    // Create config.toml with configuration gaps
    let config_toml = base_path.join("config.toml");
    fs::write(
        &config_toml,
        r#"
server_port = 8080
database_url = "postgres://localhost"
api_timeout = 30
log_level = "info"
"#,
    )
    .await?;

    Ok(temp_dir)
}

fn create_prototyping_context() -> DevelopmentContext {
    DevelopmentContext {
        phase: DevelopmentPhase::Prototyping,
        has_urgent_deadlines: false,
        team_size: 1,
        is_public_api: false,
        performance_critical: false,
        custom_boosts: HashMap::new(),
    }
}

fn create_development_context() -> DevelopmentContext {
    let mut custom_boosts = HashMap::new();
    custom_boosts.insert(GapType::TodoComment, 1.2); // Boost TODOs during development

    DevelopmentContext {
        phase: DevelopmentPhase::Development,
        has_urgent_deadlines: false,
        team_size: 5,
        is_public_api: true,
        performance_critical: false,
        custom_boosts,
    }
}

fn create_production_context() -> DevelopmentContext {
    let mut custom_boosts = HashMap::new();
    custom_boosts.insert(GapType::ApiDocumentationGap, 1.5); // Critical in production
    custom_boosts.insert(GapType::UndocumentedTechnology, 1.3);

    DevelopmentContext {
        phase: DevelopmentPhase::Production,
        has_urgent_deadlines: true,
        team_size: 10,
        is_public_api: true,
        performance_critical: true,
        custom_boosts,
    }
}

async fn demonstrate_task_scheduling_integration(
    gaps: &[DetectedGap],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create priority scorer
    let config = PrioritizationConfig::default();
    let context = create_production_context();
    let scorer = PriorityScorer::new(config, context).await?;

    // Create background scheduler
    let scheduler_config = BackgroundSchedulerConfig::default();
    let scheduler = BackgroundScheduler::new(scheduler_config).await?;

    info!(
        "Converting {} gaps to prioritized research tasks...",
        gaps.len()
    );

    let mut research_tasks = Vec::new();
    for gap in gaps.iter().take(5) {
        // Limit to 5 for demo
        // Score the gap
        let breakdown = scorer.score_gap_priority(gap).await?;

        // Create research task with prioritized level
        let task = ResearchTask::from_gap(gap.clone(), breakdown.priority_level);

        info!(
            "Created task: {} | Priority: {} | Query: '{}'",
            task.id,
            format_task_priority(&task.priority),
            task.research_query
        );

        research_tasks.push(task);
    }

    // Enqueue tasks (they will be ordered by priority)
    for task in research_tasks {
        scheduler.enqueue(task).await?;
    }

    // Show queue status
    let metrics = scheduler.get_metrics().await;
    info!(
        "Queue metrics - Total tasks: {}, Queue size: {}, Completed: {}",
        metrics.total_tasks_processed, metrics.current_queue_size, metrics.completed_tasks
    );

    Ok(())
}

async fn run_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    // Create large batch of gaps for performance testing
    let mut gaps = Vec::new();
    for i in 0..50 {
        let gap = DetectedGap {
            gap_type: match i % 5 {
                0 => GapType::TodoComment,
                1 => GapType::MissingDocumentation,
                2 => GapType::UndocumentedTechnology,
                3 => GapType::ApiDocumentationGap,
                _ => GapType::ConfigurationGap,
            },
            file_path: PathBuf::from(format!("src/test_{}.rs", i)),
            line_number: 10 + i,
            column_number: Some(5),
            context: format!("Test context {}", i),
            description: format!("Test gap description {}", i),
            confidence: 0.8,
            priority: (i % 10) as u8 + 1,
            metadata: HashMap::new(),
        };
        gaps.push(gap);
    }

    // Performance test with different configurations
    let configs = vec![
        (
            "Performance Optimized",
            PrioritizationConfig::for_performance(),
        ),
        ("Accuracy Optimized", PrioritizationConfig::for_accuracy()),
        ("Default", PrioritizationConfig::default()),
    ];

    for (config_name, config) in configs {
        let context = create_development_context();
        let scorer = PriorityScorer::new(config, context).await?;

        let start_time = Instant::now();
        let _results = scorer.score_gaps_batch(&gaps).await?;
        let duration = start_time.elapsed();

        let metrics = scorer.get_metrics().await;

        info!("{} Config Performance:", config_name);
        info!(
            "  Batch scoring time: {:?} ({:.2}ms per gap)",
            duration,
            duration.as_millis() as f64 / gaps.len() as f64
        );
        info!("  Average scoring time: {:?}", metrics.average_scoring_time);
        info!(
            "  Cache performance: {} hits, {} misses",
            metrics.cache_hits, metrics.cache_misses
        );

        // Verify performance requirement
        if duration.as_millis() <= 100 {
            info!("  ✅ Meets performance requirement (<100ms for 50 gaps)");
        } else {
            warn!(
                "  ⚠️  Exceeds performance requirement: {}ms > 100ms",
                duration.as_millis()
            );
        }
    }

    Ok(())
}

fn format_gap_type(gap_type: &GapType) -> &'static str {
    match gap_type {
        GapType::TodoComment => "TODO",
        GapType::MissingDocumentation => "DOCS",
        GapType::UndocumentedTechnology => "TECH",
        GapType::ApiDocumentationGap => "API ",
        GapType::ConfigurationGap => "CONF",
    }
}

fn format_task_priority(priority: &TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "LOW ",
        TaskPriority::Medium => "MED ",
        TaskPriority::High => "HIGH",
        TaskPriority::Critical => "CRIT",
    }
}
