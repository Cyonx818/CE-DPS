//! Demonstration of the background task scheduler with priority queue and persistence
//!
//! This example shows:
//! - Creating research tasks from gap analysis results
//! - Priority-based ordering in the queue
//! - Basic queue operations (enqueue, dequeue, peek)
//! - Task state management
//! - Integration with existing gap analysis types

use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, GapType, ResearchTask,
    TaskPriority, TaskState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Background Task Scheduler Demo");
    println!("==================================");

    // Create a temporary directory for demo
    let temp_dir = TempDir::new()?;
    let queue_file = temp_dir.path().join("demo_queue.json");

    // Configure the scheduler
    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 100,
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 3,
        default_timeout: Duration::from_secs(60),
    };

    println!("📋 Creating background scheduler...");
    let scheduler = BackgroundScheduler::new(config).await?;

    // Create some test gaps from different types
    let gaps = vec![
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/main.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Implement error handling".to_string(),
            description: "Missing error handling implementation".to_string(),
            confidence: 0.9,
            priority: 8,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::MissingDocumentation,
            file_path: PathBuf::from("src/api.rs"),
            line_number: 15,
            column_number: None,
            context: "pub fn process_request() {".to_string(),
            description: "Public function without documentation".to_string(),
            confidence: 0.8,
            priority: 5,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::UndocumentedTechnology,
            file_path: PathBuf::from("Cargo.toml"),
            line_number: 12,
            column_number: None,
            context: "tokio = \"1.0\"".to_string(),
            description: "Tokio async runtime usage not documented".to_string(),
            confidence: 0.7,
            priority: 9,
            metadata: HashMap::new(),
        },
    ];

    // Create research tasks with different priorities
    println!("\n📝 Creating research tasks...");
    let tasks: Vec<ResearchTask> = gaps
        .into_iter()
        .enumerate()
        .map(|(i, gap)| {
            let priority = match i {
                0 => TaskPriority::High,     // TODO comment - high priority
                1 => TaskPriority::Medium,   // Missing docs - medium priority
                2 => TaskPriority::Critical, // Undocumented tech - critical priority
                _ => TaskPriority::Low,
            };

            let task = ResearchTask::from_gap(gap, priority);
            println!(
                "  ✓ Created task: {} (Priority: {:?})",
                task.id, task.priority
            );
            println!("    Query: {}", task.research_query);
            task
        })
        .collect();

    // Add tasks to queue
    println!("\n📤 Adding tasks to queue...");
    for task in tasks {
        scheduler.enqueue(task).await?;
    }

    println!("Queue size: {}", scheduler.queue_size().await);

    // Demonstrate priority ordering
    println!("\n🔄 Processing tasks in priority order...");
    let mut processed_count = 0;

    while let Some(task) = scheduler.dequeue().await? {
        processed_count += 1;
        println!(
            "  {} Processing task: {} (Priority: {:?})",
            processed_count, task.id, task.priority
        );
        println!("    File: {}", task.gap.file_path.display());
        println!("    Type: {:?}", task.gap.gap_type);
        println!("    Query: {}", task.research_query);

        // Simulate task execution by updating state
        let mut executing_task = task;
        executing_task.state = TaskState::Executing;
        executing_task.started_at = Some(chrono::Utc::now());

        scheduler.update_task(executing_task.clone()).await?;

        // Simulate completion
        executing_task.state = TaskState::Completed;
        executing_task.completed_at = Some(chrono::Utc::now());

        scheduler.update_task(executing_task).await?;

        println!("    ✅ Task completed");

        if processed_count >= 3 {
            break; // Process only the first 3 tasks for demo
        }
    }

    // Show queue metrics
    println!("\n📊 Queue Metrics:");
    let metrics = scheduler.get_metrics().await;
    println!("  Current queue size: {}", metrics.current_queue_size);
    println!("  Executing tasks: {}", metrics.executing_tasks);
    println!("  Completed tasks: {}", metrics.completed_tasks);
    println!("  Failed tasks: {}", metrics.failed_tasks);
    println!(
        "  Queue utilization: {:.2}%",
        metrics.queue_utilization * 100.0
    );

    // Demonstrate persistence
    println!("\n💾 Testing persistence...");
    scheduler.persist().await?;
    println!("  ✅ Queue state persisted successfully");

    println!("\n🎉 Demo completed successfully!");
    println!("    Priority ordering: ✅");
    println!("    Queue operations: ✅");
    println!("    State management: ✅");
    println!("    Persistence: ✅");
    println!("    Gap analysis integration: ✅");

    Ok(())
}
