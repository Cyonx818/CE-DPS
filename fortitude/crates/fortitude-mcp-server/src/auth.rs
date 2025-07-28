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

// ABOUTME: JWT authentication and authorization system for Fortitude MCP server
// Provides production-ready security with token generation, validation, and permission-based access control
// Includes rate limiting, input validation, and comprehensive security middleware

use crate::config::ServerConfig;
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rmcp::Error as McpError;
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
            Permission::Admin,
            Permission::ReadWrite,
            Permission::System,
        ]
    }
}

/// Implementation of std::str::FromStr for Permission
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
            Permission::ResearchRead | Permission::ResourcesRead | Permission::ConfigRead => {
                ApiPermissionLevel::ReadOnly
            }
            Permission::ReadWrite => ApiPermissionLevel::ReadWrite,
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

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub max_requests_per_minute: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            window_seconds: 60,
        }
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

/// Authentication and authorization manager
pub struct AuthManager {
    /// JWT encoding key
    encoding_key: EncodingKey,
    /// JWT decoding key
    decoding_key: DecodingKey,
    /// JWT validation configuration
    validation: Validation,
    /// Server configuration
    config: Arc<ServerConfig>,
    /// Rate limiting state per client
    rate_limits: Arc<RwLock<HashMap<String, RateLimitState>>>,
    /// Rate limiting configuration
    rate_limit_config: RateLimitConfig,
}

impl AuthManager {
    /// Create new authentication manager
    pub fn new(config: Arc<ServerConfig>) -> Result<Self> {
        let jwt_secret = config.auth.jwt_secret.as_bytes();

        let encoding_key = EncodingKey::from_secret(jwt_secret);
        let decoding_key = DecodingKey::from_secret(jwt_secret);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["fortitude-mcp-server"]);
        validation.validate_exp = true;

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            config: config.clone(),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            rate_limit_config: RateLimitConfig {
                max_requests_per_minute: config.auth.rate_limit.max_requests_per_minute,
                window_seconds: config.auth.rate_limit.window_seconds,
            },
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
            iss: "fortitude-mcp-server".to_string(),
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
            >= self.rate_limit_config.window_seconds
        {
            rate_limit.request_count = 0;
            rate_limit.window_start = now;
        }

        // Check if limit exceeded
        if rate_limit.request_count >= self.rate_limit_config.max_requests_per_minute {
            warn!("Rate limit exceeded for client: {}", client_id);
            return Err(anyhow!("Rate limit exceeded"));
        }

        // Increment request count
        rate_limit.request_count += 1;

        debug!(
            "Rate limit check passed for client: {} ({}/{})",
            client_id, rate_limit.request_count, self.rate_limit_config.max_requests_per_minute
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
                >= self.rate_limit_config.window_seconds
            {
                return self.rate_limit_config.max_requests_per_minute;
            }

            self.rate_limit_config
                .max_requests_per_minute
                .saturating_sub(rate_limit.request_count)
        } else {
            self.rate_limit_config.max_requests_per_minute
        }
    }

    /// Cleanup expired rate limit entries
    #[instrument(skip(self))]
    pub async fn cleanup_expired_rate_limits(&self) {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Instant::now();

        rate_limits.retain(|_, rate_limit| {
            now.duration_since(rate_limit.window_start).as_secs()
                < self.rate_limit_config.window_seconds * 2
        });

        debug!("Cleaned up expired rate limit entries");
    }

    /// Create a default admin token for development
    pub async fn create_default_admin_token(&self) -> Result<String> {
        self.generate_token("admin", Permission::all()).await
    }

    /// Validate authentication is enabled
    pub fn is_auth_enabled(&self) -> bool {
        self.config.auth.enabled
    }

    /// Update rate limit configuration (for testing purposes)
    pub fn set_rate_limit_config(&mut self, config: RateLimitConfig) {
        self.rate_limit_config = config;
    }
}

/// Authentication middleware for MCP operations
pub struct AuthMiddleware {
    auth_manager: Arc<AuthManager>,
}

impl AuthMiddleware {
    /// Create new authentication middleware
    pub fn new(auth_manager: Arc<AuthManager>) -> Self {
        Self { auth_manager }
    }

    /// Authenticate and authorize a request
    #[instrument(skip(self, auth_header))]
    pub async fn authenticate_request(
        &self,
        auth_header: Option<&str>,
        client_id: &str,
        required_permission: Permission,
    ) -> Result<Claims, McpError> {
        // Skip authentication if disabled
        if !self.auth_manager.is_auth_enabled() {
            debug!("Authentication disabled, creating default claims");
            return Ok(Claims {
                sub: "anonymous".to_string(),
                permissions: Permission::all()
                    .iter()
                    .map(|p| p.as_str().to_string())
                    .collect(),
                exp: (Utc::now() + Duration::hours(24)).timestamp(),
                iat: Utc::now().timestamp(),
                iss: "fortitude-mcp-server".to_string(),
            });
        }

        // Check rate limit first
        self.auth_manager
            .check_rate_limit(client_id)
            .await
            .map_err(|e| McpError::invalid_request(format!("Rate limit exceeded: {e}"), None))?;

        // Extract token from Authorization header
        let token = auth_header
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                McpError::invalid_request(
                    "Missing or invalid Authorization header".to_string(),
                    None,
                )
            })?;

        // Verify token
        let claims = self.auth_manager.verify_token(token).await.map_err(|e| {
            McpError::invalid_request(format!("Token verification failed: {e}"), None)
        })?;

        // Check permission
        self.auth_manager
            .check_permission(&claims, required_permission)
            .await
            .map_err(|e| McpError::invalid_request(format!("Permission denied: {e}"), None))?;

        info!(
            "Request authenticated successfully for user: {}",
            claims.sub
        );
        Ok(claims)
    }

    /// Get rate limit headers for response
    pub async fn get_rate_limit_headers(&self, client_id: &str) -> HashMap<String, String> {
        let remaining = self.auth_manager.get_remaining_requests(client_id).await;

        let mut headers = HashMap::new();
        headers.insert(
            "X-RateLimit-Limit".to_string(),
            self.auth_manager
                .rate_limit_config
                .max_requests_per_minute
                .to_string(),
        );
        headers.insert("X-RateLimit-Remaining".to_string(), remaining.to_string());
        headers.insert(
            "X-RateLimit-Reset".to_string(),
            (Utc::now().timestamp() + self.auth_manager.rate_limit_config.window_seconds as i64)
                .to_string(),
        );

        headers
    }
}

/// Input validation helpers
pub mod validation {
    use rmcp::Error as McpError;
    use validator::Validate;

    /// Validate input using the validator crate
    pub fn validate_input<T: Validate>(input: &T) -> Result<(), McpError> {
        input
            .validate()
            .map_err(|e| McpError::invalid_params(format!("Input validation failed: {e}"), None))
    }

    /// Sanitize string input to prevent injection attacks
    pub fn sanitize_string(input: &str) -> String {
        // Remove null bytes and control characters
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect()
    }

    /// Validate and sanitize file path to prevent path traversal
    pub fn validate_file_path(path: &str) -> Result<String, McpError> {
        if path.contains("..") || path.contains("~") || path.starts_with('/') {
            return Err(McpError::invalid_params(
                "Invalid file path".to_string(),
                None,
            ));
        }

        Ok(sanitize_string(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration as TokioDuration};
    use validator::Validate;

    fn create_test_config() -> Arc<ServerConfig> {
        let mut config = ServerConfig::default();
        config.auth.jwt_secret = "test_secret_key_at_least_32_chars_long".to_string();
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
        assert_eq!(claims.iss, "fortitude-mcp-server");
        assert!(claims
            .permissions
            .contains(&Permission::ResearchRead.as_str().to_string()));
        assert!(claims
            .permissions
            .contains(&Permission::ResourcesRead.as_str().to_string()));
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
        let mut auth_manager = AuthManager::new(config).unwrap();
        auth_manager.rate_limit_config.max_requests_per_minute = 2;

        let client_id = "test_client";

        // First two requests should pass
        assert!(auth_manager.check_rate_limit(client_id).await.is_ok());
        assert!(auth_manager.check_rate_limit(client_id).await.is_ok());

        // Third request should fail
        assert!(auth_manager.check_rate_limit(client_id).await.is_err());

        // Check remaining requests
        let remaining = auth_manager.get_remaining_requests(client_id).await;
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let config = create_test_config();
        let mut auth_manager = AuthManager::new(config).unwrap();
        auth_manager.rate_limit_config.max_requests_per_minute = 1;
        auth_manager.rate_limit_config.window_seconds = 1; // Very short window for testing

        let client_id = "test_client";

        // First request should pass
        assert!(auth_manager.check_rate_limit(client_id).await.is_ok());

        // Second request should fail
        assert!(auth_manager.check_rate_limit(client_id).await.is_err());

        // Wait for window to reset
        sleep(TokioDuration::from_secs(2)).await;

        // Should be able to make request again
        assert!(auth_manager.check_rate_limit(client_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_auth_middleware_with_disabled_auth() {
        let mut config = create_test_config();
        Arc::get_mut(&mut config).unwrap().auth.enabled = false;

        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        let middleware = AuthMiddleware::new(auth_manager);

        // Should succeed even without auth header when auth is disabled
        let claims = middleware
            .authenticate_request(None, "test_client", Permission::ResearchRead)
            .await
            .unwrap();
        assert_eq!(claims.sub, "anonymous");
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        let config = create_test_config();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        let middleware = AuthMiddleware::new(auth_manager.clone());

        // Generate token
        let token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        let auth_header = format!("Bearer {token}");

        // Should succeed with valid token
        let claims = middleware
            .authenticate_request(Some(&auth_header), "test_client", Permission::ResearchRead)
            .await
            .unwrap();
        assert_eq!(claims.sub, "test_user");
    }

    #[tokio::test]
    async fn test_auth_middleware_permission_denied() {
        let config = create_test_config();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        let middleware = AuthMiddleware::new(auth_manager.clone());

        // Generate token with limited permissions
        let token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        let auth_header = format!("Bearer {token}");

        // Should fail when requesting admin permission
        let result = middleware
            .authenticate_request(Some(&auth_header), "test_client", Permission::Admin)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_input_validation() {
        use validation::*;

        // Test sanitization
        let dirty_input = "test\x00string\x01with\x02control\x03chars";
        let clean_input = sanitize_string(dirty_input);
        assert!(!clean_input.contains('\x00'));
        assert!(!clean_input.contains('\x01'));

        // Test file path validation
        assert!(validate_file_path("valid/path.txt").is_ok());
        assert!(validate_file_path("../../../etc/passwd").is_err());
        assert!(validate_file_path("~/secret").is_err());
        assert!(validate_file_path("/absolute/path").is_err());
    }

    #[tokio::test]
    async fn test_permission_string_conversion() {
        assert_eq!(Permission::ResearchRead.as_str(), "fortitude:research:read");
        assert_eq!(
            Permission::ResourcesRead.as_str(),
            "fortitude:resources:read"
        );
        assert_eq!(Permission::ConfigRead.as_str(), "fortitude:config:read");
        assert_eq!(Permission::Admin.as_str(), "fortitude:admin");

        assert_eq!(
            Permission::parse("fortitude:research:read"),
            Some(Permission::ResearchRead)
        );
        assert_eq!(
            Permission::parse("fortitude:resources:read"),
            Some(Permission::ResourcesRead)
        );
        assert_eq!(
            Permission::parse("fortitude:config:read"),
            Some(Permission::ConfigRead)
        );
        assert_eq!(
            Permission::parse("fortitude:admin"),
            Some(Permission::Admin)
        );
        assert_eq!(Permission::parse("invalid:permission"), None);
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
    async fn test_rate_limit_headers() {
        let config = create_test_config();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        let middleware = AuthMiddleware::new(auth_manager);

        let headers = middleware.get_rate_limit_headers("test_client").await;

        assert!(headers.contains_key("X-RateLimit-Limit"));
        assert!(headers.contains_key("X-RateLimit-Remaining"));
        assert!(headers.contains_key("X-RateLimit-Reset"));
    }

    #[tokio::test]
    async fn test_expired_token_rejection() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Manually create an expired token
        let expired_claims = Claims {
            sub: "test_user".to_string(),
            permissions: vec![Permission::ResearchRead.as_str().to_string()],
            exp: Utc::now().timestamp() - 3600, // Expired 1 hour ago
            iat: Utc::now().timestamp() - 7200, // Issued 2 hours ago
            iss: "fortitude-mcp-server".to_string(),
        };

        let token = encode(
            &Header::default(),
            &expired_claims,
            &auth_manager.encoding_key,
        )
        .unwrap();

        // Should fail verification
        let result = auth_manager.verify_token(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_validation_edge_cases() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Test malformed tokens
        let malformed_tokens = vec![
            "not.a.jwt.token",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.header",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.invalid_signature",
            "",
            "...",
            "a.b.c.d.e", // Too many parts
            "onlyonepart",
        ];

        for token in malformed_tokens {
            let result = auth_manager.verify_token(token).await;
            assert!(result.is_err(), "Should reject malformed token: {token}");
        }
    }

    #[tokio::test]
    async fn test_permission_string_conversion_comprehensive() {
        // Test all permission types
        let permissions = Permission::all();

        for permission in permissions {
            let permission_str = permission.as_str();
            let parsed = Permission::parse(permission_str);
            assert_eq!(
                parsed,
                Some(permission),
                "Permission conversion should be bidirectional"
            );
        }

        // Test invalid permission strings
        let invalid_permissions = vec![
            "invalid:permission",
            "fortitude:invalid",
            "fortitude:",
            ":read",
            "",
            "FORTITUDE:RESEARCH:READ", // Case sensitivity
            "fortitude research read", // Wrong format
        ];

        for invalid in invalid_permissions {
            let result = Permission::parse(invalid);
            assert_eq!(result, None, "Should reject invalid permission: {invalid}");
        }
    }

    #[tokio::test]
    async fn test_rate_limit_cleanup() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Create some rate limit entries
        let _ = auth_manager.check_rate_limit("client1").await;
        let _ = auth_manager.check_rate_limit("client2").await;
        let _ = auth_manager.check_rate_limit("client3").await;

        // Cleanup should not crash
        auth_manager.cleanup_expired_rate_limits().await;

        // Should still work after cleanup
        let result = auth_manager.check_rate_limit("client1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_rate_limiting() {
        let config = create_test_config();
        let mut auth_manager = AuthManager::new(config).unwrap();
        auth_manager.rate_limit_config.max_requests_per_minute = 10;

        let auth_manager = Arc::new(auth_manager);

        // Create multiple concurrent requests
        let mut handles = Vec::new();
        for i in 0..20 {
            let auth_manager = auth_manager.clone();
            let client_id = format!("client_{i}");
            let handle =
                tokio::spawn(async move { auth_manager.check_rate_limit(&client_id).await });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;

        // All should succeed because they're different clients
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_per_client_isolation() {
        let config = create_test_config();
        let mut auth_manager = AuthManager::new(config).unwrap();
        auth_manager.rate_limit_config.max_requests_per_minute = 2;

        // Exhaust limit for client1
        assert!(auth_manager.check_rate_limit("client1").await.is_ok());
        assert!(auth_manager.check_rate_limit("client1").await.is_ok());
        assert!(auth_manager.check_rate_limit("client1").await.is_err());

        // Client2 should still work
        assert!(auth_manager.check_rate_limit("client2").await.is_ok());
        assert!(auth_manager.check_rate_limit("client2").await.is_ok());
        assert!(auth_manager.check_rate_limit("client2").await.is_err());

        // Client3 should also work
        assert!(auth_manager.check_rate_limit("client3").await.is_ok());
    }

    #[tokio::test]
    async fn test_auth_manager_with_disabled_auth() {
        let mut config = ServerConfig::default();
        config.auth.enabled = false;
        config.auth.jwt_secret = "test_secret_key_at_least_32_chars_long".to_string();
        let config = Arc::new(config);

        let auth_manager = AuthManager::new(config).unwrap();
        assert!(!auth_manager.is_auth_enabled());

        // Should still be able to generate tokens
        let token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_jwt_issuer_validation() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Create token with wrong issuer
        let wrong_issuer_claims = Claims {
            sub: "test_user".to_string(),
            permissions: vec![Permission::ResearchRead.as_str().to_string()],
            exp: (Utc::now() + Duration::hours(24)).timestamp(),
            iat: Utc::now().timestamp(),
            iss: "wrong-issuer".to_string(),
        };

        let wrong_issuer_token = encode(
            &Header::default(),
            &wrong_issuer_claims,
            &auth_manager.encoding_key,
        )
        .unwrap();

        // Should fail verification due to wrong issuer
        let result = auth_manager.verify_token(&wrong_issuer_token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_generation_with_empty_permissions() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Should be able to generate token with no permissions
        let token = auth_manager
            .generate_token("test_user", vec![])
            .await
            .unwrap();
        let claims = auth_manager.verify_token(&token).await.unwrap();

        assert_eq!(claims.sub, "test_user");
        assert!(claims.permissions.is_empty());

        // Should fail permission checks
        assert!(auth_manager
            .check_permission(&claims, Permission::ResearchRead)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_remaining_requests() {
        let config = create_test_config();
        let mut auth_manager = AuthManager::new(config).unwrap();
        auth_manager.rate_limit_config.max_requests_per_minute = 5;

        let client_id = "test_client";

        // Initial remaining should be max
        let remaining = auth_manager.get_remaining_requests(client_id).await;
        assert_eq!(remaining, 5);

        // Make some requests
        assert!(auth_manager.check_rate_limit(client_id).await.is_ok());
        assert!(auth_manager.check_rate_limit(client_id).await.is_ok());

        // Remaining should decrease
        let remaining = auth_manager.get_remaining_requests(client_id).await;
        assert_eq!(remaining, 3);
    }

    #[tokio::test]
    async fn test_validation_helpers() {
        use validation::*;
        use validator::Validate;

        // Test input validation
        #[derive(Validate)]
        struct TestInput {
            #[validate(length(min = 1, max = 10))]
            field: String,
        }

        let valid_input = TestInput {
            field: "valid".to_string(),
        };
        assert!(validate_input(&valid_input).is_ok());

        let invalid_input = TestInput {
            field: "".to_string(),
        };
        assert!(validate_input(&invalid_input).is_err());

        // Test string sanitization
        let dirty_string = "test\x00string\x01with\x02control\x03chars";
        let clean_string = sanitize_string(dirty_string);
        assert!(!clean_string.contains('\x00'));
        assert!(!clean_string.contains('\x01'));
        assert!(!clean_string.contains('\x02'));
        assert!(!clean_string.contains('\x03'));

        // Test legitimate characters are preserved
        let legitimate_string = "normal\nstring\twith\rvalid\nchars";
        let clean_legitimate = sanitize_string(legitimate_string);
        assert!(clean_legitimate.contains('\n'));
        assert!(clean_legitimate.contains('\t'));
        assert!(clean_legitimate.contains('\r'));

        // Test file path validation
        assert!(validate_file_path("valid/path.txt").is_ok());
        assert!(validate_file_path("../invalid/path").is_err());
        assert!(validate_file_path("~/home/path").is_err());
        assert!(validate_file_path("/absolute/path").is_err());
    }

    #[tokio::test]
    async fn test_auth_middleware_comprehensive() {
        let config = create_test_config();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        let middleware = AuthMiddleware::new(auth_manager.clone());

        // Test successful authentication
        let token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        let auth_header = format!("Bearer {token}");

        let result = middleware
            .authenticate_request(Some(&auth_header), "test_client", Permission::ResearchRead)
            .await;
        assert!(result.is_ok());

        // Test missing authorization header
        let result = middleware
            .authenticate_request(None, "test_client", Permission::ResearchRead)
            .await;
        assert!(result.is_err());

        // Test malformed authorization header
        let malformed_headers = vec![
            "invalid_token",
            "Bearer",
            "Basic token",
            "Bearer ",
            "token_without_bearer",
        ];

        for header in malformed_headers {
            let result = middleware
                .authenticate_request(Some(header), "test_client", Permission::ResearchRead)
                .await;
            assert!(result.is_err(), "Should reject malformed header: {header}");
        }
    }

    #[tokio::test]
    async fn test_auth_middleware_rate_limit_headers() {
        let config = create_test_config();
        let mut auth_manager = AuthManager::new(config).unwrap();
        auth_manager.rate_limit_config.max_requests_per_minute = 10;
        auth_manager.rate_limit_config.window_seconds = 60;

        let auth_manager = Arc::new(auth_manager);
        let middleware = AuthMiddleware::new(auth_manager.clone());

        // Generate token and make some requests
        let token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        let auth_header = format!("Bearer {token}");

        // Make a few requests
        for _ in 0..3 {
            let _ = middleware
                .authenticate_request(
                    Some(&auth_header),
                    "header_test_client",
                    Permission::ResearchRead,
                )
                .await;
        }

        // Check rate limit headers
        let headers = middleware
            .get_rate_limit_headers("header_test_client")
            .await;

        assert!(headers.contains_key("X-RateLimit-Limit"));
        assert!(headers.contains_key("X-RateLimit-Remaining"));
        assert!(headers.contains_key("X-RateLimit-Reset"));

        assert_eq!(headers.get("X-RateLimit-Limit").unwrap(), "10");
        assert_eq!(headers.get("X-RateLimit-Remaining").unwrap(), "7"); // 10 - 3 = 7

        // Reset time should be in the future
        let reset_time: i64 = headers.get("X-RateLimit-Reset").unwrap().parse().unwrap();
        assert!(reset_time > Utc::now().timestamp());
    }

    #[tokio::test]
    async fn test_permission_boundary_conditions() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Test with all permissions
        let all_permissions = Permission::all();
        let token = auth_manager
            .generate_token("test_user", all_permissions.clone())
            .await
            .unwrap();
        let claims = auth_manager.verify_token(&token).await.unwrap();

        // Should have all permissions
        for permission in all_permissions {
            assert!(auth_manager
                .check_permission(&claims, permission)
                .await
                .is_ok());
        }

        // Test with duplicate permissions
        let duplicate_permissions = vec![Permission::ResearchRead, Permission::ResearchRead];
        let token = auth_manager
            .generate_token("test_user", duplicate_permissions)
            .await
            .unwrap();
        let claims = auth_manager.verify_token(&token).await.unwrap();

        // Should still work with duplicates
        assert!(auth_manager
            .check_permission(&claims, Permission::ResearchRead)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_token_operations() {
        let config = create_test_config();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());

        // Generate multiple tokens concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let auth_manager = auth_manager.clone();
            let handle = tokio::spawn(async move {
                let user_id = format!("user_{i}");
                auth_manager
                    .generate_token(&user_id, vec![Permission::ResearchRead])
                    .await
            });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;

        // All token generations should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Verify tokens concurrently
        let tokens: Vec<String> = (0..10)
            .map(|i| {
                let user_id = format!("user_{i}");
                futures::executor::block_on(
                    auth_manager.generate_token(&user_id, vec![Permission::ResearchRead]),
                )
                .unwrap()
            })
            .collect();

        let mut verify_handles = Vec::new();
        for token in tokens {
            let auth_manager = auth_manager.clone();
            let handle = tokio::spawn(async move { auth_manager.verify_token(&token).await });
            verify_handles.push(handle);
        }

        let verify_results = futures::future::join_all(verify_handles).await;

        // All verifications should succeed
        for result in verify_results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    #[tokio::test]
    async fn test_auth_error_handling() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Test with invalid JWT secret length
        let mut invalid_config = ServerConfig::default();
        invalid_config.auth.jwt_secret = "short".to_string(); // Too short
        invalid_config.auth.enabled = true;

        // Should fail validation in normal config validation, but AuthManager might handle it
        let _result = AuthManager::new(Arc::new(invalid_config));
        // This might succeed or fail depending on implementation

        // Test token verification with tampered token
        let valid_token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        let tampered_token = format!("{valid_token}tampered");

        let result = auth_manager.verify_token(&tampered_token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_config_validation() {
        let config = create_test_config();
        let mut auth_manager = AuthManager::new(config).unwrap();

        // Test setting invalid rate limit config
        let invalid_config = RateLimitConfig {
            max_requests_per_minute: 0, // Invalid
            window_seconds: 60,
        };

        auth_manager.set_rate_limit_config(invalid_config);

        // Should handle gracefully (might allow 0 or treat as unlimited)
        let _result = auth_manager.check_rate_limit("test_client").await;
        // Implementation dependent behavior
    }

    #[tokio::test]
    async fn test_token_expiration_edge_cases() {
        let config = create_test_config();
        let auth_manager = AuthManager::new(config).unwrap();

        // Test token at exact expiration time
        let now = Utc::now();
        let exact_exp_claims = Claims {
            sub: "test_user".to_string(),
            permissions: vec![Permission::ResearchRead.as_str().to_string()],
            exp: now.timestamp() - 1, // Expires 1 second ago to ensure it's definitely expired
            iat: (now - Duration::hours(1)).timestamp(),
            iss: "fortitude-mcp-server".to_string(),
        };

        let token = encode(
            &Header::default(),
            &exact_exp_claims,
            &auth_manager.encoding_key,
        )
        .unwrap();

        // Should fail (token is expired)
        let result = auth_manager.verify_token(&token).await;
        assert!(result.is_err());

        // Test token expiring in the future
        let future_exp_claims = Claims {
            sub: "test_user".to_string(),
            permissions: vec![Permission::ResearchRead.as_str().to_string()],
            exp: (now + Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
            iss: "fortitude-mcp-server".to_string(),
        };

        let future_token = encode(
            &Header::default(),
            &future_exp_claims,
            &auth_manager.encoding_key,
        )
        .unwrap();

        // Should succeed
        let result = auth_manager.verify_token(&future_token).await;
        assert!(result.is_ok());
    }
}
