// ABOUTME: Integration tests for the prompt template system
//! These tests verify the prompt template system works end-to-end
//! without mocking internal components.

use fortitude_core::prompts::*;
use fortitude_types::research::ResearchType;
use std::collections::HashMap;

#[test]
fn test_template_registry_integration() {
    let registry = DefaultTemplateFactory::create_default_registry();

    // Verify all research types are supported
    assert!(registry.has_templates_for_type(&ResearchType::Decision));
    assert!(registry.has_templates_for_type(&ResearchType::Implementation));
    assert!(registry.has_templates_for_type(&ResearchType::Troubleshooting));
    assert!(registry.has_templates_for_type(&ResearchType::Learning));
    assert!(registry.has_templates_for_type(&ResearchType::Validation));

    // Verify template counts
    assert_eq!(registry.get_total_template_count(), 5);
    assert_eq!(
        registry.get_template_count_by_type(&ResearchType::Decision),
        1
    );
    assert_eq!(
        registry.get_template_count_by_type(&ResearchType::Implementation),
        1
    );
    assert_eq!(
        registry.get_template_count_by_type(&ResearchType::Troubleshooting),
        1
    );
    assert_eq!(
        registry.get_template_count_by_type(&ResearchType::Learning),
        1
    );
    assert_eq!(
        registry.get_template_count_by_type(&ResearchType::Validation),
        1
    );
}

#[test]
fn test_decision_template_end_to_end() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Decision, ComplexityLevel::Basic)
        .expect("Should find decision template");

    // Test with minimal parameters
    let mut params = HashMap::new();
    params.insert(
        "problem".to_string(),
        ParameterValue::Text("Should we adopt microservices?".to_string()),
    );
    params.insert(
        "context".to_string(),
        ParameterValue::Text(
            "We're a growing startup with a monolithic Ruby on Rails app".to_string(),
        ),
    );

    let result = template.render(&params);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert!(rendered.contains("Should we adopt microservices?"));
    assert!(rendered.contains("We're a growing startup with a monolithic Ruby on Rails app"));
    assert!(rendered.contains("Decision Analysis"));
    assert!(rendered.contains("Problem Statement"));
    assert!(rendered.contains("Recommendation"));
    assert!(rendered.contains("<summary priority=\"high\">"));
    assert!(rendered.contains("<evidence priority=\"medium\">"));
    assert!(rendered.contains("<implementation priority=\"low\">"));
}

#[test]
fn test_implementation_template_end_to_end() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Implementation, ComplexityLevel::Basic)
        .expect("Should find implementation template");

    let mut params = HashMap::new();
    params.insert(
        "feature".to_string(),
        ParameterValue::Text("user authentication system".to_string()),
    );
    params.insert(
        "technology".to_string(),
        ParameterValue::Text("Rust with Actix-web and JWT".to_string()),
    );

    let result = template.render(&params);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert!(rendered.contains("user authentication system"));
    assert!(rendered.contains("Rust with Actix-web and JWT"));
    assert!(rendered.contains("Implementation Guide"));
    assert!(rendered.contains("Overview"));
    assert!(rendered.contains("Step-by-Step Implementation"));
    assert!(rendered.contains("Testing Strategy"));
    assert!(rendered.contains("<summary priority=\"high\">"));
    assert!(rendered.contains("<evidence priority=\"medium\">"));
    assert!(rendered.contains("<implementation priority=\"low\">"));
}

#[test]
fn test_troubleshooting_template_end_to_end() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Troubleshooting, ComplexityLevel::Basic)
        .expect("Should find troubleshooting template");

    let mut params = HashMap::new();
    params.insert(
        "problem".to_string(),
        ParameterValue::Text("Database connection timeout errors".to_string()),
    );
    params.insert(
        "symptoms".to_string(),
        ParameterValue::Text("Error: connection pool exhausted after 30 seconds".to_string()),
    );

    let result = template.render(&params);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert!(rendered.contains("Database connection timeout errors"));
    assert!(rendered.contains("connection pool exhausted after 30 seconds"));
    assert!(rendered.contains("Troubleshooting Guide"));
    assert!(rendered.contains("Problem Description"));
    assert!(rendered.contains("Observed Symptoms"));
    assert!(rendered.contains("Diagnostic Steps"));
    assert!(rendered.contains("Solution Implementation"));
    assert!(rendered.contains("<summary priority=\"high\">"));
    assert!(rendered.contains("<evidence priority=\"medium\">"));
    assert!(rendered.contains("<implementation priority=\"low\">"));
}

#[test]
fn test_learning_template_end_to_end() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Learning, ComplexityLevel::Basic)
        .expect("Should find learning template");

    let mut params = HashMap::new();
    params.insert(
        "concept".to_string(),
        ParameterValue::Text("Rust ownership system".to_string()),
    );
    params.insert(
        "level".to_string(),
        ParameterValue::Text("beginner".to_string()),
    );

    let result = template.render(&params);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert!(rendered.contains("Rust ownership system"));
    assert!(rendered.contains("beginner"));
    assert!(rendered.contains("Learning Guide"));
    assert!(rendered.contains("Concept Overview"));
    assert!(rendered.contains("Core Concepts"));
    assert!(rendered.contains("Hands-On Learning"));
    assert!(rendered.contains("<summary priority=\"high\">"));
    assert!(rendered.contains("<evidence priority=\"medium\">"));
    assert!(rendered.contains("<implementation priority=\"low\">"));
}

#[test]
fn test_validation_template_end_to_end() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Validation, ComplexityLevel::Basic)
        .expect("Should find validation template");

    let mut params = HashMap::new();
    params.insert(
        "approach".to_string(),
        ParameterValue::Text("Test-driven development approach".to_string()),
    );
    params.insert(
        "criteria".to_string(),
        ParameterValue::Text("Code quality, maintainability, and development speed".to_string()),
    );

    let result = template.render(&params);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert!(rendered.contains("Test-driven development approach"));
    assert!(rendered.contains("Code quality, maintainability, and development speed"));
    assert!(rendered.contains("Validation Analysis"));
    assert!(rendered.contains("Approach Overview"));
    assert!(rendered.contains("Detailed Analysis"));
    assert!(rendered.contains("Validation Results"));
    assert!(rendered.contains("<summary priority=\"high\">"));
    assert!(rendered.contains("<evidence priority=\"medium\">"));
    assert!(rendered.contains("<implementation priority=\"low\">"));
}

#[test]
fn test_template_parameter_validation_integration() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Decision, ComplexityLevel::Basic)
        .expect("Should find decision template");

    // Test missing required parameter
    let empty_params = HashMap::new();
    let result = template.render(&empty_params);
    assert!(result.is_err());

    // Test with valid parameters
    let mut valid_params = HashMap::new();
    valid_params.insert(
        "problem".to_string(),
        ParameterValue::Text("Test problem".to_string()),
    );
    valid_params.insert(
        "context".to_string(),
        ParameterValue::Text("Test context".to_string()),
    );

    let result = template.render(&valid_params);
    assert!(result.is_ok());
}

#[test]
fn test_progressive_disclosure_integration() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let template = registry
        .get_best_for_type(&ResearchType::Implementation, ComplexityLevel::Basic)
        .expect("Should find implementation template");

    // Verify template has progressive disclosure parameters
    let parameters = template.get_parameters();
    assert!(!parameters.is_empty());

    // Test progressive disclosure with parameters
    let mut disclosure = ProgressiveDisclosure::new(parameters);

    // Basic level should have some parameters
    let basic_params_len = disclosure.get_available_parameters().len();
    assert!(basic_params_len > 0);

    // Should be able to advance level
    let can_advance = disclosure.advance_level();
    assert!(can_advance);

    let intermediate_params_len = disclosure.get_available_parameters().len();
    assert!(intermediate_params_len >= basic_params_len);
}

#[test]
fn test_template_complexity_selection() {
    let registry = DefaultTemplateFactory::create_default_registry();

    // Test basic complexity selection
    let basic_template =
        registry.get_best_for_type(&ResearchType::Decision, ComplexityLevel::Basic);
    assert!(basic_template.is_ok());

    // Test intermediate complexity selection
    let intermediate_template =
        registry.get_best_for_type(&ResearchType::Decision, ComplexityLevel::Intermediate);
    assert!(intermediate_template.is_ok());

    // Test advanced complexity selection
    let advanced_template =
        registry.get_best_for_type(&ResearchType::Decision, ComplexityLevel::Advanced);
    assert!(advanced_template.is_ok());

    // Test expert complexity selection
    let expert_template =
        registry.get_best_for_type(&ResearchType::Decision, ComplexityLevel::Expert);
    assert!(expert_template.is_ok());
}

#[test]
fn test_quality_validation_integration() {
    let _registry = DefaultTemplateFactory::create_default_registry();
    let validator = QualityValidator::new();

    // Test that validator has criteria for all template types
    assert!(validator.get_criteria(&ResearchType::Decision).is_some());
    assert!(validator
        .get_criteria(&ResearchType::Implementation)
        .is_some());
    assert!(validator
        .get_criteria(&ResearchType::Troubleshooting)
        .is_some());
    assert!(validator.get_criteria(&ResearchType::Learning).is_some());
    assert!(validator.get_criteria(&ResearchType::Validation).is_some());

    // Test criteria consistency
    let decision_criteria = validator.get_criteria(&ResearchType::Decision).unwrap();
    assert!(!decision_criteria.required_sections.is_empty());
    assert!(!decision_criteria.required_tags.is_empty());
    assert!(!decision_criteria.required_layers.is_empty());
    assert!(decision_criteria.quality_threshold > 0.0);
}

#[test]
fn test_substitution_engine_integration() {
    let engine = SubstitutionEngine::new().expect("Should create substitution engine");

    // Test complex template with multiple parameter types
    let template = r#"
# Analysis: {{title}}

## Problem: {{problem}}

## Status: {{is_urgent}}

## Count: {{item_count}}

## Items:
{{items}}

## Optional note: {{note}}
"#;

    let mut params = HashMap::new();
    params.insert(
        "title".to_string(),
        ParameterValue::Text("System Analysis".to_string()),
    );
    params.insert(
        "problem".to_string(),
        ParameterValue::Text("Performance degradation".to_string()),
    );
    params.insert("is_urgent".to_string(), ParameterValue::Boolean(true));
    params.insert("item_count".to_string(), ParameterValue::Number(5.0));
    params.insert(
        "items".to_string(),
        ParameterValue::List(vec![
            ParameterValue::Text("CPU usage".to_string()),
            ParameterValue::Text("Memory leak".to_string()),
            ParameterValue::Text("Database queries".to_string()),
        ]),
    );
    params.insert("note".to_string(), ParameterValue::None);

    let result = engine.substitute(template, &params);
    assert!(result.is_ok());

    let rendered = result.unwrap();
    assert!(rendered.contains("System Analysis"));
    assert!(rendered.contains("Performance degradation"));
    assert!(rendered.contains("true"));
    assert!(rendered.contains("5"));
    assert!(rendered.contains("CPU usage, Memory leak, Database queries"));
    assert!(rendered.contains("Optional note: "));
}

#[test]
fn test_template_registry_statistics() {
    let registry = DefaultTemplateFactory::create_default_registry();
    let stats = registry.get_stats();

    // Verify statistics are correct
    assert_eq!(stats.total_templates, 5);
    assert_eq!(stats.templates_by_type.len(), 5);
    assert_eq!(stats.templates_by_complexity.len(), 1); // All basic templates

    // Check specific counts
    assert_eq!(stats.templates_by_type[&ResearchType::Decision], 1);
    assert_eq!(stats.templates_by_type[&ResearchType::Implementation], 1);
    assert_eq!(stats.templates_by_type[&ResearchType::Troubleshooting], 1);
    assert_eq!(stats.templates_by_type[&ResearchType::Learning], 1);
    assert_eq!(stats.templates_by_type[&ResearchType::Validation], 1);

    assert_eq!(stats.templates_by_complexity[&ComplexityLevel::Basic], 5);
}
