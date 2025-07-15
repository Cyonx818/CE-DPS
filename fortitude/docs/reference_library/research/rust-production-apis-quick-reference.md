# Production-Ready Rust API Systems

## Quick Reference

**Framework**: **Axum** (recommended 2025) - optimal performance/ergonomics balance  
**Auth**: JWT + Argon2 password hashing  
**Rate Limiting**: tower-governor with GCRA algorithm  
**Real-time**: WebSockets + Server-Sent Events  
**Documentation**: utoipa for OpenAPI generation  
**Monitoring**: OpenTelemetry + Prometheus

## Core Dependencies

```toml
# Essential Production Stack
axum = { version = "0.8", features = ["macros"] }
tokio = { version = "1.0", features = ["full"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "compression", "trace"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }

# Security & Auth
jsonwebtoken = "9.2"
argon2 = "0.5"
tower-governor = "0.3"  # Rate limiting

# Documentation
utoipa = { version = "4.2", features = ["axum_extras"] }
utoipa-swagger-ui = "6.0"

# Observability
tracing = "0.1"
opentelemetry = "0.23"
prometheus = "0.13"
```

## Authentication System

### JWT Middleware Implementation

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Debug, Clone)]
pub struct AuthState {
    pub decoding_key: DecodingKey<'static>,
}

pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode::<Claims>(token, &auth_state.decoding_key, &Validation::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .claims;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
```

### Argon2 Password Hashing

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}
```

## Rate Limiting

### tower-governor Integration

```rust
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

pub fn create_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor> {
    let config = GovernorConfigBuilder::default()
        .per_second(10)     // 10 requests per second
        .burst_size(20)     // Allow bursts up to 20
        .finish()
        .unwrap();
    
    GovernorLayer::new(Box::new(config))
}

// Custom API key extractor
#[derive(Clone)]
pub struct ApiKeyExtractor;

impl KeyExtractor for ApiKeyExtractor {
    type Key = String;
    
    fn extract<B>(&self, req: &axum::http::Request<B>) -> Result<Self::Key, GovernorError> {
        req.headers()
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .ok_or(GovernorError::UnableToExtractKey)
    }
}
```

## Real-Time Features

### WebSocket Implementation

```rust
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct WebSocketState {
    broadcaster: broadcast::Sender<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WebSocketState>>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(socket: WebSocket, state: Arc<WebSocketState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_receiver = state.broadcaster.subscribe();
    
    // Handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_receiver.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = state.broadcaster.send(format!("Echo: {}", text));
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
```

### Server-Sent Events

```rust
use axum::response::sse::{Event, Sse};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

pub async fn sse_handler() -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let mut counter = 0;
        
        loop {
            interval.tick().await;
            counter += 1;
            
            if tx.send(format!("Update #{}: {}", counter, chrono::Utc::now())).await.is_err() {
                break;
            }
        }
    });

    let stream = ReceiverStream::new(rx)
        .map(|msg| Event::default().data(msg))
        .map(Ok::<_, std::convert::Infallible>);

    Sse::new(stream)
}
```

## API Documentation

### OpenAPI with utoipa

```rust
use utoipa::{OpenApi, Modify};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(api_handler, protected_handler),
    components(schemas(User, ApiResponse<String>)),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "api", description = "Core API endpoints")
    )
)]
pub struct ApiDoc;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

// Add Bearer authentication
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                )
            );
        }
    }
}
```

## Observability

### OpenTelemetry Setup

```rust
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .install_batch(runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Ok(())
}

// Instrumented handler
#[instrument(skip(payload), fields(user.email = %payload.email))]
pub async fn create_user(
    State(db): State<Arc<sqlx::PgPool>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    info!("Creating new user");
    
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
        payload.username,
        payload.email,
        PasswordService::hash_password(&payload.password)?
    )
    .fetch_one(&*db)
    .await?;

    info!(user_id = %user.id, "User created successfully");
    Ok(Json(user))
}
```

## API Versioning

### URL-Based Versioning

```rust
// Version-specific modules
pub mod v1 {
    #[derive(Serialize, Deserialize, ToSchema)]
    pub struct User {
        pub id: i32,
        pub name: String,
        pub email: String,
    }
}

pub mod v2 {
    #[derive(Serialize, Deserialize, ToSchema)]
    pub struct User {
        pub id: uuid::Uuid,    // Changed from i32
        pub full_name: String, // Renamed from 'name'
        pub email: String,
        pub created_at: chrono::DateTime<chrono::Utc>, // New field
    }
}

// Router composition
pub fn create_versioned_router() -> Router {
    Router::new()
        .nest("/api/v1", v1_routes())
        .nest("/api/v2", v2_routes())
        .fallback(version_not_supported)
}
```

## Production Application Structure

### Main Application

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<sqlx::PgPool>,
    pub auth: Arc<AuthState>,
    pub websocket: Arc<WebSocketState>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry()?;

    let db = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&std::env::var("DATABASE_URL")?)
            .await?
    );

    let app_state = AppState {
        db,
        auth: Arc::new(AuthState::new(b"secret-key")),
        websocket: Arc::new(WebSocketState::new()),
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_routes())
        .route("/ws", get(websocket_handler))
        .route("/events", get(sse_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
                .layer(create_rate_limiter())
                .layer(middleware::from_fn_with_state(app_state.auth.clone(), auth_middleware))
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server: http://0.0.0.0:3000");
    println!("Swagger: http://0.0.0.0:3000/swagger-ui");
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Testing Strategy

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_authentication_flow() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Test unauthorized access
        let response = server.get("/api/v1/protected").await;
        assert_eq!(response.status_code(), 401);

        // Test with valid token
        let token = create_test_jwt();
        let response = server
            .get("/api/v1/protected")
            .add_header("Authorization", format!("Bearer {}", token))
            .await;
        assert_eq!(response.status_code(), 200);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();

        // Make requests up to limit
        for _ in 0..10 {
            let response = server.get("/api/v1/public").await;
            assert_eq!(response.status_code(), 200);
        }

        // Should be rate limited
        let response = server.get("/api/v1/public").await;
        assert_eq!(response.status_code(), 429);
    }
}
```

## Key Production Considerations

### Security Checklist
- ✅ JWT tokens with proper expiration
- ✅ Argon2 password hashing
- ✅ Rate limiting per IP/API key
- ✅ CORS configuration
- ✅ Request size limits
- ✅ Input validation

### Performance Optimizations
- ✅ Connection pooling (sqlx)
- ✅ Response compression
- ✅ Async processing
- ✅ Efficient serialization
- ✅ Database query optimization

### Monitoring & Observability
- ✅ Distributed tracing
- ✅ Metrics collection
- ✅ Error tracking
- ✅ Performance monitoring
- ✅ Cost tracking

## Why Axum in 2025

**Performance**: Nearly identical to Actix Web with lower memory footprint  
**Ergonomics**: Simpler API, better error messages, excellent Tower integration  
**Ecosystem**: Strong Tokio team backing, mature middleware ecosystem  
**Developer Experience**: Smooth learning curve, comprehensive documentation  
**Future-Ready**: Active development, modern async patterns

This implementation provides a solid foundation for production-grade Rust APIs that scale from startup to enterprise with modern features and operational excellence.