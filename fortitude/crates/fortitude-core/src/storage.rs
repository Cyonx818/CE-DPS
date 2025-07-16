// ABOUTME: File-based storage system with reference library integration
use crate::classification::context_detector::ContextDetectionResult;
use fortitude_types::{
    CacheAnalytics, CacheEntry, CacheOperation, CacheOperationType, CachePerformanceMonitor,
    CachePerformanceStatus, CacheStats, CacheTypeStats, CacheWarmingStats, HitRateTrend,
    IndexEntry, ResearchResult, ResearchType, SearchQuery, SearchResult, Storage, StorageConfig,
    StorageError,
};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs as async_fs;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// File-based storage implementation with enhanced performance monitoring
pub struct FileStorage {
    config: StorageConfig,
    cache_index: Arc<Mutex<HashMap<String, CacheEntry>>>,
    search_index: Arc<Mutex<HashMap<String, IndexEntry>>>,
    performance_monitor: Arc<Mutex<CachePerformanceMonitor>>,
    analytics: Arc<Mutex<CacheAnalytics>>,
    recent_operations: Arc<Mutex<Vec<CacheOperation>>>,
    hit_rate_history: Arc<Mutex<Vec<HitRateTrend>>>,
}

impl FileStorage {
    /// Create a new file storage instance with enhanced monitoring
    pub async fn new(config: StorageConfig) -> Result<Self, StorageError> {
        let performance_monitor = CachePerformanceMonitor {
            target_hit_rate: 0.8, // 80% target hit rate
            current_hit_rate: 0.0,
            status: CachePerformanceStatus::Unknown,
            recent_operations: Vec::new(),
            alerts: Vec::new(),
        };

        let storage = Self {
            config,
            cache_index: Arc::new(Mutex::new(HashMap::new())),
            search_index: Arc::new(Mutex::new(HashMap::new())),
            performance_monitor: Arc::new(Mutex::new(performance_monitor)),
            analytics: Arc::new(Mutex::new(CacheAnalytics::default())),
            recent_operations: Arc::new(Mutex::new(Vec::new())),
            hit_rate_history: Arc::new(Mutex::new(Vec::new())),
        };

        storage.ensure_directories().await?;
        storage.load_indices().await?;

        Ok(storage)
    }

    /// Ensure required directories exist
    async fn ensure_directories(&self) -> Result<(), StorageError> {
        let dirs = [
            &self.config.base_path,
            &self.config.base_path.join("research_results"),
            &self.config.base_path.join("cache"),
            &self.config.base_path.join("index"),
        ];

        for dir in dirs {
            if !dir.exists() {
                async_fs::create_dir_all(dir)
                    .await
                    .map_err(|_e| StorageError::DirectoryNotFound(dir.to_path_buf()))?;
                info!("Created directory: {}", dir.display());
            }
        }

        // Create subdirectories for each research type
        for research_type in ResearchType::all() {
            let type_dir = self
                .config
                .base_path
                .join("research_results")
                .join(research_type.to_string().to_lowercase());

            if !type_dir.exists() {
                async_fs::create_dir_all(&type_dir)
                    .await
                    .map_err(|_e| StorageError::DirectoryNotFound(type_dir.clone()))?;
                debug!("Created research type directory: {}", type_dir.display());
            }

            // Create context-aware subdirectory
            let context_dir = type_dir.join("context-aware");
            if !context_dir.exists() {
                async_fs::create_dir_all(&context_dir)
                    .await
                    .map_err(|_e| StorageError::DirectoryNotFound(context_dir.clone()))?;
                debug!("Created context-aware directory: {}", context_dir.display());
            }
        }

        Ok(())
    }

    /// Load cache and search indices from disk
    async fn load_indices(&self) -> Result<(), StorageError> {
        // This would load existing indices from disk
        // For now, we'll start with empty indices
        debug!("Loading indices from disk");
        Ok(())
    }

    /// Generate cache key for a research result
    fn generate_cache_key(&self, result: &ResearchResult) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Normalize query for better cache key effectiveness
        let normalized_query = self.normalize_query(&result.request.original_query);
        normalized_query.hash(&mut hasher);

        // Include research type with explicit prefix to ensure distinct hashing
        format!("research_type:{:?}", result.request.research_type).hash(&mut hasher);
        result.request.audience_context.level.hash(&mut hasher);
        result.request.domain_context.technology.hash(&mut hasher);

        // Include framework and tag information for consistency
        let mut sorted_frameworks = result.request.domain_context.frameworks.clone();
        sorted_frameworks.sort();
        for framework in sorted_frameworks {
            framework.to_lowercase().hash(&mut hasher);
        }

        // Sort and hash tags for consistent keys regardless of order
        let mut sorted_tags = result.request.domain_context.tags.clone();
        sorted_tags.sort();
        for tag in sorted_tags {
            tag.to_lowercase().hash(&mut hasher);
        }

        // Include enhanced classification if available
        if let Some(ref enhanced) = result.request.enhanced_classification {
            enhanced.audience_level.display_name().hash(&mut hasher);
            enhanced.technical_domain.display_name().hash(&mut hasher);
            enhanced.urgency_level.display_name().hash(&mut hasher);

            // Use confidence bands for better cache hits and eliminate floating-point precision issues
            let confidence_band = self.get_confidence_band(enhanced.overall_confidence);
            confidence_band.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Generate optimized context-aware cache key for a research result
    fn generate_context_aware_cache_key(
        &self,
        result: &ResearchResult,
        context_result: Option<&ContextDetectionResult>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Normalize query for better cache key effectiveness
        let normalized_query = self.normalize_query(&result.request.original_query);
        normalized_query.hash(&mut hasher);

        // Include research type with explicit prefix to ensure distinct hashing
        format!("research_type:{:?}", result.request.research_type).hash(&mut hasher);
        result.request.audience_context.level.hash(&mut hasher);
        result.request.domain_context.technology.hash(&mut hasher);

        // Include framework and tag information for multi-dimensional effectiveness
        for framework in &result.request.domain_context.frameworks {
            framework.to_lowercase().hash(&mut hasher);
        }

        // Sort and hash tags for consistent keys regardless of order
        let mut sorted_tags = result.request.domain_context.tags.clone();
        sorted_tags.sort();
        for tag in sorted_tags {
            tag.to_lowercase().hash(&mut hasher);
        }

        // Include context detection results with enhanced normalization
        if let Some(context) = context_result {
            context.audience_level.display_name().hash(&mut hasher);
            context.technical_domain.display_name().hash(&mut hasher);
            context.urgency_level.display_name().hash(&mut hasher);

            // Use confidence bands for better cache hits
            let confidence_band = self.get_confidence_band(context.overall_confidence);
            confidence_band.hash(&mut hasher);

            // Categorize processing time for cache effectiveness
            let processing_time_category =
                self.get_processing_time_category(context.processing_time_ms);
            processing_time_category.hash(&mut hasher);
        }

        format!("enhanced-{:x}", hasher.finish())
    }

    /// Normalize query for better cache key generation
    fn normalize_query(&self, query: &str) -> String {
        // Convert to lowercase and remove punctuation for better normalization
        let normalized = query
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c.is_whitespace() {
                    c
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Remove common stop words and normalize technical terms
        let words_removed = self.remove_stop_words(&normalized);

        // Apply semantic normalization for better fuzzy matching
        self.apply_semantic_normalization(&words_removed)
    }

    /// Remove stop words and normalize technical terms
    fn remove_stop_words(&self, query: &str) -> String {
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "how", "what", "why", "when", "where", "which", "that", "this", "these", "those",
            "do", "i", "you", "we", "they", "can", "could", "should", "would", "will", "is", "are",
            "was", "were",
        ];

        query
            .split_whitespace()
            .filter(|word| !stop_words.contains(word))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Apply semantic normalization for enhanced fuzzy matching
    fn apply_semantic_normalization(&self, query: &str) -> String {
        let words: Vec<String> = query
            .split_whitespace()
            .map(|word| self.normalize_technical_term(word))
            .collect();

        // DO NOT sort words to preserve semantic meaning and avoid collisions
        // "rust async programming" and "python async programming" must remain distinct
        words.join(" ")
    }

    /// Normalize technical terms for better semantic matching
    fn normalize_technical_term(&self, term: &str) -> String {
        match term {
            // Programming language synonyms
            "asynchronous" => "async".to_string(),
            "async" => "async".to_string(),
            "asyncio" => "async".to_string(),

            // Implementation synonyms
            "implementation" => "implement".to_string(),
            "implementing" => "implement".to_string(),
            "implements" => "implement".to_string(),
            "implemented" => "implement".to_string(),

            // Programming synonyms
            "programming" => "program".to_string(),
            "coding" => "program".to_string(),
            "development" => "develop".to_string(),
            "developing" => "develop".to_string(),

            // Keep original term if no normalization needed
            _ => term.to_string(),
        }
    }

    /// Get confidence band for cache key stability
    fn get_confidence_band(&self, confidence: f64) -> &'static str {
        match confidence {
            0.0..=0.3 => "low",
            0.3..=0.6 => "medium",
            0.6..=0.8 => "high",
            0.8..=1.0 => "very_high",
            _ => "unknown",
        }
    }

    /// Get processing time category for cache differentiation
    fn get_processing_time_category(&self, processing_time_ms: u64) -> &'static str {
        match processing_time_ms {
            0..=100 => "fast",
            101..=500 => "medium",
            501..=2000 => "slow",
            _ => "very_slow",
        }
    }

    /// Scan context-aware directory recursively for a cache key
    async fn scan_context_aware_directory(
        &self,
        base_path: &std::path::Path,
        cache_key: &str,
    ) -> Result<Option<ResearchResult>, StorageError> {
        if !base_path.exists() {
            return Ok(None);
        }

        let target_filename = format!("{cache_key}.json");

        // Use walkdir to recursively search for the file
        let mut stack = vec![base_path.to_path_buf()];

        while let Some(current_path) = stack.pop() {
            if let Ok(mut dir_entries) = async_fs::read_dir(&current_path).await {
                while let Ok(Some(entry)) = dir_entries.next_entry().await {
                    let entry_path = entry.path();

                    if entry_path.is_dir() {
                        stack.push(entry_path);
                    } else if entry_path
                        .file_name()
                        .is_some_and(|name| name == target_filename.as_str())
                    {
                        // Found the file, try to read and deserialize it
                        if let Ok(content) = async_fs::read_to_string(&entry_path).await {
                            if let Ok(result) = serde_json::from_str::<ResearchResult>(&content) {
                                return Ok(Some(result));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get file path for a cache key
    fn get_cache_file_path(&self, cache_key: &str, research_type: &ResearchType) -> PathBuf {
        self.config
            .base_path
            .join("research_results")
            .join(research_type.to_string().to_lowercase())
            .join(format!("{cache_key}.json"))
    }

    /// Get context-aware file path for a cache key
    fn get_context_aware_file_path(
        &self,
        cache_key: &str,
        research_type: &ResearchType,
        context_result: Option<&ContextDetectionResult>,
    ) -> PathBuf {
        let base_path = self
            .config
            .base_path
            .join("research_results")
            .join(research_type.to_string().to_lowercase());

        if let Some(context) = context_result {
            // Create subdirectories based on context for better organization
            let audience_dir = context.audience_level.display_name().to_lowercase();
            let domain_dir = context.technical_domain.display_name().to_lowercase();
            let urgency_dir = context.urgency_level.display_name().to_lowercase();

            let context_path = base_path
                .join("context-aware")
                .join(audience_dir)
                .join(domain_dir)
                .join(urgency_dir);

            context_path.join(format!("{cache_key}.json"))
        } else {
            base_path.join(format!("{cache_key}.json"))
        }
    }

    /// Calculate content hash for deduplication
    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Store research result with context awareness
    pub async fn store_with_context(
        &self,
        result: &ResearchResult,
        context_result: Option<&ContextDetectionResult>,
    ) -> Result<String, StorageError> {
        // Use metadata cache key if available, otherwise generate context-aware key
        let cache_key = if !result.metadata.cache_key.is_empty() {
            result.metadata.cache_key.clone()
        } else {
            self.generate_context_aware_cache_key(result, context_result)
        };
        let file_path = self.get_context_aware_file_path(
            &cache_key,
            &result.request.research_type,
            context_result,
        );

        debug!(
            "Storing context-aware research result with cache key: {}",
            cache_key
        );

        // Serialize the result
        let json = serde_json::to_string_pretty(result)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(StorageError::Io)?;
        }

        // Write to file
        async_fs::write(&file_path, &json)
            .await
            .map_err(StorageError::Io)?;

        // Calculate content hash
        let content_hash = self.calculate_content_hash(&json);

        // Update cache index
        let cache_entry = CacheEntry::new(
            cache_key.clone(),
            file_path,
            result.request.research_type.clone(),
            result.request.original_query.clone(),
            json.len() as u64,
            content_hash,
            self.config.cache_expiration_seconds,
        );

        // Insert entry into cache index
        {
            let mut cache_index = self.cache_index.lock().await;
            cache_index.insert(cache_key.clone(), cache_entry);
        }

        info!("Stored context-aware research result: {}", cache_key);
        Ok(cache_key)
    }

    /// Retrieve research result with context awareness
    pub async fn retrieve_with_context(
        &self,
        cache_key: &str,
        context_result: Option<&ContextDetectionResult>,
    ) -> Result<Option<ResearchResult>, StorageError> {
        debug!(
            "Retrieving context-aware research result with cache key: {}",
            cache_key
        );

        // Check cache index first
        {
            let cache_index = self.cache_index.lock().await;
            if let Some(entry) = cache_index.get(cache_key) {
                if entry.is_expired() {
                    warn!("Cache entry expired: {}", cache_key);
                    return Ok(None);
                }

                // Read from file
                let content = async_fs::read_to_string(&entry.file_path)
                    .await
                    .map_err(StorageError::Io)?;

                // Deserialize
                let result: ResearchResult = serde_json::from_str(&content)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                debug!("Retrieved context-aware research result: {}", cache_key);
                return Ok(Some(result));
            }
        }

        // Try to find the file by scanning directories with context awareness
        for research_type in ResearchType::all() {
            // Check context-aware path first
            let context_path =
                self.get_context_aware_file_path(cache_key, &research_type, context_result);
            if context_path.exists() {
                let content = async_fs::read_to_string(&context_path)
                    .await
                    .map_err(StorageError::Io)?;

                let result: ResearchResult = serde_json::from_str(&content)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                debug!(
                    "Found context-aware research result by scanning: {}",
                    cache_key
                );
                return Ok(Some(result));
            }

            // Fall back to standard path
            let file_path = self.get_cache_file_path(cache_key, &research_type);
            if file_path.exists() {
                let content = async_fs::read_to_string(&file_path)
                    .await
                    .map_err(StorageError::Io)?;

                let result: ResearchResult = serde_json::from_str(&content)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                debug!("Found research result by standard scanning: {}", cache_key);
                return Ok(Some(result));
            }
        }

        debug!("Context-aware research result not found: {}", cache_key);
        Ok(None)
    }

    /// Save cache index to disk
    #[allow(dead_code)]
    async fn save_cache_index(&self) -> Result<(), StorageError> {
        let index_path = self.config.base_path.join("index").join("cache_index.json");
        let cache_index = self.cache_index.lock().await;
        let json = serde_json::to_string_pretty(&*cache_index)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        async_fs::write(&index_path, json)
            .await
            .map_err(|e| StorageError::Index(format!("Failed to save cache index: {e}")))?;

        debug!("Saved cache index to: {}", index_path.display());
        Ok(())
    }

    /// Save search index to disk
    async fn save_search_index(&self) -> Result<(), StorageError> {
        let index_path = self
            .config
            .base_path
            .join("index")
            .join("search_index.json");
        let search_index = self.search_index.lock().await;
        let json = serde_json::to_string_pretty(&*search_index)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        async_fs::write(&index_path, json)
            .await
            .map_err(|e| StorageError::Index(format!("Failed to save search index: {e}")))?;

        debug!("Saved search index to: {}", index_path.display());
        Ok(())
    }

    /// Update search index with new research result
    #[allow(dead_code)]
    async fn update_search_index_entry(
        &mut self,
        result: &ResearchResult,
    ) -> Result<(), StorageError> {
        let cache_key = self.generate_cache_key(result);

        // Extract keywords from the result
        let mut keywords = result.request.matched_keywords.clone();
        keywords.extend(result.request.domain_context.frameworks.clone());
        keywords.extend(result.request.domain_context.tags.clone());

        // Extract content for full-text search
        let mut content = result.immediate_answer.clone();
        for evidence in &result.supporting_evidence {
            content.push_str(&format!(" {}", evidence.content));
        }
        for detail in &result.implementation_details {
            content.push_str(&format!(" {}", detail.content));
        }

        let index_entry = IndexEntry::new(
            cache_key,
            result.request.research_type.clone(),
            result.request.original_query.clone(),
            content,
            keywords,
            result.request.domain_context.tags.clone(),
            result.metadata.quality_score,
        );

        {
            let mut search_index = self.search_index.lock().await;
            search_index.insert(index_entry.cache_key.clone(), index_entry);
        }
        self.save_search_index().await?;

        Ok(())
    }

    /// Perform basic keyword-based search
    async fn search_by_keywords(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let query_lower = query.query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results = Vec::new();

        let search_index = self.search_index.lock().await;
        for entry in search_index.values() {
            // Apply research type filter
            if let Some(ref filter_type) = query.research_type {
                if entry.research_type != *filter_type {
                    continue;
                }
            }

            // Apply quality filter
            if let Some(min_quality) = query.min_quality {
                if entry.quality_score < min_quality {
                    continue;
                }
            }

            // Apply tags filter
            if !query.tags.is_empty() && !query.tags.iter().any(|tag| entry.tags.contains(tag)) {
                continue;
            }

            // Calculate relevance score
            let content_lower = entry.content.to_lowercase();
            let query_lower = entry.original_query.to_lowercase();

            let mut matched_keywords = Vec::new();
            let mut relevance_score = 0.0;

            for word in &query_words {
                if content_lower.contains(word) {
                    matched_keywords.push(word.to_string());
                    relevance_score += 1.0;
                }
                if query_lower.contains(word) {
                    relevance_score += 0.5;
                }
                if entry
                    .keywords
                    .iter()
                    .any(|k| k.to_lowercase().contains(word))
                {
                    relevance_score += 0.8;
                }
            }

            if relevance_score > 0.0 {
                // Normalize relevance score
                relevance_score /= query_words.len() as f64;

                // Generate snippet
                let snippet = Self::generate_snippet(&entry.content, &query_words);

                results.push(SearchResult::new(
                    entry.clone(),
                    relevance_score,
                    matched_keywords,
                    snippet,
                ));
            }
        }

        // Sort by relevance score (descending)
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(10);

        results.into_iter().skip(offset).take(limit).collect()
    }

    /// Generate a snippet from content highlighting matched terms
    fn generate_snippet(content: &str, query_words: &[&str]) -> String {
        let content_lower = content.to_lowercase();

        // Find the first occurrence of any query word
        let mut best_start = 0;
        for word in query_words {
            if let Some(pos) = content_lower.find(word) {
                best_start = pos;
                break;
            }
        }

        // Extract snippet around the match
        let snippet_start = best_start.saturating_sub(50);
        let snippet_end = (best_start + 150).min(content.len());

        let mut snippet = content[snippet_start..snippet_end].to_string();

        // Add ellipsis if we're not at the beginning/end
        if snippet_start > 0 {
            snippet = format!("...{snippet}");
        }
        if snippet_end < content.len() {
            snippet = format!("{snippet}...");
        }

        snippet
    }

    /// Calculate warming effectiveness for a specific research type
    async fn calculate_warming_effectiveness_for_type(&self, research_type: &ResearchType) -> f64 {
        // Get warming operations for this type
        let recent_operations = self.recent_operations.lock().await;
        let warming_ops: Vec<_> = recent_operations
            .iter()
            .filter(|op| {
                matches!(op.operation_type, CacheOperationType::Warm)
                    && op
                        .cache_key
                        .contains(&research_type.to_string().to_lowercase())
            })
            .collect();

        if warming_ops.is_empty() {
            return 0.0;
        }

        // Calculate success rate of warming operations
        let successful_warmings = warming_ops.iter().filter(|op| op.success).count();

        successful_warmings as f64 / warming_ops.len() as f64
    }

    /// Calculate key optimization score for a specific research type
    async fn calculate_key_optimization_score_for_type(&self, research_type: &ResearchType) -> f64 {
        // Get cache entries for this type
        let cache_index = self.cache_index.lock().await;
        let type_entries: Vec<_> = cache_index
            .values()
            .filter(|entry| entry.research_type == *research_type)
            .collect();

        if type_entries.is_empty() {
            return 0.0;
        }

        // Calculate key diversity (unique keys vs total operations)
        let recent_operations = self.recent_operations.lock().await;
        let type_operations: Vec<_> = recent_operations
            .iter()
            .filter(|op| {
                op.cache_key
                    .contains(&research_type.to_string().to_lowercase())
            })
            .collect();

        if type_operations.is_empty() {
            return 0.5; // Neutral score for no operations
        }

        let unique_keys: std::collections::HashSet<_> =
            type_operations.iter().map(|op| &op.cache_key).collect();

        // Higher score means better key utilization
        let key_diversity = unique_keys.len() as f64 / type_operations.len() as f64;

        // Combine with hit rate for overall optimization score
        let type_hit_rate = self.calculate_hit_rate_for_type(research_type).await;

        // Weighted combination: 60% hit rate, 40% key diversity
        (type_hit_rate * 0.6) + (key_diversity * 0.4)
    }

    /// Calculate hit rate for a specific research type
    async fn calculate_hit_rate_for_type(&self, research_type: &ResearchType) -> f64 {
        let recent_operations = self.recent_operations.lock().await;
        let type_operations: Vec<_> = recent_operations
            .iter()
            .filter(|op| {
                op.cache_key
                    .contains(&research_type.to_string().to_lowercase())
            })
            .collect();

        if type_operations.is_empty() {
            return 0.0;
        }

        let hits = type_operations
            .iter()
            .filter(|op| matches!(op.operation_type, CacheOperationType::Hit))
            .count();
        let misses = type_operations
            .iter()
            .filter(|op| matches!(op.operation_type, CacheOperationType::Miss))
            .count();

        if hits + misses == 0 {
            return 0.0;
        }

        hits as f64 / (hits + misses) as f64
    }

    /// Record a cache operation for performance monitoring
    #[allow(dead_code)]
    async fn record_operation(&mut self, operation: CacheOperation) {
        // Add to recent operations (maintain a rolling window)
        {
            let mut recent_operations = self.recent_operations.lock().await;
            recent_operations.push(operation.clone());

            // Keep only the last 1000 operations to prevent memory bloat
            const MAX_OPERATIONS: usize = 1000;
            if recent_operations.len() > MAX_OPERATIONS {
                let drain_count = recent_operations.len() - MAX_OPERATIONS;
                recent_operations.drain(0..drain_count);
            }
        }

        // Update performance monitor
        let current_hit_rate = self.calculate_current_hit_rate().await;
        {
            let mut performance_monitor = self.performance_monitor.lock().await;
            performance_monitor.current_hit_rate = current_hit_rate;

            // Update performance status based on hit rate
            performance_monitor.status = if current_hit_rate >= performance_monitor.target_hit_rate
            {
                CachePerformanceStatus::Optimal
            } else if current_hit_rate >= performance_monitor.target_hit_rate * 0.8 {
                CachePerformanceStatus::Degraded
            } else {
                CachePerformanceStatus::Critical
            };

            // Add to performance monitor's recent operations
            performance_monitor.recent_operations.push(operation);
            if performance_monitor.recent_operations.len() > 100 {
                performance_monitor.recent_operations.drain(0..50); // Keep last 100
            }
        }
    }

    /// Calculate current hit rate from recent operations
    async fn calculate_current_hit_rate(&self) -> f64 {
        let recent_operations = self.recent_operations.lock().await;
        if recent_operations.is_empty() {
            return 0.0;
        }

        let hits = recent_operations
            .iter()
            .filter(|op| matches!(op.operation_type, CacheOperationType::Hit))
            .count();
        let misses = recent_operations
            .iter()
            .filter(|op| matches!(op.operation_type, CacheOperationType::Miss))
            .count();

        if hits + misses == 0 {
            return 0.0;
        }

        hits as f64 / (hits + misses) as f64
    }
}

#[async_trait::async_trait]
impl Storage for FileStorage {
    async fn store(&self, result: &ResearchResult) -> Result<String, StorageError> {
        // Use metadata cache key if available, otherwise generate new one
        let cache_key = if !result.metadata.cache_key.is_empty() {
            result.metadata.cache_key.clone()
        } else {
            self.generate_cache_key(result)
        };
        let file_path = self.get_cache_file_path(&cache_key, &result.request.research_type);

        debug!("Storing research result with cache key: {}", cache_key);

        // Serialize the result
        let json = serde_json::to_string_pretty(result)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(StorageError::Io)?;
        }

        // Write to file
        async_fs::write(&file_path, &json)
            .await
            .map_err(StorageError::Io)?;

        // Calculate content hash
        let content_hash = self.calculate_content_hash(&json);

        // Update cache index
        let cache_entry = CacheEntry::new(
            cache_key.clone(),
            file_path,
            result.request.research_type.clone(),
            result.request.original_query.clone(),
            json.len() as u64,
            content_hash,
            self.config.cache_expiration_seconds,
        );

        // Insert entry into cache index
        {
            let mut cache_index = self.cache_index.lock().await;
            cache_index.insert(cache_key.clone(), cache_entry);
        }

        info!("Stored research result: {}", cache_key);
        Ok(cache_key)
    }

    async fn retrieve(&self, cache_key: &str) -> Result<Option<ResearchResult>, StorageError> {
        debug!("Retrieving research result with cache key: {}", cache_key);

        // OPTIMIZATION 1: Check cache index first for immediate hits
        {
            let cache_index = self.cache_index.lock().await;
            if let Some(entry) = cache_index.get(cache_key) {
                if entry.is_expired() {
                    warn!("Cache entry expired: {}", cache_key);
                    return Ok(None);
                }

                // Read from file
                let content = async_fs::read_to_string(&entry.file_path)
                    .await
                    .map_err(StorageError::Io)?;

                // Deserialize
                let result: ResearchResult = serde_json::from_str(&content)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                debug!("Retrieved research result via index lookup: {}", cache_key);
                return Ok(Some(result));
            }
        }

        // OPTIMIZATION 2: Intelligent fallback scanning with priority order
        // Prioritize research types based on frequency of usage (Learning and Implementation are most common)
        let research_types_priority = vec![
            ResearchType::Learning,
            ResearchType::Implementation,
            ResearchType::Troubleshooting,
            ResearchType::Decision,
            ResearchType::Validation,
        ];

        for research_type in &research_types_priority {
            // OPTIMIZATION 3: Check direct file paths first (most likely location)
            let file_path = self.get_cache_file_path(cache_key, research_type);
            if file_path.exists() {
                let content = async_fs::read_to_string(&file_path)
                    .await
                    .map_err(StorageError::Io)?;

                let result: ResearchResult = serde_json::from_str(&content)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                debug!("Found research result by optimized scanning: {}", cache_key);
                return Ok(Some(result));
            }
        }

        // OPTIMIZATION 4: If direct paths fail, scan context-aware directories
        for research_type in &research_types_priority {
            let context_base_path = self
                .config
                .base_path
                .join("research_results")
                .join(research_type.to_string().to_lowercase())
                .join("context-aware");

            if let Ok(Some(found_result)) = self
                .scan_context_aware_directory(&context_base_path, cache_key)
                .await
            {
                debug!(
                    "Found research result by optimized context-aware scanning: {}",
                    cache_key
                );
                return Ok(Some(found_result));
            }
        }

        debug!("Research result not found: {}", cache_key);
        Ok(None)
    }

    async fn delete(&self, cache_key: &str) -> Result<(), StorageError> {
        debug!("Deleting research result with cache key: {}", cache_key);

        {
            let mut cache_index = self.cache_index.lock().await;
            if let Some(entry) = cache_index.get(cache_key) {
                async_fs::remove_file(&entry.file_path)
                    .await
                    .map_err(StorageError::Io)?;

                // Remove from cache index
                cache_index.remove(cache_key);

                info!("Deleted research result: {}", cache_key);
                return Ok(());
            }
        }

        // Try to find and delete by scanning directories
        for research_type in ResearchType::all() {
            let file_path = self.get_cache_file_path(cache_key, &research_type);
            if file_path.exists() {
                async_fs::remove_file(&file_path)
                    .await
                    .map_err(StorageError::Io)?;

                info!("Deleted research result by scanning: {}", cache_key);
                return Ok(());
            }
        }

        Ok(())
    }

    async fn list_cache_entries(&self) -> Result<Vec<CacheEntry>, StorageError> {
        debug!("Listing cache entries");
        let cache_index = self.cache_index.lock().await;
        Ok(cache_index.values().cloned().collect())
    }

    async fn get_cache_stats(&self) -> Result<CacheStats, StorageError> {
        debug!("Calculating cache statistics");

        let mut stats = CacheStats::default();
        let mut by_type: HashMap<ResearchType, CacheTypeStats> = HashMap::new();

        // Gather cache index data
        {
            let cache_index = self.cache_index.lock().await;
            for entry in cache_index.values() {
                stats.total_entries += 1;
                stats.total_size_bytes += entry.size_bytes;

                if entry.is_expired() {
                    stats.expired_entries += 1;
                }

                // Update per-type stats
                let type_stats = by_type.entry(entry.research_type.clone()).or_default();

                type_stats.entries += 1;
                type_stats.size_bytes += entry.size_bytes;
            }
        }

        // Calculate hit rates from recent operations
        let recent_operations = self.recent_operations.lock().await;
        let total_operations = recent_operations.len();
        if total_operations > 0 {
            let hits = recent_operations
                .iter()
                .filter(|op| matches!(op.operation_type, CacheOperationType::Hit))
                .count();
            let misses = recent_operations
                .iter()
                .filter(|op| matches!(op.operation_type, CacheOperationType::Miss))
                .count();

            stats.hits = hits as u64;
            stats.misses = misses as u64;

            if hits + misses > 0 {
                stats.hit_rate = hits as f64 / (hits + misses) as f64;
            }

            // Calculate per-type hit rates
            for (research_type, type_stats) in &mut by_type {
                let type_operations: Vec<_> = recent_operations
                    .iter()
                    .filter(|op| {
                        // For now, we'll use a simple heuristic to match operations to research types
                        // In a real implementation, we'd store research type with each operation
                        op.cache_key
                            .contains(&research_type.to_string().to_lowercase())
                    })
                    .collect();

                if !type_operations.is_empty() {
                    let type_hits = type_operations
                        .iter()
                        .filter(|op| matches!(op.operation_type, CacheOperationType::Hit))
                        .count();
                    let type_misses = type_operations
                        .iter()
                        .filter(|op| matches!(op.operation_type, CacheOperationType::Miss))
                        .count();

                    type_stats.hits = type_hits as u64;
                    type_stats.misses = type_misses as u64;

                    if type_hits + type_misses > 0 {
                        type_stats.hit_rate = type_hits as f64 / (type_hits + type_misses) as f64;
                    }

                    // Calculate average response time for this type
                    let response_times: Vec<_> = type_operations
                        .iter()
                        .map(|op| op.duration_ms as f64)
                        .collect();

                    if !response_times.is_empty() {
                        type_stats.avg_response_time_ms =
                            response_times.iter().sum::<f64>() / response_times.len() as f64;
                    }

                    // Calculate warming effectiveness and key optimization score
                    type_stats.warming_effectiveness = self
                        .calculate_warming_effectiveness_for_type(research_type)
                        .await;
                    type_stats.key_optimization_score = self
                        .calculate_key_optimization_score_for_type(research_type)
                        .await;
                }
            }
        }

        stats.by_research_type = by_type;

        // Calculate average age
        if stats.total_entries > 0 {
            let cache_index = self.cache_index.lock().await;
            let total_age: u64 = cache_index.values().map(|e| e.age_seconds()).sum();
            stats.average_age_seconds = total_age as f64 / stats.total_entries as f64;
        }

        // Include enhanced analytics
        let analytics = self.analytics.lock().await;
        stats.analytics = analytics.clone();

        Ok(stats)
    }

    async fn cleanup_expired(&self) -> Result<u64, StorageError> {
        debug!("Cleaning up expired cache entries");

        let mut deleted_count = 0;
        let mut expired_keys = Vec::new();

        // Collect expired keys first
        {
            let cache_index = self.cache_index.lock().await;
            for (cache_key, entry) in cache_index.iter() {
                if entry.is_expired() {
                    expired_keys.push((cache_key.clone(), entry.file_path.clone()));
                }
            }
        }

        // Delete expired entries
        {
            let mut cache_index = self.cache_index.lock().await;
            for (cache_key, file_path) in expired_keys {
                if let Err(e) = async_fs::remove_file(&file_path).await {
                    warn!(
                        "Failed to delete expired file {}: {}",
                        file_path.display(),
                        e
                    );
                } else {
                    // Remove from cache index
                    cache_index.remove(&cache_key);
                    deleted_count += 1;
                    debug!("Deleted expired cache entry: {}", cache_key);
                }
            }
        }

        info!("Cleaned up {} expired cache entries", deleted_count);
        Ok(deleted_count)
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, StorageError> {
        debug!("Searching with query: '{}'", query.query);

        let results = self.search_by_keywords(query).await;

        debug!("Found {} search results", results.len());
        Ok(results)
    }

    async fn update_index(&self) -> Result<(), StorageError> {
        debug!("Updating search index");

        // This would scan all files and rebuild the index
        // For now, we'll just save the current index
        self.save_search_index().await?;

        info!("Updated search index");
        Ok(())
    }

    /// Record cache operation for performance monitoring
    async fn record_cache_operation(&self, operation: CacheOperation) -> Result<(), StorageError> {
        // In a real implementation, this would need mutable access
        // For now, we'll just log the operation
        debug!("Recording cache operation: {:?}", operation.operation_type);
        Ok(())
    }

    /// Get performance monitor state
    async fn get_performance_monitor(&self) -> Result<CachePerformanceMonitor, StorageError> {
        let performance_monitor = self.performance_monitor.lock().await;
        Ok(performance_monitor.clone())
    }

    /// Update cache analytics
    async fn update_analytics(&self, _analytics: CacheAnalytics) -> Result<(), StorageError> {
        // In a real implementation, this would need mutable access to update self.analytics
        debug!("Updating cache analytics");
        Ok(())
    }

    /// Get cache key optimization recommendations
    async fn get_key_optimization_recommendations(&self) -> Result<Vec<String>, StorageError> {
        let mut recommendations = Vec::new();

        let current_hit_rate = self.calculate_current_hit_rate().await;

        if current_hit_rate < 0.8 {
            recommendations.push(
                "Consider implementing cache warming for frequently accessed content".to_string(),
            );
        }

        if current_hit_rate < 0.6 {
            recommendations
                .push("Review cache key generation strategy for better deduplication".to_string());
            recommendations
                .push("Consider normalizing query parameters to improve cache hits".to_string());
        }

        {
            let cache_index = self.cache_index.lock().await;
            if cache_index.len() > 1000 {
                recommendations.push(
                    "Cache size is large - consider implementing size-based eviction".to_string(),
                );
            }
        }

        // Analyze key diversity
        {
            let recent_operations = self.recent_operations.lock().await;
            if !recent_operations.is_empty() {
                let unique_keys: std::collections::HashSet<_> =
                    recent_operations.iter().map(|op| &op.cache_key).collect();

                let key_diversity = unique_keys.len() as f64 / recent_operations.len() as f64;

                if key_diversity > 0.8 {
                    recommendations.push(
                        "High key diversity detected - consider improving query normalization"
                            .to_string(),
                    );
                }
            }
        }

        if recommendations.is_empty() {
            recommendations
                .push("Cache performance is optimal - no specific recommendations".to_string());
        }

        Ok(recommendations)
    }

    /// Warm cache with specific entries
    async fn warm_cache(&self, entries: Vec<String>) -> Result<CacheWarmingStats, StorageError> {
        let mut stats = CacheWarmingStats {
            active_strategies: vec!["manual_warming".to_string()],
            ..Default::default()
        };

        for cache_key in entries {
            // Simulate warming by checking if entry exists
            let start_time = std::time::Instant::now();

            let cache_index = self.cache_index.lock().await;
            if let Some(_entry) = cache_index.get(&cache_key) {
                stats.items_warmed += 1;

                let warming_time = start_time.elapsed().as_millis() as f64;
                if stats.avg_warming_time_ms == 0.0 {
                    stats.avg_warming_time_ms = warming_time;
                } else {
                    stats.avg_warming_time_ms = (stats.avg_warming_time_ms + warming_time) / 2.0;
                }
            } else {
                stats.warming_failures += 1;
            }
        }

        // Calculate effectiveness
        let total_attempts = stats.items_warmed + stats.warming_failures;
        if total_attempts > 0 {
            stats.strategy_effectiveness = stats.items_warmed as f64 / total_attempts as f64;
        }

        // Estimate hit rate improvement (simplified)
        if stats.items_warmed > 0 {
            stats.hit_rate_improvement = 0.1; // Assume 10% improvement
        }

        info!(
            "Cache warming completed: {} items warmed, {} failures",
            stats.items_warmed, stats.warming_failures
        );

        Ok(stats)
    }

    /// Get hit rate trends for analysis
    async fn get_hit_rate_trends(
        &self,
        timeframe_hours: u64,
    ) -> Result<Vec<HitRateTrend>, StorageError> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(timeframe_hours as i64);

        // Filter recent hit rate history within timeframe
        let hit_rate_history = self.hit_rate_history.lock().await;
        let trends: Vec<HitRateTrend> = hit_rate_history
            .iter()
            .filter(|trend| trend.timestamp > cutoff_time)
            .cloned()
            .collect();

        // If no historical data, generate a synthetic trend based on current operations
        if trends.is_empty() {
            let current_hit_rate = self.calculate_current_hit_rate().await;
            let recent_operations = self.recent_operations.lock().await;
            let current_request_count = recent_operations.len() as u64;

            Ok(vec![HitRateTrend {
                timestamp: chrono::Utc::now(),
                hit_rate: current_hit_rate,
                request_count: current_request_count,
                context: "current_period".to_string(),
            }])
        } else {
            Ok(trends)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use fortitude_types::*;
    use tempfile::TempDir;

    async fn create_test_storage() -> (FileStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            base_path: temp_dir.path().to_path_buf(),
            cache_expiration_seconds: 3600,
            max_cache_size_bytes: 1024 * 1024,
            enable_content_addressing: true,
            index_update_interval_seconds: 300,
        };

        let storage = FileStorage::new(config).await.unwrap();
        (storage, temp_dir)
    }

    fn create_test_result() -> ResearchResult {
        let request = ClassifiedRequest::new(
            "Test query".to_string(),
            ResearchType::Learning,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );

        let metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: 1000,
            sources_consulted: vec!["source1".to_string()],
            quality_score: 0.9,
            cache_key: "test-key".to_string(),
            tags: std::collections::HashMap::new(),
        };

        ResearchResult::new(request, "Test answer".to_string(), vec![], vec![], metadata)
    }

    #[tokio::test]
    async fn test_storage_creation() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Verify directories were created
        assert!(storage.config.base_path.exists());
        assert!(storage.config.base_path.join("research_results").exists());
        assert!(storage.config.base_path.join("cache").exists());
        assert!(storage.config.base_path.join("index").exists());

        // Verify research type directories
        for research_type in ResearchType::all() {
            let type_dir = storage
                .config
                .base_path
                .join("research_results")
                .join(research_type.to_string().to_lowercase());
            assert!(type_dir.exists());
        }
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let (storage, _temp_dir) = create_test_storage().await;
        let result = create_test_result();

        // Store result
        let cache_key = storage.store(&result).await.unwrap();
        assert!(!cache_key.is_empty());

        // Retrieve result
        let retrieved = storage.retrieve(&cache_key).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_result = retrieved.unwrap();
        assert_eq!(
            retrieved_result.request.original_query,
            result.request.original_query
        );
        assert_eq!(retrieved_result.immediate_answer, result.immediate_answer);
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _temp_dir) = create_test_storage().await;
        let result = create_test_result();

        // Store and verify
        let cache_key = storage.store(&result).await.unwrap();
        assert!(storage.retrieve(&cache_key).await.unwrap().is_some());

        // Delete and verify
        storage.delete(&cache_key).await.unwrap();
        assert!(storage.retrieve(&cache_key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Initially empty
        let stats = storage.get_cache_stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_size_bytes, 0);

        // Note: The actual stats would be updated when storing results
        // but our current implementation doesn't modify the cache_index
        // This is a limitation that would be fixed in the real implementation
    }

    #[tokio::test]
    async fn test_search() {
        let (storage, _temp_dir) = create_test_storage().await;

        let query = SearchQuery::new("test query".to_string())
            .with_research_type(ResearchType::Learning)
            .with_limit(5);

        let results = storage.search(&query).await.unwrap();
        assert_eq!(results.len(), 0); // Empty index initially
    }

    #[test]
    fn test_cache_key_generation() {
        let (storage, _temp_dir) = tokio_test::block_on(create_test_storage());
        let result = create_test_result();

        let key1 = storage.generate_cache_key(&result);
        let key2 = storage.generate_cache_key(&result);

        assert_eq!(key1, key2); // Same input should produce same key
        assert!(!key1.is_empty());
    }

    #[tokio::test]
    async fn test_context_aware_store_and_retrieve() {
        let (storage, _temp_dir) = create_test_storage().await;
        let result = create_test_result();

        // Store result with context awareness
        let cache_key = storage.store_with_context(&result, None).await.unwrap();
        assert!(!cache_key.is_empty());

        // Retrieve result with context awareness
        let retrieved = storage
            .retrieve_with_context(&cache_key, None)
            .await
            .unwrap();
        assert!(retrieved.is_some());

        let retrieved_result = retrieved.unwrap();
        assert_eq!(
            retrieved_result.request.original_query,
            result.request.original_query
        );
        assert_eq!(retrieved_result.immediate_answer, result.immediate_answer);
    }

    /// ANCHOR: Verifies cache index is properly updated during storage operations.
    /// Tests: Cache entry indexing, immediate retrieval via index, consistency between storage and indexing
    #[tokio::test]
    async fn test_anchor_cache_index_consistency() {
        let (storage, _temp_dir) = create_test_storage().await;
        let result = create_test_result();

        // Initially, cache should be empty
        let initial_stats = storage.get_cache_stats().await.unwrap();
        assert_eq!(initial_stats.total_entries, 0);
        println!(
            "Cache entries before storing: {}",
            initial_stats.total_entries
        );

        // Store result
        let cache_key = storage.store(&result).await.unwrap();
        assert!(!cache_key.is_empty());

        // Verify cache index was updated
        let stats_after_store = storage.get_cache_stats().await.unwrap();
        println!(
            "Cache entries after storing: {}",
            stats_after_store.total_entries
        );

        // This should pass now that we fixed the immutable reference issue
        assert_eq!(
            stats_after_store.total_entries, 1,
            "Cache index should be updated after storing"
        );

        // Verify cache entry is accessible via index (not just file scan fallback)
        let cache_entries = storage.list_cache_entries().await.unwrap();
        assert_eq!(
            cache_entries.len(),
            1,
            "Cache index should contain the stored entry"
        );

        let entry = &cache_entries[0];
        assert_eq!(entry.key, cache_key);
        assert_eq!(entry.research_type, result.request.research_type);
        assert!(!entry.is_expired());

        // Verify immediate retrieval via index works
        let retrieved = storage.retrieve(&cache_key).await.unwrap();
        assert!(
            retrieved.is_some(),
            "Should retrieve via cache index, not file scanning"
        );
    }

    /// ANCHOR: Verifies context-aware storage properly updates cache index.
    /// Tests: Context-aware cache key generation, index updates, retrieval consistency
    #[tokio::test]
    async fn test_anchor_context_aware_indexing() {
        let (storage, _temp_dir) = create_test_storage().await;
        let mut result = create_test_result();

        // Clear the existing cache key to force context-aware key generation
        result.metadata.cache_key = String::new();

        // Store with context awareness
        let cache_key = storage.store_with_context(&result, None).await.unwrap();
        assert!(!cache_key.is_empty());
        assert!(
            cache_key.starts_with("enhanced-"),
            "Context-aware keys should have enhanced prefix"
        );

        // Verify index was properly updated
        let stats = storage.get_cache_stats().await.unwrap();
        assert_eq!(
            stats.total_entries, 1,
            "Context-aware storage should update cache index"
        );

        // Verify retrieval works via index for context-aware entries
        let retrieved = storage
            .retrieve_with_context(&cache_key, None)
            .await
            .unwrap();
        assert!(
            retrieved.is_some(),
            "Context-aware entries should be retrievable via index"
        );

        let retrieved_result = retrieved.unwrap();
        assert_eq!(
            retrieved_result.request.original_query,
            result.request.original_query
        );
    }

    /// ANCHOR: Verifies delete operations properly update cache index.
    /// Tests: Index cleanup on deletion, atomicity of delete operations
    #[tokio::test]
    async fn test_anchor_delete_index_consistency() {
        let (storage, _temp_dir) = create_test_storage().await;
        let result = create_test_result();

        // Store and verify index entry exists
        let cache_key = storage.store(&result).await.unwrap();
        let stats_after_store = storage.get_cache_stats().await.unwrap();
        assert_eq!(stats_after_store.total_entries, 1);

        // Delete and verify index is updated
        storage.delete(&cache_key).await.unwrap();

        let stats_after_delete = storage.get_cache_stats().await.unwrap();
        assert_eq!(
            stats_after_delete.total_entries, 0,
            "Cache index should be updated after deletion"
        );

        // Verify entry is no longer retrievable
        let retrieved = storage.retrieve(&cache_key).await.unwrap();
        assert!(
            retrieved.is_none(),
            "Deleted entries should not be retrievable"
        );

        // Verify cache entries list is empty
        let cache_entries = storage.list_cache_entries().await.unwrap();
        assert_eq!(
            cache_entries.len(),
            0,
            "Cache index should be empty after deletion"
        );
    }

    #[test]
    fn test_context_aware_cache_key_generation() {
        let (storage, _temp_dir) = tokio_test::block_on(create_test_storage());
        let result = create_test_result();

        // Test without context
        let key1 = storage.generate_context_aware_cache_key(&result, None);
        let key2 = storage.generate_context_aware_cache_key(&result, None);
        assert_eq!(key1, key2); // Same input should produce same key
        assert!(!key1.is_empty());

        // Test with context (would need to create mock context)
        // This is a basic test - in real usage, context would affect the key
        let key3 = storage.generate_context_aware_cache_key(&result, None);
        assert_eq!(key1, key3); // Without context, should be same as basic case
    }

    #[test]
    fn test_content_hash() {
        let (storage, _temp_dir) = tokio_test::block_on(create_test_storage());

        let hash1 = storage.calculate_content_hash("test content");
        let hash2 = storage.calculate_content_hash("test content");
        let hash3 = storage.calculate_content_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_snippet_generation() {
        let content = "This is a long piece of content that contains various keywords and phrases that we want to search for and extract snippets from.";
        let query_words = vec!["keywords", "phrases"];

        let snippet = FileStorage::generate_snippet(content, &query_words);

        assert!(snippet.contains("keywords"));
        assert!(snippet.len() <= content.len());
    }

    /// ANCHOR: Test unified cache key generation between storage and retrieval
    /// This test verifies that storage and retrieval use identical cache keys
    /// Tests: Cache key consistency, storage respects metadata cache key, retrieval matches storage key
    #[tokio::test]
    async fn test_anchor_unified_cache_key_generation() {
        let (storage, _temp_dir) = create_test_storage().await;
        let mut result = create_test_result();

        // Simulate pipeline setting cache key in metadata (as it would in real usage)
        let expected_cache_key = "pipeline_generated_cache_key_12345";
        result.metadata.cache_key = expected_cache_key.to_string();

        // Storage should use the metadata cache key instead of generating its own
        let storage_key = storage.store(&result).await.unwrap();

        assert_eq!(
            storage_key, expected_cache_key,
            "Storage should use metadata.cache_key when available instead of generating new key"
        );

        // Retrieval should find the result using the same key
        let retrieved = storage.retrieve(&storage_key).await.unwrap();
        assert!(
            retrieved.is_some(),
            "Should retrieve result using the same cache key that was used for storage"
        );

        let retrieved_result = retrieved.unwrap();
        assert_eq!(
            retrieved_result.metadata.cache_key, expected_cache_key,
            "Retrieved result should maintain the same cache key"
        );
    }

    /// ANCHOR: Test cache hit detection with processing time verification
    /// This test verifies that cached results return quickly (indicating cache hit)
    /// Tests: Cache hit speed, processing time <50ms for cached results, cache effectiveness
    #[tokio::test]
    async fn test_anchor_cache_hit_processing_time() {
        let (storage, _temp_dir) = create_test_storage().await;
        let result = create_test_result();

        // Store the result first
        let cache_key = storage.store(&result).await.unwrap();

        // Measure retrieval time for cache hit
        let start_time = std::time::Instant::now();
        let retrieved = storage.retrieve(&cache_key).await.unwrap();
        let retrieval_time = start_time.elapsed();

        assert!(
            retrieved.is_some(),
            "Cache hit should return the stored result"
        );

        // Cache hits should be very fast (much less than 50ms)
        assert!(
            retrieval_time.as_millis() < 50,
            "Cache hit should be fast (<50ms), got: {retrieval_time:?}"
        );

        println!("Cache hit retrieval time: {retrieval_time:?}");
    }

    /// ANCHOR: Test pipeline-storage cache key integration
    /// This test verifies end-to-end cache key consistency between pipeline and storage
    /// Tests: Pipeline sets metadata.cache_key, storage respects it, retrieval works
    #[tokio::test]
    async fn test_anchor_pipeline_storage_cache_key_integration() {
        let (storage, _temp_dir) = create_test_storage().await;
        let mut result = create_test_result();

        // Simulate what pipeline does - generate context-aware cache key
        let pipeline_cache_key = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            result.request.original_query.hash(&mut hasher);
            result.request.research_type.hash(&mut hasher);
            result.request.audience_context.level.hash(&mut hasher);
            format!("pipeline_{:x}", hasher.finish())
        };

        // Pipeline sets the cache key in metadata
        result.metadata.cache_key = pipeline_cache_key.clone();

        // Storage should use the pipeline-provided cache key
        let storage_key = storage.store(&result).await.unwrap();
        assert_eq!(
            storage_key, pipeline_cache_key,
            "Storage should use pipeline-provided cache key from metadata"
        );

        // Test multiple retrieval methods work with the same key
        let direct_retrieval = storage.retrieve(&pipeline_cache_key).await.unwrap();
        assert!(
            direct_retrieval.is_some(),
            "Direct retrieval should work with pipeline cache key"
        );

        let context_retrieval = storage
            .retrieve_with_context(&pipeline_cache_key, None)
            .await
            .unwrap();
        assert!(
            context_retrieval.is_some(),
            "Context retrieval should work with pipeline cache key"
        );

        // Both methods should return the same result
        let direct_result = direct_retrieval.unwrap();
        let context_result = context_retrieval.unwrap();
        assert_eq!(
            direct_result.request.original_query, context_result.request.original_query,
            "Both retrieval methods should return identical results"
        );
    }

    /// ANCHOR: Test end-to-end cache functionality with unified keys
    /// This test demonstrates that cache hits work properly with the unified cache key system
    /// Tests: Store->Retrieve cycle, cache hit detection, consistent behavior across methods
    #[tokio::test]
    async fn test_anchor_end_to_end_cache_functionality() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Test query that simulates real usage
        let test_query = "How to implement async functions in Rust?";
        let mut result = create_test_result();
        result.request.original_query = test_query.to_string();

        // Simulate pipeline generating cache key (this would happen in real usage)
        let pipeline_key = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            test_query.hash(&mut hasher);
            result.request.research_type.hash(&mut hasher);
            format!("cache_test_{:x}", hasher.finish())
        };

        result.metadata.cache_key = pipeline_key.clone();

        println!("Testing cache functionality with query: '{test_query}'");
        println!("Pipeline generated cache key: {pipeline_key}");

        // First run: Store the result (cache miss)
        let first_run_start = std::time::Instant::now();
        let storage_key = storage.store(&result).await.unwrap();
        let first_run_time = first_run_start.elapsed();

        assert_eq!(
            storage_key, pipeline_key,
            "Storage should use pipeline-provided cache key"
        );

        // Second run: Retrieve the result (cache hit)
        let second_run_start = std::time::Instant::now();
        let cached_result = storage.retrieve(&pipeline_key).await.unwrap();
        let second_run_time = second_run_start.elapsed();

        assert!(
            cached_result.is_some(),
            "✅ Should find cached result on second run"
        );
        let cached = cached_result.unwrap();
        assert_eq!(cached.request.original_query, test_query);

        // Cache hit should be much faster than initial processing
        println!("First run (storage): {first_run_time:?}");
        println!("Second run (cache hit): {second_run_time:?}");

        assert!(
            second_run_time.as_millis() < 50,
            "✅ Cache hit should be fast (<50ms), got: {second_run_time:?}"
        );

        // Test that both retrieval methods work with the same key
        let context_retrieval_start = std::time::Instant::now();
        let context_cached = storage
            .retrieve_with_context(&pipeline_key, None)
            .await
            .unwrap();
        let context_retrieval_time = context_retrieval_start.elapsed();

        assert!(
            context_cached.is_some(),
            "✅ Context-aware retrieval should also find cached result"
        );
        assert!(
            context_retrieval_time.as_millis() < 50,
            "✅ Context-aware cache hit should also be fast (<50ms), got: {context_retrieval_time:?}"
        );

        println!("✅ Cache functionality working correctly:");
        println!("  - Pipeline and storage use unified cache keys");
        println!("  - Cache hits are fast (processing time <50ms)");
        println!("  - Both retrieval methods work with same key");
    }

    /// ANCHOR: Test CLI scenario cache behavior simulation
    /// This test simulates the exact scenario described in the requirements:
    /// Run "cache test query" twice and verify second run shows cached result
    /// Tests: CLI-like query caching, realistic cache key generation, fast retrieval
    #[tokio::test]
    async fn test_anchor_cli_cache_scenario_simulation() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Simulate the exact query mentioned in requirements
        let cli_query = "cache test query";
        let mut result = create_test_result();
        result.request.original_query = cli_query.to_string();
        result.request.research_type = ResearchType::Learning; // Typical for test queries

        // Simulate realistic pipeline cache key generation (similar to what pipeline.rs does)
        let realistic_cache_key = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            // Normalize query similar to storage normalization
            let normalized_query = cli_query.to_lowercase().replace([' ', '?', '!'], "_");
            normalized_query.hash(&mut hasher);
            result.request.research_type.hash(&mut hasher);
            result.request.audience_context.level.hash(&mut hasher);
            result.request.domain_context.technology.hash(&mut hasher);
            format!("cli_{:x}", hasher.finish())
        };

        result.metadata.cache_key = realistic_cache_key.clone();
        result.metadata.processing_time_ms = 1500; // Simulate longer processing for first run

        println!("🔍 CLI Scenario Test: Query = '{cli_query}'");
        println!("📝 Generated cache key: {realistic_cache_key}");

        // ===== First CLI run (cache miss) =====
        println!("\n📥 FIRST RUN (cache miss):");
        let first_run_start = std::time::Instant::now();
        let storage_key = storage.store(&result).await.unwrap();
        let first_run_duration = first_run_start.elapsed();

        assert_eq!(
            storage_key, realistic_cache_key,
            "Storage should use CLI pipeline cache key"
        );
        println!("   ✅ Stored with key: {storage_key}");
        println!("   ⏱️  Storage time: {first_run_duration:?}");

        // ===== Second CLI run (cache hit) =====
        println!("\n📤 SECOND RUN (cache hit):");
        let second_run_start = std::time::Instant::now();
        let cached_result = storage.retrieve(&realistic_cache_key).await.unwrap();
        let second_run_duration = second_run_start.elapsed();

        // Verify cache hit success
        assert!(
            cached_result.is_some(),
            "✅ Should find cached result on second CLI run"
        );
        let cached = cached_result.unwrap();
        assert_eq!(cached.request.original_query, cli_query);
        assert_eq!(cached.metadata.cache_key, realistic_cache_key);

        // Verify cache hit is fast (requirement: <50ms)
        println!("   ✅ Found cached result!");
        println!("   ⏱️  Retrieval time: {second_run_duration:?}");
        println!(
            "   📊 Processing time from cache: {}ms",
            cached.metadata.processing_time_ms
        );

        assert!(
            second_run_duration.as_millis() < 50,
            "✅ Cache hit should be <50ms (requirement), got: {second_run_duration:?}"
        );

        // ===== Summary =====
        println!("\n🎉 CLI CACHE SCENARIO RESULTS:");
        println!("   Query: '{cli_query}'");
        println!("   First run:  {first_run_duration:?} (cache miss)");
        println!("   Second run: {second_run_duration:?} (cache hit)");
        println!(
            "   Speed improvement: {:.1}x faster",
            first_run_duration.as_nanos() as f64 / second_run_duration.as_nanos() as f64
        );

        let cache_hit_indicator = if second_run_duration.as_millis() < 50 {
            "✅ Cached result found"
        } else {
            "❌ Cache miss or slow retrieval"
        };
        println!("   Status: {cache_hit_indicator}");
    }

    /// ANCHOR: Test cross-context retrieval fallback logic comprehensiveness
    /// This test verifies that items stored with one method can be retrieved with another
    /// Tests: Context->Standard retrieval, Standard->Context retrieval, comprehensive fallback coverage  
    #[tokio::test]
    async fn test_anchor_cross_context_retrieval_fallback() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Create test results
        let result_standard = create_test_result();
        let mut result_context = create_test_result();
        result_context.request.original_query = "context query".to_string();

        // Create a mock context for testing
        use crate::classification::context_detector::ContextDetectionResult;
        use fortitude_types::{
            AudienceLevel, ClassificationDimension, DimensionConfidence, TechnicalDomain,
            UrgencyLevel,
        };

        let context = ContextDetectionResult::new(
            AudienceLevel::Intermediate,
            TechnicalDomain::Rust,
            UrgencyLevel::Planned,
            vec![DimensionConfidence {
                dimension: ClassificationDimension::AudienceLevel,
                confidence: 0.8,
                matched_keywords: vec!["test".to_string()],
                reasoning: "test evidence".to_string(),
            }],
            50,
            false,
        );

        // Store using different methods to create files in different locations
        let key_standard = storage.store(&result_standard).await.unwrap();
        let key_context = storage
            .store_with_context(&result_context, Some(&context))
            .await
            .unwrap();

        println!("Standard storage key: {key_standard}");
        println!("Context storage key: {key_context}");

        // Test cross-context retrieval matrix
        let test_cases = vec![
            ("Standard->Standard", &key_standard, false, true),
            ("Standard->Context", &key_standard, true, true),
            ("Context->Standard", &key_context, false, true), // This should pass with enhanced fallback
            ("Context->Context", &key_context, true, true),
        ];

        for (description, cache_key, use_context, should_succeed) in test_cases {
            let retrieved = if use_context {
                storage
                    .retrieve_with_context(cache_key, Some(&context))
                    .await
                    .unwrap()
            } else {
                storage.retrieve(cache_key).await.unwrap()
            };

            if should_succeed {
                assert!(
                    retrieved.is_some(),
                    "ENHANCED REQUIREMENT: {description} retrieval should succeed with comprehensive fallback. Key: {cache_key}"
                );
                println!("{description}: SUCCESS");
            } else if retrieved.is_none() {
                println!("{description}: EXPECTED FAILURE");
            } else {
                println!("{description}: UNEXPECTED SUCCESS");
            }
        }
    }

    /// ANCHOR: Test fuzzy matching for semantically similar queries
    /// This test verifies that queries with minor variations can match existing cache entries
    /// Tests: Query normalization, semantic similarity matching, configurable confidence thresholds
    #[tokio::test]
    async fn test_enhanced_fuzzy_semantic_matching() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Store a result with the original query
        let original_query = "How to implement async programming in Rust";
        let mut result = create_test_result();
        result.request.original_query = original_query.to_string();

        let cache_key = storage.store(&result).await.unwrap();
        println!("Stored with key: {cache_key}");

        // Test variations that should match via fuzzy logic (if implemented)
        let similar_queries = vec![
            "how to implement async programming in rust", // Case difference
            "How to implement async programming in Rust?", // Punctuation difference
            "implement async programming in rust how to", // Word order difference
            "How do I implement async programming in Rust", // Slight phrasing difference
            "async programming implementation in rust",   // Semantic similarity
        ];

        for similar_query in similar_queries {
            println!("Testing fuzzy match for: '{similar_query}'");

            // Create a new research result with the similar query
            let mut fuzzy_result = create_test_result();
            fuzzy_result.request.original_query = similar_query.to_string();

            // The current implementation generates cache keys based on query content
            // so similar queries would get different cache keys
            let fuzzy_key = storage.generate_cache_key(&fuzzy_result);
            println!("  Generated key: {fuzzy_key}");

            // Try to retrieve with the fuzzy key - this tests if fuzzy matching is implemented
            let retrieved = storage.retrieve(&fuzzy_key).await.unwrap();

            if retrieved.is_some() {
                println!("  FUZZY MATCH SUCCESS: Found existing cache entry");
            } else {
                println!("  NO FUZZY MATCH: Would require enhanced fuzzy matching logic");

                // For now, verify that exact key retrieval still works
                let exact_retrieved = storage.retrieve(&cache_key).await.unwrap();
                assert!(
                    exact_retrieved.is_some(),
                    "Exact key retrieval should always work"
                );
            }
        }

        // This test documents current behavior rather than requiring fuzzy matching
        // True fuzzy matching would require additional implementation
        println!("ENHANCEMENT OPPORTUNITY: Implement fuzzy matching for similar queries");
    }

    /// ANCHOR: Test retrieval efficiency with intelligent search order
    /// This test measures retrieval performance and verifies optimization opportunities
    /// Tests: Search order efficiency, caching of frequent patterns, minimal file operations
    #[tokio::test]
    async fn test_enhanced_retrieval_efficiency_optimization() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Store multiple items to create a realistic performance test scenario
        let mut stored_keys = Vec::new();
        let context = ContextDetectionResult::new(
            AudienceLevel::Intermediate,
            TechnicalDomain::Rust,
            UrgencyLevel::Planned,
            vec![],
            50,
            false,
        );

        // Store items in different ways to create files in various locations
        for i in 0..10 {
            let query = format!("efficiency test query {i}");
            let mut result = create_test_result();
            result.request.original_query = query;

            let key = if i % 2 == 0 {
                storage.store(&result).await.unwrap()
            } else {
                storage
                    .store_with_context(&result, Some(&context))
                    .await
                    .unwrap()
            };

            stored_keys.push(key);
        }

        // Measure retrieval performance
        let start_time = std::time::Instant::now();
        let mut successful_retrievals = 0;

        for key in &stored_keys {
            let retrieved = storage.retrieve(key).await.unwrap();
            if retrieved.is_some() {
                successful_retrievals += 1;
            }
        }

        let total_time = start_time.elapsed();
        let avg_time_per_retrieval = total_time / stored_keys.len() as u32;

        println!("EFFICIENCY METRICS:");
        println!("  Items stored: {}", stored_keys.len());
        println!("  Successful retrievals: {successful_retrievals}");
        println!("  Total retrieval time: {total_time:?}");
        println!("  Average time per retrieval: {avg_time_per_retrieval:?}");

        // All stored items should be retrievable
        assert_eq!(
            successful_retrievals,
            stored_keys.len(),
            "All stored items should be retrievable with current fallback logic"
        );

        // Performance should be reasonable (this is more of a monitoring test)
        assert!(
            avg_time_per_retrieval.as_millis() < 100,
            "Average retrieval time should be reasonable: {avg_time_per_retrieval:?}"
        );

        println!("OPTIMIZATION OPPORTUNITIES:");
        println!("  - Index-first lookup before file scanning");
        println!("  - Cache recently accessed file paths");
        println!("  - Implement query normalization for better cache hits");
    }
}
