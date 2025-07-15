// ABOUTME: File system monitoring for automated gap analysis in proactive research mode
//! This module provides real-time file system monitoring capabilities for detecting
//! changes that could indicate knowledge gaps in project documentation.

use notify::{EventKind, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Errors that can occur during file monitoring
#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Failed to create file watcher: {0}")]
    WatcherCreation(#[from] notify::Error),

    #[error("Failed to watch path {path}: {error}")]
    WatchPath { path: String, error: notify::Error },

    #[error("Debouncer setup failed: {0}")]
    DebouncerSetup(String),

    #[error("Event processing failed: {0}")]
    EventProcessing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Shutdown timeout")]
    ShutdownTimeout,

    #[error("Channel communication error: {0}")]
    Channel(String),
}

/// Type of file system event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    Create,
    Write,
    Remove,
    Rename,
    Other(String),
}

impl From<EventKind> for EventType {
    fn from(kind: EventKind) -> Self {
        match kind {
            EventKind::Create(_) => EventType::Create,
            EventKind::Modify(_) => EventType::Write,
            EventKind::Remove(_) => EventType::Remove,
            EventKind::Other => EventType::Other("other".to_string()),
            _ => EventType::Other(format!("{kind:?}")),
        }
    }
}

/// A processed file system event
#[derive(Debug, Clone)]
pub struct FileEvent {
    /// Path of the affected file
    pub path: PathBuf,
    /// Type of file system event
    pub event_type: EventType,
    /// Timestamp when the event was processed
    pub timestamp: SystemTime,
    /// Whether this event should trigger gap analysis
    pub should_trigger_analysis: bool,
    /// Priority score for processing (1-10, higher is more urgent)
    pub priority: u8,
}

impl FileEvent {
    /// Create a new file event
    pub fn new(path: PathBuf, event_type: EventType) -> Self {
        let priority = match event_type {
            EventType::Create => 8,   // High priority for new files
            EventType::Write => 6,    // Medium priority for modifications
            EventType::Remove => 4,   // Lower priority for deletions
            EventType::Rename => 5,   // Medium-low priority for renames
            EventType::Other(_) => 3, // Low priority for other events
        };

        Self {
            path,
            event_type,
            timestamp: SystemTime::now(),
            should_trigger_analysis: true,
            priority,
        }
    }

    /// Check if this event should trigger gap analysis based on file type
    pub fn update_analysis_trigger(&mut self, config: &FileMonitorConfig) {
        self.should_trigger_analysis = config.should_analyze_file(&self.path);
    }
}

/// Configuration for file system monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMonitorConfig {
    /// Debounce duration in milliseconds
    pub debounce_ms: u64,
    /// Include patterns for files to monitor
    pub include_patterns: Vec<String>,
    /// Exclude patterns for files to ignore
    pub exclude_patterns: Vec<String>,
    /// Directories to exclude from monitoring
    pub exclude_dirs: HashSet<String>,
    /// Maximum file size to process (in MB)
    pub max_file_size_mb: u64,
    /// Maximum queue size for events
    pub max_queue_size: usize,
    /// Maximum events per second before rate limiting
    pub max_events_per_second: usize,
}

impl Default for FileMonitorConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 300,
            include_patterns: vec!["*".to_string()],
            exclude_patterns: Vec::new(),
            exclude_dirs: HashSet::new(),
            max_file_size_mb: 50,
            max_queue_size: 1000,
            max_events_per_second: 100,
        }
    }
}

impl FileMonitorConfig {
    /// Create configuration optimized for Rust projects
    pub fn for_rust_project() -> Self {
        Self {
            debounce_ms: 300,
            include_patterns: vec![
                "*.rs".to_string(),
                "Cargo.toml".to_string(),
                "Cargo.lock".to_string(),
                "*.md".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
                "*.toml".to_string(),
            ],
            exclude_patterns: vec![
                "target/*".to_string(),
                ".git/*".to_string(),
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".cargo/*".to_string(),
                "*/.DS_Store".to_string(),
                "*/Thumbs.db".to_string(),
            ],
            exclude_dirs: [
                "target",
                ".git",
                ".vscode",
                ".idea",
                "deps",
                "build",
                ".cargo",
                "node_modules",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            max_file_size_mb: 50,
            max_queue_size: 1000,
            max_events_per_second: 100,
        }
    }

    /// Builder method to set debounce duration
    pub fn with_debounce_ms(mut self, debounce_ms: u64) -> Self {
        self.debounce_ms = debounce_ms;
        self
    }

    /// Builder method to set maximum queue size
    pub fn with_max_queue_size(mut self, max_queue_size: usize) -> Self {
        self.max_queue_size = max_queue_size;
        self
    }

    /// Builder method to set include patterns
    pub fn with_include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }

    /// Builder method to set exclude patterns
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// Check if a file should be analyzed based on patterns
    pub fn should_analyze_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check if file is in excluded directory
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            for exclude_dir in &self.exclude_dirs {
                if parent_str.contains(exclude_dir) {
                    return false;
                }
            }
        }

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if glob_match(pattern, &path_str) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.include_patterns {
            if glob_match(pattern, &path_str) {
                return true;
            }
        }

        false
    }
}

/// Simple glob pattern matching for file patterns
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(ext) = pattern.strip_prefix("*.") {
        return text.ends_with(ext);
    }

    if let Some(prefix) = pattern.strip_suffix("/*") {
        return text.contains(prefix);
    }

    pattern == text
}

/// Rate limiter for event processing
#[derive(Debug)]
struct RateLimiter {
    max_events_per_second: usize,
    event_count: Arc<AtomicU64>,
    last_reset: Arc<Mutex<SystemTime>>,
}

impl RateLimiter {
    fn new(max_events_per_second: usize) -> Self {
        Self {
            max_events_per_second,
            event_count: Arc::new(AtomicU64::new(0)),
            last_reset: Arc::new(Mutex::new(SystemTime::now())),
        }
    }

    async fn should_process_event(&self) -> bool {
        let now = SystemTime::now();
        let mut last_reset = self.last_reset.lock().await;

        // Reset counter every second
        if now
            .duration_since(*last_reset)
            .unwrap_or_default()
            .as_secs()
            >= 1
        {
            self.event_count.store(0, Ordering::Relaxed);
            *last_reset = now;
        }

        let current_count = self.event_count.fetch_add(1, Ordering::Relaxed);
        current_count < self.max_events_per_second as u64
    }
}

/// Event queue for managing file events with priority
#[derive(Debug)]
struct EventQueue {
    high_priority: VecDeque<FileEvent>,
    normal_priority: VecDeque<FileEvent>,
    low_priority: VecDeque<FileEvent>,
    max_size: usize,
}

impl EventQueue {
    fn new(max_size: usize) -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            max_size,
        }
    }

    fn enqueue(&mut self, event: FileEvent) -> bool {
        if self.total_size() >= self.max_size {
            // Drop oldest low priority event to make room
            if !self.low_priority.is_empty() {
                self.low_priority.pop_front();
            } else if !self.normal_priority.is_empty() {
                self.normal_priority.pop_front();
            } else {
                return false; // Queue full with only high priority events
            }
        }

        match event.priority {
            8..=10 => self.high_priority.push_back(event),
            4..=7 => self.normal_priority.push_back(event),
            _ => self.low_priority.push_back(event),
        }

        true
    }

    fn dequeue(&mut self) -> Option<FileEvent> {
        self.high_priority
            .pop_front()
            .or_else(|| self.normal_priority.pop_front())
            .or_else(|| self.low_priority.pop_front())
    }

    fn total_size(&self) -> usize {
        self.high_priority.len() + self.normal_priority.len() + self.low_priority.len()
    }
}

/// Main file monitoring system
#[derive(Debug)]
pub struct FileMonitor {
    event_queue: Arc<Mutex<EventQueue>>,
    #[allow(dead_code)] // TODO: Will be used for controlling event processing rate
    rate_limiter: RateLimiter,
    config: FileMonitorConfig,
    shutdown_tx: broadcast::Sender<()>,
    _watcher_task: tokio::task::JoinHandle<()>,
    _debouncer: Arc<Mutex<Debouncer<notify::RecommendedWatcher>>>,
}

impl FileMonitor {
    /// Create a new file monitor for the specified paths
    pub async fn new(paths: Vec<PathBuf>, config: FileMonitorConfig) -> Result<Self, MonitorError> {
        let event_queue = Arc::new(Mutex::new(EventQueue::new(config.max_queue_size)));
        let rate_limiter = RateLimiter::new(config.max_events_per_second);
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

        // Create event channel for debouncer
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<DebounceEventResult>();

        // Create debouncer
        let debounce_duration = Duration::from_millis(config.debounce_ms);
        let mut debouncer = new_debouncer(debounce_duration, move |result| {
            if event_tx.send(result).is_err() {
                warn!("Failed to send debounced event - receiver may be closed");
            }
        })
        .map_err(|e| MonitorError::DebouncerSetup(e.to_string()))?;

        // Watch all specified paths
        for path in &paths {
            if !path.exists() {
                return Err(MonitorError::WatchPath {
                    path: path.to_string_lossy().to_string(),
                    error: notify::Error::generic("Path does not exist"),
                });
            }

            debouncer
                .watcher()
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| MonitorError::WatchPath {
                    path: path.to_string_lossy().to_string(),
                    error: e,
                })?;
        }

        let debouncer = Arc::new(Mutex::new(debouncer));

        // Start event processing task
        let event_queue_clone = Arc::clone(&event_queue);
        let config_clone = config.clone();
        let rate_limiter_clone = RateLimiter::new(config.max_events_per_second);

        let watcher_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("File monitor shutdown signal received");
                        break;
                    }
                    event_result = event_rx.recv() => {
                        match event_result {
                            Some(Ok(events)) => {
                                Self::process_debounced_events(
                                    events,
                                    &event_queue_clone,
                                    &config_clone,
                                    &rate_limiter_clone,
                                ).await;
                            }
                            Some(Err(e)) => {
                                error!("Debouncer error: {:?}", e);
                            }
                            None => {
                                debug!("Event receiver channel closed");
                                break;
                            }
                        }
                    }
                }
            }
        });

        info!("File monitor started for {} paths", paths.len());

        Ok(Self {
            event_queue,
            rate_limiter,
            config,
            shutdown_tx,
            _watcher_task: watcher_task,
            _debouncer: debouncer,
        })
    }

    /// Process debounced events from the file watcher
    async fn process_debounced_events(
        events: Vec<DebouncedEvent>,
        event_queue: &Arc<Mutex<EventQueue>>,
        config: &FileMonitorConfig,
        rate_limiter: &RateLimiter,
    ) {
        for debounced_event in events {
            if !rate_limiter.should_process_event().await {
                warn!(
                    "Rate limit exceeded, dropping event for: {:?}",
                    debounced_event.path
                );
                continue;
            }

            // For debounced events, determine type based on file existence and metadata
            let event_type = if debounced_event.path.exists() {
                // Check if this is likely a new file by checking creation time
                if let Ok(metadata) = std::fs::metadata(&debounced_event.path) {
                    if let Ok(created) = metadata.created() {
                        if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default();
                            // If file was created within the last few seconds, consider it a create event
                            if now.as_secs() - duration.as_secs() < 5 {
                                EventType::Create
                            } else {
                                EventType::Write
                            }
                        } else {
                            EventType::Write
                        }
                    } else {
                        EventType::Write
                    }
                } else {
                    EventType::Write
                }
            } else {
                EventType::Remove
            };

            let mut file_event = FileEvent::new(debounced_event.path.clone(), event_type);

            file_event.update_analysis_trigger(config);

            if file_event.should_trigger_analysis {
                let mut queue = event_queue.lock().await;
                if !queue.enqueue(file_event.clone()) {
                    warn!(
                        "Event queue full, dropping event for: {:?}",
                        debounced_event.path
                    );
                } else {
                    debug!(
                        "Enqueued file event: {:?} (priority: {})",
                        debounced_event.path, file_event.priority
                    );
                }
            }
        }
    }

    /// Get the next file event from the queue
    pub async fn next_event(&mut self) -> Option<FileEvent> {
        loop {
            {
                let mut queue = self.event_queue.lock().await;
                if let Some(event) = queue.dequeue() {
                    return Some(event);
                }
            }

            // Wait a bit before checking again
            sleep(Duration::from_millis(10)).await;
        }
    }

    /// Shutdown the file monitor gracefully
    pub async fn shutdown(self) -> Result<(), MonitorError> {
        debug!("Shutting down file monitor");

        // Send shutdown signal
        if self.shutdown_tx.send(()).is_err() {
            warn!("Failed to send shutdown signal - receivers may be closed");
        }

        // Wait for watcher task to complete with timeout
        match timeout(Duration::from_millis(500), self._watcher_task).await {
            Ok(_) => {
                info!("File monitor shutdown completed");
                Ok(())
            }
            Err(_) => {
                warn!("File monitor shutdown timed out");
                Err(MonitorError::ShutdownTimeout)
            }
        }
    }

    /// Get current queue statistics
    pub async fn get_queue_stats(&self) -> (usize, usize) {
        let queue = self.event_queue.lock().await;
        (queue.total_size(), self.config.max_queue_size)
    }

    /// Watch a specific directory for file system events
    pub async fn watch_directory(&self, path: &Path) -> Result<(), MonitorError> {
        if !path.exists() {
            return Err(MonitorError::WatchPath {
                path: path.to_string_lossy().to_string(),
                error: notify::Error::generic("Path does not exist"),
            });
        }

        let mut debouncer = self._debouncer.lock().await;
        debouncer
            .watcher()
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| MonitorError::WatchPath {
                path: path.to_string_lossy().to_string(),
                error: e,
            })?;

        info!("Started watching directory: {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("*.rs", "test.rs"));
        assert!(glob_match("*.rs", "src/main.rs"));
        assert!(!glob_match("*.rs", "test.py"));

        assert!(glob_match("target/*", "target/debug/app"));
        assert!(glob_match("target/*", "src/target/build"));
        assert!(!glob_match("target/*", "src/main.rs"));

        assert!(glob_match("*", "anything"));
        assert!(glob_match("test.txt", "test.txt"));
        assert!(!glob_match("test.txt", "other.txt"));
    }

    #[test]
    fn test_config_should_analyze_file() {
        let config = FileMonitorConfig::for_rust_project();

        assert!(config.should_analyze_file(&PathBuf::from("src/main.rs")));
        assert!(config.should_analyze_file(&PathBuf::from("Cargo.toml")));
        assert!(config.should_analyze_file(&PathBuf::from("README.md")));

        assert!(!config.should_analyze_file(&PathBuf::from("target/debug/app")));
        assert!(!config.should_analyze_file(&PathBuf::from(".git/HEAD")));
        assert!(!config.should_analyze_file(&PathBuf::from("test.tmp")));
    }

    #[test]
    fn test_event_priority() {
        let create_event = FileEvent::new(PathBuf::from("test.rs"), EventType::Create);
        assert_eq!(create_event.priority, 8);

        let write_event = FileEvent::new(PathBuf::from("test.rs"), EventType::Write);
        assert_eq!(write_event.priority, 6);

        let remove_event = FileEvent::new(PathBuf::from("test.rs"), EventType::Remove);
        assert_eq!(remove_event.priority, 4);
    }

    #[tokio::test]
    async fn test_event_queue() {
        let mut queue = EventQueue::new(3);

        let high_event = FileEvent {
            path: PathBuf::from("test1.rs"),
            event_type: EventType::Create,
            timestamp: SystemTime::now(),
            should_trigger_analysis: true,
            priority: 9,
        };

        let low_event = FileEvent {
            path: PathBuf::from("test2.rs"),
            event_type: EventType::Remove,
            timestamp: SystemTime::now(),
            should_trigger_analysis: true,
            priority: 3,
        };

        assert!(queue.enqueue(low_event));
        assert!(queue.enqueue(high_event));

        // Should dequeue high priority event first
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.priority, 9);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2);

        assert!(limiter.should_process_event().await);
        assert!(limiter.should_process_event().await);
        assert!(!limiter.should_process_event().await); // Should be rate limited
    }
}
