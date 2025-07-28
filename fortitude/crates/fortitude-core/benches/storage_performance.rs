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

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude_core::vector::{DocumentMetadata, VectorConfig, VectorDocument};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_test_vector_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        collection_name: "test_collection".to_string(),
        embedding_dimension: 384,
        ..Default::default()
    }
}

fn create_test_document(id: &str, content: &str) -> VectorDocument {
    let mut metadata = HashMap::new();
    metadata.insert(
        "category".to_string(),
        serde_json::Value::String("test".to_string()),
    );
    metadata.insert(
        "timestamp".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1234567890)),
    );

    VectorDocument {
        id: id.to_string(),
        content: content.to_string(),
        embedding: vec![0.1; 384], // Mock embedding
        metadata: DocumentMetadata { fields: metadata },
    }
}

fn benchmark_single_document_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _config = create_test_vector_config();

    let mut group = c.benchmark_group("single_document_storage");
    group.sample_size(100);

    let document_sizes = vec![
        ("small", "Short document content"),
        ("medium", &"word ".repeat(100)),
        ("large", &"word ".repeat(1000)),
    ];

    for (size_name, content) in document_sizes {
        let doc = create_test_document("test_id", content);

        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("store", size_name), &doc, |b, doc| {
            b.to_async(&rt).iter(|| async {
                let _result = mock_store_document(black_box(doc)).await;
                black_box(_result);
            });
        });
    }

    group.finish();
}

fn benchmark_batch_document_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _config = create_test_vector_config();

    let mut group = c.benchmark_group("batch_document_storage");
    group.sample_size(50);

    let batch_sizes = [1, 5, 10, 25, 50, 100];

    for &batch_size in &batch_sizes {
        let documents: Vec<VectorDocument> = (0..batch_size)
            .map(|i| {
                create_test_document(&format!("doc_{i}"), &format!("Content for document {i}"))
            })
            .collect();

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &documents,
            |b, docs| {
                b.to_async(&rt).iter(|| async {
                    let _result = mock_store_batch_documents(black_box(docs)).await;
                    black_box(_result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_document_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _config = create_test_vector_config();

    let mut group = c.benchmark_group("document_retrieval");

    let retrieval_patterns = vec![
        ("single", vec!["doc_1"]),
        ("small_batch", vec!["doc_1", "doc_2", "doc_3"]),
        (
            "large_batch",
            (0..20)
                .map(|i| format!("doc_{i}"))
                .collect::<Vec<_>>()
                .iter()
                .map(|s| s.as_str())
                .collect(),
        ),
    ];

    for (pattern_name, ids) in retrieval_patterns {
        group.throughput(Throughput::Elements(ids.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("retrieve", pattern_name),
            &ids,
            |b, ids| {
                b.to_async(&rt).iter(|| async {
                    let _result = mock_retrieve_documents(black_box(ids)).await;
                    black_box(_result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_document_update(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _config = create_test_vector_config();

    let mut group = c.benchmark_group("document_update");

    let update_scenarios = vec![
        ("content_only", true, false),
        ("metadata_only", false, true),
        ("full_update", true, true),
    ];

    for (scenario_name, update_content, update_metadata) in update_scenarios {
        let mut doc = create_test_document("update_test", "Original content");

        if update_content {
            doc.content = "Updated content with new information".to_string();
        }

        if update_metadata {
            doc.metadata
                .fields
                .insert("updated".to_string(), serde_json::Value::Bool(true));
        }

        group.bench_with_input(BenchmarkId::new("update", scenario_name), &doc, |b, doc| {
            b.to_async(&rt).iter(|| async {
                let _result = mock_update_document(black_box(doc)).await;
                black_box(_result);
            });
        });
    }

    group.finish();
}

fn benchmark_concurrent_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _config = create_test_vector_config();

    let mut group = c.benchmark_group("concurrent_storage");

    let concurrency_levels = [1, 2, 4, 8, 16];

    for &concurrency in &concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let tasks: Vec<_> = (0..concurrency)
                        .map(|i| async move {
                            let doc = create_test_document(
                                &format!("concurrent_doc_{i}"),
                                &format!("Content {i}"),
                            );
                            let _result = mock_store_document(black_box(&doc)).await;
                            black_box(_result);
                        })
                        .collect();

                    futures::future::join_all(tasks).await;
                });
            },
        );
    }

    group.finish();
}

fn benchmark_metadata_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("metadata_operations");

    let metadata_sizes = vec![("small", 5), ("medium", 20), ("large", 100)];

    for (size_name, field_count) in metadata_sizes {
        let mut metadata = HashMap::new();
        for i in 0..field_count {
            metadata.insert(
                format!("field_{i}"),
                serde_json::Value::String(format!("value_{i}")),
            );
        }

        group.throughput(Throughput::Elements(field_count as u64));
        group.bench_with_input(
            BenchmarkId::new("serialize", size_name),
            &metadata,
            |b, metadata| {
                b.to_async(&rt).iter(|| async {
                    let _serialized = serde_json::to_string(black_box(metadata)).unwrap();
                    black_box(_serialized);
                });
            },
        );
    }

    group.finish();
}

// Mock functions for benchmarking
async fn mock_store_document(doc: &VectorDocument) -> Result<(), String> {
    // Simulate storage operation
    tokio::time::sleep(Duration::from_millis(1)).await;

    // Simulate validation
    if doc.id.is_empty() || doc.content.is_empty() {
        return Err("Invalid document".to_string());
    }

    Ok(())
}

async fn mock_store_batch_documents(docs: &[VectorDocument]) -> Result<(), String> {
    // Simulate batch storage operation
    tokio::time::sleep(Duration::from_millis(docs.len() as u64)).await;

    // Validate all documents
    for doc in docs {
        if doc.id.is_empty() || doc.content.is_empty() {
            return Err("Invalid document in batch".to_string());
        }
    }

    Ok(())
}

async fn mock_retrieve_documents(ids: &[&str]) -> Result<Vec<VectorDocument>, String> {
    // Simulate retrieval operation
    tokio::time::sleep(Duration::from_millis(ids.len() as u64)).await;

    let documents: Vec<VectorDocument> = ids
        .iter()
        .map(|id| create_test_document(id, &format!("Retrieved content for {id}")))
        .collect();

    Ok(documents)
}

async fn mock_update_document(doc: &VectorDocument) -> Result<(), String> {
    // Simulate update operation
    tokio::time::sleep(Duration::from_millis(2)).await;

    if doc.id.is_empty() {
        return Err("Cannot update document without ID".to_string());
    }

    Ok(())
}

criterion_group!(
    benches,
    benchmark_single_document_storage,
    benchmark_batch_document_storage,
    benchmark_document_retrieval,
    benchmark_document_update,
    benchmark_concurrent_storage,
    benchmark_metadata_operations
);
criterion_main!(benches);
