use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check Fortitude installation
    Check,
    /// Build Fortitude
    Build,
    /// Initialize Fortitude for CE-DPS
    Init,
    /// Start Fortitude services
    Start,
    /// Stop Fortitude services
    Stop,
    /// Check Fortitude service status
    Status,
    /// Restart Fortitude services
    Restart,
    /// Update implementation patterns
    Update,
    /// Query knowledge base
    Query {
        /// Query string
        query: String,
    },
    /// Generate knowledge report
    Report,
    /// Setup Claude Code integration
    SetupClaude,
    /// Complete installation and setup
    Install,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    let args = Args::parse();

    println!("{}", "🧠 CE-DPS Fortitude Integration".blue().bold());
    println!("{}", "====================================".blue());

    match args.command {
        Commands::Check => check_fortitude().await,
        Commands::Build => build_fortitude().await,
        Commands::Init => init_fortitude().await,
        Commands::Start => start_fortitude().await,
        Commands::Stop => stop_fortitude().await,
        Commands::Status => status_fortitude().await,
        Commands::Restart => restart_fortitude().await,
        Commands::Update => update_patterns().await,
        Commands::Query { query } => query_knowledge(&query).await,
        Commands::Report => generate_report().await,
        Commands::SetupClaude => setup_claude_integration().await,
        Commands::Install => install_complete().await,
    }
}

async fn check_fortitude() -> Result<()> {
    println!("Checking Fortitude installation...");

    let fortitude_dir = get_fortitude_dir()?;

    if !fortitude_dir.exists() {
        println!(
            "{} Fortitude directory not found: {}",
            "❌".red(),
            fortitude_dir.display()
        );
        return Ok(());
    }

    if !fortitude_dir.join("Cargo.toml").exists() {
        println!("{} Fortitude Cargo.toml not found", "❌".red());
        return Ok(());
    }

    println!("{} Fortitude installation verified", "✅".green());
    Ok(())
}

async fn build_fortitude() -> Result<()> {
    println!("Building Fortitude...");

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message("Building Fortitude workspace...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&get_fortitude_dir()?)
        .output()
        .context("Failed to build Fortitude")?;

    pb.finish_and_clear();

    if output.status.success() {
        println!("{} Fortitude build successful", "✅".green());
    } else {
        println!("{} Fortitude build failed", "❌".red());
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Build failed"));
    }

    Ok(())
}

async fn init_fortitude() -> Result<()> {
    println!("Initializing Fortitude for CE-DPS...");

    let fortitude_dir = get_fortitude_dir()?;

    // Create CE-DPS configuration
    let config_content = r#"[fortitude]
name = "CE-DPS Knowledge Management"
description = "AI implementation pattern library"
version = "1.0.0"

[classification]
types = ["Decision", "Implementation", "Troubleshooting", "Learning", "Validation"]
default_type = "Implementation"

[gap_detection]
enabled = true
focus_areas = [
    "authentication_patterns",
    "database_patterns",
    "api_patterns",
    "testing_patterns",
    "quality_patterns"
]

[research_prioritization]
ai_implementation_focus = true
security_first = true
testing_comprehensive = true

[learning]
human_collaboration = true
pattern_recognition = true
continuous_improvement = true

[notifications]
channels = ["terminal", "log"]
delivery_verification = true
"#;

    let config_dir = fortitude_dir.join("config");
    std::fs::create_dir_all(&config_dir)?;
    std::fs::write(config_dir.join("ce-dps.toml"), config_content)?;

    println!("{} CE-DPS configuration created", "✅".green());
    Ok(())
}

async fn start_fortitude() -> Result<()> {
    println!("Starting Fortitude services...");

    let fortitude_dir = get_fortitude_dir()?;

    // Start MCP server
    let mcp_child = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "fortitude-mcp-server",
            "--",
            "--config",
            "config/ce-dps.toml",
        ])
        .current_dir(&fortitude_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start Fortitude MCP server")?;

    // Save PID
    let pid_file = fortitude_dir.join(".fortitude-mcp.pid");
    std::fs::write(&pid_file, mcp_child.id().to_string())?;

    println!(
        "{} Fortitude MCP server started (PID: {})",
        "✅".green(),
        mcp_child.id()
    );

    Ok(())
}

async fn stop_fortitude() -> Result<()> {
    println!("Stopping Fortitude services...");

    let fortitude_dir = get_fortitude_dir()?;
    let pid_file = fortitude_dir.join(".fortitude-mcp.pid");

    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    let _ = Command::new("kill").arg(pid.to_string()).output();
                }
                #[cfg(windows)]
                {
                    let _ = Command::new("taskkill")
                        .args(&["/PID", &pid.to_string(), "/F"])
                        .output();
                }
                println!("{} Fortitude MCP server stopped", "✅".green());
            }
        }
        std::fs::remove_file(&pid_file)?;
    }

    Ok(())
}

async fn status_fortitude() -> Result<()> {
    println!("Checking Fortitude status...");

    let fortitude_dir = get_fortitude_dir()?;
    let pid_file = fortitude_dir.join(".fortitude-mcp.pid");

    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is running
                let is_running = check_process_running(pid);
                if is_running {
                    println!(
                        "{} Fortitude MCP server: RUNNING (PID: {})",
                        "✅".green(),
                        pid
                    );
                } else {
                    println!(
                        "{} Fortitude MCP server: STOPPED (stale PID file)",
                        "❌".red()
                    );
                    std::fs::remove_file(&pid_file)?;
                }
            }
        }
    } else {
        println!("{} Fortitude MCP server: STOPPED", "❌".red());
    }

    Ok(())
}

async fn restart_fortitude() -> Result<()> {
    stop_fortitude().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    start_fortitude().await?;
    Ok(())
}

async fn update_patterns() -> Result<()> {
    println!("Updating implementation patterns...");

    let project_dir = std::env::current_dir()?;
    let fortitude_dir = get_fortitude_dir()?;

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "fortitude-cli",
            "--",
            "update-patterns",
            "--project-path",
            &project_dir.to_string_lossy(),
        ])
        .current_dir(&fortitude_dir)
        .output()
        .context("Failed to update patterns")?;

    if output.status.success() {
        println!("{} Implementation patterns updated", "✅".green());
    } else {
        println!("{} Failed to update implementation patterns", "❌".red());
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

async fn query_knowledge(query: &str) -> Result<()> {
    println!("Querying knowledge base: {}", query);

    let project_dir = std::env::current_dir()?;
    let fortitude_dir = get_fortitude_dir()?;

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "fortitude-cli",
            "--",
            "research-query",
            "--query",
            query,
            "--project-path",
            &project_dir.to_string_lossy(),
        ])
        .current_dir(&fortitude_dir)
        .output()
        .context("Failed to query knowledge base")?;

    if output.status.success() {
        println!("{} Knowledge query completed", "✅".green());
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("{} Knowledge query failed", "❌".red());
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

async fn generate_report() -> Result<()> {
    println!("Generating knowledge report...");

    let project_dir = std::env::current_dir()?;
    let fortitude_dir = get_fortitude_dir()?;
    let report_file = format!(
        "target/fortitude-report-{}.json",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "fortitude-cli",
            "--",
            "generate-report",
            "--output",
            &report_file,
            "--project-path",
            &project_dir.to_string_lossy(),
        ])
        .current_dir(&fortitude_dir)
        .output()
        .context("Failed to generate report")?;

    if output.status.success() {
        println!(
            "{} Knowledge report generated: {}",
            "✅".green(),
            report_file
        );
    } else {
        println!("{} Failed to generate knowledge report", "❌".red());
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

async fn setup_claude_integration() -> Result<()> {
    println!("Setting up Claude Code integration...");

    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let claude_config_dir = home_dir.join(".config/claude-code");
    let claude_config_file = claude_config_dir.join("mcp.json");

    std::fs::create_dir_all(&claude_config_dir)?;

    let fortitude_dir = get_fortitude_dir()?;
    let config = serde_json::json!({
        "mcpServers": {
            "fortitude": {
                "command": "cargo",
                "args": ["run", "--bin", "fortitude-mcp-server", "--", "--config", "config/ce-dps.toml"],
                "cwd": fortitude_dir.to_string_lossy()
            }
        }
    });

    if claude_config_file.exists() {
        // Update existing configuration
        let existing: Value = serde_json::from_str(&std::fs::read_to_string(&claude_config_file)?)?;
        let mut updated = existing;
        if let Some(servers) = updated.get_mut("mcpServers") {
            servers.as_object_mut().unwrap().insert(
                "fortitude".to_string(),
                config["mcpServers"]["fortitude"].clone(),
            );
        } else {
            updated["mcpServers"] = config["mcpServers"].clone();
        }
        std::fs::write(&claude_config_file, serde_json::to_string_pretty(&updated)?)?;
    } else {
        std::fs::write(&claude_config_file, serde_json::to_string_pretty(&config)?)?;
    }

    println!("{} Claude Code integration configured", "✅".green());
    Ok(())
}

async fn install_complete() -> Result<()> {
    println!("Installing complete CE-DPS Fortitude integration...");

    check_fortitude().await?;
    build_fortitude().await?;
    init_fortitude().await?;
    setup_claude_integration().await?;

    println!(
        "{} Fortitude integration setup complete!",
        "🎉".green().bold()
    );
    println!("You can now use Fortitude with CE-DPS methodology.");

    Ok(())
}

fn get_fortitude_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let fortitude_dir = current_dir.join("fortitude");
    Ok(fortitude_dir)
}

fn check_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(&["-0", &pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| output.status.success() && !output.stdout.is_empty())
            .unwrap_or(false)
    }
}
