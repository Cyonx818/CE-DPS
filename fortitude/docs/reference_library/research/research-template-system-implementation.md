# Research Template System Implementation

<meta>
  <title>Research Template System Implementation</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Type-safe template system for 5 research types with parameter validation and progressive disclosure
- **Key Features**: Template registry, parameter validation, progressive complexity levels, type-safe builders
- **Core Benefits**: Maintainable templates, runtime validation, complexity-aware parameter exposure
- **When to use**: Sprint 1.2 research engine implementation requiring type-specific prompt templates
- **Dependencies**: serde, thiserror, std collections

## <implementation>Core Architecture</implementation>

### <pattern>Research Template Trait System</pattern>
```rust
// Core template trait for all research types
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

// Type-safe template implementation
#[derive(Debug, Clone)]
pub struct Template<T> {
    name: String,
    description: String,
    parameters: Vec<ParameterDefinition>,
    content: String,
    complexity_level: ComplexityLevel,
    _phantom: PhantomData<T>,
}

// Research type markers
pub struct DecisionMarker;
pub struct ImplementationMarker;
pub struct TroubleshootingMarker;
pub struct LearningMarker;
pub struct ValidationMarker;
```

### <pattern>Parameter System</pattern>
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Text,
    Number,
    Boolean,
    List(Box<ParameterType>),
    Optional(Box<ParameterType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub default_value: Option<ParameterValue>,
    pub required: bool,
    pub complexity_level: ComplexityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    List(Vec<ParameterValue>),
    None,
}
```

## <examples>Implementation Patterns</examples>

### <template>Template Creation with Builder Pattern</template>
```rust
// Type-safe template builder
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
```

### <template>Parameter Validation System</template>
```rust
pub struct ParameterValidator<'a> {
    definitions: &'a [ParameterDefinition],
}

impl<'a> ParameterValidator<'a> {
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
}
```

### <template>Template Substitution Engine</template>
```rust
pub struct SubstitutionEngine;

impl SubstitutionEngine {
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
```

## <concept>Progressive Disclosure System</concept>

### <concept>Complexity Level Management</concept>
```rust
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum ComplexityLevel {
    Basic = 1,
    Intermediate = 2,
    Advanced = 3,
    Expert = 4,
}

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

    pub fn get_available_parameters(&self) -> Vec<&ParameterDefinition> {
        self.template
            .get_parameters()
            .iter()
            .filter(|param| param.complexity_level <= self.current_level)
            .collect()
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
```

### <concept>Template Registry</concept>
```rust
pub struct TemplateRegistry {
    templates: HashMap<String, Box<dyn ResearchTemplate>>,
    type_index: HashMap<ResearchType, Vec<String>>,
}

impl TemplateRegistry {
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
}
```

## <examples>Default Template Examples</examples>

### <template>Decision Research Template</template>
```rust
// Decision analysis template with progressive disclosure
let decision_template = TemplateBuilder::<DecisionMarker>::new()
    .name("Basic Decision Analysis")
    .description("Template for decision analysis with progressive complexity")
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

{{#if risk_analysis}}
## Risk Analysis
{{risk_analysis}}
{{/if}}

{{#if stakeholder_impact}}
## Stakeholder Impact
{{stakeholder_impact}}
{{/if}}

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
            .complexity_level(ComplexityLevel::Basic)
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
    .parameter(
        ParameterBuilder::new()
            .name("risk_analysis")
            .param_type(ParameterType::Optional(Box::new(ParameterType::Text)))
            .description("Risk assessment for each option")
            .complexity_level(ComplexityLevel::Advanced)
            .build()
            .unwrap()
    )
    .build()
    .unwrap();
```

### <template>Implementation Research Template</template>
```rust
// Implementation planning template
let implementation_template = TemplateBuilder::<ImplementationMarker>::new()
    .name("Feature Implementation Plan")
    .description("Template for detailed implementation planning")
    .content(r#"
# Implementation Plan: {{feature_name}}

## Overview
**Feature**: {{feature_name}}
**Timeline**: {{timeline}}
**Priority**: {{priority}}

## Technical Requirements
{{requirements}}

## Architecture Considerations
{{architecture}}

## Implementation Steps
{{#each steps}}
{{@index}}. {{this}}
{{/each}}

## Testing Strategy
{{testing_strategy}}

{{#if performance_requirements}}
## Performance Requirements
{{performance_requirements}}
{{/if}}

{{#if security_considerations}}
## Security Considerations
{{security_considerations}}
{{/if}}

## Deployment Plan
{{deployment_plan}}

## Success Metrics
{{success_metrics}}
"#)
    .parameter(
        ParameterBuilder::new()
            .name("feature_name")
            .param_type(ParameterType::Text)
            .description("Name of the feature to implement")
            .required()
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .unwrap()
    )
    .parameter(
        ParameterBuilder::new()
            .name("timeline")
            .param_type(ParameterType::Text)
            .description("Estimated implementation timeline")
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .unwrap()
    )
    .parameter(
        ParameterBuilder::new()
            .name("steps")
            .param_type(ParameterType::List(Box::new(ParameterType::Text)))
            .description("Detailed implementation steps")
            .required()
            .complexity_level(ComplexityLevel::Intermediate)
            .build()
            .unwrap()
    )
    .parameter(
        ParameterBuilder::new()
            .name("performance_requirements")
            .param_type(ParameterType::Optional(Box::new(ParameterType::Text)))
            .description("Performance benchmarks and requirements")
            .complexity_level(ComplexityLevel::Advanced)
            .build()
            .unwrap()
    )
    .build()
    .unwrap();
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### <issue>Template Compilation Errors</issue>
**Problem**: Template content has syntax errors or invalid placeholders
**Solution**: 
```rust
// Validate template during build
pub fn validate_template_syntax(content: &str) -> Result<(), TemplateError> {
    // Check for balanced braces
    let open_count = content.matches("{{").count();
    let close_count = content.matches("}}").count();
    
    if open_count != close_count {
        return Err(TemplateError::ValidationError(
            "Unbalanced template braces".to_string()
        ));
    }
    
    // Check for valid parameter names
    let placeholder_regex = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
    for captures in placeholder_regex.captures_iter(content) {
        let param_name = &captures[1];
        if !param_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(TemplateError::ValidationError(
                format!("Invalid parameter name: {}", param_name)
            ));
        }
    }
    
    Ok(())
}
```

### <issue>Parameter Type Mismatches</issue>
**Problem**: Runtime parameter values don't match defined types
**Solution**: 
```rust
// Use type-safe parameter builders
let param = ParameterBuilder::new()
    .name("count")
    .param_type(ParameterType::Number)
    .description("Number of items")
    .required()
    .build()?;

// Validate at runtime
let mut params = HashMap::new();
params.insert("count".to_string(), ParameterValue::Number(42.0));

template.validate_parameters(&params)?;
```

### <issue>Missing Required Parameters</issue>
**Problem**: Template rendering fails due to missing required parameters
**Solution**: 
```rust
// Check required parameters before rendering
impl ResearchTemplate for Template<T> {
    fn validate_parameters(&self, params: &HashMap<String, ParameterValue>) -> Result<(), TemplateError> {
        let validator = ParameterValidator::new(&self.parameters);
        validator.validate(params)?;
        
        // Provide helpful error messages
        for param_def in &self.parameters {
            if param_def.required && !params.contains_key(&param_def.name) {
                return Err(TemplateError::MissingParameter(format!(
                    "Required parameter '{}' missing. Description: {}",
                    param_def.name, param_def.description
                )));
            }
        }
        
        Ok(())
    }
}
```

## <references>Integration with Fortitude Pipeline</references>

### <integration>Template Selection for Research Types</integration>
```rust
// Integration with research pipeline
impl ResearchEngine {
    pub fn select_template(&self, research_type: &ResearchType) -> Result<&dyn ResearchTemplate, TemplateError> {
        let templates = self.template_registry.get_by_type(research_type);
        
        if templates.is_empty() {
            return Err(TemplateError::TemplateNotFound(
                format!("No templates found for research type: {:?}", research_type)
            ));
        }
        
        // Select the most appropriate template (e.g., based on complexity)
        templates.into_iter()
            .min_by_key(|t| t.get_complexity_level())
            .ok_or_else(|| TemplateError::TemplateNotFound(
                "No suitable template found".to_string()
            ))
    }
    
    pub fn build_research_prompt(&self, request: &ClassifiedRequest) -> Result<String, TemplateError> {
        let template = self.select_template(&request.research_type)?;
        
        // Extract parameters from request
        let mut params = HashMap::new();
        params.insert("topic".to_string(), ParameterValue::Text(request.topic.clone()));
        params.insert("context".to_string(), ParameterValue::Text(request.context.clone()));
        
        // Add type-specific parameters
        match request.research_type {
            ResearchType::Decision => {
                params.insert("problem".to_string(), ParameterValue::Text(request.topic.clone()));
            }
            ResearchType::Implementation => {
                params.insert("feature_name".to_string(), ParameterValue::Text(request.topic.clone()));
            }
            // ... other types
        }
        
        template.render(&params)
    }
}
```

### <integration>Testing Templates</integration>
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_template_rendering() {
        let template = create_decision_template();
        
        let mut params = HashMap::new();
        params.insert("problem".to_string(), ParameterValue::Text("Should we adopt AI?".to_string()));
        params.insert("criteria".to_string(), ParameterValue::Text("Cost, benefits, risks".to_string()));
        
        let result = template.render(&params).unwrap();
        assert!(result.contains("Should we adopt AI?"));
        assert!(result.contains("Cost, benefits, risks"));
    }

    #[test]
    fn test_progressive_disclosure() {
        let template = create_complex_template();
        let mut disclosure = ProgressiveDisclosure::new(&template);
        
        // Start with basic parameters
        let basic_params = disclosure.get_available_parameters();
        assert_eq!(basic_params.len(), 2);
        
        // Advance to intermediate level
        disclosure.advance_level();
        let intermediate_params = disclosure.get_available_parameters();
        assert!(intermediate_params.len() > basic_params.len());
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();
        registry.register(create_decision_template());
        registry.register(create_implementation_template());
        
        let decision_templates = registry.get_by_type(&ResearchType::Decision);
        assert_eq!(decision_templates.len(), 1);
        
        let implementation_templates = registry.get_by_type(&ResearchType::Implementation);
        assert_eq!(implementation_templates.len(), 1);
    }
}
```

---

**Implementation Ready**: This template system provides type-safe, validated templates for all 5 research types with progressive disclosure and comprehensive error handling. The system is designed for easy integration with the Fortitude research pipeline and supports extensible template management through the registry pattern.