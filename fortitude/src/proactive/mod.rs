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

// ABOUTME: Proactive research mode module for automated gap analysis and background research
//! This module provides automated knowledge gap detection and background research execution.
//! It monitors file system changes, analyzes documentation gaps, and executes research tasks
//! to proactively build knowledge before it's needed.

pub mod background_scheduler;
pub mod comprehensive_config;
pub mod config;
pub mod configurable_analyzer;
pub mod context_aware_scorer;
pub mod error_handler;
pub mod file_monitor;
pub mod gap_analyzer;
pub mod impact_assessor;
pub mod integrated_analyzer;
pub mod manager;
pub mod notification_delivery_verifier;
pub mod notification_system;
pub mod prioritization;
pub mod progress_tracker;
pub mod research_completion;
pub mod scheduler;
pub mod semantic_analyzer;
pub mod state_manager;
pub mod task_executor;
pub mod user_preferences;

pub use background_scheduler::{
    BackgroundScheduler, BackgroundSchedulerConfig, QueueMetrics, QueueOperations, ResearchTask,
    SchedulerError, TaskPriority, TaskState,
};
pub use comprehensive_config::{
    BackgroundResearchConfig, CachingConfig, ConfigMetadata,
    ConfigurationError as ComprehensiveConfigError, CustomGapRule, CustomProjectType,
    GapAnalysisConfig as ProactiveGapAnalysisConfig, InterfacePreferences, NotificationConfig,
    NotificationRateLimiting, NotificationRule, NotificationTemplate, NotificationThrottling,
    OptimizationConfig, PerformanceAlert, PerformanceConfig as ComprehensivePerformanceConfig,
    PersonalizationConfig, ProactiveConfig, ProactiveConfigManager, ProjectTypeConfig,
    ResearchIntegrationConfig, ResearchQualityConfig, ResourceLimitsConfig, UserPreferenceConfig,
    WorkspaceConfig, WorkspaceIntegrationConfig,
};
pub use config::{
    ApiDocumentationRules, ConfigurationError, ConfigurationRules, CustomPriorityRule,
    DetectionRules, DetectionSettings, DocumentationRules, DuplicateDetectionConfig,
    ExclusionRules, FilteringConfig, GapDetectionConfig, PerformanceConfig, PriorityBoosts,
    PriorityConfig, QualityThresholds, SemanticConfig, TechnologyRules, TodoDetectionRules,
    ValidationRules,
};
pub use configurable_analyzer::{
    AppliedRule, ConfigSummary, ConfigurableAnalysisError, ConfigurableAnalysisResult,
    ConfigurableGapAnalyzer, ConfigurablePerformanceMetrics, EnhancedDetectedGap,
    PriorityAdjustment, PriorityBreakdown, RuleStatistics,
};
pub use context_aware_scorer::{
    AudiencePriorityAdjustments, ContextAwarePriorityBreakdown, ContextAwarePriorityScorer,
    ContextAwareScoringConfig, ContextAwareScoringError, DomainPriorityWeights, ExtractedContext,
    UrgencyPriorityScaling,
};
pub use error_handler::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, DeadLetterEntry, ErrorClassification,
    ErrorHandler, ErrorHandlerConfig, ErrorMetrics, NetworkStatus, ProactiveError,
    RecoveryStrategy, RetryStrategy, ServiceStatus,
};
pub use file_monitor::{EventType, FileEvent, FileMonitor, FileMonitorConfig, MonitorError};
pub use gap_analyzer::{DetectedGap, GapAnalysisConfig, GapAnalysisError, GapAnalyzer, GapType};
pub use impact_assessor::{
    ApiVisibilityAnalysis, DependencyImpactAnalysis, DevelopmentActivityAnalysis,
    ImpactAssessmentConfig, ImpactAssessmentError, ImpactAssessmentMetrics, ImpactAssessmentResult,
    ImpactAssessor, TeamImpactAnalysis, UsagePatternAnalysis,
};
pub use integrated_analyzer::{
    AnalysisPerformanceMetrics, EnhancedGap, IntegratedAnalysisConfig, IntegratedAnalysisError,
    IntegratedAnalysisResult, IntegratedGapAnalyzer,
};
pub use manager::{
    ConfigSummary as ProactiveConfigSummary, ProactiveEvent, ProactiveEventType, ProactiveManager,
    ProactiveManagerConfig, ProactiveManagerError, ProactiveStatus,
};
pub use notification_delivery_verifier::{
    AuditTrailEntry, ChannelDeliveryVerifier, ChannelPerformanceMetrics, ChannelVerificationResult,
    ChannelVerificationRules, DeliveryAttempt, DeliveryPerformanceMetrics, DeliveryStatus,
    DeliveryVerificationConfig, DeliveryVerificationError, NotificationDeliveryStatus,
    NotificationDeliveryVerifier, OverallDeliveryStatus,
};
pub use notification_system::{
    ChannelMetrics, Notification, NotificationChannel, NotificationChannelConfig,
    NotificationMetrics, NotificationSystem, NotificationSystemConfig, NotificationSystemError,
    NotificationType,
};
pub use prioritization::{
    DevelopmentContext, DevelopmentPhase, PrioritizationConfig, PrioritizationError,
    PrioritizationMetrics, PriorityScoreBreakdown, PriorityScorer,
};
pub use progress_tracker::{
    EnhancedTaskProgress, ProgressEvent, ProgressPerformanceMetrics, ProgressStep, ProgressTracker,
    ProgressTrackerConfig, ProgressTrackerError,
};
pub use research_completion::{
    CompletionNotificationLevel, NextAction, NextActionType, PerformanceMetrics, QualityMetrics,
    ResearchCompletionConfig, ResearchCompletionError, ResearchCompletionNotifier, ResearchResult,
    ResearchResultSummary, SummaryPerformanceMetrics,
};
pub use scheduler::{
    ResearchScheduler, ResearchSchedulerConfig, ResearchSchedulerError, ResourceLimits,
    ScheduledJob, ScheduledJobType, SchedulerMetrics,
};
pub use semantic_analyzer::{
    RelatedDocument, RelationshipType, SemanticAnalysisConfig, SemanticAnalysisError,
    SemanticAnalysisMetadata, SemanticGapAnalysis, SemanticGapAnalyzer,
};
pub use state_manager::{
    OrphanedTaskStrategy, StaleTaskStrategy, StateChangeEntry, StateEvent, StateManager,
    StateManagerConfig, StateManagerError, StateManagerMetrics, StateMonitoringConfig,
    StateRecoveryConfig, StateTransitionMetadata, StateValidationRules, TaskLifecycle,
};
pub use task_executor::{
    ExecutorMetrics, ResourceUsage, TaskExecutor, TaskExecutorConfig, TaskExecutorError,
    TaskProgress, TokenBucket,
};
pub use user_preferences::{
    BusinessHours, ContextualNotificationSettings, CustomPriorityWeights,
    EffectiveNotificationSettings, ExpertiseLevel, NotificationChannelSettings,
    NotificationDetailLevel, NotificationFrequency, NotificationPreferenceFilter,
    NotificationPreferences, NotificationTypeSettings, PersonalFilters,
    PreferenceAwareNotificationSender, PriorityOverrideSettings, QuietHours, TimeRange,
    UserAwarePriorityBreakdown, UserAwarePriorityScorer, UserPreferenceError,
    UserPreferenceManager, UserPreferenceProfile, WorkflowMode,
};
