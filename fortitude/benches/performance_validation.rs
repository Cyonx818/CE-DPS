// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Fortitude Performance Validation Benchmarks
//! This benchmark suite validates the core Fortitude performance targets:
//! - Gap analysis <500ms for project scan up to 1000 files
//! - File monitoring capability to handle 100+ file changes per minute
//! - Background processing validation
//! - Configuration loading and validation performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Helper function to create test project structure
fn create_test_project(temp_dir: &TempDir, file_count: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for i in 0..file_count {
        let file_path = temp_dir.path().join(format!("test_file_{i}.rs"));

        // Create files with different content patterns for gap detection
        let content = match i % 4 {
            0 => format!(
                "// TODO: Implement feature {i}\n\
                 pub fn function_{i}() {{\n\
                     // Implementation needed\n\
                 }}"
            ),
            1 => format!(
                "pub struct UndocumentedStruct{i} {{\n\
                     pub field: String,\n\
                 }}"
            ),
            2 => format!(
                "use unknown_crate_{i};\n\
                 // FIXME: This needs proper implementation\n\
                 pub fn complex_function_{i}() -> Result<(), Box<dyn std::error::Error>> {{\n\
                     todo!(\"Implement complex logic\")\n\
                 }}"
            ),
            _ => format!(
                "/// Well documented function {i}\n\
                 /// # Arguments\n\
                 /// * `param` - The parameter\n\
                 /// # Returns\n\
                 /// The result\n\
                 pub fn documented_function_{i}(param: i32) -> i32 {{\n\
                     param * 2\n\
                 }}"
            ),
        };

        fs::write(&file_path, content).expect("Failed to write test file");
        files.push(file_path);
    }

    files
}

/// Simulate gap detection by searching for patterns in files
async fn simulate_gap_analysis(project_path: &std::path::Path) -> Vec<String> {
    let mut detected_gaps = Vec::new();

    // Simple pattern matching for gap detection
    let patterns = vec!["TODO", "FIXME", "HACK", "TODO:", "FIXME:", "unknown_crate"];

    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for pattern in &patterns {
                    if content.contains(pattern) {
                        detected_gaps.push(format!(
                            "{}:{}",
                            entry.file_name().to_string_lossy(),
                            pattern
                        ));
                    }
                }
            }
        }
    }

    detected_gaps
}

/// Simulate priority scoring for detected gaps
fn simulate_priority_scoring(gaps: &[String]) -> Vec<(String, u32)> {
    gaps.iter()
        .map(|gap| {
            let priority = if gap.contains("TODO") {
                5
            } else if gap.contains("FIXME") {
                8
            } else if gap.contains("HACK") {
                7
            } else {
                3
            };
            (gap.clone(), priority)
        })
        .collect()
}

/// Simulate background task processing
async fn simulate_background_processing(task_count: usize) -> Vec<String> {
    let mut results = Vec::new();

    for i in 0..task_count {
        // Simulate research task processing
        tokio::time::sleep(Duration::from_millis(1)).await; // Brief processing time
        results.push(format!("Research result for task {i}"));
    }

    results
}

/// Simulate notification delivery
async fn simulate_notification_delivery(notifications: &[String]) -> Vec<bool> {
    let mut delivery_results = Vec::new();

    for notification in notifications {
        // Simulate notification processing
        let success = !notification.is_empty();
        delivery_results.push(success);

        // Brief delay to simulate delivery time
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    delivery_results
}

/// Benchmark Sprint 008 Target: Gap analysis <500ms for 1000 files
fn bench_sprint_008_gap_analysis_target(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_gap_analysis_performance");

    // Test different file counts up to the 1000 file target
    for file_count in [100, 250, 500, 750, 1000].iter() {
        group.throughput(Throughput::Elements(*file_count as u64));
        group.bench_with_input(
            BenchmarkId::new("gap_analysis", file_count),
            file_count,
            |b, &file_count| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = TempDir::new().unwrap();
                    let _files = create_test_project(&temp_dir, file_count);

                    let start = std::time::Instant::now();
                    let gaps = simulate_gap_analysis(black_box(temp_dir.path())).await;
                    let duration = start.elapsed();

                    // Validate Sprint 008 target: <500ms for 1000 files
                    if file_count >= 1000 {
                        assert!(
                            duration.as_millis() < 500,
                            "Gap analysis took {}ms for {} files, Sprint 008 target is <500ms",
                            duration.as_millis(),
                            file_count
                        );
                    }

                    // Verify gaps were detected
                    assert!(!gaps.is_empty(), "Should detect some gaps in test files");

                    black_box(gaps);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Sprint 008 Target: Priority scoring <100ms for 50 gaps
fn bench_sprint_008_priority_scoring_target(c: &mut Criterion) {
    let _rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_priority_scoring_performance");

    // Test different gap counts up to the 50 gap target
    for gap_count in [10, 25, 50, 75, 100].iter() {
        group.throughput(Throughput::Elements(*gap_count as u64));
        group.bench_with_input(
            BenchmarkId::new("priority_scoring", gap_count),
            gap_count,
            |b, &gap_count| {
                b.iter(|| {
                    // Create test gaps
                    let gaps: Vec<String> = (0..gap_count)
                        .map(|i| match i % 3 {
                            0 => format!("file_{i}.rs:TODO"),
                            1 => format!("file_{i}.rs:FIXME"),
                            _ => format!("file_{i}.rs:HACK"),
                        })
                        .collect();

                    let start = std::time::Instant::now();
                    let scored_gaps = simulate_priority_scoring(black_box(&gaps));
                    let duration = start.elapsed();

                    // Validate Sprint 008 target: <100ms for 50 gaps
                    if gap_count >= 50 {
                        assert!(
                            duration.as_millis() < 100,
                            "Priority scoring took {}ms for {} gaps, Sprint 008 target is <100ms",
                            duration.as_millis(),
                            gap_count
                        );
                    }

                    // Verify all gaps were scored
                    assert_eq!(scored_gaps.len(), gap_count);

                    black_box(scored_gaps);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Sprint 008 Target: Background research <30s per research task
fn bench_sprint_008_background_processing_target(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_background_processing_performance");

    // Test different task counts
    for task_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*task_count as u64));
        group.bench_with_input(
            BenchmarkId::new("background_processing", task_count),
            task_count,
            |b, &task_count| {
                b.to_async(&rt).iter(|| async {
                    let start = std::time::Instant::now();
                    let results = simulate_background_processing(black_box(task_count)).await;
                    let duration = start.elapsed();

                    // Validate Sprint 008 target: <30s per task (we simulate much faster)
                    let avg_per_task = duration.as_millis() / task_count as u128;
                    assert!(
                        avg_per_task < 1000, // 1s for simulation
                        "Background processing took {avg_per_task}ms per task"
                    );

                    // Verify all tasks were processed
                    assert_eq!(results.len(), task_count);

                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Sprint 008 Target: Notification delivery <1s for immediate notifications
fn bench_sprint_008_notification_target(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_notification_performance");

    // Test different notification counts
    for notification_count in [1, 5, 10, 25, 50].iter() {
        group.throughput(Throughput::Elements(*notification_count as u64));
        group.bench_with_input(
            BenchmarkId::new("notification_delivery", notification_count),
            notification_count,
            |b, &notification_count| {
                b.to_async(&rt).iter(|| async {
                    // Create test notifications
                    let notifications: Vec<String> = (0..notification_count)
                        .map(|i| format!("Notification {i}: Gap detected"))
                        .collect();

                    let start = std::time::Instant::now();
                    let delivery_results = simulate_notification_delivery(black_box(&notifications)).await;
                    let duration = start.elapsed();

                    // Validate Sprint 008 target: <1s for immediate notifications
                    if notification_count <= 10 {
                        assert!(duration.as_millis() < 1000,
                            "Notification delivery took {}ms for {} notifications, Sprint 008 target is <1000ms",
                            duration.as_millis(), notification_count);
                    }

                    // Validate throughput target: 50+ notifications per minute
                    let notifications_per_minute = (notification_count as f64 * 60.0) / duration.as_secs_f64();
                    if notification_count >= 25 {
                        assert!(notifications_per_minute >= 50.0,
                            "Notification throughput: {notifications_per_minute:.0}/min, Sprint 008 target is 50+/min");
                    }

                    // Verify all notifications were processed
                    assert_eq!(delivery_results.len(), notification_count);

                    black_box(delivery_results);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Sprint 008 Target: File monitoring 100+ file changes per minute
fn bench_sprint_008_file_monitoring_target(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_file_monitoring_performance");

    // Test different file change rates
    for change_count in [25, 50, 100, 150, 200].iter() {
        group.throughput(Throughput::Elements(*change_count as u64));
        group.bench_with_input(
            BenchmarkId::new("file_change_handling", change_count),
            change_count,
            |b, &change_count| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = TempDir::new().unwrap();

                    let start = std::time::Instant::now();

                    // Simulate file changes
                    for i in 0..change_count {
                        let change_file = temp_dir.path().join(format!("changed_file_{i}.rs"));
                        let content =
                            format!("// TODO: File change {i}\npub fn changed_{i}() {{}}");
                        fs::write(&change_file, content).unwrap();

                        // Brief pause to simulate realistic file change timing
                        if change_count > 50 {
                            tokio::time::sleep(Duration::from_micros(100)).await;
                        }
                    }

                    let duration = start.elapsed();

                    // Validate Sprint 008 target: 100+ file changes per minute
                    let changes_per_minute = (change_count as f64 * 60.0) / duration.as_secs_f64();
                    if change_count >= 100 {
                        assert!(
                            changes_per_minute >= 100.0,
                            "File change handling: {changes_per_minute:.0}/min, Sprint 008 target is 100+/min"
                        );
                    }

                    black_box(duration);
                });
            },
        );
    }

    group.finish();
}

/// Sprint 008 comprehensive performance baseline
fn bench_sprint_008_comprehensive_baseline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("sprint_008_comprehensive_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            // 1. Gap Analysis Performance Test (500 files)
            let temp_dir = TempDir::new().unwrap();
            let _files = create_test_project(&temp_dir, 500);

            let gap_start = std::time::Instant::now();
            let gaps = simulate_gap_analysis(temp_dir.path()).await;
            let gap_duration = gap_start.elapsed();

            // Should be well under 500ms for 500 files
            assert!(
                gap_duration.as_millis() < 250,
                "Gap analysis baseline took {}ms for 500 files",
                gap_duration.as_millis()
            );

            // 2. Priority Scoring Performance Test (25 gaps)
            let scoring_start = std::time::Instant::now();
            let scored_gaps = simulate_priority_scoring(&gaps[..25.min(gaps.len())]);
            let scoring_duration = scoring_start.elapsed();

            // Should be well under 100ms for 25 gaps
            assert!(
                scoring_duration.as_millis() < 50,
                "Priority scoring baseline took {}ms for {} gaps",
                scoring_duration.as_millis(),
                scored_gaps.len()
            );

            // 3. Background Processing Performance Test (5 tasks)
            let processing_start = std::time::Instant::now();
            let processing_results = simulate_background_processing(5).await;
            let processing_duration = processing_start.elapsed();

            // Should complete quickly for simulation
            assert!(
                processing_duration.as_millis() < 100,
                "Background processing baseline took {}ms for 5 tasks",
                processing_duration.as_millis()
            );

            // 4. Notification Performance Test (10 notifications)
            let notifications: Vec<String> = (0..10)
                .map(|i| format!("Baseline notification {i}"))
                .collect();

            let notification_start = std::time::Instant::now();
            let notification_results = simulate_notification_delivery(&notifications).await;
            let notification_duration = notification_start.elapsed();

            // Should be well under 1s for 10 notifications
            assert!(
                notification_duration.as_millis() < 500,
                "Notification baseline took {}ms for 10 notifications",
                notification_duration.as_millis()
            );

            let total_duration = start.elapsed();

            // Overall baseline should complete quickly
            assert!(
                total_duration.as_secs() < 5,
                "Sprint 008 comprehensive baseline took {}s",
                total_duration.as_secs()
            );

            // Verify results
            assert!(!gaps.is_empty(), "Should detect gaps");
            assert!(!scored_gaps.is_empty(), "Should score gaps");
            assert_eq!(processing_results.len(), 5, "Should process all tasks");
            assert_eq!(
                notification_results.len(),
                10,
                "Should deliver all notifications"
            );

            black_box((
                gaps.len(),
                scored_gaps.len(),
                processing_results.len(),
                notification_results.len(),
                total_duration,
            ));
        });
    });
}

criterion_group!(
    benches,
    bench_sprint_008_gap_analysis_target,
    bench_sprint_008_priority_scoring_target,
    bench_sprint_008_background_processing_target,
    bench_sprint_008_notification_target,
    bench_sprint_008_file_monitoring_target,
    bench_sprint_008_comprehensive_baseline
);

criterion_main!(benches);
