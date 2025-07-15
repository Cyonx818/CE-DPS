// ABOUTME: Cache management endpoint handlers for API server
// Provides HTTP endpoints for cache operations and analytics

use crate::config::ApiServerConfig;
use crate::extractors::SafeQuery;
use crate::middleware::auth::Claims;
use crate::models::errors::ApiError;
use crate::models::requests::{CacheInvalidateRequest, CacheSearchRequest};
use crate::models::responses::{
    AgeDistribution, ApiResponse, CacheCleanupResponse, CacheInvalidateResponse,
    CacheInvalidationCriteria, CacheItemResponse, CachePerformanceResponse, CacheSearchMetadata,
    CacheSearchResponse, CacheStatsResponse, CacheTypeStatsResponse, CleanupSummary,
    PaginationInfo, RecentOperationsResponse, SizeDistribution, StorageEfficiencyResponse,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use fortitude_core::storage::FileStorage;
use fortitude_types::{CacheEntry, ResearchType, SearchQuery, Storage};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};
use utoipa;
use uuid::Uuid;
use validator::Validate;

/// Shared application state for cache handlers
#[derive(Clone)]
pub struct CacheState {
    pub storage: Arc<FileStorage>,
    pub config: Arc<ApiServerConfig>,
}

impl CacheState {
    /// Create new cache state with storage initialization
    pub async fn new(config: &ApiServerConfig) -> Result<Self, anyhow::Error> {
        use fortitude_types::StorageConfig;
        use std::path::PathBuf;

        // Create storage configuration
        let storage_config = StorageConfig {
            base_path: PathBuf::from("./reference_library"),
            cache_expiration_seconds: 86400,         // 24 hours
            max_cache_size_bytes: 1024 * 1024 * 100, // 100MB
            enable_content_addressing: true,
            index_update_interval_seconds: 300, // 5 minutes
        };

        // Initialize file storage
        let file_storage = FileStorage::new(storage_config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize file storage: {}", e))?;

        Ok(Self {
            storage: Arc::new(file_storage),
            config: Arc::new(config.clone()),
        })
    }
}

/// GET /api/v1/cache/stats - Get comprehensive cache statistics
#[utoipa::path(
    get,
    path = "/api/v1/cache/stats",
    responses(
        (status = 200, description = "Cache statistics retrieved successfully", body = ApiResponse<CacheStatsResponse>),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Cache",
    security(("jwt_auth" = []))
)]
#[instrument(skip_all)]
pub async fn get_cache_stats(
    State(cache_state): State<CacheState>,
    claims_ext: Option<Extension<Claims>>,
) -> Result<Json<ApiResponse<CacheStatsResponse>>, ApiError> {
    if let Some(Extension(claims)) = claims_ext.as_ref() {
        debug!("Getting cache statistics for user: {}", claims.sub);
    } else {
        debug!("Getting cache statistics (auth disabled)");
    }

    let start_time = Instant::now();

    // Get cache statistics from storage
    let storage_stats = cache_state.storage.get_cache_stats().await.map_err(|e| {
        error!("Failed to get cache stats: {}", e);
        ApiError::InternalError {
            message: "Failed to retrieve cache statistics".to_string(),
        }
    })?;

    // Convert storage stats to response format
    let mut by_research_type = HashMap::new();
    for (research_type, type_stats) in &storage_stats.by_research_type {
        by_research_type.insert(
            research_type.to_string(),
            CacheTypeStatsResponse {
                entries: type_stats.entries,
                size_bytes: type_stats.size_bytes,
                hit_rate: type_stats.hit_rate,
                hits: type_stats.hits,
                misses: type_stats.misses,
                average_quality: 0.85, // Placeholder - would calculate from actual data
            },
        );
    }

    let response = CacheStatsResponse {
        total_entries: storage_stats.total_entries,
        expired_entries: storage_stats.expired_entries,
        total_size_bytes: storage_stats.total_size_bytes,
        hit_rate: storage_stats.hit_rate,
        hits: storage_stats.hits,
        misses: storage_stats.misses,
        average_age_seconds: storage_stats.average_age_seconds,
        by_research_type,
        storage_efficiency: StorageEfficiencyResponse {
            utilization_percent: calculate_utilization_percent(&storage_stats),
            duplicate_entries: 0, // Placeholder - would track duplicates
            bytes_saved: 0,       // Placeholder - would track deduplication savings
            compression_ratio: None,
        },
        performance_metrics: CachePerformanceResponse {
            avg_retrieval_time_ms: 15.5, // Placeholder - would track actual metrics
            avg_storage_time_ms: 25.0,   // Placeholder - would track actual metrics
            warming_status: "ready".to_string(),
            recent_operations: RecentOperationsResponse {
                last_hour: 0, // Placeholder - would track actual operations
                last_day: 0,  // Placeholder - would track actual operations
                peak_hour: "N/A".to_string(),
                top_accessed: vec![],
            },
        },
    };

    let request_id = Uuid::new_v4();
    let processing_time = start_time.elapsed().as_millis() as u64;

    info!(
        "Cache statistics retrieved successfully in {}ms",
        processing_time
    );

    Ok(Json(ApiResponse::success(response, request_id)))
}

/// GET /api/v1/cache/search - Search cached content with filters
#[utoipa::path(
    get,
    path = "/api/v1/cache/search",
    params(
        CacheSearchRequest
    ),
    responses(
        (status = 200, description = "Cache search completed successfully", body = ApiResponse<CacheSearchResponse>),
        (status = 400, description = "Invalid search parameters"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Cache",
    security(("jwt_auth" = []))
)]
#[instrument(skip_all)]
pub async fn search_cache(
    State(cache_state): State<CacheState>,
    claims_ext: Option<Extension<Claims>>,
    SafeQuery(search_request): SafeQuery<CacheSearchRequest>,
) -> Result<Json<ApiResponse<CacheSearchResponse>>, ApiError> {
    if let Some(Extension(claims)) = claims_ext.as_ref() {
        debug!(
            "Searching cache for user: {} with query: {:?}",
            claims.sub, search_request.query
        );
    } else {
        debug!(
            "Searching cache with query: {:?} (auth disabled)",
            search_request.query
        );
    }

    let start_time = Instant::now();

    // Validation is already handled by SafeQuery extractor

    // Build search query for storage
    let mut search_query = SearchQuery::new(search_request.query.clone().unwrap_or_default());

    // Apply filters
    if let Some(research_type_str) = &search_request.research_type {
        if let Ok(research_type) = research_type_str.parse::<ResearchType>() {
            search_query = search_query.with_research_type(research_type);
        }
    }

    if let Some(tags) = &search_request.tags {
        search_query = search_query.with_tags(tags.clone());
    }

    if let Some(min_quality) = search_request.min_quality {
        search_query = search_query.with_min_quality(min_quality);
    }

    search_query = search_query
        .with_limit(search_request.limit.unwrap_or(10))
        .with_offset(search_request.offset.unwrap_or(0));

    // Perform search
    let _search_results = cache_state
        .storage
        .search(&search_query)
        .await
        .map_err(|e| {
            error!("Search failed: {}", e);
            ApiError::InternalError {
                message: "Search operation failed".to_string(),
            }
        })?;

    // Get all cache entries for filtering and total count
    let all_entries = cache_state
        .storage
        .list_cache_entries()
        .await
        .map_err(|e| {
            error!("Failed to list cache entries: {}", e);
            ApiError::InternalError {
                message: "Failed to list cache entries".to_string(),
            }
        })?;

    // Apply additional filters that aren't supported by the storage search
    let filtered_entries = filter_cache_entries(&all_entries, &search_request);

    // Convert to response format
    let results: Vec<CacheItemResponse> = filtered_entries
        .into_iter()
        .skip(search_request.offset.unwrap_or(0))
        .take(search_request.limit.unwrap_or(10))
        .map(|entry| cache_entry_to_response(&entry, claims_ext.as_ref().map(|ext| &ext.0)))
        .collect();

    let total_count = all_entries.len();
    let limit = search_request.limit.unwrap_or(10);
    let offset = search_request.offset.unwrap_or(0);

    let search_metadata = CacheSearchMetadata {
        query: search_request.query.clone(),
        filters_applied: build_filters_applied(&search_request),
        sort_order: search_request
            .sort
            .clone()
            .unwrap_or("relevance".to_string()),
        total_before_filters: all_entries.len(),
        search_time_ms: start_time.elapsed().as_millis() as u64,
    };

    let pagination = PaginationInfo {
        offset,
        limit,
        total_pages: total_count.div_ceil(limit),
        has_more: offset + limit < total_count,
    };

    let response = CacheSearchResponse {
        results,
        total_count,
        pagination,
        search_metadata,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };

    let request_id = Uuid::new_v4();

    info!(
        "Cache search completed successfully with {} results",
        response.results.len()
    );

    Ok(Json(ApiResponse::success(response, request_id)))
}

/// GET /api/v1/cache/{id} - Get specific cached item by ID
#[utoipa::path(
    get,
    path = "/api/v1/cache/{id}",
    params(
        ("id" = String, Path, description = "Cache item ID (cache key)")
    ),
    responses(
        (status = 200, description = "Cache item retrieved successfully", body = ApiResponse<CacheItemResponse>),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Cache item not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Cache",
    security(("jwt_auth" = []))
)]
#[instrument(skip_all)]
pub async fn get_cache_item(
    State(cache_state): State<CacheState>,
    claims_ext: Option<Extension<Claims>>,
    Path(cache_key): Path<String>,
) -> Result<Json<ApiResponse<CacheItemResponse>>, ApiError> {
    if let Some(Extension(claims)) = claims_ext.as_ref() {
        debug!("Getting cache item {} for user: {}", cache_key, claims.sub);
    } else {
        debug!("Getting cache item {} (auth disabled)", cache_key);
    }

    let _start_time = Instant::now();

    // Retrieve from storage
    let research_result = cache_state
        .storage
        .retrieve(&cache_key)
        .await
        .map_err(|e| {
            error!("Failed to retrieve cache item {}: {}", cache_key, e);
            ApiError::InternalError {
                message: "Failed to retrieve cache item".to_string(),
            }
        })?;

    match research_result {
        Some(_result) => {
            // Get all cache entries to find the specific one
            let cache_entries = cache_state
                .storage
                .list_cache_entries()
                .await
                .map_err(|e| {
                    error!("Failed to list cache entries: {}", e);
                    ApiError::InternalError {
                        message: "Failed to access cache metadata".to_string(),
                    }
                })?;

            // Find the cache entry that matches our key
            let cache_entry = cache_entries
                .iter()
                .find(|entry| entry.key == cache_key)
                .ok_or_else(|| {
                    warn!("Cache entry metadata not found for key: {}", cache_key);
                    ApiError::NotFound {
                        resource: format!("Cache item: {cache_key}"),
                    }
                })?;

            let response =
                cache_entry_to_response(cache_entry, claims_ext.as_ref().map(|ext| &ext.0));
            let request_id = Uuid::new_v4();

            info!("Cache item {} retrieved successfully", cache_key);

            Ok(Json(ApiResponse::success(response, request_id)))
        }
        None => {
            warn!("Cache item not found: {}", cache_key);
            Err(ApiError::NotFound {
                resource: format!("Cache item: {cache_key}"),
            })
        }
    }
}

/// DELETE /api/v1/cache/{id} - Delete specific cached item
#[utoipa::path(
    delete,
    path = "/api/v1/cache/{id}",
    params(
        ("id" = String, Path, description = "Cache item ID (cache key) to delete")
    ),
    responses(
        (status = 204, description = "Cache item deleted successfully"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Cache item not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Cache",
    security(("jwt_auth" = []))
)]
#[instrument(skip_all)]
pub async fn delete_cache_item(
    State(cache_state): State<CacheState>,
    Extension(claims): Extension<Claims>,
    Path(cache_key): Path<String>,
) -> Result<StatusCode, ApiError> {
    debug!("Deleting cache item {} for user: {}", cache_key, claims.sub);

    // Delete from storage
    cache_state.storage.delete(&cache_key).await.map_err(|e| {
        error!("Failed to delete cache item {}: {}", cache_key, e);
        ApiError::InternalError {
            message: "Failed to delete cache item".to_string(),
        }
    })?;

    info!("Cache item {} deleted successfully", cache_key);

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/cache/invalidate - Invalidate cache entries (bulk operation)
#[utoipa::path(
    post,
    path = "/api/v1/cache/invalidate",
    request_body = CacheInvalidateRequest,
    responses(
        (status = 200, description = "Cache invalidation completed successfully", body = ApiResponse<CacheInvalidateResponse>),
        (status = 400, description = "Invalid invalidation request"),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Cache",
    security(("jwt_auth" = []))
)]
#[instrument(skip_all)]
pub async fn invalidate_cache(
    State(cache_state): State<CacheState>,
    Extension(claims): Extension<Claims>,
    Json(invalidate_request): Json<CacheInvalidateRequest>,
) -> Result<Json<ApiResponse<CacheInvalidateResponse>>, ApiError> {
    debug!(
        "Invalidating cache for user: {} with request: {:?}",
        claims.sub, invalidate_request
    );

    let start_time = Instant::now();

    // Validate request
    if let Err(validation_errors) = invalidate_request.validate() {
        return Err(ApiError::BadRequest {
            message: format!("Invalid invalidation request: {validation_errors:?}"),
        });
    }

    let mut invalidated_keys = Vec::new();
    let mut bytes_freed = 0u64;

    // Handle specific keys
    if let Some(keys) = &invalidate_request.keys {
        for key in keys {
            if !invalidate_request.dry_run.unwrap_or(false) {
                if let Err(e) = cache_state.storage.delete(key).await {
                    warn!("Failed to delete cache key {}: {}", key, e);
                } else {
                    invalidated_keys.push(key.clone());
                    // Note: In a real implementation, we'd track the actual bytes freed
                    bytes_freed += 1024; // Placeholder
                }
            } else {
                invalidated_keys.push(key.clone());
            }
        }
    }

    // Handle pattern-based invalidation
    if invalidate_request.pattern.is_some()
        || invalidate_request.research_type.is_some()
        || invalidate_request.tags.is_some()
        || invalidate_request.max_age_seconds.is_some()
        || invalidate_request.min_quality.is_some()
    {
        let all_entries = cache_state
            .storage
            .list_cache_entries()
            .await
            .map_err(|e| {
                error!("Failed to list cache entries for bulk invalidation: {}", e);
                ApiError::InternalError {
                    message: "Failed to list cache entries".to_string(),
                }
            })?;

        for entry in all_entries {
            if should_invalidate_entry(&entry, &invalidate_request) {
                if !invalidate_request.dry_run.unwrap_or(false) {
                    if let Err(e) = cache_state.storage.delete(&entry.key).await {
                        warn!("Failed to delete cache key {}: {}", entry.key, e);
                    } else {
                        invalidated_keys.push(entry.key.clone());
                        bytes_freed += entry.size_bytes;
                    }
                } else {
                    invalidated_keys.push(entry.key.clone());
                    bytes_freed += entry.size_bytes;
                }
            }
        }
    }

    let criteria = CacheInvalidationCriteria {
        keys: invalidate_request.keys.clone(),
        pattern: invalidate_request.pattern.clone(),
        research_type: invalidate_request.research_type.clone(),
        tags: invalidate_request.tags.clone(),
        max_age_seconds: invalidate_request.max_age_seconds,
        min_quality: invalidate_request.min_quality,
    };

    let response = CacheInvalidateResponse {
        status: "success".to_string(),
        invalidated_count: invalidated_keys.len(),
        invalidated_keys,
        bytes_freed,
        dry_run: invalidate_request.dry_run.unwrap_or(false),
        criteria,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };

    let request_id = Uuid::new_v4();

    info!(
        "Cache invalidation completed: {} items, {} bytes freed",
        response.invalidated_count, response.bytes_freed
    );

    Ok(Json(ApiResponse::success(response, request_id)))
}

/// POST /api/v1/cache/cleanup - Cleanup expired/orphaned cache entries
#[utoipa::path(
    post,
    path = "/api/v1/cache/cleanup",
    responses(
        (status = 200, description = "Cache cleanup completed successfully", body = ApiResponse<CacheCleanupResponse>),
        (status = 401, description = "Unauthorized - JWT token required"),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Cache",
    security(("jwt_auth" = []))
)]
#[instrument(skip_all)]
pub async fn cleanup_cache(
    State(cache_state): State<CacheState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<CacheCleanupResponse>>, ApiError> {
    debug!("Cleaning up cache for user: {}", claims.sub);

    let start_time = Instant::now();

    // Clean up expired entries
    let cleaned_count = cache_state.storage.cleanup_expired().await.map_err(|e| {
        error!("Failed to cleanup expired cache entries: {}", e);
        ApiError::InternalError {
            message: "Failed to cleanup expired entries".to_string(),
        }
    })?;

    // In a real implementation, we would also:
    // 1. Scan for orphaned files in the filesystem
    // 2. Rebuild/update indices if needed
    // 3. Collect detailed statistics about what was cleaned

    let cleanup_summary = CleanupSummary {
        by_research_type: HashMap::new(), // Would be populated with actual data
        age_distribution: vec![
            AgeDistribution {
                range: "0-1h".to_string(),
                count: 0,
                bytes: 0,
            },
            AgeDistribution {
                range: "1h-1d".to_string(),
                count: cleaned_count as usize / 2,
                bytes: 1024 * cleaned_count / 2,
            },
            AgeDistribution {
                range: ">1d".to_string(),
                count: cleaned_count as usize / 2,
                bytes: 1024 * cleaned_count / 2,
            },
        ],
        size_distribution: vec![
            SizeDistribution {
                range: "0-1KB".to_string(),
                count: cleaned_count as usize / 3,
                bytes: 512 * cleaned_count / 3,
            },
            SizeDistribution {
                range: "1KB-10KB".to_string(),
                count: cleaned_count as usize / 3,
                bytes: 5120 * cleaned_count / 3,
            },
            SizeDistribution {
                range: ">10KB".to_string(),
                count: cleaned_count as usize / 3,
                bytes: 15360 * cleaned_count / 3,
            },
        ],
        recommendations: generate_cleanup_recommendations(cleaned_count),
    };

    let response = CacheCleanupResponse {
        status: "success".to_string(),
        cleaned_count: cleaned_count as usize,
        orphaned_files_removed: 0, // Placeholder - would track actual orphaned files
        bytes_freed: 1024 * cleaned_count, // Placeholder calculation
        cleanup_summary,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };

    let request_id = Uuid::new_v4();

    info!(
        "Cache cleanup completed: {} items cleaned, {} bytes freed",
        response.cleaned_count, response.bytes_freed
    );

    Ok(Json(ApiResponse::success(response, request_id)))
}

// Helper functions

fn calculate_utilization_percent(stats: &fortitude_types::CacheStats) -> f64 {
    if stats.total_entries == 0 {
        0.0
    } else {
        ((stats.total_entries - stats.expired_entries) as f64 / stats.total_entries as f64) * 100.0
    }
}

fn filter_cache_entries(entries: &[CacheEntry], request: &CacheSearchRequest) -> Vec<CacheEntry> {
    let mut filtered = Vec::new();

    for entry in entries {
        // Apply age filters
        if let Some(min_age) = request.min_age_seconds {
            if entry.age_seconds() < min_age {
                continue;
            }
        }

        if let Some(max_age) = request.max_age_seconds {
            if entry.age_seconds() > max_age {
                continue;
            }
        }

        // Apply size filters
        if let Some(min_size) = request.min_size_bytes {
            if entry.size_bytes < min_size {
                continue;
            }
        }

        if let Some(max_size) = request.max_size_bytes {
            if entry.size_bytes > max_size {
                continue;
            }
        }

        // Apply keyword search
        if let Some(keywords) = &request.keywords {
            let keywords_lower = keywords.to_lowercase();
            let query_match = entry
                .original_query
                .to_lowercase()
                .contains(&keywords_lower);

            if !query_match {
                continue;
            }
        }

        filtered.push(entry.clone());
    }

    // Apply sorting
    match request.sort.as_deref() {
        Some("newest") => filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        Some("oldest") => filtered.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        Some("size") => filtered.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes)),
        _ => {} // Keep original order for relevance
    }

    filtered
}

fn cache_entry_to_response(entry: &CacheEntry, claims: Option<&Claims>) -> CacheItemResponse {
    // Check if user has admin permissions to see file paths
    let file_path = if let Some(claims) = claims {
        if claims.permissions.contains(&"fortitude:admin".to_string()) {
            Some(entry.file_path.to_string_lossy().to_string())
        } else {
            None
        }
    } else {
        // When auth is disabled, don't expose file paths for security
        None
    };

    // Extract tags from metadata (placeholder implementation)
    let tags = entry
        .metadata
        .get("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    CacheItemResponse {
        key: entry.key.clone(),
        research_type: entry.research_type.to_string(),
        original_query: entry.original_query.clone(),
        content_summary: generate_content_summary(&entry.original_query),
        quality_score: 0.85, // Placeholder - would extract from research result
        size_bytes: entry.size_bytes,
        created_at: entry.created_at,
        last_accessed: entry.last_accessed,
        expires_at: entry.expires_at,
        is_expired: entry.is_expired(),
        content_hash: entry.content_hash.clone(),
        file_path,
        metadata: entry.metadata.clone(),
        tags,
    }
}

fn build_filters_applied(request: &CacheSearchRequest) -> Vec<String> {
    let mut filters = Vec::new();

    if request.research_type.is_some() {
        filters.push("research_type".to_string());
    }
    if request.min_quality.is_some() {
        filters.push("min_quality".to_string());
    }
    if request.tags.is_some() {
        filters.push("tags".to_string());
    }
    if request.min_age_seconds.is_some() {
        filters.push("min_age".to_string());
    }
    if request.max_age_seconds.is_some() {
        filters.push("max_age".to_string());
    }
    if request.min_size_bytes.is_some() {
        filters.push("min_size".to_string());
    }
    if request.max_size_bytes.is_some() {
        filters.push("max_size".to_string());
    }

    filters
}

fn should_invalidate_entry(entry: &CacheEntry, request: &CacheInvalidateRequest) -> bool {
    // Check research type filter
    if let Some(research_type_str) = &request.research_type {
        if entry.research_type.to_string() != *research_type_str {
            return false;
        }
    }

    // Check age filter
    if let Some(max_age) = request.max_age_seconds {
        if entry.age_seconds() <= max_age {
            return false;
        }
    }

    // Check pattern matching (simplified - would use proper glob matching)
    if let Some(pattern) = &request.pattern {
        if !entry.key.contains(pattern) {
            return false;
        }
    }

    // Check tags filter (placeholder - would check actual tags)
    if let Some(tags) = &request.tags {
        let entry_tags = entry
            .metadata
            .get("tags")
            .map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if !tags.iter().any(|tag| entry_tags.contains(tag)) {
            return false;
        }
    }

    true
}

fn generate_content_summary(query: &str) -> String {
    // Simple summary generation - in a real implementation, this would
    // extract content from the actual research result
    if query.len() > 100 {
        format!("{}...", &query[..97])
    } else {
        query.to_string()
    }
}

fn generate_cleanup_recommendations(cleaned_count: u64) -> Vec<String> {
    let mut recommendations = Vec::new();

    if cleaned_count > 100 {
        recommendations
            .push("Consider reducing cache retention time to prevent buildup".to_string());
    } else if cleaned_count < 10 {
        recommendations.push("Cache cleanup frequency appears optimal".to_string());
    }

    if cleaned_count > 0 {
        recommendations
            .push("Consider implementing cache warming for frequently accessed items".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiServerConfig;
    use axum::http::StatusCode;
    use fortitude_types::{CacheEntry, ResearchType, StorageConfig};
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio;

    async fn create_test_cache_state() -> (CacheState, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = ApiServerConfig::default();

        let storage_config = StorageConfig {
            base_path: temp_dir.path().to_path_buf(),
            cache_expiration_seconds: 3600,
            max_cache_size_bytes: 1024 * 1024,
            enable_content_addressing: true,
            index_update_interval_seconds: 300,
        };

        let file_storage = FileStorage::new(storage_config).await.unwrap();

        let cache_state = CacheState {
            storage: Arc::new(file_storage),
            config: Arc::new(config),
        };

        (cache_state, temp_dir)
    }

    fn create_test_claims() -> Claims {
        Claims {
            sub: "test_user".to_string(),
            permissions: vec![
                "fortitude:resources:read".to_string(),
                "fortitude:admin".to_string(),
            ],
            exp: 9999999999, // Far in the future
            iat: 1000000000,
            iss: "fortitude-api-server".to_string(),
        }
    }

    #[tokio::test]
    async fn test_cache_state_creation() {
        let config = ApiServerConfig::default();
        let cache_state = CacheState::new(&config).await;
        assert!(cache_state.is_ok());
    }

    #[tokio::test]
    async fn test_get_cache_stats() {
        let (cache_state, _temp_dir) = create_test_cache_state().await;
        let claims = create_test_claims();

        let result =
            get_cache_stats(axum::extract::State(cache_state), Some(Extension(claims))).await;

        assert!(result.is_ok());
        let _response = result.unwrap();
        // total_entries is unsigned, so it's always >= 0
    }

    #[tokio::test]
    async fn test_search_cache_empty() {
        let (cache_state, _temp_dir) = create_test_cache_state().await;
        let claims = create_test_claims();

        let search_request = CacheSearchRequest {
            query: Some("test".to_string()),
            research_type: None,
            keywords: None,
            min_quality: None,
            tags: None,
            min_age_seconds: None,
            max_age_seconds: None,
            min_size_bytes: None,
            max_size_bytes: None,
            limit: Some(10),
            offset: Some(0),
            sort: None,
        };

        let result = search_cache(
            axum::extract::State(cache_state),
            Some(Extension(claims)),
            SafeQuery(search_request),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.results.len(), 0);
        assert_eq!(response.0.data.total_count, 0);
    }

    #[tokio::test]
    async fn test_get_cache_item_not_found() {
        let (cache_state, _temp_dir) = create_test_cache_state().await;
        let claims = create_test_claims();

        let result = get_cache_item(
            axum::extract::State(cache_state),
            Some(Extension(claims)),
            axum::extract::Path("nonexistent_key".to_string()),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::NotFound { resource } => {
                assert!(resource.contains("Cache item"));
                assert!(resource.contains("nonexistent_key"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_cache_item() {
        let (cache_state, _temp_dir) = create_test_cache_state().await;
        let claims = create_test_claims();

        // Test deleting a non-existent item (should not error)
        let result = delete_cache_item(
            axum::extract::State(cache_state),
            Extension(claims),
            axum::extract::Path("nonexistent_key".to_string()),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_invalidate_cache_dry_run() {
        let (cache_state, _temp_dir) = create_test_cache_state().await;
        let claims = create_test_claims();

        let invalidate_request = CacheInvalidateRequest {
            keys: Some(vec!["key1".to_string(), "key2".to_string()]),
            pattern: None,
            research_type: None,
            tags: None,
            max_age_seconds: None,
            min_quality: None,
            dry_run: Some(true),
        };

        let result = invalidate_cache(
            axum::extract::State(cache_state),
            Extension(claims),
            Json(invalidate_request),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.invalidated_count, 2);
        assert!(response.0.data.dry_run);
        assert_eq!(response.0.data.invalidated_keys, vec!["key1", "key2"]);
    }

    #[tokio::test]
    async fn test_cleanup_cache() {
        let (cache_state, _temp_dir) = create_test_cache_state().await;
        let claims = create_test_claims();

        let result = cleanup_cache(axum::extract::State(cache_state), Extension(claims)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.status, "success");
        // cleaned_count is unsigned, so it's always >= 0
    }

    #[test]
    fn test_filter_cache_entries() {
        let entries = vec![
            CacheEntry::new(
                "key1".to_string(),
                PathBuf::from("test1.json"),
                ResearchType::Implementation,
                "rust async programming".to_string(),
                1024,
                "hash1".to_string(),
                3600,
            ),
            CacheEntry::new(
                "key2".to_string(),
                PathBuf::from("test2.json"),
                ResearchType::Learning,
                "python web development".to_string(),
                2048,
                "hash2".to_string(),
                3600,
            ),
        ];

        let request = CacheSearchRequest {
            query: None,
            research_type: Some("implementation".to_string()),
            keywords: Some("rust".to_string()),
            min_quality: None,
            tags: None,
            min_age_seconds: None,
            max_age_seconds: None,
            min_size_bytes: Some(1000),
            max_size_bytes: Some(1500),
            limit: Some(10),
            offset: Some(0),
            sort: Some("newest".to_string()),
        };

        let filtered = filter_cache_entries(&entries, &request);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].key, "key1");
    }

    #[test]
    fn test_should_invalidate_entry() {
        let entry = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("test.json"),
            ResearchType::Implementation,
            "test query".to_string(),
            1024,
            "hash".to_string(),
            3600,
        );

        let request = CacheInvalidateRequest {
            keys: None,
            pattern: Some("test".to_string()),
            research_type: Some("Implementation".to_string()),
            tags: None,
            max_age_seconds: None, // Skip age check for test
            min_quality: None,
            dry_run: None,
        };

        assert!(should_invalidate_entry(&entry, &request));

        let request_no_match = CacheInvalidateRequest {
            keys: None,
            pattern: Some("nomatch".to_string()),
            research_type: None,
            tags: None,
            max_age_seconds: None,
            min_quality: None,
            dry_run: None,
        };

        assert!(!should_invalidate_entry(&entry, &request_no_match));
    }

    #[test]
    fn test_cache_entry_to_response() {
        let entry = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/path/to/test.json"),
            ResearchType::Implementation,
            "test query".to_string(),
            1024,
            "hash123".to_string(),
            3600,
        );

        let claims = create_test_claims();
        let response = cache_entry_to_response(&entry, Some(&claims));

        assert_eq!(response.key, "test_key");
        assert_eq!(response.research_type, "Implementation");
        assert_eq!(response.original_query, "test query");
        assert_eq!(response.size_bytes, 1024);
        assert_eq!(response.content_hash, "hash123");
        assert!(response.file_path.is_some()); // Admin user can see file paths
    }

    #[test]
    fn test_cache_entry_to_response_non_admin() {
        let entry = CacheEntry::new(
            "test_key".to_string(),
            PathBuf::from("/path/to/test.json"),
            ResearchType::Implementation,
            "test query".to_string(),
            1024,
            "hash123".to_string(),
            3600,
        );

        let claims = Claims {
            sub: "regular_user".to_string(),
            permissions: vec!["fortitude:resources:read".to_string()],
            exp: 9999999999,
            iat: 1000000000,
            iss: "fortitude-api-server".to_string(),
        };

        let response = cache_entry_to_response(&entry, Some(&claims));

        assert_eq!(response.key, "test_key");
        assert!(response.file_path.is_none()); // Non-admin user cannot see file paths
    }

    #[test]
    fn test_generate_content_summary() {
        let short_query = "short query";
        let summary = generate_content_summary(short_query);
        assert_eq!(summary, short_query);

        let long_query = "a".repeat(150);
        let summary = generate_content_summary(&long_query);
        assert!(summary.len() <= 100);
        assert!(summary.ends_with("..."));
    }

    #[test]
    fn test_build_filters_applied() {
        let request = CacheSearchRequest {
            query: Some("test".to_string()),
            research_type: Some("implementation".to_string()),
            keywords: None,
            min_quality: Some(0.8),
            tags: Some(vec!["rust".to_string()]),
            min_age_seconds: Some(100),
            max_age_seconds: None,
            min_size_bytes: None,
            max_size_bytes: Some(1000),
            limit: Some(10),
            offset: Some(0),
            sort: None,
        };

        let filters = build_filters_applied(&request);
        assert!(filters.contains(&"research_type".to_string()));
        assert!(filters.contains(&"min_quality".to_string()));
        assert!(filters.contains(&"tags".to_string()));
        assert!(filters.contains(&"min_age".to_string()));
        assert!(filters.contains(&"max_size".to_string()));
        assert!(!filters.contains(&"max_age".to_string()));
        assert!(!filters.contains(&"min_size".to_string()));
    }

    #[test]
    fn test_generate_cleanup_recommendations() {
        let recommendations_high = generate_cleanup_recommendations(150);
        assert!(recommendations_high
            .iter()
            .any(|r| r.contains("reducing cache retention")));

        let recommendations_low = generate_cleanup_recommendations(5);
        assert!(recommendations_low
            .iter()
            .any(|r| r.contains("appears optimal")));

        let recommendations_zero = generate_cleanup_recommendations(0);
        assert!(!recommendations_zero
            .iter()
            .any(|r| r.contains("cache warming")));

        let recommendations_some = generate_cleanup_recommendations(50);
        assert!(recommendations_some
            .iter()
            .any(|r| r.contains("cache warming")));
    }
}
