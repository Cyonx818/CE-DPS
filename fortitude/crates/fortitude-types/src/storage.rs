// ABOUTME: Storage types for the Fortitude research system
use crate::research::{ResearchResult, ResearchType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Storage configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for reference library
    pub base_path: PathBuf,
    /// Cache expiration time in seconds
    pub cache_expiration_seconds: u64,
    /// Maximum cache size in bytes
    pub max_cache_size_bytes: u64,
    /// Enable content-addressed storage
    pub enable_content_addressing: bool,
    /// Index update interval in seconds
    pub index_update_interval_seconds: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./reference_library"),
            cache_expiration_seconds: 86400,         // 24 hours
            max_cache_size_bytes: 1024 * 1024 * 100, // 100MB
            enable_content_addressing: true,
            index_update_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Unique cache key
    pub key: String,
    /// Path to cached file
    pub file_path: PathBuf,
    /// Research type
    pub research_type: ResearchType,
    /// Original query
    pub original_query: String,
    /// Entry creation timestamp
    pub created_at: DateTime<Utc>,
    /// Entry last accessed timestamp
    pub last_accessed: DateTime<Utc>,
    /// Entry expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// File size in bytes
    pub size_bytes: u64,
    /// Content hash for deduplication
    pub content_hash: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(
        key: String,
        file_path: PathBuf,
        research_type: ResearchType,
        original_query: String,
        size_bytes: u64,
        content_hash: String,
        expiration_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(expiration_seconds as i64);

        Self {
            key,
            file_path,
            research_type,
            original_query,
            created_at: now,
            last_accessed: now,
            expires_at,
            size_bytes,
            content_hash,
            metadata: HashMap::new(),
        }
    }

    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = Utc::now();
        (now - self.created_at).num_seconds().max(0) as u64
    }
}

/// Cache statistics with enhanced analytics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of entries
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
    pub by_research_type: HashMap<ResearchType, CacheTypeStats>,
    /// Enhanced analytics
    pub analytics: CacheAnalytics,
    /// Newest cache entry
    pub newest_entry: Option<CacheEntry>,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            expired_entries: 0,
            total_size_bytes: 0,
            hit_rate: 0.0,
            hits: 0,
            misses: 0,
            average_age_seconds: 0.0,
            by_research_type: HashMap::new(),
            analytics: CacheAnalytics::default(),
            newest_entry: None,
        }
    }
}

/// Cache statistics by research type with enhanced metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheTypeStats {
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
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache warming effectiveness
    pub warming_effectiveness: f64,
    /// Key optimization score
    pub key_optimization_score: f64,
}

impl Default for CacheTypeStats {
    fn default() -> Self {
        Self {
            entries: 0,
            size_bytes: 0,
            hit_rate: 0.0,
            hits: 0,
            misses: 0,
            avg_response_time_ms: 0.0,
            warming_effectiveness: 0.0,
            key_optimization_score: 0.0,
        }
    }
}

/// Enhanced cache analytics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CacheAnalytics {
    /// Performance metrics over time
    pub performance_trends: Vec<CachePerformanceTrend>,
    /// Hit rate trends by endpoint
    pub hit_rate_trends: HashMap<String, Vec<HitRateTrend>>,
    /// Cache key optimization metrics
    pub key_optimization: CacheKeyOptimization,
    /// Cache warming statistics
    pub warming_stats: CacheWarmingStats,
    /// Response time analytics
    pub response_times: ResponseTimeAnalytics,
    /// Content effectiveness metrics
    pub content_effectiveness: ContentEffectivenessMetrics,
}

/// Cache performance trend data point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachePerformanceTrend {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// Hit rate at this time
    pub hit_rate: f64,
    /// Response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache size in bytes
    pub cache_size_bytes: u64,
    /// Number of entries
    pub entry_count: usize,
}

/// Hit rate trend for specific endpoint or content type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitRateTrend {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// Hit rate for this specific dimension
    pub hit_rate: f64,
    /// Number of requests
    pub request_count: u64,
    /// Context information
    pub context: String,
}

/// Cache key optimization metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheKeyOptimization {
    /// Key collision rate
    pub collision_rate: f64,
    /// Key normalization effectiveness
    pub normalization_effectiveness: f64,
    /// Multi-dimensional key performance
    pub dimensional_performance: HashMap<String, f64>,
    /// Key space utilization
    pub key_space_utilization: f64,
    /// Recommendations for optimization
    pub optimization_recommendations: Vec<String>,
}

impl Default for CacheKeyOptimization {
    fn default() -> Self {
        Self {
            collision_rate: 0.0,
            normalization_effectiveness: 0.0,
            dimensional_performance: HashMap::new(),
            key_space_utilization: 0.0,
            optimization_recommendations: Vec::new(),
        }
    }
}

/// Cache warming statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheWarmingStats {
    /// Warming strategy effectiveness
    pub strategy_effectiveness: f64,
    /// Items warmed successfully
    pub items_warmed: u64,
    /// Items failed to warm
    pub warming_failures: u64,
    /// Average warming time
    pub avg_warming_time_ms: f64,
    /// Warming hit rate improvement
    pub hit_rate_improvement: f64,
    /// Active warming strategies
    pub active_strategies: Vec<String>,
}

impl Default for CacheWarmingStats {
    fn default() -> Self {
        Self {
            strategy_effectiveness: 0.0,
            items_warmed: 0,
            warming_failures: 0,
            avg_warming_time_ms: 0.0,
            hit_rate_improvement: 0.0,
            active_strategies: Vec::new(),
        }
    }
}

/// Response time analytics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseTimeAnalytics {
    /// Average response time for cache hits
    pub cache_hit_avg_ms: f64,
    /// Average response time for cache misses
    pub cache_miss_avg_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// 99th percentile response time
    pub p99_response_time_ms: f64,
    /// Response time trends
    pub time_trends: Vec<ResponseTimeTrend>,
}

impl Default for ResponseTimeAnalytics {
    fn default() -> Self {
        Self {
            cache_hit_avg_ms: 0.0,
            cache_miss_avg_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            time_trends: Vec::new(),
        }
    }
}

/// Response time trend data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseTimeTrend {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Hit rate at this time
    pub hit_rate: f64,
    /// Request volume
    pub request_count: u64,
}

/// Content effectiveness metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentEffectivenessMetrics {
    /// Content quality correlation with hit rate
    pub quality_hit_correlation: f64,
    /// Most accessed content types
    pub top_content_types: Vec<ContentTypeStats>,
    /// Cache efficiency by content size
    pub size_efficiency_correlation: f64,
    /// Content freshness impact
    pub freshness_impact_score: f64,
}

impl Default for ContentEffectivenessMetrics {
    fn default() -> Self {
        Self {
            quality_hit_correlation: 0.0,
            top_content_types: Vec::new(),
            size_efficiency_correlation: 0.0,
            freshness_impact_score: 0.0,
        }
    }
}

/// Content type statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentTypeStats {
    /// Content type identifier
    pub content_type: String,
    /// Hit rate for this content type
    pub hit_rate: f64,
    /// Access frequency
    pub access_count: u64,
    /// Average quality score
    pub avg_quality: f64,
    /// Cache effectiveness score
    pub effectiveness_score: f64,
}

/// Cache performance monitoring state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachePerformanceMonitor {
    /// Performance target hit rate
    pub target_hit_rate: f64,
    /// Current hit rate
    pub current_hit_rate: f64,
    /// Performance status
    pub status: CachePerformanceStatus,
    /// Recent operations tracking
    pub recent_operations: Vec<CacheOperation>,
    /// Performance alerts
    pub alerts: Vec<CachePerformanceAlert>,
}

/// Cache performance status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CachePerformanceStatus {
    /// Performance meets targets
    Optimal,
    /// Performance below targets but acceptable
    Degraded,
    /// Performance significantly below targets
    Critical,
    /// Monitoring system error
    Unknown,
}

/// Cache operation tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheOperation {
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Operation type
    pub operation_type: CacheOperationType,
    /// Cache key involved
    pub cache_key: String,
    /// Operation duration
    pub duration_ms: u64,
    /// Whether operation was successful
    pub success: bool,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Cache operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CacheOperationType {
    /// Cache hit - successful retrieval
    Hit,
    /// Cache miss - item not found
    Miss,
    /// Cache store operation
    Store,
    /// Cache eviction
    Evict,
    /// Cache warming operation
    Warm,
    /// Cache invalidation
    Invalidate,
}

/// Cache performance alert
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachePerformanceAlert {
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Alert category
    pub category: AlertCategory,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertCategory {
    HitRate,
    ResponseTime,
    CacheSize,
    KeyOptimization,
    Warming,
    ContentEffectiveness,
}

/// Search index entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Unique identifier
    pub id: Uuid,
    /// Cache key
    pub cache_key: String,
    /// Research type
    pub research_type: ResearchType,
    /// Original query
    pub original_query: String,
    /// Searchable content
    pub content: String,
    /// Keywords for search
    pub keywords: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Quality score (0.0-1.0)
    pub quality_score: f64,
    /// Index timestamp
    pub indexed_at: DateTime<Utc>,
    /// Full text search vector (for future implementation)
    pub search_vector: Option<Vec<f32>>,
}

impl IndexEntry {
    /// Create a new index entry
    pub fn new(
        cache_key: String,
        research_type: ResearchType,
        original_query: String,
        content: String,
        keywords: Vec<String>,
        tags: Vec<String>,
        quality_score: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            cache_key,
            research_type,
            original_query,
            content,
            keywords,
            tags,
            quality_score,
            indexed_at: Utc::now(),
            search_vector: None,
        }
    }
}

/// Search query parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search text
    pub query: String,
    /// Filter by research type
    pub research_type: Option<ResearchType>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Minimum quality score
    pub min_quality: Option<f64>,
    /// Maximum results to return
    pub limit: Option<usize>,
    /// Result offset for pagination
    pub offset: Option<usize>,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(query: String) -> Self {
        Self {
            query,
            research_type: None,
            tags: vec![],
            min_quality: None,
            limit: Some(10),
            offset: Some(0),
        }
    }

    /// Set research type filter
    pub fn with_research_type(mut self, research_type: ResearchType) -> Self {
        self.research_type = Some(research_type);
        self
    }

    /// Set tags filter
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set quality filter
    pub fn with_min_quality(mut self, quality: f64) -> Self {
        self.min_quality = Some(quality);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set result offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Search result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching index entry
    pub entry: IndexEntry,
    /// Relevance score (0.0-1.0)
    pub relevance_score: f64,
    /// Matching keywords
    pub matched_keywords: Vec<String>,
    /// Snippet of matching content
    pub snippet: String,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        entry: IndexEntry,
        relevance_score: f64,
        matched_keywords: Vec<String>,
        snippet: String,
    ) -> Self {
        Self {
            entry,
            relevance_score,
            matched_keywords,
            snippet,
        }
    }
}

/// Trait for storage systems with enhanced performance monitoring
#[async_trait::async_trait]
pub trait Storage {
    /// Store a research result
    async fn store(&self, result: &ResearchResult) -> Result<String, crate::error::StorageError>;

    /// Retrieve a research result by cache key
    async fn retrieve(
        &self,
        cache_key: &str,
    ) -> Result<Option<ResearchResult>, crate::error::StorageError>;

    /// Delete a research result by cache key
    async fn delete(&self, cache_key: &str) -> Result<(), crate::error::StorageError>;

    /// List all cache entries
    async fn list_cache_entries(&self) -> Result<Vec<CacheEntry>, crate::error::StorageError>;

    /// Get cache statistics with enhanced analytics
    async fn get_cache_stats(&self) -> Result<CacheStats, crate::error::StorageError>;

    /// Clean up expired entries
    async fn cleanup_expired(&self) -> Result<u64, crate::error::StorageError>;

    /// Search research results
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, crate::error::StorageError>;

    /// Update search index
    async fn update_index(&self) -> Result<(), crate::error::StorageError>;

    /// Record cache operation for performance monitoring
    async fn record_cache_operation(
        &self,
        operation: CacheOperation,
    ) -> Result<(), crate::error::StorageError>;

    /// Get performance monitor state
    async fn get_performance_monitor(
        &self,
    ) -> Result<CachePerformanceMonitor, crate::error::StorageError>;

    /// Update cache analytics
    async fn update_analytics(
        &self,
        analytics: CacheAnalytics,
    ) -> Result<(), crate::error::StorageError>;

    /// Get cache key optimization recommendations
    async fn get_key_optimization_recommendations(
        &self,
    ) -> Result<Vec<String>, crate::error::StorageError>;

    /// Warm cache with specific entries
    async fn warm_cache(
        &self,
        entries: Vec<String>,
    ) -> Result<CacheWarmingStats, crate::error::StorageError>;

    /// Get hit rate trends for analysis
    async fn get_hit_rate_trends(
        &self,
        timeframe_hours: u64,
    ) -> Result<Vec<HitRateTrend>, crate::error::StorageError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::*;

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.base_path, PathBuf::from("./reference_library"));
        assert_eq!(config.cache_expiration_seconds, 86400);
        assert!(config.enable_content_addressing);
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new(
            "test-key".to_string(),
            PathBuf::from("test.json"),
            ResearchType::Learning,
            "Test query".to_string(),
            1024,
            "hash123".to_string(),
            3600,
        );

        assert_eq!(entry.key, "test-key");
        assert_eq!(entry.research_type, ResearchType::Learning);
        assert_eq!(entry.size_bytes, 1024);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let mut entry = CacheEntry::new(
            "test-key".to_string(),
            PathBuf::from("test.json"),
            ResearchType::Learning,
            "Test query".to_string(),
            1024,
            "hash123".to_string(),
            0, // Expires immediately
        );

        // Give it a moment to expire
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(entry.is_expired());

        // Test touch functionality
        entry.touch();
        assert!(entry.last_accessed > entry.created_at);
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("rust async".to_string())
            .with_research_type(ResearchType::Implementation)
            .with_tags(vec!["rust".to_string(), "async".to_string()])
            .with_min_quality(0.8)
            .with_limit(5)
            .with_offset(10);

        assert_eq!(query.query, "rust async");
        assert_eq!(query.research_type, Some(ResearchType::Implementation));
        assert_eq!(query.tags.len(), 2);
        assert_eq!(query.min_quality, Some(0.8));
        assert_eq!(query.limit, Some(5));
        assert_eq!(query.offset, Some(10));
    }

    #[test]
    fn test_index_entry_creation() {
        let entry = IndexEntry::new(
            "cache-key".to_string(),
            ResearchType::Validation,
            "How to test?".to_string(),
            "Testing content".to_string(),
            vec!["test".to_string(), "validation".to_string()],
            vec!["testing".to_string()],
            0.85,
        );

        assert_eq!(entry.cache_key, "cache-key");
        assert_eq!(entry.research_type, ResearchType::Validation);
        assert_eq!(entry.quality_score, 0.85);
        assert_eq!(entry.keywords.len(), 2);
        assert_eq!(entry.tags.len(), 1);
    }

    #[test]
    fn test_search_result_creation() {
        let entry = IndexEntry::new(
            "cache-key".to_string(),
            ResearchType::Decision,
            "What to choose?".to_string(),
            "Decision content".to_string(),
            vec!["choose".to_string(), "decision".to_string()],
            vec!["decision-making".to_string()],
            0.9,
        );

        let result = SearchResult::new(
            entry,
            0.95,
            vec!["choose".to_string()],
            "...to choose...".to_string(),
        );

        assert_eq!(result.relevance_score, 0.95);
        assert_eq!(result.matched_keywords.len(), 1);
        assert_eq!(result.snippet, "...to choose...");
    }
}
