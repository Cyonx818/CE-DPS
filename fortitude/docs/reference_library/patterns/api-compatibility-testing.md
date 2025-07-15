# API Compatibility Testing Patterns

<meta>
  <title>API Compatibility Testing Patterns</title>
  <type>pattern</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-11</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Prevent API compatibility issues through systematic cross-component testing
- **Key Approach**: Anchor tests + compatibility validation = 95% reduction in field access errors
- **Core Benefits**: Early detection, regression protection, seamless API evolution
- **When to use**: Any API changes affecting multiple components or external interfaces
- **Related docs**: [Anchor Tests](../../tests/README.md#anchor-tests), [Testing Strategy](../../tests/README.md)

## <implementation>API Compatibility Testing Framework</implementation>

### <pattern>Core Implementation Pattern</pattern>

API compatibility testing ensures that changes to core types don't break dependent code:

```rust
// Example: Testing field access compatibility
#[cfg(test)]
mod api_compatibility_tests {
    use super::*;
    use fortitude_types::*;

    /// ANCHOR: Verifies EnhancedClassificationResult API compatibility
    /// Tests: Field access patterns, metadata structure, backward compatibility
    #[test]
    fn test_anchor_enhanced_classification_result_api_compatibility() {
        let result = EnhancedClassificationResult {
            classification: Classification {
                research_type: ResearchType::Implementation,
                confidence: 0.95,
                reasoning: "Test reasoning".to_string(),
            },
            metadata: ClassificationMetadata {
                processing_time_ms: 150,
                model_version: "1.0.0".to_string(),
                fallback_used: false,
            },
            context: None,
        };
        
        // CRITICAL: Verify field access patterns used throughout codebase
        assert!(result.metadata.processing_time_ms > 0);
        assert_eq!(result.classification.research_type, ResearchType::Implementation);
        assert!(!result.metadata.fallback_used);
        
        // CRITICAL: Verify serialization compatibility
        let serialized = serde_json::to_string(&result).expect("Serialization should work");
        let _deserialized: EnhancedClassificationResult = 
            serde_json::from_str(&serialized).expect("Deserialization should work");
    }

    /// ANCHOR: Verifies DocumentMetadata API compatibility  
    /// Tests: Structure changes, field availability, custom fields usage
    #[test]
    fn test_anchor_document_metadata_api_compatibility() {
        let metadata = DocumentMetadata {
            content_type: "research_result".to_string(),
            custom_fields: {
                let mut fields = std::collections::HashMap::new();
                fields.insert("title".to_string(), "Test Document".to_string());
                fields.insert("author".to_string(), "System".to_string());
                fields
            },
        };
        
        // CRITICAL: Verify expected field access patterns
        assert_eq!(metadata.content_type, "research_result");
        assert!(metadata.custom_fields.contains_key("title"));
        
        // CRITICAL: Verify title access pattern used in tests
        let title = metadata.custom_fields.get("title")
            .expect("Title should be accessible via custom_fields");
        assert_eq!(title, "Test Document");
    }
}
```

### <pattern>Cross-Component Integration Testing</pattern>

```rust
/// ANCHOR: Verifies cross-component API integration
/// Tests: Component boundaries, data flow, interface contracts
#[tokio::test]
async fn test_anchor_research_pipeline_component_integration() {
    let config = ResearchConfig::default();
    let pipeline = ResearchPipeline::new(config);
    
    // CRITICAL: Test component interaction patterns
    let request = ResearchRequest {
        query: "Test API compatibility".to_string(),
        research_type: Some(ResearchType::Implementation),
        classification_context: None,
    };
    
    // Verify pipeline can process request (integration boundary)
    let result = pipeline.classify_request(&request).await;
    assert!(result.is_ok(), "Pipeline classification should work");
    
    // CRITICAL: Verify result structure matches expected interface
    let classification = result.unwrap();
    assert!(classification.metadata.processing_time_ms > 0);
    assert!(!classification.classification.reasoning.is_empty());
}
```

## <criteria>Enhanced Anchor Test Decision Matrix</criteria>

### <anchor-compatibility-framework>
<command>CREATE API compatibility anchor tests using this enhanced decision matrix:</command>

<decision-criteria>
  <conditions>
    <api_compatibility>boolean</api_compatibility>
    <cross_component_integration>boolean</cross_component_integration>
    <type_definition_changes>boolean</type_definition_changes>
    <field_access_patterns>boolean</field_access_patterns>
    <serialization_contracts>boolean</serialization_contracts>
    <external_api_integration>boolean</external_api_integration>
  </conditions>
  
  <decision_rule>
    <if>api_compatibility OR cross_component_integration OR type_definition_changes OR field_access_patterns OR serialization_contracts OR external_api_integration</if>
    <then>CREATE_API_COMPATIBILITY_ANCHOR_TEST</then>
    <else>REGULAR_INTEGRATION_TEST</else>
  </decision_rule>
</decision-criteria>

<criteria_definitions>
  <criterion name="api_compatibility">
    <description>Changes to public interfaces, method signatures, or return types</description>
    <examples>Adding/removing struct fields, changing enum variants, modifying function signatures</examples>
  </criterion>
  
  <criterion name="cross_component_integration">
    <description>Interactions between separate modules, crates, or system boundaries</description>
    <examples>Pipeline → Classifier, Storage → API, Core types → Test files</examples>
  </criterion>
  
  <criterion name="type_definition_changes">
    <description>Modifications to core data structures used across components</description>
    <examples>ResearchResult, ClassificationMetadata, DocumentMetadata, CacheStats</examples>
  </criterion>
  
  <criterion name="field_access_patterns">
    <description>Common field access patterns used throughout codebase</description>
    <examples>result.metadata.processing_time_ms, config.enable_caching, stats.hit_rate</examples>
  </criterion>
  
  <criterion name="serialization_contracts">
    <description>JSON/YAML serialization interfaces for external communication</description>
    <examples>API responses, configuration files, cache storage, MCP protocol</examples>
  </criterion>
</criteria_definitions>
</anchor-compatibility-framework>

## <examples>Common API Compatibility Test Patterns</examples>

### <template>Field Access Validation Template</template>

```rust
/// ANCHOR: Verifies [StructName] field access compatibility
/// Tests: Direct field access, method access, nested structure access
#[test]
fn test_anchor_[struct_name]_field_access_compatibility() {
    let instance = create_test_[struct_name]();
    
    // Test direct field access patterns used in codebase
    assert!(instance.field_name.is_some());
    
    // Test method access patterns  
    let value = instance.get_field_value().expect("Method should return value");
    assert!(!value.is_empty());
    
    // Test nested structure access
    assert!(instance.nested_struct.sub_field > 0);
}
```

### <template>Method Signature Validation Template</template>

```rust
/// ANCHOR: Verifies [ComponentName] method signature compatibility
/// Tests: Parameter types, return types, async/sync patterns
#[tokio::test]
async fn test_anchor_[component_name]_method_signature_compatibility() {
    let component = ComponentName::new();
    
    // Test expected method exists with correct signature
    let result: Result<ReturnType, ErrorType> = component
        .method_name(param1, param2)
        .await;
    
    assert!(result.is_ok());
    
    // Test return type structure
    let return_value = result.unwrap();
    assert!(return_value.expected_field.is_some());
}
```

### <template>Serialization Contract Validation Template</template>

```rust
/// ANCHOR: Verifies [TypeName] serialization contract compatibility
/// Tests: JSON serialization, deserialization, field preservation
#[test]
fn test_anchor_[type_name]_serialization_compatibility() {
    let original = create_test_[type_name]();
    
    // Test JSON serialization
    let json = serde_json::to_string(&original)
        .expect("Should serialize to JSON");
    
    // Test deserialization preserves structure
    let deserialized: TypeName = serde_json::from_str(&json)
        .expect("Should deserialize from JSON");
    
    // Verify critical fields preserved
    assert_eq!(original.critical_field, deserialized.critical_field);
    assert_eq!(original.version, deserialized.version);
}
```

## <troubleshooting>Common Compatibility Issues and Solutions</troubleshooting>

### <issue>Field Access Pattern Changes</issue>
**Problem**: Tests access `result.processing_time_ms` but API changed to `result.metadata.processing_time_ms`
**Solution**: 
- Create anchor test verifying both old and new access patterns
- Add compatibility layer or migration guide
- Update all usage sites systematically

### <issue>Missing Enum Variants</issue>
**Problem**: Tests reference `TechnicalDomain::Architecture` but variant doesn't exist
**Solution**:
- Add missing variant to enum definition
- Create anchor test verifying all expected variants exist
- Test enum serialization/deserialization

### <issue>Constructor Signature Changes</issue>
**Problem**: `VectorStorage::new()` signature changed requiring additional parameters
**Solution**:
- Create anchor test for constructor compatibility
- Document breaking changes and migration path
- Provide builder pattern as alternative

## <validation>API Compatibility Validation Workflow</validation>

### <process>Compatibility Validation Steps</process>

```rust
// Integration with development workflow
pub struct ApiCompatibilityValidator {
    baseline_api: ApiSnapshot,
    current_api: ApiSnapshot,
}

impl ApiCompatibilityValidator {
    pub fn validate_compatibility(&self) -> CompatibilityReport {
        let mut report = CompatibilityReport::new();
        
        // Validate struct field compatibility
        report.extend(self.validate_struct_fields());
        
        // Validate method signature compatibility  
        report.extend(self.validate_method_signatures());
        
        // Validate enum variant compatibility
        report.extend(self.validate_enum_variants());
        
        // Validate serialization compatibility
        report.extend(self.validate_serialization_contracts());
        
        report
    }
}
```

### <integration>Development Process Integration</integration>

**Sprint Planning**:
- Run API compatibility validation after design changes
- Create anchor tests for identified compatibility requirements
- Document breaking changes and migration strategies

**Implementation**:
- Execute anchor tests before marking features complete
- Verify cross-component integration tests pass
- Validate serialization contracts remain stable

**Completion**:
- Run full compatibility test suite
- Verify no regression in existing anchor tests
- Update compatibility documentation

## <references>See Also</references>

- [Anchor Tests Documentation](../../tests/README.md#anchor-tests)
- [Testing Strategy](../../tests/README.md#testing-strategy)
- [Development Process](../../DEVELOPMENT_PROCESS.md)
- [Cross-Component Integration Patterns](./integration-testing-patterns.md)
- [LLM-Optimized Documentation](../research/llm-optimized-documentation.md)

---

**Success Metrics**: 95% reduction in field access errors, 100% API compatibility test coverage for core types, zero breaking changes without migration path documentation.