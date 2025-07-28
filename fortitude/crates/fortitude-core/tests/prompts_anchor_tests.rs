// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Anchor tests for prompt template system critical functionality
//! These tests verify critical functionality and should be maintained
//! as the system evolves. Do not delete these tests.

use fortitude_core::prompts::*;
use fortitude_types::research::ResearchType;
use std::collections::HashMap;

/// ANCHOR: Verifies template registry works end-to-end with all research types.
/// Tests: Template registration, retrieval, type-specific selection, complexity matching
#[test]
fn test_anchor_template_registry_core_functionality() {
    let registry = DefaultTemplateFactory::create_default_registry();

    // Test core registry functionality
    assert_eq!(registry.get_total_template_count(), 5);
    assert_eq!(registry.get_supported_types().len(), 5);

    // Test all research types are supported
    for research_type in ResearchType::all() {
        assert!(
            registry.has_templates_for_type(&research_type),
            "Missing template for research type: {research_type:?}"
        );

        let templates = registry.get_by_type(&research_type);
        assert!(
            !templates.is_empty(),
            "No templates found for research type: {research_type:?}"
        );

        // Test complexity-based selection
        let best_template = registry.get_best_for_type(&research_type, ComplexityLevel::Basic);
        assert!(
            best_template.is_ok(),
            "Failed to find best template for research type: {research_type:?}"
        );
    }
}

/// ANCHOR: Verifies template rendering works end-to-end with parameter validation.
/// Tests: Parameter validation, template rendering, substitution engine integration
#[test]
fn test_anchor_template_rendering_workflow() {
    let registry = DefaultTemplateFactory::create_default_registry();

    // Test template rendering for each research type
    let test_cases = vec![
        (
            ResearchType::Decision,
            vec![
                ("problem", "Should we migrate to microservices?"),
                ("context", "We have a monolithic Rails app"),
            ],
        ),
        (
            ResearchType::Implementation,
            vec![
                ("feature", "user authentication"),
                ("technology", "Rust with JWT"),
            ],
        ),
        (
            ResearchType::Troubleshooting,
            vec![
                ("problem", "Database connection timeouts"),
                ("symptoms", "Connection pool exhausted"),
            ],
        ),
        (
            ResearchType::Learning,
            vec![("concept", "Rust ownership"), ("level", "beginner")],
        ),
        (
            ResearchType::Validation,
            vec![
                ("approach", "Test-driven development"),
                ("criteria", "Code quality and maintainability"),
            ],
        ),
    ];

    for (research_type, param_data) in test_cases {
        let template = registry
            .get_best_for_type(&research_type, ComplexityLevel::Basic)
            .unwrap_or_else(|_| panic!("Should find template for {research_type:?}"));

        // Test parameter validation - missing required parameters should fail
        let empty_params = HashMap::new();
        let result = template.render(&empty_params);
        assert!(
            result.is_err(),
            "Template rendering should fail with missing parameters for {research_type:?}"
        );

        // Test successful rendering with valid parameters
        let mut params = HashMap::new();
        for (key, value) in param_data {
            params.insert(key.to_string(), ParameterValue::Text(value.to_string()));
        }

        let result = template.render(&params);
        assert!(
            result.is_ok(),
            "Template rendering should succeed with valid parameters for {research_type:?}"
        );

        let rendered = result.unwrap();
        assert!(
            !rendered.is_empty(),
            "Rendered template should not be empty for {research_type:?}"
        );

        // Verify progressive disclosure structure is present
        assert!(
            rendered.contains("<summary priority=\"high\">"),
            "Template should contain high priority summary for {research_type:?}"
        );
        assert!(
            rendered.contains("<evidence priority=\"medium\">"),
            "Template should contain medium priority evidence for {research_type:?}"
        );
        assert!(
            rendered.contains("<implementation priority=\"low\">"),
            "Template should contain low priority implementation for {research_type:?}"
        );
    }
}

/// ANCHOR: Verifies substitution engine works end-to-end with all parameter types.
/// Tests: Parameter type handling, template validation, placeholder substitution
#[test]
fn test_anchor_substitution_engine_parameter_handling() {
    let engine = SubstitutionEngine::new().expect("Should create substitution engine");

    // Test template with all parameter types
    let template = r#"
Text: {{text_param}}
Number: {{number_param}}
Boolean: {{bool_param}}
List: {{list_param}}
Optional: {{optional_param}}
"#;

    // Test with all parameter types
    let mut params = HashMap::new();
    params.insert(
        "text_param".to_string(),
        ParameterValue::Text("Hello World".to_string()),
    );
    params.insert("number_param".to_string(), ParameterValue::Number(42.5));
    params.insert("bool_param".to_string(), ParameterValue::Boolean(true));
    params.insert(
        "list_param".to_string(),
        ParameterValue::List(vec![
            ParameterValue::Text("item1".to_string()),
            ParameterValue::Text("item2".to_string()),
            ParameterValue::Text("item3".to_string()),
        ]),
    );
    params.insert("optional_param".to_string(), ParameterValue::None);

    let result = engine.substitute(template, &params);
    assert!(
        result.is_ok(),
        "Substitution should succeed with all parameter types"
    );

    let rendered = result.unwrap();
    assert!(
        rendered.contains("Hello World"),
        "Should contain text parameter"
    );
    assert!(rendered.contains("42.5"), "Should contain number parameter");
    assert!(
        rendered.contains("true"),
        "Should contain boolean parameter"
    );
    assert!(
        rendered.contains("item1, item2, item3"),
        "Should contain list parameter"
    );
    assert!(
        rendered.contains("Optional: "),
        "Should handle None parameter"
    );

    // Test template validation
    let invalid_template = "Hello {{unclosed_param, world!";
    let validation_result = engine.validate_template(invalid_template);
    assert!(
        validation_result.is_err(),
        "Should reject invalid template syntax"
    );

    let empty_param_template = "Hello {{}}, world!";
    let validation_result = engine.validate_template(empty_param_template);
    assert!(
        validation_result.is_err(),
        "Should reject empty parameter names"
    );
}

/// ANCHOR: Verifies quality validation works end-to-end with all research types.
/// Tests: Completion criteria checking, quality scoring, validation reporting
#[test]
fn test_anchor_quality_validation_workflow() {
    let validator = QualityValidator::new();

    // Test that validator has criteria for all research types
    for research_type in ResearchType::all() {
        let criteria = validator.get_criteria(&research_type);
        assert!(
            criteria.is_some(),
            "Should have completion criteria for research type: {research_type:?}"
        );

        let criteria = criteria.unwrap();
        assert!(
            !criteria.required_sections.is_empty(),
            "Should have required sections for research type: {research_type:?}"
        );
        assert!(
            !criteria.required_tags.is_empty(),
            "Should have required tags for research type: {research_type:?}"
        );
        assert!(
            !criteria.required_layers.is_empty(),
            "Should have required layers for research type: {research_type:?}"
        );
        assert!(
            criteria.quality_threshold > 0.0,
            "Should have positive quality threshold for research type: {research_type:?}"
        );
    }

    // Test quality scoring
    let mut report = ValidationReport::new(ResearchType::Decision);
    report.progressive_disclosure_score = 0.8;
    report.semantic_markup_score = 0.6;
    report.content_quality_score = 0.9;

    report.calculate_overall_score();

    // Expected: 0.8 * 0.3 + 0.6 * 0.2 + 0.9 * 0.5 = 0.24 + 0.12 + 0.45 = 0.81
    assert!(
        (report.overall_score - 0.81).abs() < 0.01,
        "Overall score calculation should be correct"
    );

    // Test validation reporting
    assert_eq!(report.research_type, ResearchType::Decision);
    assert!(
        report.issues.is_empty(),
        "Report should start with no issues"
    );

    report.add_issue("Test issue".to_string());
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0], "Test issue");
}

/// ANCHOR: Verifies progressive disclosure works end-to-end with complexity levels.
/// Tests: Complexity-based parameter filtering, level advancement, parameter availability
#[test]
fn test_anchor_progressive_disclosure_complexity_management() {
    let registry = DefaultTemplateFactory::create_default_registry();

    // Test progressive disclosure with different complexity levels
    for research_type in ResearchType::all() {
        let template = registry
            .get_best_for_type(&research_type, ComplexityLevel::Basic)
            .unwrap_or_else(|_| panic!("Should find template for {research_type:?}"));

        let parameters = template.get_parameters();
        if parameters.is_empty() {
            continue; // Skip if no parameters
        }

        let mut disclosure = ProgressiveDisclosure::new(parameters);

        // Test complexity level progression
        let mut previous_count = 0;
        let mut current_level = ComplexityLevel::Basic;

        loop {
            let available_params = disclosure.get_available_parameters();
            assert!(
                available_params.len() >= previous_count,
                "Parameter count should not decrease as complexity increases for {research_type:?}"
            );

            // Verify all parameters are at or below current complexity level
            for param in &available_params {
                assert!(
                    param.complexity_level <= current_level,
                    "Parameter complexity should not exceed current level for {research_type:?}"
                );
            }

            previous_count = available_params.len();

            if !disclosure.advance_level() {
                break;
            }

            current_level = disclosure.current_level().clone();
        }

        // Should end at Expert level
        assert_eq!(
            *disclosure.current_level(),
            ComplexityLevel::Expert,
            "Should advance to Expert level for {research_type:?}"
        );
    }
}

/// ANCHOR: Verifies template parameter validation works end-to-end with type checking.
/// Tests: Parameter type validation, required parameter checking, validation error reporting
#[test]
fn test_anchor_parameter_validation_type_checking() {
    // Create a template with various parameter types
    let param_definitions = vec![
        ParameterBuilder::new()
            .name("required_text")
            .param_type(ParameterType::Text)
            .description("Required text parameter")
            .required()
            .build()
            .unwrap(),
        ParameterBuilder::new()
            .name("optional_number")
            .param_type(ParameterType::Optional(Box::new(ParameterType::Number)))
            .description("Optional number parameter")
            .build()
            .unwrap(),
        ParameterBuilder::new()
            .name("required_list")
            .param_type(ParameterType::List(Box::new(ParameterType::Text)))
            .description("Required list parameter")
            .required()
            .build()
            .unwrap(),
        ParameterBuilder::new()
            .name("boolean_param")
            .param_type(ParameterType::Boolean)
            .description("Boolean parameter")
            .build()
            .unwrap(),
    ];

    let validator = ParameterValidator::new(&param_definitions);

    // Test successful validation
    let mut valid_params = HashMap::new();
    valid_params.insert(
        "required_text".to_string(),
        ParameterValue::Text("test".to_string()),
    );
    valid_params.insert("optional_number".to_string(), ParameterValue::Number(42.0));
    valid_params.insert(
        "required_list".to_string(),
        ParameterValue::List(vec![
            ParameterValue::Text("item1".to_string()),
            ParameterValue::Text("item2".to_string()),
        ]),
    );
    valid_params.insert("boolean_param".to_string(), ParameterValue::Boolean(false));

    let result = validator.validate(&valid_params);
    assert!(
        result.is_ok(),
        "Validation should succeed with valid parameters"
    );

    // Test missing required parameter
    let mut missing_params = HashMap::new();
    missing_params.insert("optional_number".to_string(), ParameterValue::Number(42.0));

    let result = validator.validate(&missing_params);
    assert!(
        result.is_err(),
        "Validation should fail with missing required parameters"
    );

    // Test type mismatch
    let mut type_mismatch_params = HashMap::new();
    type_mismatch_params.insert("required_text".to_string(), ParameterValue::Number(42.0));
    type_mismatch_params.insert(
        "required_list".to_string(),
        ParameterValue::List(vec![ParameterValue::Text("item1".to_string())]),
    );

    let result = validator.validate(&type_mismatch_params);
    assert!(
        result.is_err(),
        "Validation should fail with type mismatches"
    );

    // Test optional parameter handling
    let mut optional_params = HashMap::new();
    optional_params.insert(
        "required_text".to_string(),
        ParameterValue::Text("test".to_string()),
    );
    optional_params.insert(
        "required_list".to_string(),
        ParameterValue::List(vec![ParameterValue::Text("item1".to_string())]),
    );
    optional_params.insert("optional_number".to_string(), ParameterValue::None);

    let result = validator.validate(&optional_params);
    assert!(
        result.is_ok(),
        "Validation should succeed with None optional parameters"
    );
}
