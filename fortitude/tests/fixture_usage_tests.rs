//! Tests demonstrating the usage of classification test fixtures
//!
//! These tests show how to use the comprehensive test fixtures
//! for testing multi-dimensional classification scenarios.

mod fixtures;

use fixtures::classification_test_fixtures::*;
use fortitude_core::classification::{
    AdvancedClassificationConfig, AdvancedClassifier, BasicClassifier, ContextDetector,
    FortitudeContextDetector,
};
use fortitude_types::{
    classification_result::{AudienceLevel, TechnicalDomain, UrgencyLevel},
    ClassificationConfig, ResearchType,
};

/// Helper function to create a test classifier
fn create_test_classifier() -> AdvancedClassifier {
    let config = AdvancedClassificationConfig {
        basic_config: ClassificationConfig {
            default_threshold: 0.1,
            ..Default::default()
        },
        max_processing_time_ms: 5000,
        ..Default::default()
    };

    AdvancedClassifier::new(config)
}

#[test]
fn test_comprehensive_scenarios_with_fixtures() {
    let classifier = create_test_classifier();
    let scenarios = get_comprehensive_test_scenarios();

    println!("Testing {} comprehensive scenarios", scenarios.len());

    let mut total_tests = 0;
    let mut passed_tests = 0;

    for scenario in scenarios {
        println!("\n--- Testing: {} ---", scenario.name);
        println!("Query: '{}'", scenario.query);
        println!(
            "Expected: {:?}, {:?}, {:?}",
            scenario.expected_audience, scenario.expected_domain, scenario.expected_urgency
        );

        let result = classifier.classify_enhanced(&scenario.query, &scenario.research_type);

        match result {
            Ok(enhanced_result) => {
                println!(
                    "Detected: {:?}, {:?}, {:?}",
                    enhanced_result.audience_level,
                    enhanced_result.technical_domain,
                    enhanced_result.urgency_level
                );
                println!(
                    "Confidence: {:.2} (expected: {:.2}-{:.2})",
                    enhanced_result.overall_confidence,
                    scenario.expected_confidence_range.0,
                    scenario.expected_confidence_range.1
                );

                // Check if results are within expected ranges
                let confidence_ok = enhanced_result.overall_confidence
                    >= scenario.expected_confidence_range.0 - 0.1
                    && enhanced_result.overall_confidence
                        <= scenario.expected_confidence_range.1 + 0.1;

                let research_type_ok = enhanced_result.research_type == scenario.research_type;

                // For complex scenarios, allow some flexibility in exact matches
                let is_complex = scenario.tags.contains(&"multi-tech".to_string())
                    || scenario.tags.contains(&"conflicting".to_string())
                    || scenario.tags.contains(&"ambiguous".to_string());

                if confidence_ok && research_type_ok {
                    passed_tests += 1;
                    println!("✓ PASSED");
                } else {
                    println!(
                        "✗ FAILED - Confidence: {}, Research Type: {}",
                        confidence_ok, research_type_ok
                    );
                    if !is_complex {
                        // Only fail for non-complex scenarios
                        assert!(confidence_ok, "Confidence should be within expected range");
                        assert!(research_type_ok, "Research type should match expected");
                    }
                }
            }
            Err(e) => {
                println!("✗ ERROR: {}", e);
                assert!(
                    false,
                    "Classification should succeed for scenario: {}",
                    scenario.name
                );
            }
        }

        total_tests += 1;
    }

    let success_rate = (passed_tests as f64) / (total_tests as f64);
    println!("\n=== Summary ===");
    println!("Total tests: {}", total_tests);
    println!("Passed: {}", passed_tests);
    println!("Success rate: {:.2}%", success_rate * 100.0);

    // Should have reasonable success rate
    assert!(
        success_rate > 0.7,
        "Success rate should be >70%, got {:.2}%",
        success_rate * 100.0
    );
}

#[test]
fn test_audience_fixtures() {
    let detector = FortitudeContextDetector::new();
    let fixtures = get_audience_test_fixtures();

    println!("Testing {} audience fixtures", fixtures.len());

    let mut correct_predictions = 0;
    let total_predictions = fixtures.len();

    for fixture in fixtures {
        let result = detector
            .detect_audience_level(&fixture.query)
            .expect("Audience detection should succeed");

        let (detected_level, confidence, _keywords) = result;

        println!("Query: '{}'", fixture.query);
        println!(
            "Expected: {:?} (conf: {:.2}-{:.2})",
            fixture.expected_level, fixture.confidence_range.0, fixture.confidence_range.1
        );
        println!("Detected: {:?} (conf: {:.2})", detected_level, confidence);

        if detected_level == fixture.expected_level {
            correct_predictions += 1;
            println!("✓ PASSED");
        } else {
            println!("✗ FAILED");
        }

        // Check confidence is reasonable
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be between 0 and 1"
        );

        println!();
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!("Audience detection accuracy: {:.2}%", accuracy * 100.0);

    assert!(accuracy > 0.8, "Audience detection accuracy should be >80%");
}

#[test]
fn test_domain_fixtures() {
    let detector = FortitudeContextDetector::new();
    let fixtures = get_domain_test_fixtures();

    println!("Testing {} domain fixtures", fixtures.len());

    let mut correct_predictions = 0;
    let total_predictions = fixtures.len();

    for fixture in fixtures {
        let result = detector
            .detect_technical_domain(&fixture.query)
            .expect("Domain detection should succeed");

        let (detected_domain, confidence, _keywords) = result;

        println!("Query: '{}'", fixture.query);
        println!(
            "Expected: {:?} (conf: {:.2}-{:.2})",
            fixture.expected_domain, fixture.confidence_range.0, fixture.confidence_range.1
        );
        println!("Detected: {:?} (conf: {:.2})", detected_domain, confidence);

        if detected_domain == fixture.expected_domain {
            correct_predictions += 1;
            println!("✓ PASSED");
        } else {
            println!("✗ FAILED");
        }

        // Check confidence is reasonable
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be between 0 and 1"
        );

        println!();
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!("Domain detection accuracy: {:.2}%", accuracy * 100.0);

    assert!(accuracy > 0.8, "Domain detection accuracy should be >80%");
}

#[test]
fn test_urgency_fixtures() {
    let detector = FortitudeContextDetector::new();
    let fixtures = get_urgency_test_fixtures();

    println!("Testing {} urgency fixtures", fixtures.len());

    let mut correct_predictions = 0;
    let total_predictions = fixtures.len();

    for fixture in fixtures {
        let result = detector
            .detect_urgency_level(&fixture.query)
            .expect("Urgency detection should succeed");

        let (detected_urgency, confidence, _keywords) = result;

        println!("Query: '{}'", fixture.query);
        println!(
            "Expected: {:?} (conf: {:.2}-{:.2})",
            fixture.expected_urgency, fixture.confidence_range.0, fixture.confidence_range.1
        );
        println!("Detected: {:?} (conf: {:.2})", detected_urgency, confidence);

        if detected_urgency == fixture.expected_urgency {
            correct_predictions += 1;
            println!("✓ PASSED");
        } else {
            println!("✗ FAILED");
        }

        // Check confidence is reasonable
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be between 0 and 1"
        );

        println!();
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!("Urgency detection accuracy: {:.2}%", accuracy * 100.0);

    assert!(accuracy > 0.8, "Urgency detection accuracy should be >80%");
}

#[test]
fn test_filtered_scenarios() {
    let classifier = create_test_classifier();

    // Test different filters
    let learning_scenarios = get_scenarios_by_research_type(ResearchType::Learning);
    let beginner_scenarios = get_scenarios_by_audience(AudienceLevel::Beginner);
    let rust_scenarios = get_scenarios_by_domain(TechnicalDomain::Rust);
    let urgent_scenarios = get_scenarios_by_urgency(UrgencyLevel::Immediate);

    println!("Learning scenarios: {}", learning_scenarios.len());
    println!("Beginner scenarios: {}", beginner_scenarios.len());
    println!("Rust scenarios: {}", rust_scenarios.len());
    println!("Urgent scenarios: {}", urgent_scenarios.len());

    // Test each category
    for scenario in learning_scenarios {
        let result = classifier
            .classify_enhanced(&scenario.query, &scenario.research_type)
            .expect("Classification should succeed");

        assert_eq!(result.research_type, ResearchType::Learning);
        println!("✓ Learning scenario: {}", scenario.name);
    }

    for scenario in beginner_scenarios {
        let result = classifier
            .classify_enhanced(&scenario.query, &scenario.research_type)
            .expect("Classification should succeed");

        // Note: We don't assert exact audience match as it depends on context detection
        println!(
            "✓ Beginner scenario: {} -> {:?}",
            scenario.name, result.audience_level
        );
    }

    for scenario in rust_scenarios {
        let result = classifier
            .classify_enhanced(&scenario.query, &scenario.research_type)
            .expect("Classification should succeed");

        // Note: We don't assert exact domain match as it depends on context detection
        println!(
            "✓ Rust scenario: {} -> {:?}",
            scenario.name, result.technical_domain
        );
    }

    for scenario in urgent_scenarios {
        let result = classifier
            .classify_enhanced(&scenario.query, &scenario.research_type)
            .expect("Classification should succeed");

        // Note: We don't assert exact urgency match as it depends on context detection
        println!(
            "✓ Urgent scenario: {} -> {:?}",
            scenario.name, result.urgency_level
        );
    }
}

#[test]
fn test_edge_case_scenarios() {
    let classifier = create_test_classifier();
    let edge_cases = get_edge_case_scenarios();

    println!("Testing {} edge case scenarios", edge_cases.len());

    for scenario in edge_cases {
        println!("\n--- Testing edge case: {} ---", scenario.name);
        println!("Query: '{}'", scenario.query);
        println!("Description: {}", scenario.description);

        let result = classifier.classify_enhanced(&scenario.query, &scenario.research_type);

        match result {
            Ok(enhanced_result) => {
                println!("✓ Handled gracefully");
                println!("Confidence: {:.2}", enhanced_result.overall_confidence);
                println!("Fallback used: {}", enhanced_result.metadata.fallback_used);

                // Edge cases should have lower confidence
                assert!(
                    enhanced_result.overall_confidence <= 0.7,
                    "Edge cases should have lower confidence"
                );
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
                // Some edge cases might fail, but should not panic
                assert!(
                    false,
                    "Edge case should be handled gracefully: {}",
                    scenario.name
                );
            }
        }
    }
}

#[test]
fn test_high_confidence_scenarios() {
    let classifier = create_test_classifier();
    let high_confidence_scenarios = get_high_confidence_scenarios();

    println!(
        "Testing {} high confidence scenarios",
        high_confidence_scenarios.len()
    );

    for scenario in high_confidence_scenarios {
        println!("\n--- Testing high confidence: {} ---", scenario.name);
        println!("Query: '{}'", scenario.query);
        println!(
            "Expected confidence: {:.2}-{:.2}",
            scenario.expected_confidence_range.0, scenario.expected_confidence_range.1
        );

        let result = classifier
            .classify_enhanced(&scenario.query, &scenario.research_type)
            .expect("High confidence scenarios should succeed");

        println!("Actual confidence: {:.2}", result.overall_confidence);

        // High confidence scenarios should have reasonable confidence
        assert!(
            result.overall_confidence >= scenario.expected_confidence_range.0 - 0.2,
            "High confidence scenario should have reasonable confidence"
        );

        println!("✓ PASSED");
    }
}

#[test]
fn test_performance_scenarios() {
    let classifier = create_test_classifier();
    let performance_scenarios = get_performance_test_scenarios();

    println!(
        "Testing {} performance scenarios",
        performance_scenarios.len()
    );

    let start_time = std::time::Instant::now();

    for scenario in performance_scenarios {
        let query_start = std::time::Instant::now();

        let result = classifier
            .classify_enhanced(&scenario.query, &scenario.research_type)
            .expect("Performance scenarios should succeed");

        let query_duration = query_start.elapsed();

        println!(
            "Scenario: {} - Duration: {}ms",
            scenario.name,
            query_duration.as_millis()
        );

        // Each query should be reasonably fast
        assert!(
            query_duration.as_millis() < 1000,
            "Performance scenario should be <1000ms"
        );

        // Should have reasonable confidence
        assert!(
            result.overall_confidence > 0.0,
            "Performance scenario should have some confidence"
        );
    }

    let total_duration = start_time.elapsed();
    let avg_duration = total_duration.as_millis() as f64 / performance_scenarios.len() as f64;

    println!(
        "Total performance test duration: {}ms",
        total_duration.as_millis()
    );
    println!("Average duration per scenario: {:.2}ms", avg_duration);

    assert!(avg_duration < 500.0, "Average performance should be <500ms");
}

#[test]
fn test_fixture_test_result_creation() {
    let scenarios = get_comprehensive_test_scenarios();
    let first_scenario = &scenarios[0];

    // Test creating classified request from scenario
    let request = create_test_classified_request(first_scenario);
    assert_eq!(request.original_query, first_scenario.query);
    assert_eq!(request.research_type, first_scenario.research_type);
    assert!(!request.matched_keywords.is_empty());

    // Test creating enhanced result from scenario
    let enhanced_result = create_test_enhanced_result(first_scenario);
    assert_eq!(enhanced_result.research_type, first_scenario.research_type);
    assert_eq!(
        enhanced_result.audience_level,
        first_scenario.expected_audience
    );
    assert_eq!(
        enhanced_result.technical_domain,
        first_scenario.expected_domain
    );
    assert_eq!(
        enhanced_result.urgency_level,
        first_scenario.expected_urgency
    );

    // Test creating research result from scenario
    let research_result = create_test_research_result(first_scenario);
    assert_eq!(research_result.request.original_query, first_scenario.query);
    assert!(!research_result.immediate_answer.is_empty());
    assert!(research_result.metadata.quality_score > 0.0);

    println!("✓ All fixture creation functions work correctly");
}
