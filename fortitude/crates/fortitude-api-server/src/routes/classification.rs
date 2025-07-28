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

// ABOUTME: Classification endpoint handlers for API server
// Provides HTTP endpoints for advanced classification system integration with full fortitude-core support

use crate::extractors::SafeQuery;
use crate::middleware::auth::{Claims, Permission};
use crate::models::{
    errors::ApiError,
    requests::{ClassificationListRequest, ClassificationOptions, ClassificationRequest},
    responses::{
        ApiResponse, AudienceLevelInfo, ClassificationCandidate as ApiClassificationCandidate,
        ClassificationListResponse, ClassificationMetadata, ClassificationResponse,
        ClassificationResult as ApiClassificationResult, ClassificationSummary,
        ClassificationSystemInfo, ClassificationTypesResponse, ContextDetectionResponse,
        DimensionConfidence, PaginationInfo, ResearchTypeInfo, TechnicalDomainInfo,
        UrgencyLevelInfo,
    },
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use fortitude_core::{
    AdvancedClassifier, BasicClassifier, ContextDetector, FileStorage, FortitudeContextDetector,
};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationMetadata as CoreClassificationMetadata,
        EnhancedClassificationResult, TechnicalDomain, UrgencyLevel,
    },
    ClassificationConfig, ClassificationError, ClassificationResult, Classifier, ResearchType,
    StorageConfig, StorageError,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, instrument};
use utoipa;
use uuid::Uuid;
use validator::Validate;

/// Classification service state
#[derive(Clone)]
pub struct ClassificationState {
    pub basic_classifier: Arc<BasicClassifier>,
    pub advanced_classifier: Option<Arc<AdvancedClassifier>>,
    pub context_detector: Arc<FortitudeContextDetector>,
    pub storage: Arc<FileStorage>,
}

impl ClassificationState {
    /// Create new classification state with classifiers and storage
    pub async fn new() -> Result<Self, ApiError> {
        // Initialize storage
        let storage_config = StorageConfig::default();
        let storage = Arc::new(FileStorage::new(storage_config).await.map_err(|e| {
            ApiError::InternalError {
                message: format!("Failed to initialize storage: {e}"),
            }
        })?);

        // Initialize basic classifier
        let classification_config = ClassificationConfig::default();
        let basic_classifier = Arc::new(BasicClassifier::new(classification_config));

        // Try to initialize advanced classifier
        let config = fortitude_core::AdvancedClassificationConfig::default();
        let classifier = AdvancedClassifier::new(config);
        let advanced_classifier = Some(Arc::new(classifier));

        // Initialize context detector
        let context_detector = Arc::new(FortitudeContextDetector::new());

        Ok(Self {
            basic_classifier,
            advanced_classifier,
            context_detector,
            storage,
        })
    }
}

/// Submit content for classification analysis
///
/// Processes content through the classification pipeline including:
/// - Research type classification using basic or advanced classifier
/// - Context detection for audience, domain, and urgency analysis
/// - Result caching for future retrieval
/// - Multi-dimensional analysis with confidence scoring
#[utoipa::path(
    post,
    path = "/api/v1/classify",
    request_body = ClassificationRequest,
    responses(
        (status = 201, description = "Classification request submitted successfully", body = ApiResponse<ClassificationResponse>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Classification"
)]
#[instrument(skip(state, claims_ext))]
pub async fn submit_classification(
    State(state): State<ClassificationState>,
    claims_ext: Option<Extension<Claims>>,
    Json(request): Json<ClassificationRequest>,
) -> Result<Response, ApiError> {
    let start_time = Instant::now();

    // Validate request
    request.validate().map_err(|e| ApiError::BadRequest {
        message: format!("Request validation failed: {e}"),
    })?;

    if let Some(Extension(claims)) = claims_ext.as_ref() {
        info!(
            "Processing classification request for content length: {} characters, user: {}",
            request.content.len(),
            claims.sub
        );
        // Check permission
        check_classification_permission(claims)?;
    } else {
        info!(
            "Processing classification request for content length: {} characters (auth disabled)",
            request.content.len()
        );
    }

    // Extract options with defaults
    let options = request.options.unwrap_or_default();
    let enable_context = options.enable_context_detection.unwrap_or(true);
    let enable_advanced = options.enable_advanced_classification.unwrap_or(false);
    let confidence_threshold = options.confidence_threshold.unwrap_or(0.6);

    // Perform research type classification
    let classifier_result = if enable_advanced && state.advanced_classifier.is_some() {
        // Use advanced classifier
        let advanced_classifier = state.advanced_classifier.as_ref().unwrap();
        classify_with_custom_threshold(
            &**advanced_classifier,
            &request.content,
            confidence_threshold,
        )?
    } else {
        // Use basic classifier
        classify_with_custom_threshold(
            &*state.basic_classifier,
            &request.content,
            confidence_threshold,
        )?
    };

    // Perform context detection if enabled
    let context_result = if enable_context {
        Some(
            state
                .context_detector
                .detect_context(&request.content, &classifier_result.research_type)
                .map_err(convert_classification_error)?,
        )
    } else {
        None
    };

    // Generate cache key
    let cache_key = generate_cache_key(&request.content, &options);

    // For now, we'll store classification results differently since Storage trait is for ResearchResult
    // In a production system, we'd need a separate classification storage system
    // Create enhanced result for response
    let _enhanced_result = create_enhanced_result(
        &classifier_result,
        context_result.as_ref(),
        enable_advanced,
        enable_context,
    );

    let processing_time = start_time.elapsed();

    // Convert to API response format
    let response = convert_to_api_response(
        &cache_key,
        &request.content,
        &classifier_result,
        context_result.as_ref(),
        enable_advanced,
        enable_context,
        processing_time.as_millis() as u64,
    );

    let api_response = ApiResponse::success(response, Uuid::new_v4());

    if let Some(Extension(claims)) = claims_ext.as_ref() {
        info!(
            "Classification completed in {:.2}ms for user: {} (confidence: {:.2})",
            processing_time.as_millis(),
            claims.sub,
            classifier_result.confidence
        );
    } else {
        info!(
            "Classification completed in {:.2}ms (auth disabled) (confidence: {:.2})",
            processing_time.as_millis(),
            classifier_result.confidence
        );
    }

    Ok((StatusCode::CREATED, Json(api_response)).into_response())
}

/// Retrieve a specific classification result by ID
///
/// Returns a cached classification result using the cache key as the ID.
/// Results are stored with content addressing for efficient retrieval.
#[utoipa::path(
    get,
    path = "/api/v1/classify/{id}",
    params(
        ("id" = String, Path, description = "Classification result ID (cache key)")
    ),
    responses(
        (status = 200, description = "Classification result retrieved successfully", body = ApiResponse<ClassificationResponse>),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Classification result not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Classification",
    security(("jwt_auth" = []))
)]
#[instrument(skip(_state, claims))]
pub async fn get_classification_by_id(
    State(_state): State<ClassificationState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let _start_time = Instant::now();

    info!(
        "Retrieving classification result: {} for user: {}",
        id, claims.sub
    );

    // Check permission
    check_classification_permission(&claims)?;

    // For now, return not found since we don't have classification-specific storage
    // In a production system, we'd have a separate classification storage
    return Err(ApiError::NotFound {
        resource: format!("Classification result with ID: {id}"),
    });
}

/// List classification results with filtering and pagination
///
/// Searches through cached classification results with support for:
/// - Research type filtering
/// - Context dimension filtering (audience, domain, urgency)
/// - Keyword search across content
/// - Confidence score filtering
/// - Pagination with configurable limits
#[utoipa::path(
    get,
    path = "/api/v1/classify",
    params(
        ClassificationListRequest
    ),
    responses(
        (status = 200, description = "Classification results retrieved successfully", body = ApiResponse<ClassificationListResponse>),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Classification",
    security(("jwt_auth" = []))
)]
#[instrument(skip(_state, claims))]
pub async fn list_classification_results(
    State(_state): State<ClassificationState>,
    Extension(claims): Extension<Claims>,
    SafeQuery(request): SafeQuery<ClassificationListRequest>,
) -> Result<Response, ApiError> {
    let start_time = Instant::now();

    // Validation is already handled by SafeQuery extractor

    info!(
        "Listing classification results for user: {} with filters: {:?}",
        claims.sub, request
    );

    // Check permission
    check_classification_permission(&claims)?;

    // Build search criteria from request
    let _search_criteria = build_classification_search_criteria(&request)?;

    // For now, return empty results since we don't have classification-specific storage
    // In a production system, this would search through cached classification results
    let all_results: Vec<EnhancedClassificationResult> = vec![];

    let processing_time = start_time.elapsed();

    // Calculate pagination
    let total_count = all_results.len();
    let limit = request.limit.unwrap_or(10);
    let offset = request.offset.unwrap_or(0);
    let total_pages = (total_count + limit - 1) / limit.max(1);
    let has_more = offset + limit < total_count;

    // Apply pagination to results (empty for now)
    let paginated_results: Vec<ClassificationSummary> = vec![];

    let response = ClassificationListResponse {
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
        "Listed {} classification results in {:.2}ms for user: {}",
        total_count,
        processing_time.as_millis(),
        claims.sub
    );

    Ok((StatusCode::OK, Json(api_response)).into_response())
}

/// Get available classification types and system information
///
/// Returns information about:
/// - Available research types with descriptions and examples
/// - Available audience levels, technical domains, and urgency levels
/// - Classification system capabilities and configuration
#[utoipa::path(
    get,
    path = "/api/v1/classify/types",
    responses(
        (status = 200, description = "Classification types retrieved successfully", body = ApiResponse<ClassificationTypesResponse>),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Classification"
)]
#[instrument(skip(_state, claims_ext))]
pub async fn get_classification_types(
    State(_state): State<ClassificationState>,
    claims_ext: Option<Extension<Claims>>,
) -> Result<Response, ApiError> {
    let start_time = Instant::now();

    if let Some(Extension(claims)) = claims_ext.as_ref() {
        info!("Retrieving classification types for user: {}", claims.sub);
        // Check permission
        check_classification_permission(claims)?;
    } else {
        info!("Retrieving classification types (authentication disabled)");
    }

    // Build classification types response
    let response = ClassificationTypesResponse {
        research_types: vec![
            ResearchTypeInfo {
                name: "learning".to_string(),
                display_name: "Learning".to_string(),
                description: "Understanding concepts, definitions, or how things work".to_string(),
                example_keywords: vec![
                    "what is".to_string(),
                    "how does".to_string(),
                    "explain".to_string(),
                    "understand".to_string(),
                ],
            },
            ResearchTypeInfo {
                name: "implementation".to_string(),
                display_name: "Implementation".to_string(),
                description: "How to build, create, or implement something specific".to_string(),
                example_keywords: vec![
                    "implement".to_string(),
                    "build".to_string(),
                    "create".to_string(),
                    "make".to_string(),
                ],
            },
            ResearchTypeInfo {
                name: "troubleshooting".to_string(),
                display_name: "Troubleshooting".to_string(),
                description: "Debugging issues, fixing problems, or resolving errors".to_string(),
                example_keywords: vec![
                    "error".to_string(),
                    "debug".to_string(),
                    "fix".to_string(),
                    "problem".to_string(),
                ],
            },
            ResearchTypeInfo {
                name: "decision".to_string(),
                display_name: "Decision".to_string(),
                description: "Choosing between alternatives or evaluating options".to_string(),
                example_keywords: vec![
                    "choose".to_string(),
                    "compare".to_string(),
                    "vs".to_string(),
                    "better".to_string(),
                ],
            },
            ResearchTypeInfo {
                name: "validation".to_string(),
                display_name: "Validation".to_string(),
                description: "Verifying approaches, testing strategies, or quality assurance"
                    .to_string(),
                example_keywords: vec![
                    "test".to_string(),
                    "verify".to_string(),
                    "validate".to_string(),
                    "check".to_string(),
                ],
            },
        ],
        audience_levels: vec![
            AudienceLevelInfo {
                name: "beginner".to_string(),
                display_name: "Beginner".to_string(),
                description: "New to the technology or domain, needs foundational explanations"
                    .to_string(),
            },
            AudienceLevelInfo {
                name: "intermediate".to_string(),
                display_name: "Intermediate".to_string(),
                description:
                    "Has some experience, understands basics, can handle moderate complexity"
                        .to_string(),
            },
            AudienceLevelInfo {
                name: "advanced".to_string(),
                display_name: "Advanced".to_string(),
                description:
                    "Experienced with the technology, can handle complex topics and edge cases"
                        .to_string(),
            },
        ],
        technical_domains: vec![
            TechnicalDomainInfo {
                name: "general".to_string(),
                display_name: "General".to_string(),
                description: "Cross-cutting or non-specific technical content".to_string(),
            },
            TechnicalDomainInfo {
                name: "rust".to_string(),
                display_name: "Rust".to_string(),
                description: "Rust programming language and ecosystem".to_string(),
            },
            TechnicalDomainInfo {
                name: "web".to_string(),
                display_name: "Web Development".to_string(),
                description: "Web technologies, frameworks, and development practices".to_string(),
            },
            TechnicalDomainInfo {
                name: "devops".to_string(),
                display_name: "DevOps".to_string(),
                description: "Development operations, deployment, and infrastructure".to_string(),
            },
        ],
        urgency_levels: vec![
            UrgencyLevelInfo {
                name: "exploratory".to_string(),
                display_name: "Exploratory".to_string(),
                description: "Research and discovery, no immediate deadline".to_string(),
            },
            UrgencyLevelInfo {
                name: "planned".to_string(),
                display_name: "Planned".to_string(),
                description: "Scheduled work with reasonable timeline".to_string(),
            },
            UrgencyLevelInfo {
                name: "immediate".to_string(),
                display_name: "Immediate".to_string(),
                description: "Urgent issue requiring quick resolution".to_string(),
            },
        ],
        system_info: ClassificationSystemInfo {
            version: "1.0.0".to_string(),
            default_confidence_threshold: 0.6,
            advanced_classification_available: true,
            context_detection_available: true,
        },
    };

    let processing_time = start_time.elapsed();
    let api_response = ApiResponse::success(response, Uuid::new_v4());

    if let Some(Extension(claims)) = claims_ext.as_ref() {
        info!(
            "Classification types retrieved in {:.2}ms for user: {}",
            processing_time.as_millis(),
            claims.sub
        );
    } else {
        info!(
            "Classification types retrieved in {:.2}ms (auth disabled)",
            processing_time.as_millis()
        );
    }

    Ok((StatusCode::OK, Json(api_response)).into_response())
}

/// Helper function to check classification permission
fn check_classification_permission(claims: &Claims) -> Result<(), ApiError> {
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

/// Generate cache key for classification request
fn generate_cache_key(content: &str, options: &ClassificationOptions) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);

    // Include options in hash for different configurations
    options
        .enable_context_detection
        .unwrap_or(true)
        .hash(&mut hasher);
    options
        .enable_advanced_classification
        .unwrap_or(false)
        .hash(&mut hasher);

    format!("classification-{:x}", hasher.finish())
}

/// Create enhanced classification result for response
fn create_enhanced_result(
    classifier_result: &ClassificationResult,
    context_result: Option<&fortitude_core::ContextDetectionResult>,
    advanced_used: bool,
    _context_used: bool,
) -> EnhancedClassificationResult {
    // Extract context information or use defaults
    let (audience_level, technical_domain, urgency_level, dimension_confidences) =
        if let Some(ctx) = context_result {
            (
                ctx.audience_level.clone(),
                ctx.technical_domain.clone(),
                ctx.urgency_level.clone(),
                ctx.dimension_confidences.clone(),
            )
        } else {
            (
                AudienceLevel::default(),
                TechnicalDomain::default(),
                UrgencyLevel::default(),
                vec![],
            )
        };

    EnhancedClassificationResult {
        research_type: classifier_result.research_type.clone(),
        overall_confidence: classifier_result.confidence,
        audience_level,
        technical_domain,
        urgency_level,
        dimension_confidences,
        matched_keywords: classifier_result.matched_keywords.clone(),
        rule_priority: classifier_result.rule_priority,
        candidates: classifier_result.candidates.clone(),
        metadata: CoreClassificationMetadata {
            processing_time_ms: 0,
            algorithm: if advanced_used {
                "advanced".to_string()
            } else {
                "basic".to_string()
            },
            classifier_version: "1.0.0".to_string(),
            fallback_used: context_result.map(|ctx| ctx.fallback_used).unwrap_or(false),
            tags: HashMap::new(),
        },
    }
}

/// Convert classification result to API response format
fn convert_to_api_response(
    cache_key: &str,
    content: &str,
    classifier_result: &ClassificationResult,
    context_result: Option<&fortitude_core::ContextDetectionResult>,
    advanced_used: bool,
    _context_used: bool,
    processing_time_ms: u64,
) -> ClassificationResponse {
    let context_response = context_result.map(|ctx| ContextDetectionResponse {
        audience_level: ctx.audience_level.display_name().to_string(),
        technical_domain: ctx.technical_domain.display_name().to_string(),
        urgency_level: ctx.urgency_level.display_name().to_string(),
        dimension_confidences: ctx
            .dimension_confidences
            .iter()
            .map(|dc| DimensionConfidence {
                dimension: format!("{:?}", dc.dimension).to_lowercase(),
                confidence: dc.confidence,
                matched_keywords: dc.matched_keywords.clone(),
                explanation: dc.reasoning.clone(),
            })
            .collect(),
        overall_confidence: ctx.overall_confidence,
        fallback_used: ctx.fallback_used,
    });

    ClassificationResponse {
        id: cache_key.to_string(),
        content: content.to_string(),
        research_type: ApiClassificationResult {
            research_type: classifier_result.research_type.to_string(),
            confidence: classifier_result.confidence,
            matched_keywords: classifier_result.matched_keywords.clone(),
            rule_priority: classifier_result.rule_priority,
            candidates: classifier_result
                .candidates
                .iter()
                .map(|c| ApiClassificationCandidate {
                    research_type: c.research_type.to_string(),
                    confidence: c.confidence,
                    matched_keywords: c.matched_keywords.clone(),
                    rule_priority: c.rule_priority,
                })
                .collect(),
        },
        context: context_response,
        metadata: ClassificationMetadata {
            completed_at: Utc::now(),
            processing_time_ms,
            advanced_classification_used: advanced_used,
            context_detection_used: _context_used,
            algorithm_version: "1.0.0".to_string(),
            tags: HashMap::new(),
        },
        processing_time_ms,
    }
}

/// Default implementation for ClassificationOptions
impl Default for ClassificationOptions {
    fn default() -> Self {
        Self {
            enable_context_detection: Some(true),
            enable_advanced_classification: Some(false),
            confidence_threshold: Some(0.6),
            max_processing_time_ms: Some(5000),
            include_explanations: Some(true),
        }
    }
}

/// Build search criteria for classification filtering
fn build_classification_search_criteria(
    request: &ClassificationListRequest,
) -> Result<ClassificationSearchCriteria, ApiError> {
    Ok(ClassificationSearchCriteria {
        research_type: request
            .research_type
            .as_ref()
            .map(|rt| {
                rt.parse::<ResearchType>()
                    .map_err(|e| ApiError::BadRequest {
                        message: format!("Invalid research type: {e}"),
                    })
            })
            .transpose()?,
        audience_level: request.audience_level.clone(),
        technical_domain: request.technical_domain.clone(),
        urgency_level: request.urgency_level.clone(),
        keywords: request.keywords.clone().unwrap_or_default(),
        min_confidence: request.min_confidence,
        limit: request.limit.unwrap_or(10),
        offset: request.offset.unwrap_or(0),
    })
}

/// Classification search criteria
#[derive(Debug)]
#[allow(dead_code)]
struct ClassificationSearchCriteria {
    research_type: Option<ResearchType>,
    audience_level: Option<String>,
    technical_domain: Option<String>,
    urgency_level: Option<String>,
    keywords: String,
    min_confidence: Option<f64>,
    limit: usize,
    offset: usize,
}

/// Create content preview
#[allow(dead_code)]
fn create_content_preview(content: &str, max_length: usize) -> String {
    if content.len() <= max_length {
        content.to_string()
    } else {
        let truncated = &content[..max_length];
        let last_space = truncated.rfind(' ').unwrap_or(max_length);
        format!("{}...", &content[..last_space])
    }
}

/// Classify content with a custom confidence threshold, bypassing the classifier's default threshold
fn classify_with_custom_threshold(
    classifier: &dyn Classifier,
    content: &str,
    custom_threshold: f64,
) -> Result<ClassificationResult, ApiError> {
    // First, get all classification candidates without threshold validation
    let candidates = classifier.get_all_classifications(content);

    if candidates.is_empty() {
        return Err(ApiError::BadRequest {
            message: "No classification rules matched".to_string(),
        });
    }

    // Get the best candidate (highest confidence)
    let best_candidate = &candidates[0];

    // Apply the custom threshold validation
    if best_candidate.confidence < custom_threshold {
        return Err(ApiError::BadRequest {
            message: format!(
                "Classification confidence too low: {:.2} < {:.2}",
                best_candidate.confidence, custom_threshold
            ),
        });
    }

    // Create a classification result from the best candidate
    Ok(ClassificationResult::new(
        best_candidate.research_type.clone(),
        best_candidate.confidence,
        best_candidate.matched_keywords.clone(),
        best_candidate.rule_priority,
        candidates,
    ))
}

/// Convert classification error to API error
fn convert_classification_error(err: ClassificationError) -> ApiError {
    match err {
        ClassificationError::InvalidInput(message) => ApiError::BadRequest { message },
        ClassificationError::Failed(message) => ApiError::InternalError {
            message: format!("Classification failed: {message}"),
        },
        ClassificationError::LowConfidence { actual, threshold } => ApiError::BadRequest {
            message: format!("Classification confidence too low: {actual:.2} < {threshold:.2}"),
        },
        _ => {
            error!("Unexpected classification error: {:?}", err);
            ApiError::InternalError {
                message: "Classification system error".to_string(),
            }
        }
    }
}

/// Convert storage error to API error
#[allow(dead_code)]
fn convert_storage_error(err: StorageError) -> ApiError {
    match err {
        StorageError::FileNotFound(_) => ApiError::NotFound {
            resource: "Classification result not found".to_string(),
        },
        StorageError::Serialization(_) => ApiError::InternalError {
            message: "Failed to process classification data".to_string(),
        },
        StorageError::Io(_) => ApiError::InternalError {
            message: "Storage system error".to_string(),
        },
        StorageError::PermissionDenied(_) => ApiError::Forbidden {
            reason: "Access denied to classification data".to_string(),
        },
        _ => {
            error!("Unexpected storage error: {:?}", err);
            ApiError::InternalError {
                message: "Storage system error".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::Claims;

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
    fn test_check_classification_permission_success() {
        let claims = create_test_claims();
        assert!(check_classification_permission(&claims).is_ok());
    }

    #[test]
    fn test_check_classification_permission_admin() {
        let claims = create_admin_claims();
        assert!(check_classification_permission(&claims).is_ok());
    }

    #[test]
    fn test_check_classification_permission_denied() {
        let claims = create_no_permission_claims();
        assert!(check_classification_permission(&claims).is_err());
    }

    #[test]
    fn test_generate_cache_key() {
        let content = "test content";
        let options = ClassificationOptions::default();

        let key1 = generate_cache_key(content, &options);
        let key2 = generate_cache_key(content, &options);

        assert_eq!(key1, key2);
        assert!(key1.starts_with("classification-"));
    }

    #[test]
    fn test_generate_cache_key_different_content() {
        let options = ClassificationOptions::default();

        let key1 = generate_cache_key("content1", &options);
        let key2 = generate_cache_key("content2", &options);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_classification_options_default() {
        let options = ClassificationOptions::default();
        assert_eq!(options.enable_context_detection, Some(true));
        assert_eq!(options.enable_advanced_classification, Some(false));
        assert_eq!(options.confidence_threshold, Some(0.6));
        assert_eq!(options.max_processing_time_ms, Some(5000));
        assert_eq!(options.include_explanations, Some(true));
    }

    #[test]
    fn test_classify_with_custom_threshold() {
        use fortitude_types::{
            ClassificationCandidate, ClassificationError, ClassificationResult, ResearchType,
        };

        // Create a simple mock classifier for testing
        struct MockClassifier {
            candidates: Vec<ClassificationCandidate>,
        }

        impl Classifier for MockClassifier {
            fn classify(&self, _query: &str) -> Result<ClassificationResult, ClassificationError> {
                // Not used in our test
                Ok(ClassificationResult::new(
                    ResearchType::Learning,
                    0.5,
                    vec![],
                    1,
                    vec![],
                ))
            }

            fn get_confidence(&self, _query: &str, _research_type: &ResearchType) -> f64 {
                0.5
            }

            fn get_all_classifications(&self, _query: &str) -> Vec<ClassificationCandidate> {
                self.candidates.clone()
            }
        }

        // Create mock classifier with known candidates
        let candidates = vec![
            ClassificationCandidate::new(
                ResearchType::Implementation,
                0.08, // Above 0.05 threshold but below 0.6 default
                vec!["implement".to_string()],
                1,
            ),
            ClassificationCandidate::new(ResearchType::Learning, 0.05, vec!["how".to_string()], 1),
        ];
        let mock_classifier = MockClassifier { candidates };

        // Test with low threshold (should succeed)
        let result = classify_with_custom_threshold(&mock_classifier, "test content", 0.05);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.research_type, ResearchType::Implementation);
        assert_eq!(result.confidence, 0.08);

        // Test with high threshold (should fail)
        let result = classify_with_custom_threshold(&mock_classifier, "test content", 0.5);
        assert!(result.is_err());
        if let Err(ApiError::BadRequest { message }) = result {
            assert!(message.contains("Classification confidence too low"));
            assert!(message.contains("0.08 < 0.50"));
        } else {
            panic!("Expected BadRequest error");
        }
    }
}
