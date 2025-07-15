# Prompt Engineering - Quick Reference

<meta>
  <title>Prompt Engineering - Quick Reference</title>
  <type>quick-reference</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Essential prompt engineering patterns for AI development
- **Key Approach**: Pattern-based thinking + systematic evaluation + security
- **Core Benefits**: 340% higher ROI, structured solutions, enhanced collaboration
- **When to use**: All AI interactions requiring reliable, secure outputs
- **Related docs**: [Full Guide](../research/prompt-engineering-concepts.md)

## <implementation>Core Strategies</implementation>

### <pattern>Three Basic Approaches</pattern>
```rust
// 1. Instruction-Based (precision)
const INSTRUCTION: &str = "You are a Rust expert. Use thiserror for errors. Follow async/await patterns.";

// 2. Context-Based (accuracy)
struct Context {
    domain: String,
    tech_stack: Vec<String>,
    constraints: Vec<String>,
}

// 3. Example-Based (consistency)
const EXAMPLE: &str = r#"
Example: Error handling
```rust
#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```
Now implement similar for: {task}
"#;
```

### <pattern>Research Request Template</pattern>
```rust
struct ResearchRequest {
    topic: String,
    context: String,
    implementation_need: String,
    required_format: String,
    quality_criteria: String,
}

impl ResearchRequest {
    fn to_prompt(&self) -> String {
        format!(
            "RESEARCH REQUEST: {}\n\
            Project Context: {}\n\
            Implementation Need: {}\n\
            Required Format: {}\n\
            Quality Criteria: {}\n\
            Output Length Target: 800-1200 words",
            self.topic, self.context, self.implementation_need,
            self.required_format, self.quality_criteria
        )
    }
}
```

### <pattern>Chain-of-Thought Structure</pattern>
```rust
const CHAIN_OF_THOUGHT: &str = r#"
Let's solve this step-by-step:

1. **Analyze Requirements**: What exactly needs to be implemented?
2. **Choose Architecture**: Which Rust patterns fit best?
3. **Identify Dependencies**: What crates are needed?
4. **Design Error Handling**: How should errors be structured?
5. **Implement Core Logic**: Write the main functionality
6. **Add Tests**: Create comprehensive test coverage
7. **Optimize**: Review for performance and memory usage

Working through each step:
[Continue with specific implementation]
"#;
```

## <examples>Framework Comparison</examples>

### <framework>Quick Decision Matrix</framework>
```rust
// LangChain: Rapid prototyping, high abstraction
// Strengths: Fast development, community support
// Weaknesses: Over-abstraction, performance issues

// Guidance: Constraint-focused, precise control
// Strengths: Deterministic output, format control
// Weaknesses: Limited flexibility, steep learning curve

// Custom: Tailored implementation
// Strengths: Complete control, optimal performance
// Weaknesses: Development overhead, maintenance burden
```

### <template>Security Pattern</template>
```rust
struct SecurePrompt {
    user_input: String,
    system_instructions: String,
    constraints: Vec<String>,
}

impl SecurePrompt {
    fn validate_input(&self) -> Result<(), SecurityError> {
        // Check for injection patterns
        if self.user_input.contains("ignore previous instructions") {
            return Err(SecurityError::InjectionAttempt);
        }
        Ok(())
    }
    
    fn apply_structure(&self) -> String {
        format!(
            r#"{{
                "system": "{}",
                "user": "{}",
                "constraints": {:?}
            }}"#,
            self.system_instructions,
            self.user_input.replace('"', "\\\""),
            self.constraints
        )
    }
}
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Inconsistent Outputs</issue>
**Problem**: Same prompt, different results
**Solution**: 
```rust
struct ConsistentPrompt {
    content: String,
    temperature: f32,  // Use 0.1 for consistency
    seed: Option<u64>, // Set fixed seed
}
```

### <issue>Token Limit Exceeded</issue>
**Problem**: Prompt too long
**Solution**: 
```rust
fn compress_prompt(prompt: &str) -> String {
    // Remove verbose explanations
    // Use bullet points
    // Apply semantic compression
    prompt.replace("This function does", "Function:")
}
```

### <issue>Security Vulnerabilities</issue>
**Problem**: Prompt injection attacks
**Solution**: Apply multi-layered validation (see full guide)

## <references>Quality Metrics</references>

### <metrics>Key Performance Indicators</metrics>
```rust
struct PromptMetrics {
    accuracy: f64,      // Correctness of outputs
    relevance: f64,     // Alignment with intent
    consistency: f64,   // Reproducibility
    efficiency: f64,    // Token usage optimization
    security: f64,      // Injection resistance
}

// Target thresholds:
// accuracy > 0.85
// consistency > 0.80
// efficiency > 0.70
```

### <validation>Quick Validation Checklist</validation>
- [ ] Clear, specific instructions
- [ ] Relevant context provided
- [ ] Examples included where helpful
- [ ] Security considerations addressed
- [ ] Token efficiency optimized
- [ ] Success criteria defined
- [ ] Error handling specified

## <references>See Also</references>
- [Full Prompt Engineering Guide](../research/prompt-engineering-concepts.md)
- [LLM Documentation Patterns](llm-documentation-patterns.md)
- [Security Patterns](../patterns/security-patterns.md)