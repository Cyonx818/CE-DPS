// ABOUTME: Template registry for managing research templates by type
use crate::prompts::parameters::ComplexityLevel;
use crate::prompts::templates::{ResearchTemplate, TemplateError};
use fortitude_types::research::ResearchType;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during registry operations
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("No templates found for research type: {0:?}")]
    NoTemplatesForType(ResearchType),
    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),
    #[error("Registry error: {0}")]
    RegistryError(String),
}

/// Registry for managing research templates
pub struct TemplateRegistry {
    /// Templates indexed by name
    templates: HashMap<String, Box<dyn ResearchTemplate>>,
    /// Templates indexed by research type
    type_index: HashMap<ResearchType, Vec<String>>,
}

impl TemplateRegistry {
    /// Create a new template registry
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// Register a template in the registry
    pub fn register<T: ResearchTemplate + 'static>(&mut self, template: T) {
        let name = template.get_name().to_string();
        let research_type = template.get_type();

        // Add to type index
        self.type_index
            .entry(research_type)
            .or_default()
            .push(name.clone());

        // Store template
        self.templates.insert(name, Box::new(template));
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&dyn ResearchTemplate> {
        self.templates.get(name).map(|t| t.as_ref())
    }

    /// Get all templates for a research type
    pub fn get_by_type(&self, research_type: &ResearchType) -> Vec<&dyn ResearchTemplate> {
        self.type_index
            .get(research_type)
            .map(|names| names.iter().filter_map(|name| self.get(name)).collect())
            .unwrap_or_default()
    }

    /// Get the best template for a research type based on complexity
    pub fn get_best_for_type(
        &self,
        research_type: &ResearchType,
        complexity: ComplexityLevel,
    ) -> Result<&dyn ResearchTemplate, RegistryError> {
        let templates = self.get_by_type(research_type);

        if templates.is_empty() {
            return Err(RegistryError::NoTemplatesForType(research_type.clone()));
        }

        // Find template with matching or closest complexity level
        let best_template = templates
            .into_iter()
            .min_by_key(|template| {
                let template_complexity = template.get_complexity_level();
                // Prefer templates at or below the requested complexity
                if template_complexity <= complexity {
                    // Lower complexity is better if it's <= requested
                    complexity.clone() as i32 - template_complexity as i32
                } else {
                    // Higher complexity is penalized
                    (template_complexity as i32 - complexity.clone() as i32) * 100
                }
            })
            .ok_or_else(|| RegistryError::NoTemplatesForType(research_type.clone()))?;

        Ok(best_template)
    }

    /// Get all template names
    pub fn get_all_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Get all supported research types
    pub fn get_supported_types(&self) -> Vec<ResearchType> {
        self.type_index.keys().cloned().collect()
    }

    /// Get template count by type
    pub fn get_template_count_by_type(&self, research_type: &ResearchType) -> usize {
        self.type_index
            .get(research_type)
            .map(|names| names.len())
            .unwrap_or(0)
    }

    /// Get total template count
    pub fn get_total_template_count(&self) -> usize {
        self.templates.len()
    }

    /// Check if registry has templates for a research type
    pub fn has_templates_for_type(&self, research_type: &ResearchType) -> bool {
        self.type_index.contains_key(research_type) && !self.type_index[research_type].is_empty()
    }

    /// Remove a template from the registry
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn ResearchTemplate>> {
        if let Some(template) = self.templates.remove(name) {
            let research_type = template.get_type();

            // Remove from type index
            if let Some(names) = self.type_index.get_mut(&research_type) {
                names.retain(|n| n != name);
                if names.is_empty() {
                    self.type_index.remove(&research_type);
                }
            }

            Some(template)
        } else {
            None
        }
    }

    /// Clear all templates
    pub fn clear(&mut self) {
        self.templates.clear();
        self.type_index.clear();
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        let mut stats = RegistryStats {
            total_templates: self.templates.len(),
            templates_by_type: HashMap::new(),
            templates_by_complexity: HashMap::new(),
        };

        for template in self.templates.values() {
            let research_type = template.get_type();
            let complexity = template.get_complexity_level();

            *stats.templates_by_type.entry(research_type).or_insert(0) += 1;
            *stats.templates_by_complexity.entry(complexity).or_insert(0) += 1;
        }

        stats
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about template registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of templates
    pub total_templates: usize,
    /// Number of templates by research type
    pub templates_by_type: HashMap<ResearchType, usize>,
    /// Number of templates by complexity level
    pub templates_by_complexity: HashMap<ComplexityLevel, usize>,
}

/// Factory for creating default templates
pub struct DefaultTemplateFactory;

impl DefaultTemplateFactory {
    /// Create a registry with default templates for all research types
    pub fn create_default_registry() -> TemplateRegistry {
        let mut registry = TemplateRegistry::new();

        // Add default templates for each research type
        Self::register_decision_template(&mut registry);
        Self::register_implementation_template(&mut registry);
        Self::register_troubleshooting_template(&mut registry);
        Self::register_learning_template(&mut registry);
        Self::register_validation_template(&mut registry);

        registry
    }

    /// Register a basic decision template
    fn register_decision_template(registry: &mut TemplateRegistry) {
        use crate::prompts::parameters::{ParameterBuilder, ParameterType};
        use crate::prompts::templates::{DecisionMarker, TemplateBuilder};

        let problem_param = ParameterBuilder::new()
            .name("problem")
            .param_type(ParameterType::Text)
            .description("The decision problem to analyze")
            .required()
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create problem parameter");

        let context_param = ParameterBuilder::new()
            .name("context")
            .param_type(ParameterType::Text)
            .description("Additional context about the decision")
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create context parameter");

        let template = TemplateBuilder::<DecisionMarker>::new()
            .name("Basic Decision Analysis")
            .description("Template for decision analysis with progressive complexity")
            .content(r#"
<summary priority="high">Decision Analysis: {{problem}}</summary>

## Problem Statement
{{problem}}

{{context}}

## Analysis Approach
This decision analysis follows a structured approach to evaluate options and provide a clear recommendation.

## Options Consideration
- Evaluate available alternatives
- Consider constraints and requirements
- Assess risks and benefits

## Recommendation
Based on the analysis above, the recommended approach is to:
- [Specific recommendation will be provided based on the analysis]

<evidence priority="medium">
## Supporting Evidence
- Context analysis shows key factors
- Risk assessment identifies potential challenges
- Stakeholder impact has been considered
</evidence>

<implementation priority="low">
## Implementation Considerations
- Next steps for implementation
- Resource requirements
- Timeline considerations
- Success metrics
</implementation>
"#)
            .parameter(problem_param)
            .parameter(context_param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create decision template");

        registry.register(template);
    }

    /// Register a basic implementation template
    fn register_implementation_template(registry: &mut TemplateRegistry) {
        use crate::prompts::parameters::{ParameterBuilder, ParameterType};
        use crate::prompts::templates::{ImplementationMarker, TemplateBuilder};

        let feature_param = ParameterBuilder::new()
            .name("feature")
            .param_type(ParameterType::Text)
            .description("The feature or functionality to implement")
            .required()
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create feature parameter");

        let technology_param = ParameterBuilder::new()
            .name("technology")
            .param_type(ParameterType::Text)
            .description("The technology stack or framework to use")
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create technology parameter");

        let template = TemplateBuilder::<ImplementationMarker>::new()
            .name("Feature Implementation Guide")
            .description("Template for detailed implementation planning")
            .content(
                r#"
<summary priority="high">Implementation Guide: {{feature}}</summary>

## Overview
This guide provides step-by-step implementation instructions for {{feature}}.

Technology Stack: {{technology}}

## Implementation Approach
The implementation follows established patterns and best practices for maintainable, scalable code.

## Core Requirements
- Functional requirements analysis
- Technical requirements specification
- Architecture considerations

<evidence priority="medium">
## Technical Specifications
- System architecture overview
- Component interactions
- Data flow design
- API specifications
</evidence>

<implementation priority="low">
## Step-by-Step Implementation
1. Set up development environment
2. Create core components
3. Implement business logic
4. Add error handling
5. Write comprehensive tests
6. Deploy and monitor

## Code Examples
[Working code examples will be provided]

## Testing Strategy
- Unit tests for core functionality
- Integration tests for component interactions
- End-to-end tests for user workflows

## Deployment Considerations
- Environment setup
- Configuration management
- Performance optimization
- Monitoring and logging
</implementation>
"#,
            )
            .parameter(feature_param)
            .parameter(technology_param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create implementation template");

        registry.register(template);
    }

    /// Register a basic troubleshooting template
    fn register_troubleshooting_template(registry: &mut TemplateRegistry) {
        use crate::prompts::parameters::{ParameterBuilder, ParameterType};
        use crate::prompts::templates::{TemplateBuilder, TroubleshootingMarker};

        let problem_param = ParameterBuilder::new()
            .name("problem")
            .param_type(ParameterType::Text)
            .description("The problem or issue to troubleshoot")
            .required()
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create problem parameter");

        let symptoms_param = ParameterBuilder::new()
            .name("symptoms")
            .param_type(ParameterType::Text)
            .description("Observed symptoms or error messages")
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create symptoms parameter");

        let template = TemplateBuilder::<TroubleshootingMarker>::new()
            .name("Problem Troubleshooting Guide")
            .description("Template for systematic problem resolution")
            .content(
                r#"
<summary priority="high">Troubleshooting Guide: {{problem}}</summary>

## Problem Description
{{problem}}

## Observed Symptoms
{{symptoms}}

## Diagnostic Approach
This troubleshooting guide follows a systematic approach to identify and resolve the issue.

## Initial Assessment
- Problem scope and impact
- Affected systems or components
- Timeline and frequency

<evidence priority="medium">
## Diagnostic Steps
1. **Symptom Analysis**
   - Review error messages and logs
   - Identify patterns and triggers
   - Assess system state

2. **Root Cause Investigation**
   - Check system configuration
   - Verify dependencies
   - Analyze recent changes

3. **Hypothesis Testing**
   - Test potential causes
   - Validate assumptions
   - Gather additional data
</evidence>

<implementation priority="low">
## Solution Implementation
### Immediate Actions
- Steps to resolve the immediate issue
- Temporary workarounds if needed
- Impact mitigation measures

### Root Cause Resolution
- Permanent fixes for underlying issues
- Configuration adjustments
- System improvements

### Verification Steps
- How to confirm the fix works
- Regression testing
- Monitoring for recurrence

## Prevention Measures
- Best practices to avoid similar issues
- Monitoring and alerting improvements
- Documentation updates
</implementation>
"#,
            )
            .parameter(problem_param)
            .parameter(symptoms_param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create troubleshooting template");

        registry.register(template);
    }

    /// Register a basic learning template
    fn register_learning_template(registry: &mut TemplateRegistry) {
        use crate::prompts::parameters::{ParameterBuilder, ParameterType};
        use crate::prompts::templates::{LearningMarker, TemplateBuilder};

        let concept_param = ParameterBuilder::new()
            .name("concept")
            .param_type(ParameterType::Text)
            .description("The concept, technology, or pattern to learn about")
            .required()
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create concept parameter");

        let level_param = ParameterBuilder::new()
            .name("level")
            .param_type(ParameterType::Text)
            .description("Learning level: beginner, intermediate, or advanced")
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create level parameter");

        let template = TemplateBuilder::<LearningMarker>::new()
            .name("Concept Learning Guide")
            .description("Template for understanding concepts, technologies, and patterns")
            .content(
                r#"
<summary priority="high">Learning Guide: {{concept}}</summary>

## Concept Overview
{{concept}} is a fundamental concept that plays an important role in software development.

Target Level: {{level}}

## Core Concepts
Understanding {{concept}} requires grasping several key principles and their interactions.

## Why This Matters
- Practical applications in development
- Benefits and advantages
- Common use cases

<evidence priority="medium">
## Detailed Explanation
### Key Principles
- Fundamental concepts and theory
- How it works under the hood
- Relationship to other concepts

### Practical Examples
- Real-world applications
- Code examples and demonstrations
- Common patterns and practices

### Best Practices
- Recommended approaches
- Common pitfalls to avoid
- Performance considerations
</evidence>

<implementation priority="low">
## Hands-On Learning
### Getting Started
- Environment setup
- Basic examples
- First steps

### Practice Exercises
- Simple implementations
- Incremental complexity
- Real-world scenarios

### Advanced Topics
- Complex use cases
- Performance optimization
- Integration patterns

## Resources for Further Learning
- Official documentation
- Tutorials and guides
- Community resources
- Books and courses
</implementation>
"#,
            )
            .parameter(concept_param)
            .parameter(level_param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create learning template");

        registry.register(template);
    }

    /// Register a basic validation template
    fn register_validation_template(registry: &mut TemplateRegistry) {
        use crate::prompts::parameters::{ParameterBuilder, ParameterType};
        use crate::prompts::templates::{TemplateBuilder, ValidationMarker};

        let approach_param = ParameterBuilder::new()
            .name("approach")
            .param_type(ParameterType::Text)
            .description("The approach, method, or solution to validate")
            .required()
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create approach parameter");

        let criteria_param = ParameterBuilder::new()
            .name("criteria")
            .param_type(ParameterType::Text)
            .description("Validation criteria or requirements")
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create criteria parameter");

        let template = TemplateBuilder::<ValidationMarker>::new()
            .name("Approach Validation Guide")
            .description("Template for validating approaches, methods, and solutions")
            .content(
                r#"
<summary priority="high">Validation Analysis: {{approach}}</summary>

## Approach Overview
{{approach}} is being evaluated for suitability and effectiveness.

Validation Criteria: {{criteria}}

## Validation Framework
This analysis evaluates the approach against established criteria and best practices.

## Initial Assessment
- Approach summary and scope
- Stated goals and objectives
- Success criteria definition

<evidence priority="medium">
## Detailed Analysis
### Strengths
- Key advantages and benefits
- Alignment with requirements
- Proven success factors

### Weaknesses
- Potential limitations
- Risk factors
- Implementation challenges

### Best Practice Alignment
- Industry standards compliance
- Established patterns usage
- Community acceptance

### Comparative Analysis
- Alternative approaches
- Trade-offs and considerations
- Contextual suitability
</evidence>

<implementation priority="low">
## Validation Results
### Recommendation
- Overall assessment
- Suitability rating
- Conditional recommendations

### Implementation Considerations
- Prerequisites and dependencies
- Resource requirements
- Timeline implications

### Risk Mitigation
- Identified risks and mitigation strategies
- Contingency plans
- Monitoring requirements

## Quality Assurance
### Testing Strategy
- Validation testing approach
- Success metrics
- Acceptance criteria

### Continuous Improvement
- Feedback mechanisms
- Iteration strategies
- Performance monitoring
</implementation>
"#,
            )
            .parameter(approach_param)
            .parameter(criteria_param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .expect("Failed to create validation template");

        registry.register(template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::parameters::{ParameterBuilder, ParameterType};
    use crate::prompts::templates::{DecisionMarker, TemplateBuilder};

    fn create_test_template() -> impl ResearchTemplate {
        let param = ParameterBuilder::new()
            .name("test_param")
            .param_type(ParameterType::Text)
            .description("Test parameter")
            .required()
            .build()
            .unwrap();

        TemplateBuilder::<DecisionMarker>::new()
            .name("Test Template")
            .description("Template for testing")
            .content("Test content: {{test_param}}")
            .parameter(param)
            .complexity_level(ComplexityLevel::Basic)
            .build()
            .unwrap()
    }

    #[test]
    fn test_registry_creation() {
        let registry = TemplateRegistry::new();
        assert_eq!(registry.get_total_template_count(), 0);
        assert!(registry.get_supported_types().is_empty());
    }

    #[test]
    fn test_template_registration() {
        let mut registry = TemplateRegistry::new();
        let template = create_test_template();

        registry.register(template);

        assert_eq!(registry.get_total_template_count(), 1);
        assert!(registry.get("Test Template").is_some());
        assert!(registry.has_templates_for_type(&ResearchType::Decision));
        assert_eq!(
            registry.get_template_count_by_type(&ResearchType::Decision),
            1
        );
    }

    #[test]
    fn test_get_by_type() {
        let mut registry = TemplateRegistry::new();
        let template = create_test_template();

        registry.register(template);

        let templates = registry.get_by_type(&ResearchType::Decision);
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].get_name(), "Test Template");

        let empty_templates = registry.get_by_type(&ResearchType::Implementation);
        assert_eq!(empty_templates.len(), 0);
    }

    #[test]
    fn test_get_best_for_type() {
        let mut registry = TemplateRegistry::new();
        let template = create_test_template();

        registry.register(template);

        let best = registry.get_best_for_type(&ResearchType::Decision, ComplexityLevel::Basic);
        assert!(best.is_ok());
        assert_eq!(best.unwrap().get_name(), "Test Template");

        let no_template =
            registry.get_best_for_type(&ResearchType::Implementation, ComplexityLevel::Basic);
        assert!(no_template.is_err());
    }

    #[test]
    fn test_template_removal() {
        let mut registry = TemplateRegistry::new();
        let template = create_test_template();

        registry.register(template);
        assert_eq!(registry.get_total_template_count(), 1);

        let removed = registry.remove("Test Template");
        assert!(removed.is_some());
        assert_eq!(registry.get_total_template_count(), 0);
        assert!(!registry.has_templates_for_type(&ResearchType::Decision));
    }

    #[test]
    fn test_registry_stats() {
        let mut registry = TemplateRegistry::new();
        let template = create_test_template();

        registry.register(template);

        let stats = registry.get_stats();
        assert_eq!(stats.total_templates, 1);
        assert_eq!(
            stats.templates_by_type.get(&ResearchType::Decision),
            Some(&1)
        );
        assert_eq!(
            stats.templates_by_complexity.get(&ComplexityLevel::Basic),
            Some(&1)
        );
    }

    #[test]
    fn test_default_template_factory() {
        let registry = DefaultTemplateFactory::create_default_registry();

        // Should have templates for all research types
        assert!(registry.has_templates_for_type(&ResearchType::Decision));
        assert!(registry.has_templates_for_type(&ResearchType::Implementation));
        assert!(registry.has_templates_for_type(&ResearchType::Troubleshooting));
        assert!(registry.has_templates_for_type(&ResearchType::Learning));
        assert!(registry.has_templates_for_type(&ResearchType::Validation));

        assert_eq!(registry.get_total_template_count(), 5);
    }
}
