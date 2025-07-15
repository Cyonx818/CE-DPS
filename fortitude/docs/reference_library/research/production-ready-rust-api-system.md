# Production-Ready Rust API System Implementation

## Document Metadata
- **Type**: Production Implementation Guide
- **Category**: Backend Development
- **Priority**: HIGH - Production-grade API patterns
- **Last Updated**: 2025-01-08
- **Token Budget**: ~3,000 tokens

## Executive Summary

**Framework Recommendation**: Use **Axum** for production APIs in 2025 - optimal balance of performance, ergonomics, and ecosystem integration.

**Key Benefits**:
- Lowest memory footprint per connection
- Performance nearly identical to Actix Web
- Excellent Tower ecosystem compatibility
- Backed by Tokio team

## Core Architecture

### Essential Dependencies

```toml
# Cargo.toml - Production Setup
[dependencies]
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.9", features = ["cookie"] }
tokio = { version = "1.0", features = ["full"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "compression", "trace"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
tower-governor = "0.3"  # Rate limiting
utoipa = { version = "4.2", features = ["axum_extras"] }  # OpenAPI
```

### JWT Authentication Pattern

```rust
use axum::{extract::{Request, State}, middleware::Next, response::Response};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Clone)]
pub struct AuthState {
    pub encoding_key: EncodingKey,
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

### Rate Limiting Implementation

```rust
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

pub fn create_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor> {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(10)       // 10 requests per second
            .burst_size(20)       // Allow bursts up to 20 requests
            .finish()
            .unwrap(),
    );
    GovernorLayer::new(governor_conf)
}
```

## Real-Time Features

### WebSocket Implementation

```rust
use axum::{extract::{ws::{Message, WebSocket}, WebSocketUpgrade}, response::Response};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct WebSocketState {
    connections: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
    broadcaster: broadcast::Sender<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WebSocketState>>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(socket: WebSocket, state: Arc<WebSocketState>) {
    let connection_id = Uuid::new_v4();
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
            let _ = state.broadcast(format!("Echo: {}", text)).await;
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
```

### Server-Sent Events (SSE)

```rust
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

pub async fn sse_handler() -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let mut counter = 0;
        
        loop {
            interval.tick().await;
            counter += 1;
            
            let message = format!("Update #{}: {}", counter, chrono::Utc::now());
            if tx.send(message).await.is_err() {
                break;
            }
        }
    });

    let stream = ReceiverStream::new(rx)
        .map(|msg| Event::default().data(msg))
        .map(Ok::<_, Infallible>);

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

## API Documentation

### OpenAPI with utoipa

```rust
use utoipa::{OpenApi, openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(protected_handler, websocket_handler, sse_handler),
    components(schemas(Claims, User, ApiResponse<String>)),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "realtime", description = "Real-time communication"),
    )
)]
pub struct ApiDoc;

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
                        .build(),
                ),
            )
        }
    }
}
```

## Observability & Monitoring

### OpenTelemetry Setup

```rust
use opentelemetry::{global, trace::{TraceContextExt, Tracer}, Context, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "rust-api"),
                    KeyValue::new("service.version", "1.0.0"),
                ])),
        )
        .install_batch(runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Ok(())
}
```

### Instrumented Handlers

```rust
use tracing::{info, instrument};

#[instrument(
    name = "user_creation",
    skip(payload),
    fields(
        user.email = %payload.email,
        user.username = %payload.username
    )
)]
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
        PasswordService::hash_password(&payload.password).unwrap()
    )
    .fetch_one(&*db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %user.id, "User created successfully");
    Ok(Json(user))
}
```

## Production Configuration

### Complete Application Setup

```rust
use axum::{middleware, Router};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

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
        auth: Arc::new(AuthState::new(b"your-secret-key")),
        websocket: Arc::new(WebSocketState::new()),
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_v1_routes())
        .nest("/api/v2", api_v2_routes())
        .route("/ws", get(websocket_handler))
        .route("/events", get(sse_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
                .layer(create_rate_limiter())
                .layer(middleware::from_fn_with_state(
                    app_state.auth.clone(), 
                    auth_middleware
                ))
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    println!("Swagger UI available at http://0.0.0.0:3000/swagger-ui");
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Testing Strategy

### Integration Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_api_authentication() {
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

        // Make requests up to rate limit
        for _ in 0..10 {
            let response = server.get("/api/v1/public").await;
            assert_eq!(response.status_code(), 200);
        }

        // Next request should be rate limited
        let response = server.get("/api/v1/public").await;
        assert_eq!(response.status_code(), 429);
    }
}
```

## Client SDK Generation

### OpenAPI Code Generation

```bash
# Generate Rust client SDK
docker run --rm \
  -v "${PWD}:/local" \
  openapitools/openapi-generator-cli generate \
  -i /local/api-docs/openapi.json \
  -g rust \
  -o /local/clients/rust \
  --additional-properties=packageName=my_api_client,packageVersion=1.0.0
```

### Generated Client Usage

```rust
use my_api_client::{apis::{configuration::Configuration, default_api}, models::User};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Configuration {
        base_path: "http://localhost:3000".to_string(),
        bearer_access_token: Some("your-jwt-token".to_string()),
        ..Default::default()
    };

    let user = default_api::get_user(&config, 1).await?;
    println!("User: {:?}", user);
    Ok(())
}
```

## Troubleshooting Guide

### Common Issues

**Problem**: JWT token validation fails
**Solution**: Ensure proper secret key management and token expiry handling

**Problem**: Rate limiting too aggressive
**Solution**: Adjust `per_second` and `burst_size` in governor config

**Problem**: WebSocket connections dropping
**Solution**: Implement proper connection lifecycle management and heartbeat

**Problem**: High memory usage
**Solution**: Implement connection pooling and proper cleanup in websocket handlers

### Performance Optimization

1. **Connection Pooling**: Use `sqlx::PgPoolOptions` with appropriate max_connections
2. **Compression**: Enable gzip compression with `CompressionLayer`
3. **Caching**: Implement Redis for session and API response caching
4. **Tracing**: Use structured logging for performance monitoring

## Key Takeaways

1. **Framework Choice**: Axum provides optimal balance for production APIs in 2025
2. **Security**: JWT + middleware pattern with Argon2 password hashing
3. **Scalability**: Tower-governor for rate limiting, OpenTelemetry for observability
4. **Developer Experience**: Automatic OpenAPI documentation with utoipa
5. **Real-time**: Built-in WebSocket and SSE support for modern applications
6. **Production Ready**: Comprehensive monitoring, testing, and client SDK generation

This implementation provides a solid foundation for building production-grade Rust APIs that scale from startup to enterprise with modern features and operational excellence.