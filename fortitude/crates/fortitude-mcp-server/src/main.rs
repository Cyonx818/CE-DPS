// ABOUTME: Main entry point for Fortitude MCP server
// Provides command-line interface for starting the MCP server
// Handles configuration loading and graceful shutdown

use clap::{Parser, Subcommand};
use fortitude_mcp_server::{McpServer, ServerConfig};
use std::process::ExitCode;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    /// Enable quiet mode (minimal output)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server (default if no subcommand)
    Start {
        /// Server port (overrides config file)
        #[arg(short, long)]
        port: Option<u16>,

        /// Server host (overrides config file)
        #[arg(long)]
        host: Option<String>,

        /// Run in daemon mode
        #[arg(long)]
        daemon: bool,
    },
    /// Stop the MCP server
    Stop {
        /// Force stop (kill process)
        #[arg(short, long)]
        force: bool,
    },
    /// Show server status
    Status,
    /// Validate configuration file
    ValidateConfig,
    /// Generate sample configuration file
    GenerateConfig {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Output format (json, toml)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    /// Show configuration documentation
    ConfigHelp,
    /// Show current configuration
    ShowConfig,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.debug {
        EnvFilter::new("debug")
    } else if args.quiet {
        EnvFilter::new("warn")
    } else {
        EnvFilter::from_default_env()
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Execute command
    match args.command.as_ref().unwrap_or(&Commands::Start {
        port: None,
        host: None,
        daemon: false,
    }) {
        Commands::Start { port, host, daemon } => {
            start_server(&args, *port, host.clone(), *daemon).await
        }
        Commands::Stop { force } => stop_server(*force).await,
        Commands::Status => show_status().await,
        Commands::ValidateConfig => validate_config(&args).await,
        Commands::GenerateConfig { output, format } => {
            generate_config(output.clone(), format.clone()).await
        }
        Commands::ConfigHelp => show_config_help().await,
        Commands::ShowConfig => show_config(&args).await,
    }
}

async fn start_server(
    args: &Args,
    port: Option<u16>,
    host: Option<String>,
    daemon: bool,
) -> ExitCode {
    // Load configuration
    let config = match load_config(args, port, host).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if daemon {
        info!("Starting MCP server in daemon mode...");
        // TODO: Implement daemon mode
        warn!("Daemon mode not yet implemented, running in foreground");
    }

    // Create and run server
    let server = match McpServer::new(config).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create MCP server: {}", e);
            return ExitCode::FAILURE;
        }
    };

    info!("Starting Fortitude MCP server...");

    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        return ExitCode::FAILURE;
    }

    info!("Server shutdown complete");
    ExitCode::SUCCESS
}

async fn stop_server(force: bool) -> ExitCode {
    // TODO: Implement server stopping logic
    if force {
        info!("Force stopping MCP server...");
        // Find and kill the process
        warn!("Force stop not yet implemented");
    } else {
        info!("Gracefully stopping MCP server...");
        // Send shutdown signal
        warn!("Graceful stop not yet implemented");
    }
    ExitCode::SUCCESS
}

async fn show_status() -> ExitCode {
    // TODO: Implement status checking logic
    info!("Checking MCP server status...");
    warn!("Status check not yet implemented");
    ExitCode::SUCCESS
}

async fn validate_config(args: &Args) -> ExitCode {
    info!("Validating configuration...");

    let config = match load_config(args, None, None).await {
        Ok(config) => config,
        Err(e) => {
            error!("Configuration validation failed: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Additional validation can be added here
    match config.validate() {
        Ok(_) => {
            info!("Configuration is valid");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Configuration validation failed: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn generate_config(output: Option<String>, format: String) -> ExitCode {
    info!("Generating sample configuration...");

    let config = ServerConfig::default();
    let output_path = output.unwrap_or_else(|| match format.as_str() {
        "toml" => "fortitude-mcp-server.toml".to_string(),
        "json" => "fortitude-mcp-server.json".to_string(),
        _ => "fortitude-mcp-server.toml".to_string(),
    });

    match config.save_to_file(&output_path).await {
        Ok(_) => {
            info!("Sample configuration saved to: {}", output_path);
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Failed to save configuration: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn show_config_help() -> ExitCode {
    println!("Fortitude MCP Server Configuration");
    println!("==================================\\n");

    println!("Environment Variables:");
    println!("=====================\\n");

    for (var, desc) in ServerConfig::get_env_var_documentation() {
        println!("{var:<50} {desc}");
    }

    println!("\\nConfiguration Files:");
    println!("==================\\n");

    println!("The MCP server supports both JSON and TOML configuration formats.");
    println!("The format is determined by the file extension (.json or .toml).");
    println!("\\nDefault configuration search paths:");
    println!("- ./fortitude-mcp-server.toml");
    println!("- ./fortitude-mcp-server.json");
    println!("- ~/.config/fortitude/mcp-server.toml");
    println!("- ~/.config/fortitude/mcp-server.json");

    println!("\\nExamples:");
    println!("========\\n");

    println!("Generate a sample configuration:");
    println!("  fortitude-mcp-server generate-config --output config.toml --format toml");
    println!("\\nValidate configuration:");
    println!("  fortitude-mcp-server validate-config --config config.toml");
    println!("\\nStart with custom configuration:");
    println!("  fortitude-mcp-server start --config config.toml --port 8090");

    ExitCode::SUCCESS
}

async fn show_config(args: &Args) -> ExitCode {
    info!("Loading current configuration...");

    let config = match load_config(args, None, None).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            println!("Current Configuration:");
            println!("=====================\\n");
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Failed to serialize configuration: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn load_config(
    args: &Args,
    port: Option<u16>,
    host: Option<String>,
) -> anyhow::Result<ServerConfig> {
    let mut config = if let Some(config_path) = &args.config {
        ServerConfig::from_file_with_format(config_path).await?
    } else {
        // Try to load from default locations
        let default_paths = [
            "fortitude-mcp-server.toml",
            "fortitude-mcp-server.json",
            "~/.config/fortitude/mcp-server.toml",
            "~/.config/fortitude/mcp-server.json",
        ];

        let mut loaded = false;
        let mut config = ServerConfig::default();

        for path in &default_paths {
            if std::path::Path::new(path).exists() {
                config = ServerConfig::from_file_with_format(path).await?;
                loaded = true;
                break;
            }
        }

        if !loaded {
            // Load from environment variables
            config = ServerConfig::from_env()?;
        }

        config
    };

    // Override with command line arguments
    if let Some(port) = port {
        config.port = port;
    }

    if let Some(host) = host {
        config.host = host;
    }

    Ok(config)
}
