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

// ABOUTME: Research endpoint handlers for API server
// Provides HTTP endpoints for research pipeline integration with full fortitude-core integration

use crate::extractors::SafeQuery;
use crate::middleware::auth::{Claims, Permission};
use crate::models::{
    errors::ApiError,
    requests::{ResearchListRequest, ResearchRequest},
    responses::{
        ApiResponse, Detail, Evidence, PaginationInfo, ResearchListResponse, ResearchMetadata,
        ResearchResponse, ResearchSummary,
    },
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use fortitude_core::api::ClaudeConfig;
use fortitude_core::{
    BasicClassifier, ClaudeResearchEngine, FileStorage, PipelineBuilder, ResearchPipeline,
};
use fortitude_types::{
    AudienceContext, CacheOperation, CacheOperationType, ClassificationConfig, ClassificationError,
    DomainContext, PipelineError, ResearchType, SearchQuery, Storage, StorageConfig, StorageError,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument};
use utoipa;
use uuid::Uuid;
use validator::Validate;

/// Research service state
#[derive(Clone)]
pub struct ResearchState {
    pub pipeline: Arc<ResearchPipeline>,
}

impl ResearchState {
    /// Create new research state with pipeline
    pub async fn new() -> Result<Self, ApiError> {
        // Initialize storage
        let storage_config = StorageConfig::default();
        let storage = Arc::new(FileStorage::new(storage_config).await.map_err(|e| {
            ApiError::InternalError {
                message: format!("Failed to initialize storage: {e}"),
            }
        })?);

        // Initialize classifier
        let classification_config = ClassificationConfig::default();
        let classifier = Arc::new(BasicClassifier::new(classification_config));

        // Try to initialize research engine with Claude API
        let research_engine = if let Ok(api_key) = std::env::var("CLAUDE_API_KEY") {
            let claude_config = ClaudeConfig::new(api_key);
            match ClaudeResearchEngine::new(claude_config) {
                Ok(engine) => {
                    Some(Arc::new(engine) as Arc<dyn fortitude_core::ResearchEngine + Send + Sync>)
                }
                Err(e) => {
                    error!("Failed to initialize Claude research engine: {}", e);
                    None
                }
            }
        } else {
            debug!("CLAUDE_API_KEY not set, research will use placeholder responses");
            None
        };

        // Build pipeline
        let builder = PipelineBuilder::new()
            .with_caching(true)
            .with_context_detection(true)
            .with_advanced_classification(false); // Start with basic classification

        let pipeline = if let Some(engine) = research_engine {
            builder
                .with_research_engine(engine)
                .build(classifier, storage)
        } else {
            builder.build(classifier, storage)
        };

        Ok(Self {
            pipeline: Arc::new(pipeline),
        })
    }
}

/// Submit a new research request
///
/// Processes a research query through the full pipeline including:
/// - Classification of research type
/// - Context detection (audience, domain, urgency)
/// - Research generation using Claude API or fallback
/// - Result caching for future retrieval
#[utoipa::path(
    post,
    path = "/api/v1/research",
    request_body = ResearchRequest,
    responses(
        (status = 201, description = "Research request submitted successfully", body = ApiResponse<ResearchResponse>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Research"
)]
#[instrument(skip(state, claims_ext))]
pub async fn submit_research(
    State(state): State<ResearchState>,
    claims_ext: Option<Extension<Claims>>,
    Json(request): Json<ResearchRequest>,
) -> Result<Response, ApiError> {
    let start_time = Instant::now();

    // Validate request
    request.validate().map_err(|e| ApiError::BadRequest {
        message: format!("Request validation failed: {e}"),
    })?;

    if let Some(Extension(claims)) = claims_ext.as_ref() {
        info!(
            "Processing research request: '{}' for user: {}",
            request.query, claims.sub
        );
        // Check permission
        check_research_permission(claims)?;
    } else {
        info!(
            "Processing research request: '{}' (auth disabled)",
            request.query
        );
    }

    // Convert request contexts to pipeline types
    let audience_context = request.audience_context.map(|ctx| AudienceContext {
        level: ctx.level,
        domain: ctx.domain,
        format: ctx.format,
    });

    let domain_context = request.domain_context.map(|ctx| DomainContext {
        technology: ctx.technology,
        project_type: ctx.project_type,
        frameworks: ctx.frameworks,
        tags: ctx.tags,
    });

    // Process through pipeline
    let result = state
        .pipeline
        .process_query(&request.query, audience_context, domain_context)
        .await
        .map_err(convert_pipeline_error)?;

    let processing_time = start_time.elapsed();

    // Record cache operation for performance monitoring
    let cache_operation = CacheOperation {
        timestamp: chrono::Utc::now(),
        operation_type: if result.metadata.cache_key.starts_with("enhanced-") {
            CacheOperationType::Hit // Assume enhanced keys indicate cache hits
        } else {
            CacheOperationType::Miss // Regular processing indicates cache miss
        },
        cache_key: result.metadata.cache_key.clone(),
        duration_ms: processing_time.as_millis() as u64,
        success: true,
        context: std::collections::HashMap::from([
            ("endpoint".to_string(), "research".to_string()),
            (
                "research_type".to_string(),
                result.request.research_type.to_string(),
            ),
            (
                "user".to_string(),
                claims_ext
                    .as_ref()
                    .map(|ext| ext.0.sub.clone())
                    .unwrap_or_else(|| "anonymous".to_string()),
            ),
        ]),
    };

    // In a production system, we'd record this operation to the storage system
    // For now, we'll just log it for monitoring purposes
    debug!(
        "Cache operation: {:?} for key: {} in {}ms",
        cache_operation.operation_type, cache_operation.cache_key, cache_operation.duration_ms
    );

    // Convert to API response format
    let response = ResearchResponse {
        id: result.cache_key().to_string(),
        query: result.original_query().to_string(),
        research_type: result.research_type().to_string(),
        immediate_answer: result.immediate_answer.clone(),
        supporting_evidence: result
            .supporting_evidence
            .iter()
            .map(|e| Evidence {
                source: e.source.clone(),
                content: e.content.clone(),
                relevance: e.relevance,
                evidence_type: e.evidence_type.clone(),
            })
            .collect(),
        implementation_details: result
            .implementation_details
            .iter()
            .map(|d| Detail {
                category: d.category.clone(),
                content: d.content.clone(),
                priority: d.priority.clone(),
                prerequisites: d.prerequisites.clone(),
            })
            .collect(),
        metadata: ResearchMetadata {
            completed_at: result.metadata.completed_at,
            processing_time_ms: result.metadata.processing_time_ms,
            sources_consulted: result.metadata.sources_consulted.clone(),
            quality_score: result.metadata.quality_score,
            tags: result.metadata.tags.clone(),
        },
        processing_time_ms: processing_time.as_millis() as u64,
    };

    let api_response = ApiResponse::success(response, Uuid::new_v4());

    info!(
        "Research request completed in {:.2}s for user: {} (quality: {:.2})",
        processing_time.as_secs_f64(),
        claims_ext
            .as_ref()
            .map(|ext| ext.0.sub.as_str())
            .unwrap_or("anonymous"),
        result.metadata.quality_score
    );

    Ok((StatusCode::CREATED, Json(api_response)).into_response())
}

/// Retrieve a specific research result by ID
///
/// Returns a cached research result using the cache key as the ID.
/// Results are stored with content addressing for efficient retrieval.
#[utoipa::path(
    get,
    path = "/api/v1/research/{id}",
    params(
        ("id" = String, Path, description = "Research result ID (cache key)")
    ),
    responses(
        (status = 200, description = "Research result retrieved successfully", body = ApiResponse<ResearchResponse>),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Research result not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Research",
    security(("jwt_auth" = []))
)]
#[instrument(skip(_state, claims))]
pub async fn get_research_by_id(
    State(_state): State<ResearchState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let start_time = Instant::now();

    info!(
        "Retrieving research result: {} for user: {}",
        id, claims.sub
    );

    // Check permission
    check_research_permission(&claims)?;

    // Create temporary storage to retrieve directly
    // TODO: Expose storage through ResearchPipeline in a future version
    let storage_config = StorageConfig::default();
    let storage = FileStorage::new(storage_config)
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Failed to initialize storage: {e}"),
        })?;

    // Retrieve from cache using Storage trait
    let result = storage
        .retrieve(&id)
        .await
        .map_err(convert_storage_error)?
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Research result with ID: {id}"),
        })?;

    let processing_time = start_time.elapsed();

    // Record cache hit operation for performance monitoring
    let cache_operation = CacheOperation {
        timestamp: chrono::Utc::now(),
        operation_type: CacheOperationType::Hit, // Successful retrieval is a cache hit
        cache_key: id.clone(),
        duration_ms: processing_time.as_millis() as u64,
        success: true,
        context: std::collections::HashMap::from([
            ("endpoint".to_string(), "research_retrieve".to_string()),
            (
                "research_type".to_string(),
                result.request.research_type.to_string(),
            ),
            ("user".to_string(), claims.sub.clone()),
        ]),
    };

    debug!(
        "Cache hit for research retrieval: {} in {}ms",
        cache_operation.cache_key, cache_operation.duration_ms
    );

    // Convert to API response format
    let response = ResearchResponse {
        id: result.cache_key().to_string(),
        query: result.original_query().to_string(),
        research_type: result.research_type().to_string(),
        immediate_answer: result.immediate_answer.clone(),
        supporting_evidence: result
            .supporting_evidence
            .iter()
            .map(|e| Evidence {
                source: e.source.clone(),
                content: e.content.clone(),
                relevance: e.relevance,
                evidence_type: e.evidence_type.clone(),
            })
            .collect(),
        implementation_details: result
            .implementation_details
            .iter()
            .map(|d| Detail {
                category: d.category.clone(),
                content: d.content.clone(),
                priority: d.priority.clone(),
                prerequisites: d.prerequisites.clone(),
            })
            .collect(),
        metadata: ResearchMetadata {
            completed_at: result.metadata.completed_at,
            processing_time_ms: result.metadata.processing_time_ms,
            sources_consulted: result.metadata.sources_consulted.clone(),
            quality_score: result.metadata.quality_score,
            tags: result.metadata.tags.clone(),
        },
        processing_time_ms: processing_time.as_millis() as u64,
    };

    let api_response = ApiResponse::success(response, Uuid::new_v4());

    info!(
        "Research result retrieved in {:.2}ms for user: {}",
        processing_time.as_millis(),
        claims.sub
    );

    Ok((StatusCode::OK, Json(api_response)).into_response())
}

/// List research results with filtering and pagination
///
/// Searches through cached research results with support for:
/// - Research type filtering
/// - Keyword search across content
/// - Quality score filtering
/// - Tag-based filtering
/// - Pagination with configurable limits
#[utoipa::path(
    get,
    path = "/api/v1/research",
    params(
        ResearchListRequest
    ),
    responses(
        (status = 200, description = "Research results retrieved successfully", body = ApiResponse<ResearchListResponse>),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Research",
    security(("jwt_auth" = []))
)]
#[instrument(skip(state, claims))]
pub async fn list_research_results(
    State(state): State<ResearchState>,
    Extension(claims): Extension<Claims>,
    SafeQuery(request): SafeQuery<ResearchListRequest>,
) -> Result<Response, ApiError> {
    let start_time = Instant::now();

    // Validation is already handled by SafeQuery extractor

    info!(
        "Listing research results for user: {} with filters: {:?}",
        claims.sub, request
    );

    // Check permission
    check_research_permission(&claims)?;

    // Build search query
    let search_query = build_search_query(&request)?;

    // Search through cached results
    let search_results = state
        .pipeline
        .search_results(&search_query)
        .await
        .map_err(convert_pipeline_error)?;

    let processing_time = start_time.elapsed();

    // Calculate pagination
    let total_count = search_results.len();
    let limit = request.limit.unwrap_or(10);
    let offset = request.offset.unwrap_or(0);
    let total_pages = (total_count + limit - 1) / limit.max(1);
    let has_more = offset + limit < total_count;

    // Apply pagination to results
    let paginated_results = search_results
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|sr| ResearchSummary {
            id: sr.entry.cache_key.clone(),
            query: sr.entry.original_query.clone(),
            research_type: sr.entry.research_type.to_string(),
            summary: create_summary(&sr.snippet, 200),
            quality_score: sr.entry.quality_score,
            created_at: sr.entry.indexed_at,
            tags: sr.entry.tags.clone(),
        })
        .collect();

    let response = ResearchListResponse {
        results: paginated_results,
        total_count,
        pagination: PaginationInfo {
            offset,
            limit,
            total_pages,
            has_more,
        },
        processing_time_ms: processing_time.as_millis() as u64,
    };

    let api_response = ApiResponse::success(response, Uuid::new_v4());

    info!(
        "Listed {} research results in {:.2}ms for user: {}",
        total_count,
        processing_time.as_millis(),
        claims.sub
    );

    Ok((StatusCode::OK, Json(api_response)).into_response())
}

/// Helper function to check research permission
fn check_research_permission(claims: &Claims) -> Result<(), ApiError> {
    let required_permission = Permission::ResearchRead.as_str();

    // Admin has all permissions
    if claims
        .permissions
        .contains(&Permission::Admin.as_str().to_string())
    {
        return Ok(());
    }

    // Check specific permission
    if !claims
        .permissions
        .contains(&required_permission.to_string())
    {
        return Err(ApiError::Forbidden {
            reason: format!("Permission denied: {required_permission}"),
        });
    }

    Ok(())
}

/// Helper function to build search query from request
fn build_search_query(request: &ResearchListRequest) -> Result<SearchQuery, ApiError> {
    let keywords = request.keywords.clone().unwrap_or_default();

    let mut query = SearchQuery::new(keywords)
        .with_limit(request.limit.unwrap_or(10))
        .with_offset(request.offset.unwrap_or(0));

    // Add research type filter
    if let Some(ref research_type_str) = request.research_type {
        let research_type =
            research_type_str
                .parse::<ResearchType>()
                .map_err(|e| ApiError::BadRequest {
                    message: format!("Invalid research type: {e}"),
                })?;
        query = query.with_research_type(research_type);
    }

    // Add quality filter
    if let Some(min_quality) = request.min_quality {
        query = query.with_min_quality(min_quality);
    }

    // Add tags filter
    if let Some(ref tags) = request.tags {
        query = query.with_tags(tags.clone());
    }

    Ok(query)
}

/// Helper function to create summary from content
fn create_summary(content: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        content.to_string()
    } else {
        let truncated = &content[..max_length];
        let last_space = truncated.rfind(' ').unwrap_or(max_length);
        format!("{}...", &content[..last_space])
    }
}

/// Convert pipeline error to API error
fn convert_pipeline_error(err: PipelineError) -> ApiError {
    match err {
        PipelineError::StageFailed { stage, error } => {
            error!("Pipeline stage '{}' failed: {}", stage, error);
            ApiError::InternalError {
                message: format!("Research processing failed at stage: {stage}"),
            }
        }
        PipelineError::Processing(message) => ApiError::BadRequest { message },
        PipelineError::Config(message) => ApiError::InternalError {
            message: format!("Configuration error: {message}"),
        },
        PipelineError::Timeout => ApiError::InternalError {
            message: "Research processing timed out".to_string(),
        },
        PipelineError::Aborted => ApiError::InternalError {
            message: "Research processing was aborted".to_string(),
        },
    }
}

/// Convert storage error to API error
fn convert_storage_error(err: StorageError) -> ApiError {
    match err {
        StorageError::FileNotFound(_) => ApiError::NotFound {
            resource: "Research result not found".to_string(),
        },
        StorageError::Serialization(_) => ApiError::InternalError {
            message: "Failed to process research data".to_string(),
        },
        StorageError::Io(_) => ApiError::InternalError {
            message: "Storage system error".to_string(),
        },
        StorageError::PermissionDenied(_) => ApiError::Forbidden {
            reason: "Access denied to research data".to_string(),
        },
        _ => {
            error!("Unexpected storage error: {:?}", err);
            ApiError::InternalError {
                message: "Storage system error".to_string(),
            }
        }
    }
}

/// Convert classification error to API error
#[allow(dead_code)]
fn convert_classification_error(err: ClassificationError) -> ApiError {
    match err {
        ClassificationError::InvalidInput(message) => ApiError::BadRequest { message },
        ClassificationError::Failed(message) => ApiError::InternalError {
            message: format!("Classification failed: {message}"),
        },
        ClassificationError::LowConfidence { actual, threshold } => ApiError::BadRequest {
            message: format!(
                "Query classification confidence too low: {actual:.2} < {threshold:.2}"
            ),
        },
        _ => {
            error!("Unexpected classification error: {:?}", err);
            ApiError::InternalError {
                message: "Classification system error".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::Claims;
    use crate::models::requests::ResearchListRequest;

    fn create_test_claims() -> Claims {
        Claims {
            sub: "test_user".to_string(),
            permissions: vec!["fortitude:research:read".to_string()],
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            iss: "fortitude-api-server".to_string(),
        }
    }

    fn create_admin_claims() -> Claims {
        Claims {
            sub: "admin_user".to_string(),
            permissions: vec!["fortitude:admin".to_string()],
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            iss: "fortitude-api-server".to_string(),
        }
    }

    fn create_no_permission_claims() -> Claims {
        Claims {
            sub: "limited_user".to_string(),
            permissions: vec!["fortitude:config:read".to_string()],
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            iss: "fortitude-api-server".to_string(),
        }
    }

    #[test]
    fn test_check_research_permission_success() {
        let claims = create_test_claims();
        assert!(check_research_permission(&claims).is_ok());
    }

    #[test]
    fn test_check_research_permission_admin() {
        let claims = create_admin_claims();
        assert!(check_research_permission(&claims).is_ok());
    }

    #[test]
    fn test_check_research_permission_denied() {
        let claims = create_no_permission_claims();
        assert!(check_research_permission(&claims).is_err());
    }

    #[test]
    fn test_build_search_query_full() {
        let request = ResearchListRequest {
            research_type: Some("implementation".to_string()),
            keywords: Some("rust async".to_string()),
            min_quality: Some(0.8),
            tags: Some(vec!["rust".to_string(), "async".to_string()]),
            limit: Some(20),
            offset: Some(10),
            sort: Some("newest".to_string()),
        };

        let query = build_search_query(&request).unwrap();

        assert_eq!(query.query, "rust async");
        assert_eq!(query.research_type, Some(ResearchType::Implementation));
        assert_eq!(query.min_quality, Some(0.8));
        assert_eq!(query.tags, vec!["rust".to_string(), "async".to_string()]);
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.offset, Some(10));
    }

    #[test]
    fn test_build_search_query_minimal() {
        let request = ResearchListRequest {
            research_type: None,
            keywords: None,
            min_quality: None,
            tags: None,
            limit: None,
            offset: None,
            sort: None,
        };

        let query = build_search_query(&request).unwrap();

        assert_eq!(query.query, "");
        assert_eq!(query.research_type, None);
        assert_eq!(query.min_quality, None);
        assert_eq!(query.tags, Vec::<String>::new());
        assert_eq!(query.limit, Some(10)); // Default
        assert_eq!(query.offset, Some(0)); // Default
    }

    #[test]
    fn test_build_search_query_invalid_research_type() {
        let request = ResearchListRequest {
            research_type: Some("invalid_type".to_string()),
            keywords: None,
            min_quality: None,
            tags: None,
            limit: None,
            offset: None,
            sort: None,
        };

        let result = build_search_query(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::BadRequest { message } => {
                assert!(message.contains("Invalid research type"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_create_summary_short() {
        let content = "Short content";
        let summary = create_summary(content, 100);
        assert_eq!(summary, "Short content");
    }

    #[test]
    fn test_create_summary_long() {
        let content = "This is a very long piece of content that exceeds the maximum length and should be truncated at a word boundary for better readability";
        let summary = create_summary(content, 50);

        assert!(summary.len() <= 53); // 50 + "..."
        assert!(summary.ends_with("..."));
        assert!(!summary.contains("boundary")); // Should be truncated before this word
    }

    #[test]
    fn test_convert_storage_error_file_not_found() {
        let err = StorageError::FileNotFound(std::path::PathBuf::from("/test/path"));
        let api_err = convert_storage_error(err);

        match api_err {
            ApiError::NotFound { resource } => {
                assert_eq!(resource, "Research result not found");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_convert_storage_error_permission_denied() {
        let err = StorageError::PermissionDenied("access denied".to_string());
        let api_err = convert_storage_error(err);

        match api_err {
            ApiError::Forbidden { reason } => {
                assert_eq!(reason, "Access denied to research data");
            }
            _ => panic!("Expected Forbidden error"),
        }
    }

    #[test]
    fn test_convert_classification_error_invalid_input() {
        let err = ClassificationError::InvalidInput("bad input".to_string());
        let api_err = convert_classification_error(err);

        match api_err {
            ApiError::BadRequest { message } => {
                assert_eq!(message, "bad input");
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_convert_classification_error_low_confidence() {
        let err = ClassificationError::LowConfidence {
            actual: 0.3,
            threshold: 0.6,
        };
        let api_err = convert_classification_error(err);

        match api_err {
            ApiError::BadRequest { message } => {
                assert!(message.contains("0.30"));
                assert!(message.contains("0.60"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_convert_pipeline_error_stage_failed() {
        let err = PipelineError::StageFailed {
            stage: "classification".to_string(),
            error: "classification failed".to_string(),
        };
        let api_err = convert_pipeline_error(err);

        match api_err {
            ApiError::InternalError { message } => {
                assert!(message.contains("classification"));
            }
            _ => panic!("Expected InternalError"),
        }
    }
}
