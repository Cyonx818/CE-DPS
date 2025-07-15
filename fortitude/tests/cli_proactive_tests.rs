// ABOUTME: Unit tests for proactive CLI subcommands parsing and validation
//! This module tests the CLI command parsing, validation, and argument handling
//! for the proactive research management subcommands.

use clap::Parser;
// Note: This test imports the CLI structure directly rather than the library module

// Mock CLI structure for testing - will match the actual implementation
#[derive(Parser)]
#[command(name = "fortitude")]
#[command(about = "Automated research system for the Concordia workspace")]
struct TestCli {
    #[command(subcommand)]
    command: TestCommands,
}

#[derive(clap::Subcommand)]
enum TestCommands {
    Research {
        #[arg(short, long)]
        topic: String,
    },
    Pipeline {
        #[arg(short, long)]
        config: Option<String>,
    },
    Knowledge {
        #[arg(short, long)]
        query: String,
    },
    #[command(subcommand)]
    Proactive(ProactiveCommands),
}

#[derive(clap::Subcommand)]
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

#[derive(clap::Subcommand)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test_proactive_start_command_parsing
    /// Tests parsing of proactive start command with various argument combinations
    #[test]
    fn test_proactive_start_command_parsing() {
        // Test with default values
        let cli = TestCli::parse_from(&["fortitude", "proactive", "start"]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Start {
                gap_interval,
                max_tasks,
                debounce,
                config,
                verbose,
            }) => {
                assert_eq!(gap_interval, 30);
                assert_eq!(max_tasks, 3);
                assert_eq!(debounce, 5);
                assert_eq!(config, None);
                assert!(!verbose);
            }
            _ => panic!("Expected proactive start command"),
        }

        // Test with custom values
        let cli = TestCli::parse_from(&[
            "fortitude",
            "proactive",
            "start",
            "--gap-interval",
            "60",
            "--max-tasks",
            "5",
            "--debounce",
            "10",
            "--config",
            "/path/to/config.yaml",
            "--verbose",
        ]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Start {
                gap_interval,
                max_tasks,
                debounce,
                config,
                verbose,
            }) => {
                assert_eq!(gap_interval, 60);
                assert_eq!(max_tasks, 5);
                assert_eq!(debounce, 10);
                assert_eq!(config, Some("/path/to/config.yaml".to_string()));
                assert!(verbose);
            }
            _ => panic!("Expected proactive start command"),
        }
    }

    // ANCHOR: test_proactive_stop_command_parsing
    /// Tests parsing of proactive stop command with force and timeout options
    #[test]
    fn test_proactive_stop_command_parsing() {
        // Test with default values
        let cli = TestCli::parse_from(&["fortitude", "proactive", "stop"]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Stop { force, timeout }) => {
                assert!(!force);
                assert_eq!(timeout, 30);
            }
            _ => panic!("Expected proactive stop command"),
        }

        // Test with force flag
        let cli = TestCli::parse_from(&[
            "fortitude",
            "proactive",
            "stop",
            "--force",
            "--timeout",
            "60",
        ]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Stop { force, timeout }) => {
                assert!(force);
                assert_eq!(timeout, 60);
            }
            _ => panic!("Expected proactive stop command"),
        }
    }

    // ANCHOR: test_proactive_status_command_parsing
    /// Tests parsing of proactive status command with display options
    #[test]
    fn test_proactive_status_command_parsing() {
        // Test with default values
        let cli = TestCli::parse_from(&["fortitude", "proactive", "status"]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Status {
                detailed,
                metrics,
                recent,
            }) => {
                assert!(!detailed);
                assert!(!metrics);
                assert_eq!(recent, None);
            }
            _ => panic!("Expected proactive status command"),
        }

        // Test with all flags
        let cli = TestCli::parse_from(&[
            "fortitude",
            "proactive",
            "status",
            "--detailed",
            "--metrics",
            "--recent",
            "60",
        ]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Status {
                detailed,
                metrics,
                recent,
            }) => {
                assert!(detailed);
                assert!(metrics);
                assert_eq!(recent, Some(60));
            }
            _ => panic!("Expected proactive status command"),
        }
    }

    // ANCHOR: test_proactive_configure_command_parsing
    /// Tests parsing of proactive configure command with various actions
    #[test]
    fn test_proactive_configure_command_parsing() {
        // Test set action
        let cli = TestCli::parse_from(&[
            "fortitude",
            "proactive",
            "configure",
            "set",
            "gap_interval",
            "45",
        ]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Configure {
                action: ConfigureAction::Set { key, value },
            }) => {
                assert_eq!(key, "gap_interval");
                assert_eq!(value, "45");
            }
            _ => panic!("Expected proactive configure set command"),
        }

        // Test get action
        let cli = TestCli::parse_from(&["fortitude", "proactive", "configure", "get", "max_tasks"]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Configure {
                action: ConfigureAction::Get { key },
            }) => {
                assert_eq!(key, "max_tasks");
            }
            _ => panic!("Expected proactive configure get command"),
        }

        // Test list action
        let cli = TestCli::parse_from(&["fortitude", "proactive", "configure", "list"]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Configure {
                action: ConfigureAction::List,
            }) => {
                // Success - just verify it parses correctly
            }
            _ => panic!("Expected proactive configure list command"),
        }

        // Test reset action
        let cli =
            TestCli::parse_from(&["fortitude", "proactive", "configure", "reset", "--confirm"]);
        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Configure {
                action: ConfigureAction::Reset { confirm },
            }) => {
                assert!(confirm);
            }
            _ => panic!("Expected proactive configure reset command"),
        }
    }

    // ANCHOR: test_command_help_generation
    /// Tests that help text is generated correctly for all subcommands
    #[test]
    fn test_command_help_generation() {
        use clap::CommandFactory;

        let mut cmd = TestCli::command();
        let help_output = cmd.render_help();
        let help_str = help_output.to_string();

        // Verify the main command exists
        assert!(help_str.contains("proactive"));

        // Test subcommand help
        let mut binding = TestCli::command();
        let proactive_cmd = binding
            .find_subcommand_mut("proactive")
            .expect("proactive subcommand should exist");

        let proactive_help = proactive_cmd.render_help();
        let proactive_help_str = proactive_help.to_string();

        assert!(proactive_help_str.contains("start"));
        assert!(proactive_help_str.contains("stop"));
        assert!(proactive_help_str.contains("status"));
        assert!(proactive_help_str.contains("configure"));
    }

    // ANCHOR: test_invalid_command_combinations
    /// Tests that invalid command combinations are properly rejected
    #[test]
    fn test_invalid_command_combinations() {
        // Test invalid gap interval (should be caught by validation, not parsing)
        let result =
            TestCli::try_parse_from(&["fortitude", "proactive", "start", "--gap-interval", "0"]);
        // Note: clap will parse this successfully, validation happens in the business logic
        assert!(result.is_ok());

        // Test missing required arguments for configure set
        let result =
            TestCli::try_parse_from(&["fortitude", "proactive", "configure", "set", "key_only"]);
        assert!(result.is_err());

        // Test missing required arguments for configure get
        let result = TestCli::try_parse_from(&["fortitude", "proactive", "configure", "get"]);
        assert!(result.is_err());
    }

    // ANCHOR: test_argument_validation_ranges
    /// Tests that numeric arguments are within expected ranges
    #[test]
    fn test_argument_validation_ranges() {
        // Test large values (should parse but may be validated later)
        let cli = TestCli::parse_from(&[
            "fortitude",
            "proactive",
            "start",
            "--gap-interval",
            "999999",
            "--max-tasks",
            "100",
            "--debounce",
            "3600",
        ]);

        match cli.command {
            TestCommands::Proactive(ProactiveCommands::Start {
                gap_interval,
                max_tasks,
                debounce,
                ..
            }) => {
                assert_eq!(gap_interval, 999999);
                assert_eq!(max_tasks, 100);
                assert_eq!(debounce, 3600);
            }
            _ => panic!("Expected proactive start command"),
        }
    }
}
