// ABOUTME: Demonstration of configurable gap detection system with various presets and custom rules
//! This example shows how to use the configurable gap detection system with different presets,
//! custom rules, and threshold adjustments for various analysis scenarios.

use fortitude::proactive::{
    ConfigurableAnalysisError, ConfigurableGapAnalyzer, CustomPriorityRule, GapDetectionConfig,
    QualityThresholds,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), ConfigurableAnalysisError> {
    println!("🔧 Configurable Gap Detection System Demo");
    println!("==========================================\n");

    // Example 1: Using preset configurations
    println!("📋 Example 1: Preset Configurations");
    println!("-----------------------------------");

    demo_preset_configurations().await?;

    // Example 2: Custom configuration with rules
    println!("\n📋 Example 2: Custom Configuration with Rules");
    println!("---------------------------------------------");

    demo_custom_configuration().await?;

    // Example 3: Performance vs Accuracy trade-offs
    println!("\n📋 Example 3: Performance vs Accuracy Trade-offs");
    println!("------------------------------------------------");

    demo_performance_accuracy_tradeoffs().await?;

    // Example 4: Rule-based filtering and priority adjustment
    println!("\n📋 Example 4: Rule-based Filtering and Priority Adjustment");
    println!("----------------------------------------------------------");

    demo_rule_based_filtering().await?;

    // Example 5: Configuration validation and error handling
    println!("\n📋 Example 5: Configuration Validation and Error Handling");
    println!("---------------------------------------------------------");

    demo_configuration_validation().await?;

    println!("\n✅ Demo completed successfully!");

    Ok(())
}

/// Demonstrate different preset configurations
async fn demo_preset_configurations() -> Result<(), ConfigurableAnalysisError> {
    let presets = vec![
        "rust",
        "performance",
        "accuracy",
        "minimal",
        "comprehensive",
    ];

    for preset_name in presets {
        println!("🎛️  Testing '{}' preset:", preset_name);

        let analyzer = ConfigurableGapAnalyzer::with_preset(preset_name, None)?;

        // Show key configuration settings
        let config = analyzer.config();
        println!("   - Semantic analysis: {}", config.semantic_config.enabled);
        println!(
            "   - Max total time: {}ms",
            config.performance_config.max_total_time_ms
        );
        println!(
            "   - Min confidence: {:.2}",
            config.detection_settings.min_confidence_threshold
        );
        println!(
            "   - Max gaps per file: {}",
            config.filtering_config.max_gaps_per_file
        );

        // Count enabled rules
        let enabled_rules = count_enabled_rules(config);
        println!("   - Enabled rule types: {}", enabled_rules);
        println!();
    }

    Ok(())
}

/// Demonstrate custom configuration with specialized rules
async fn demo_custom_configuration() -> Result<(), ConfigurableAnalysisError> {
    println!("🛠️  Creating custom configuration for security-focused analysis...");

    let mut config = GapDetectionConfig::for_rust_project();

    // Enhance TODO detection for security keywords
    config.detection_rules.todo_rules.urgent_keywords.extend([
        "SECURITY".to_string(),
        "VULNERABILITY".to_string(),
        "UNSAFE".to_string(),
        "PRIVILEGE".to_string(),
    ]);
    config.detection_rules.todo_rules.urgent_keyword_boost = 0.5; // Higher boost

    // Add custom priority rule for security-related files
    config
        .priority_config
        .custom_priority_rules
        .push(CustomPriorityRule {
            name: "security_file_boost".to_string(),
            file_pattern: Some(r".*/(auth|security|crypto).*\.rs$".to_string()),
            gap_type_pattern: None,
            content_pattern: Some(r"(?i)(password|token|key|secret|auth)".to_string()),
            priority_adjustment: 2,
            description: "Boost priority for security-related files and content".to_string(),
        });

    // Stricter quality thresholds
    config.filtering_config.quality_thresholds.min_quality_score = 0.8;
    config
        .filtering_config
        .quality_thresholds
        .min_description_length = 10;

    // Exclude test-only and debug code
    config
        .filtering_config
        .exclusion_rules
        .exclude_patterns
        .extend([
            r"(?i)test.*only".to_string(),
            r"(?i)debug.*mode".to_string(),
            r"(?i)development.*only".to_string(),
        ]);

    let analyzer = ConfigurableGapAnalyzer::new(config, None)?;

    println!("✅ Custom security-focused configuration created:");
    println!(
        "   - Enhanced urgent keywords: {:?}",
        analyzer.config().detection_rules.todo_rules.urgent_keywords
    );
    println!(
        "   - Custom priority rules: {} rules",
        analyzer
            .config()
            .priority_config
            .custom_priority_rules
            .len()
    );
    println!(
        "   - Quality threshold: {:.2}",
        analyzer
            .config()
            .filtering_config
            .quality_thresholds
            .min_quality_score
    );

    Ok(())
}

/// Demonstrate performance vs accuracy trade-offs
async fn demo_performance_accuracy_tradeoffs() -> Result<(), ConfigurableAnalysisError> {
    println!("⚡ Performance Configuration:");
    let perf_analyzer = ConfigurableGapAnalyzer::with_preset("performance", None)?;
    print_performance_metrics(perf_analyzer.config(), "Performance");

    println!("\n🎯 Accuracy Configuration:");
    let accuracy_analyzer = ConfigurableGapAnalyzer::with_preset("accuracy", None)?;
    print_performance_metrics(accuracy_analyzer.config(), "Accuracy");

    println!("\n🔬 Custom Balanced Configuration:");
    let mut balanced_config = GapDetectionConfig::default();

    // Balance performance and accuracy
    balanced_config.performance_config.max_total_time_ms = 750; // Between perf and accuracy
    balanced_config.semantic_config.max_analysis_time_ms = 75; // Moderate semantic time
    balanced_config.semantic_config.max_related_documents = 8; // Reasonable related docs
    balanced_config.filtering_config.max_gaps_per_file = 30; // Moderate gap limit
    balanced_config.detection_settings.min_confidence_threshold = 0.55; // Balanced confidence

    let balanced_analyzer = ConfigurableGapAnalyzer::new(balanced_config, None)?;
    print_performance_metrics(balanced_analyzer.config(), "Balanced");

    Ok(())
}

/// Demonstrate rule-based filtering and priority adjustment
async fn demo_rule_based_filtering() -> Result<(), ConfigurableAnalysisError> {
    println!("🎯 Creating configuration with advanced filtering rules...");

    let mut config = GapDetectionConfig::default();

    // Configure quality thresholds
    config.filtering_config.quality_thresholds = QualityThresholds {
        min_quality_score: 0.7,
        min_content_length: 15,
        max_content_length: 2000,
        min_description_length: 8,
    };

    // Add exclusion rules for non-actionable content
    config
        .filtering_config
        .exclusion_rules
        .non_actionable_keywords
        .extend([
            "someday".to_string(),
            "maybe".to_string(),
            "nice to have".to_string(),
            "wishlist".to_string(),
            "future".to_string(),
        ]);

    // Configure priority boosts
    config.priority_config.priority_boosts.urgent_keyword_boost = 0.6;
    config.priority_config.priority_boosts.high_confidence_boost = 0.4;
    config.priority_config.priority_boosts.critical_path_boost = 0.8;

    // Add custom priority rules
    config.priority_config.custom_priority_rules.extend([
        CustomPriorityRule {
            name: "critical_path_boost".to_string(),
            file_pattern: Some(r".*/src/(main|lib|core)\.rs$".to_string()),
            gap_type_pattern: None,
            content_pattern: None,
            priority_adjustment: 1,
            description: "Boost priority for main entry point files".to_string(),
        },
        CustomPriorityRule {
            name: "documentation_penalty".to_string(),
            file_pattern: None,
            gap_type_pattern: Some("MissingDocumentation".to_string()),
            content_pattern: Some(r"(?i)(private|internal|helper)".to_string()),
            priority_adjustment: -1,
            description: "Lower priority for private/internal documentation gaps".to_string(),
        },
    ]);

    let analyzer = ConfigurableGapAnalyzer::new(config, None)?;

    println!("✅ Advanced filtering configuration created:");
    println!("   - Quality thresholds configured");
    println!(
        "   - {} exclusion keywords",
        analyzer
            .config()
            .filtering_config
            .exclusion_rules
            .non_actionable_keywords
            .len()
    );
    println!(
        "   - {} custom priority rules",
        analyzer
            .config()
            .priority_config
            .custom_priority_rules
            .len()
    );
    println!("   - Priority boost settings optimized");

    Ok(())
}

/// Demonstrate configuration validation and error handling
async fn demo_configuration_validation() -> Result<(), ConfigurableAnalysisError> {
    println!("🔍 Testing configuration validation...");

    // Test 1: Valid configuration
    println!("\n✅ Test 1: Valid configuration");
    let valid_config = GapDetectionConfig::for_rust_project();
    match valid_config.validate() {
        Ok(()) => println!("   ✓ Configuration validation passed"),
        Err(e) => println!("   ✗ Unexpected validation error: {}", e),
    }

    // Test 2: Invalid threshold
    println!("\n❌ Test 2: Invalid threshold (should fail)");
    let mut invalid_config = GapDetectionConfig::default();
    invalid_config.detection_settings.min_confidence_threshold = 1.5; // Invalid: > 1.0
    match invalid_config.validate() {
        Ok(()) => println!("   ✗ Validation should have failed"),
        Err(e) => println!("   ✓ Validation correctly failed: {}", e),
    }

    // Test 3: Invalid regex pattern
    println!("\n❌ Test 3: Invalid regex pattern (should fail)");
    let mut regex_config = GapDetectionConfig::default();
    regex_config
        .detection_rules
        .todo_rules
        .custom_patterns
        .push("[invalid regex".to_string());
    match regex_config.validate() {
        Ok(()) => println!("   ✗ Validation should have failed"),
        Err(e) => println!("   ✓ Validation correctly failed: {}", e),
    }

    // Test 4: Invalid performance limit
    println!("\n❌ Test 4: Invalid performance limit (should fail)");
    let mut perf_config = GapDetectionConfig::default();
    perf_config.performance_config.max_total_time_ms = 0; // Invalid: must be > 0
    match perf_config.validate() {
        Ok(()) => println!("   ✗ Validation should have failed"),
        Err(e) => println!("   ✓ Validation correctly failed: {}", e),
    }

    // Test 5: Invalid priority bounds
    println!("\n❌ Test 5: Invalid priority bounds (should fail)");
    let mut priority_config = GapDetectionConfig::default();
    priority_config.priority_config.min_priority = 8;
    priority_config.priority_config.max_priority = 5; // Invalid: min > max
    match priority_config.validate() {
        Ok(()) => println!("   ✗ Validation should have failed"),
        Err(e) => println!("   ✓ Validation correctly failed: {}", e),
    }

    // Test 6: Configuration serialization/deserialization
    println!("\n💾 Test 6: Configuration serialization");
    let original_config = GapDetectionConfig::for_rust_project();
    match serde_json::to_string_pretty(&original_config) {
        Ok(json) => {
            println!(
                "   ✓ Configuration serialized successfully ({} chars)",
                json.len()
            );
            match serde_json::from_str::<GapDetectionConfig>(&json) {
                Ok(deserialized) => match deserialized.validate() {
                    Ok(()) => println!("   ✓ Deserialized configuration is valid"),
                    Err(e) => println!("   ✗ Deserialized configuration invalid: {}", e),
                },
                Err(e) => println!("   ✗ Deserialization failed: {}", e),
            }
        }
        Err(e) => println!("   ✗ Serialization failed: {}", e),
    }

    Ok(())
}

/// Helper function to count enabled rules
fn count_enabled_rules(config: &GapDetectionConfig) -> usize {
    let mut count = 0;
    if config.detection_rules.todo_rules.enabled {
        count += 1;
    }
    if config.detection_rules.documentation_rules.enabled {
        count += 1;
    }
    if config.detection_rules.technology_rules.enabled {
        count += 1;
    }
    if config.detection_rules.api_rules.enabled {
        count += 1;
    }
    if config.detection_rules.config_rules.enabled {
        count += 1;
    }
    if config.semantic_config.enabled {
        count += 1;
    }
    if config.filtering_config.enabled {
        count += 1;
    }
    count
}

/// Helper function to print performance metrics comparison
fn print_performance_metrics(config: &GapDetectionConfig, config_name: &str) {
    println!("   {} Settings:", config_name);
    println!(
        "     - Max total time: {}ms",
        config.performance_config.max_total_time_ms
    );
    println!(
        "     - Semantic time: {}ms",
        config.semantic_config.max_analysis_time_ms
    );
    println!(
        "     - Max related docs: {}",
        config.semantic_config.max_related_documents
    );
    println!("     - Batch size: {}", config.semantic_config.batch_size);
    println!(
        "     - Gap validation threshold: {:.2}",
        config.semantic_config.gap_validation_threshold
    );
    println!(
        "     - Related content threshold: {:.2}",
        config.semantic_config.related_content_threshold
    );
    println!(
        "     - Max gaps per file: {}",
        config.filtering_config.max_gaps_per_file
    );
}
