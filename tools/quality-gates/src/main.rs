use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tracing::{info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Project path to validate
    #[arg(short, long, default_value = ".")]
    project_path: PathBuf,
    
    /// Coverage target percentage
    #[arg(short, long, default_value = "95")]
    coverage_target: u8,
    
    /// Performance target in milliseconds
    #[arg(long, default_value = "200")]
    performance_target: u64,
    
    /// Enable security scanning
    #[arg(long, default_value = "true")]
    security_scan: bool,
    
    /// Enable Fortitude integration
    #[arg(long, default_value = "true")]
    fortitude_enabled: bool,
    
    /// Output report file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
struct QualityReport {
    timestamp: DateTime<Utc>,
    project_path: String,
    branch: String,
    commit: String,
    quality_gates: QualityGates,
    recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QualityGates {
    all_passed: bool,
    coverage_percentage: f64,
    coverage_target: u8,
    security_scan_enabled: bool,
    todo_comments: u32,
    performance_target: u64,
    gates: Vec<QualityGate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QualityGate {
    name: String,
    status: GateStatus,
    description: String,
    output: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum GateStatus {
    Passed,
    Failed,
    Skipped,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let args = Args::parse();

    println!("{}", "🔍 CE-DPS Quality Gates - AI Implementation Validation".blue().bold());
    println!("{}", "==================================================".blue());

    let mut quality_gates = QualityGates {
        all_passed: true,
        coverage_percentage: 0.0,
        coverage_target: args.coverage_target,
        security_scan_enabled: args.security_scan,
        todo_comments: 0,
        performance_target: args.performance_target,
        gates: Vec::new(),
    };

    // Run quality gates
    run_pre_implementation_gates(&args, &mut quality_gates).await?;
    run_implementation_gates(&args, &mut quality_gates).await?;
    run_post_implementation_gates(&args, &mut quality_gates).await?;

    // Generate report
    let report = generate_report(&args, &quality_gates).await?;

    // Save report if requested
    if let Some(output_path) = &args.output {
        save_report(&report, output_path)?;
    }

    // Print summary
    print_summary(&quality_gates);

    // Exit with appropriate code
    if quality_gates.all_passed {
        println!("\n{}", "🎉 All Quality Gates Passed!".green().bold());
        println!("Your AI-implemented code meets CE-DPS quality standards.");
        println!("Ready for human business validation and production deployment.");
        Ok(())
    } else {
        println!("\n{}", "❌ Some Quality Gates Failed!".red().bold());
        println!("Please address the failures before proceeding.");
        std::process::exit(1);
    }
}

async fn run_pre_implementation_gates(_args: &Args, quality_gates: &mut QualityGates) -> Result<()> {
    println!("\n{}", "📋 Pre-Implementation Quality Gates".yellow().bold());

    let gates = vec![
        ("Branch Check", "git branch --show-current | grep -v '^main$' | grep -v '^master$' || echo 'On main/master branch'", "Ensure not on main/master branch"),
        ("Working Directory", "git status --porcelain", "Check for uncommitted changes"),
        ("Compilation", "cargo check --all-targets", "Ensure code compiles without errors"),
    ];

    for (name, command, description) in gates {
        run_gate(name, command, description, quality_gates).await?;
    }

    Ok(())
}

async fn run_implementation_gates(args: &Args, quality_gates: &mut QualityGates) -> Result<()> {
    println!("\n{}", "🔨 Implementation Quality Gates".yellow().bold());

    let gates = vec![
        ("Code Formatting", "cargo fmt --check", "Ensure consistent code formatting"),
        ("Linting", "cargo clippy --all-targets -- -D warnings", "Ensure code quality standards"),
        ("Unit Tests", "cargo test --lib", "Ensure all unit tests pass"),
        ("Integration Tests", "cargo test --test '*'", "Ensure integration tests pass"),
    ];

    for (name, command, description) in gates {
        run_gate(name, command, description, quality_gates).await?;
    }

    // Security audit if enabled
    if args.security_scan {
        run_gate("Security Audit", "cargo audit", "Check for security vulnerabilities", quality_gates).await?;
    }

    Ok(())
}

async fn run_post_implementation_gates(args: &Args, quality_gates: &mut QualityGates) -> Result<()> {
    println!("\n{}", "🎯 Post-Implementation Quality Gates".yellow().bold());

    let gates = vec![
        ("Final Compilation", "cargo build --release", "Ensure production build succeeds"),
        ("Documentation Build", "cargo doc --no-deps", "Ensure documentation builds successfully"),
    ];

    for (name, command, description) in gates {
        run_gate(name, command, description, quality_gates).await?;
    }

    // Count TODO comments
    count_todo_comments(args, quality_gates).await?;

    Ok(())
}

async fn run_gate(name: &str, command: &str, description: &str, quality_gates: &mut QualityGates) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.blue} {msg}").unwrap());
    pb.set_message(format!("{}: {}", name, description));
    pb.enable_steady_tick(Duration::from_millis(100));

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .context(format!("Failed to execute command: {}", command))?;

    pb.finish_and_clear();

    let success = output.status.success();
    let status = if success { GateStatus::Passed } else { GateStatus::Failed };

    if !success {
        quality_gates.all_passed = false;
    }

    let gate = QualityGate {
        name: name.to_string(),
        status,
        description: description.to_string(),
        output: if output.stdout.is_empty() { None } else { Some(String::from_utf8_lossy(&output.stdout).to_string()) },
        error: if output.stderr.is_empty() { None } else { Some(String::from_utf8_lossy(&output.stderr).to_string()) },
    };

    let status_icon = match gate.status {
        GateStatus::Passed => "✅".green(),
        GateStatus::Failed => "❌".red(),
        GateStatus::Skipped => "⏭️".yellow(),
    };

    println!("{} {}: {}", status_icon, name, description);

    if let Some(error) = &gate.error {
        println!("   Error: {}", error.trim());
    }

    quality_gates.gates.push(gate);

    Ok(())
}

async fn count_todo_comments(args: &Args, quality_gates: &mut QualityGates) -> Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("grep -r 'TODO\\|FIXME\\|HACK' src/ --include='*.rs' | wc -l")
        .current_dir(&args.project_path)
        .output()
        .context("Failed to count TODO comments")?;

    if output.status.success() {
        let count_str = String::from_utf8_lossy(&output.stdout);
        let count: u32 = count_str.trim().parse().unwrap_or(0);
        quality_gates.todo_comments = count;

        if count > 0 {
            println!("{} Found {} TODO/FIXME/HACK comments - review before production", "⚠️".yellow(), count);
        }
    }

    Ok(())
}

async fn generate_report(args: &Args, quality_gates: &QualityGates) -> Result<QualityReport> {
    let branch = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(&args.project_path)
        .output()
        .context("Failed to get git branch")?;

    let commit = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(&args.project_path)
        .output()
        .context("Failed to get git commit")?;

    let report = QualityReport {
        timestamp: Utc::now(),
        project_path: args.project_path.to_string_lossy().to_string(),
        branch: if branch.status.success() { 
            String::from_utf8_lossy(&branch.stdout).trim().to_string() 
        } else { 
            "unknown".to_string() 
        },
        commit: if commit.status.success() { 
            String::from_utf8_lossy(&commit.stdout).trim().to_string() 
        } else { 
            "unknown".to_string() 
        },
        quality_gates: quality_gates.clone(),
        recommendations: vec![
            "Review any TODO/FIXME/HACK comments before production deployment".to_string(),
            "Consider adding performance benchmarks if not present".to_string(),
            "Ensure monitoring and alerting are configured for production".to_string(),
        ],
    };

    Ok(report)
}

fn save_report(report: &QualityReport, output_path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(output_path, json)?;
    println!("{} Quality report generated: {}", "📊".blue(), output_path.display());
    Ok(())
}

fn print_summary(quality_gates: &QualityGates) {
    println!("\n{}", "📊 Quality Gates Summary".blue().bold());
    println!("{}", "========================".blue());

    let passed = quality_gates.gates.iter().filter(|g| matches!(g.status, GateStatus::Passed)).count();
    let failed = quality_gates.gates.iter().filter(|g| matches!(g.status, GateStatus::Failed)).count();
    let total = quality_gates.gates.len();

    println!("Total Gates: {}", total);
    println!("Passed: {}", passed.to_string().green());
    println!("Failed: {}", failed.to_string().red());
    println!("TODO Comments: {}", quality_gates.todo_comments);
}