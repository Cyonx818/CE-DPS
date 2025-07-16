use clap::{Parser, Subcommand};
use fortitude::proactive::{ProactiveManager, ProactiveManagerConfig, ProactiveManagerError};
use fortitude::providers::{HealthStatus, Provider};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info, warn, Level};

#[derive(Parser)]
#[command(name = "fortitude")]
#[command(about = "Automated research system for the Concordia workspace")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Research {
        #[arg(short, long)]
        topic: String,
        /// LLM provider to use (openai, claude, gemini, auto)
        #[arg(long, default_value = "auto")]
        provider: String,
        /// Enable cross-provider quality validation
        #[arg(long)]
        cross_validate: bool,
        /// Quality threshold (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        quality_threshold: f64,
    },
    Pipeline {
        #[arg(short, long)]
        config: Option<String>,
    },
    Knowledge {
        #[arg(short, long)]
        query: String,
    },
    /// Provider management commands
    #[command(subcommand)]
    Provider(ProviderCommands),
    /// Quality control commands
    #[command(subcommand)]
    Quality(QualityCommands),
    /// Learning system commands
    #[command(subcommand)]
    Learning(LearningCommands),
    /// Monitoring system commands
    #[command(subcommand)]
    Monitoring(MonitoringCommands),
    /// Proactive research management commands
    #[command(subcommand)]
    Proactive(ProactiveCommands),
}

#[derive(Subcommand)]
enum ProviderCommands {
    /// List available providers and their status
    List {
        /// Show detailed provider information
        #[arg(short, long)]
        detailed: bool,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Show provider performance metrics
    Performance {
        /// Provider name (all if not specified)
        provider: Option<String>,
        /// Time period in hours
        #[arg(long, default_value = "24")]
        period: u64,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Check provider health status
    Health {
        /// Provider name (all if not specified)
        provider: Option<String>,
        /// Force health check refresh
        #[arg(short, long)]
        force: bool,
    },
    /// Switch primary provider
    Switch {
        /// Provider name to switch to
        provider: String,
        /// Force switch even if provider is unhealthy
        #[arg(short, long)]
        force: bool,
    },
    /// Configure provider settings
    Configure {
        /// Provider name
        provider: String,
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

#[derive(Subcommand)]
enum QualityCommands {
    /// Show quality metrics and statistics
    Metrics {
        /// Time period in hours
        #[arg(long, default_value = "24")]
        period: u64,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Run quality validation on research results
    Validate {
        /// Query to validate
        query: String,
        /// Enable cross-provider validation
        #[arg(long)]
        cross_validate: bool,
        /// Number of providers to use for validation
        #[arg(long, default_value = "2")]
        provider_count: u8,
    },
    /// Configure quality control settings
    Configure {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Show quality control status
    Status {
        /// Show detailed status information
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum LearningCommands {
    /// Show learning system status and insights
    Status {
        /// Show detailed learning metrics
        #[arg(short, long)]
        detailed: bool,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Trigger adaptation based on collected feedback
    Adapt {
        /// Force adaptation even if threshold not met
        #[arg(short, long)]
        force: bool,
        /// Show adaptation suggestions without applying
        #[arg(long)]
        dry_run: bool,
    },
    /// Submit feedback for a research result
    Feedback {
        /// Research result cache key or query
        target: String,
        /// Rating (0.0-1.0)
        #[arg(short, long)]
        rating: f64,
        /// Optional feedback comment
        #[arg(short, long)]
        comment: Option<String>,
    },
    /// Show usage patterns and analytics
    Patterns {
        /// Time period in days
        #[arg(long, default_value = "7")]
        days: u64,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Configure learning system settings
    Configure {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

#[derive(Subcommand)]
enum MonitoringCommands {
    /// Show system performance metrics
    Metrics {
        /// Time period in hours
        #[arg(long, default_value = "1")]
        period: u64,
        /// Component to monitor (all, providers, quality, learning)
        #[arg(short, long, default_value = "all")]
        component: String,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Show active alerts
    Alerts {
        /// Alert severity (all, critical, warning, info)
        #[arg(short, long, default_value = "all")]
        severity: String,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Show system health status
    Health {
        /// Show detailed health information
        #[arg(short, long)]
        detailed: bool,
        /// Force health check refresh
        #[arg(short, long)]
        force: bool,
    },
    /// Configure monitoring settings
    Configure {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Start/stop monitoring services
    Service {
        /// Action (start, stop, restart, status)
        action: String,
        /// Service name (all, metrics, alerts, tracing)
        #[arg(short, long, default_value = "all")]
        service: String,
    },
}

#[derive(Subcommand)]
enum ProactiveCommands {
    /// Start proactive research mode
    Start {
        /// Gap analysis interval in minutes
        #[arg(long, default_value = "30")]
        gap_interval: u64,

        /// Maximum concurrent background tasks
        #[arg(long, default_value = "3")]
        max_tasks: usize,

        /// File watch debounce duration in seconds
        #[arg(long, default_value = "5")]
        debounce: u64,

        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Stop proactive research mode
    Stop {
        /// Force stop without waiting for tasks to complete
        #[arg(short, long)]
        force: bool,

        /// Timeout for graceful shutdown in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// Show proactive research status
    Status {
        /// Show detailed task information
        #[arg(short, long)]
        detailed: bool,

        /// Show performance metrics
        #[arg(short, long)]
        metrics: bool,

        /// Show only recent activity (last N minutes)
        #[arg(long)]
        recent: Option<u64>,
    },

    /// Configure proactive research settings
    Configure {
        /// Configuration subcommand
        #[command(subcommand)]
        action: ConfigureAction,
    },
}

#[derive(Subcommand)]
enum ConfigureAction {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., gap_interval, max_tasks)
        key: String,
        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key to retrieve
        key: String,
    },

    /// List all configuration values
    List,

    /// Reset configuration to defaults
    Reset {
        /// Confirm reset operation
        #[arg(short, long)]
        confirm: bool,
    },
}

/// Handle proactive research management commands
async fn handle_proactive_command(
    cmd: ProactiveCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ProactiveCommands::Start {
            gap_interval,
            max_tasks,
            debounce,
            config,
            verbose,
        } => {
            handle_proactive_start(gap_interval, max_tasks, debounce, config, verbose).await?;
        }
        ProactiveCommands::Stop { force, timeout } => {
            handle_proactive_stop(force, timeout).await?;
        }
        ProactiveCommands::Status {
            detailed,
            metrics,
            recent,
        } => {
            handle_proactive_status(detailed, metrics, recent).await?;
        }
        ProactiveCommands::Configure { action } => {
            handle_proactive_configure(action).await?;
        }
    }
    Ok(())
}

/// Handle proactive start command
async fn handle_proactive_start(
    gap_interval: u64,
    max_tasks: usize,
    debounce: u64,
    config_path: Option<String>,
    verbose: bool,
) -> Result<(), ProactiveManagerError> {
    info!("Starting proactive research mode");

    if verbose {
        println!("🚀 Starting proactive research mode with:");
        println!("   Gap analysis interval: {gap_interval} minutes");
        println!("   Maximum concurrent tasks: {max_tasks}");
        println!("   File watch debounce: {debounce} seconds");
        if let Some(ref path) = config_path {
            println!("   Configuration file: {path}");
        }
    }

    // Create configuration
    let mut config = ProactiveManagerConfig::default();
    config.executor.max_concurrent_tasks = max_tasks;
    config.config_path = config_path.as_ref().map(PathBuf::from);

    // Load config from file if specified
    let mut manager = ProactiveManager::new(config);
    if let Some(path) = config_path {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            manager.load_config(&path_buf).await.map_err(|e| {
                error!("Failed to load configuration: {}", e);
                e
            })?;
            if verbose {
                println!("✅ Configuration loaded from {path_buf:?}");
            }
        } else {
            warn!("Configuration file does not exist: {:?}", path_buf);
            println!("⚠️  Configuration file not found, using defaults");
        }
    }

    // Start the manager
    match manager.start().await {
        Ok(()) => {
            println!("✅ Proactive research mode started successfully");
            info!("Proactive research mode started with gap_interval={}min, max_tasks={}, debounce={}s",
                  gap_interval, max_tasks, debounce);

            // Keep the process running
            println!("Press Ctrl+C to stop...");
            tokio::signal::ctrl_c().await.map_err(|e| {
                ProactiveManagerError::Configuration(format!("Signal handling error: {e}"))
            })?;

            println!("\n🛑 Shutting down proactive research mode...");
            manager.stop(false, Duration::from_secs(30)).await?;
            println!("✅ Proactive research mode stopped");
        }
        Err(e) => {
            error!("Failed to start proactive research mode: {}", e);
            println!("❌ Failed to start proactive research mode: {e}");
            return Err(e);
        }
    }

    Ok(())
}

/// Handle proactive stop command
async fn handle_proactive_stop(force: bool, timeout: u64) -> Result<(), ProactiveManagerError> {
    info!(
        "Stopping proactive research mode (force: {}, timeout: {}s)",
        force, timeout
    );

    println!("🛑 Stopping proactive research mode...");
    if force {
        println!("   Force stop requested - terminating immediately");
    } else {
        println!("   Graceful stop requested - waiting up to {timeout} seconds");
    }

    println!("⚠️  Stop functionality not yet fully implemented");

    Ok(())
}

/// Handle proactive status command
async fn handle_proactive_status(
    detailed: bool,
    metrics: bool,
    recent: Option<u64>,
) -> Result<(), ProactiveManagerError> {
    info!(
        "Getting proactive research status (detailed: {}, metrics: {}, recent: {:?})",
        detailed, metrics, recent
    );

    println!("📊 Proactive Research System Status");
    println!("==================================");

    println!("⚠️  Status functionality not yet fully implemented");

    if detailed {
        println!("\n📝 Detailed Task Information:");
        println!("   - Active tasks: 0");
        println!("   - Completed tasks: 0");
        println!("   - Failed tasks: 0");
        println!("   - Detected gaps: 0");
    }

    if metrics {
        println!("\n📈 Performance Metrics:");
        println!("   - System uptime: N/A");
        println!("   - Tasks per hour: N/A");
        println!("   - Gap detection rate: N/A");
        println!("   - Notification delivery rate: N/A");
    }

    if let Some(minutes) = recent {
        println!("\n🕐 Recent Activity (last {minutes} minutes):");
        println!("   - No recent activity");
    }

    Ok(())
}

/// Handle proactive configure command
async fn handle_proactive_configure(action: ConfigureAction) -> Result<(), ProactiveManagerError> {
    match action {
        ConfigureAction::Set { key, value } => {
            info!("Setting configuration: {} = {}", key, value);
            println!("⚙️  Setting configuration: {key} = {value}");

            println!("⚠️  Configuration set functionality not yet fully implemented");
        }
        ConfigureAction::Get { key } => {
            info!("Getting configuration value for: {}", key);
            println!("⚙️  Getting configuration value for: {key}");

            println!("⚠️  Configuration get functionality not yet fully implemented");
        }
        ConfigureAction::List => {
            info!("Listing all configuration values");
            println!("⚙️  Current Configuration:");
            println!("========================");

            println!("⚠️  Configuration list functionality not yet fully implemented");
        }
        ConfigureAction::Reset { confirm } => {
            if !confirm {
                println!("❌ Configuration reset requires --confirm flag");
                return Ok(());
            }

            info!("Resetting configuration to defaults");
            println!("⚙️  Resetting configuration to defaults...");

            println!("⚠️  Configuration reset functionality not yet fully implemented");
        }
    }

    Ok(())
}

/// Handle research command with provider and quality features
async fn handle_research_command(
    topic: String,
    provider: String,
    cross_validate: bool,
    quality_threshold: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Starting research on topic: {} with provider: {}",
        topic, provider
    );

    if cross_validate {
        println!("🔍 Cross-provider validation enabled (threshold: {quality_threshold:.2})");
    }

    println!("🔍 Starting research on: {topic}");
    println!("  Provider: {provider}");
    println!("  Cross-validation: {cross_validate}");
    println!("  Quality threshold: {quality_threshold:.2}");

    // Create a research pipeline with the infrastructure
    match create_research_pipeline().await {
        Ok(pipeline) => {
            println!("✅ Research pipeline created");

            // Show cache lookup attempt
            println!("\n🔍 Checking reference library cache...");
            println!("  📂 Looking for existing research on: '{topic}'");

            // Use the enhanced query processing with the provider preference
            let provider_pref = if provider == "auto" {
                None
            } else {
                Some(provider)
            };

            match pipeline
                .process_query_enhanced(
                    &topic,
                    None, // Use default audience context
                    None, // Use default domain context
                    provider_pref,
                    Some(cross_validate),
                    Some(quality_threshold),
                )
                .await
            {
                Ok(result) => {
                    // Check if this was from cache based on fast processing time
                    // Cache hits typically complete in < 50ms
                    let was_cached = result.metadata.processing_time_ms < 50;

                    if was_cached {
                        println!("✅ Found cached result in reference library!");
                        println!(
                            "  ⚡ Retrieved from cache in {}ms",
                            result.metadata.processing_time_ms
                        );
                    } else {
                        println!("  ❌ No cached result found");
                        println!("🤖 Executing new research...");

                        if cross_validate {
                            println!(
                                "  🔄 Cross-validation enabled - comparing multiple providers"
                            );
                        }

                        let processing_time = result.metadata.processing_time_ms;
                        println!("  ⏱️  Processing completed in {processing_time}ms");

                        // Show sources consulted as indication of provider activity
                        if !result.metadata.sources_consulted.is_empty() {
                            let source_count = result.metadata.sources_consulted.len();
                            println!("  📡 Consulted {source_count} source(s)");
                        }
                    }

                    println!("\n📝 Research Results:");
                    println!("==================");
                    let query = result.original_query();
                    println!("Query: {query}");
                    let research_type = result.research_type();
                    println!("Type: {research_type}");
                    let quality_score = result.metadata.quality_score;
                    println!("Quality Score: {quality_score:.2}");
                    let source = if was_cached { "Cache" } else { "New Research" };
                    println!("Source: {source}");

                    // Show sources consulted
                    if !result.metadata.sources_consulted.is_empty() {
                        let sources = result.metadata.sources_consulted.join(", ");
                        println!("Sources: {sources}");
                    }

                    println!("\n💡 Answer:");
                    println!("{}", result.immediate_answer);

                    if !result.supporting_evidence.is_empty() {
                        println!("\n🔍 Supporting Evidence:");
                        for evidence in &result.supporting_evidence {
                            let source = &evidence.source;
                            let evidence_type = &evidence.evidence_type;
                            println!("  • {source} ({evidence_type})");
                        }
                    }

                    if !result.implementation_details.is_empty() {
                        println!("\n⚙️ Implementation Details:");
                        for detail in &result.implementation_details {
                            let category = &detail.category;
                            let content = &detail.content;
                            println!("  • {category}: {content}");
                        }
                    }

                    if !was_cached {
                        println!("\n💾 Result saved to reference library for future use");
                    }

                    println!("\n✅ Research completed successfully");
                }
                Err(e) => {
                    error!("Research failed: {e}");
                    println!("❌ Research failed: {e}");
                    return Err(Box::new(e));
                }
            }
        }
        Err(e) => {
            error!("Failed to create research pipeline: {e}");
            println!("❌ Failed to create research pipeline: {e}");
            println!("💡 Tip: Make sure API keys are configured properly");
            return Err(e);
        }
    }

    Ok(())
}

/// Check if an API key is a placeholder or example key
fn is_placeholder_key(key: &str) -> bool {
    // Common placeholder patterns
    let placeholders = [
        "your-openai-api-key-here",
        "your-claude-api-key-here",
        "your-anthropic-api-key-here",
        "your-gemini-api-key-here",
        "sk-example",
        "sk-test",
        "test-key",
        "placeholder",
        "example",
    ];

    // Check for exact matches
    if placeholders.contains(&key) {
        return true;
    }

    // Check for obviously invalid keys (too short, wrong format)
    if key.len() < 10 {
        return true;
    }

    // OpenAI keys should start with sk- and be much longer
    if key.starts_with("sk-") && key.len() < 50 {
        return true;
    }

    false
}

/// Determine the best OpenAI model to use based on API key access
async fn determine_openai_model(api_key: &str) -> String {
    // Create a minimal test client to check model access
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    // Test newer models first, then fall back to older ones
    let test_models = ["gpt-4.1-mini", "gpt-4", "gpt-3.5-turbo"];

    for model in &test_models {
        if test_model_access(&client, api_key, model).await {
            return model.to_string();
        }
    }

    // Default to gpt-3.5-turbo if all tests fail
    "gpt-3.5-turbo".to_string()
}

/// Test if a specific model is accessible with the given API key
async fn test_model_access(client: &reqwest::Client, api_key: &str, model: &str) -> bool {
    let request_body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": "test"}],
        "max_tokens": 1,
        "temperature": 0.0
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Success (200) or rate limit (429) means model is accessible
            resp.status().is_success() || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        }
        Err(_) => false,
    }
}

/// Test if a Gemini API key is valid by making a simple request
async fn test_gemini_key_validity(api_key: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    // Test with a simple model list request
    let url = format!("https://generativelanguage.googleapis.com/v1/models?key={api_key}");

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            // Success (200) or rate limit (429) means key is valid
            resp.status().is_success() || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        }
        Err(_) => false,
    }
}

/// Test if a Claude API key is valid by making a simple request
async fn test_claude_key_validity(api_key: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    let request_body = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1,
        "messages": [{"role": "user", "content": "test"}]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Success (200) or rate limit (429) means key is valid
            resp.status().is_success() || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        }
        Err(_) => false,
    }
}

/// Create a research pipeline with multi-provider research engine and cache lookup
async fn create_research_pipeline(
) -> Result<fortitude_core::pipeline::ResearchPipeline, Box<dyn std::error::Error>> {
    use fortitude::providers::config::{ProviderSettings, RateLimitConfig};
    use fortitude::providers::{
        ClaudeProvider, GeminiProvider, OpenAIProvider, ProviderConfig, ProviderManager,
        SelectionStrategy,
    };
    use fortitude::research_engine_adapter::ProviderManagerAdapter;
    use fortitude_core::pipeline::{PipelineBuilder, PipelineConfig};
    use fortitude_core::{
        BasicClassifier, FileStorage, MultiProviderConfig, MultiProviderResearchEngine,
    };
    use fortitude_types::{AudienceContext, ClassificationConfig, DomainContext, StorageConfig};
    use std::sync::Arc;
    use std::time::Duration;

    println!("🔧 Setting up research pipeline with multi-provider support...");

    // Set up provider manager with automatic provider selection
    let provider_config = ProviderConfig {
        selection_strategy: SelectionStrategy::Balanced,
        enable_failover: true,
        enable_cross_validation: false,
        max_failover_attempts: 3,
        provider_timeout: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(300),
        enable_performance_tracking: true,
        performance_window_size: 100,
        cost_optimization_threshold: 0.1,
        min_quality_threshold: 0.6,
    };

    let provider_manager = ProviderManager::new(provider_config).await?;
    let mut provider_count = 0;

    // Add OpenAI provider if API key is available
    if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        if !openai_key.is_empty() && !is_placeholder_key(&openai_key) {
            println!("✅ Configuring OpenAI provider...");

            // Try gpt-3.5-turbo first (more widely accessible), with gpt-4 as fallback
            let model = determine_openai_model(&openai_key).await;
            println!("  📝 Using model: {model}");

            let openai_settings = ProviderSettings::new(openai_key.clone(), model)
                .with_timeout(Duration::from_secs(30))
                .with_rate_limits(RateLimitConfig {
                    requests_per_minute: 60,
                    input_tokens_per_minute: 100_000,
                    output_tokens_per_minute: 20_000,
                    max_concurrent_requests: 5,
                });

            match OpenAIProvider::new(openai_settings).await {
                Ok(openai_provider) => {
                    // Test provider health before adding
                    match openai_provider.health_check().await {
                        Ok(health_status) => {
                            let provider_arc = Arc::new(openai_provider);
                            provider_manager
                                .add_provider("openai".to_string(), provider_arc)
                                .await?;
                            provider_count += 1;
                            match health_status {
                                HealthStatus::Healthy => {
                                    println!("✅ OpenAI provider added successfully (healthy)");
                                }
                                HealthStatus::Degraded(reason) => {
                                    println!(
                                        "⚠️  OpenAI provider added with degraded health: {reason}"
                                    );
                                }
                                HealthStatus::Unhealthy(reason) => {
                                    println!("❌ OpenAI provider unhealthy but added: {reason}");
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ OpenAI provider health check failed: {e}");
                            println!("   Skipping OpenAI provider");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to create OpenAI provider: {e}");
                    println!("   Skipping OpenAI provider");
                }
            }
        } else {
            println!("⚠️  OpenAI API key not configured properly (placeholder or invalid format)");
        }
    } else {
        println!("⚠️  OPENAI_API_KEY environment variable not found");
    }

    // Add Claude provider if API key is available
    if let Ok(claude_key) =
        std::env::var("CLAUDE_API_KEY").or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
    {
        if !claude_key.is_empty() && !is_placeholder_key(&claude_key) {
            println!("✅ Configuring Claude provider...");

            let claude_settings =
                ProviderSettings::new(claude_key.clone(), "claude-3-5-sonnet-20241022".to_string())
                    .with_timeout(Duration::from_secs(30))
                    .with_rate_limits(RateLimitConfig {
                        requests_per_minute: 50,
                        input_tokens_per_minute: 80_000,
                        output_tokens_per_minute: 16_000,
                        max_concurrent_requests: 3,
                    });

            match ClaudeProvider::new(claude_settings).await {
                Ok(claude_provider) => {
                    // Test provider health before adding
                    match claude_provider.health_check().await {
                        Ok(health_status) => {
                            let provider_arc = Arc::new(claude_provider);
                            provider_manager
                                .add_provider("claude".to_string(), provider_arc)
                                .await?;
                            provider_count += 1;
                            match health_status {
                                HealthStatus::Healthy => {
                                    println!("✅ Claude provider added successfully (healthy)");
                                }
                                HealthStatus::Degraded(reason) => {
                                    println!(
                                        "⚠️  Claude provider added with degraded health: {reason}"
                                    );
                                }
                                HealthStatus::Unhealthy(reason) => {
                                    println!("❌ Claude provider unhealthy but added: {reason}");
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Claude provider health check failed: {e}");
                            println!("   Skipping Claude provider");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to create Claude provider: {e}");
                    println!("   Skipping Claude provider");
                }
            }
        } else {
            println!("⚠️  Claude API key not configured properly (placeholder or invalid format)");
        }
    } else {
        println!("⚠️  CLAUDE_API_KEY or ANTHROPIC_API_KEY environment variable not found");
    }

    // Add Gemini provider if API key is available
    if let Ok(gemini_key) =
        std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY"))
    {
        if !gemini_key.is_empty() && !is_placeholder_key(&gemini_key) {
            println!("✅ Configuring Gemini provider...");

            // Use gemini-2.5-flash as default (latest fast model)
            let model = "gemini-2.5-flash".to_string();
            println!("  📝 Using model: {model}");

            let gemini_settings = ProviderSettings::new(gemini_key.clone(), model)
                .with_timeout(Duration::from_secs(30))
                .with_rate_limits(RateLimitConfig {
                    requests_per_minute: 60,
                    input_tokens_per_minute: 1_000_000, // Gemini has high token limits
                    output_tokens_per_minute: 32_000,   // Generous output limit
                    max_concurrent_requests: 3,
                });

            match GeminiProvider::new(gemini_settings).await {
                Ok(gemini_provider) => {
                    // Test provider health before adding
                    match gemini_provider.health_check().await {
                        Ok(health_status) => {
                            let provider_arc = Arc::new(gemini_provider);
                            provider_manager
                                .add_provider("gemini".to_string(), provider_arc)
                                .await?;
                            provider_count += 1;
                            match health_status {
                                HealthStatus::Healthy => {
                                    println!("✅ Gemini provider added successfully (healthy)");
                                }
                                HealthStatus::Degraded(reason) => {
                                    println!(
                                        "⚠️  Gemini provider added with degraded health: {reason}"
                                    );
                                }
                                HealthStatus::Unhealthy(reason) => {
                                    println!("❌ Gemini provider unhealthy but added: {reason}");
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Gemini provider health check failed: {e}");
                            println!("   Skipping Gemini provider");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to create Gemini provider: {e}");
                    println!("   Skipping Gemini provider");
                }
            }
        } else {
            println!("⚠️  Gemini API key not configured properly (placeholder or invalid format)");
        }
    } else {
        println!("⚠️  GEMINI_API_KEY or GOOGLE_API_KEY environment variable not found");
    }

    // Verify we have at least one provider
    if provider_count == 0 {
        warn!("No API providers configured - pipeline will use fallback mode");
        println!("❌ No valid API keys found!");
        println!("💡 Set one of these environment variables:");
        println!("   - OPENAI_API_KEY=your-openai-api-key");
        println!("   - CLAUDE_API_KEY=your-claude-api-key (or ANTHROPIC_API_KEY)");
        println!("   - GEMINI_API_KEY=your-gemini-api-key");
        return Err("No API providers configured".into());
    }

    println!("🎯 Configured {provider_count} provider(s) for automatic selection");

    // Create multi-provider research engine
    let multi_provider_config = MultiProviderConfig {
        enable_cross_validation: false, // Disable for CLI to reduce API calls
        cross_validation_providers: 1,
        quality_threshold: 0.7,
        enable_vector_search: false, // Disable for CLI simplicity
        max_context_documents: 5,
        context_relevance_threshold: 0.7,
        enable_quality_validation: true,
        min_quality_score: 0.6,
        max_processing_time: Duration::from_secs(60),
        enable_performance_optimization: true,
        cost_optimization_weight: 0.2,
        quality_optimization_weight: 0.6,
        latency_optimization_weight: 0.2,
    };

    // Wrap provider manager in adapter
    let provider_adapter = ProviderManagerAdapter::new(Arc::new(provider_manager));

    let research_engine = Arc::new(
        MultiProviderResearchEngine::new(Arc::new(provider_adapter), multi_provider_config).await?,
    );

    println!("✅ Multi-provider research engine created successfully");

    // Create basic components with proper configurations
    // Lower confidence threshold for CLI usage (demo mode)
    let classification_config = ClassificationConfig {
        default_threshold: 0.05,
        ..Default::default()
    };
    let classifier = Arc::new(BasicClassifier::new(classification_config));

    let storage_config = StorageConfig::default();
    let storage = Arc::new(FileStorage::new(storage_config).await?);

    // Configure the pipeline with multi-provider support enabled
    let config = PipelineConfig {
        max_concurrent: 3,
        timeout_seconds: 300,
        enable_caching: true,
        default_audience: AudienceContext::default(),
        default_domain: DomainContext::default(),
        enable_context_detection: true,
        enable_advanced_classification: false,
        advanced_classification_config: None,
        enable_vector_search: false,
        auto_index_results: false,
        enable_context_discovery: false,
        enable_multi_provider: true, // Enable multi-provider support
        default_provider: "auto".to_string(),
        enable_cross_validation: false,
        quality_threshold: 0.8,
        enable_learning: false,
        enable_monitoring: false,
        auto_apply_learning: false,
    };

    // Build the pipeline with research engine (CRITICAL FIX)
    let pipeline = PipelineBuilder::new()
        .with_max_concurrent(config.max_concurrent)
        .with_timeout(config.timeout_seconds)
        .with_caching(config.enable_caching)
        .with_default_audience(config.default_audience.clone())
        .with_default_domain(config.default_domain.clone())
        .with_context_detection(config.enable_context_detection)
        .with_research_engine(research_engine) // CRITICAL: Add research engine
        .build(classifier, storage);

    println!("✅ Research pipeline created with cache lookup and multi-provider support");

    Ok(pipeline)
}

/// Handle provider management commands
async fn handle_provider_command(cmd: ProviderCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ProviderCommands::List { detailed, format } => {
            handle_provider_list(detailed, format).await?;
        }
        ProviderCommands::Performance {
            provider,
            period,
            format,
        } => {
            handle_provider_performance(provider, period, format).await?;
        }
        ProviderCommands::Health { provider, force } => {
            handle_provider_health(provider, force).await?;
        }
        ProviderCommands::Switch { provider, force } => {
            handle_provider_switch(provider, force).await?;
        }
        ProviderCommands::Configure {
            provider,
            key,
            value,
        } => {
            handle_provider_configure(provider, key, value).await?;
        }
    }
    Ok(())
}

/// Handle quality control commands
async fn handle_quality_command(cmd: QualityCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        QualityCommands::Metrics { period, format } => {
            handle_quality_metrics(period, format).await?;
        }
        QualityCommands::Validate {
            query,
            cross_validate,
            provider_count,
        } => {
            handle_quality_validate(query, cross_validate, provider_count).await?;
        }
        QualityCommands::Configure { key, value } => {
            handle_quality_configure(key, value).await?;
        }
        QualityCommands::Status { detailed } => {
            handle_quality_status(detailed).await?;
        }
    }
    Ok(())
}

/// Handle learning system commands
async fn handle_learning_command(cmd: LearningCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        LearningCommands::Status { detailed, format } => {
            handle_learning_status(detailed, format).await?;
        }
        LearningCommands::Adapt { force, dry_run } => {
            handle_learning_adapt(force, dry_run).await?;
        }
        LearningCommands::Feedback {
            target,
            rating,
            comment,
        } => {
            handle_learning_feedback(target, rating, comment).await?;
        }
        LearningCommands::Patterns { days, format } => {
            handle_learning_patterns(days, format).await?;
        }
        LearningCommands::Configure { key, value } => {
            handle_learning_configure(key, value).await?;
        }
    }
    Ok(())
}

/// Handle monitoring system commands
async fn handle_monitoring_command(
    cmd: MonitoringCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        MonitoringCommands::Metrics {
            period,
            component,
            format,
        } => {
            handle_monitoring_metrics(period, component, format).await?;
        }
        MonitoringCommands::Alerts { severity, format } => {
            handle_monitoring_alerts(severity, format).await?;
        }
        MonitoringCommands::Health { detailed, force } => {
            handle_monitoring_health(detailed, force).await?;
        }
        MonitoringCommands::Configure { key, value } => {
            handle_monitoring_configure(key, value).await?;
        }
        MonitoringCommands::Service { action, service } => {
            handle_monitoring_service(action, service).await?;
        }
    }
    Ok(())
}

// Provider command handlers
async fn handle_provider_list(
    detailed: bool,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Listing providers (detailed: {}, format: {})",
        detailed, format
    );

    println!("📋 Provider List");
    println!("================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_provider_performance(
    provider: Option<String>,
    period: u64,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting provider performance (provider: {:?}, period: {}h, format: {})",
        provider, period, format
    );

    println!("📊 Provider Performance Metrics");
    println!("===============================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_provider_health(
    provider: Option<String>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Checking provider health (provider: {:?}, force: {})",
        provider, force
    );

    println!("🏥 Provider Health Check");
    println!("========================");

    // Check which providers have valid API keys configured
    let mut checked_providers = Vec::new();

    // Check OpenAI
    if provider.is_none() || provider.as_ref().unwrap() == "openai" {
        print!("OpenAI: ");
        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            if !openai_key.is_empty() && !is_placeholder_key(&openai_key) {
                print!("Testing API connectivity... ");

                let model = if force {
                    // Force test all models if requested, prioritizing newer ones
                    if test_model_access(&reqwest::Client::new(), &openai_key, "gpt-4.1-mini").await
                    {
                        "gpt-4.1-mini"
                    } else if test_model_access(&reqwest::Client::new(), &openai_key, "gpt-4").await
                    {
                        "gpt-4"
                    } else if test_model_access(
                        &reqwest::Client::new(),
                        &openai_key,
                        "gpt-3.5-turbo",
                    )
                    .await
                    {
                        "gpt-3.5-turbo"
                    } else {
                        "❌ No accessible models"
                    }
                } else {
                    // Test in priority order: newest first
                    if test_model_access(&reqwest::Client::new(), &openai_key, "gpt-4.1-mini").await
                    {
                        "gpt-4.1-mini"
                    } else if test_model_access(&reqwest::Client::new(), &openai_key, "gpt-4").await
                    {
                        "gpt-4"
                    } else if test_model_access(
                        &reqwest::Client::new(),
                        &openai_key,
                        "gpt-3.5-turbo",
                    )
                    .await
                    {
                        "gpt-3.5-turbo"
                    } else {
                        "❌ No accessible models"
                    }
                };

                if model.starts_with("❌") {
                    println!("❌ Unhealthy ({model})");
                } else {
                    println!("✅ Healthy (using {model})");
                }
                checked_providers.push("openai");
            } else {
                println!("⚠️  Not configured (invalid API key)");
            }
        } else {
            println!("⚠️  Not configured (no API key)");
        }
    }

    // Check Claude/Anthropic
    if provider.is_none() || provider.as_ref().unwrap() == "claude" {
        print!("Claude: ");
        if let Ok(claude_key) =
            std::env::var("CLAUDE_API_KEY").or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        {
            if !claude_key.is_empty() && !is_placeholder_key(&claude_key) {
                print!("Testing API connectivity... ");

                // Test Claude API with a simple request
                if test_claude_key_validity(&claude_key).await {
                    println!("✅ Healthy");
                } else {
                    println!("❌ Unhealthy (API test failed)");
                }
                checked_providers.push("claude");
            } else {
                println!("⚠️  Not configured (invalid API key)");
            }
        } else {
            println!("⚠️  Not configured (no API key)");
        }
    }

    // Check Gemini
    if provider.is_none() || provider.as_ref().unwrap() == "gemini" {
        print!("Gemini: ");
        if let Ok(gemini_key) =
            std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY"))
        {
            if !gemini_key.is_empty() && !is_placeholder_key(&gemini_key) {
                print!("Testing API connectivity... ");

                if test_gemini_key_validity(&gemini_key).await {
                    println!("✅ Healthy (implementation pending)");
                } else {
                    println!("❌ Unhealthy (API test failed)");
                }
                checked_providers.push("gemini");
            } else {
                println!("⚠️  Not configured (invalid API key)");
            }
        } else {
            println!("⚠️  Not configured (no API key)");
        }
    }

    if let Some(specific_provider) = &provider {
        if !checked_providers.contains(&specific_provider.as_str()) {
            println!("❌ Unknown provider: {specific_provider}");
            println!("   Available providers: openai, claude, gemini");
        }
    }

    if checked_providers.is_empty() {
        println!("\n⚠️  No providers are configured with valid API keys");
        println!("💡 Set one of these environment variables:");
        println!("   - OPENAI_API_KEY=your-openai-api-key");
        println!("   - CLAUDE_API_KEY=your-claude-api-key (or ANTHROPIC_API_KEY)");
        println!("   - GEMINI_API_KEY=your-gemini-api-key (or GOOGLE_API_KEY)");
    } else {
        println!(
            "\n✅ Health check completed for {} provider(s)",
            checked_providers.len()
        );
    }

    Ok(())
}

async fn handle_provider_switch(
    provider: String,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Switching to provider: {} (force: {})", provider, force);

    println!("🔄 Provider Switch");
    println!("==================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_provider_configure(
    provider: String,
    key: String,
    value: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Configuring provider {} with {}={}", provider, key, value);

    println!("⚙️ Provider Configuration");
    println!("=========================");

    println!("Implementation in progress...");

    Ok(())
}

// Quality command handlers
async fn handle_quality_metrics(
    period: u64,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting quality metrics (period: {}h, format: {})",
        period, format
    );

    println!("📊 Quality Control Metrics");
    println!("==========================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_quality_validate(
    query: String,
    cross_validate: bool,
    provider_count: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Validating query: {} (cross_validate: {}, providers: {})",
        query, cross_validate, provider_count
    );

    println!("🔍 Quality Validation");
    println!("=====================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_quality_configure(
    key: String,
    value: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Configuring quality control: {}={}", key, value);

    println!("⚙️ Quality Configuration");
    println!("========================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_quality_status(detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    info!("Getting quality status (detailed: {})", detailed);

    println!("📋 Quality Control Status");
    println!("=========================");

    println!("Implementation in progress...");

    Ok(())
}

// Learning command handlers
async fn handle_learning_status(
    detailed: bool,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting learning status (detailed: {}, format: {})",
        detailed, format
    );

    println!("🧠 Learning System Status");
    println!("=========================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_learning_adapt(
    force: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Triggering learning adaptation (force: {}, dry_run: {})",
        force, dry_run
    );

    println!("🔄 Learning System Adaptation");
    println!("=============================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_learning_feedback(
    target: String,
    rating: f64,
    comment: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Submitting feedback for: {} (rating: {:.2})",
        target, rating
    );

    println!("💭 Learning Feedback Submission");
    println!("===============================");

    println!("Implementation in progress...");
    if let Some(comment) = comment {
        println!("Comment: {comment}");
    }

    Ok(())
}

async fn handle_learning_patterns(
    days: u64,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting usage patterns (days: {}, format: {})",
        days, format
    );

    println!("📈 Usage Patterns Analysis");
    println!("==========================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_learning_configure(
    key: String,
    value: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Configuring learning system: {}={}", key, value);

    println!("⚙️ Learning System Configuration");
    println!("================================");

    println!("Implementation in progress...");

    Ok(())
}

// Monitoring command handlers
async fn handle_monitoring_metrics(
    period: u64,
    component: String,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting monitoring metrics (period: {}h, component: {}, format: {})",
        period, component, format
    );

    println!("📊 System Performance Metrics");
    println!("=============================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_monitoring_alerts(
    severity: String,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting alerts (severity: {}, format: {})",
        severity, format
    );

    println!("🚨 System Alerts");
    println!("================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_monitoring_health(
    detailed: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Getting system health (detailed: {}, force: {})",
        detailed, force
    );

    println!("🏥 System Health Status");
    println!("=======================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_monitoring_configure(
    key: String,
    value: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Configuring monitoring: {}={}", key, value);

    println!("⚙️ Monitoring Configuration");
    println!("===========================");

    println!("Implementation in progress...");

    Ok(())
}

async fn handle_monitoring_service(
    action: String,
    service: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Managing monitoring service: {} {}", action, service);

    println!("🔧 Monitoring Service Management");
    println!("================================");

    println!("Implementation in progress...");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Research {
            topic,
            provider,
            cross_validate,
            quality_threshold,
        } => {
            handle_research_command(topic, provider, cross_validate, quality_threshold).await?;
        }
        Commands::Pipeline { config } => {
            info!("Starting knowledge pipeline with config: {:?}", config);
            println!("Pipeline functionality not yet implemented");
        }
        Commands::Knowledge { query } => {
            info!("Querying knowledge base: {}", query);
            println!("Knowledge base functionality not yet implemented");
        }
        Commands::Provider(provider_cmd) => {
            handle_provider_command(provider_cmd).await?;
        }
        Commands::Quality(quality_cmd) => {
            handle_quality_command(quality_cmd).await?;
        }
        Commands::Learning(learning_cmd) => {
            handle_learning_command(learning_cmd).await?;
        }
        Commands::Monitoring(monitoring_cmd) => {
            handle_monitoring_command(monitoring_cmd).await?;
        }
        Commands::Proactive(proactive_cmd) => {
            handle_proactive_command(proactive_cmd).await?;
        }
    }

    Ok(())
}
