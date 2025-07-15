# LLM Documentation Quick Reference - CE-DPS

<meta>
  <title>LLM Documentation Quick Reference - CE-DPS</title>
  <type>quick-reference</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-15</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Essential documentation patterns for CE-DPS AI assistants
- **Key Approach**: Semantic markup + progressive disclosure
- **Core Benefits**: 40-60% improved parsing, 6-8x compression
- **When to use**: All AI-generated documentation in CE-DPS projects
- **Related docs**: [Full Guidelines](../methodology/ai-implementation/llm-documentation-guidelines.md)

## <implementation>Core Patterns</implementation>

### <pattern>Basic Semantic Structure</pattern>
```markdown
# <context>Topic Overview</context>

## <method>Implementation Method</method>
### <implementation>
```rust
// Working Rust code example
async fn example() -> Result<(), Error> {
    Ok(())
}
```
### </implementation>

### <constraints>
- Constraint 1
- Constraint 2
### </constraints>
```

### <pattern>Progressive Disclosure</pattern>
```xml
<documentation-unit>
  <summary priority="high">Quick answer</summary>
  <evidence priority="medium">Supporting details</evidence>
  <implementation priority="low">Full code example</implementation>
</documentation-unit>
```

### <pattern>Token-Efficient Code</pattern>
```rust
// GOOD: Concise with semantic clarity
/// **Parameters:**
/// - `data` (String): Input data
/// - `config` (Config): Settings
async fn process(data: String, config: Config) -> Result<Output, ProcessError> {
    // Core logic with >95% test coverage
    Ok(Output::default())
}

// AVOID: Verbose descriptions
// "This function accepts a parameter called data which should be a string..."
```

## <examples>CE-DPS Templates</examples>

### <template>API Documentation</template>
```xml
<api-endpoint>
  <identity>
    <name>GET /api/resource</name>
    <purpose>Brief description</purpose>
  </identity>
  <parameters>
    <required>
      <param name="id" type="String">Resource ID</param>
    </required>
  </parameters>
  <response-format>
    <success code="200">{"data": "..."}</success>
    <error code="404">Resource not found</error>
  </response-format>
  <security-requirements>
    <authentication>JWT Bearer token required</authentication>
    <input-validation>ID must be alphanumeric</input-validation>
  </security-requirements>
</api-endpoint>
```

### <template>Error Handling Pattern</template>
```rust
#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

async fn operation() -> Result<Data, MyError> {
    let response = reqwest::get("url").await?;
    let data = response.json().await?;
    Ok(data)
}

// ANCHOR: Core functionality regression test
#[tokio::test]
async fn test_operation_success() {
    // Test implementation with >95% coverage
    assert!(operation().await.is_ok());
}
```

### <template>Security-First Implementation</template>
```rust
use validator::Validate;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Deserialize, Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
}

async fn secure_operation(input: UserInput) -> Result<Response, SecurityError> {
    // Input validation is mandatory in CE-DPS
    input.validate()?;
    
    // Security-first implementation
    let hashed = hash(&input.password, DEFAULT_COST)?;
    
    Ok(Response::success())
}
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Poor LLM Parsing</issue>
**Problem**: AI doesn't understand structure
**Solution**: Add semantic markup:
```markdown
<!-- BEFORE -->
## Authentication
Use bearer tokens.

<!-- AFTER -->
## <method>Authentication</method>
### <implementation>
Use bearer tokens in Authorization header
### </implementation>
### <security-requirements>
- Token expiry: 1 hour
- Rate limit: 100 requests/minute
### </security-requirements>
```

### <issue>Token Limit Exceeded</issue>
**Problem**: Documentation too verbose
**Solution**: Apply compression:
```markdown
<!-- BEFORE (87 tokens) -->
This function accepts three parameters: userEmail which should be a string...

<!-- AFTER (23 tokens) -->
**Parameters:**
- `user_email` (String): User's email
- `user_id` (u64): Unique identifier
```

### <issue>CE-DPS Quality Gates Failing</issue>
**Problem**: Documentation doesn't meet CE-DPS standards
**Solution**: Include required elements:
```rust
// REQUIRED: Security patterns
use validator::Validate;

// REQUIRED: Comprehensive error handling
#[derive(thiserror::Error, Debug)]
enum AppError { ... }

// REQUIRED: Anchor tests for critical functionality
// ANCHOR: Core business logic test
#[tokio::test]
async fn test_core_functionality() {
    // >95% test coverage required
}
```

## <ce-dps-integration>CE-DPS Integration</ce-dps-integration>

### <phase-alignment>Documentation by Phase</phase-alignment>

**Phase 1: Strategic Planning**
- Use `<context>` for business requirements
- Apply progressive disclosure for human review
- Include architectural decision documentation

**Phase 2: Sprint Planning**  
- Document approach with `<method>` tags
- Use `<constraints>` for limitations
- Apply token optimization

**Phase 3: Implementation**
- Generate API docs using templates
- Include comprehensive error handling
- Validate against quality metrics

### <quality-standards>CE-DPS Quality Requirements</quality-standards>

```bash
# Validate documentation meets CE-DPS standards
cargo run --bin quality-gates -- --validate-docs

# Required elements for all CE-DPS documentation:
# - Security-first patterns
# - >95% test coverage documentation  
# - Comprehensive error handling
# - Token-efficient structure
# - Semantic markup for AI parsing
```

### <human-oversight>Human Review Integration</human-oversight>

```xml
<human-review-section>
  <business-alignment>
    <requirement>Feature meets business objectives</requirement>
    <validation>User experience acceptable</validation>
  </business-alignment>
  <technical-approval>
    <architecture>System design approved</architecture>
    <security>Security review completed</security>
  </technical-approval>
</human-review-section>
```

## <references>See Also</references>
- [Full LLM Documentation Guidelines](../methodology/ai-implementation/llm-documentation-guidelines.md)
- [CE-DPS Phase 1: Strategic Planning](../methodology/ai-implementation/phase-1-planning.md)
- [CE-DPS Phase 3: Implementation](../methodology/ai-implementation/phase-3-execution.md)
- [CE-DPS Quality Framework](../methodology/ai-implementation/quality-framework.md)
- [Implementation Patterns](../methodology/ai-implementation/implementation-patterns.md)