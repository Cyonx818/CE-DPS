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

// ABOUTME: Template system for research prompt generation
use crate::prompts::parameters::{
    ComplexityLevel, ParameterDefinition, ParameterError, ParameterValidator, ParameterValue,
};
use crate::prompts::substitution::{SubstitutionEngine, SubstitutionError};
use fortitude_types::research::ResearchType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use thiserror::Error;

/// Errors that can occur during template operations
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Parameter error: {0}")]
    ParameterError(#[from] ParameterError),
    #[error("Substitution error: {0}")]
    SubstitutionError(#[from] SubstitutionError),
    #[error("Template validation failed: {0}")]
    ValidationError(String),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template format: {0}")]
    InvalidFormat(String),
}

/// Trait for research templates
pub trait ResearchTemplate: Send + Sync {
    /// Get the research type this template supports
    fn get_type(&self) -> ResearchType;

    /// Get the template name
    fn get_name(&self) -> &str;

    /// Get the template description
    fn get_description(&self) -> &str;

    /// Get the parameter definitions
    fn get_parameters(&self) -> &[ParameterDefinition];

    /// Get the template content
    fn get_template_content(&self) -> &str;

    /// Get the complexity level
    fn get_complexity_level(&self) -> ComplexityLevel;

    /// Validate parameters against template requirements
    fn validate_parameters(
        &self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), TemplateError>;

    /// Render the template with provided parameters
    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError>;
}

/// Type-safe template implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template<T> {
    /// Template name
    name: String,
    /// Template description
    description: String,
    /// Parameter definitions
    parameters: Vec<ParameterDefinition>,
    /// Template content with placeholders
    content: String,
    /// Complexity level
    complexity_level: ComplexityLevel,
    /// Phantom data for type safety
    _phantom: PhantomData<T>,
}

impl<T> Template<T> {
    /// Create a new template
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

/// Research type markers for type safety
pub struct DecisionMarker;
pub struct ImplementationMarker;
pub struct TroubleshootingMarker;
pub struct LearningMarker;
pub struct ValidationMarker;

/// Implementation of ResearchTemplate for Decision templates
impl ResearchTemplate for Template<DecisionMarker> {
    fn get_type(&self) -> ResearchType {
        ResearchType::Decision
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

    fn validate_parameters(
        &self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), TemplateError> {
        let validator = ParameterValidator::new(&self.parameters);
        validator.validate(params)?;
        Ok(())
    }

    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
        // Validate parameters first
        self.validate_parameters(params)?;

        // Substitute parameters
        let engine = SubstitutionEngine::new()?;
        let result = engine.substitute(&self.content, params)?;

        Ok(result)
    }
}

/// Implementation of ResearchTemplate for Implementation templates
impl ResearchTemplate for Template<ImplementationMarker> {
    fn get_type(&self) -> ResearchType {
        ResearchType::Implementation
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

    fn validate_parameters(
        &self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), TemplateError> {
        let validator = ParameterValidator::new(&self.parameters);
        validator.validate(params)?;
        Ok(())
    }

    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
        // Validate parameters first
        self.validate_parameters(params)?;

        // Substitute parameters
        let engine = SubstitutionEngine::new()?;
        let result = engine.substitute(&self.content, params)?;

        Ok(result)
    }
}

/// Implementation of ResearchTemplate for Troubleshooting templates
impl ResearchTemplate for Template<TroubleshootingMarker> {
    fn get_type(&self) -> ResearchType {
        ResearchType::Troubleshooting
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

    fn validate_parameters(
        &self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), TemplateError> {
        let validator = ParameterValidator::new(&self.parameters);
        validator.validate(params)?;
        Ok(())
    }

    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
        // Validate parameters first
        self.validate_parameters(params)?;

        // Substitute parameters
        let engine = SubstitutionEngine::new()?;
        let result = engine.substitute(&self.content, params)?;

        Ok(result)
    }
}

/// Implementation of ResearchTemplate for Learning templates
impl ResearchTemplate for Template<LearningMarker> {
    fn get_type(&self) -> ResearchType {
        ResearchType::Learning
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

    fn validate_parameters(
        &self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), TemplateError> {
        let validator = ParameterValidator::new(&self.parameters);
        validator.validate(params)?;
        Ok(())
    }

    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
        // Validate parameters first
        self.validate_parameters(params)?;

        // Substitute parameters
        let engine = SubstitutionEngine::new()?;
        let result = engine.substitute(&self.content, params)?;

        Ok(result)
    }
}

/// Implementation of ResearchTemplate for Validation templates
impl ResearchTemplate for Template<ValidationMarker> {
    fn get_type(&self) -> ResearchType {
        ResearchType::Validation
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

    fn validate_parameters(
        &self,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<(), TemplateError> {
        let validator = ParameterValidator::new(&self.parameters);
        validator.validate(params)?;
        Ok(())
    }

    fn render(&self, params: &HashMap<String, ParameterValue>) -> Result<String, TemplateError> {
        // Validate parameters first
        self.validate_parameters(params)?;

        // Substitute parameters
        let engine = SubstitutionEngine::new()?;
        let result = engine.substitute(&self.content, params)?;

        Ok(result)
    }
}

/// Builder for creating templates
pub struct TemplateBuilder<T> {
    name: Option<String>,
    description: Option<String>,
    parameters: Vec<ParameterDefinition>,
    content: Option<String>,
    complexity_level: ComplexityLevel,
    _phantom: PhantomData<T>,
}

impl<T> TemplateBuilder<T> {
    /// Create a new template builder
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

    /// Set the template name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the template description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the template content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Add a parameter definition
    pub fn parameter(mut self, param: ParameterDefinition) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set the complexity level
    pub fn complexity_level(mut self, level: ComplexityLevel) -> Self {
        self.complexity_level = level;
        self
    }

    /// Build the template
    pub fn build(self) -> Result<Template<T>, TemplateError> {
        let name = self.name.ok_or_else(|| {
            TemplateError::ValidationError("Template name is required".to_string())
        })?;

        let description = self.description.ok_or_else(|| {
            TemplateError::ValidationError("Template description is required".to_string())
        })?;

        let content = self.content.ok_or_else(|| {
            TemplateError::ValidationError("Template content is required".to_string())
        })?;

        // Validate template content
        let engine = SubstitutionEngine::new()?;
        engine.validate_template(&content)?;

        Ok(Template::new(
            name,
            description,
            self.parameters,
            content,
            self.complexity_level,
        ))
    }
}

impl<T> Default for TemplateBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::parameters::{ParameterBuilder, ParameterType};

    #[test]
    fn test_template_builder() {
        let param = ParameterBuilder::new()
            .name("topic")
            .param_type(ParameterType::Text)
            .description("The topic to research")
            .required()
            .build()
            .unwrap();

        let template = TemplateBuilder::<DecisionMarker>::new()
            .name("Basic Decision Template")
            .description("Template for decision analysis")
            .content("Analyze the decision: {{topic}}")
            .parameter(param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .unwrap();

        assert_eq!(template.get_name(), "Basic Decision Template");
        assert_eq!(template.get_description(), "Template for decision analysis");
        assert_eq!(template.get_type(), ResearchType::Decision);
        assert_eq!(template.get_parameters().len(), 1);
        assert_eq!(template.get_complexity_level(), ComplexityLevel::Basic);
    }

    #[test]
    fn test_template_render() {
        let param = ParameterBuilder::new()
            .name("topic")
            .param_type(ParameterType::Text)
            .description("The topic to research")
            .required()
            .build()
            .unwrap();

        let template = TemplateBuilder::<ImplementationMarker>::new()
            .name("Implementation Template")
            .description("Template for implementation research")
            .content("How to implement: {{topic}}")
            .parameter(param)
            .build()
            .unwrap();

        let mut params = HashMap::new();
        params.insert(
            "topic".to_string(),
            ParameterValue::Text("async Rust".to_string()),
        );

        let result = template.render(&params).unwrap();
        assert_eq!(result, "How to implement: async Rust");
    }

    #[test]
    fn test_template_validation_missing_parameter() {
        let param = ParameterBuilder::new()
            .name("topic")
            .param_type(ParameterType::Text)
            .description("The topic to research")
            .required()
            .build()
            .unwrap();

        let template = TemplateBuilder::<LearningMarker>::new()
            .name("Learning Template")
            .description("Template for learning research")
            .content("Learn about: {{topic}}")
            .parameter(param)
            .build()
            .unwrap();

        let empty_params = HashMap::new();
        let result = template.render(&empty_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_validation_invalid_content() {
        let result = TemplateBuilder::<TroubleshootingMarker>::new()
            .name("Troubleshooting Template")
            .description("Template for troubleshooting")
            .content("Debug: {{unbalanced}")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_different_research_types() {
        let decision_template = TemplateBuilder::<DecisionMarker>::new()
            .name("Decision Template")
            .description("Decision template")
            .content("Decision: {{topic}}")
            .build()
            .unwrap();

        let validation_template = TemplateBuilder::<ValidationMarker>::new()
            .name("Validation Template")
            .description("Validation template")
            .content("Validate: {{topic}}")
            .build()
            .unwrap();

        assert_eq!(decision_template.get_type(), ResearchType::Decision);
        assert_eq!(validation_template.get_type(), ResearchType::Validation);
    }
}
