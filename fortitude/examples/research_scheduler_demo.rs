// ABOUTME: Demonstration of research task scheduling system functionality
//! This example shows how to use the research scheduler with configurable intervals,
//! event-driven and time-based scheduling, and resource-aware scheduling logic.

use fortitude::proactive::{
    DetectedGap, GapType, ResearchScheduler, ResearchSchedulerConfig, ResourceLimits,
    ResourceUsage, TaskPriority,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Research Task Scheduler Demo");
    println!("==============================");

    // Create scheduler configuration with different intervals for priorities
    let mut config = ResearchSchedulerConfig {
        gap_analysis_interval: Duration::from_secs(30),
        time_based_intervals: HashMap::new(),
        gap_type_intervals: HashMap::new(),
        max_concurrent_schedules: 3,
        resource_limits: ResourceLimits {
            max_cpu_percent: 20.0,
            max_memory_percent: 80.0,
        },
        enable_event_driven: true,
        enable_time_based: true,
        scheduler_persistence_file: PathBuf::from("demo_scheduler_state.json"),
    };

    // Configure different intervals for different priorities
    config
        .time_based_intervals
        .insert(TaskPriority::Critical, Duration::from_secs(5));
    config
        .time_based_intervals
        .insert(TaskPriority::High, Duration::from_secs(30));
    config
        .time_based_intervals
        .insert(TaskPriority::Medium, Duration::from_secs(300));
    config
        .time_based_intervals
        .insert(TaskPriority::Low, Duration::from_secs(600));

    // Configure different intervals for gap types
    config
        .gap_type_intervals
        .insert(GapType::TodoComment, Duration::from_secs(60));
    config
        .gap_type_intervals
        .insert(GapType::ApiDocumentationGap, Duration::from_secs(30));
    config
        .gap_type_intervals
        .insert(GapType::UndocumentedTechnology, Duration::from_secs(45));
    config
        .gap_type_intervals
        .insert(GapType::MissingDocumentation, Duration::from_secs(180));
    config
        .gap_type_intervals
        .insert(GapType::ConfigurationGap, Duration::from_secs(300));

    println!("📋 Creating scheduler with configuration:");
    println!(
        "  - Max concurrent schedules: {}",
        config.max_concurrent_schedules
    );
    println!(
        "  - Gap analysis interval: {:?}",
        config.gap_analysis_interval
    );
    println!(
        "  - Priority intervals: {} configured",
        config.time_based_intervals.len()
    );
    println!(
        "  - Gap type intervals: {} configured",
        config.gap_type_intervals.len()
    );

    // Create the scheduler
    let scheduler = ResearchScheduler::new(config).await?;
    println!("✅ Scheduler created successfully");

    // Display initial state
    println!("\n📊 Initial scheduler state:");
    let metrics = scheduler.get_metrics().await;
    println!(
        "  - Total scheduling cycles: {}",
        metrics.total_scheduling_cycles
    );
    println!(
        "  - Event-driven triggers: {}",
        metrics.event_driven_triggers
    );
    println!("  - Time-based triggers: {}", metrics.time_based_triggers);
    println!("  - Running: {}", scheduler.is_running().await);

    // Test configuration retrieval
    println!("\n⚙️  Configured intervals:");
    let intervals = scheduler.get_scheduled_intervals().await;
    for (priority, duration) in intervals {
        println!("  - {:?}: {:?}", priority, duration);
    }

    let gap_intervals = scheduler.get_gap_type_intervals().await;
    println!("\n📝 Gap type intervals:");
    for (gap_type, duration) in gap_intervals {
        println!("  - {:?}: {:?}", gap_type, duration);
    }

    // Test resource-aware scheduling
    println!("\n🔋 Testing resource-aware scheduling:");

    let low_usage = ResourceUsage {
        cpu_percent: 10.0,
        memory_percent: 50.0,
        memory_mb: 500.0,
        network_in_kb: 100.0,
        network_out_kb: 50.0,
        timestamp: chrono::Utc::now(),
    };

    let should_schedule_low = scheduler.should_schedule_now(low_usage).await;
    println!(
        "  - Low resource usage (10% CPU, 50% memory): Should schedule = {}",
        should_schedule_low
    );

    let high_usage = ResourceUsage {
        cpu_percent: 25.0,    // Above 20% limit
        memory_percent: 85.0, // Above 80% limit
        memory_mb: 2000.0,
        network_in_kb: 1000.0,
        network_out_kb: 500.0,
        timestamp: chrono::Utc::now(),
    };

    let should_schedule_high = scheduler.should_schedule_now(high_usage).await;
    println!(
        "  - High resource usage (25% CPU, 85% memory): Should schedule = {}",
        should_schedule_high
    );

    // Test component configuration status
    println!("\n🔗 Component configuration status:");
    println!(
        "  - Queue configured: {}",
        scheduler.has_queue_configured().await
    );
    println!(
        "  - Executor configured: {}",
        scheduler.has_executor_configured().await
    );

    // Test scheduler lifecycle
    println!("\n🔄 Testing scheduler lifecycle:");
    println!("  - Starting scheduler...");
    scheduler.start().await?;
    println!("  - Scheduler started: {}", scheduler.is_running().await);

    // Let it run briefly to demonstrate time-based scheduling
    println!("  - Running for 3 seconds to demonstrate scheduling...");
    sleep(Duration::from_secs(3)).await;

    // Check updated metrics
    let updated_metrics = scheduler.get_metrics().await;
    println!("  - Updated metrics:");
    println!(
        "    * Total cycles: {}",
        updated_metrics.total_scheduling_cycles
    );
    println!(
        "    * Time-based triggers: {}",
        updated_metrics.time_based_triggers
    );

    println!("  - Stopping scheduler...");
    scheduler.stop().await?;
    println!("  - Scheduler stopped: {}", !scheduler.is_running().await);

    // Demonstrate gap processing (mock)
    println!("\n📊 Demonstrating gap processing capabilities:");
    let sample_gaps = vec![
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/main.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Implement better error handling".to_string(),
            description: "Error handling TODO".to_string(),
            confidence: 0.9,
            priority: 8,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::ApiDocumentationGap,
            file_path: PathBuf::from("src/api.rs"),
            line_number: 15,
            column_number: Some(1),
            context: "pub fn process_request() -> Result<(), Error>".to_string(),
            description: "Missing API documentation".to_string(),
            confidence: 0.85,
            priority: 9,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::UndocumentedTechnology,
            file_path: PathBuf::from("Cargo.toml"),
            line_number: 20,
            column_number: None,
            context: "tokio-cron-scheduler = \"0.9\"".to_string(),
            description: "Undocumented cron scheduler usage".to_string(),
            confidence: 0.75,
            priority: 6,
            metadata: HashMap::new(),
        },
    ];

    println!("  - Sample gaps to process: {}", sample_gaps.len());
    for (i, gap) in sample_gaps.iter().enumerate() {
        println!(
            "    {}. {:?} in {} (priority {})",
            i + 1,
            gap.gap_type,
            gap.file_path.display(),
            gap.priority
        );
    }

    // Note: To fully demonstrate processing, we would need queue and executor configured
    println!("\n💡 To fully demonstrate gap processing:");
    println!("  - Configure a BackgroundScheduler (task queue)");
    println!("  - Configure a TaskExecutor");
    println!("  - Call scheduler.process_detected_gaps(gaps)");

    println!("\n✨ Research Task Scheduler Demo Complete!");
    println!("The scheduler is ready for integration with:");
    println!("  - File monitoring for event-driven scheduling");
    println!("  - Gap analysis engines for detecting knowledge gaps");
    println!("  - Background task queues for managing research tasks");
    println!("  - Task executors for performing research operations");

    Ok(())
}
