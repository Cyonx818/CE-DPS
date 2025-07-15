# Fortitude API - Rust Client Examples

This directory contains Rust client examples for integrating with the Fortitude API.

## Requirements

Rust 1.70+ required. Build with:

```bash
cargo build --release
```

## Quick Start

1. **Set environment variables**:
```bash
export FORTITUDE_API_KEY="your-api-key-here"
export FORTITUDE_BASE_URL="http://localhost:8080"
```

2. **Basic usage**:
```bash
cargo run --bin basic_example
```

## Available Examples

Build and run examples:

```bash
# Basic client usage
cargo run --bin basic_example

# Research operations
cargo run --bin research_examples

# Classification operations 
cargo run --bin classification_examples

# Cache management
cargo run --bin cache_examples

# Performance testing
cargo run --bin performance_test

# Integration scenarios
cargo run --bin integration_examples
```

## Client Features

- **Type-safe API** with serde serialization
- **Async/await support** with tokio
- **Automatic retries** with exponential backoff  
- **Error handling** with thiserror
- **Request/response logging** with tracing
- **Connection pooling** with reqwest
- **TLS support** with rustls
- **Concurrent request** handling
- **Performance monitoring** and metrics

## Environment Variables

- `FORTITUDE_API_KEY`: Your API authentication key (required)
- `FORTITUDE_BASE_URL`: API server URL (default: http://localhost:8080)
- `FORTITUDE_TIMEOUT`: Request timeout in seconds (default: 30)
- `FORTITUDE_MAX_RETRIES`: Maximum retry attempts (default: 3)
- `RUST_LOG`: Logging level (default: info)

## Error Handling

The client provides structured error handling:

```rust
use fortitude_client::{FortitudeClient, FortitudeError};

#[tokio::main]
async fn main() -> Result<(), FortitudeError> {
    let client = FortitudeClient::new()?;
    
    match client.research("query").await {
        Ok(result) => println!("Success: {:#?}", result),
        Err(FortitudeError::ApiError { code, message, .. }) => {
            eprintln!("API Error {}: {}", code, message);
        }
        Err(e) => eprintln!("Client Error: {}", e),
    }
    
    Ok(())
}
```

## Performance Examples

### Concurrent Requests
```rust
use futures::future::join_all;

let client = FortitudeClient::new()?;
let queries = vec!["query1", "query2", "query3"];

let tasks: Vec<_> = queries.into_iter()
    .map(|query| client.research(query))
    .collect();

let results = join_all(tasks).await;
```

### Custom Configuration
```rust
let client = FortitudeClient::builder()
    .api_key("your-key")
    .base_url("https://api.fortitude.example.com")
    .timeout(Duration::from_secs(60))
    .max_retries(5)
    .build()?;
```

## Testing

Run tests:
```bash
cargo test
```

Run performance tests:
```bash
cargo run --bin performance_test
```

Enable debug logging:
```bash
RUST_LOG=debug cargo run --bin basic_example
```

## Integration with Applications

### Library Usage
Add to your `Cargo.toml`:
```toml
[dependencies]
fortitude-client = { path = "../fortitude-api-client-examples" }
tokio = { version = "1.0", features = ["full"] }
```

### Example Integration
```rust
use fortitude_client::FortitudeClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FortitudeClient::new()?;
    
    // Perform research
    let research_result = client.research("Rust async best practices").await?;
    println!("Found {} results", research_result.data.total_count);
    
    // Classify content
    let classification = client.classify("Technical documentation about Rust").await?;
    println!("Classification confidence: {}", classification.data.confidence);
    
    Ok(())
}
```