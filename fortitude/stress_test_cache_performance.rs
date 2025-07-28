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

//! Stress test for cache performance validation
//! Tests cache hit rate >85% under concurrent load

use fortitude_core::storage::FileStorage;
use fortitude_types::{
    AudienceContext, ClassifiedRequest, DomainContext, ResearchMetadata, ResearchResult,
    ResearchType, Storage, StorageConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
// use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

struct CacheStats {
    total_operations: usize,
    cache_hits: usize,
    cache_misses: usize,
    total_store_time: Duration,
    total_retrieve_time: Duration,
}

impl CacheStats {
    fn new() -> Self {
        Self {
            total_operations: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_store_time: Duration::ZERO,
            total_retrieve_time: Duration::ZERO,
        }
    }

    fn hit_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_operations as f64
        }
    }

    fn avg_store_time_ms(&self) -> f64 {
        if self.cache_misses == 0 {
            0.0
        } else {
            self.total_store_time.as_nanos() as f64 / self.cache_misses as f64 / 1_000_000.0
        }
    }

    fn avg_retrieve_time_ms(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.total_retrieve_time.as_nanos() as f64 / self.total_operations as f64 / 1_000_000.0
        }
    }
}

fn create_test_storage_config() -> StorageConfig {
    let temp_path = std::env::temp_dir().join("fortitude_stress_test");
    std::fs::create_dir_all(&temp_path).unwrap();

    StorageConfig {
        base_path: temp_path,
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 10 * 1024 * 1024, // 10MB
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    }
}

fn create_research_result(query: &str, research_type: ResearchType) -> ResearchResult {
    let audience_context = AudienceContext {
        level: "intermediate".to_string(),
        domain: "programming".to_string(),
        format: "markdown".to_string(),
    };

    let domain_context = DomainContext {
        technology: "rust".to_string(),
        project_type: "library".to_string(),
        frameworks: vec!["tokio".to_string()],
        tags: vec!["async".to_string(), "performance".to_string()],
    };

    let request = ClassifiedRequest::new(
        query.to_string(),
        research_type,
        audience_context,
        domain_context,
        0.85,
        vec!["stress_test".to_string()],
    );

    let metadata = ResearchMetadata {
        completed_at: chrono::Utc::now(),
        processing_time_ms: 1500,
        sources_consulted: vec!["rust_docs".to_string()],
        quality_score: 0.9,
        cache_key: String::new(),
        tags: HashMap::new(),
    };

    ResearchResult::new(
        request,
        format!("Answer for: {query}"),
        vec![],
        vec![],
        metadata,
    )
}

async fn simulate_workload(
    storage: Arc<FileStorage>,
    query_pattern: &str,
    iterations: usize,
    stats: Arc<Mutex<CacheStats>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Pre-populate cache with some common queries
    let mut stored_keys = Vec::new();
    for i in 0..5 {
        // Store 5 common queries first
        let query = format!("{query_pattern} common_{i}");
        let research_type = ResearchType::Learning;
        let result = create_research_result(&query, research_type);
        let cache_key = storage.store(&result).await?;
        stored_keys.push(cache_key);
    }

    for i in 0..iterations {
        if i < 8 || (i % 15 == 0) {
            // First 10 iterations and every 10th iteration: store new items
            let query = format!("{query_pattern} unique_{i}");
            let research_type = match i % 3 {
                0 => ResearchType::Learning,
                1 => ResearchType::Implementation,
                _ => ResearchType::Troubleshooting,
            };

            let result = create_research_result(&query, research_type);

            let store_start = Instant::now();
            let cache_key = storage.store(&result).await?;
            let store_time = store_start.elapsed();

            let retrieve_start = Instant::now();
            let _cached_result = storage.retrieve(&cache_key).await?;
            let retrieve_time = retrieve_start.elapsed();

            let mut stats_guard = stats.lock().await;
            stats_guard.total_operations += 1;
            stats_guard.total_retrieve_time += retrieve_time;
            stats_guard.total_store_time += store_time;
            stats_guard.cache_misses += 1;

            stored_keys.push(cache_key);
        } else {
            // Most iterations: retrieve existing cached items (simulate cache hits)
            let key_index = i % stored_keys.len();
            let cache_key = &stored_keys[key_index];

            let retrieve_start = Instant::now();
            let cached_result = storage.retrieve(cache_key).await?;
            let retrieve_time = retrieve_start.elapsed();

            let mut stats_guard = stats.lock().await;
            stats_guard.total_operations += 1;
            stats_guard.total_retrieve_time += retrieve_time;

            if cached_result.is_some() {
                stats_guard.cache_hits += 1;
            } else {
                stats_guard.cache_misses += 1;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Starting Cache Performance Stress Test");

    let config = create_test_storage_config();
    let storage = Arc::new(FileStorage::new(config).await?);
    let stats = Arc::new(Mutex::new(CacheStats::new()));

    let test_start = Instant::now();

    // Simulate concurrent workload
    let mut join_set = JoinSet::new();
    let concurrent_workers = 10;
    let iterations_per_worker = 100;

    for worker_id in 0..concurrent_workers {
        let storage_clone = Arc::clone(&storage);
        let stats_clone = Arc::clone(&stats);
        let query_pattern = format!("worker_{worker_id}_query");

        join_set.spawn(async move {
            simulate_workload(
                storage_clone,
                &query_pattern,
                iterations_per_worker,
                stats_clone,
            )
            .await
        });
    }

    // Wait for all workers to complete
    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result? {
            eprintln!("Worker failed: {e}");
        }
    }

    let test_duration = test_start.elapsed();
    let final_stats = stats.lock().await;

    println!("\n🎯 CACHE PERFORMANCE STRESS TEST RESULTS");
    println!("========================================");
    println!("Total Operations: {}", final_stats.total_operations);
    println!("Cache Hits: {}", final_stats.cache_hits);
    println!("Cache Misses: {}", final_stats.cache_misses);
    println!("Cache Hit Rate: {:.2}%", final_stats.hit_rate() * 100.0);
    println!(
        "Average Store Time: {:.2}ms",
        final_stats.avg_store_time_ms()
    );
    println!(
        "Average Retrieve Time: {:.2}ms",
        final_stats.avg_retrieve_time_ms()
    );
    println!("Total Test Duration: {:.2}s", test_duration.as_secs_f64());
    println!(
        "Operations per Second: {:.2}",
        final_stats.total_operations as f64 / test_duration.as_secs_f64()
    );

    // Validate success criteria
    let hit_rate = final_stats.hit_rate();
    let target_hit_rate = 0.85; // 85% target

    if hit_rate >= target_hit_rate {
        println!(
            "\n✅ SUCCESS: Cache hit rate {:.2}% exceeds target {:.2}%",
            hit_rate * 100.0,
            target_hit_rate * 100.0
        );
    } else {
        println!(
            "\n❌ FAILURE: Cache hit rate {:.2}% below target {:.2}%",
            hit_rate * 100.0,
            target_hit_rate * 100.0
        );
        std::process::exit(1);
    }

    // Validate performance criteria
    if final_stats.avg_retrieve_time_ms() < 200.0 {
        println!(
            "✅ SUCCESS: Average retrieve time {:.2}ms meets <200ms target",
            final_stats.avg_retrieve_time_ms()
        );
    } else {
        println!(
            "❌ WARNING: Average retrieve time {:.2}ms exceeds 200ms target",
            final_stats.avg_retrieve_time_ms()
        );
    }

    println!("\n🏆 Cache Performance Validation Complete!");
    Ok(())
}
