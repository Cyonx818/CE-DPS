// ABOUTME: Template substitution engine for parameter replacement
use crate::prompts::parameters::{ParameterError, ParameterValue};
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during template substitution
#[derive(Error, Debug)]
pub enum SubstitutionError {
    #[error("Template contains unsubstituted placeholders: {0}")]
    UnsubstitutedPlaceholders(String),
    #[error("Invalid placeholder format: {0}")]
    InvalidPlaceholder(String),
    #[error("Parameter error: {0}")]
    ParameterError(#[from] ParameterError),
    #[error("Template substitution failed: {0}")]
    SubstitutionFailed(String),
}

/// Template substitution engine
pub struct SubstitutionEngine {
    /// Regex for finding placeholders
    placeholder_regex: Regex,
}

impl SubstitutionEngine {
    /// Create a new substitution engine
    pub fn new() -> Result<Self, SubstitutionError> {
        let placeholder_regex = Regex::new(r"\{\{([^}]*)\}\}").map_err(|e| {
            SubstitutionError::SubstitutionFailed(format!("Regex compilation failed: {e}"))
        })?;

        Ok(Self { placeholder_regex })
    }

    /// Substitute parameters in template
    pub fn substitute(
        &self,
        template: &str,
        params: &HashMap<String, ParameterValue>,
    ) -> Result<String, SubstitutionError> {
        let mut result = template.to_string();

        // Track which placeholders we've seen
        let mut found_placeholders = Vec::new();

        // Find all placeholders first
        for captures in self.placeholder_regex.captures_iter(template) {
            let full_match = captures.get(0).unwrap().as_str();
            let param_name = captures.get(1).unwrap().as_str().trim();
            found_placeholders.push((full_match.to_string(), param_name.to_string()));
        }

        // Substitute each placeholder
        for (placeholder, param_name) in found_placeholders {
            if let Some(value) = params.get(&param_name) {
                let replacement = self.format_value(value);
                result = result.replace(&placeholder, &replacement);
            }
        }

        // Check for remaining unsubstituted placeholders
        if self.placeholder_regex.is_match(&result) {
            let remaining: Vec<_> = self
                .placeholder_regex
                .captures_iter(&result)
                .map(|cap| cap.get(0).unwrap().as_str())
                .collect();
            return Err(SubstitutionError::UnsubstitutedPlaceholders(
                remaining.join(", "),
            ));
        }

        Ok(result)
    }

    /// Format a parameter value for substitution
    #[allow(clippy::only_used_in_recursion)]
    fn format_value(&self, value: &ParameterValue) -> String {
        match value {
            ParameterValue::Text(s) => s.clone(),
            ParameterValue::Number(n) => n.to_string(),
            ParameterValue::Boolean(b) => b.to_string(),
            ParameterValue::List(values) => values
                .iter()
                .map(|v| self.format_value(v))
                .collect::<Vec<_>>()
                .join(", "),
            ParameterValue::None => String::new(),
        }
    }

    /// Get all placeholder names from a template
    pub fn get_placeholders(&self, template: &str) -> Vec<String> {
        self.placeholder_regex
            .captures_iter(template)
            .map(|cap| cap.get(1).unwrap().as_str().trim().to_string())
            .collect()
    }

    /// Validate template syntax
    pub fn validate_template(&self, template: &str) -> Result<(), SubstitutionError> {
        // Check for balanced braces
        let open_count = template.matches("{{").count();
        let close_count = template.matches("}}").count();

        if open_count != close_count {
            return Err(SubstitutionError::InvalidPlaceholder(
                "Unbalanced template braces".to_string(),
            ));
        }

        // Check for valid parameter names
        for captures in self.placeholder_regex.captures_iter(template) {
            let param_name = captures.get(1).unwrap().as_str().trim();
            if param_name.is_empty() {
                return Err(SubstitutionError::InvalidPlaceholder(
                    "Empty parameter name".to_string(),
                ));
            }

            // Check for valid parameter name characters
            if !param_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(SubstitutionError::InvalidPlaceholder(format!(
                    "Invalid parameter name: {param_name}"
                )));
            }
        }

        Ok(())
    }
}

impl Default for SubstitutionEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create substitution engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::parameters::ParameterValue;

    #[test]
    fn test_simple_substitution() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{name}}, welcome to {{project}}!";

        let mut params = HashMap::new();
        params.insert(
            "name".to_string(),
            ParameterValue::Text("Alice".to_string()),
        );
        params.insert(
            "project".to_string(),
            ParameterValue::Text("Fortitude".to_string()),
        );

        let result = engine.substitute(template, &params).unwrap();
        assert_eq!(result, "Hello Alice, welcome to Fortitude!");
    }

    #[test]
    fn test_number_substitution() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Processing {{count}} items with {{confidence}} confidence";

        let mut params = HashMap::new();
        params.insert("count".to_string(), ParameterValue::Number(42.0));
        params.insert("confidence".to_string(), ParameterValue::Number(0.95));

        let result = engine.substitute(template, &params).unwrap();
        assert_eq!(result, "Processing 42 items with 0.95 confidence");
    }

    #[test]
    fn test_boolean_substitution() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Debug mode: {{debug}}, Production ready: {{production}}";

        let mut params = HashMap::new();
        params.insert("debug".to_string(), ParameterValue::Boolean(true));
        params.insert("production".to_string(), ParameterValue::Boolean(false));

        let result = engine.substitute(template, &params).unwrap();
        assert_eq!(result, "Debug mode: true, Production ready: false");
    }

    #[test]
    fn test_list_substitution() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Technologies: {{technologies}}";

        let mut params = HashMap::new();
        params.insert(
            "technologies".to_string(),
            ParameterValue::List(vec![
                ParameterValue::Text("Rust".to_string()),
                ParameterValue::Text("Tokio".to_string()),
                ParameterValue::Text("Serde".to_string()),
            ]),
        );

        let result = engine.substitute(template, &params).unwrap();
        assert_eq!(result, "Technologies: Rust, Tokio, Serde");
    }

    #[test]
    fn test_optional_none_substitution() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Optional value: {{optional}}";

        let mut params = HashMap::new();
        params.insert("optional".to_string(), ParameterValue::None);

        let result = engine.substitute(template, &params).unwrap();
        assert_eq!(result, "Optional value: ");
    }

    #[test]
    fn test_unsubstituted_placeholders_error() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{name}}, welcome to {{project}}!";

        let mut params = HashMap::new();
        params.insert(
            "name".to_string(),
            ParameterValue::Text("Alice".to_string()),
        );
        // Missing "project" parameter

        let result = engine.substitute(template, &params);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SubstitutionError::UnsubstitutedPlaceholders(_)
        ));
    }

    #[test]
    fn test_get_placeholders() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{name}}, your {{item}} is {{status}}";

        let placeholders = engine.get_placeholders(template);
        assert_eq!(placeholders.len(), 3);
        assert!(placeholders.contains(&"name".to_string()));
        assert!(placeholders.contains(&"item".to_string()));
        assert!(placeholders.contains(&"status".to_string()));
    }

    #[test]
    fn test_validate_template_valid() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{name}}, welcome to {{project_name}}!";

        assert!(engine.validate_template(template).is_ok());
    }

    #[test]
    fn test_validate_template_unbalanced_braces() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{name}, welcome to {{project}}!";

        let result = engine.validate_template(template);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SubstitutionError::InvalidPlaceholder(_)
        ));
    }

    #[test]
    fn test_validate_template_empty_parameter() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{}}, welcome!";

        let result = engine.validate_template(template);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SubstitutionError::InvalidPlaceholder(_)
        ));
    }

    #[test]
    fn test_validate_template_invalid_parameter_name() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{name-with-dash}}, welcome!";

        let result = engine.validate_template(template);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SubstitutionError::InvalidPlaceholder(_)
        ));
    }

    #[test]
    fn test_whitespace_in_placeholders() {
        let engine = SubstitutionEngine::new().unwrap();
        let template = "Hello {{ name }}, welcome to {{ project }}!";

        let mut params = HashMap::new();
        params.insert(
            "name".to_string(),
            ParameterValue::Text("Alice".to_string()),
        );
        params.insert(
            "project".to_string(),
            ParameterValue::Text("Fortitude".to_string()),
        );

        let result = engine.substitute(template, &params).unwrap();
        assert_eq!(result, "Hello Alice, welcome to Fortitude!");
    }
}
