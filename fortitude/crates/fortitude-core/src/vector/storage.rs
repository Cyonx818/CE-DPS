// ABOUTME: Vector storage abstraction combining Qdrant client and embedding service
//! This module provides a high-level vector storage interface that combines
//! the Qdrant client for vector database operations and the embedding service
//! for text-to-vector conversion. It offers CRUD operations, batch processing,
//! and metadata management for research content.

use crate::vector::{
    client::QdrantClient,
    embeddings::{EmbeddingGenerator, LocalEmbeddingService},
    error::{VectorError, VectorResult},
};
use async_trait::async_trait;
use fortitude_types::research::*;
use qdrant_client::qdrant::{
    DeletePointsBuilder, GetPointsBuilder, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    Value,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Document stored in vector database with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    /// Unique document identifier
    pub id: String,
    /// Original text content
    pub content: String,
    /// Vector embedding of the content
    pub embedding: Vec<f32>,
    /// Research-specific metadata
    pub metadata: DocumentMetadata,
    /// Storage timestamp
    pub stored_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata associated with vector documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Research type classification
    pub research_type: Option<ResearchType>,
    /// Content type (query, result, evidence, etc.)
    pub content_type: String,
    /// Quality score for the content
    pub quality_score: Option<f64>,
    /// Source information
    pub source: Option<String>,
    /// Additional tags for categorization
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            research_type: None,
            content_type: "document".to_string(),
            quality_score: None,
            source: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilaritySearchResult {
    /// The matching document
    pub document: VectorDocument,
    /// Similarity score (0.0-1.0)
    pub score: f64,
}

/// Configuration for similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Maximum number of results to return
    pub limit: usize,
    /// Minimum similarity score threshold
    pub threshold: Option<f64>,
    /// Collection to search in (optional, uses default if None)
    pub collection: Option<String>,
    /// Additional filters
    pub filters: Vec<SearchFilter>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            limit: 10,
            threshold: None,
            collection: None,
            filters: Vec::new(),
        }
    }
}

/// Filter for vector search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Field to filter on
    pub field: String,
    /// Filter operation
    pub operation: FilterOperation,
    /// Filter value
    pub value: serde_json::Value,
}

/// Filter operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperation {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    In,
    NotIn,
}

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult<T> {
    /// Successful operations
    pub successful: Vec<T>,
    /// Failed operations with errors
    pub failed: Vec<BatchError>,
    /// Total operations attempted
    pub total_attempted: usize,
}

/// Error information for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Index of the failed operation
    pub index: usize,
    /// Document ID (if available)
    pub document_id: Option<String>,
    /// Error message
    pub error: String,
}

/// Statistics about vector storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStorageStats {
    /// Total documents stored
    pub total_documents: u64,
    /// Total search operations
    pub total_searches: u64,
    /// Total batch operations
    pub total_batch_operations: u64,
    /// Average search latency in milliseconds
    pub avg_search_latency_ms: f64,
    /// Average embedding generation time
    pub avg_embedding_time_ms: f64,
    /// Cache hit rate from embedding service
    pub embedding_cache_hit_rate: f64,
}

/// High-level vector storage service combining Qdrant and embedding generation
pub struct VectorStorage {
    /// Qdrant client for vector database operations
    qdrant_client: Arc<QdrantClient>,
    /// Embedding service for text-to-vector conversion
    embedding_service: Arc<LocalEmbeddingService>,
    /// Default collection name
    default_collection: String,
    /// Service statistics
    stats: Arc<tokio::sync::RwLock<VectorStorageStats>>,
}

impl VectorStorage {
    /// Create a new vector storage service
    pub fn new(
        qdrant_client: Arc<QdrantClient>,
        embedding_service: Arc<LocalEmbeddingService>,
    ) -> Self {
        let default_collection = qdrant_client.default_collection().to_string();

        Self {
            qdrant_client,
            embedding_service,
            default_collection,
            stats: Arc::new(tokio::sync::RwLock::new(VectorStorageStats {
                total_documents: 0,
                total_searches: 0,
                total_batch_operations: 0,
                avg_search_latency_ms: 0.0,
                avg_embedding_time_ms: 0.0,
                embedding_cache_hit_rate: 0.0,
            })),
        }
    }

    /// Initialize the storage service (ensure collections exist)
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> VectorResult<()> {
        info!("Initializing vector storage service");

        // Initialize embedding service
        self.embedding_service.initialize().await?;

        // Ensure default collection exists
        self.ensure_collection(&self.default_collection).await?;

        info!("Vector storage service initialized successfully");
        Ok(())
    }

    /// Ensure a collection exists with proper configuration
    #[instrument(skip(self))]
    pub async fn ensure_collection(&self, collection_name: &str) -> VectorResult<()> {
        debug!("Ensuring collection exists: {}", collection_name);
        self.qdrant_client.ensure_collection(collection_name).await
    }

    /// Store a single document with automatic embedding generation
    #[instrument(skip(self, content))]
    pub async fn store_document(
        &self,
        content: &str,
        metadata: DocumentMetadata,
    ) -> VectorResult<VectorDocument> {
        let start_time = Instant::now();

        // Generate embedding for the content
        let embedding = self.embedding_service.generate_embedding(content).await?;

        let document = VectorDocument {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            embedding: embedding.clone(),
            metadata: metadata.clone(),
            stored_at: chrono::Utc::now(),
        };

        // Store in Qdrant
        self.store_document_with_embedding(&document).await?;

        // Update statistics
        self.update_embedding_stats(start_time.elapsed().as_millis() as f64)
            .await;

        info!("Document stored successfully: {}", document.id);
        Ok(document)
    }

    /// Store a document with pre-computed embedding
    #[instrument(skip(self, document))]
    async fn store_document_with_embedding(&self, document: &VectorDocument) -> VectorResult<()> {
        let collection = &self.default_collection;

        // Convert metadata to Qdrant payload
        let mut payload = HashMap::new();
        payload.insert("content".to_string(), Value::from(document.content.clone()));
        payload.insert(
            "stored_at".to_string(),
            Value::from(document.stored_at.to_rfc3339()),
        );

        if let Some(research_type) = &document.metadata.research_type {
            payload.insert(
                "research_type".to_string(),
                Value::from(research_type.to_string()),
            );
        }
        payload.insert(
            "content_type".to_string(),
            Value::from(document.metadata.content_type.clone()),
        );

        if let Some(quality_score) = document.metadata.quality_score {
            payload.insert("quality_score".to_string(), Value::from(quality_score));
        }

        if let Some(source) = &document.metadata.source {
            payload.insert("source".to_string(), Value::from(source.clone()));
        }

        if !document.metadata.tags.is_empty() {
            payload.insert(
                "tags".to_string(),
                Value::from(document.metadata.tags.clone()),
            );
        }

        // Add custom fields
        for (key, value) in &document.metadata.custom_fields {
            payload.insert(format!("custom_{key}"), value.clone().into());
        }

        // Create point for upsert
        let point = PointStruct::new(document.id.clone(), document.embedding.clone(), payload);

        // Upsert the point
        let upsert_request = UpsertPointsBuilder::new(collection.clone(), vec![point]);

        self.qdrant_client
            .client()
            .upsert_points(upsert_request)
            .await
            .map_err(|e| VectorError::from_operation_failed("store_document", e.to_string()))?;

        // Update document count
        let mut stats = self.stats.write().await;
        stats.total_documents += 1;

        Ok(())
    }

    /// Retrieve documents similar to a query
    #[instrument(skip(self, query))]
    pub async fn retrieve_similar(
        &self,
        query: &str,
        config: SearchConfig,
    ) -> VectorResult<Vec<SimilaritySearchResult>> {
        let start_time = Instant::now();

        // Generate embedding for query
        let query_embedding = self.embedding_service.generate_embedding(query).await?;

        // Perform vector search
        let results = self.search_by_vector(&query_embedding, config).await?;

        // Update search statistics
        self.update_search_stats(start_time.elapsed().as_millis() as f64)
            .await;

        debug!("Retrieved {} similar documents for query", results.len());
        Ok(results)
    }

    /// Search by pre-computed vector
    #[instrument(skip(self, query_vector))]
    async fn search_by_vector(
        &self,
        query_vector: &[f32],
        config: SearchConfig,
    ) -> VectorResult<Vec<SimilaritySearchResult>> {
        let collection = config
            .collection
            .as_deref()
            .unwrap_or(&self.default_collection);

        // Build search request
        let mut search_request =
            SearchPointsBuilder::new(collection, query_vector.to_vec(), config.limit as u64);

        // Add threshold if specified
        if let Some(threshold) = config.threshold {
            search_request = search_request.score_threshold(threshold as f32);
        }

        // Execute search
        let search_response = self
            .qdrant_client
            .client()
            .search_points(search_request)
            .await
            .map_err(|e| VectorError::from_operation_failed("search_similar", e.to_string()))?;

        // Convert results to our format
        let mut results = Vec::new();
        for scored_point in search_response.result {
            if let Some(ref vector_data) = scored_point.vectors {
                if let Some(qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(
                    dense_vector,
                )) = &vector_data.vectors_options
                {
                    let document = self.build_document_from_scored_point(
                        &scored_point,
                        dense_vector.data.clone(),
                    )?;
                    results.push(SimilaritySearchResult {
                        document,
                        score: scored_point.score as f64,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Build VectorDocument from Qdrant ScoredPoint data
    fn build_document_from_scored_point(
        &self,
        point: &qdrant_client::qdrant::ScoredPoint,
        vector_data: Vec<f32>,
    ) -> VectorResult<VectorDocument> {
        let payload = &point.payload;

        // Extract basic fields
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let stored_at = payload
            .get("stored_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        // Extract metadata
        let research_type = payload
            .get("research_type")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<ResearchType>().ok());

        let content_type = payload
            .get("content_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "document".to_string());

        let quality_score = payload.get("quality_score").and_then(|v| {
            if let Some(kind) = &v.kind {
                match kind {
                    qdrant_client::qdrant::value::Kind::DoubleValue(d) => Some(*d),
                    qdrant_client::qdrant::value::Kind::IntegerValue(i) => Some(*i as f64),
                    _ => None,
                }
            } else {
                None
            }
        });

        let source = payload
            .get("source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tags = payload
            .get("tags")
            .and_then(|v| {
                v.as_list().map(|list| {
                    list.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
            })
            .unwrap_or_default();

        // Extract custom fields
        let mut custom_fields = HashMap::new();
        for (key, value) in payload {
            if key.starts_with("custom_") {
                let custom_key = key.strip_prefix("custom_").unwrap().to_string();
                custom_fields.insert(custom_key, value.clone().into());
            }
        }

        let metadata = DocumentMetadata {
            research_type,
            content_type,
            quality_score,
            source,
            tags,
            custom_fields,
        };

        Ok(VectorDocument {
            id: point
                .id
                .as_ref()
                .map(|id| {
                    if let Some(point_id_options) = &id.point_id_options {
                        match point_id_options {
                            qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid) => {
                                uuid.clone()
                            }
                            qdrant_client::qdrant::point_id::PointIdOptions::Num(num) => {
                                num.to_string()
                            }
                        }
                    } else {
                        "unknown".to_string()
                    }
                })
                .unwrap_or_else(|| "unknown".to_string()),
            content,
            embedding: vector_data,
            metadata,
            stored_at,
        })
    }

    /// Build VectorDocument from Qdrant RetrievedPoint data
    fn build_document_from_retrieved_point(
        &self,
        point: &qdrant_client::qdrant::RetrievedPoint,
        vector_data: Vec<f32>,
    ) -> VectorResult<VectorDocument> {
        let payload = &point.payload;

        // Extract basic fields
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let stored_at = payload
            .get("stored_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        // Extract metadata
        let research_type = payload
            .get("research_type")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<ResearchType>().ok());

        let content_type = payload
            .get("content_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "document".to_string());

        let quality_score = payload.get("quality_score").and_then(|v| {
            if let Some(kind) = &v.kind {
                match kind {
                    qdrant_client::qdrant::value::Kind::DoubleValue(d) => Some(*d),
                    qdrant_client::qdrant::value::Kind::IntegerValue(i) => Some(*i as f64),
                    _ => None,
                }
            } else {
                None
            }
        });

        let source = payload
            .get("source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tags = payload
            .get("tags")
            .and_then(|v| {
                v.as_list().map(|list| {
                    list.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
            })
            .unwrap_or_default();

        // Extract custom fields
        let mut custom_fields = HashMap::new();
        for (key, value) in payload {
            if key.starts_with("custom_") {
                let custom_key = key.strip_prefix("custom_").unwrap().to_string();
                custom_fields.insert(custom_key, value.clone().into());
            }
        }

        let metadata = DocumentMetadata {
            research_type,
            content_type,
            quality_score,
            source,
            tags,
            custom_fields,
        };

        Ok(VectorDocument {
            id: point
                .id
                .as_ref()
                .map(|id| {
                    if let Some(point_id_options) = &id.point_id_options {
                        match point_id_options {
                            qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid) => {
                                uuid.clone()
                            }
                            qdrant_client::qdrant::point_id::PointIdOptions::Num(num) => {
                                num.to_string()
                            }
                        }
                    } else {
                        "unknown".to_string()
                    }
                })
                .unwrap_or_else(|| "unknown".to_string()),
            content,
            embedding: vector_data,
            metadata,
            stored_at,
        })
    }

    /// Retrieve a specific document by ID
    #[instrument(skip(self))]
    pub async fn retrieve_by_id(&self, id: &str) -> VectorResult<Option<VectorDocument>> {
        let collection = &self.default_collection;

        // Get points by ID
        let get_request = GetPointsBuilder::new(collection, vec![id.into()])
            .with_payload(true)
            .with_vectors(true);

        let response = self
            .qdrant_client
            .client()
            .get_points(get_request)
            .await
            .map_err(|e| VectorError::from_operation_failed("retrieve_by_id", e.to_string()))?;

        if let Some(point) = response.result.first() {
            if let Some(vector_data) = &point.vectors {
                if let Some(qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(
                    dense_vector,
                )) = &vector_data.vectors_options
                {
                    let document =
                        self.build_document_from_retrieved_point(point, dense_vector.data.clone())?;
                    return Ok(Some(document));
                }
            }
        }

        Ok(None)
    }

    /// Update an existing document
    #[instrument(skip(self, content))]
    pub async fn update_document(
        &self,
        id: &str,
        content: &str,
        metadata: DocumentMetadata,
    ) -> VectorResult<VectorDocument> {
        let start_time = Instant::now();

        // Generate new embedding for updated content
        let embedding = self.embedding_service.generate_embedding(content).await?;

        let document = VectorDocument {
            id: id.to_string(),
            content: content.to_string(),
            embedding,
            metadata,
            stored_at: chrono::Utc::now(),
        };

        // Store updated document (upsert will replace existing)
        self.store_document_with_embedding(&document).await?;

        // Update statistics
        self.update_embedding_stats(start_time.elapsed().as_millis() as f64)
            .await;

        info!("Document updated successfully: {}", id);
        Ok(document)
    }

    /// Delete a document by ID
    #[instrument(skip(self))]
    pub async fn delete_document(&self, id: &str) -> VectorResult<bool> {
        let collection = &self.default_collection;

        let delete_request = DeletePointsBuilder::new(collection)
            .points(vec![qdrant_client::qdrant::PointId::from(id.to_string())]);

        let response = self
            .qdrant_client
            .client()
            .delete_points(delete_request)
            .await
            .map_err(|e| VectorError::from_operation_failed("delete_document", e.to_string()))?;

        let deleted = response.result.map(|r| r.status).unwrap_or(0) > 0;

        if deleted {
            // Update document count
            let mut stats = self.stats.write().await;
            if stats.total_documents > 0 {
                stats.total_documents -= 1;
            }
            info!("Document deleted successfully: {}", id);
        }

        Ok(deleted)
    }

    /// Store multiple documents in batch
    #[instrument(skip(self, documents))]
    pub async fn store_documents(
        &self,
        documents: Vec<(String, DocumentMetadata)>,
    ) -> VectorResult<BatchResult<VectorDocument>> {
        let start_time = Instant::now();

        // Update batch operation count
        {
            let mut stats = self.stats.write().await;
            stats.total_batch_operations += 1;
        }

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        // Generate embeddings for all documents
        let contents: Vec<String> = documents
            .iter()
            .map(|(content, _)| content.clone())
            .collect();

        let embeddings_result = self.embedding_service.generate_embeddings(&contents).await;

        let embeddings = match embeddings_result {
            Ok(emb) => emb,
            Err(e) => {
                // If batch embedding fails, try individual embeddings
                warn!(
                    "Batch embedding failed, falling back to individual generation: {}",
                    e
                );
                let mut individual_embeddings = Vec::new();

                for (idx, (content, _)) in documents.iter().enumerate() {
                    match self.embedding_service.generate_embedding(content).await {
                        Ok(embedding) => individual_embeddings.push(embedding),
                        Err(embedding_err) => {
                            failed.push(BatchError {
                                index: idx,
                                document_id: None,
                                error: embedding_err.to_string(),
                            });
                            individual_embeddings.push(vec![]); // Placeholder
                        }
                    }
                }
                individual_embeddings
            }
        };

        // Create documents with embeddings
        let mut vector_documents = Vec::new();
        for (idx, ((content, metadata), embedding)) in documents
            .into_iter()
            .zip(embeddings.into_iter())
            .enumerate()
        {
            if embedding.is_empty() {
                // Skip documents that failed embedding
                continue;
            }

            let document = VectorDocument {
                id: Uuid::new_v4().to_string(),
                content,
                embedding,
                metadata,
                stored_at: chrono::Utc::now(),
            };
            vector_documents.push((idx, document));
        }

        // Store documents in Qdrant
        for (original_idx, document) in vector_documents {
            match self.store_document_with_embedding(&document).await {
                Ok(_) => successful.push(document),
                Err(e) => failed.push(BatchError {
                    index: original_idx,
                    document_id: Some(document.id),
                    error: e.to_string(),
                }),
            }
        }

        // Update statistics
        self.update_embedding_stats(start_time.elapsed().as_millis() as f64)
            .await;

        info!(
            "Batch store completed: {} successful, {} failed",
            successful.len(),
            failed.len()
        );

        let total_attempted = successful.len() + failed.len();
        Ok(BatchResult {
            successful,
            failed,
            total_attempted,
        })
    }

    /// Retrieve multiple documents by IDs
    #[instrument(skip(self))]
    pub async fn retrieve_batch(
        &self,
        ids: Vec<String>,
    ) -> VectorResult<BatchResult<VectorDocument>> {
        let collection = &self.default_collection;

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        // Convert IDs to Qdrant point IDs
        let point_ids: Vec<qdrant_client::qdrant::PointId> =
            ids.iter().map(|id| id.clone().into()).collect();

        let get_request = GetPointsBuilder::new(collection, point_ids)
            .with_payload(true)
            .with_vectors(true);

        let response = self
            .qdrant_client
            .client()
            .get_points(get_request)
            .await
            .map_err(|e| VectorError::from_operation_failed("retrieve_batch", e.to_string()))?;

        let found_ids: std::collections::HashSet<String> = response
            .result
            .iter()
            .filter_map(|point| {
                point.id.as_ref().map(|id| {
                    if let Some(point_id_options) = &id.point_id_options {
                        match point_id_options {
                            qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid) => {
                                uuid.clone()
                            }
                            qdrant_client::qdrant::point_id::PointIdOptions::Num(num) => {
                                num.to_string()
                            }
                        }
                    } else {
                        "unknown".to_string()
                    }
                })
            })
            .collect();

        for (idx, id) in ids.iter().enumerate() {
            if found_ids.contains(id) {
                // Find the corresponding point
                if let Some(point) = response.result.iter().find(|p| {
                    p.id.as_ref().map(|pid| {
                        if let Some(point_id_options) = &pid.point_id_options {
                            match point_id_options {
                                qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid) => {
                                    uuid.clone()
                                }
                                qdrant_client::qdrant::point_id::PointIdOptions::Num(num) => {
                                    num.to_string()
                                }
                            }
                        } else {
                            "unknown".to_string()
                        }
                    }) == Some(id.clone())
                }) {
                    if let Some(vector_data) = &point.vectors {
                        if let Some(
                            qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(
                                dense_vector,
                            ),
                        ) = &vector_data.vectors_options
                        {
                            match self.build_document_from_retrieved_point(
                                point,
                                dense_vector.data.clone(),
                            ) {
                                Ok(document) => successful.push(document),
                                Err(e) => failed.push(BatchError {
                                    index: idx,
                                    document_id: Some(id.clone()),
                                    error: e.to_string(),
                                }),
                            }
                        }
                    }
                }
            } else {
                failed.push(BatchError {
                    index: idx,
                    document_id: Some(id.clone()),
                    error: "Document not found".to_string(),
                });
            }
        }

        Ok(BatchResult {
            successful,
            failed,
            total_attempted: ids.len(),
        })
    }

    /// Delete multiple documents by IDs
    #[instrument(skip(self))]
    pub async fn delete_batch(&self, ids: Vec<String>) -> VectorResult<BatchResult<String>> {
        let collection = &self.default_collection;

        // Update batch operation count
        {
            let mut stats = self.stats.write().await;
            stats.total_batch_operations += 1;
        }

        let point_ids: Vec<qdrant_client::qdrant::PointId> =
            ids.iter().map(|id| id.clone().into()).collect();

        let delete_request = DeletePointsBuilder::new(collection).points(point_ids);

        let response = self
            .qdrant_client
            .client()
            .delete_points(delete_request)
            .await
            .map_err(|e| VectorError::from_operation_failed("delete_batch", e.to_string()))?;

        let deleted_count = response.result.map(|r| r.status).unwrap_or(0) as usize;

        // Determine which deletions were successful
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        // Note: Qdrant doesn't provide per-ID success/failure info for batch deletes
        // We assume the first `deleted_count` IDs were successfully deleted
        for (idx, id) in ids.iter().enumerate() {
            if idx < deleted_count {
                successful.push(id.clone());
            } else {
                failed.push(BatchError {
                    index: idx,
                    document_id: Some(id.clone()),
                    error: "Delete operation failed".to_string(),
                });
            }
        }

        // Update document count
        {
            let mut stats = self.stats.write().await;
            if stats.total_documents >= deleted_count as u64 {
                stats.total_documents -= deleted_count as u64;
            }
        }

        info!(
            "Batch delete completed: {} successful, {} failed",
            successful.len(),
            failed.len()
        );

        Ok(BatchResult {
            successful,
            failed,
            total_attempted: ids.len(),
        })
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> VectorResult<VectorStorageStats> {
        let mut stats = self.stats.read().await.clone();

        // Get embedding service stats
        let embedding_stats = self.embedding_service.get_stats().await;
        stats.embedding_cache_hit_rate = embedding_stats.cache_hit_rate;
        stats.avg_embedding_time_ms = embedding_stats.avg_generation_time_ms;

        Ok(stats)
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) -> VectorResult<()> {
        self.embedding_service.clear_cache().await?;
        info!("Vector storage cache cleared");
        Ok(())
    }

    /// Update embedding statistics
    async fn update_embedding_stats(&self, generation_time_ms: f64) {
        let mut stats = self.stats.write().await;
        let current_avg = stats.avg_embedding_time_ms;
        let current_count = stats.total_documents;

        if current_count == 0 {
            stats.avg_embedding_time_ms = generation_time_ms;
        } else {
            stats.avg_embedding_time_ms = (current_avg * current_count as f64 + generation_time_ms)
                / (current_count + 1) as f64;
        }
    }

    /// Update search statistics
    async fn update_search_stats(&self, search_time_ms: f64) {
        let mut stats = self.stats.write().await;
        stats.total_searches += 1;

        let current_avg = stats.avg_search_latency_ms;
        let current_count = stats.total_searches;

        if current_count == 1 {
            stats.avg_search_latency_ms = search_time_ms;
        } else {
            stats.avg_search_latency_ms =
                (current_avg * (current_count - 1) as f64 + search_time_ms) / current_count as f64;
        }
    }

    /// Get the default collection name
    pub fn default_collection(&self) -> &str {
        &self.default_collection
    }

    /// Get embedding dimension from the embedding service
    pub fn embedding_dimension(&self) -> usize {
        self.embedding_service.embedding_dimension()
    }
}

/// Trait for vector storage operations (for testing and alternative implementations)
#[async_trait]
pub trait VectorStorageService: Send + Sync {
    /// Store a single document
    async fn store_document(
        &self,
        content: &str,
        metadata: DocumentMetadata,
    ) -> VectorResult<VectorDocument>;

    /// Retrieve similar documents
    async fn retrieve_similar(
        &self,
        query: &str,
        config: SearchConfig,
    ) -> VectorResult<Vec<SimilaritySearchResult>>;

    /// Retrieve document by ID
    async fn retrieve_by_id(&self, id: &str) -> VectorResult<Option<VectorDocument>>;

    /// Update an existing document
    async fn update_document(
        &self,
        id: &str,
        content: &str,
        metadata: DocumentMetadata,
    ) -> VectorResult<VectorDocument>;

    /// Delete a document
    async fn delete_document(&self, id: &str) -> VectorResult<bool>;

    /// Store multiple documents
    async fn store_documents(
        &self,
        documents: Vec<(String, DocumentMetadata)>,
    ) -> VectorResult<BatchResult<VectorDocument>>;

    /// Retrieve multiple documents
    async fn retrieve_batch(&self, ids: Vec<String>) -> VectorResult<BatchResult<VectorDocument>>;

    /// Delete multiple documents
    async fn delete_batch(&self, ids: Vec<String>) -> VectorResult<BatchResult<String>>;

    /// Get storage statistics
    async fn get_stats(&self) -> VectorResult<VectorStorageStats>;

    /// Initialize the service
    async fn initialize(&self) -> VectorResult<()>;
}

#[async_trait]
impl VectorStorageService for VectorStorage {
    async fn store_document(
        &self,
        content: &str,
        metadata: DocumentMetadata,
    ) -> VectorResult<VectorDocument> {
        self.store_document(content, metadata).await
    }

    async fn retrieve_similar(
        &self,
        query: &str,
        config: SearchConfig,
    ) -> VectorResult<Vec<SimilaritySearchResult>> {
        self.retrieve_similar(query, config).await
    }

    async fn retrieve_by_id(&self, id: &str) -> VectorResult<Option<VectorDocument>> {
        self.retrieve_by_id(id).await
    }

    async fn update_document(
        &self,
        id: &str,
        content: &str,
        metadata: DocumentMetadata,
    ) -> VectorResult<VectorDocument> {
        self.update_document(id, content, metadata).await
    }

    async fn delete_document(&self, id: &str) -> VectorResult<bool> {
        self.delete_document(id).await
    }

    async fn store_documents(
        &self,
        documents: Vec<(String, DocumentMetadata)>,
    ) -> VectorResult<BatchResult<VectorDocument>> {
        self.store_documents(documents).await
    }

    async fn retrieve_batch(&self, ids: Vec<String>) -> VectorResult<BatchResult<VectorDocument>> {
        self.retrieve_batch(ids).await
    }

    async fn delete_batch(&self, ids: Vec<String>) -> VectorResult<BatchResult<String>> {
        self.delete_batch(ids).await
    }

    async fn get_stats(&self) -> VectorResult<VectorStorageStats> {
        self.get_stats().await
    }

    async fn initialize(&self) -> VectorResult<()> {
        self.initialize().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::{config::VectorConfig, embeddings::EmbeddingConfig};
    use std::sync::Arc;

    // Mock implementations for testing
    #[allow(dead_code)]
    struct MockQdrantClient {
        default_collection: String,
    }

    impl MockQdrantClient {
        #[allow(dead_code)]
        fn new() -> Arc<Self> {
            Arc::new(Self {
                default_collection: "test_collection".to_string(),
            })
        }

        #[allow(dead_code)]
        fn default_collection(&self) -> &str {
            &self.default_collection
        }

        #[allow(dead_code)]
        async fn ensure_collection(&self, _collection_name: &str) -> VectorResult<()> {
            Ok(())
        }
    }

    fn create_test_storage() -> VectorStorage {
        let embedding_config = EmbeddingConfig::default();
        let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));

        // Note: This creates a mock-like setup for testing
        // In a real test, you would use actual mock implementations
        let qdrant_config = VectorConfig::default();
        let qdrant_client = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(QdrantClient::new(qdrant_config))
            .unwrap();

        VectorStorage::new(Arc::new(qdrant_client), embedding_service)
    }

    #[test]
    fn test_document_metadata_default() {
        let metadata = DocumentMetadata::default();
        assert_eq!(metadata.content_type, "document");
        assert!(metadata.tags.is_empty());
        assert!(metadata.custom_fields.is_empty());
        assert!(metadata.research_type.is_none());
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.limit, 10);
        assert!(config.threshold.is_none());
        assert!(config.collection.is_none());
        assert!(config.filters.is_empty());
    }

    #[test]
    fn test_batch_result_basic_creation() {
        let result: BatchResult<String> = BatchResult {
            successful: vec!["doc1".to_string(), "doc2".to_string()],
            failed: vec![BatchError {
                index: 2,
                document_id: Some("doc3".to_string()),
                error: "Test error".to_string(),
            }],
            total_attempted: 3,
        };

        assert_eq!(result.successful.len(), 2);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.total_attempted, 3);
    }

    #[tokio::test]
    async fn test_vector_storage_stats() {
        let storage = create_test_storage();
        let stats = storage.get_stats().await.unwrap();

        assert_eq!(stats.total_documents, 0);
        assert_eq!(stats.total_searches, 0);
        assert_eq!(stats.total_batch_operations, 0);
    }

    #[test]
    fn test_vector_document_creation() {
        let metadata = DocumentMetadata {
            research_type: Some(ResearchType::Learning),
            content_type: "query".to_string(),
            quality_score: Some(0.8),
            source: Some("test".to_string()),
            tags: vec!["rust".to_string(), "async".to_string()],
            custom_fields: HashMap::new(),
        };

        let document = VectorDocument {
            id: "test-id".to_string(),
            content: "Test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: metadata.clone(),
            stored_at: chrono::Utc::now(),
        };

        assert_eq!(document.id, "test-id");
        assert_eq!(document.content, "Test content");
        assert_eq!(document.embedding.len(), 3);
        assert_eq!(
            document.metadata.research_type,
            Some(ResearchType::Learning)
        );
        assert_eq!(document.metadata.tags.len(), 2);
    }

    #[test]
    fn test_similarity_search_result() {
        let metadata = DocumentMetadata::default();
        let document = VectorDocument {
            id: "test-id".to_string(),
            content: "Test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata,
            stored_at: chrono::Utc::now(),
        };

        let result = SimilaritySearchResult {
            document: document.clone(),
            score: 0.95,
        };

        assert_eq!(result.score, 0.95);
        assert_eq!(result.document.id, "test-id");
    }

    #[test]
    fn test_document_metadata_creation() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("priority".to_string(), serde_json::json!("high"));
        custom_fields.insert("version".to_string(), serde_json::json!(1));

        let metadata = DocumentMetadata {
            research_type: Some(ResearchType::Learning),
            content_type: "research_paper".to_string(),
            quality_score: Some(0.92),
            source: Some("arxiv.org".to_string()),
            tags: vec!["machine_learning".to_string(), "nlp".to_string()],
            custom_fields,
        };

        assert_eq!(metadata.research_type, Some(ResearchType::Learning));
        assert_eq!(metadata.content_type, "research_paper");
        assert_eq!(metadata.quality_score, Some(0.92));
        assert_eq!(metadata.source, Some("arxiv.org".to_string()));
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.custom_fields.len(), 2);
        assert_eq!(metadata.custom_fields.get("priority").unwrap(), "high");
    }

    #[test]
    fn test_search_config_creation() {
        let filters = vec![
            SearchFilter {
                field: "research_type".to_string(),
                operation: FilterOperation::Equals,
                value: serde_json::json!("Learning"),
            },
            SearchFilter {
                field: "quality_score".to_string(),
                operation: FilterOperation::GreaterThan,
                value: serde_json::json!(0.8),
            },
        ];

        let config = SearchConfig {
            limit: 20,
            threshold: Some(0.7),
            collection: Some("custom_collection".to_string()),
            filters,
        };

        assert_eq!(config.limit, 20);
        assert_eq!(config.threshold, Some(0.7));
        assert_eq!(config.collection, Some("custom_collection".to_string()));
        assert_eq!(config.filters.len(), 2);
        assert!(matches!(
            config.filters[0].operation,
            FilterOperation::Equals
        ));
        assert!(matches!(
            config.filters[1].operation,
            FilterOperation::GreaterThan
        ));
    }

    #[test]
    fn test_filter_operation_types() {
        let operations = vec![
            FilterOperation::Equals,
            FilterOperation::NotEquals,
            FilterOperation::Contains,
            FilterOperation::GreaterThan,
            FilterOperation::LessThan,
            FilterOperation::In,
            FilterOperation::NotIn,
        ];

        // Test that all operations can be created and serialized
        for operation in operations {
            let filter = SearchFilter {
                field: "test_field".to_string(),
                operation: operation.clone(),
                value: serde_json::json!("test_value"),
            };

            assert_eq!(filter.field, "test_field");
            assert_eq!(filter.value, serde_json::json!("test_value"));
        }
    }

    #[test]
    fn test_batch_result_creation() {
        let successful_docs = vec![
            VectorDocument {
                id: "doc1".to_string(),
                content: "Content 1".to_string(),
                embedding: vec![0.1, 0.2],
                metadata: DocumentMetadata::default(),
                stored_at: chrono::Utc::now(),
            },
            VectorDocument {
                id: "doc2".to_string(),
                content: "Content 2".to_string(),
                embedding: vec![0.3, 0.4],
                metadata: DocumentMetadata::default(),
                stored_at: chrono::Utc::now(),
            },
        ];

        let failed_operations = vec![
            BatchError {
                index: 2,
                document_id: Some("doc3".to_string()),
                error: "Validation failed".to_string(),
            },
            BatchError {
                index: 4,
                document_id: Some("doc5".to_string()),
                error: "Network timeout".to_string(),
            },
        ];

        let batch_result = BatchResult {
            successful: successful_docs,
            failed: failed_operations,
            total_attempted: 5,
        };

        assert_eq!(batch_result.successful.len(), 2);
        assert_eq!(batch_result.failed.len(), 2);
        assert_eq!(batch_result.total_attempted, 5);
        assert_eq!(batch_result.successful[0].id, "doc1");
        assert_eq!(batch_result.failed[0].index, 2);
    }

    #[test]
    fn test_batch_error_structure() {
        let error = BatchError {
            index: 5,
            document_id: Some("failed_doc".to_string()),
            error: "Embedding generation failed".to_string(),
        };

        assert_eq!(error.index, 5);
        assert_eq!(error.document_id, Some("failed_doc".to_string()));
        assert_eq!(error.error, "Embedding generation failed");
    }

    #[test]
    fn test_vector_storage_stats_structure() {
        let stats = VectorStorageStats {
            total_documents: 1000,
            total_searches: 250,
            total_batch_operations: 15,
            avg_search_latency_ms: 45.7,
            avg_embedding_time_ms: 12.3,
            embedding_cache_hit_rate: 0.85,
        };

        assert_eq!(stats.total_documents, 1000);
        assert_eq!(stats.total_searches, 250);
        assert_eq!(stats.total_batch_operations, 15);
        assert_eq!(stats.avg_search_latency_ms, 45.7);
        assert_eq!(stats.avg_embedding_time_ms, 12.3);
        assert_eq!(stats.embedding_cache_hit_rate, 0.85);
    }

    #[test]
    fn test_similarity_search_result_serialization() {
        let metadata = DocumentMetadata {
            research_type: Some(ResearchType::Learning),
            content_type: "test".to_string(),
            quality_score: Some(0.9),
            source: None,
            tags: vec!["test".to_string()],
            custom_fields: HashMap::new(),
        };

        let document = VectorDocument {
            id: "test-id".to_string(),
            content: "Test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata,
            stored_at: chrono::Utc::now(),
        };

        let result = SimilaritySearchResult {
            document,
            score: 0.95,
        };

        let serialized = serde_json::to_string(&result).expect("Failed to serialize result");
        let deserialized: SimilaritySearchResult =
            serde_json::from_str(&serialized).expect("Failed to deserialize result");

        assert_eq!(result.score, deserialized.score);
        assert_eq!(result.document.id, deserialized.document.id);
        assert_eq!(result.document.content, deserialized.document.content);
        assert_eq!(
            result.document.embedding.len(),
            deserialized.document.embedding.len()
        );
    }

    #[test]
    fn test_document_metadata_serialization() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("category".to_string(), serde_json::json!("research"));
        custom_fields.insert("importance".to_string(), serde_json::json!(8));

        let metadata = DocumentMetadata {
            research_type: Some(ResearchType::Validation),
            content_type: "article".to_string(),
            quality_score: Some(0.88),
            source: Some("nature.com".to_string()),
            tags: vec!["biology".to_string(), "genetics".to_string()],
            custom_fields,
        };

        let serialized = serde_json::to_string(&metadata).expect("Failed to serialize metadata");
        let deserialized: DocumentMetadata =
            serde_json::from_str(&serialized).expect("Failed to deserialize metadata");

        assert_eq!(metadata.research_type, deserialized.research_type);
        assert_eq!(metadata.content_type, deserialized.content_type);
        assert_eq!(metadata.quality_score, deserialized.quality_score);
        assert_eq!(metadata.source, deserialized.source);
        assert_eq!(metadata.tags, deserialized.tags);
        assert_eq!(
            metadata.custom_fields.len(),
            deserialized.custom_fields.len()
        );
    }

    #[test]
    fn test_search_config_serialization() {
        let config = SearchConfig {
            limit: 50,
            threshold: Some(0.75),
            collection: Some("test_collection".to_string()),
            filters: vec![SearchFilter {
                field: "type".to_string(),
                operation: FilterOperation::Equals,
                value: serde_json::json!("research"),
            }],
        };

        let serialized = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: SearchConfig =
            serde_json::from_str(&serialized).expect("Failed to deserialize config");

        assert_eq!(config.limit, deserialized.limit);
        assert_eq!(config.threshold, deserialized.threshold);
        assert_eq!(config.collection, deserialized.collection);
        assert_eq!(config.filters.len(), deserialized.filters.len());
    }

    #[test]
    fn test_vector_document_with_large_embedding() {
        let large_embedding = (0..1024).map(|i| i as f32 / 1024.0).collect::<Vec<f32>>();

        let document = VectorDocument {
            id: "large_embedding_doc".to_string(),
            content: "Document with large embedding vector".to_string(),
            embedding: large_embedding.clone(),
            metadata: DocumentMetadata::default(),
            stored_at: chrono::Utc::now(),
        };

        assert_eq!(document.embedding.len(), 1024);
        assert_eq!(document.embedding[0], 0.0);
        assert_eq!(document.embedding[1023], 1023.0 / 1024.0);
        assert_eq!(document.id, "large_embedding_doc");
    }

    #[test]
    fn test_document_metadata_edge_cases() {
        // Test with empty values
        let empty_metadata = DocumentMetadata {
            research_type: None,
            content_type: "".to_string(),
            quality_score: None,
            source: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
        };

        assert!(empty_metadata.research_type.is_none());
        assert!(empty_metadata.content_type.is_empty());
        assert!(empty_metadata.quality_score.is_none());
        assert!(empty_metadata.source.is_none());
        assert!(empty_metadata.tags.is_empty());
        assert!(empty_metadata.custom_fields.is_empty());

        // Test with maximum values
        let max_metadata = DocumentMetadata {
            research_type: Some(ResearchType::Learning),
            content_type: "a".repeat(1000),
            quality_score: Some(1.0),
            source: Some("https://very-long-domain-name.example.com/path/to/resource".to_string()),
            tags: (0..100).map(|i| format!("tag_{}", i)).collect(),
            custom_fields: (0..50)
                .map(|i| (format!("field_{}", i), serde_json::json!(i)))
                .collect(),
        };

        assert_eq!(max_metadata.content_type.len(), 1000);
        assert_eq!(max_metadata.tags.len(), 100);
        assert_eq!(max_metadata.custom_fields.len(), 50);
        assert_eq!(max_metadata.quality_score, Some(1.0));
    }

    #[test]
    fn test_search_filter_value_types() {
        let string_filter = SearchFilter {
            field: "category".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!("research"),
        };

        let number_filter = SearchFilter {
            field: "score".to_string(),
            operation: FilterOperation::GreaterThan,
            value: serde_json::json!(0.8),
        };

        let array_filter = SearchFilter {
            field: "tags".to_string(),
            operation: FilterOperation::In,
            value: serde_json::json!(["rust", "async", "programming"]),
        };

        let bool_filter = SearchFilter {
            field: "published".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!(true),
        };

        assert_eq!(string_filter.value, serde_json::json!("research"));
        assert_eq!(number_filter.value, serde_json::json!(0.8));
        assert_eq!(
            array_filter.value,
            serde_json::json!(["rust", "async", "programming"])
        );
        assert_eq!(bool_filter.value, serde_json::json!(true));
    }

    #[test]
    fn test_batch_result_success_rate_calculation() {
        let batch_result = BatchResult {
            successful: vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()],
            failed: vec![BatchError {
                index: 3,
                document_id: Some("doc4".to_string()),
                error: "Error 1".to_string(),
            }],
            total_attempted: 4,
        };

        let success_rate =
            batch_result.successful.len() as f64 / batch_result.total_attempted as f64;
        assert_eq!(success_rate, 0.75); // 3 out of 4 successful

        let failure_rate = batch_result.failed.len() as f64 / batch_result.total_attempted as f64;
        assert_eq!(failure_rate, 0.25); // 1 out of 4 failed
    }

    #[test]
    fn test_vector_storage_stats_calculations() {
        let mut stats = VectorStorageStats {
            total_documents: 0,
            total_searches: 0,
            total_batch_operations: 0,
            avg_search_latency_ms: 0.0,
            avg_embedding_time_ms: 0.0,
            embedding_cache_hit_rate: 0.0,
        };

        // Simulate adding documents
        stats.total_documents += 100;
        assert_eq!(stats.total_documents, 100);

        // Simulate search operations with running average
        let search_times = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        for (_i, time) in search_times.iter().enumerate() {
            stats.total_searches += 1;
            let current_avg = stats.avg_search_latency_ms;
            let current_count = stats.total_searches as f64;
            stats.avg_search_latency_ms =
                (current_avg * (current_count - 1.0) + time) / current_count;
        }

        assert_eq!(stats.total_searches, 5);
        assert_eq!(stats.avg_search_latency_ms, 30.0); // Average of 10,20,30,40,50

        // Test cache hit rate calculation
        let total_requests = 100.0;
        let cache_hits = 85.0;
        stats.embedding_cache_hit_rate = cache_hits / total_requests;
        assert_eq!(stats.embedding_cache_hit_rate, 0.85);
    }
}
