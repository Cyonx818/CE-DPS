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

// ABOUTME: Parameter system for prompt templates with type validation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Supported parameter types for template substitution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    /// Simple text parameter
    Text,
    /// Numeric parameter
    Number,
    /// Boolean parameter
    Boolean,
    /// List of values of a specific type
    List(Box<ParameterType>),
    /// Optional parameter that can be None
    Optional(Box<ParameterType>),
}

/// Complexity level for progressive disclosure
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Basic = 1,
    Intermediate = 2,
    Advanced = 3,
    Expert = 4,
}

impl Default for ComplexityLevel {
    fn default() -> Self {
        Self::Basic
    }
}

/// Parameter definition for templates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Description of the parameter
    pub description: String,
    /// Default value if not provided
    pub default_value: Option<ParameterValue>,
    /// Whether this parameter is required
    pub required: bool,
    /// Complexity level for progressive disclosure
    pub complexity_level: ComplexityLevel,
}

/// Runtime parameter value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    /// Text value
    Text(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// List of values
    List(Vec<ParameterValue>),
    /// No value (for optional parameters)
    None,
}

/// Errors that can occur during parameter operations
#[derive(Error, Debug)]
pub enum ParameterError {
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    #[error("Type mismatch for parameter '{name}': expected {expected:?}, got {actual:?}")]
    TypeMismatch {
        name: String,
        expected: ParameterType,
        actual: ParameterType,
    },
    #[error("Invalid parameter value: {0}")]
    InvalidValue(String),
    #[error("Parameter validation failed: {0}")]
    ValidationError(String),
}

/// Builder for parameter definitions
pub struct ParameterBuilder {
    name: Option<String>,
    param_type: Option<ParameterType>,
    description: Option<String>,
    default_value: Option<ParameterValue>,
    required: bool,
    complexity_level: ComplexityLevel,
}

impl ParameterBuilder {
    /// Create a new parameter builder
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

    /// Set the parameter name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the parameter type
    pub fn param_type(mut self, param_type: ParameterType) -> Self {
        self.param_type = Some(param_type);
        self
    }

    /// Set the parameter description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the default value
    pub fn default_value(mut self, value: ParameterValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Mark parameter as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set complexity level
    pub fn complexity_level(mut self, level: ComplexityLevel) -> Self {
        self.complexity_level = level;
        self
    }

    /// Build the parameter definition
    pub fn build(self) -> Result<ParameterDefinition, ParameterError> {
        let name = self.name.ok_or_else(|| {
            ParameterError::ValidationError("Parameter name is required".to_string())
        })?;

        let param_type = self.param_type.ok_or_else(|| {
            ParameterError::ValidationError("Parameter type is required".to_string())
        })?;

        let description = self.description.ok_or_else(|| {
            ParameterError::ValidationError("Parameter description is required".to_string())
        })?;

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

impl Default for ParameterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameter validator for type checking
pub struct ParameterValidator<'a> {
    definitions: &'a [ParameterDefinition],
}

impl<'a> ParameterValidator<'a> {
    /// Create a new parameter validator
    pub fn new(definitions: &'a [ParameterDefinition]) -> Self {
        Self { definitions }
    }

    /// Validate parameters against definitions
    pub fn validate(&self, params: &HashMap<String, ParameterValue>) -> Result<(), ParameterError> {
        // Check required parameters
        for def in self.definitions {
            if def.required && !params.contains_key(&def.name) {
                return Err(ParameterError::MissingParameter(def.name.clone()));
            }

            if let Some(value) = params.get(&def.name) {
                self.validate_type(&def.name, &def.param_type, value)?;
            }
        }

        Ok(())
    }

    /// Validate a specific parameter type
    fn validate_type(
        &self,
        name: &str,
        expected: &ParameterType,
        actual: &ParameterValue,
    ) -> Result<(), ParameterError> {
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
            (ParameterType::Optional(_inner_type), ParameterValue::None) => Ok(()),
            (ParameterType::Optional(inner_type), value) => {
                self.validate_type(name, inner_type, value)
            }
            _ => Err(ParameterError::TypeMismatch {
                name: name.to_string(),
                expected: expected.clone(),
                actual: self.infer_type(actual),
            }),
        }
    }

    /// Infer the type of a parameter value
    #[allow(clippy::only_used_in_recursion)]
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

/// Progressive disclosure manager for parameters
pub struct ProgressiveDisclosure<'a> {
    definitions: &'a [ParameterDefinition],
    current_level: ComplexityLevel,
}

impl<'a> ProgressiveDisclosure<'a> {
    /// Create a new progressive disclosure manager
    pub fn new(definitions: &'a [ParameterDefinition]) -> Self {
        Self {
            definitions,
            current_level: ComplexityLevel::Basic,
        }
    }

    /// Set the current complexity level
    pub fn set_level(&mut self, level: ComplexityLevel) {
        self.current_level = level;
    }

    /// Get parameters available at current complexity level
    pub fn get_available_parameters(&self) -> Vec<&ParameterDefinition> {
        self.definitions
            .iter()
            .filter(|param| param.complexity_level <= self.current_level)
            .collect()
    }

    /// Check if we can advance to the next level
    pub fn can_advance(&self) -> bool {
        self.current_level < ComplexityLevel::Expert
    }

    /// Advance to the next complexity level
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

    /// Get the current complexity level
    pub fn current_level(&self) -> &ComplexityLevel {
        &self.current_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_builder() {
        let param = ParameterBuilder::new()
            .name("test_param")
            .param_type(ParameterType::Text)
            .description("Test parameter")
            .required()
            .complexity_level(ComplexityLevel::Intermediate)
            .build()
            .unwrap();

        assert_eq!(param.name, "test_param");
        assert_eq!(param.param_type, ParameterType::Text);
        assert_eq!(param.description, "Test parameter");
        assert!(param.required);
        assert_eq!(param.complexity_level, ComplexityLevel::Intermediate);
    }

    #[test]
    fn test_parameter_validator() {
        let definitions = vec![
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
        ];

        let validator = ParameterValidator::new(&definitions);

        // Valid parameters
        let mut params = HashMap::new();
        params.insert(
            "required_text".to_string(),
            ParameterValue::Text("test".to_string()),
        );
        assert!(validator.validate(&params).is_ok());

        // Missing required parameter
        let empty_params = HashMap::new();
        assert!(validator.validate(&empty_params).is_err());

        // Type mismatch
        let mut bad_params = HashMap::new();
        bad_params.insert("required_text".to_string(), ParameterValue::Number(42.0));
        assert!(validator.validate(&bad_params).is_err());
    }

    #[test]
    fn test_progressive_disclosure() {
        let definitions = vec![
            ParameterBuilder::new()
                .name("basic_param")
                .param_type(ParameterType::Text)
                .description("Basic parameter")
                .complexity_level(ComplexityLevel::Basic)
                .build()
                .unwrap(),
            ParameterBuilder::new()
                .name("advanced_param")
                .param_type(ParameterType::Text)
                .description("Advanced parameter")
                .complexity_level(ComplexityLevel::Advanced)
                .build()
                .unwrap(),
        ];

        let mut disclosure = ProgressiveDisclosure::new(&definitions);

        // Start at basic level
        let basic_params = disclosure.get_available_parameters();
        assert_eq!(basic_params.len(), 1);
        assert_eq!(basic_params[0].name, "basic_param");

        // Advance to intermediate level
        disclosure.advance_level();
        let intermediate_params = disclosure.get_available_parameters();
        assert_eq!(intermediate_params.len(), 1); // Still only basic param

        // Advance to advanced level
        disclosure.advance_level();
        let advanced_params = disclosure.get_available_parameters();
        assert_eq!(advanced_params.len(), 2); // Both parameters now available
    }

    #[test]
    fn test_complexity_level_ordering() {
        assert!(ComplexityLevel::Basic < ComplexityLevel::Intermediate);
        assert!(ComplexityLevel::Intermediate < ComplexityLevel::Advanced);
        assert!(ComplexityLevel::Advanced < ComplexityLevel::Expert);
    }
}
