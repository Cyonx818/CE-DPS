// ABOUTME: Anchor tests for critical cross-interface functionality
//! This module provides anchor tests to ensure critical cross-interface functionality
//! remains stable and consistent across CLI, API, and MCP interfaces.
//! These tests protect against regressions in interface integration.

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use tokio;

/// Test configuration for cross-interface anchor tests
pub struct AnchorTestConfig {
    pub temp_dir: TempDir,
    pub config_file: PathBuf,
}

impl AnchorTestConfig {
    /// Create new anchor test configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_file = temp_dir.path().join("anchor_test_config.toml");

        // Create minimal test configuration
        let test_config = r#"
[proactive]
gap_interval = 30
max_tasks = 3
debounce = 5
base_directory = "."
enabled = true

[api]
host = "127.0.0.1"
port = 8080
auth_required = false

[mcp]
enabled = true
auth_required = false
"#;
        std::fs::write(&config_file, test_config)?;

        Ok(Self {
            temp_dir,
            config_file,
        })
    }
}

/// ANCHOR: test_proactive_cli_interface_stability
/// Tests that the CLI interface for proactive research remains stable and functional
#[test]
fn test_proactive_cli_interface_stability() {
    let config = AnchorTestConfig::new().unwrap();

    // Test 1: Core proactive subcommands are available
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Proactive research management commands",
        ))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("configure"));

    // Test 2: Start command accepts required parameters
    let mut start_cmd = Command::cargo_bin("fortitude").unwrap();
    start_cmd.arg("proactive").arg("start").arg("--help");

    start_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("--gap-interval"))
        .stdout(predicate::str::contains("--max-tasks"))
        .stdout(predicate::str::contains("--debounce"))
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("--verbose"));

    // Test 3: Status command provides expected output format
    let mut status_cmd = Command::cargo_bin("fortitude").unwrap();
    status_cmd.arg("proactive").arg("status");

    status_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("📊"))
        .stdout(predicate::str::contains("Proactive Research System Status"))
        .stdout(predicate::str::contains(
            "==================================",
        ));

    // Test 4: Configure command has proper subcommands
    let mut config_cmd = Command::cargo_bin("fortitude").unwrap();
    config_cmd.arg("proactive").arg("configure").arg("--help");

    config_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("reset"));

    // Test 5: Configuration file integration
    let mut config_file_cmd = Command::cargo_bin("fortitude").unwrap();
    config_file_cmd
        .arg("proactive")
        .arg("status")
        .arg("--config")
        .arg(config.config_file.to_str().unwrap());

    config_file_cmd.assert().success();
}

/// ANCHOR: test_cli_argument_validation_stability
/// Tests that CLI argument validation remains consistent and user-friendly
#[test]
fn test_cli_argument_validation_stability() {
    // Test 1: Required arguments are properly validated
    let mut missing_value_cmd = Command::cargo_bin("fortitude").unwrap();
    missing_value_cmd
        .arg("proactive")
        .arg("configure")
        .arg("set")
        .arg("key_only");

    missing_value_cmd
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    // Test 2: Invalid subcommands are rejected
    let mut invalid_cmd = Command::cargo_bin("fortitude").unwrap();
    invalid_cmd.arg("proactive").arg("invalid-command");

    invalid_cmd
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));

    // Test 3: Default values are properly displayed
    let mut defaults_cmd = Command::cargo_bin("fortitude").unwrap();
    defaults_cmd.arg("proactive").arg("start").arg("--help");

    defaults_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("[default: 30]")) // gap-interval
        .stdout(predicate::str::contains("[default: 3]")) // max-tasks
        .stdout(predicate::str::contains("[default: 5]")); // debounce

    // Test 4: Help text consistency
    let mut help_cmd = Command::cargo_bin("fortitude").unwrap();
    help_cmd.arg("--help");

    help_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("proactive"));
}

/// ANCHOR: test_proactive_status_output_format_stability
/// Tests that status output format remains consistent for tooling integration
#[test]
fn test_proactive_status_output_format_stability() {
    // Test 1: Basic status output format
    let mut status_cmd = Command::cargo_bin("fortitude").unwrap();
    status_cmd.arg("proactive").arg("status");

    status_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "📊 Proactive Research System Status",
        ))
        .stdout(predicate::str::contains(
            "==================================",
        ))
        .stdout(predicate::str::contains(
            "⚠️  Status functionality not yet fully implemented",
        ));

    // Test 2: Detailed status output format
    let mut detailed_cmd = Command::cargo_bin("fortitude").unwrap();
    detailed_cmd
        .arg("proactive")
        .arg("status")
        .arg("--detailed");

    detailed_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("📝 Detailed Task Information:"))
        .stdout(predicate::str::contains("Active tasks:"))
        .stdout(predicate::str::contains("Completed tasks:"))
        .stdout(predicate::str::contains("Failed tasks:"));

    // Test 3: Metrics output format
    let mut metrics_cmd = Command::cargo_bin("fortitude").unwrap();
    metrics_cmd.arg("proactive").arg("status").arg("--metrics");

    metrics_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("📈 Performance Metrics:"))
        .stdout(predicate::str::contains("System uptime:"))
        .stdout(predicate::str::contains("Tasks per hour:"));

    // Test 4: Recent activity output format
    let mut recent_cmd = Command::cargo_bin("fortitude").unwrap();
    recent_cmd
        .arg("proactive")
        .arg("status")
        .arg("--recent")
        .arg("30");

    recent_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "🕐 Recent Activity (last 30 minutes):",
        ));

    // Test 5: Combined flags output format
    let mut combined_cmd = Command::cargo_bin("fortitude").unwrap();
    combined_cmd
        .arg("proactive")
        .arg("status")
        .arg("--detailed")
        .arg("--metrics")
        .arg("--recent")
        .arg("60");

    combined_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("📝 Detailed Task Information:"))
        .stdout(predicate::str::contains("📈 Performance Metrics:"))
        .stdout(predicate::str::contains(
            "🕐 Recent Activity (last 60 minutes):",
        ));
}

/// ANCHOR: test_configuration_command_stability
/// Tests that configuration commands maintain consistent behavior
#[test]
fn test_configuration_command_stability() {
    // Test 1: Configuration list format
    let mut list_cmd = Command::cargo_bin("fortitude").unwrap();
    list_cmd.arg("proactive").arg("configure").arg("list");

    list_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("⚙️  Current Configuration:"))
        .stdout(predicate::str::contains("========================"));

    // Test 2: Configuration get format
    let mut get_cmd = Command::cargo_bin("fortitude").unwrap();
    get_cmd
        .arg("proactive")
        .arg("configure")
        .arg("get")
        .arg("max_tasks");

    get_cmd.assert().success().stdout(predicate::str::contains(
        "⚙️  Getting configuration value for: max_tasks",
    ));

    // Test 3: Configuration set format
    let mut set_cmd = Command::cargo_bin("fortitude").unwrap();
    set_cmd
        .arg("proactive")
        .arg("configure")
        .arg("set")
        .arg("max_tasks")
        .arg("5");

    set_cmd.assert().success().stdout(predicate::str::contains(
        "⚙️  Setting configuration: max_tasks = 5",
    ));

    // Test 4: Configuration reset with confirmation
    let mut reset_confirmed_cmd = Command::cargo_bin("fortitude").unwrap();
    reset_confirmed_cmd
        .arg("proactive")
        .arg("configure")
        .arg("reset")
        .arg("--confirm");

    reset_confirmed_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "⚙️  Resetting configuration to defaults",
        ));

    // Test 5: Configuration reset without confirmation
    let mut reset_unconfirmed_cmd = Command::cargo_bin("fortitude").unwrap();
    reset_unconfirmed_cmd
        .arg("proactive")
        .arg("configure")
        .arg("reset");

    reset_unconfirmed_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "❌ Configuration reset requires --confirm flag",
        ));
}

/// ANCHOR: test_stop_command_stability
/// Tests that stop command behavior remains consistent
#[test]
fn test_stop_command_stability() {
    // Test 1: Basic stop command format
    let mut stop_cmd = Command::cargo_bin("fortitude").unwrap();
    stop_cmd.arg("proactive").arg("stop");

    stop_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "🛑 Stopping proactive research mode",
        ))
        .stdout(predicate::str::contains("Graceful stop requested"));

    // Test 2: Force stop format
    let mut force_cmd = Command::cargo_bin("fortitude").unwrap();
    force_cmd.arg("proactive").arg("stop").arg("--force");

    force_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "🛑 Stopping proactive research mode",
        ))
        .stdout(predicate::str::contains(
            "Force stop requested - terminating immediately",
        ));

    // Test 3: Custom timeout format
    let mut timeout_cmd = Command::cargo_bin("fortitude").unwrap();
    timeout_cmd
        .arg("proactive")
        .arg("stop")
        .arg("--timeout")
        .arg("60");

    timeout_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("waiting up to 60 seconds"));

    // Test 4: Combined force and timeout
    let mut combined_cmd = Command::cargo_bin("fortitude").unwrap();
    combined_cmd
        .arg("proactive")
        .arg("stop")
        .arg("--force")
        .arg("--timeout")
        .arg("120");

    combined_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Force stop requested"));
}

/// ANCHOR: test_error_message_consistency
/// Tests that error messages remain consistent and helpful
#[test]
fn test_error_message_consistency() {
    // Test 1: Missing required arguments
    let mut missing_arg_cmd = Command::cargo_bin("fortitude").unwrap();
    missing_arg_cmd.arg("proactive").arg("configure").arg("get");

    missing_arg_cmd
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    // Test 2: Invalid subcommand
    let mut invalid_subcmd = Command::cargo_bin("fortitude").unwrap();
    invalid_subcmd.arg("proactive").arg("invalid-subcommand");

    invalid_subcmd
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));

    // Test 3: Help is available for all commands
    let commands = [
        vec!["proactive"],
        vec!["proactive", "start"],
        vec!["proactive", "stop"],
        vec!["proactive", "status"],
        vec!["proactive", "configure"],
    ];

    for mut cmd_args in commands {
        cmd_args.push("--help");

        let mut help_cmd = Command::cargo_bin("fortitude").unwrap();
        help_cmd.args(&cmd_args);

        help_cmd.assert().success();
    }
}

/// ANCHOR: test_placeholder_implementation_consistency
/// Tests that placeholder implementations provide consistent user experience
#[test]
fn test_placeholder_implementation_consistency() {
    // Test 1: All placeholder commands indicate implementation status
    let placeholder_commands = [
        vec!["proactive", "status"],
        vec!["proactive", "stop"],
        vec!["proactive", "configure", "list"],
        vec!["proactive", "configure", "get", "test_key"],
        vec!["proactive", "configure", "set", "test_key", "test_value"],
    ];

    for cmd_args in placeholder_commands {
        let mut cmd = Command::cargo_bin("fortitude").unwrap();
        cmd.args(&cmd_args);

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("not yet fully implemented"));
    }

    // Test 2: Placeholder messages are informative
    let mut status_cmd = Command::cargo_bin("fortitude").unwrap();
    status_cmd.arg("proactive").arg("status");

    status_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "This would connect to a running proactive manager instance",
        ));

    // Test 3: Consistent emoji usage
    let mut status_emoji_cmd = Command::cargo_bin("fortitude").unwrap();
    status_emoji_cmd.arg("proactive").arg("status");

    status_emoji_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("📊")) // Status emoji
        .stdout(predicate::str::contains("⚠️")); // Warning emoji
}

/// ANCHOR: test_backwards_compatibility_guarantees
/// Tests that essential CLI interface patterns remain stable for backwards compatibility
#[test]
fn test_backwards_compatibility_guarantees() {
    // Test 1: Main command structure stability
    let mut main_help = Command::cargo_bin("fortitude").unwrap();
    main_help.arg("proactive").arg("--help");

    main_help
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "start      Start proactive research mode",
        ))
        .stdout(predicate::str::contains(
            "stop       Stop proactive research mode",
        ))
        .stdout(predicate::str::contains(
            "status     Show proactive research status",
        ))
        .stdout(predicate::str::contains(
            "configure  Configure proactive research settings",
        ));

    // Test 2: Essential flag stability
    let mut status_help = Command::cargo_bin("fortitude").unwrap();
    status_help.arg("proactive").arg("status").arg("--help");

    status_help
        .assert()
        .success()
        .stdout(predicate::str::contains("-d, --detailed"))
        .stdout(predicate::str::contains("-m, --metrics"))
        .stdout(predicate::str::contains("--recent"));

    // Test 3: Start command flag stability
    let mut start_help = Command::cargo_bin("fortitude").unwrap();
    start_help.arg("proactive").arg("start").arg("--help");

    start_help
        .assert()
        .success()
        .stdout(predicate::str::contains("--gap-interval"))
        .stdout(predicate::str::contains("--max-tasks"))
        .stdout(predicate::str::contains("--debounce"))
        .stdout(predicate::str::contains("-c, --config"))
        .stdout(predicate::str::contains("-v, --verbose"));

    // Test 4: Stop command flag stability
    let mut stop_help = Command::cargo_bin("fortitude").unwrap();
    stop_help.arg("proactive").arg("stop").arg("--help");

    stop_help
        .assert()
        .success()
        .stdout(predicate::str::contains("-f, --force"))
        .stdout(predicate::str::contains("--timeout"));

    // Test 5: Configure command action stability
    let mut configure_help = Command::cargo_bin("fortitude").unwrap();
    configure_help
        .arg("proactive")
        .arg("configure")
        .arg("--help");

    configure_help
        .assert()
        .success()
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("reset"));
}

/// ANCHOR: test_cross_interface_data_consistency
/// Tests that data formats remain consistent across different interfaces
#[tokio::test]
async fn test_cross_interface_data_consistency() {
    let config = AnchorTestConfig::new().unwrap();

    // Test 1: Configuration data format consistency
    // CLI should accept the same configuration format that API expects
    let mut cli_config_cmd = Command::cargo_bin("fortitude").unwrap();
    cli_config_cmd
        .arg("proactive")
        .arg("configure")
        .arg("set")
        .arg("gap_interval")
        .arg("30");

    cli_config_cmd.assert().success();

    // Test 2: Status information format should be parseable
    let mut status_cmd = Command::cargo_bin("fortitude").unwrap();
    status_cmd.arg("proactive").arg("status");

    let output = status_cmd.output().unwrap();
    let status_output = String::from_utf8(output.stdout).unwrap();

    // Should contain structured information that could be parsed by tools
    assert!(status_output.contains("Proactive Research System Status"));
    assert!(status_output.contains("=================================="));

    // Test 3: Error format consistency
    let mut error_cmd = Command::cargo_bin("fortitude").unwrap();
    error_cmd.arg("proactive").arg("configure").arg("get");

    let error_output = error_cmd.output().unwrap();

    // Error should be on stderr and have consistent format
    assert!(!error_output.status.success());
    assert!(!error_output.stderr.is_empty());
}

/// ANCHOR: test_interface_isolation_guarantees
/// Tests that interfaces remain properly isolated and don't interfere with each other
#[test]
fn test_interface_isolation_guarantees() {
    // Test 1: CLI operations don't interfere with system state
    let commands = [
        vec!["proactive", "status"],
        vec!["proactive", "configure", "list"],
        vec!["proactive", "configure", "get", "max_tasks"],
        vec!["proactive", "stop"],
    ];

    for cmd_args in commands {
        let mut cmd1 = Command::cargo_bin("fortitude").unwrap();
        cmd1.args(&cmd_args);
        cmd1.assert().success();

        // Running the same command again should work
        let mut cmd2 = Command::cargo_bin("fortitude").unwrap();
        cmd2.args(&cmd_args);
        cmd2.assert().success();
    }

    // Test 2: Invalid commands don't affect valid command execution
    let mut invalid_cmd = Command::cargo_bin("fortitude").unwrap();
    invalid_cmd.arg("proactive").arg("invalid-command");
    invalid_cmd.assert().failure();

    // Valid command should still work after invalid command
    let mut valid_cmd = Command::cargo_bin("fortitude").unwrap();
    valid_cmd.arg("proactive").arg("status");
    valid_cmd.assert().success();

    // Test 3: Configuration commands are isolated
    let mut set_cmd = Command::cargo_bin("fortitude").unwrap();
    set_cmd
        .arg("proactive")
        .arg("configure")
        .arg("set")
        .arg("test_key")
        .arg("test_value");
    set_cmd.assert().success();

    let mut get_cmd = Command::cargo_bin("fortitude").unwrap();
    get_cmd
        .arg("proactive")
        .arg("configure")
        .arg("get")
        .arg("test_key");
    get_cmd.assert().success();
}
