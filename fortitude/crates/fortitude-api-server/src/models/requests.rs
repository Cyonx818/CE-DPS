// ABOUTME: Request model definitions for API endpoints

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Research request parameters for submitting new research queries
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ResearchRequest {
    /// The research query or topic
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Query must be between 1 and 1000 characters"
    ))]
    pub query: String,

    /// Optional additional context for the research
    #[validate(length(max = 5000, message = "Context must be less than 5000 characters"))]
    pub context: Option<String>,

    /// Research priority level (low, medium, high)
    pub priority: Option<String>,

    /// Optional audience context override
    pub audience_context: Option<AudienceContextRequest>,

    /// Optional domain context override
    pub domain_context: Option<DomainContextRequest>,
}

/// Audience context parameters for research customization
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct AudienceContextRequest {
    /// Technical level (beginner, intermediate, advanced)
    #[validate(length(min = 1, max = 50))]
    pub level: String,

    /// Domain expertise (rust, web, devops, etc.)
    #[validate(length(min = 1, max = 50))]
    pub domain: String,

    /// Output format preference (markdown, json, plain)
    #[validate(length(min = 1, max = 20))]
    pub format: String,
}

/// Domain context parameters for research customization
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct DomainContextRequest {
    /// Technology stack (rust, javascript, python, etc.)
    #[validate(length(min = 1, max = 50))]
    pub technology: String,

    /// Project type (web, cli, library, etc.)
    #[validate(length(min = 1, max = 50))]
    pub project_type: String,

    /// Specific frameworks or libraries
    pub frameworks: Vec<String>,

    /// Additional context tags
    pub tags: Vec<String>,
}

/// Research listing request parameters for filtering and pagination
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema, IntoParams)]
pub struct ResearchListRequest {
    /// Filter by research type
    pub research_type: Option<String>,

    /// Filter by keywords (searches in query, content, and tags)
    pub keywords: Option<String>,

    /// Filter by minimum quality score (0.0-1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_quality: Option<f64>,

    /// Filter by tags
    pub tags: Option<Vec<String>>,

    /// Maximum results to return (1-100)
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Result offset for pagination
    #[validate(range(min = 0))]
    pub offset: Option<usize>,

    /// Sort order (newest, oldest, quality, relevance)
    pub sort: Option<String>,
}

/// Classification request parameters for advanced multi-dimensional classification
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ClassificationRequest {
    /// Content to classify
    #[validate(length(
        min = 1,
        max = 10000,
        message = "Content must be between 1 and 10000 characters"
    ))]
    pub content: String,

    /// Classification options for processing
    #[validate(nested)]
    pub options: Option<ClassificationOptions>,

    /// Context preferences for classification
    #[validate(nested)]
    pub context_preferences: Option<ContextPreferences>,
}

/// Classification options for controlling the analysis
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ClassificationOptions {
    /// Enable context detection (audience, domain, urgency)
    pub enable_context_detection: Option<bool>,

    /// Enable advanced classification with signal composition
    pub enable_advanced_classification: Option<bool>,

    /// Minimum confidence threshold for results
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence_threshold: Option<f64>,

    /// Maximum processing time in milliseconds
    #[validate(range(min = 100, max = 30000))]
    pub max_processing_time_ms: Option<u64>,

    /// Include confidence explanations in response
    pub include_explanations: Option<bool>,
}

/// Context preferences for classification
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ContextPreferences {
    /// Preferred audience level for context (beginner, intermediate, advanced)
    pub preferred_audience: Option<String>,

    /// Preferred technical domain (rust, web, devops, etc.)
    pub preferred_domain: Option<String>,

    /// Expected urgency level (exploratory, planned, immediate)
    pub expected_urgency: Option<String>,
}

/// Classification listing request for filtering cached results
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema, IntoParams)]
pub struct ClassificationListRequest {
    /// Filter by research type
    pub research_type: Option<String>,

    /// Filter by audience level
    pub audience_level: Option<String>,

    /// Filter by technical domain
    pub technical_domain: Option<String>,

    /// Filter by urgency level  
    pub urgency_level: Option<String>,

    /// Filter by keywords (searches in content and matched keywords)
    pub keywords: Option<String>,

    /// Filter by minimum confidence score (0.0-1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_confidence: Option<f64>,

    /// Maximum results to return (1-100)
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Result offset for pagination
    #[validate(range(min = 0))]
    pub offset: Option<usize>,

    /// Sort order (newest, oldest, confidence, research_type)
    pub sort: Option<String>,
}

/// Cache invalidation request
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CacheInvalidationRequest {
    /// Cache keys to invalidate (optional - if empty, clears all)
    pub keys: Option<Vec<String>>,

    /// Cache namespace to target
    pub namespace: Option<String>,
}

/// Authentication/login request
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Username or user ID
    #[validate(length(
        min = 1,
        max = 100,
        message = "Username must be between 1 and 100 characters"
    ))]
    pub username: String,

    /// Password or authentication token
    #[validate(length(
        min = 1,
        max = 500,
        message = "Password must be between 1 and 500 characters"
    ))]
    pub password: String,

    /// Optional permissions to request
    pub permissions: Option<Vec<String>>,
}

/// Token refresh request
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct TokenRefreshRequest {
    /// Current valid token to refresh
    #[validate(length(min = 1, message = "Token cannot be empty"))]
    pub token: String,
}

/// Cache search request parameters for filtering and pagination
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema, IntoParams)]
pub struct CacheSearchRequest {
    /// Search query for content matching
    pub query: Option<String>,

    /// Filter by research type
    pub research_type: Option<String>,

    /// Filter by keywords (searches in query, content, and tags)
    pub keywords: Option<String>,

    /// Filter by minimum quality score (0.0-1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_quality: Option<f64>,

    /// Filter by tags
    pub tags: Option<Vec<String>>,

    /// Filter by minimum age in seconds
    #[validate(range(min = 0))]
    pub min_age_seconds: Option<u64>,

    /// Filter by maximum age in seconds
    #[validate(range(min = 0))]
    pub max_age_seconds: Option<u64>,

    /// Filter by minimum size in bytes
    #[validate(range(min = 0))]
    pub min_size_bytes: Option<u64>,

    /// Filter by maximum size in bytes
    #[validate(range(min = 0))]
    pub max_size_bytes: Option<u64>,

    /// Maximum results to return (1-100)
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Result offset for pagination
    #[validate(range(min = 0))]
    pub offset: Option<usize>,

    /// Sort order (newest, oldest, quality, size, relevance)
    pub sort: Option<String>,
}

/// Cache invalidation request for bulk operations
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct CacheInvalidateRequest {
    /// Specific cache keys to invalidate
    pub keys: Option<Vec<String>>,

    /// Pattern to match for bulk invalidation (supports wildcards)
    pub pattern: Option<String>,

    /// Filter by research type for bulk invalidation
    pub research_type: Option<String>,

    /// Filter by tags for bulk invalidation
    pub tags: Option<Vec<String>>,

    /// Filter by maximum age for bulk invalidation (seconds)
    #[validate(range(min = 0))]
    pub max_age_seconds: Option<u64>,

    /// Filter by minimum quality score for bulk invalidation
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_quality: Option<f64>,

    /// Dry run mode - show what would be invalidated without actually doing it
    pub dry_run: Option<bool>,
}

/// Proactive research configuration update request
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ProactiveConfigRequest {
    /// Base directory for monitoring
    pub base_directory: Option<String>,

    /// File patterns to monitor
    pub file_patterns: Option<Vec<String>>,

    /// File patterns to ignore
    pub ignore_patterns: Option<Vec<String>>,

    /// Enable/disable proactive research mode
    pub enabled: Option<bool>,

    /// Monitoring interval in seconds
    #[validate(range(min = 10, max = 3600))]
    pub monitoring_interval_seconds: Option<u64>,

    /// Maximum concurrent research tasks
    #[validate(range(min = 1, max = 20))]
    pub max_concurrent_tasks: Option<u8>,

    /// Research priority threshold (0.0-1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub priority_threshold: Option<f64>,

    /// Auto-execute high priority research
    pub auto_execute_high_priority: Option<bool>,

    /// Notification preferences
    pub notification_preferences: Option<ProactiveNotificationPreferences>,
}

/// Proactive notification preferences
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ProactiveNotificationPreferences {
    /// Enable gap detection notifications
    pub gap_detection_enabled: Option<bool>,

    /// Enable research completion notifications
    pub research_completion_enabled: Option<bool>,

    /// Enable error notifications
    pub error_notifications_enabled: Option<bool>,

    /// Notification frequency (immediate, hourly, daily)
    pub frequency: Option<String>,

    /// Minimum priority level for notifications
    pub min_priority_level: Option<String>,
}

/// Proactive research task list request with filtering
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema, IntoParams)]
pub struct ProactiveTaskListRequest {
    /// Filter by task status (pending, running, completed, failed)
    pub status: Option<String>,

    /// Filter by task priority (low, medium, high, urgent)
    pub priority: Option<String>,

    /// Filter by research type
    pub research_type: Option<String>,

    /// Filter by gap type (documentation, implementation, configuration, testing)
    pub gap_type: Option<String>,

    /// Filter by creation date (after this timestamp)
    pub created_after: Option<String>,

    /// Filter by creation date (before this timestamp)
    pub created_before: Option<String>,

    /// Filter by keywords in task description
    pub keywords: Option<String>,

    /// Maximum results to return (1-100)
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Result offset for pagination
    #[validate(range(min = 0))]
    pub offset: Option<usize>,

    /// Sort order (newest, oldest, priority, status)
    pub sort: Option<String>,
}

/// Proactive notification list request with filtering
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema, IntoParams)]
pub struct ProactiveNotificationListRequest {
    /// Filter by notification type (gap_detected, research_completed, error, status_change)
    pub notification_type: Option<String>,

    /// Filter by notification level (info, warning, error, critical)
    pub level: Option<String>,

    /// Filter by read status
    pub read: Option<bool>,

    /// Filter by creation date (after this timestamp)
    pub created_after: Option<String>,

    /// Filter by creation date (before this timestamp)
    pub created_before: Option<String>,

    /// Maximum results to return (1-100)
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Result offset for pagination
    #[validate(range(min = 0))]
    pub offset: Option<usize>,

    /// Sort order (newest, oldest, level)
    pub sort: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_request_validation() {
        let valid_request = ResearchRequest {
            query: "Test query".to_string(),
            context: Some("Test context".to_string()),
            priority: Some("high".to_string()),
            audience_context: Some(AudienceContextRequest {
                level: "intermediate".to_string(),
                domain: "rust".to_string(),
                format: "markdown".to_string(),
            }),
            domain_context: Some(DomainContextRequest {
                technology: "rust".to_string(),
                project_type: "cli".to_string(),
                frameworks: vec!["clap".to_string()],
                tags: vec!["async".to_string()],
            }),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ResearchRequest {
            query: "".to_string(), // Too short
            context: None,
            priority: None,
            audience_context: None,
            domain_context: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_research_list_request_validation() {
        let valid_request = ResearchListRequest {
            research_type: Some("implementation".to_string()),
            keywords: Some("rust async".to_string()),
            min_quality: Some(0.8),
            tags: Some(vec!["rust".to_string()]),
            limit: Some(20),
            offset: Some(0),
            sort: Some("newest".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ResearchListRequest {
            research_type: None,
            keywords: None,
            min_quality: Some(1.5), // Too high
            tags: None,
            limit: Some(150), // Too high
            offset: None,
            sort: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_audience_context_validation() {
        let valid_context = AudienceContextRequest {
            level: "advanced".to_string(),
            domain: "web".to_string(),
            format: "json".to_string(),
        };

        assert!(valid_context.validate().is_ok());

        let invalid_context = AudienceContextRequest {
            level: "".to_string(), // Too short
            domain: "test".to_string(),
            format: "markdown".to_string(),
        };

        assert!(invalid_context.validate().is_err());
    }

    #[test]
    fn test_classification_request_validation() {
        let valid_request = ClassificationRequest {
            content: "Test content for classification".to_string(),
            options: Some(ClassificationOptions {
                enable_context_detection: Some(true),
                enable_advanced_classification: Some(false),
                confidence_threshold: Some(0.6),
                max_processing_time_ms: Some(5000),
                include_explanations: Some(true),
            }),
            context_preferences: Some(ContextPreferences {
                preferred_audience: Some("intermediate".to_string()),
                preferred_domain: Some("rust".to_string()),
                expected_urgency: Some("planned".to_string()),
            }),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ClassificationRequest {
            content: "".to_string(), // Too short
            options: None,
            context_preferences: None,
        };

        assert!(invalid_request.validate().is_err());

        let invalid_options = ClassificationRequest {
            content: "Valid content".to_string(),
            options: Some(ClassificationOptions {
                enable_context_detection: Some(true),
                enable_advanced_classification: Some(false),
                confidence_threshold: Some(1.5), // Invalid - too high
                max_processing_time_ms: Some(50), // Invalid - too low
                include_explanations: Some(true),
            }),
            context_preferences: None,
        };

        assert!(invalid_options.validate().is_err());
    }

    #[test]
    fn test_classification_list_request_validation() {
        let valid_request = ClassificationListRequest {
            research_type: Some("implementation".to_string()),
            audience_level: Some("intermediate".to_string()),
            technical_domain: Some("rust".to_string()),
            urgency_level: Some("planned".to_string()),
            keywords: Some("async await".to_string()),
            min_confidence: Some(0.7),
            limit: Some(20),
            offset: Some(0),
            sort: Some("confidence".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ClassificationListRequest {
            research_type: None,
            audience_level: None,
            technical_domain: None,
            urgency_level: None,
            keywords: None,
            min_confidence: Some(1.5), // Invalid - too high
            limit: Some(150),          // Invalid - too high
            offset: None,
            sort: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_cache_search_request_validation() {
        let valid_request = CacheSearchRequest {
            query: Some("rust async".to_string()),
            research_type: Some("implementation".to_string()),
            keywords: Some("async await".to_string()),
            min_quality: Some(0.8),
            tags: Some(vec!["rust".to_string(), "async".to_string()]),
            min_age_seconds: Some(0),
            max_age_seconds: Some(86400),
            min_size_bytes: Some(100),
            max_size_bytes: Some(1000000),
            limit: Some(20),
            offset: Some(0),
            sort: Some("relevance".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CacheSearchRequest {
            query: None,
            research_type: None,
            keywords: None,
            min_quality: Some(1.5), // Invalid - too high
            tags: None,
            min_age_seconds: None,
            max_age_seconds: None,
            min_size_bytes: None,
            max_size_bytes: None,
            limit: Some(150), // Invalid - too high
            offset: None,
            sort: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_cache_invalidate_request_validation() {
        let valid_request = CacheInvalidateRequest {
            keys: Some(vec!["key1".to_string(), "key2".to_string()]),
            pattern: Some("*.json".to_string()),
            research_type: Some("implementation".to_string()),
            tags: Some(vec!["test".to_string()]),
            max_age_seconds: Some(3600),
            min_quality: Some(0.5),
            dry_run: Some(true),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CacheInvalidateRequest {
            keys: None,
            pattern: None,
            research_type: None,
            tags: None,
            max_age_seconds: None,
            min_quality: Some(1.5), // Invalid - too high
            dry_run: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_proactive_config_request_validation() {
        let valid_request = ProactiveConfigRequest {
            base_directory: Some("/path/to/monitor".to_string()),
            file_patterns: Some(vec!["*.rs".to_string(), "*.md".to_string()]),
            ignore_patterns: Some(vec!["target/".to_string()]),
            enabled: Some(true),
            monitoring_interval_seconds: Some(300),
            max_concurrent_tasks: Some(5),
            priority_threshold: Some(0.7),
            auto_execute_high_priority: Some(true),
            notification_preferences: Some(ProactiveNotificationPreferences {
                gap_detection_enabled: Some(true),
                research_completion_enabled: Some(true),
                error_notifications_enabled: Some(true),
                frequency: Some("immediate".to_string()),
                min_priority_level: Some("medium".to_string()),
            }),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ProactiveConfigRequest {
            base_directory: None,
            file_patterns: None,
            ignore_patterns: None,
            enabled: None,
            monitoring_interval_seconds: Some(5), // Too low
            max_concurrent_tasks: Some(25),       // Too high
            priority_threshold: Some(1.5),        // Too high
            auto_execute_high_priority: None,
            notification_preferences: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_proactive_task_list_request_validation() {
        let valid_request = ProactiveTaskListRequest {
            status: Some("running".to_string()),
            priority: Some("high".to_string()),
            research_type: Some("implementation".to_string()),
            gap_type: Some("documentation".to_string()),
            created_after: Some("2024-01-01T00:00:00Z".to_string()),
            created_before: Some("2024-12-31T23:59:59Z".to_string()),
            keywords: Some("rust async".to_string()),
            limit: Some(50),
            offset: Some(0),
            sort: Some("priority".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ProactiveTaskListRequest {
            status: None,
            priority: None,
            research_type: None,
            gap_type: None,
            created_after: None,
            created_before: None,
            keywords: None,
            limit: Some(150), // Too high
            offset: None,
            sort: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_proactive_notification_list_request_validation() {
        let valid_request = ProactiveNotificationListRequest {
            notification_type: Some("gap_detected".to_string()),
            level: Some("warning".to_string()),
            read: Some(false),
            created_after: Some("2024-01-01T00:00:00Z".to_string()),
            created_before: Some("2024-12-31T23:59:59Z".to_string()),
            limit: Some(25),
            offset: Some(0),
            sort: Some("newest".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = ProactiveNotificationListRequest {
            notification_type: None,
            level: None,
            read: None,
            created_after: None,
            created_before: None,
            limit: Some(150), // Too high
            offset: None,
            sort: None,
        };

        assert!(invalid_request.validate().is_err());
    }
}
