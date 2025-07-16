# <context>AI Implementation Patterns - Code Templates and Best Practices</context>

<meta>
  <title>AI Implementation Patterns - Code Templates and Best Practices</title>
  <type>ai-implementation</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-16</updated>
  <mdeval-score>0.92</mdeval-score>
  <token-efficiency>0.16</token-efficiency>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Standardized code patterns for AI-driven development under human strategic oversight
- **Core Patterns**: Authentication, database repository, API design, testing, configuration management
- **Usage Context**: AI references these patterns before creating new implementations
- **Quality Standards**: Security-first, comprehensive testing, error handling, performance optimization
- **Integration**: Works with CE-DPS quality gates and Fortitude knowledge management
- **Token Optimization**: 6-8x compression with 92% parsing accuracy

## <implementation>Core Implementation Patterns</implementation>

### <pattern-overview priority="high">Pattern Selection Guide</pattern-overview>

**Decision Matrix**:
| Use Case | Pattern | Complexity | Security Level |
|----------|---------|------------|----------------|
| User authentication | JWT Authentication | High | Critical |
| Data persistence | Repository Pattern | Medium | High |
| API endpoints | RESTful with Validation | Medium | High |
| Quality assurance | Comprehensive Testing | High | Critical |
| Environment config | Configuration Management | Low | Medium |

### <pattern priority="critical">1. Authentication & Authorization Patterns</pattern>

#### <constraints priority="high">
- Token expiry: 1 hour default, configurable
- Password requirements: 8+ characters, complexity validation
- Rate limiting: 5 attempts per minute per IP
- Session security: Secure cookies with HttpOnly flag
</constraints>

**JWT Authentication Service Pattern:**
```rust
// AI Implementation Template: JWT Authentication
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // subject (user ID)
    pub exp: usize,   // expiration time
    pub iat: usize,   // issued at
    pub roles: Vec<String>,
}

pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    token_expiry: Duration,
}

impl AuthService {
    pub fn new(secret: &str, token_expiry_hours: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            token_expiry: Duration::from_secs(token_expiry_hours * 3600),
        }
    }
    
    pub fn create_token(&self, user_id: &str, roles: Vec<String>) -> Result<String, AuthError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + self.token_expiry.as_secs() as usize,
            iat: now,
            roles,
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(AuthError::TokenCreation)
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::new(Algorithm::HS256);
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(AuthError::TokenValidation)
    }
    
    pub fn has_role(&self, claims: &Claims, required_role: &str) -> bool {
        claims.roles.iter().any(|role| role == required_role)
    }
}

// AI Pattern: Comprehensive Error Handling
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Token creation failed: {0}")]
    TokenCreation(#[from] jsonwebtoken::errors::Error),
    
    #[error("Token validation failed: {0}")]
    TokenValidation(jsonwebtoken::errors::Error),
    
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Token expired")]
    TokenExpired,
}

// AI Pattern: Authorization Middleware
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn auth_middleware(
    auth_service: AuthService,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));
    
    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claims = auth_service
        .validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add claims to request extensions for use in handlers
    let mut request = request;
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}
```

**Authorization Pattern with Role-Based Access:**
```rust
// AI Pattern: Role-Based Authorization
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> Result<Response, StatusCode> {
    move |request: Request, next: Next| async move {
        let claims = request
            .extensions()
            .get::<Claims>()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        if !claims.roles.iter().any(|role| role == required_role) {
            return Err(StatusCode::FORBIDDEN);
        }
        
        Ok(next.run(request).await)
    }
}

// Usage in API routes
pub fn protected_routes() -> Router {
    Router::new()
        .route("/admin", get(admin_handler))
        .layer(middleware::from_fn(require_role("admin")))
        .route("/user", get(user_handler))
        .layer(middleware::from_fn(require_role("user")))
}
```

### <pattern priority="critical">2. Database Repository Patterns</pattern>

#### <constraints priority="high">
- Connection pooling: 5-50 connections based on environment
- Query timeout: 30 seconds default
- Transaction isolation: READ COMMITTED default
- Migration strategy: Forward-only with rollback procedures
</constraints>

**Repository Pattern with Error Handling:**
```rust
// AI Implementation Template: Repository Pattern
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &NewUser) -> Result<User, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;
    async fn update(&self, id: Uuid, updates: &UserUpdates) -> Result<User, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &NewUser) -> Result<User, RepositoryError> {
        let query = r#"
            INSERT INTO users (id, email, password_hash, roles, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, email, password_hash, roles, created_at, updated_at
        "#;
        
        let now = chrono::Utc::now();
        let id = Uuid::new_v4();
        
        sqlx::query_as!(
            User,
            query,
            id,
            user.email,
            user.password_hash,
            &user.roles,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let query = r#"
            SELECT id, email, password_hash, roles, created_at, updated_at
            FROM users
            WHERE email = $1
        "#;
        
        sqlx::query_as!(User, query, email)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::Database)
    }
}

// AI Pattern: Repository Error Handling
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Entity not found")]
    NotFound,
    
    #[error("Unique constraint violation")]
    UniqueViolation,
    
    #[error("Foreign key constraint violation")]
    ForeignKeyViolation,
}

// AI Pattern: Database Migration
-- migrations/001_create_users.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    roles TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_roles ON users USING GIN(roles);

-- Trigger for updating updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### <pattern priority="high">3. API Design Patterns</pattern>

#### <constraints priority="high">
- Response time: <200ms for standard endpoints
- Payload size: 10MB max request/response
- Rate limiting: 100 requests/minute per authenticated user
- Versioning: URL path versioning (/api/v1/)
</constraints>

**RESTful API with Validation:**
```rust
// AI Implementation Template: REST API with Validation
use axum::{extract::{Path, Query, State}, http::StatusCode, Json, response::Response, Router};
use serde::{Deserialize, Serialize};
use validator::Validate;

// AI Pattern: Request/Response DTOs with Validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    
    #[validate(length(min = 1))]
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub role: Option<String>,
}

// AI Pattern: API Handler with Comprehensive Error Handling
pub async fn create_user(
    State(app_state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // Validate request
    request.validate().map_err(ApiError::Validation)?;
    
    // Hash password
    let password_hash = hash_password(&request.password)
        .map_err(|_| ApiError::InternalServer("Password hashing failed".to_string()))?;
    
    // Create user
    let new_user = NewUser {
        email: request.email,
        password_hash,
        roles: request.roles,
    };
    
    let user = app_state
        .user_repository
        .create(&new_user)
        .await
        .map_err(|e| match e {
            RepositoryError::UniqueViolation => ApiError::Conflict("Email already exists".to_string()),
            _ => ApiError::InternalServer("Database error".to_string()),
        })?;
    
    Ok(Json(UserResponse::from(user)))
}

pub async fn get_users(
    State(app_state): State<AppState>,
    Query(query): Query<UserQuery>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100); // Cap at 100
    
    let users = app_state
        .user_repository
        .find_paginated(page, limit, query.role.as_deref())
        .await
        .map_err(|_| ApiError::InternalServer("Database error".to_string()))?;
    
    let responses = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(responses))
}

// AI Pattern: Centralized Error Handling
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Internal server error: {0}")]
    InternalServer(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Validation(errors) => {
                (StatusCode::BAD_REQUEST, format!("Validation error: {}", errors))
            }
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            ApiError::InternalServer(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));
        
        (status, body).into_response()
    }
}
```

### <pattern priority="high">4. Testing Patterns</pattern>

#### <constraints priority="critical">
- Test coverage: >95% for business logic
- Test execution: <30 seconds for full unit test suite
- Test isolation: No shared state between tests
- Test data: Realistic scenarios with edge cases
</constraints>

**Comprehensive Testing Template:**
```rust
// AI Implementation Template: Comprehensive Testing
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
    
    pub struct TestContext {
        pub pool: PgPool,
        pub user_repository: PostgresUserRepository,
        pub auth_service: AuthService,
        _container: Container<'static, Postgres>,
    }
    
    impl TestContext {
        pub async fn new() -> Self {
            let docker = Cli::default();
            let container = docker.run(Postgres::default());
            let port = container.get_host_port_ipv4(5432);
            
            let database_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
            let pool = PgPool::connect(&database_url).await.unwrap();
            
            // Run migrations
            sqlx::migrate!("./migrations").run(&pool).await.unwrap();
            
            let user_repository = PostgresUserRepository::new(pool.clone());
            let auth_service = AuthService::new("test_secret", 1);
            
            Self {
                pool,
                user_repository,
                auth_service,
                _container: container,
            }
        }
        
        pub fn create_test_user(&self) -> NewUser {
            NewUser {
                email: "test@example.com".to_string(),
                password_hash: "hashed_password".to_string(),
                roles: vec!["user".to_string()],
            }
        }
    }
    
    // ANCHOR: User repository core functionality test
    #[tokio::test]
    async fn test_user_repository_crud() {
        let ctx = TestContext::new().await;
        let test_user = ctx.create_test_user();
        
        // Create user
        let created_user = ctx.user_repository.create(&test_user).await.unwrap();
        assert_eq!(created_user.email, test_user.email);
        assert_eq!(created_user.roles, test_user.roles);
        
        // Find by email
        let found_user = ctx
            .user_repository
            .find_by_email(&test_user.email)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found_user.id, created_user.id);
        
        // Update user
        let updates = UserUpdates {
            roles: Some(vec!["admin".to_string()]),
            ..Default::default()
        };
        let updated_user = ctx
            .user_repository
            .update(created_user.id, &updates)
            .await
            .unwrap();
        assert_eq!(updated_user.roles, vec!["admin"]);
        
        // Delete user
        ctx.user_repository.delete(created_user.id).await.unwrap();
        let deleted_user = ctx
            .user_repository
            .find_by_id(created_user.id)
            .await
            .unwrap();
        assert!(deleted_user.is_none());
    }
    
    // ANCHOR: Authentication security test
    #[tokio::test]
    async fn test_jwt_authentication_security() {
        let auth_service = AuthService::new("test_secret", 1);
        let user_id = "test_user";
        let roles = vec!["user".to_string()];
        
        // Valid token creation and validation
        let token = auth_service.create_token(user_id, roles.clone()).unwrap();
        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.roles, roles);
        
        // Invalid token rejection
        let invalid_token = "invalid.jwt.token";
        assert!(auth_service.validate_token(invalid_token).is_err());
        
        // Expired token rejection (simulate with different service)
        let expired_service = AuthService::new("test_secret", 0); // 0 hour expiry
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        let expired_token = expired_service.create_token(user_id, roles).unwrap();
        // Token should be immediately expired due to 0 hour expiry
        assert!(auth_service.validate_token(&expired_token).is_err());
    }
    
    // API Integration Test
    #[tokio::test]
    async fn test_create_user_api() {
        let ctx = TestContext::new().await;
        let app = create_app(ctx.user_repository, ctx.auth_service).await;
        
        let request_body = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "secure_password".to_string(),
            roles: vec!["user".to_string()],
        };
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let user_response: UserResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(user_response.email, request_body.email);
    }
}
```

### <pattern priority="medium">5. Configuration and Environment Patterns</pattern>

#### <constraints priority="medium">
- Environment separation: dev/staging/production configs
- Secret management: Environment variables for sensitive data
- Validation: Startup-time configuration validation
- Defaults: Secure defaults for all configuration values
</constraints>

**Configuration Management Pattern:**
```rust
// AI Implementation Template: Configuration Management
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: u64,
    pub bcrypt_cost: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Config {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")?,
                max_connections: env::var("DB_MAX_CONNECTIONS")?.parse()?,
                min_connections: env::var("DB_MIN_CONNECTIONS")?.parse()?,
                connection_timeout: env::var("DB_CONNECTION_TIMEOUT")?.parse()?,
            },
            auth: AuthConfig {
                jwt_secret: env::var("JWT_SECRET")?,
                token_expiry_hours: env::var("TOKEN_EXPIRY_HOURS")?.parse()?,
                bcrypt_cost: env::var("BCRYPT_COST")?.parse()?,
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")?.parse()?,
                cors_origins: env::var("CORS_ORIGINS")?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            },
        };
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        if self.auth.jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidConfig("JWT secret too short".to_string()));
        }
        
        if self.auth.bcrypt_cost < 10 || self.auth.bcrypt_cost > 15 {
            return Err(ConfigError::InvalidConfig("BCrypt cost out of range".to_string()));
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] env::VarError),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

## <usage-guidelines priority="high">Pattern Implementation Guidelines</usage-guidelines>

### <implementation-principles>AI Implementation Priorities</implementation-principles>

**Priority Order** (highest to lowest):
1. **Security First**: Authentication, authorization, input validation
2. **Reliability**: Error handling, comprehensive testing
3. **Performance**: Scalability and response time requirements
4. **Maintainability**: Clean architecture and code clarity
5. **Efficiency**: Resource optimization and cost management

### <pattern-selection>Pattern Decision Framework</pattern-selection>

**Selection Criteria**:
```markdown
**IF** user-facing application:
- Use Authentication Patterns (JWT + RBAC)
- Implement API Patterns (REST + validation)
- Apply Testing Patterns (unit + integration + security)

**ELSE IF** data persistence required:
- Use Repository Patterns (async + connection pooling)
- Apply Testing Patterns (unit + integration)
- Implement Configuration Patterns (environment-specific)

**ELSE IF** internal services:
- Use Configuration Patterns (environment + secrets)
- Apply Testing Patterns (unit + performance)
```

### <customization-framework>Adaptive Pattern Implementation</customization-framework>

**Adaptation Strategy**:
```xml
<customization-rules>
  <technology-stack>
    <rust>Use tokio for async, sqlx for database, axum for HTTP</rust>
    <typescript>Use express for HTTP, prisma for database, jest for testing</typescript>
    <python>Use fastapi for HTTP, sqlalchemy for database, pytest for testing</python>
  </technology-stack>
  
  <business-domain>
    <fintech>Enhanced security patterns, audit logging, compliance validation</fintech>
    <healthcare>Data encryption, access logging, HIPAA compliance patterns</healthcare>
    <ecommerce>Payment security, session management, performance optimization</ecommerce>
  </business-domain>
  
  <performance-requirements>
    <high-throughput>Connection pooling, caching layers, async patterns</high-throughput>
    <low-latency>In-memory storage, optimized queries, minimal serialization</low-latency>
    <resource-constrained>Efficient algorithms, memory optimization, lazy loading</resource-constrained>
  </performance-requirements>
</customization-rules>
```

## <validation>Pattern Quality Validation</validation>

### <quality-metrics>Pattern Success Criteria</quality-metrics>

| Pattern Type | Success Metric | Target Value |
|--------------|----------------|---------------|
| Authentication | Login response time | <100ms |
| Repository | Query execution time | <50ms |
| API | Endpoint response time | <200ms |
| Testing | Test coverage | >95% |
| Configuration | Startup validation time | <5s |

### <integration-validation>CE-DPS Integration Requirements</integration-validation>

**Quality Gates**:
- Pattern implementation passes security audit
- Performance benchmarks meet requirements
- Test coverage exceeds 95% threshold
- Documentation includes usage examples
- Error handling covers all failure modes