# LLM Documentation Patterns - Quick Reference

<meta>
  <title>LLM Documentation Patterns - Quick Reference</title>
  <type>quick-reference</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Essential patterns for LLM-optimized documentation
- **Key Approach**: Semantic markup + progressive disclosure
- **Core Benefits**: 40-60% improved parsing, 6-8x compression
- **When to use**: All documentation for AI assistant consumption
- **Related docs**: [Full Guide](../research/llm-optimized-documentation.md)

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
    // Core logic
    Ok(Output::default())
}

// AVOID: Verbose descriptions
// "This function accepts a parameter called data which should be a string..."
```

## <examples>Templates</examples>

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

## <references>See Also</references>
- [Full LLM Documentation Guide](../research/llm-optimized-documentation.md)
- [Prompt Engineering Quick Reference](prompt-engineering-quick-ref.md)
- [Rust Patterns](rust-patterns.md)