# MCP Server Implementation Guide for Rust

<meta>
  <title>MCP Server Implementation Guide</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Build production-ready Model Context Protocol servers in Rust for AI model integration
- **Key Approach**: Authentication, error handling, performance optimization, comprehensive testing
- **Core Benefits**: Type-safe implementation, 100+ concurrent requests, sub-100ms latency
- **When to use**: Creating secure external data/tool access for AI models
- **Related docs**: [Production-Ready Rust API System](production-ready-rust-api-system.md)

## <implementation>Core Server Architecture</implementation>

### <pattern>Project Dependencies</pattern>

```toml
# Essential MCP dependencies
[dependencies]
rmcp = { version = "0.2.0", features = ["server", "transport-io"] }
rust-mcp-schema = "0.2.0"

# Async and networking
tokio = { version = "1.45", features = ["full"] }
axum = { version = "0.7", features = ["ws", "tracing"] }

# Security and validation
jsonwebtoken = "9.0"
bcrypt = "0.15"
validator = { version = "0.18", features = ["derive"] }

# Observability
tracing = "0.1"
prometheus = "0.13"
metrics = "0.22"
```

### <template>Production Server Structure</template>

```rust
use rmcp::{ServerHandler, ServiceExt, model::ServerCapabilities};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ProductionMcpServer {
    config: Arc<ServerConfig>,
    auth_manager: Arc<AuthManager>,
    metrics: Arc<MetricsCollector>,
    connection_pool: Arc<RwLock<ConnectionPool>>,
}

impl ProductionMcpServer {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let config = Arc::new(config);
        let auth_manager = Arc::new(AuthManager::new(&config.auth)?);
        let metrics = Arc::new(MetricsCollector::new()?);
        
        let connection_pool = Arc::new(RwLock::new(
            ConnectionPool::new(&config.database).await?
        ));

        Ok(Self {
            config,
            auth_manager,
            metrics,
            connection_pool,
        })
    }

    pub async fn run(self) -> Result<()> {
        // Graceful shutdown setup
        let shutdown = async {
            tokio::signal::ctrl_c().await
                .expect("Failed to install CTRL+C handler");
        };

        // Run server with metrics
        tokio::select! {
            result = self.serve_mcp() => result?,
            _ = shutdown => info!("Shutting down gracefully"),
        }

        Ok(())
    }
}
```

### <pattern>Tool Implementation Pattern</pattern>

```rust
use rmcp::{tool, schemars};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, schemars::JsonSchema)]
pub struct QueryRequest {
    #[validate(length(min = 1, max = 1000))]
    #[schemars(description = "SQL query (read-only)")]
    pub query: String,
    
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<u32>,
}

#[tool(tool_box)]
impl ProductionMcpServer {
    #[tool(description = "Execute read-only database queries")]
    #[instrument(skip(self), fields(user_id = %claims.sub))]
    async fn database_query(
        &self,
        #[tool(aggr)] request: QueryRequest,
        claims: Claims,  // Injected auth context
    ) -> Result<QueryResponse, ServerError> {
        // Validate request
        request.validate()
            .map_err(|e| ServerError::ValidationError(e.to_string()))?;
        
        // Check permissions
        self.auth_manager
            .check_permission(&claims, "database:read")
            .await?;
        
        // Validate query safety
        self.validate_read_only_query(&request.query)?;
        
        // Execute with metrics
        let start = Instant::now();
        let result = self.execute_query(request).await?;
        
        self.metrics.record_duration(
            "database_query",
            start.elapsed()
        );
        
        Ok(result)
    }
}
```

## <examples>Authentication & Security</examples>

### <template>JWT Authentication System</template>

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // User ID
    pub permissions: Vec<String>,
    pub exp: i64,           // Expiration
    pub iat: i64,           // Issued at
}

pub struct AuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl AuthManager {
    pub fn new(secret: &str) -> Result<Self> {
        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation: Validation::new(Algorithm::HS256),
        })
    }

    pub async fn generate_token(&self, user_id: &str, permissions: Vec<String>) -> Result<String> {
        let claims = Claims {
            sub: user_id.to_string(),
            permissions,
            exp: (Utc::now() + Duration::hours(24)).timestamp(),
            iat: Utc::now().timestamp(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Token generation failed: {}", e))
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| anyhow!("Token validation failed: {}", e))
    }

    pub async fn check_permission(&self, claims: &Claims, required: &str) -> Result<()> {
        if !claims.permissions.contains(&required.to_string()) {
            return Err(anyhow!("Permission denied: {}", required));
        }
        Ok(())
    }
}
```

### <template>Security Middleware</template>

```rust
pub async fn security_middleware(
    State(auth): State<Arc<AuthManager>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, ServerError> {
    // Extract token
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ServerError::Unauthorized)?;
    
    // Verify token
    let claims = auth.verify_token(token).await?;
    
    // Inject claims into request
    req.extensions_mut().insert(claims);
    
    // Continue
    Ok(next.run(req).await)
}
```

## <implementation>Performance Optimization</implementation>

### <pattern>Response Caching</pattern>

```rust
use moka::future::Cache;
use std::time::Duration;

pub struct CacheManager {
    cache: Cache<String, CachedResponse>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(300))
                .build(),
        }
    }

    pub async fn get_or_compute<F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> Result<CachedResponse>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<CachedResponse>>,
    {
        if let Some(cached) = self.cache.get(key).await {
            return Ok(cached);
        }
        
        let response = compute().await?;
        self.cache.insert(key.to_string(), response.clone()).await;
        Ok(response)
    }
}
```

### <pattern>Request Deduplication</pattern>

```rust
use tokio::sync::{Mutex, broadcast};
use std::collections::HashMap;

pub struct RequestDeduplicator {
    in_flight: Arc<Mutex<HashMap<String, broadcast::Sender<Result<Response>>>>>,
}

impl RequestDeduplicator {
    pub async fn deduplicate<F, Fut>(
        &self,
        key: String,
        compute: F,
    ) -> Result<Response>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Response>>,
    {
        let mut in_flight = self.in_flight.lock().await;
        
        if let Some(sender) = in_flight.get(&key) {
            let mut receiver = sender.subscribe();
            drop(in_flight);
            return receiver.recv().await?;
        }
        
        let (sender, _) = broadcast::channel(1);
        in_flight.insert(key.clone(), sender.clone());
        drop(in_flight);
        
        let result = compute().await;
        
        let mut in_flight = self.in_flight.lock().await;
        in_flight.remove(&key);
        
        let _ = sender.send(result.clone());
        result
    }
}
```

## <troubleshooting>Error Handling</troubleshooting>

### <pattern>Comprehensive Error Types</pattern>

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Authentication required")]
    Unauthorized,
    
    #[error("Permission denied: {0}")]
    Forbidden(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServerError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required".to_string()),
            ServerError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServerError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            ServerError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
            "timestamp": Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}
```

## <implementation>Testing Strategy</implementation>

### <template>Integration Tests</template>

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    use mockall::mock;

    mock! {
        Database {
            async fn query(&self, sql: &str) -> Result<Vec<Row>>;
        }
    }

    #[test]
    async fn test_database_query_success() {
        let mut mock_db = MockDatabase::new();
        mock_db
            .expect_query()
            .with(eq("SELECT * FROM users LIMIT 10"))
            .times(1)
            .returning(|_| Ok(vec![]));

        let server = create_test_server(mock_db).await;
        let claims = create_test_claims(vec!["database:read"]);
        
        let request = QueryRequest {
            query: "SELECT * FROM users".to_string(),
            limit: Some(10),
        };

        let result = server.database_query(request, claims).await;
        assert!(result.is_ok());
    }

    #[test]
    async fn test_concurrent_requests() {
        let server = create_test_server().await;
        let handles: Vec<_> = (0..100)
            .map(|i| {
                let server = server.clone();
                tokio::spawn(async move {
                    let request = create_test_request(i);
                    server.process_request(request).await
                })
            })
            .collect();

        let results = futures::future::join_all(handles).await;
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
```

## <constraints>Production Deployment Checklist</constraints>

### <checklist>Security</checklist>
- [ ] JWT authentication implemented
- [ ] Permission-based authorization
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention
- [ ] Rate limiting configured
- [ ] Security headers added

### <checklist>Performance</checklist>
- [ ] Response caching enabled
- [ ] Connection pooling configured
- [ ] Request deduplication active
- [ ] Metrics collection setup
- [ ] Resource limits defined

### <checklist>Reliability</checklist>
- [ ] Graceful shutdown implemented
- [ ] Error handling comprehensive
- [ ] Retry logic for external calls
- [ ] Health checks exposed
- [ ] Logging structured and complete

## <references>See Also</references>
- [Production-Ready Rust API System](production-ready-rust-api-system.md)
- [Observability System Implementation](observability-system-implementation.md)
- [Multi-LLM Provider System](multi-llm-provider-system.md)
- [Testing Patterns](../patterns/testing-patterns.md)