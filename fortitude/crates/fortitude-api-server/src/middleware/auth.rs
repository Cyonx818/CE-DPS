// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Authentication middleware for HTTP API server
// Adapts JWT authentication from MCP server for HTTP Bearer tokens

use crate::config::ApiServerConfig;
use crate::models::errors::ApiError;
use anyhow::{anyhow, Result};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// JWT claims structure containing user information and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// List of permissions granted to the user
    pub permissions: Vec<String>,
    /// Expiration time (timestamp)
    pub exp: i64,
    /// Issued at time (timestamp)
    pub iat: i64,
    /// Issuer
    pub iss: String,
}

/// Permission levels for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Can call research tools
    ResearchRead,
    /// Can access reference library resources
    ResourcesRead,
    /// Can access configuration resources
    ConfigRead,
    /// Can read learning system data
    LearningRead,
    /// Can write to learning system
    LearningWrite,
    /// Can read monitoring data
    MonitoringRead,
    /// Can write monitoring configurations
    MonitoringWrite,
    /// Full administrative access
    Admin,
    /// Read-write access to system resources
    ReadWrite,
    /// System-level permissions
    System,
}

/// API permission levels for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ApiPermissionLevel {
    /// Read-only access
    ReadOnly,
    /// Read-write access
    ReadWrite,
    /// Administrative access
    Admin,
    /// System-level access
    System,
}

impl Permission {
    /// Convert permission to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ResearchRead => "fortitude:research:read",
            Permission::ResourcesRead => "fortitude:resources:read",
            Permission::ConfigRead => "fortitude:config:read",
            Permission::LearningRead => "fortitude:learning:read",
            Permission::LearningWrite => "fortitude:learning:write",
            Permission::MonitoringRead => "fortitude:monitoring:read",
            Permission::MonitoringWrite => "fortitude:monitoring:write",
            Permission::Admin => "fortitude:admin",
            Permission::ReadWrite => "fortitude:readwrite",
            Permission::System => "fortitude:system",
        }
    }

    /// Parse permission from string  
    pub fn parse(permission: &str) -> Option<Self> {
        match permission {
            "fortitude:research:read" => Some(Permission::ResearchRead),
            "fortitude:resources:read" => Some(Permission::ResourcesRead),
            "fortitude:config:read" => Some(Permission::ConfigRead),
            "fortitude:learning:read" => Some(Permission::LearningRead),
            "fortitude:learning:write" => Some(Permission::LearningWrite),
            "fortitude:monitoring:read" => Some(Permission::MonitoringRead),
            "fortitude:monitoring:write" => Some(Permission::MonitoringWrite),
            "fortitude:admin" => Some(Permission::Admin),
            "fortitude:readwrite" => Some(Permission::ReadWrite),
            "fortitude:system" => Some(Permission::System),
            _ => None,
        }
    }

    /// Get all available permissions
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::ResearchRead,
            Permission::ResourcesRead,
            Permission::ConfigRead,
            Permission::LearningRead,
            Permission::LearningWrite,
            Permission::MonitoringRead,
            Permission::MonitoringWrite,
            Permission::Admin,
            Permission::ReadWrite,
            Permission::System,
        ]
    }
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Invalid permission: {s}"))
    }
}

impl ApiPermissionLevel {
    /// Convert to Permission enum
    pub fn to_permission(&self) -> Permission {
        match self {
            ApiPermissionLevel::ReadOnly => Permission::ResourcesRead,
            ApiPermissionLevel::ReadWrite => Permission::ReadWrite,
            ApiPermissionLevel::Admin => Permission::Admin,
            ApiPermissionLevel::System => Permission::System,
        }
    }

    /// Convert from Permission enum
    pub fn from_permission(permission: Permission) -> Self {
        match permission {
            Permission::ResearchRead
            | Permission::ResourcesRead
            | Permission::ConfigRead
            | Permission::LearningRead
            | Permission::MonitoringRead => ApiPermissionLevel::ReadOnly,
            Permission::ReadWrite | Permission::LearningWrite | Permission::MonitoringWrite => {
                ApiPermissionLevel::ReadWrite
            }
            Permission::Admin => ApiPermissionLevel::Admin,
            Permission::System => ApiPermissionLevel::System,
        }
    }

    /// Get all available API permission levels
    pub fn all() -> Vec<ApiPermissionLevel> {
        vec![
            ApiPermissionLevel::ReadOnly,
            ApiPermissionLevel::ReadWrite,
            ApiPermissionLevel::Admin,
            ApiPermissionLevel::System,
        ]
    }
}

/// Rate limiter state for a single client
#[derive(Debug, Clone)]
struct RateLimitState {
    /// Number of requests in current window
    request_count: u32,
    /// Start time of current window
    window_start: Instant,
}

/// Authentication and authorization manager for HTTP API
pub struct AuthManager {
    /// JWT encoding key
    encoding_key: EncodingKey,
    /// JWT decoding key
    decoding_key: DecodingKey,
    /// JWT validation configuration
    validation: Validation,
    /// Server configuration
    config: Arc<ApiServerConfig>,
    /// Rate limiting state per client
    rate_limits: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl AuthManager {
    /// Create new authentication manager
    pub fn new(config: Arc<ApiServerConfig>) -> Result<Self> {
        let jwt_secret = config.auth.jwt_secret.as_bytes();

        let encoding_key = EncodingKey::from_secret(jwt_secret);
        let decoding_key = DecodingKey::from_secret(jwt_secret);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["fortitude-api-server"]);
        validation.validate_exp = true;

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            config: config.clone(),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate JWT token for a user with given permissions
    #[instrument(skip(self, permissions))]
    pub async fn generate_token(
        &self,
        user_id: &str,
        permissions: Vec<Permission>,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.auth.token_expiration_hours as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            permissions: permissions.iter().map(|p| p.as_str().to_string()).collect(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: "fortitude-api-server".to_string(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Token generation failed: {e}"))?;

        info!("Generated JWT token for user: {}", user_id);
        debug!("Token permissions: {:?}", claims.permissions);

        Ok(token)
    }

    /// Verify and decode JWT token
    #[instrument(skip(self, token))]
    pub async fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| anyhow!("Token validation failed: {e}"))?;

        let claims = token_data.claims;

        // Additional validation
        if claims.exp < Utc::now().timestamp() {
            return Err(anyhow!("Token has expired"));
        }

        debug!("Token verified for user: {}", claims.sub);
        Ok(claims)
    }

    /// Check if user has required permission
    #[instrument(skip(self, claims))]
    pub async fn check_permission(&self, claims: &Claims, required: Permission) -> Result<()> {
        let required_str = required.as_str();

        // Admin has all permissions
        if claims
            .permissions
            .contains(&Permission::Admin.as_str().to_string())
        {
            debug!("Admin access granted for user: {}", claims.sub);
            return Ok(());
        }

        // Check specific permission
        if !claims.permissions.contains(&required_str.to_string()) {
            warn!(
                "Permission denied for user {} - missing: {}",
                claims.sub, required_str
            );
            return Err(anyhow!("Permission denied: {required_str}"));
        }

        debug!(
            "Permission {} granted for user: {}",
            required_str, claims.sub
        );
        Ok(())
    }

    /// Check rate limit for a client
    #[instrument(skip(self))]
    pub async fn check_rate_limit(&self, client_id: &str) -> Result<()> {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Instant::now();

        let rate_limit =
            rate_limits
                .entry(client_id.to_string())
                .or_insert_with(|| RateLimitState {
                    request_count: 0,
                    window_start: now,
                });

        // Check if we need to reset the window
        if now.duration_since(rate_limit.window_start).as_secs()
            >= self.config.auth.rate_limit.window_seconds
        {
            rate_limit.request_count = 0;
            rate_limit.window_start = now;
        }

        // Check if limit exceeded
        if rate_limit.request_count >= self.config.auth.rate_limit.max_requests_per_minute {
            warn!("Rate limit exceeded for client: {}", client_id);
            return Err(anyhow!("Rate limit exceeded"));
        }

        // Increment request count
        rate_limit.request_count += 1;

        debug!(
            "Rate limit check passed for client: {} ({}/{})",
            client_id,
            rate_limit.request_count,
            self.config.auth.rate_limit.max_requests_per_minute
        );

        Ok(())
    }

    /// Get remaining requests for a client
    #[instrument(skip(self))]
    pub async fn get_remaining_requests(&self, client_id: &str) -> u32 {
        let rate_limits = self.rate_limits.read().await;

        if let Some(rate_limit) = rate_limits.get(client_id) {
            let now = Instant::now();

            // If window has expired, return full limit
            if now.duration_since(rate_limit.window_start).as_secs()
                >= self.config.auth.rate_limit.window_seconds
            {
                return self.config.auth.rate_limit.max_requests_per_minute;
            }

            self.config
                .auth
                .rate_limit
                .max_requests_per_minute
                .saturating_sub(rate_limit.request_count)
        } else {
            self.config.auth.rate_limit.max_requests_per_minute
        }
    }

    /// Create a default admin token for development
    pub async fn create_default_admin_token(&self) -> Result<String> {
        self.generate_token("admin", Permission::all()).await
    }

    /// Validate authentication is enabled
    pub fn is_auth_enabled(&self) -> bool {
        self.config.auth.enabled
    }

    /// Get rate limit headers for response
    pub async fn get_rate_limit_headers(&self, client_id: &str) -> HashMap<String, String> {
        let remaining = self.get_remaining_requests(client_id).await;

        let mut headers = HashMap::new();
        headers.insert(
            "X-RateLimit-Limit".to_string(),
            self.config
                .auth
                .rate_limit
                .max_requests_per_minute
                .to_string(),
        );
        headers.insert("X-RateLimit-Remaining".to_string(), remaining.to_string());
        headers.insert(
            "X-RateLimit-Reset".to_string(),
            (Utc::now().timestamp() + self.config.auth.rate_limit.window_seconds as i64)
                .to_string(),
        );

        headers
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, ApiError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        if token.is_empty() {
            return Err(ApiError::Unauthorized);
        }
        Ok(token.to_string())
    } else {
        Err(ApiError::Unauthorized)
    }
}

/// Get client ID from request headers and connection info
fn get_client_id(headers: &HeaderMap) -> String {
    // Try to get client ID from various headers, fallback to "unknown"
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|ips| ips.split(',').next())
        .map(|ip| ip.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|ip| ip.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Authentication middleware state
#[derive(Clone)]
pub struct AuthState {
    pub auth_manager: Arc<AuthManager>,
}

/// JWT authentication middleware for protecting routes
///
/// This middleware:
/// 1. Extracts Bearer tokens from Authorization headers
/// 2. Validates JWT tokens using the AuthManager
/// 3. Checks rate limits per client
/// 4. Adds rate limit headers to responses
/// 5. Returns 401/403 for authentication/authorization failures
#[instrument(skip_all)]
pub async fn jwt_auth_middleware(
    State(auth_state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = request.headers();
    let client_id = get_client_id(headers);

    // Skip authentication if disabled
    if !auth_state.auth_manager.is_auth_enabled() {
        debug!("Authentication disabled, allowing request");
        return Ok(next.run(request).await);
    }

    // Check rate limit first
    auth_state
        .auth_manager
        .check_rate_limit(&client_id)
        .await
        .map_err(|_| ApiError::RateLimitExceeded)?;

    // Extract and verify token
    let token = extract_bearer_token(headers)?;
    let claims = auth_state
        .auth_manager
        .verify_token(&token)
        .await
        .map_err(|_| ApiError::Unauthorized)?;

    // Add claims to request extensions for use in handlers
    request.extensions_mut().insert(claims.clone());

    // Process request
    let mut response = next.run(request).await;

    // Add rate limit headers to response
    let rate_limit_headers = auth_state
        .auth_manager
        .get_rate_limit_headers(&client_id)
        .await;
    let response_headers = response.headers_mut();

    for (key, value) in rate_limit_headers {
        if let Ok(header_value) = HeaderValue::from_str(&value) {
            response_headers.insert(
                axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                header_value,
            );
        }
    }

    info!(
        "Request authenticated successfully for user: {}",
        claims.sub
    );
    Ok(response)
}

/// Require specific permission middleware
///
/// This middleware checks if the authenticated user has the required permission.
/// Should be used after jwt_auth_middleware.
pub fn require_permission(
    required_permission: Permission,
) -> impl Fn(
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ApiError>> + Send>>
       + Clone {
    move |request: Request, next: Next| {
        let required = required_permission;
        Box::pin(async move {
            // Get claims from request extensions (set by jwt_auth_middleware)
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or(ApiError::Unauthorized)?
                .clone();

            // Check permission
            let required_str = required.as_str();

            // Admin has all permissions
            if claims
                .permissions
                .contains(&Permission::Admin.as_str().to_string())
            {
                debug!("Admin access granted for user: {}", claims.sub);
                return Ok(next.run(request).await);
            }

            // Check specific permission
            if !claims.permissions.contains(&required_str.to_string()) {
                warn!(
                    "Permission denied for user {} - missing: {}",
                    claims.sub, required_str
                );
                return Err(ApiError::Forbidden {
                    reason: format!("Permission denied: {required_str}"),
                });
            }

            debug!(
                "Permission {} granted for user: {}",
                required_str, claims.sub
            );
            Ok(next.run(request).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiServerConfig;
    use axum::http::HeaderMap;
    use std::sync::Arc;

    fn create_test_config() -> Arc<ApiServerConfig> {
        let mut config = ApiServerConfig::default();
        config.auth.jwt_secret = "test_secret_key_at_least_32_characters_long".to_string();
        config.auth.token_expiration_hours = 1;
        config.auth.enabled = true;
        Arc::new(config)
    }

    #[tokio::test]
    async fn test_auth_manager_creation() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();
        assert!(auth_manager.is_auth_enabled());
    }

    #[tokio::test]
    async fn test_token_generation_and_verification() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        let permissions = vec![Permission::ResearchRead, Permission::ResourcesRead];
        let token = auth_manager
            .generate_token("test_user", permissions)
            .await
            .unwrap();

        let claims = auth_manager.verify_token(&token).await.unwrap();

        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.iss, "fortitude-api-server");
        assert!(claims
            .permissions
            .contains(&Permission::ResearchRead.as_str().to_string()));
        assert!(claims
            .permissions
            .contains(&Permission::ResourcesRead.as_str().to_string()));
    }

    #[tokio::test]
    async fn test_bearer_token_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, "Bearer valid_token_123".parse().unwrap());

        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "valid_token_123");

        // Test invalid formats
        headers.insert(AUTHORIZATION, "Basic dXNlcjpwYXNz".parse().unwrap());
        assert!(extract_bearer_token(&headers).is_err());

        headers.insert(AUTHORIZATION, "Bearer ".parse().unwrap());
        assert!(extract_bearer_token(&headers).is_err());

        headers.remove(AUTHORIZATION);
        assert!(extract_bearer_token(&headers).is_err());
    }

    #[tokio::test]
    async fn test_client_id_extraction() {
        let mut headers = HeaderMap::new();

        // Test X-Forwarded-For header
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());
        let client_id = get_client_id(&headers);
        assert_eq!(client_id, "192.168.1.1");

        // Test X-Real-IP header
        headers.remove("x-forwarded-for");
        headers.insert("x-real-ip", "192.168.1.2".parse().unwrap());
        let client_id = get_client_id(&headers);
        assert_eq!(client_id, "192.168.1.2");

        // Test fallback to unknown
        headers.clear();
        let client_id = get_client_id(&headers);
        assert_eq!(client_id, "unknown");
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        let permissions = vec![Permission::ResearchRead];
        let token = auth_manager
            .generate_token("test_user", permissions)
            .await
            .unwrap();
        let claims = auth_manager.verify_token(&token).await.unwrap();

        // Should have research permission
        assert!(auth_manager
            .check_permission(&claims, Permission::ResearchRead)
            .await
            .is_ok());

        // Should not have admin permission
        assert!(auth_manager
            .check_permission(&claims, Permission::Admin)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_admin_permissions() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        let permissions = vec![Permission::Admin];
        let token = auth_manager
            .generate_token("admin_user", permissions)
            .await
            .unwrap();
        let claims = auth_manager.verify_token(&token).await.unwrap();

        // Admin should have all permissions
        assert!(auth_manager
            .check_permission(&claims, Permission::ResearchRead)
            .await
            .is_ok());
        assert!(auth_manager
            .check_permission(&claims, Permission::ResourcesRead)
            .await
            .is_ok());
        assert!(auth_manager
            .check_permission(&claims, Permission::ConfigRead)
            .await
            .is_ok());
        assert!(auth_manager
            .check_permission(&claims, Permission::Admin)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        let client_id = "test_client";

        // Make requests up to the limit
        for _ in 0..auth_manager.config.auth.rate_limit.max_requests_per_minute {
            assert!(auth_manager.check_rate_limit(client_id).await.is_ok());
        }

        // Next request should fail
        assert!(auth_manager.check_rate_limit(client_id).await.is_err());
    }

    #[tokio::test]
    async fn test_default_admin_token() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        let token = auth_manager.create_default_admin_token().await.unwrap();
        let claims = auth_manager.verify_token(&token).await.unwrap();

        assert_eq!(claims.sub, "admin");
        assert!(claims
            .permissions
            .contains(&Permission::Admin.as_str().to_string()));
    }

    #[tokio::test]
    async fn test_disabled_auth() {
        let mut config = ApiServerConfig::default();
        config.auth.enabled = false;
        config.auth.jwt_secret = "test_secret_key_at_least_32_characters_long".to_string();
        let config = Arc::new(config);

        let auth_manager = AuthManager::new(config).unwrap();
        assert!(!auth_manager.is_auth_enabled());
    }
}
