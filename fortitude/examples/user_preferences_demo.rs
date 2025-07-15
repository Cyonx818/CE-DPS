// ABOUTME: Demonstration of user preference integration for priority customization
//! This example demonstrates how to use the user preference system to customize
//! priority scoring based on user workflow modes, expertise levels, and personal filters.

use fortitude::proactive::{
    DetectedGap, ExpertiseLevel, GapType, UserAwarePriorityScorer, UserPreferenceManager,
    UserPreferenceProfile, WorkflowMode,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 User Preference Integration Demo");
    println!("===================================\n");

    // 1. Create preference manager with temp storage
    let temp_dir = tempfile::TempDir::new()?;
    let storage_path = temp_dir.path().join("preferences");
    let manager = Arc::new(UserPreferenceManager::new(storage_path).await?);

    println!("📁 Created preference manager with temporary storage\n");

    // 2. Create preset profiles
    manager.create_preset_profiles().await?;
    println!(
        "✅ Created preset profiles: {:?}\n",
        manager.list_profiles().await
    );

    // 3. Create a custom profile for security-focused development
    let mut security_profile = UserPreferenceProfile::new(
        "security_dev".to_string(),
        "Security-focused development workflow".to_string(),
    );

    security_profile.workflow_mode = WorkflowMode::Development;
    security_profile.expertise_level = ExpertiseLevel::Expert;

    // Customize personal filters for security focus
    security_profile.personal_filters.priority_keywords = vec![
        "security".to_string(),
        "vulnerable".to_string(),
        "auth".to_string(),
        "crypto".to_string(),
    ];
    security_profile
        .personal_filters
        .priority_gap_types
        .insert(GapType::TodoComment);
    security_profile
        .personal_filters
        .priority_gap_types
        .insert(GapType::UndocumentedTechnology);

    // Apply workflow and expertise adjustments
    security_profile.apply_workflow_adjustments();
    security_profile.apply_expertise_adjustments();

    manager.update_profile(security_profile).await?;
    println!("🔒 Created custom security development profile");

    // 4. Set active profile to security development
    manager.set_active_profile("security_dev").await?;
    println!("🎯 Set active profile to 'security_dev'\n");

    // 5. Create user-aware priority scorer
    let scorer = UserAwarePriorityScorer::new(manager.clone()).await?;
    println!("🧮 Created user-aware priority scorer");

    // 6. Test gaps with different characteristics
    let test_gaps = vec![
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/auth.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Add authentication security validation".to_string(),
            description: "Security validation for authentication module".to_string(),
            confidence: 0.95,
            priority: 8,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::MissingDocumentation,
            file_path: PathBuf::from("src/utils.rs"),
            line_number: 15,
            column_number: None,
            context: "pub fn helper_function()".to_string(),
            description: "Missing documentation for utility function".to_string(),
            confidence: 0.7,
            priority: 5,
            metadata: HashMap::new(),
        },
        DetectedGap {
            gap_type: GapType::UndocumentedTechnology,
            file_path: PathBuf::from("src/crypto.rs"),
            line_number: 1,
            column_number: None,
            context: "use crypto_library::AES;".to_string(),
            description: "Undocumented crypto library usage".to_string(),
            confidence: 0.8,
            priority: 7,
            metadata: HashMap::new(),
        },
    ];

    println!("\n📊 Scoring gaps with user preferences:");
    println!("=====================================\n");

    for (i, gap) in test_gaps.iter().enumerate() {
        let result = scorer.score_gap_priority_with_preferences(gap).await?;

        println!(
            "Gap {}: {} ({})",
            i + 1,
            gap.description,
            GapTypeDisplay::to_string(&gap.gap_type)
        );
        println!("  📍 File: {}", gap.file_path.display());
        println!("  📈 Base Score: {:.2}", result.base_breakdown.final_score);
        println!("  ⚡ Enhanced Score: {:.2}", result.enhanced_score);
        println!("  🎛️  Applied Profile: {:?}", result.applied_profile);
        println!("  ⏱️  Processing Time: {:?}", result.processing_time);

        if !result.preference_adjustments.is_empty() {
            println!("  🔧 Preference Adjustments:");
            for (adjustment, value) in &result.preference_adjustments {
                println!("    • {}: {:.2}x", adjustment, value);
            }
        }
        println!();
    }

    // 7. Switch to documentation workflow mode
    println!("📝 Switching to documentation workflow...\n");
    manager.set_active_profile("documentation").await?;

    // Create new scorer with updated profile
    let doc_scorer = UserAwarePriorityScorer::new(manager.clone()).await?;

    // Test same gaps with documentation profile
    println!("📊 Re-scoring with documentation profile:");
    println!("========================================\n");

    for (i, gap) in test_gaps.iter().enumerate() {
        let result = doc_scorer.score_gap_priority_with_preferences(gap).await?;

        println!(
            "Gap {}: Enhanced Score with 'documentation' profile: {:.2}",
            i + 1,
            result.enhanced_score
        );
    }

    // 8. Performance demonstration
    println!("\n⚡ Performance Test:");
    println!("==================\n");

    let test_gap = &test_gaps[0];
    let iterations = 100;

    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = scorer.score_gap_priority_with_preferences(test_gap).await?;
    }
    let total_time = start.elapsed();
    let avg_time = total_time / iterations;

    println!("🏃 Scored {} gaps in {:?}", iterations, total_time);
    println!("📊 Average time per gap: {:?}", avg_time);
    println!(
        "✅ Performance target: <5ms per gap - {}",
        if avg_time.as_millis() < 5 {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    println!("\n🎉 User preference integration demo completed successfully!");

    Ok(())
}

// Helper trait implementation for better display
trait GapTypeDisplay {
    fn to_string(&self) -> String;
}

impl GapTypeDisplay for GapType {
    fn to_string(&self) -> String {
        match self {
            GapType::TodoComment => "TODO Comment".to_string(),
            GapType::MissingDocumentation => "Missing Documentation".to_string(),
            GapType::UndocumentedTechnology => "Undocumented Technology".to_string(),
            GapType::ApiDocumentationGap => "API Documentation Gap".to_string(),
            GapType::ConfigurationGap => "Configuration Gap".to_string(),
        }
    }
}
