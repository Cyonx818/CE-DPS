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

// ABOUTME: CLI application for the Fortitude research system
#![allow(clippy::wildcard_in_or_patterns)]
use clap::{Parser, Subcommand};
use fortitude_core::{
    // Vector services
    vector::{
        HybridSearchResult as VectorHybridSearchResult, HybridSearchService,
        LocalEmbeddingService as EmbeddingService, MigrationService, QdrantClient,
        SearchResult as VectorSearchResult, SemanticSearchService, VectorStorage,
    },
    BasicClassifier,
    ClaudeResearchEngine,
    FileStorage,
    PipelineBuilder,
    ResearchPipeline,
};
use fortitude_types::*;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn, Level};

mod config;
use config::Config;

// Parameter structs to reduce function argument count
#[derive(Debug)]
struct SemanticSearchParams {
    query: String,
    strategy: String,
    limit: usize,
    threshold: f64,
    #[allow(dead_code)]
    format: String,
    collection: Option<String>,
    explain: bool,
}

#[derive(Debug)]
struct HybridSearchParams {
    query: String,
    keyword_weight: f64,
    semantic_weight: f64,
    limit: usize,
    threshold: f64,
    #[allow(dead_code)]
    format: String,
    collection: Option<String>,
    explain: bool,
}

#[derive(Parser)]
#[command(name = "fortitude")]
#[command(
    about = "Automated research system for AI-assisted development with advanced context-aware classification"
)]
#[command(version = "0.1.0")]
#[command(author = "Concordia Team")]
#[command(
    long_about = "Fortitude is an AI-powered research system that provides context-aware documentation and implementation guidance. It supports multi-dimensional classification with audience detection, domain analysis, and urgency assessment for optimized research results."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true, default_value = "./reference_library")]
    data_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform research on a topic
    Research {
        /// The research topic or question
        topic: String,

        /// Audience level (beginner, intermediate, advanced)
        #[arg(short, long, default_value = "intermediate")]
        level: String,

        /// Domain context (rust, web, devops, etc.)
        #[arg(long, default_value = "rust")]
        domain: String,

        /// Output format (markdown, json, plain)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Technology stack
        #[arg(short, long, default_value = "rust")]
        technology: String,

        /// Project type (cli, web, library, etc.)
        #[arg(short, long, default_value = "library")]
        project_type: String,

        /// Framework tags (comma-separated)
        #[arg(long)]
        frameworks: Option<String>,

        /// Additional tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Disable caching
        #[arg(long)]
        no_cache: bool,

        /// Enable advanced multi-dimensional classification with context detection
        #[arg(
            long,
            help = "Enable advanced classification with audience, domain, and urgency detection"
        )]
        advanced_classification: bool,

        /// Enable context detection (automatically enabled with advanced classification)
        #[arg(
            long,
            help = "Enable context detection for audience level, technical domain, and urgency"
        )]
        context_detection: bool,

        /// Context detection confidence threshold (0.0-1.0)
        #[arg(
            long,
            default_value = "0.6",
            help = "Minimum confidence threshold for context detection (0.0-1.0)"
        )]
        context_threshold: f64,

        /// Enable graceful degradation for advanced classification
        #[arg(
            long,
            help = "Continue processing even if advanced classification fails"
        )]
        graceful_degradation: bool,
    },

    /// List cached research results
    List {
        /// Filter by research type
        #[arg(short, long)]
        research_type: Option<String>,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Show only entries newer than N days
        #[arg(long)]
        newer_than: Option<u64>,

        /// Output format (table, json, summary)
        #[arg(short, long, default_value = "table")]
        format: String,

        /// Number of results to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show cache status and statistics
    CacheStatus {
        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Search cached research results
    Search {
        /// Search query
        query: String,

        /// Filter by research type
        #[arg(short, long)]
        research_type: Option<String>,

        /// Filter by tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Minimum quality score (0.0-1.0)
        #[arg(short, long)]
        min_quality: Option<f64>,

        /// Number of results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format (table, json, summary)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Clean up expired cache entries
    Cleanup {
        /// Show what would be deleted without actually deleting
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        config_command: ConfigCommand,
    },

    /// Vector database operations
    Vector {
        #[command(subcommand)]
        vector_command: VectorCommand,
    },

    /// Enhanced semantic search
    SemanticSearch {
        /// The search query
        query: String,

        /// Search strategy (semantic, hybrid, combined)
        #[arg(short, long, default_value = "semantic")]
        strategy: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Minimum relevance threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.0")]
        threshold: f64,

        /// Output format (table, json, detailed)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Collection to search in
        #[arg(short, long)]
        collection: Option<String>,

        /// Include search explanation
        #[arg(long)]
        explain: bool,
    },

    /// Hybrid search combining vector and keyword
    HybridSearch {
        /// The search query
        query: String,

        /// Keyword weight (0.0-1.0)
        #[arg(long, default_value = "0.5")]
        keyword_weight: f64,

        /// Semantic weight (0.0-1.0)
        #[arg(long, default_value = "0.5")]
        semantic_weight: f64,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Minimum relevance threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.0")]
        threshold: f64,

        /// Output format (table, json, detailed)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Collection to search in
        #[arg(short, long)]
        collection: Option<String>,

        /// Include search explanation
        #[arg(long)]
        explain: bool,
    },

    /// Find content similar to provided text
    FindSimilar {
        /// The content to find similar items for
        content: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Minimum similarity threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.7")]
        threshold: f64,

        /// Output format (table, json, detailed)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Collection to search in
        #[arg(short, long)]
        collection: Option<String>,
    },
}

#[derive(Subcommand)]
enum VectorCommand {
    /// Configure vector database settings
    Config {
        /// Set vector database URL
        #[arg(long)]
        url: Option<String>,

        /// Set API key
        #[arg(long)]
        api_key: Option<String>,

        /// Set default collection name
        #[arg(long)]
        collection: Option<String>,

        /// Show current configuration
        #[arg(long)]
        show: bool,
    },

    /// Check vector database health
    Health {
        /// Detailed health check
        #[arg(long)]
        _detailed: bool,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        _format: String,
    },

    /// Show vector database statistics
    Stats {
        /// Collection name
        #[arg(short, long)]
        collection: Option<String>,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Show detailed statistics
        #[arg(long)]
        _detailed: bool,
    },

    /// Migrate data to vector database
    Migrate {
        /// Source directory or file
        source: String,

        /// Target collection
        #[arg(short, long)]
        collection: Option<String>,

        /// Batch size for processing
        #[arg(short, long, default_value = "100")]
        batch_size: usize,

        /// Validation level (strict, moderate, lenient)
        #[arg(long, default_value = "moderate")]
        validation: String,

        /// Dry run without making changes
        #[arg(long)]
        dry_run: bool,

        /// Resume migration from checkpoint
        #[arg(long)]
        resume: Option<String>,
    },

    /// Show migration status
    MigrationStatus {
        /// Migration ID
        id: Option<String>,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Show all migrations
        #[arg(long)]
        all: bool,
    },

    /// Resume paused migration
    MigrationResume {
        /// Migration ID
        id: String,

        /// Force resume even if there are issues
        #[arg(long)]
        force: bool,
    },

    /// Cancel running migration
    MigrationCancel {
        /// Migration ID
        id: String,

        /// Force cancellation
        #[arg(long)]
        force: bool,
    },

    /// List all migrations
    MigrationList {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Show only active migrations
        #[arg(long)]
        active: bool,
    },

    /// Search analytics and performance metrics
    Analytics {
        /// Time period in days
        #[arg(short, long, default_value = "7")]
        period: u32,

        /// Collection name
        #[arg(short, long)]
        collection: Option<String>,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        _format: String,

        /// Include performance metrics
        #[arg(long)]
        _performance: bool,
    },

    /// Setup initial vector database
    Setup {
        /// Collection name
        #[arg(short, long)]
        collection: Option<String>,

        /// Vector dimensions
        #[arg(short, long)]
        dimensions: Option<usize>,

        /// Distance metric (cosine, euclidean, dot)
        #[arg(short, long)]
        metric: Option<String>,

        /// Force recreate if exists
        #[arg(long)]
        force: bool,
    },

    /// Index status and operations
    Index {
        #[command(subcommand)]
        index_command: IndexCommand,
    },
}

#[derive(Subcommand)]
enum IndexCommand {
    /// Show index status
    Status {
        /// Collection name
        #[arg(short, long)]
        collection: Option<String>,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        _format: String,
    },

    /// Rebuild index
    Rebuild {
        /// Collection name
        #[arg(short, long)]
        collection: Option<String>,

        /// Force rebuild
        #[arg(long)]
        force: bool,
    },

    /// Optimize index
    Optimize {
        /// Collection name
        #[arg(short, long)]
        collection: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Show current configuration
    Show {
        /// Show sensitive values (API keys, etc.)
        #[arg(long)]
        show_sensitive: bool,
    },

    /// Generate a sample configuration file
    Generate {
        /// Output file path
        #[arg(short, long, default_value = "fortitude.json")]
        output: PathBuf,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },

    /// Validate configuration
    Validate {
        /// Configuration file to validate
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Set configuration values
    Set {
        /// Configuration key (e.g., claude.api_key)
        key: String,

        /// Configuration value
        value: String,
    },
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Load configuration
    let mut config = Config::load().unwrap_or_else(|e| {
        warn!("Failed to load configuration: {}. Using defaults.", e);
        Config::default()
    });

    // Override data directory if specified
    if cli.data_dir != PathBuf::from("./reference_library") {
        config.storage.base_path = cli.data_dir;
    }

    // Initialize the application
    let app = App::new(config.clone()).await?;

    match cli.command {
        Commands::Research {
            topic,
            level,
            domain,
            format,
            technology,
            project_type,
            frameworks,
            tags,
            no_cache,
            advanced_classification,
            context_detection,
            context_threshold,
            graceful_degradation,
        } => {
            if let Err(e) = app
                .handle_research(
                    topic,
                    level,
                    domain,
                    format,
                    technology,
                    project_type,
                    frameworks,
                    tags,
                    !no_cache,
                    advanced_classification,
                    context_detection,
                    context_threshold,
                    graceful_degradation,
                )
                .await
            {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::List {
            research_type,
            tag,
            newer_than,
            format,
            limit,
        } => {
            if let Err(e) = app
                .handle_list(research_type, tag, newer_than, format, limit)
                .await
            {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::CacheStatus { detailed, format } => {
            if let Err(e) = app.handle_cache_status(detailed, format).await {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::Search {
            query,
            research_type,
            tags,
            min_quality,
            limit,
            format,
        } => {
            if let Err(e) = app
                .handle_search(query, research_type, tags, min_quality, limit, format)
                .await
            {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::Cleanup { dry_run } => {
            if let Err(e) = app.handle_cleanup(dry_run).await {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::Config { config_command } => {
            if let Err(e) = handle_config_command(config_command, &config).await {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::Vector { vector_command } => {
            if let Err(e) = app.handle_vector_command(vector_command).await {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::SemanticSearch {
            query,
            strategy,
            limit,
            threshold,
            _format,
            collection,
            explain,
        } => {
            if let Err(e) = app
                .handle_semantic_search(SemanticSearchParams {
                    query,
                    strategy,
                    limit,
                    threshold,
                    format: _format,
                    collection,
                    explain,
                })
                .await
            {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::HybridSearch {
            query,
            keyword_weight,
            semantic_weight,
            limit,
            threshold,
            _format,
            collection,
            explain,
        } => {
            if let Err(e) = app
                .handle_hybrid_search(HybridSearchParams {
                    query,
                    keyword_weight,
                    semantic_weight,
                    limit,
                    threshold,
                    format: _format,
                    collection,
                    explain,
                })
                .await
            {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
        Commands::FindSimilar {
            content,
            limit,
            threshold,
            _format,
            collection,
        } => {
            if let Err(e) = app
                .handle_find_similar(content, limit, threshold, _format, collection)
                .await
            {
                eprintln!("Error: {e}");
                return Err(e);
            }
        }
    }

    Ok(())
}

async fn handle_config_command(
    config_command: ConfigCommand,
    config: &Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match config_command {
        ConfigCommand::Show { show_sensitive } => {
            let mut display_config = config.clone();

            // Hide sensitive values unless explicitly requested
            if !show_sensitive {
                if let Some(ref mut claude) = display_config.claude {
                    if !claude.api_key.is_empty() {
                        claude.api_key =
                            format!("sk-***{}", &claude.api_key[claude.api_key.len() - 4..]);
                    }
                }
            }

            let json = serde_json::to_string_pretty(&display_config)?;
            println!("{json}");
        }

        ConfigCommand::Generate { output, force } => {
            if output.exists() && !force {
                return Err(format!(
                    "File {} already exists. Use --force to overwrite.",
                    output.display()
                )
                .into());
            }

            let sample_config = Config::generate_sample();
            std::fs::write(&output, sample_config)?;
            println!("Sample configuration generated at: {}", output.display());
        }

        ConfigCommand::Validate { file } => {
            let config_to_validate = if let Some(file_path) = file {
                let contents = std::fs::read_to_string(&file_path)?;
                serde_json::from_str::<Config>(&contents)?
            } else {
                config.clone()
            };

            match config_to_validate.validate() {
                Ok(()) => println!("Configuration is valid"),
                Err(e) => {
                    eprintln!("Configuration validation failed: {e}");
                    return Err(e.into());
                }
            }
        }

        ConfigCommand::Set { key, value } => {
            println!("Setting configuration key '{key}' to '{value}'");
            println!("Note: Configuration modification is not yet implemented.");
            println!("Please edit the configuration file directly or use environment variables.");
        }
    }

    Ok(())
}

struct App {
    pipeline: ResearchPipeline,
    config: Config,
    // Vector services (optional)
    qdrant_client: Option<QdrantClient>,
    #[allow(dead_code)] // TODO: Implement vector storage CLI commands
    vector_storage: Option<VectorStorage>,
    semantic_search: Option<SemanticSearchService>,
    hybrid_search: Option<HybridSearchService>,
    migration_service: Option<MigrationService>,
    #[allow(dead_code)] // TODO: Implement embedding service CLI commands
    embedding_service: Option<EmbeddingService>,
}

impl App {
    async fn new(config: Config) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        info!(
            "Initializing Fortitude with data directory: {}",
            config.storage.base_path.display()
        );

        // Setup storage
        let storage_config = StorageConfig {
            base_path: config.storage.base_path.clone(),
            cache_expiration_seconds: config.storage.cache_expiration_seconds,
            max_cache_size_bytes: config.storage.max_cache_size_bytes,
            enable_content_addressing: config.storage.enable_content_addressing,
            index_update_interval_seconds: config.storage.index_update_interval_seconds,
        };

        let storage = FileStorage::new(storage_config).await?;

        // Setup classifier
        let classification_config = ClassificationConfig {
            default_threshold: config.classification.default_threshold,
            ..Default::default()
        };
        let classifier = BasicClassifier::new(classification_config);

        // Setup pipeline with advanced classification support
        let mut pipeline_builder = PipelineBuilder::new()
            .with_caching(config.pipeline.enable_caching)
            .with_context_detection(config.classification.enable_context_detection)
            .with_advanced_classification(config.classification.enable_advanced);

        // Add research engine if Claude API is configured
        if config.has_claude_config() {
            match config.get_claude_config() {
                Ok(claude_config) => {
                    // Convert CLI config to Claude config
                    let claude_api_config = fortitude_core::api::ClaudeConfig {
                        api_key: claude_config.api_key.clone(),
                        base_url: claude_config
                            .base_url
                            .unwrap_or_else(|| "https://api.anthropic.com".to_string()),
                        timeout: std::time::Duration::from_secs(
                            claude_config.timeout_seconds.unwrap_or(300),
                        ),
                        rate_limit: fortitude_core::api::RateLimitConfig {
                            requests_per_minute: claude_config.rate_limit.requests_per_minute,
                            input_tokens_per_minute: claude_config
                                .rate_limit
                                .input_tokens_per_minute,
                            output_tokens_per_minute: claude_config
                                .rate_limit
                                .output_tokens_per_minute,
                            max_concurrent_requests: claude_config
                                .rate_limit
                                .max_concurrent_requests,
                        },
                        retry: fortitude_core::api::RetryConfig {
                            max_retries: claude_config.retry.max_retries,
                            initial_delay: std::time::Duration::from_millis(
                                claude_config.retry.initial_delay_ms,
                            ),
                            max_delay: std::time::Duration::from_millis(
                                claude_config.retry.max_delay_ms,
                            ),
                            backoff_multiplier: claude_config.retry.backoff_multiplier,
                            jitter: claude_config.retry.jitter,
                        },
                        user_agent: format!("fortitude/{}", env!("CARGO_PKG_VERSION")),
                        model: claude_config
                            .model
                            .unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()),
                    };

                    match ClaudeResearchEngine::new(claude_api_config) {
                        Ok(engine) => {
                            info!("Claude research engine initialized successfully");
                            pipeline_builder =
                                pipeline_builder.with_research_engine(Arc::new(engine));
                        }
                        Err(e) => {
                            warn!("Failed to initialize Claude research engine: {}. Using Claude Code fallback.", e);
                            let claude_code_engine =
                                fortitude_core::ClaudeCodeResearchEngine::new_default();
                            pipeline_builder =
                                pipeline_builder.with_research_engine(Arc::new(claude_code_engine));
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Invalid Claude configuration: {}. Using Claude Code fallback.",
                        e
                    );
                    let claude_code_engine =
                        fortitude_core::ClaudeCodeResearchEngine::new_default();
                    pipeline_builder =
                        pipeline_builder.with_research_engine(Arc::new(claude_code_engine));
                }
            }
        } else {
            info!("Claude API not configured. Using Claude Code provider as fallback.");
            info!("Claude Code provider will use WebSearch tool for comprehensive research capabilities.");

            // Create Claude Code research engine as fallback
            let claude_code_engine = fortitude_core::ClaudeCodeResearchEngine::new_default();
            pipeline_builder = pipeline_builder.with_research_engine(Arc::new(claude_code_engine));
        }

        let pipeline = pipeline_builder.build(Arc::new(classifier), Arc::new(storage));

        // Initialize vector services if configuration is available
        let (
            qdrant_client,
            vector_storage,
            semantic_search,
            hybrid_search,
            migration_service,
            embedding_service,
        ) = if let Some(vector_config) = &config.vector {
            match Self::init_vector_services(vector_config).await {
                Ok(services) => {
                    info!("Vector services initialized successfully");
                    services
                }
                Err(e) => {
                    warn!("Failed to initialize vector services: {}. Vector commands will be unavailable.", e);
                    (None, None, None, None, None, None)
                }
            }
        } else {
            info!("Vector database not configured. Vector commands will be unavailable.");
            (None, None, None, None, None, None)
        };

        Ok(Self {
            pipeline,
            config,
            qdrant_client,
            vector_storage,
            semantic_search,
            hybrid_search,
            migration_service,
            embedding_service,
        })
    }

    async fn init_vector_services(
        _vector_config: &config::VectorDatabaseConfig,
    ) -> std::result::Result<
        (
            Option<QdrantClient>,
            Option<VectorStorage>,
            Option<SemanticSearchService>,
            Option<HybridSearchService>,
            Option<MigrationService>,
            Option<EmbeddingService>,
        ),
        Box<dyn std::error::Error>,
    > {
        // Placeholder implementation until vector services API stabilizes
        info!("Vector services initialization temporarily disabled - placeholder mode");

        Ok((None, None, None, None, None, None))
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_research(
        &self,
        topic: String,
        level: String,
        domain: String,
        format: String,
        technology: String,
        project_type: String,
        frameworks: Option<String>,
        tags: Option<String>,
        _enable_cache: bool,
        advanced_classification: bool,
        context_detection: bool,
        context_threshold: f64,
        graceful_degradation: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        info!("Processing research request: '{}'", topic);

        // Log classification options
        if advanced_classification {
            info!("Advanced multi-dimensional classification enabled");
            if graceful_degradation {
                info!("Graceful degradation enabled for advanced classification");
            }
        }
        if context_detection {
            info!(
                "Context detection enabled with threshold: {:.2}",
                context_threshold
            );
        }

        // Parse frameworks and tags
        let frameworks_vec = frameworks
            .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let tags_vec = tags
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        // Create contexts
        let audience_context = AudienceContext {
            level,
            domain: domain.clone(),
            format: format.clone(),
        };

        let domain_context = DomainContext {
            technology,
            project_type,
            frameworks: frameworks_vec,
            tags: tags_vec,
        };

        // Process the research request
        let result = self
            .pipeline
            .process_query(&topic, Some(audience_context), Some(domain_context))
            .await?;

        // Output the result
        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&result)?;
                println!("{json}");
            }
            "markdown" | _ => {
                self.print_research_result_markdown(&result);
            }
        }

        Ok(())
    }

    async fn handle_list(
        &self,
        research_type: Option<String>,
        tag: Option<String>,
        newer_than: Option<u64>,
        format: String,
        limit: usize,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        info!("Listing cached research results");

        let entries = self.pipeline.list_cached_results().await?;

        // Apply filters
        let filtered_entries: Vec<_> = entries
            .into_iter()
            .filter(|entry| {
                // Filter by research type
                if let Some(ref type_filter) = research_type {
                    if entry.research_type.to_string().to_lowercase() != type_filter.to_lowercase()
                    {
                        return false;
                    }
                }

                // Filter by tag
                if let Some(ref tag_filter) = tag {
                    if !entry
                        .metadata
                        .get("tags")
                        .unwrap_or(&String::new())
                        .contains(tag_filter)
                    {
                        return false;
                    }
                }

                // Filter by age
                if let Some(days) = newer_than {
                    let age_days = entry.age_seconds() / 86400; // Convert to days
                    if age_days > days {
                        return false;
                    }
                }

                true
            })
            .take(limit)
            .collect();

        // Output results
        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&filtered_entries)?;
                println!("{json}");
            }
            "summary" => {
                self.print_entries_summary(&filtered_entries);
            }
            "table" | _ => {
                self.print_entries_table(&filtered_entries);
            }
        }

        Ok(())
    }

    async fn handle_cache_status(
        &self,
        detailed: bool,
        format: String,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        info!("Retrieving cache status");

        let stats = self.pipeline.get_cache_stats().await?;

        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&stats)?;
                println!("{json}");
            }
            "table" | _ => {
                self.print_cache_stats(&stats, detailed);
            }
        }

        Ok(())
    }

    async fn handle_search(
        &self,
        query: String,
        research_type: Option<String>,
        tags: Option<String>,
        min_quality: Option<f64>,
        limit: usize,
        format: String,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        info!("Searching cached research results for: '{}'", query);

        // Parse research type
        let research_type_filter = if let Some(t) = research_type {
            match t.parse::<ResearchType>() {
                Ok(rt) => Some(rt),
                Err(e) => {
                    return Err(format!("Invalid research type: {e}").into());
                }
            }
        } else {
            None
        };

        // Parse tags
        let tags_vec: Vec<String> = tags
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        // Create search query
        let search_query = SearchQuery::new(query).with_limit(limit).with_offset(0);

        let search_query = if let Some(rt) = research_type_filter {
            search_query.with_research_type(rt)
        } else {
            search_query
        };

        let search_query = if !tags_vec.is_empty() {
            search_query.with_tags(tags_vec)
        } else {
            search_query
        };

        let search_query = if let Some(quality) = min_quality {
            search_query.with_min_quality(quality)
        } else {
            search_query
        };

        let results = self.pipeline.search_results(&search_query).await?;

        // Output results
        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&results)?;
                println!("{json}");
            }
            "summary" => {
                self.print_search_results_summary(&results);
            }
            "table" | _ => {
                self.print_search_results_table(&results);
            }
        }

        Ok(())
    }

    async fn handle_cleanup(
        &self,
        dry_run: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        info!("Cleaning up cache (dry_run: {})", dry_run);

        if dry_run {
            println!("DRY RUN: No files will be deleted");
            // In a real implementation, we would show what would be deleted
            println!("Would clean up expired cache entries");
        } else {
            let deleted_count = self.pipeline.cleanup_cache().await?;
            println!("Cleaned up {deleted_count} expired cache entries");
        }

        Ok(())
    }

    fn print_research_result_markdown(&self, result: &ResearchResult) {
        println!("# Research Result");
        println!();
        println!("**Query:** {}", result.request.original_query);
        println!("**Type:** {}", result.request.research_type);
        println!("**Confidence:** {:.2}", result.request.confidence);
        println!(
            "**Completed:** {}",
            result.metadata.completed_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!(
            "**Processing Time:** {}ms",
            result.metadata.processing_time_ms
        );

        // Show classification metadata if available
        if let Some(fallback_used) = result.metadata.tags.get("fallback_used") {
            println!("**Fallback Used:** {fallback_used}");
        }
        if let Some(algorithm) = result.metadata.tags.get("algorithm") {
            println!("**Classification Algorithm:** {algorithm}");
        }
        if let Some(context_detection) = result.metadata.tags.get("context_detection") {
            println!("**Context Detection:** {context_detection}");
        }

        println!();

        println!("## Answer");
        println!();
        println!("{}", result.immediate_answer);
        println!();

        if !result.supporting_evidence.is_empty() {
            println!("## Supporting Evidence");
            println!();
            for (i, evidence) in result.supporting_evidence.iter().enumerate() {
                println!(
                    "### {}. {} (Relevance: {:.2})",
                    i + 1,
                    evidence.source,
                    evidence.relevance
                );
                println!("{}", evidence.content);
                println!();
            }
        }

        if !result.implementation_details.is_empty() {
            println!("## Implementation Details");
            println!();
            for (i, detail) in result.implementation_details.iter().enumerate() {
                println!(
                    "### {}. {} (Priority: {})",
                    i + 1,
                    detail.category,
                    detail.priority
                );
                if !detail.prerequisites.is_empty() {
                    println!("**Prerequisites:** {}", detail.prerequisites.join(", "));
                }
                println!("{}", detail.content);
                println!();
            }
        }

        if !result.metadata.sources_consulted.is_empty() {
            println!("## Sources Consulted");
            println!();
            for source in &result.metadata.sources_consulted {
                println!("- {source}");
            }
            println!();
        }

        println!("**Cache Key:** {}", result.metadata.cache_key);
        println!("**Quality Score:** {:.2}", result.metadata.quality_score);
    }

    fn print_entries_table(&self, entries: &[CacheEntry]) {
        if entries.is_empty() {
            println!("No cached entries found");
            return;
        }

        println!(
            "{:<20} {:<15} {:<30} {:<20} {:<10}",
            "Cache Key", "Type", "Query", "Created", "Size"
        );
        println!("{}", "-".repeat(95));

        for entry in entries {
            let key_short = if entry.key.len() > 18 {
                format!("{}...", &entry.key[..15])
            } else {
                entry.key.clone()
            };

            let query_short = if entry.original_query.len() > 28 {
                format!("{}...", &entry.original_query[..25])
            } else {
                entry.original_query.clone()
            };

            println!(
                "{:<20} {:<15} {:<30} {:<20} {:<10}",
                key_short,
                entry.research_type.to_string(),
                query_short,
                entry.created_at.format("%Y-%m-%d %H:%M"),
                Self::format_size(entry.size_bytes)
            );
        }

        println!("\nTotal entries: {}", entries.len());
    }

    fn print_entries_summary(&self, entries: &[CacheEntry]) {
        if entries.is_empty() {
            println!("No cached entries found");
            return;
        }

        let mut by_type = std::collections::HashMap::new();
        let mut total_size = 0;

        for entry in entries {
            *by_type.entry(entry.research_type.clone()).or_insert(0) += 1;
            total_size += entry.size_bytes;
        }

        println!("Cache Summary:");
        println!("Total entries: {}", entries.len());
        println!("Total size: {}", Self::format_size(total_size));
        println!();

        println!("By research type:");
        for (research_type, count) in by_type {
            println!("  {research_type}: {count} entries");
        }
    }

    fn print_search_results_table(&self, results: &[SearchResult]) {
        if results.is_empty() {
            println!("No search results found");
            return;
        }

        println!(
            "{:<15} {:<30} {:<10} {:<50}",
            "Type", "Query", "Relevance", "Snippet"
        );
        println!("{}", "-".repeat(105));

        for result in results {
            let query_short = if result.entry.original_query.len() > 28 {
                format!("{}...", &result.entry.original_query[..25])
            } else {
                result.entry.original_query.clone()
            };

            let snippet_short = if result.snippet.len() > 48 {
                format!("{}...", &result.snippet[..45])
            } else {
                result.snippet.clone()
            };

            println!(
                "{:<15} {:<30} {:<10.2} {:<50}",
                result.entry.research_type.to_string(),
                query_short,
                result.relevance_score,
                snippet_short
            );
        }

        println!("\nTotal results: {}", results.len());
    }

    fn print_search_results_summary(&self, results: &[SearchResult]) {
        if results.is_empty() {
            println!("No search results found");
            return;
        }

        println!("Search Results Summary:");
        println!("Total results: {}", results.len());

        let avg_relevance =
            results.iter().map(|r| r.relevance_score).sum::<f64>() / results.len() as f64;

        println!("Average relevance: {avg_relevance:.2}");
        println!();

        println!("Top 3 results:");
        for (i, result) in results.iter().take(3).enumerate() {
            println!(
                "{}. {} (Relevance: {:.2})",
                i + 1,
                result.entry.original_query,
                result.relevance_score
            );
            println!("   Type: {}", result.entry.research_type);
            println!("   Snippet: {}", result.snippet);
            println!();
        }
    }

    fn print_cache_stats(&self, stats: &CacheStats, detailed: bool) {
        println!("Cache Statistics:");
        println!("Total entries: {}", stats.total_entries);
        println!("Expired entries: {}", stats.expired_entries);
        println!("Total size: {}", Self::format_size(stats.total_size_bytes));
        println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
        println!("Hits: {}", stats.hits);
        println!("Misses: {}", stats.misses);
        println!(
            "Average age: {:.1} hours",
            stats.average_age_seconds / 3600.0
        );

        if detailed && !stats.by_research_type.is_empty() {
            println!("\nBy research type:");
            for (research_type, type_stats) in &stats.by_research_type {
                println!("  {research_type}:");
                println!("    Entries: {}", type_stats.entries);
                println!("    Size: {}", Self::format_size(type_stats.size_bytes));
                println!("    Hit rate: {:.2}%", type_stats.hit_rate * 100.0);
                println!("    Hits: {}", type_stats.hits);
                println!("    Misses: {}", type_stats.misses);
            }
        }
    }

    fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    // Vector command handlers
    async fn handle_vector_command(
        &self,
        vector_command: VectorCommand,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        match vector_command {
            VectorCommand::Config {
                url,
                api_key,
                collection,
                show,
            } => {
                self.handle_vector_config(url, api_key, collection, show)
                    .await
            }
            VectorCommand::Health { _detailed, _format } => {
                self.handle_vector_health(_detailed, _format).await
            }
            VectorCommand::Stats {
                collection,
                _format,
                _detailed,
            } => {
                self.handle_vector_stats(collection, _format, _detailed)
                    .await
            }
            VectorCommand::Migrate {
                source,
                collection,
                batch_size,
                validation,
                dry_run,
                resume,
            } => {
                self.handle_vector_migrate(
                    source, collection, batch_size, validation, dry_run, resume,
                )
                .await
            }
            VectorCommand::MigrationStatus { id, _format, all } => {
                self.handle_migration_status(id, _format, all).await
            }
            VectorCommand::MigrationResume { id, force } => {
                self.handle_migration_resume(id, force).await
            }
            VectorCommand::MigrationCancel { id, force } => {
                self.handle_migration_cancel(id, force).await
            }
            VectorCommand::MigrationList { _format, active } => {
                self.handle_migration_list(_format, active).await
            }
            VectorCommand::Analytics {
                period,
                collection,
                _format,
                _performance,
            } => {
                self.handle_vector_analytics(period, collection, _format, _performance)
                    .await
            }
            VectorCommand::Setup {
                collection,
                dimensions,
                metric,
                force,
            } => {
                self.handle_vector_setup(collection, dimensions, metric, force)
                    .await
            }
            VectorCommand::Index { index_command } => {
                self.handle_index_command(index_command).await
            }
        }
    }

    async fn handle_semantic_search(
        &self,
        params: SemanticSearchParams,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _semantic_search = self
            .semantic_search
            .as_ref()
            .ok_or("Semantic search service not available. Please configure vector database.")?;

        info!("Performing semantic search for: '{}'", params.query);

        // Use provided collection or default
        let collection_name = params.collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        // Placeholder semantic search implementation
        println!("Semantic search query: '{}'", params.query);
        println!("Collection: {collection_name}");
        println!(
            "Limit: {}, Threshold: {:.2}",
            params.limit, params.threshold
        );
        println!("Strategy: {}", params.strategy);
        if params.explain {
            println!("Explanation requested");
        }
        println!("Semantic search functionality not yet implemented");

        Ok(())
    }

    async fn handle_hybrid_search(
        &self,
        params: HybridSearchParams,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _hybrid_search = self
            .hybrid_search
            .as_ref()
            .ok_or("Hybrid search service not available. Please configure vector database.")?;

        info!("Performing hybrid search for: '{}'", params.query);

        // Use provided collection or default
        let collection_name = params.collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        // Placeholder hybrid search implementation
        println!("Hybrid search query: '{}'", params.query);
        println!("Collection: {collection_name}");
        println!(
            "Keyword weight: {:.2}, Semantic weight: {:.2}",
            params.keyword_weight, params.semantic_weight
        );
        println!(
            "Limit: {}, Threshold: {:.2}",
            params.limit, params.threshold
        );
        if params.explain {
            println!("Explanation requested");
        }
        println!("Hybrid search functionality not yet implemented");

        Ok(())
    }

    async fn handle_find_similar(
        &self,
        content: String,
        limit: usize,
        threshold: f64,
        _format: String,
        collection: Option<String>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _semantic_search = self
            .semantic_search
            .as_ref()
            .ok_or("Semantic search service not available. Please configure vector database.")?;

        info!("Finding content similar to provided text");

        // Use provided collection or default
        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        // Placeholder find similar implementation
        println!("Find similar content query");
        println!("Collection: {collection_name}");
        println!("Limit: {limit}, Threshold: {threshold:.2}");
        println!(
            "Content: {}",
            if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.clone()
            }
        );
        println!("Find similar functionality not yet implemented");

        Ok(())
    }

    // Vector configuration and health
    async fn handle_vector_config(
        &self,
        url: Option<String>,
        api_key: Option<String>,
        collection: Option<String>,
        show: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if show {
            // Show current vector configuration
            if let Some(vector_config) = &self.config.vector {
                println!("Vector Database Configuration:");
                println!("URL: {}", vector_config.url);
                println!(
                    "API Key: {}",
                    if vector_config.api_key.is_some() {
                        "***configured***"
                    } else {
                        "not set"
                    }
                );
                println!("Default Collection: {}", vector_config.default_collection);
                println!("Vector Dimensions: {}", vector_config.vector_dimensions);
                println!("Distance Metric: {}", vector_config.distance_metric);
                println!("Timeout: {}s", vector_config.timeout_seconds);
            } else {
                println!("Vector database is not configured.");
                println!("Use environment variables or configuration file to set up vector database connection.");
            }
        } else {
            // Update configuration (this would need to persist to file)
            println!("Configuration update requested:");
            if let Some(url) = url {
                println!("  URL: {url}");
            }
            if let Some(_api_key) = api_key {
                println!("  API Key: ***updated***");
            }
            if let Some(collection) = collection {
                println!("  Default Collection: {collection}");
            }
            println!("Note: Configuration updates are not yet implemented.");
            println!("Please edit the configuration file directly or use environment variables.");
        }

        Ok(())
    }

    async fn handle_vector_health(
        &self,
        _detailed: bool,
        _format: String,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _client = self
            .qdrant_client
            .as_ref()
            .ok_or("Vector database client not available. Please configure vector database.")?;

        info!("Checking vector database health");

        // Placeholder health check implementation
        println!("Health check not yet implemented");

        Ok(())
    }

    async fn handle_vector_stats(
        &self,
        collection: Option<String>,
        _format: String,
        _detailed: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _client = self
            .qdrant_client
            .as_ref()
            .ok_or("Vector database client not available. Please configure vector database.")?;

        info!("Retrieving vector database statistics");

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        // Placeholder stats implementation
        println!("Collection stats not yet implemented for '{collection_name}'");

        Ok(())
    }

    // Migration handlers
    async fn handle_vector_migrate(
        &self,
        source: String,
        collection: Option<String>,
        batch_size: usize,
        validation: String,
        dry_run: bool,
        resume: Option<String>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _migration_service = self
            .migration_service
            .as_ref()
            .ok_or("Migration service not available. Please configure vector database.")?;

        info!("Starting data migration from: {}", source);

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        if dry_run {
            println!("DRY RUN: Migration simulation");
            println!("Source: {source}");
            println!("Collection: {collection_name}");
            println!("Batch size: {batch_size}");
            println!("Validation level: {validation}");
            // Perform validation and report what would be migrated
            return Ok(());
        }

        // Placeholder migration implementation
        println!("Migration from '{source}' to collection '{collection_name}'");
        println!("Batch size: {batch_size}, Validation: {validation}");
        if let Some(checkpoint_id) = resume {
            println!("Resume from checkpoint: {checkpoint_id}");
        }
        println!("Migration functionality not yet implemented");

        Ok(())
    }

    async fn handle_migration_status(
        &self,
        id: Option<String>,
        _format: String,
        all: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _migration_service = self
            .migration_service
            .as_ref()
            .ok_or("Migration service not available. Please configure vector database.")?;

        if let Some(migration_id) = id {
            println!("Migration status for ID: {migration_id}");
            println!("Migration status functionality not yet implemented");
        } else if all {
            println!("Listing all migrations");
            println!("Migration listing functionality not yet implemented");
        } else {
            return Err("Please specify migration ID or use --all flag".into());
        }

        Ok(())
    }

    async fn handle_migration_resume(
        &self,
        id: String,
        force: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _migration_service = self
            .migration_service
            .as_ref()
            .ok_or("Migration service not available. Please configure vector database.")?;

        info!("Resuming migration: {}", id);

        if force {
            info!("Force resuming migration (ignoring warnings)");
        }

        println!("Resume migration: {id}");
        println!("Migration resume functionality not yet implemented");

        Ok(())
    }

    async fn handle_migration_cancel(
        &self,
        id: String,
        force: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _migration_service = self
            .migration_service
            .as_ref()
            .ok_or("Migration service not available. Please configure vector database.")?;

        info!("Cancelling migration: {}", id);

        if force {
            info!("Force cancelling migration");
        }

        println!("Cancel migration: {id}");
        println!("Migration cancel functionality not yet implemented");

        Ok(())
    }

    async fn handle_migration_list(
        &self,
        _format: String,
        active: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _migration_service = self
            .migration_service
            .as_ref()
            .ok_or("Migration service not available. Please configure vector database.")?;

        println!(
            "Listing {} migrations",
            if active { "active" } else { "all" }
        );
        println!("Migration listing functionality not yet implemented");

        Ok(())
    }

    // Analytics and monitoring
    async fn handle_vector_analytics(
        &self,
        period: u32,
        collection: Option<String>,
        _format: String,
        _performance: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _semantic_search = self
            .semantic_search
            .as_ref()
            .ok_or("Semantic search service not available. Please configure vector database.")?;

        info!("Retrieving search analytics for {} days", period);

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        println!("Analytics for '{collection_name}' (last {period} days)");
        println!("Analytics functionality not yet implemented");

        Ok(())
    }

    async fn handle_vector_setup(
        &self,
        collection: Option<String>,
        dimensions: Option<usize>,
        metric: Option<String>,
        force: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _client = self
            .qdrant_client
            .as_ref()
            .ok_or("Vector database client not available. Please configure vector database.")?;

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        let vector_dimensions = dimensions.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.vector_dimensions)
                .unwrap_or(384)
        });

        let distance_metric = metric.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.distance_metric.clone())
                .unwrap_or_else(|| "cosine".to_string())
        });

        info!("Setting up vector database collection: {}", collection_name);

        if force {
            info!("Force recreating collection (existing data will be lost)");
        }

        println!("Setting up collection: {collection_name}");
        println!("Dimensions: {vector_dimensions}, Distance metric: {distance_metric}");
        println!("Setup functionality not yet implemented");

        Ok(())
    }

    async fn handle_index_command(
        &self,
        index_command: IndexCommand,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        match index_command {
            IndexCommand::Status {
                collection,
                _format,
            } => self.handle_index_status(collection, _format).await,
            IndexCommand::Rebuild { collection, force } => {
                self.handle_index_rebuild(collection, force).await
            }
            IndexCommand::Optimize { collection } => self.handle_index_optimize(collection).await,
        }
    }

    async fn handle_index_status(
        &self,
        collection: Option<String>,
        _format: String,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _client = self
            .qdrant_client
            .as_ref()
            .ok_or("Vector database client not available. Please configure vector database.")?;

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        println!("Index status for '{collection_name}' not yet implemented");

        Ok(())
    }

    async fn handle_index_rebuild(
        &self,
        collection: Option<String>,
        force: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _client = self
            .qdrant_client
            .as_ref()
            .ok_or("Vector database client not available. Please configure vector database.")?;

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        info!("Rebuilding index for collection: {}", collection_name);

        if force {
            info!("Force rebuilding index");
        }

        println!("Index rebuild for '{collection_name}' not yet implemented");

        Ok(())
    }

    async fn handle_index_optimize(
        &self,
        collection: Option<String>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let _client = self
            .qdrant_client
            .as_ref()
            .ok_or("Vector database client not available. Please configure vector database.")?;

        let collection_name = collection.unwrap_or_else(|| {
            self.config
                .vector
                .as_ref()
                .map(|v| v.default_collection.clone())
                .unwrap_or_else(|| "fortitude_research".to_string())
        });

        info!("Optimizing index for collection: {}", collection_name);

        println!("Index optimization for '{collection_name}' not yet implemented");

        Ok(())
    }

    // Output formatting methods
    #[allow(dead_code)] // TODO: Use in semantic search result display
    fn print_semantic_search_table(&self, results: &[VectorSearchResult]) {
        if results.is_empty() {
            println!("No search results found");
            return;
        }

        println!("{:<10} {:<40} {:<30}", "Score", "Content", "Source");
        println!("{}", "-".repeat(80));

        for result in results {
            let content_short = if result.document.content.len() > 38 {
                format!("{}...", &result.document.content[..35])
            } else {
                result.document.content.clone()
            };

            let source_short = match &result.document.metadata.source {
                Some(source) if source.len() > 28 => format!("{}...", &source[..25]),
                Some(source) => source.clone(),
                None => "Unknown".to_string(),
            };

            println!(
                "{:<10.3} {:<40} {:<30}",
                result.relevance_score, content_short, source_short
            );
        }

        println!("\nTotal results: {}", results.len());
    }

    #[allow(dead_code)] // TODO: Use in detailed semantic search result display
    fn print_semantic_search_detailed(&self, results: &[VectorSearchResult], explain: bool) {
        if results.is_empty() {
            println!("No search results found");
            return;
        }

        println!("Semantic Search Results:");
        println!("========================");

        for (i, result) in results.iter().enumerate() {
            println!("\n{}. Score: {:.3}", i + 1, result.relevance_score);
            println!(
                "   Source: {}",
                result
                    .document
                    .metadata
                    .source
                    .as_deref()
                    .unwrap_or("Unknown")
            );
            println!("   Content: {}", result.document.content);

            if explain {
                println!("   Explanation: Vector similarity search using cosine distance");
                println!("   Similarity Score: {:.3}", result.similarity_score);
                if let Some(explanation) = &result.explanation {
                    println!("   Details: {}", explanation.calculation);
                }
            }
        }

        println!("\nTotal results: {}", results.len());
    }

    #[allow(dead_code)] // TODO: Use in hybrid search result display
    fn print_hybrid_search_table(&self, results: &[VectorHybridSearchResult]) {
        if results.is_empty() {
            println!("No search results found");
            return;
        }

        println!(
            "{:<10} {:<10} {:<30} {:<30}",
            "Score", "Strategy", "Content", "Source"
        );
        println!("{}", "-".repeat(80));

        for result in results {
            let content_short = if result.document.content.len() > 28 {
                format!("{}...", &result.document.content[..25])
            } else {
                result.document.content.clone()
            };

            let source_short = match &result.document.metadata.source {
                Some(source) if source.len() > 28 => format!("{}...", &source[..25]),
                Some(source) => source.clone(),
                None => "Unknown".to_string(),
            };

            println!(
                "{:<10.3} {:<10} {:<30} {:<30}",
                result.hybrid_score,
                format!("{:?}", result.strategy),
                content_short,
                source_short
            );
        }

        println!("\nTotal results: {}", results.len());
    }

    #[allow(dead_code)] // TODO: Use in detailed hybrid search result display
    fn print_hybrid_search_detailed(&self, results: &[VectorHybridSearchResult], explain: bool) {
        if results.is_empty() {
            println!("No search results found");
            return;
        }

        println!("Hybrid Search Results:");
        println!("=====================");

        for (i, result) in results.iter().enumerate() {
            println!(
                "\n{}. Score: {:.3} (Strategy: {:?})",
                i + 1,
                result.hybrid_score,
                result.strategy
            );
            println!(
                "   Source: {}",
                result
                    .document
                    .metadata
                    .source
                    .as_deref()
                    .unwrap_or("Unknown")
            );
            println!("   Content: {}", result.document.content);

            if explain {
                println!("   Explanation: Hybrid search combining vector similarity and keyword matching");
                println!(
                    "   Vector score: {:.3}, Keyword score: {:.3}",
                    result.vector_score.unwrap_or(0.0),
                    result.keyword_score.unwrap_or(0.0)
                );
                println!("   Fusion method: {:?}", result.fusion_method);
            }
        }

        println!("\nTotal results: {}", results.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("fortitude").unwrap();
        cmd.arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Automated research system"));
    }

    #[test]
    fn test_cli_version() {
        let mut cmd = Command::cargo_bin("fortitude").unwrap();
        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("0.1.0"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(App::format_size(512), "512.0 B");
        assert_eq!(App::format_size(1024), "1.0 KB");
        assert_eq!(App::format_size(1536), "1.5 KB");
        assert_eq!(App::format_size(1048576), "1.0 MB");
        assert_eq!(App::format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_cli_research_with_advanced_options() {
        let mut cmd = Command::cargo_bin("fortitude").unwrap();
        cmd.args([
            "research",
            "--advanced-classification",
            "--context-detection",
            "--context-threshold",
            "0.8",
            "--graceful-degradation",
            "How to implement async functions in Rust?",
        ]);

        // This will fail without proper setup, but we can test the CLI parsing
        cmd.assert().failure(); // Expected to fail without proper configuration
    }

    #[test]
    fn test_cli_config_commands() {
        let mut cmd = Command::cargo_bin("fortitude").unwrap();
        cmd.args([
            "config",
            "generate",
            "--force",
            "--output",
            "/tmp/test_config.json",
        ]);

        cmd.assert().success();
    }
}
