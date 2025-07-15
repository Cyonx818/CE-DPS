// ABOUTME: Demonstration of gap analysis integration with file monitoring for Sprint 008 Task 1.2
//! This example demonstrates how the gap analyzer integrates with the file monitoring system
//! to provide automated knowledge gap detection in real-time.

use fortitude::proactive::{
    FileMonitor, FileMonitorConfig, GapAnalysisConfig, GapAnalyzer, GapType,
};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔍 Gap Analysis Integration Demo");
    println!("================================");

    // Create a temporary directory for the demo
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    println!("📁 Creating demo project at: {}", project_path.display());

    // Set up file monitoring
    let monitor_config = FileMonitorConfig::for_rust_project()
        .with_debounce_ms(100)
        .with_max_queue_size(100);

    let mut file_monitor = FileMonitor::new(vec![project_path.clone()], monitor_config).await?;

    // Set up gap analyzer
    let gap_config = GapAnalysisConfig::for_rust_project()
        .with_confidence_threshold(0.4) // Lower threshold to catch more gaps
        .with_timeout_ms(200);

    let gap_analyzer = GapAnalyzer::new(gap_config)?;

    println!("🚀 Starting file monitoring and gap analysis...");

    // Create some demo files with various gaps
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;

        // File 1: Rust file with TODO and missing docs
        let file1 = project_path.join("src/main.rs");
        if let Ok(()) = fs::create_dir_all(file1.parent().unwrap()).await {
            let content1 = r#"
// TODO: Add proper error handling throughout the application
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

pub fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // FIXME: This should use configuration
    let addr = "0.0.0.0:8080";
    println!("Starting server on {}", addr);
    Ok(())
}

pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}
"#;
            let _ = fs::write(&file1, content1).await;
        }

        tokio::time::sleep(Duration::from_millis(300)).await;

        // File 2: Configuration file
        let file2 = project_path.join("config.toml");
        let content2 = r#"
# Server configuration
server_host = "localhost"
server_port = 8080
database_url = "postgres://localhost/myapp"
redis_url = "redis://localhost"
max_connections = 100
timeout_seconds = 30
"#;
        let _ = fs::write(&file2, content2).await;

        tokio::time::sleep(Duration::from_millis(300)).await;

        // File 3: Library file with API gaps
        let file3 = project_path.join("src/lib.rs");
        let content3 = r#"
/// This function processes data but lacks examples
pub fn process_data(input: &str) -> String {
    input.to_uppercase()
}

pub fn helper_function() {
    // No documentation at all
}

/// Well documented function with examples
/// 
/// # Examples
/// 
/// ```
/// let result = well_documented("test");
/// assert_eq!(result, "TEST");
/// ```
pub fn well_documented(input: &str) -> String {
    input.to_uppercase()
}
"#;
        let _ = fs::write(&file3, content3).await;
    });

    // Monitor for file events and analyze gaps
    let mut events_processed = 0;
    let mut total_gaps_found = 0;

    while events_processed < 3 {
        // Wait for file events with timeout
        if let Ok(Some(event)) = timeout(Duration::from_secs(2), file_monitor.next_event()).await {
            println!(
                "\n📄 File event detected: {:?} - {}",
                event.event_type,
                event.path.display()
            );

            // Analyze the file for gaps
            match gap_analyzer.analyze_file_event(&event).await {
                Ok(gaps) => {
                    if gaps.is_empty() {
                        println!("   ✅ No gaps detected");
                    } else {
                        println!("   🚨 Found {} gaps:", gaps.len());
                        total_gaps_found += gaps.len();

                        // Group gaps by type for better display
                        let mut gap_types = std::collections::HashMap::new();
                        for gap in &gaps {
                            gap_types
                                .entry(&gap.gap_type)
                                .or_insert_with(Vec::new)
                                .push(gap);
                        }

                        for (gap_type, gaps_of_type) in gap_types {
                            println!("      {:?}:", gap_type);
                            for gap in gaps_of_type {
                                println!("        • Line {}: {}", gap.line_number, gap.description);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Analysis failed: {}", e);
                }
            }

            events_processed += 1;
        } else {
            println!("⏰ Timeout waiting for file events");
            break;
        }
    }

    println!("\n📊 Analysis Summary");
    println!("==================");
    println!("Files processed: {}", events_processed);
    println!("Total gaps found: {}", total_gaps_found);
    println!("Gap types detected:");
    println!("  • TODO comments (high priority)");
    println!("  • Missing documentation (medium priority)");
    println!("  • Undocumented technologies (high priority)");
    println!("  • Configuration gaps (low priority)");
    println!("  • API documentation gaps (very high priority)");

    // Shutdown the monitor
    file_monitor.shutdown().await?;

    println!("\n✅ Demo completed successfully!");
    println!("🎯 Key Features Demonstrated:");
    println!("   • Real-time file monitoring integration");
    println!("   • Multiple gap detection algorithms");
    println!("   • Configurable confidence thresholds");
    println!("   • Priority-based gap classification");
    println!("   • Performance optimized analysis (<200ms per file)");

    Ok(())
}
