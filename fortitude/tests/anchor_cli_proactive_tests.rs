// ABOUTME: Anchor tests for proactive CLI functionality to ensure critical features remain stable
//! This module provides anchor tests for the proactive CLI subcommands to ensure that
//! the CLI interface remains functional and backwards compatible across changes.

use assert_cmd::Command;
use predicates::prelude::*;

// ANCHOR: test_proactive_cli_help_commands_available
/// Tests that all proactive CLI subcommands are available and display help correctly
#[test]
fn test_proactive_cli_help_commands_available() {
    // Test main proactive help
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

    // Test start subcommand help
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("start").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start proactive research mode"))
        .stdout(predicate::str::contains("gap-interval"))
        .stdout(predicate::str::contains("max-tasks"))
        .stdout(predicate::str::contains("debounce"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("verbose"));

    // Test status subcommand help
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show proactive research status"))
        .stdout(predicate::str::contains("detailed"))
        .stdout(predicate::str::contains("metrics"))
        .stdout(predicate::str::contains("recent"));

    // Test configure subcommand help
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("configure").arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "Configure proactive research settings",
    ));
}

// ANCHOR: test_proactive_status_command_execution
/// Tests that the proactive status command executes successfully with various options
#[test]
fn test_proactive_status_command_execution() {
    // Test basic status command
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Proactive Research System Status"))
        .stdout(predicate::str::contains(
            "==================================",
        ));

    // Test status with detailed flag
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status").arg("--detailed");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Detailed Task Information:"))
        .stdout(predicate::str::contains("Active tasks:"))
        .stdout(predicate::str::contains("Completed tasks:"));

    // Test status with metrics flag
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status").arg("--metrics");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Performance Metrics:"))
        .stdout(predicate::str::contains("System uptime:"))
        .stdout(predicate::str::contains("Tasks per hour:"));

    // Test status with recent activity filter
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status").arg("--recent").arg("60");

    cmd.assert().success().stdout(predicate::str::contains(
        "Recent Activity (last 60 minutes):",
    ));

    // Test status with all flags combined
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive")
        .arg("status")
        .arg("--detailed")
        .arg("--metrics")
        .arg("--recent")
        .arg("30");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Detailed Task Information:"))
        .stdout(predicate::str::contains("Performance Metrics:"))
        .stdout(predicate::str::contains(
            "Recent Activity (last 30 minutes):",
        ));
}

// ANCHOR: test_proactive_configure_command_execution
/// Tests that the proactive configure command executes successfully with various actions
#[test]
fn test_proactive_configure_command_execution() {
    // Test configure list command
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("configure").arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Current Configuration:"))
        .stdout(predicate::str::contains("========================"));

    // Test configure get command
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive")
        .arg("configure")
        .arg("get")
        .arg("max_tasks");

    cmd.assert().success().stdout(predicate::str::contains(
        "Getting configuration value for: max_tasks",
    ));

    // Test configure set command
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive")
        .arg("configure")
        .arg("set")
        .arg("max_tasks")
        .arg("5");

    cmd.assert().success().stdout(predicate::str::contains(
        "Setting configuration: max_tasks = 5",
    ));

    // Test configure reset with confirmation
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive")
        .arg("configure")
        .arg("reset")
        .arg("--confirm");

    cmd.assert().success().stdout(predicate::str::contains(
        "Resetting configuration to defaults",
    ));

    // Test configure reset without confirmation (should fail)
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("configure").arg("reset");

    cmd.assert().success().stdout(predicate::str::contains(
        "Configuration reset requires --confirm flag",
    ));
}

// ANCHOR: test_proactive_stop_command_execution
/// Tests that the proactive stop command executes successfully with various options
#[test]
fn test_proactive_stop_command_execution() {
    // Test basic stop command
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("stop");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Stopping proactive research mode"))
        .stdout(predicate::str::contains("Graceful stop requested"));

    // Test stop with force flag
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("stop").arg("--force");

    cmd.assert().success().stdout(predicate::str::contains(
        "Force stop requested - terminating immediately",
    ));

    // Test stop with custom timeout
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("stop").arg("--timeout").arg("60");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("waiting up to 60 seconds"));

    // Test stop with force and timeout combined
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive")
        .arg("stop")
        .arg("--force")
        .arg("--timeout")
        .arg("120");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Force stop requested"));
}

// ANCHOR: test_proactive_cli_argument_validation
/// Tests that the CLI properly validates arguments and provides meaningful error messages
#[test]
fn test_proactive_cli_argument_validation() {
    // Test configure set without value (should fail)
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive")
        .arg("configure")
        .arg("set")
        .arg("max_tasks");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    // Test configure get without key (should fail)
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("configure").arg("get");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    // Test invalid subcommand (should fail)
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

// ANCHOR: test_proactive_default_argument_values
/// Tests that default argument values are correctly applied when not specified
#[test]
fn test_proactive_default_argument_values() {
    // We can't easily test the start command since it would try to run the actual system,
    // but we can verify the help text shows the correct defaults

    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("start").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[default: 30]")) // gap-interval default
        .stdout(predicate::str::contains("[default: 3]")) // max-tasks default
        .stdout(predicate::str::contains("[default: 5]")); // debounce default

    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("stop").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[default: 30]")); // timeout default
}

// ANCHOR: test_proactive_cli_error_handling
/// Tests that the CLI handles errors gracefully and provides user-friendly messages
#[test]
fn test_proactive_cli_error_handling() {
    // Test that placeholders are shown for not-yet-implemented functionality
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("not yet fully implemented"));

    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("stop");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("not yet fully implemented"));

    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("configure").arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("not yet fully implemented"));
}

// ANCHOR: test_proactive_cli_integration_patterns
/// Tests that the CLI follows consistent patterns and user experience guidelines
#[test]
fn test_proactive_cli_integration_patterns() {
    // Test that all commands use consistent emoji and formatting
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊")) // Status emoji
        .stdout(predicate::str::contains("⚠️")); // Warning emoji

    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("stop");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🛑")); // Stop emoji

    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("configure").arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("⚙️")); // Configure emoji

    // Test that all commands provide informative output
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status");

    cmd.assert().success().stdout(predicate::str::contains(
        "This would connect to a running proactive manager instance",
    ));
}

// ANCHOR: test_proactive_cli_backwards_compatibility
/// Tests that the CLI maintains backwards compatibility for essential functionality
#[test]
fn test_proactive_cli_backwards_compatibility() {
    // Test that the main proactive command structure remains stable
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("--help");

    cmd.assert()
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

    // Test that essential flags remain available
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("status").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("-d, --detailed"))
        .stdout(predicate::str::contains("-m, --metrics"))
        .stdout(predicate::str::contains("--recent"));

    // Test that start command essential options remain available
    let mut cmd = Command::cargo_bin("fortitude").unwrap();
    cmd.arg("proactive").arg("start").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--gap-interval"))
        .stdout(predicate::str::contains("--max-tasks"))
        .stdout(predicate::str::contains("--debounce"))
        .stdout(predicate::str::contains("-c, --config"))
        .stdout(predicate::str::contains("-v, --verbose"));
}
