// ABOUTME: Response model definitions for API endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Standard API response wrapper
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,

    /// Request ID for tracing
    pub request_id: Uuid,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Success status
    pub success: bool,
}

/// Complete research response with structured result
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResearchResponse {
    /// Research result ID for retrieval
    pub id: String,

    /// Original query
    pub query: String,

    /// Classified research type
    pub research_type: String,

    /// Immediate answer (first layer of progressive disclosure)
    pub immediate_answer: String,

    /// Supporting evidence (second layer)
    pub supporting_evidence: Vec<Evidence>,

    /// Implementation details (third layer)
    pub implementation_details: Vec<Detail>,

    /// Research metadata
    pub metadata: ResearchMetadata,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Research listing response with pagination
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResearchListResponse {
    /// List of research result summaries
    pub results: Vec<ResearchSummary>,

    /// Total number of results available
    pub total_count: usize,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Summary information for research result listings
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResearchSummary {
    /// Research result ID
    pub id: String,

    /// Original query
    pub query: String,

    /// Research type
    pub research_type: String,

    /// Brief summary of the answer
    pub summary: String,

    /// Quality score (0.0-1.0)
    pub quality_score: f64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Result tags
    pub tags: Vec<String>,
}

/// Evidence supporting a research result
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Evidence {
    /// Source of the evidence
    pub source: String,

    /// Evidence content
    pub content: String,

    /// Relevance score (0.0-1.0)
    pub relevance: f64,

    /// Evidence type (documentation, example, reference)
    pub evidence_type: String,
}

/// Implementation detail for research results
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Detail {
    /// Detail category (code, config, setup, etc.)
    pub category: String,

    /// Detail content
    pub content: String,

    /// Priority level (low, medium, high)
    pub priority: String,

    /// Prerequisites for this detail
    pub prerequisites: Vec<String>,
}

/// Metadata about research results
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResearchMetadata {
    /// Research completion timestamp
    pub completed_at: DateTime<Utc>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Sources consulted
    pub sources_consulted: Vec<String>,

    /// Quality score (0.0-1.0)
    pub quality_score: f64,

    /// Additional metadata tags
    pub tags: std::collections::HashMap<String, String>,
}

/// Pagination information for result listings
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginationInfo {
    /// Current page offset
    pub offset: usize,

    /// Number of results per page
    pub limit: usize,

    /// Total number of pages
    pub total_pages: usize,

    /// Whether there are more results
    pub has_more: bool,
}

/// Complete classification response with multi-dimensional analysis
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationResponse {
    /// Classification result ID for caching
    pub id: String,

    /// Original content that was classified
    pub content: String,

    /// Research type classification
    pub research_type: ClassificationResult,

    /// Context detection results
    pub context: Option<ContextDetectionResponse>,

    /// Classification metadata
    pub metadata: ClassificationMetadata,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Research type classification result
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationResult {
    /// Classified research type
    pub research_type: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Keywords that influenced the classification
    pub matched_keywords: Vec<String>,

    /// Rule priority that was applied
    pub rule_priority: u32,

    /// All candidate classifications considered
    pub candidates: Vec<ClassificationCandidate>,
}

/// Context detection response across all dimensions
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ContextDetectionResponse {
    /// Detected audience level
    pub audience_level: String,

    /// Detected technical domain  
    pub technical_domain: String,

    /// Detected urgency level
    pub urgency_level: String,

    /// Confidence scores for each dimension
    pub dimension_confidences: Vec<DimensionConfidence>,

    /// Overall context confidence
    pub overall_confidence: f64,

    /// Whether fallback was used in detection
    pub fallback_used: bool,
}

/// Confidence information for a classification dimension
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DimensionConfidence {
    /// Classification dimension (audience_level, technical_domain, urgency)
    pub dimension: String,

    /// Confidence score for this dimension
    pub confidence: f64,

    /// Keywords that influenced this dimension's classification
    pub matched_keywords: Vec<String>,

    /// Explanation of the classification
    pub explanation: String,
}

/// Candidate classification result
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationCandidate {
    /// Candidate research type
    pub research_type: String,

    /// Confidence score for this candidate
    pub confidence: f64,

    /// Keywords that matched for this candidate
    pub matched_keywords: Vec<String>,

    /// Rule priority that generated this candidate
    pub rule_priority: u32,
}

/// Classification metadata
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationMetadata {
    /// Classification completion timestamp
    pub completed_at: DateTime<Utc>,

    /// Classification processing time in milliseconds
    pub processing_time_ms: u64,

    /// Whether advanced classification was used
    pub advanced_classification_used: bool,

    /// Whether context detection was performed
    pub context_detection_used: bool,

    /// Classification algorithm version
    pub algorithm_version: String,

    /// Additional metadata tags
    pub tags: std::collections::HashMap<String, String>,
}

/// Classification listing response with pagination
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationListResponse {
    /// List of classification result summaries
    pub results: Vec<ClassificationSummary>,

    /// Total number of results available
    pub total_count: usize,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Summary information for classification result listings
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationSummary {
    /// Classification result ID
    pub id: String,

    /// Brief content preview
    pub content_preview: String,

    /// Research type
    pub research_type: String,

    /// Context summary
    pub context_summary: Option<String>,

    /// Overall confidence score
    pub confidence: f64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Classification tags
    pub tags: Vec<String>,
}

/// Classification types response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationTypesResponse {
    /// Available research types
    pub research_types: Vec<ResearchTypeInfo>,

    /// Available audience levels
    pub audience_levels: Vec<AudienceLevelInfo>,

    /// Available technical domains
    pub technical_domains: Vec<TechnicalDomainInfo>,

    /// Available urgency levels
    pub urgency_levels: Vec<UrgencyLevelInfo>,

    /// System information
    pub system_info: ClassificationSystemInfo,
}

/// Research type information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResearchTypeInfo {
    /// Research type name
    pub name: String,

    /// Display name for UI
    pub display_name: String,

    /// Description of when to use this type
    pub description: String,

    /// Example keywords that trigger this type
    pub example_keywords: Vec<String>,
}

/// Audience level information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AudienceLevelInfo {
    /// Audience level name
    pub name: String,

    /// Display name for UI
    pub display_name: String,

    /// Description of this audience level
    pub description: String,
}

/// Technical domain information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TechnicalDomainInfo {
    /// Technical domain name
    pub name: String,

    /// Display name for UI
    pub display_name: String,

    /// Description of this domain
    pub description: String,
}

/// Urgency level information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UrgencyLevelInfo {
    /// Urgency level name
    pub name: String,

    /// Display name for UI
    pub display_name: String,

    /// Description of this urgency level
    pub description: String,
}

/// Classification system information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClassificationSystemInfo {
    /// System version
    pub version: String,

    /// Default confidence threshold
    pub default_confidence_threshold: f64,

    /// Whether advanced classification is available
    pub advanced_classification_available: bool,

    /// Whether context detection is available
    pub context_detection_available: bool,
}

/// Health check response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Service version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Component health checks
    pub components: std::collections::HashMap<String, ComponentHealth>,
}

/// Component health status
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ComponentHealth {
    /// Component status
    pub status: String,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,

    /// Additional details
    pub details: Option<String>,
}

/// Cache operation response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheResponse {
    /// Operation status
    pub status: String,

    /// Number of keys affected
    pub keys_affected: usize,

    /// Cache statistics
    pub stats: Option<CacheStatsResponse>,
}

/// Comprehensive cache statistics response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheStatsResponse {
    /// Total number of cache entries
    pub total_entries: usize,

    /// Number of expired entries
    pub expired_entries: usize,

    /// Total cache size in bytes
    pub total_size_bytes: u64,

    /// Cache hit rate (0.0-1.0)
    pub hit_rate: f64,

    /// Number of cache hits
    pub hits: u64,

    /// Number of cache misses
    pub misses: u64,

    /// Average entry age in seconds
    pub average_age_seconds: f64,

    /// Statistics by research type
    pub by_research_type: std::collections::HashMap<String, CacheTypeStatsResponse>,

    /// Storage efficiency metrics
    pub storage_efficiency: StorageEfficiencyResponse,

    /// Cache performance metrics
    pub performance_metrics: CachePerformanceResponse,
}

/// Cache statistics by research type
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheTypeStatsResponse {
    /// Number of entries for this type
    pub entries: usize,

    /// Total size in bytes for this type
    pub size_bytes: u64,

    /// Hit rate for this type
    pub hit_rate: f64,

    /// Hits for this type
    pub hits: u64,

    /// Misses for this type
    pub misses: u64,

    /// Average quality score for this type
    pub average_quality: f64,
}

/// Storage efficiency metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct StorageEfficiencyResponse {
    /// Storage utilization percentage
    pub utilization_percent: f64,

    /// Number of duplicate content items
    pub duplicate_entries: usize,

    /// Bytes saved through deduplication
    pub bytes_saved: u64,

    /// Compression ratio if applicable
    pub compression_ratio: Option<f64>,
}

/// Cache performance metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CachePerformanceResponse {
    /// Average retrieval time in milliseconds
    pub avg_retrieval_time_ms: f64,

    /// Average storage time in milliseconds
    pub avg_storage_time_ms: f64,

    /// Cache warming status
    pub warming_status: String,

    /// Recent operation statistics
    pub recent_operations: RecentOperationsResponse,
}

/// Recent cache operations statistics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct RecentOperationsResponse {
    /// Operations in the last hour
    pub last_hour: u64,

    /// Operations in the last day
    pub last_day: u64,

    /// Most active time period
    pub peak_hour: String,

    /// Most frequently accessed items
    pub top_accessed: Vec<String>,
}

/// Cache search results response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheSearchResponse {
    /// List of matching cache items
    pub results: Vec<CacheItemResponse>,

    /// Total number of results available
    pub total_count: usize,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Search metadata
    pub search_metadata: CacheSearchMetadata,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Individual cache item response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheItemResponse {
    /// Cache key identifier
    pub key: String,

    /// Research type
    pub research_type: String,

    /// Original query
    pub original_query: String,

    /// Brief content summary
    pub content_summary: String,

    /// Quality score (0.0-1.0)
    pub quality_score: f64,

    /// File size in bytes
    pub size_bytes: u64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,

    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Whether the item is expired
    pub is_expired: bool,

    /// Content hash for deduplication
    pub content_hash: String,

    /// File path (for admin users only)
    pub file_path: Option<String>,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Tags associated with this cache item
    pub tags: Vec<String>,
}

/// Cache search metadata
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheSearchMetadata {
    /// Search query used
    pub query: Option<String>,

    /// Filters applied
    pub filters_applied: Vec<String>,

    /// Sort order used
    pub sort_order: String,

    /// Number of results before filtering
    pub total_before_filters: usize,

    /// Search performance metrics
    pub search_time_ms: u64,
}

/// Cache invalidation response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheInvalidateResponse {
    /// Operation status
    pub status: String,

    /// Number of items invalidated
    pub invalidated_count: usize,

    /// List of invalidated cache keys
    pub invalidated_keys: Vec<String>,

    /// Bytes freed by invalidation
    pub bytes_freed: u64,

    /// Whether this was a dry run
    pub dry_run: bool,

    /// Invalidation criteria used
    pub criteria: CacheInvalidationCriteria,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Cache invalidation criteria
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheInvalidationCriteria {
    /// Specific keys requested
    pub keys: Option<Vec<String>>,

    /// Pattern used for matching
    pub pattern: Option<String>,

    /// Research type filter
    pub research_type: Option<String>,

    /// Tags filter
    pub tags: Option<Vec<String>>,

    /// Age threshold in seconds
    pub max_age_seconds: Option<u64>,

    /// Quality threshold
    pub min_quality: Option<f64>,
}

/// Cache cleanup response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheCleanupResponse {
    /// Operation status
    pub status: String,

    /// Number of expired items cleaned up
    pub cleaned_count: usize,

    /// Number of orphaned files removed
    pub orphaned_files_removed: usize,

    /// Bytes freed by cleanup
    pub bytes_freed: u64,

    /// Cleanup summary
    pub cleanup_summary: CleanupSummary,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Cache cleanup summary
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CleanupSummary {
    /// Items removed by research type
    pub by_research_type: std::collections::HashMap<String, usize>,

    /// Age ranges of cleaned items
    pub age_distribution: Vec<AgeDistribution>,

    /// Size distribution of cleaned items
    pub size_distribution: Vec<SizeDistribution>,

    /// Cleanup recommendations
    pub recommendations: Vec<String>,
}

/// Age distribution information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AgeDistribution {
    /// Age range description
    pub range: String,

    /// Number of items in this range
    pub count: usize,

    /// Total bytes in this range
    pub bytes: u64,
}

/// Size distribution information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SizeDistribution {
    /// Size range description
    pub range: String,

    /// Number of items in this range
    pub count: usize,

    /// Total bytes in this range
    pub bytes: u64,
}

/// Authentication/login response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Token expiration in seconds
    pub expires_in: i64,

    /// User information
    pub user: UserInfo,

    /// Granted permissions
    pub permissions: Vec<String>,
}

/// User information in authentication responses
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UserInfo {
    /// User ID
    pub id: String,

    /// Username
    pub username: String,

    /// User roles
    pub roles: Vec<String>,
}

/// Token validation response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TokenValidationResponse {
    /// Whether the token is valid
    pub valid: bool,

    /// User information (if valid)
    pub user: Option<UserInfo>,

    /// Remaining token validity in seconds
    pub expires_in: Option<i64>,

    /// Token permissions
    pub permissions: Option<Vec<String>>,
}

/// Proactive research status response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveStatusResponse {
    /// Whether proactive research is currently running
    pub is_running: bool,

    /// Current operational status
    pub status: String,

    /// Uptime in seconds (if running)
    pub uptime_seconds: Option<u64>,

    /// Number of files currently being monitored
    pub monitored_files_count: usize,

    /// Number of active background research tasks
    pub active_tasks_count: usize,

    /// Number of completed tasks today
    pub completed_tasks_today: usize,

    /// Number of pending tasks in queue
    pub pending_tasks_count: usize,

    /// Last gap detection timestamp
    pub last_gap_detection: Option<DateTime<Utc>>,

    /// System health metrics
    pub health_metrics: ProactiveHealthMetrics,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Proactive research health metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveHealthMetrics {
    /// File monitor status
    pub file_monitor_status: String,

    /// Background scheduler status
    pub scheduler_status: String,

    /// Task executor status
    pub executor_status: String,

    /// Notification system status
    pub notification_status: String,

    /// Error count in last 24 hours
    pub error_count_24h: usize,

    /// Memory usage percentage
    pub memory_usage_percent: f64,

    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// Proactive research configuration response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveConfigResponse {
    /// Base directory being monitored
    pub base_directory: String,

    /// File patterns being monitored
    pub file_patterns: Vec<String>,

    /// File patterns being ignored
    pub ignore_patterns: Vec<String>,

    /// Whether proactive research is enabled
    pub enabled: bool,

    /// Monitoring interval in seconds
    pub monitoring_interval_seconds: u64,

    /// Maximum concurrent research tasks
    pub max_concurrent_tasks: u8,

    /// Research priority threshold (0.0-1.0)
    pub priority_threshold: f64,

    /// Whether auto-execute high priority research is enabled
    pub auto_execute_high_priority: bool,

    /// Notification preferences
    pub notification_preferences: ProactiveNotificationPreferencesResponse,

    /// Configuration last updated timestamp
    pub last_updated: DateTime<Utc>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Proactive notification preferences response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveNotificationPreferencesResponse {
    /// Whether gap detection notifications are enabled
    pub gap_detection_enabled: bool,

    /// Whether research completion notifications are enabled
    pub research_completion_enabled: bool,

    /// Whether error notifications are enabled
    pub error_notifications_enabled: bool,

    /// Notification frequency
    pub frequency: String,

    /// Minimum priority level for notifications
    pub min_priority_level: String,
}

/// Proactive research task list response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveTaskListResponse {
    /// List of proactive research tasks
    pub tasks: Vec<ProactiveTaskResponse>,

    /// Total number of tasks available
    pub total_count: usize,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Task statistics
    pub task_statistics: ProactiveTaskStatistics,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Individual proactive research task response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveTaskResponse {
    /// Task unique identifier
    pub id: String,

    /// Task description or title
    pub description: String,

    /// Research type for this task
    pub research_type: String,

    /// Gap type that triggered this task
    pub gap_type: String,

    /// Current task status
    pub status: String,

    /// Task priority level
    pub priority: String,

    /// Task creation timestamp
    pub created_at: DateTime<Utc>,

    /// Task start timestamp (if started)
    pub started_at: Option<DateTime<Utc>>,

    /// Task completion timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,

    /// Estimated completion time (if running)
    pub estimated_completion: Option<DateTime<Utc>>,

    /// File path that triggered this task
    pub trigger_file: Option<String>,

    /// Detected gap information
    pub gap_details: ProactiveGapDetails,

    /// Progress information (if running)
    pub progress: Option<ProactiveTaskProgress>,

    /// Research result (if completed successfully)
    pub result: Option<ProactiveResearchResult>,

    /// Error information (if failed)
    pub error: Option<ProactiveTaskError>,

    /// Task tags
    pub tags: Vec<String>,
}

/// Proactive gap details
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveGapDetails {
    /// Gap type identifier
    pub gap_type: String,

    /// Gap description
    pub description: String,

    /// Gap severity score (0.0-1.0)
    pub severity: f64,

    /// File path where gap was detected
    pub file_path: String,

    /// Line number where gap was detected (if applicable)
    pub line_number: Option<u32>,

    /// Context around the gap
    pub context: String,

    /// Suggested keywords for research
    pub suggested_keywords: Vec<String>,
}

/// Proactive task progress information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveTaskProgress {
    /// Progress percentage (0.0-1.0)
    pub percentage: f64,

    /// Current step description
    pub current_step: String,

    /// Total number of steps
    pub total_steps: u32,

    /// Current step number
    pub current_step_number: u32,

    /// Estimated time remaining in seconds
    pub estimated_time_remaining_seconds: Option<u64>,
}

/// Proactive research result
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveResearchResult {
    /// Research result identifier
    pub result_id: String,

    /// Research query used
    pub query: String,

    /// Research summary
    pub summary: String,

    /// Quality score of the research (0.0-1.0)
    pub quality_score: f64,

    /// Whether the result was automatically applied
    pub auto_applied: bool,

    /// Result content length in characters
    pub content_length: usize,

    /// Storage location
    pub storage_path: String,
}

/// Proactive task error information
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveTaskError {
    /// Error code
    pub error_code: String,

    /// Error message
    pub message: String,

    /// Error category
    pub category: String,

    /// Whether the task can be retried
    pub retryable: bool,

    /// Retry count
    pub retry_count: u32,

    /// Next retry time (if retryable)
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Proactive task statistics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveTaskStatistics {
    /// Number of tasks by status
    pub by_status: std::collections::HashMap<String, usize>,

    /// Number of tasks by priority
    pub by_priority: std::collections::HashMap<String, usize>,

    /// Number of tasks by research type
    pub by_research_type: std::collections::HashMap<String, usize>,

    /// Number of tasks by gap type
    pub by_gap_type: std::collections::HashMap<String, usize>,

    /// Average task completion time in seconds
    pub avg_completion_time_seconds: f64,

    /// Success rate percentage
    pub success_rate_percent: f64,
}

/// Proactive notification list response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveNotificationListResponse {
    /// List of proactive notifications
    pub notifications: Vec<ProactiveNotificationResponse>,

    /// Total number of notifications available
    pub total_count: usize,

    /// Number of unread notifications
    pub unread_count: usize,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Notification statistics
    pub notification_statistics: ProactiveNotificationStatistics,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Individual proactive notification response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveNotificationResponse {
    /// Notification unique identifier
    pub id: String,

    /// Notification type
    pub notification_type: String,

    /// Notification level (info, warning, error, critical)
    pub level: String,

    /// Notification title
    pub title: String,

    /// Notification message
    pub message: String,

    /// Whether the notification has been read
    pub read: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Read timestamp (if read)
    pub read_at: Option<DateTime<Utc>>,

    /// Expiration timestamp (if applicable)
    pub expires_at: Option<DateTime<Utc>>,

    /// Related task ID (if applicable)
    pub related_task_id: Option<String>,

    /// Related file path (if applicable)
    pub related_file_path: Option<String>,

    /// Notification metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Action buttons/links
    pub actions: Vec<ProactiveNotificationAction>,
}

/// Proactive notification action
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveNotificationAction {
    /// Action identifier
    pub id: String,

    /// Action label
    pub label: String,

    /// Action type (url, api_call, task_action)
    pub action_type: String,

    /// Action target (URL, API endpoint, etc.)
    pub target: String,

    /// Whether this is the primary action
    pub primary: bool,
}

/// Proactive notification statistics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProactiveNotificationStatistics {
    /// Number of notifications by type
    pub by_type: std::collections::HashMap<String, usize>,

    /// Number of notifications by level
    pub by_level: std::collections::HashMap<String, usize>,

    /// Number of read vs unread notifications
    pub read_status: std::collections::HashMap<String, usize>,

    /// Notifications created in last 24 hours
    pub created_last_24h: usize,

    /// Average time to read in seconds
    pub avg_time_to_read_seconds: f64,
}

/// Learning dashboard data response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningDashboardResponse {
    /// Current learning metrics snapshot
    pub current_metrics: LearningMetricsResponse,

    /// Health status of learning components
    pub health_status: LearningHealthResponse,

    /// Active alerts for learning system
    pub alerts: Vec<LearningAlert>,

    /// Performance graphs data for dashboard visualization
    pub performance_graphs: std::collections::HashMap<String, Vec<LearningDataPoint>>,

    /// System overview summary
    pub system_overview: LearningSystemOverview,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Learning metrics response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningMetricsResponse {
    /// Adaptation algorithm metrics
    pub adaptation_metrics: LearningAdaptationMetrics,

    /// Storage operation metrics
    pub storage_metrics: LearningStorageMetrics,

    /// Pattern recognition metrics
    pub pattern_recognition_metrics: LearningPatternRecognitionMetrics,

    /// User feedback metrics
    pub feedback_metrics: LearningFeedbackMetrics,

    /// Optimization metrics
    pub optimization_metrics: LearningOptimizationMetrics,

    /// System resource metrics
    pub system_metrics: LearningSystemMetrics,

    /// Metrics collection timestamp
    pub timestamp: DateTime<Utc>,
}

/// Learning adaptation metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningAdaptationMetrics {
    /// Number of adaptations applied successfully
    pub adaptations_applied: u64,

    /// Number of adaptations that failed
    pub adaptations_failed: u64,

    /// Average time for adaptations in milliseconds
    pub average_adaptation_time_ms: f64,

    /// Recent confidence scores for adaptations
    pub confidence_scores: Vec<f64>,

    /// Overall success rate (0.0-1.0)
    pub success_rate: f64,

    /// Timestamp of last adaptation
    pub last_adaptation: Option<DateTime<Utc>>,
}

/// Learning storage metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningStorageMetrics {
    /// Total number of storage operations
    pub total_operations: u64,

    /// Number of successful storage operations
    pub successful_operations: u64,

    /// Number of failed storage operations
    pub failed_operations: u64,

    /// Average response time in milliseconds
    pub average_response_time_ms: f64,

    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,

    /// Storage size in megabytes
    pub storage_size_mb: f64,

    /// Error rate (0.0-1.0)
    pub error_rate: f64,
}

/// Learning pattern recognition metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningPatternRecognitionMetrics {
    /// Number of patterns analyzed
    pub patterns_analyzed: u64,

    /// Number of patterns successfully recognized
    pub patterns_recognized: u64,

    /// Recognition accuracy (0.0-1.0)
    pub recognition_accuracy: f64,

    /// Average analysis time in milliseconds
    pub average_analysis_time_ms: f64,

    /// False positive rate (0.0-1.0)
    pub false_positive_rate: f64,

    /// False negative rate (0.0-1.0)
    pub false_negative_rate: f64,
}

/// Learning feedback metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningFeedbackMetrics {
    /// Number of feedback entries received
    pub feedback_received: u64,

    /// Number of feedback entries processed
    pub feedback_processed: u64,

    /// Average feedback score
    pub average_feedback_score: f64,

    /// Average feedback processing time in milliseconds
    pub feedback_processing_time_ms: f64,

    /// Feedback trends by category
    pub feedback_trends: std::collections::HashMap<String, f64>,
}

/// Learning optimization metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningOptimizationMetrics {
    /// Number of optimizations suggested
    pub optimizations_suggested: u64,

    /// Number of optimizations applied
    pub optimizations_applied: u64,

    /// Performance improvement measurements
    pub performance_improvements: Vec<f64>,

    /// Optimization success rate (0.0-1.0)
    pub optimization_success_rate: f64,

    /// Average optimization time in milliseconds
    pub average_optimization_time_ms: f64,
}

/// Learning system resource metrics
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningSystemMetrics {
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,

    /// CPU usage percentage
    pub cpu_usage_percent: f64,

    /// Disk usage in megabytes
    pub disk_usage_mb: f64,

    /// Network I/O in megabytes
    pub network_io_mb: f64,

    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Learning health check response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningHealthResponse {
    /// Overall health status
    pub overall_status: LearningHealthStatus,

    /// Individual component health results
    pub component_results: Vec<LearningComponentHealth>,

    /// Health summary message
    pub summary: String,

    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
}

/// Learning health status
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum LearningHealthStatus {
    /// All components healthy
    Healthy,
    /// Some components have warnings
    Warning,
    /// Critical issues detected
    Critical,
    /// Health status unknown
    Unknown,
}

/// Learning component health
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningComponentHealth {
    /// Component name
    pub component: String,

    /// Health status
    pub status: LearningHealthStatus,

    /// Status message
    pub message: String,

    /// Check timestamp
    pub timestamp: DateTime<Utc>,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Additional details
    pub details: std::collections::HashMap<String, String>,
}

/// Learning system alert
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningAlert {
    /// Alert ID
    pub id: String,

    /// Alert severity
    pub severity: LearningAlertSeverity,

    /// Component that triggered the alert
    pub component: String,

    /// Alert message
    pub message: String,

    /// Metric value that triggered the alert
    pub metric_value: f64,

    /// Threshold that was exceeded
    pub threshold: f64,

    /// Alert timestamp
    pub timestamp: DateTime<Utc>,

    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
}

/// Learning alert severity
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum LearningAlertSeverity {
    /// Informational alert
    Info,
    /// Warning level alert
    Warning,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Learning data point for time-series graphs
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningDataPoint {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,

    /// Metric value
    pub value: f64,
}

/// Learning system overview for dashboard
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningSystemOverview {
    /// Total number of adaptations performed
    pub total_adaptations: u64,

    /// Overall success rate (0.0-1.0)
    pub success_rate: f64,

    /// Average response time across all operations
    pub average_response_time: f64,

    /// System uptime duration in seconds
    pub uptime_seconds: u64,

    /// Resource utilization percentage
    pub resource_utilization: f64,
}

/// Learning performance summary response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningPerformanceSummaryResponse {
    /// Overall health status
    pub overall_health: LearningHealthStatus,

    /// Key performance metrics
    pub key_metrics: std::collections::HashMap<String, f64>,

    /// Active alerts
    pub active_alerts: Vec<LearningAlert>,

    /// Performance trends over time
    pub performance_trends: std::collections::HashMap<String, Vec<f64>>,

    /// System recommendations
    pub recommendations: Vec<String>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Monitoring dashboard overview response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringDashboardResponse {
    /// Overall system status
    pub overall_status: String,

    /// Current system metrics snapshot
    pub current_metrics: MonitoringCurrentMetricsResponse,

    /// Health status of all monitored components
    pub health_status: MonitoringHealthStatusResponse,

    /// Active alerts for the monitoring system
    pub active_alerts: Vec<MonitoringAlertResponse>,

    /// Performance graphs data for dashboard visualization
    pub performance_graphs: std::collections::HashMap<String, Vec<MonitoringDataPoint>>,

    /// System overview summary
    pub system_overview: MonitoringSystemOverviewResponse,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Current metrics snapshot for monitoring dashboard
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringCurrentMetricsResponse {
    /// API performance metrics
    pub api_metrics: MonitoringApiMetricsResponse,

    /// Provider performance metrics
    pub provider_metrics: std::collections::HashMap<String, MonitoringProviderMetricsResponse>,

    /// Quality processing metrics
    pub quality_metrics: MonitoringQualityMetricsResponse,

    /// Cache performance metrics
    pub cache_metrics: std::collections::HashMap<String, MonitoringCacheMetricsResponse>,

    /// Learning system metrics
    pub learning_metrics: MonitoringLearningMetricsResponse,

    /// Resource utilization metrics
    pub resource_metrics: MonitoringResourceMetricsResponse,

    /// Metrics collection timestamp
    pub timestamp: DateTime<Utc>,
}

/// API performance metrics for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringApiMetricsResponse {
    /// Total number of API requests processed
    pub total_requests: u64,

    /// Number of successful requests
    pub successful_requests: u64,

    /// Number of failed requests
    pub failed_requests: u64,

    /// Average response time in milliseconds
    pub average_response_time_ms: f64,

    /// 95th percentile response time in milliseconds
    pub p95_response_time_ms: f64,

    /// Error rate (0.0-1.0)
    pub error_rate: f64,

    /// Requests by HTTP method
    pub requests_by_method: std::collections::HashMap<String, u64>,

    /// Requests by endpoint path
    pub requests_by_path: std::collections::HashMap<String, u64>,

    /// Last request timestamp
    pub last_request_time: Option<DateTime<Utc>>,
}

/// Provider performance metrics for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringProviderMetricsResponse {
    /// Provider name
    pub provider_name: String,

    /// Total requests to provider
    pub request_count: u64,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// Average latency in milliseconds
    pub average_latency_ms: f64,

    /// Error count
    pub error_count: u64,

    /// Last successful request timestamp
    pub last_success_time: DateTime<Utc>,
}

/// Quality processing metrics for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringQualityMetricsResponse {
    /// Total quality evaluations performed
    pub total_evaluations: u64,

    /// Average processing time in milliseconds
    pub average_processing_time_ms: f64,

    /// Total tokens processed
    pub total_tokens_processed: u64,

    /// Evaluations by operation type
    pub evaluations_by_type: std::collections::HashMap<String, u64>,
}

/// Cache performance metrics for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringCacheMetricsResponse {
    /// Cache name
    pub cache_name: String,

    /// Total cache operations
    pub total_operations: u64,

    /// Number of cache hits
    pub hit_count: u64,

    /// Number of cache misses
    pub miss_count: u64,

    /// Number of cache writes
    pub write_count: u64,

    /// Number of cache evictions
    pub eviction_count: u64,

    /// Hit rate (0.0-1.0)
    pub hit_rate: f64,

    /// Average hit time in milliseconds
    pub average_hit_time_ms: f64,

    /// Average miss time in milliseconds
    pub average_miss_time_ms: f64,
}

/// Learning system metrics for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringLearningMetricsResponse {
    /// Number of feedback items processed
    pub feedback_processed: u64,

    /// Number of patterns recognized
    pub patterns_recognized: u64,

    /// Number of adaptations applied
    pub adaptations_applied: u64,

    /// Learning accuracy (0.0-1.0)
    pub learning_accuracy: f64,

    /// Processing time in milliseconds
    pub processing_time_ms: f64,
}

/// Resource utilization metrics for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringResourceMetricsResponse {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,

    /// Memory usage in megabytes
    pub memory_usage_mb: f64,

    /// Network bytes sent
    pub network_bytes_sent: u64,

    /// Network bytes received
    pub network_bytes_received: u64,

    /// Disk I/O bytes
    pub disk_io_bytes: u64,

    /// Metrics timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health status for monitoring dashboard
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringHealthStatusResponse {
    /// Overall health status
    pub overall_status: MonitoringHealthStatus,

    /// Individual component health results
    pub component_results: Vec<MonitoringComponentHealthResponse>,

    /// Health summary message
    pub summary: String,

    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health status enumeration for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum MonitoringHealthStatus {
    /// All components healthy
    Healthy,
    /// Some components have warnings
    Warning,
    /// Critical issues detected
    Critical,
    /// Health status unknown
    Unknown,
}

/// Component health for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringComponentHealthResponse {
    /// Component name
    pub component: String,

    /// Health status
    pub status: MonitoringHealthStatus,

    /// Status message
    pub message: String,

    /// Check timestamp
    pub timestamp: DateTime<Utc>,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Additional details
    pub details: std::collections::HashMap<String, String>,
}

/// Monitoring system alert
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringAlertResponse {
    /// Alert ID
    pub id: String,

    /// Alert severity
    pub severity: MonitoringAlertSeverity,

    /// Component that triggered the alert
    pub component: String,

    /// Alert message
    pub message: String,

    /// Metric value that triggered the alert
    pub metric_value: f64,

    /// Threshold that was exceeded
    pub threshold: f64,

    /// Alert timestamp
    pub timestamp: DateTime<Utc>,

    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
}

/// Alert severity for monitoring
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum MonitoringAlertSeverity {
    /// Informational alert
    Info,
    /// Warning level alert
    Warning,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Data point for monitoring time-series graphs
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringDataPoint {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,

    /// Metric value
    pub value: f64,
}

/// System overview for monitoring dashboard
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringSystemOverviewResponse {
    /// Total number of operations performed
    pub total_operations: u64,

    /// Overall success rate (0.0-1.0)
    pub success_rate: f64,

    /// Average response time across all operations
    pub average_response_time_ms: f64,

    /// System uptime duration in seconds
    pub uptime_seconds: u64,

    /// Resource utilization percentage
    pub resource_utilization: f64,

    /// Number of active alerts
    pub active_alerts_count: usize,

    /// Number of threshold violations
    pub threshold_violations_count: usize,
}

/// Monitoring metrics endpoint response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringMetricsResponse {
    /// Current metrics snapshot
    pub metrics: MonitoringCurrentMetricsResponse,

    /// Total count of metrics collected
    pub total_count: u64,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Monitoring health endpoint response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringHealthResponse {
    /// Health status information
    pub health: MonitoringHealthStatusResponse,

    /// Overall health status summary
    pub overall_status: String,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Monitoring alerts endpoint response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringAlertsResponse {
    /// List of monitoring alerts
    pub alerts: Vec<MonitoringAlertResponse>,

    /// Total number of alerts available
    pub total_count: usize,

    /// Number of unacknowledged alerts
    pub unacknowledged_count: usize,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Monitoring performance summary response
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringPerformanceSummaryResponse {
    /// Overall health status
    pub overall_health: MonitoringHealthStatus,

    /// Key performance metrics
    pub key_metrics: std::collections::HashMap<String, f64>,

    /// Active alerts
    pub active_alerts: Vec<MonitoringAlertResponse>,

    /// Performance trends over time
    pub performance_trends: std::collections::HashMap<String, Vec<f64>>,

    /// System recommendations
    pub recommendations: Vec<String>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Learning insight data structure for API responses  
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LearningInsight {
    /// Unique identifier for the insight
    pub id: String,

    /// Type of insight (user_preference, performance_optimization, etc.)
    pub insight_type: String,

    /// Content description of the insight
    pub content: String,

    /// Confidence score for this insight (0.0-1.0)
    pub confidence_score: f64,

    /// Number of data points that contributed to this insight
    pub source_data_count: usize,

    /// When this insight was created
    pub created_at: DateTime<Utc>,

    /// Tags associated with this insight
    pub tags: Vec<String>,
}

/// Monitoring metrics query parameters
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MonitoringMetricsQuery {
    /// Start time for metrics query
    pub start_time: Option<DateTime<Utc>>,

    /// End time for metrics query
    pub end_time: Option<DateTime<Utc>>,

    /// Specific metrics to include
    pub metrics: Option<Vec<String>>,

    /// Aggregation interval in seconds
    pub interval_seconds: Option<u64>,

    /// Component filter
    pub component: Option<String>,
}

/// Health check request parameters
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct HealthCheckRequest {
    /// Components to check (if None, check all)
    pub components: Option<Vec<String>>,

    /// Include detailed diagnostics
    pub include_details: Option<bool>,

    /// Timeout for health checks in seconds
    pub timeout_seconds: Option<u64>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T, request_id: Uuid) -> Self {
        Self {
            data,
            request_id,
            timestamp: Utc::now(),
            success: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_creation() {
        let request_id = Uuid::new_v4();
        let data = "test data";

        let response = ApiResponse::success(data, request_id);

        assert_eq!(response.data, data);
        assert_eq!(response.request_id, request_id);
        assert!(response.success);
    }

    #[test]
    fn test_research_response_serialization() {
        let response = ResearchResponse {
            id: "test-id".to_string(),
            query: "Test query".to_string(),
            research_type: "implementation".to_string(),
            immediate_answer: "Test answer".to_string(),
            supporting_evidence: vec![],
            implementation_details: vec![],
            metadata: ResearchMetadata {
                completed_at: Utc::now(),
                processing_time_ms: 100,
                sources_consulted: vec![],
                quality_score: 0.8,
                tags: std::collections::HashMap::new(),
            },
            processing_time_ms: 100,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_research_list_response_serialization() {
        let response = ResearchListResponse {
            results: vec![],
            total_count: 0,
            pagination: PaginationInfo {
                offset: 0,
                limit: 10,
                total_pages: 0,
                has_more: false,
            },
            processing_time_ms: 50,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_classification_response_serialization() {
        let response = ClassificationResponse {
            id: "classification-123".to_string(),
            content: "Test content".to_string(),
            research_type: ClassificationResult {
                research_type: "implementation".to_string(),
                confidence: 0.85,
                matched_keywords: vec!["implement".to_string(), "build".to_string()],
                rule_priority: 1,
                candidates: vec![],
            },
            context: Some(ContextDetectionResponse {
                audience_level: "intermediate".to_string(),
                technical_domain: "rust".to_string(),
                urgency_level: "planned".to_string(),
                dimension_confidences: vec![],
                overall_confidence: 0.8,
                fallback_used: false,
            }),
            metadata: ClassificationMetadata {
                completed_at: Utc::now(),
                processing_time_ms: 50,
                advanced_classification_used: false,
                context_detection_used: true,
                algorithm_version: "1.0".to_string(),
                tags: std::collections::HashMap::new(),
            },
            processing_time_ms: 50,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_classification_types_response_serialization() {
        let response = ClassificationTypesResponse {
            research_types: vec![ResearchTypeInfo {
                name: "implementation".to_string(),
                display_name: "Implementation".to_string(),
                description: "How to implement or build something".to_string(),
                example_keywords: vec!["implement".to_string(), "build".to_string()],
            }],
            audience_levels: vec![AudienceLevelInfo {
                name: "intermediate".to_string(),
                display_name: "Intermediate".to_string(),
                description: "Has some experience with the technology".to_string(),
            }],
            technical_domains: vec![TechnicalDomainInfo {
                name: "rust".to_string(),
                display_name: "Rust".to_string(),
                description: "Rust programming language".to_string(),
            }],
            urgency_levels: vec![UrgencyLevelInfo {
                name: "planned".to_string(),
                display_name: "Planned".to_string(),
                description: "Non-urgent, planned work".to_string(),
            }],
            system_info: ClassificationSystemInfo {
                version: "1.0.0".to_string(),
                default_confidence_threshold: 0.6,
                advanced_classification_available: true,
                context_detection_available: true,
            },
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_cache_stats_response_serialization() {
        let response = CacheStatsResponse {
            total_entries: 100,
            expired_entries: 5,
            total_size_bytes: 1024 * 1024,
            hit_rate: 0.85,
            hits: 850,
            misses: 150,
            average_age_seconds: 3600.0,
            by_research_type: std::collections::HashMap::new(),
            storage_efficiency: StorageEfficiencyResponse {
                utilization_percent: 75.0,
                duplicate_entries: 3,
                bytes_saved: 512,
                compression_ratio: Some(0.8),
            },
            performance_metrics: CachePerformanceResponse {
                avg_retrieval_time_ms: 15.5,
                avg_storage_time_ms: 25.0,
                warming_status: "complete".to_string(),
                recent_operations: RecentOperationsResponse {
                    last_hour: 50,
                    last_day: 1200,
                    peak_hour: "14:00-15:00".to_string(),
                    top_accessed: vec!["key1".to_string(), "key2".to_string()],
                },
            },
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_cache_search_response_serialization() {
        let response = CacheSearchResponse {
            results: vec![CacheItemResponse {
                key: "test-key".to_string(),
                research_type: "implementation".to_string(),
                original_query: "Test query".to_string(),
                content_summary: "Test content summary".to_string(),
                quality_score: 0.9,
                size_bytes: 1024,
                created_at: Utc::now(),
                last_accessed: Utc::now(),
                expires_at: Utc::now(),
                is_expired: false,
                content_hash: "hash123".to_string(),
                file_path: Some("/path/to/file".to_string()),
                metadata: std::collections::HashMap::new(),
                tags: vec!["rust".to_string()],
            }],
            total_count: 1,
            pagination: PaginationInfo {
                offset: 0,
                limit: 10,
                total_pages: 1,
                has_more: false,
            },
            search_metadata: CacheSearchMetadata {
                query: Some("test".to_string()),
                filters_applied: vec!["research_type".to_string()],
                sort_order: "relevance".to_string(),
                total_before_filters: 1,
                search_time_ms: 25,
            },
            processing_time_ms: 50,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_cache_invalidate_response_serialization() {
        let response = CacheInvalidateResponse {
            status: "success".to_string(),
            invalidated_count: 5,
            invalidated_keys: vec!["key1".to_string(), "key2".to_string()],
            bytes_freed: 5120,
            dry_run: false,
            criteria: CacheInvalidationCriteria {
                keys: Some(vec!["key1".to_string()]),
                pattern: None,
                research_type: Some("implementation".to_string()),
                tags: None,
                max_age_seconds: Some(3600),
                min_quality: None,
            },
            processing_time_ms: 100,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_cache_cleanup_response_serialization() {
        let response = CacheCleanupResponse {
            status: "success".to_string(),
            cleaned_count: 10,
            orphaned_files_removed: 2,
            bytes_freed: 10240,
            cleanup_summary: CleanupSummary {
                by_research_type: std::collections::HashMap::new(),
                age_distribution: vec![AgeDistribution {
                    range: "0-1h".to_string(),
                    count: 5,
                    bytes: 2048,
                }],
                size_distribution: vec![SizeDistribution {
                    range: "0-1KB".to_string(),
                    count: 3,
                    bytes: 1024,
                }],
                recommendations: vec!["Consider increasing cache retention".to_string()],
            },
            processing_time_ms: 200,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_proactive_status_response_serialization() {
        let response = ProactiveStatusResponse {
            is_running: true,
            status: "active".to_string(),
            uptime_seconds: Some(3600),
            monitored_files_count: 150,
            active_tasks_count: 3,
            completed_tasks_today: 12,
            pending_tasks_count: 5,
            last_gap_detection: Some(Utc::now()),
            health_metrics: ProactiveHealthMetrics {
                file_monitor_status: "healthy".to_string(),
                scheduler_status: "active".to_string(),
                executor_status: "running".to_string(),
                notification_status: "operational".to_string(),
                error_count_24h: 1,
                memory_usage_percent: 45.2,
                cpu_usage_percent: 12.8,
            },
            processing_time_ms: 25,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_proactive_config_response_serialization() {
        let response = ProactiveConfigResponse {
            base_directory: "/path/to/monitor".to_string(),
            file_patterns: vec!["*.rs".to_string(), "*.md".to_string()],
            ignore_patterns: vec!["target/".to_string(), "*.log".to_string()],
            enabled: true,
            monitoring_interval_seconds: 300,
            max_concurrent_tasks: 5,
            priority_threshold: 0.7,
            auto_execute_high_priority: true,
            notification_preferences: ProactiveNotificationPreferencesResponse {
                gap_detection_enabled: true,
                research_completion_enabled: true,
                error_notifications_enabled: true,
                frequency: "immediate".to_string(),
                min_priority_level: "medium".to_string(),
            },
            last_updated: Utc::now(),
            processing_time_ms: 15,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_proactive_task_response_serialization() {
        let response = ProactiveTaskResponse {
            id: "task-123".to_string(),
            description: "Research async patterns for error handling".to_string(),
            research_type: "implementation".to_string(),
            gap_type: "documentation".to_string(),
            status: "running".to_string(),
            priority: "high".to_string(),
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: None,
            estimated_completion: Some(Utc::now()),
            trigger_file: Some("/src/main.rs".to_string()),
            gap_details: ProactiveGapDetails {
                gap_type: "documentation".to_string(),
                description: "Missing error handling documentation".to_string(),
                severity: 0.8,
                file_path: "/src/main.rs".to_string(),
                line_number: Some(42),
                context: "// TODO: Add error handling".to_string(),
                suggested_keywords: vec!["error handling".to_string(), "async".to_string()],
            },
            progress: Some(ProactiveTaskProgress {
                percentage: 0.6,
                current_step: "Analyzing context".to_string(),
                total_steps: 5,
                current_step_number: 3,
                estimated_time_remaining_seconds: Some(120),
            }),
            result: None,
            error: None,
            tags: vec!["rust".to_string(), "async".to_string()],
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_proactive_task_list_response_serialization() {
        let response = ProactiveTaskListResponse {
            tasks: vec![],
            total_count: 0,
            pagination: PaginationInfo {
                offset: 0,
                limit: 20,
                total_pages: 0,
                has_more: false,
            },
            task_statistics: ProactiveTaskStatistics {
                by_status: std::collections::HashMap::new(),
                by_priority: std::collections::HashMap::new(),
                by_research_type: std::collections::HashMap::new(),
                by_gap_type: std::collections::HashMap::new(),
                avg_completion_time_seconds: 180.0,
                success_rate_percent: 85.5,
            },
            processing_time_ms: 35,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_proactive_notification_response_serialization() {
        let response = ProactiveNotificationResponse {
            id: "notification-456".to_string(),
            notification_type: "gap_detected".to_string(),
            level: "warning".to_string(),
            title: "Documentation gap detected".to_string(),
            message: "Missing documentation for error handling in main.rs".to_string(),
            read: false,
            created_at: Utc::now(),
            read_at: None,
            expires_at: None,
            related_task_id: Some("task-123".to_string()),
            related_file_path: Some("/src/main.rs".to_string()),
            metadata: std::collections::HashMap::new(),
            actions: vec![ProactiveNotificationAction {
                id: "view_task".to_string(),
                label: "View Task".to_string(),
                action_type: "api_call".to_string(),
                target: "/api/proactive/tasks/task-123".to_string(),
                primary: true,
            }],
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_proactive_notification_list_response_serialization() {
        let response = ProactiveNotificationListResponse {
            notifications: vec![],
            total_count: 0,
            unread_count: 0,
            pagination: PaginationInfo {
                offset: 0,
                limit: 20,
                total_pages: 0,
                has_more: false,
            },
            notification_statistics: ProactiveNotificationStatistics {
                by_type: std::collections::HashMap::new(),
                by_level: std::collections::HashMap::new(),
                read_status: std::collections::HashMap::new(),
                created_last_24h: 5,
                avg_time_to_read_seconds: 300.0,
            },
            processing_time_ms: 20,
        };

        let serialized = serde_json::to_string(&response);
        assert!(serialized.is_ok());
    }
}
