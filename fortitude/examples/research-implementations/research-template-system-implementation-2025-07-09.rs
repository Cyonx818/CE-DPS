use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Research taxonomy types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResearchType {
    Decision,
    Implementation,
    Troubleshooting,
    Learning,
    Validation,
}

// Template complexity levels for progressive disclosure
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum ComplexityLevel {
    Basic = 1,
    Intermediate = 2,
    Advanced = 3,
    Expert = 4,
}

// Parameter types for template validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Text,
    Number,
    Boolean,
    List(Box<ParameterType>),
    Optional(Box<ParameterType>),
}

// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub default_value: Option<ParameterValue>,
    pub required: bool,
    pub complexity_level: ComplexityLevel,
}

// Runtime parameter values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    List(Vec<ParameterValue>),
    None,
}

// Template validation errors
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    #[error("Type mismatch for parameter {name}: expected {expected:?}, got {actual:?}")]
    TypeMismatch {
        name: String,
        expected: ParameterType,
        actual: ParameterType,
    },
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Substitution error: {0}")]
    SubstitutionError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

// Core template trait for different research types
pub trait ResearchTemplate: Send + Sync {
    fn get_type(&self) -> ResearchType;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_parameters(&self) -> &[ParameterDefinition];
    fn get_template_content(&self) -> &str;
    fn get_complexity_level(&self) -> ComplexityLevel;
    fn validate_parameters(&self, params: &HashMap<String, ParameterValue>) -> Result<(), TemplateError>;
    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError>;
}

// Template implementation for specific research types
#[derive(Debug, Clone)]
pub struct Template<T> {
    name: String,
    description: String,
    parameters: Vec<ParameterDefinition>,
    content: String,
    complexity_level: ComplexityLevel,
    _phantom: PhantomData<T>,
}

// Type markers for different research types
pub struct DecisionMarker;
pub struct ImplementationMarker;
pub struct TroubleshootingMarker;
pub struct LearningMarker;
pub struct ValidationMarker;

// Template type aliases
pub type DecisionTemplate = Template<DecisionMarker>;
pub type ImplementationTemplate = Template<ImplementationMarker>;
pub type TroubleshootingTemplate = Template<TroubleshootingMarker>;
pub type LearningTemplate = Template<LearningMarker>;
pub type ValidationTemplate = Template<ValidationMarker>;

impl<T> Template<T> {
    pub fn new(
        name: String,
        description: String,
        parameters: Vec<ParameterDefinition>,
        content: String,
        complexity_level: ComplexityLevel,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            content,
            complexity_level,
            _phantom: PhantomData,
        }
    }
}

// Research template trait implementations
macro_rules! impl_research_template {
    ($marker:ty, $research_type:expr) => {
        impl ResearchTemplate for Template<$marker> {
            fn get_type(&self) -> ResearchType {
                $research_type
            }

            fn get_name(&self) -> &str {
                &self.name
            }

            fn get_description(&self) -> &str {
                &self.description
            }

            fn get_parameters(&self) -> &[ParameterDefinition] {
                &self.parameters
            }

            fn get_template_content(&self) -> &str {
                &self.content
            }

            fn get_complexity_level(&self) -> ComplexityLevel {
                self.complexity_level.clone()
            }

            fn validate_parameters(&self, params: &HashMap<String, ParameterValue>) -> Result<(), TemplateError> {
                let validator = ParameterValidator::new(&self.parameters);
                validator.validate(params)
            }

            fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
                // Validate first
                self.validate_parameters(params)?;
                
                // Render with substitution engine
                let engine = SubstitutionEngine::new();
                engine.substitute(&self.content, params)
            }
        }
    };
}

impl_research_template!(DecisionMarker, ResearchType::Decision);
impl_research_template!(ImplementationMarker, ResearchType::Implementation);
impl_research_template!(TroubleshootingMarker, ResearchType::Troubleshooting);
impl_research_template!(LearningMarker, ResearchType::Learning);
impl_research_template!(ValidationMarker, ResearchType::Validation);

// Parameter validation system
pub struct ParameterValidator<'a> {
    definitions: &'a [ParameterDefinition],
}

impl<'a> ParameterValidator<'a> {
    pub fn new(definitions: &'a [ParameterDefinition]) -> Self {
        Self { definitions }
    }

    pub fn validate(&self, params: &HashMap<String, ParameterValue>) -> Result<(), TemplateError> {
        // Check required parameters
        for def in self.definitions {
            if def.required && !params.contains_key(&def.name) {
                return Err(TemplateError::MissingParameter(def.name.clone()));
            }

            if let Some(value) = params.get(&def.name) {
                self.validate_type(&def.name, &def.param_type, value)?;
            }
        }

        Ok(())
    }

    fn validate_type(&self, name: &str, expected: &ParameterType, actual: &ParameterValue) -> Result<(), TemplateError> {
        match (expected, actual) {
            (ParameterType::Text, ParameterValue::Text(_)) => Ok(()),
            (ParameterType::Number, ParameterValue::Number(_)) => Ok(()),
            (ParameterType::Boolean, ParameterValue::Boolean(_)) => Ok(()),
            (ParameterType::List(inner_type), ParameterValue::List(values)) => {
                for value in values {
                    self.validate_type(name, inner_type, value)?;
                }
                Ok(())
            }
            (ParameterType::Optional(inner_type), ParameterValue::None) => Ok(()),
            (ParameterType::Optional(inner_type), value) => {
                self.validate_type(name, inner_type, value)
            }
            _ => Err(TemplateError::TypeMismatch {
                name: name.to_string(),
                expected: expected.clone(),
                actual: self.infer_type(actual),
            }),
        }
    }

    fn infer_type(&self, value: &ParameterValue) -> ParameterType {
        match value {
            ParameterValue::Text(_) => ParameterType::Text,
            ParameterValue::Number(_) => ParameterType::Number,
            ParameterValue::Boolean(_) => ParameterType::Boolean,
            ParameterValue::List(values) => {
                if let Some(first) = values.first() {
                    ParameterType::List(Box::new(self.infer_type(first)))
                } else {
                    ParameterType::List(Box::new(ParameterType::Text))
                }
            }
            ParameterValue::None => ParameterType::Optional(Box::new(ParameterType::Text)),
        }
    }
}

// Template substitution engine
pub struct SubstitutionEngine;

impl SubstitutionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn substitute(&self, template: &str, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
        let mut result = template.to_string();
        
        // Simple placeholder substitution: {{parameter_name}}
        for (name, value) in params {
            let placeholder = format!("{{{{{}}}}}", name);
            let replacement = self.format_value(value);
            result = result.replace(&placeholder, &replacement);
        }

        // Check for unsubstituted placeholders
        if result.contains("{{") && result.contains("}}") {
            return Err(TemplateError::SubstitutionError(
                "Template contains unsubstituted placeholders".to_string()
            ));
        }

        Ok(result)
    }

    fn format_value(&self, value: &ParameterValue) -> String {
        match value {
            ParameterValue::Text(s) => s.clone(),
            ParameterValue::Number(n) => n.to_string(),
            ParameterValue::Boolean(b) => b.to_string(),
            ParameterValue::List(values) => {
                values.iter()
                    .map(|v| self.format_value(v))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            ParameterValue::None => String::new(),
        }
    }
}

// Progressive disclosure manager
pub struct ProgressiveDisclosure<'a> {
    template: &'a dyn ResearchTemplate,
    current_level: ComplexityLevel,
}

impl<'a> ProgressiveDisclosure<'a> {
    pub fn new(template: &'a dyn ResearchTemplate) -> Self {
        Self {
            template,
            current_level: ComplexityLevel::Basic,
        }
    }

    pub fn set_complexity_level(&mut self, level: ComplexityLevel) {
        self.current_level = level;
    }

    pub fn get_available_parameters(&self) -> Vec<&ParameterDefinition> {
        self.template
            .get_parameters()
            .iter()
            .filter(|param| param.complexity_level <= self.current_level)
            .collect()
    }

    pub fn get_next_level_parameters(&self) -> Vec<&ParameterDefinition> {
        let next_level = match self.current_level {
            ComplexityLevel::Basic => ComplexityLevel::Intermediate,
            ComplexityLevel::Intermediate => ComplexityLevel::Advanced,
            ComplexityLevel::Advanced => ComplexityLevel::Expert,
            ComplexityLevel::Expert => return vec![],
        };

        self.template
            .get_parameters()
            .iter()
            .filter(|param| param.complexity_level == next_level)
            .collect()
    }

    pub fn can_advance(&self) -> bool {
        !self.get_next_level_parameters().is_empty()
    }

    pub fn advance_level(&mut self) -> bool {
        if !self.can_advance() {
            return false;
        }

        self.current_level = match self.current_level {
            ComplexityLevel::Basic => ComplexityLevel::Intermediate,
            ComplexityLevel::Intermediate => ComplexityLevel::Advanced,
            ComplexityLevel::Advanced => ComplexityLevel::Expert,
            ComplexityLevel::Expert => return false,
        };

        true
    }
}

// Template registry for storage and management
pub struct TemplateRegistry {
    templates: HashMap<String, Box<dyn ResearchTemplate>>,
    type_index: HashMap<ResearchType, Vec<String>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    pub fn register<T: ResearchTemplate + 'static>(&mut self, template: T) {
        let name = template.get_name().to_string();
        let research_type = template.get_type();
        
        // Add to type index
        self.type_index
            .entry(research_type)
            .or_insert_with(Vec::new)
            .push(name.clone());
        
        // Store template
        self.templates.insert(name, Box::new(template));
    }

    pub fn get(&self, name: &str) -> Option<&dyn ResearchTemplate> {
        self.templates.get(name).map(|t| t.as_ref())
    }

    pub fn get_by_type(&self, research_type: &ResearchType) -> Vec<&dyn ResearchTemplate> {
        self.type_index
            .get(research_type)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn list_templates(&self) -> Vec<(&str, ResearchType, ComplexityLevel)> {
        self.templates
            .values()
            .map(|t| (t.get_name(), t.get_type(), t.get_complexity_level()))
            .collect()
    }
}

// Builder for creating templates with validation
pub struct TemplateBuilder<T> {
    name: Option<String>,
    description: Option<String>,
    parameters: Vec<ParameterDefinition>,
    content: Option<String>,
    complexity_level: ComplexityLevel,
    _phantom: PhantomData<T>,
}

impl<T> TemplateBuilder<T> {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            parameters: Vec::new(),
            content: None,
            complexity_level: ComplexityLevel::Basic,
            _phantom: PhantomData,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn complexity_level(mut self, level: ComplexityLevel) -> Self {
        self.complexity_level = level;
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn parameter(mut self, param: ParameterDefinition) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn build(self) -> Result<Template<T>, TemplateError> {
        let name = self.name.ok_or_else(|| TemplateError::ValidationError("Name is required".to_string()))?;
        let description = self.description.ok_or_else(|| TemplateError::ValidationError("Description is required".to_string()))?;
        let content = self.content.ok_or_else(|| TemplateError::ValidationError("Content is required".to_string()))?;

        Ok(Template::new(name, description, self.parameters, content, self.complexity_level))
    }
}

// Parameter definition builder
pub struct ParameterBuilder {
    name: Option<String>,
    param_type: Option<ParameterType>,
    description: Option<String>,
    default_value: Option<ParameterValue>,
    required: bool,
    complexity_level: ComplexityLevel,
}

impl ParameterBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            param_type: None,
            description: None,
            default_value: None,
            required: false,
            complexity_level: ComplexityLevel::Basic,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn param_type(mut self, param_type: ParameterType) -> Self {
        self.param_type = Some(param_type);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn default_value(mut self, value: ParameterValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn complexity_level(mut self, level: ComplexityLevel) -> Self {
        self.complexity_level = level;
        self
    }

    pub fn build(self) -> Result<ParameterDefinition, TemplateError> {
        let name = self.name.ok_or_else(|| TemplateError::ValidationError("Parameter name is required".to_string()))?;
        let param_type = self.param_type.ok_or_else(|| TemplateError::ValidationError("Parameter type is required".to_string()))?;
        let description = self.description.ok_or_else(|| TemplateError::ValidationError("Parameter description is required".to_string()))?;

        Ok(ParameterDefinition {
            name,
            param_type,
            description,
            default_value: self.default_value,
            required: self.required,
            complexity_level: self.complexity_level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_template_creation() {
        let template = TemplateBuilder::<DecisionMarker>::new()
            .name("Basic Decision Analysis")
            .description("Template for simple decision analysis prompts")
            .content("Analyze the decision: {{problem}}. Consider factors: {{factors}}.")
            .parameter(
                ParameterBuilder::new()
                    .name("problem")
                    .param_type(ParameterType::Text)
                    .description("The decision problem to analyze")
                    .required()
                    .build()
                    .unwrap()
            )
            .parameter(
                ParameterBuilder::new()
                    .name("factors")
                    .param_type(ParameterType::List(Box::new(ParameterType::Text)))
                    .description("List of factors to consider")
                    .complexity_level(ComplexityLevel::Intermediate)
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        assert_eq!(template.get_type(), ResearchType::Decision);
        assert_eq!(template.get_name(), "Basic Decision Analysis");
    }

    #[test]
    fn test_parameter_validation() {
        let mut params = HashMap::new();
        params.insert("problem".to_string(), ParameterValue::Text("Should we adopt AI?".to_string()));
        params.insert("factors".to_string(), ParameterValue::List(vec![
            ParameterValue::Text("Cost".to_string()),
            ParameterValue::Text("Benefits".to_string()),
        ]));

        let template = TemplateBuilder::<DecisionMarker>::new()
            .name("Test Template")
            .description("Test")
            .content("{{problem}} {{factors}}")
            .parameter(
                ParameterBuilder::new()
                    .name("problem")
                    .param_type(ParameterType::Text)
                    .description("Problem")
                    .required()
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        assert!(template.validate_parameters(&params).is_ok());
    }

    #[test]
    fn test_template_rendering() {
        let mut params = HashMap::new();
        params.insert("problem".to_string(), ParameterValue::Text("Should we adopt AI?".to_string()));

        let template = TemplateBuilder::<DecisionMarker>::new()
            .name("Test Template")
            .description("Test")
            .content("Analyze: {{problem}}")
            .parameter(
                ParameterBuilder::new()
                    .name("problem")
                    .param_type(ParameterType::Text)
                    .description("Problem")
                    .required()
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        let result = template.render(&params).unwrap();
        assert_eq!(result, "Analyze: Should we adopt AI?");
    }

    #[test]
    fn test_progressive_disclosure() {
        let template = TemplateBuilder::<LearningMarker>::new()
            .name("Learning Template")
            .description("Test progressive disclosure")
            .content("{{basic}} {{advanced}}")
            .parameter(
                ParameterBuilder::new()
                    .name("basic")
                    .param_type(ParameterType::Text)
                    .description("Basic parameter")
                    .complexity_level(ComplexityLevel::Basic)
                    .build()
                    .unwrap()
            )
            .parameter(
                ParameterBuilder::new()
                    .name("advanced")
                    .param_type(ParameterType::Text)
                    .description("Advanced parameter")
                    .complexity_level(ComplexityLevel::Advanced)
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        let mut disclosure = ProgressiveDisclosure::new(&template);
        
        // Initially only basic parameters should be available
        assert_eq!(disclosure.get_available_parameters().len(), 1);
        assert_eq!(disclosure.get_available_parameters()[0].name, "basic");
        
        // Advance to intermediate level
        disclosure.set_complexity_level(ComplexityLevel::Advanced);
        assert_eq!(disclosure.get_available_parameters().len(), 2);
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();
        
        let decision_template = TemplateBuilder::<DecisionMarker>::new()
            .name("Decision Template")
            .description("Decision analysis")
            .content("Decision: {{problem}}")
            .build()
            .unwrap();

        registry.register(decision_template);
        
        assert!(registry.get("Decision Template").is_some());
        assert_eq!(registry.get_by_type(&ResearchType::Decision).len(), 1);
    }
}

// Example usage and factory functions
impl TemplateRegistry {
    /// Create a registry with default research templates
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.add_default_templates();
        registry
    }

    fn add_default_templates(&mut self) {
        // Decision templates
        let basic_decision = TemplateBuilder::<DecisionMarker>::new()
            .name("Basic Decision Analysis")
            .description("Simple decision analysis template")
            .content(r#"
# Decision Analysis: {{problem}}

## Problem Statement
{{problem}}

## Options to Consider
{{#if options}}
{{#each options}}
- {{this}}
{{/each}}
{{else}}
Please list the available options for this decision.
{{/if}}

## Decision Criteria
{{criteria}}

## Recommendation
Based on the analysis above, I recommend:
{{recommendation}}
"#)
            .parameter(
                ParameterBuilder::new()
                    .name("problem")
                    .param_type(ParameterType::Text)
                    .description("The decision problem to analyze")
                    .required()
                    .build()
                    .unwrap()
            )
            .parameter(
                ParameterBuilder::new()
                    .name("options")
                    .param_type(ParameterType::Optional(Box::new(ParameterType::List(Box::new(ParameterType::Text)))))
                    .description("Available options to choose from")
                    .complexity_level(ComplexityLevel::Intermediate)
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        // Implementation templates
        let implementation_template = TemplateBuilder::<ImplementationMarker>::new()
            .name("Feature Implementation Plan")
            .description("Template for planning feature implementation")
            .content(r#"
# Implementation Plan: {{feature_name}}

## Overview
Implementing: {{feature_name}}
Estimated Timeline: {{timeline}}

## Technical Requirements
{{requirements}}

## Implementation Steps
1. {{step1}}
2. {{step2}}
3. {{step3}}

## Testing Strategy
{{testing_strategy}}

## Deployment Plan
{{deployment_plan}}
"#)
            .parameter(
                ParameterBuilder::new()
                    .name("feature_name")
                    .param_type(ParameterType::Text)
                    .description("Name of the feature to implement")
                    .required()
                    .build()
                    .unwrap()
            )
            .parameter(
                ParameterBuilder::new()
                    .name("timeline")
                    .param_type(ParameterType::Text)
                    .description("Estimated implementation timeline")
                    .complexity_level(ComplexityLevel::Intermediate)
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        self.register(basic_decision);
        self.register(implementation_template);
    }
}