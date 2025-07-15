//! Fortitude API Rust Client Library
//! 
//! A comprehensive Rust client for the Fortitude API with support for
//! async operations, error handling, retries, and type safety.

use anyhow::Context;
use chrono::{DateTime, Utc};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Custom error types for the Fortitude API client
#[derive(Debug, Error)]
pub enum FortitudeError {
    #[error("API error {status_code}: {code} - {message}")]
    ApiError {
        status_code: u16,
        code: String,
        message: String,
        request_id: Option<String>,
        details: Option<String>,
    },
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Timeout error")]
    TimeoutError,
    
    #[error("Rate limit exceeded")]
    RateLimitError,
}

/// API response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub data: T,
}

/// Error response from API
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    pub details: Option<String>,
    pub request_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub path: Option<String>,
}

/// Health status response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub last_check: DateTime<Utc>,
    pub details: Option<String>,
}

/// Research request
#[derive(Debug, Serialize)]
pub struct ResearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience_context: Option<AudienceContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_context: Option<DomainContext>,
}

#[derive(Debug, Serialize)]
pub struct AudienceContext {
    pub level: String,
    pub domain: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct DomainContext {
    pub technology: String,
    pub architecture: String,
}

/// Research response
#[derive(Debug, Deserialize)]
pub struct ResearchResponse {
    pub results: Vec<ResearchResult>,
    pub total_count: u32,
    pub processing_time_ms: u32,
}

#[derive(Debug, Deserialize)]
pub struct ResearchResult {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub relevance_score: f64,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Research list response
#[derive(Debug, Deserialize)]
pub struct ResearchListResponse {
    pub results: Vec<ResearchResult>,
    pub total_count: u32,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Deserialize)]
pub struct PaginationInfo {
    pub limit: u32,
    pub offset: u32,
    pub total_count: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

/// Classification request
#[derive(Debug, Serialize)]
pub struct ClassificationRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_preferences: Option<ClassificationContextPreferences>,
}

#[derive(Debug, Serialize)]
pub struct ClassificationContextPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect_urgency: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect_audience: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect_domain: Option<bool>,
}

/// Classification response
#[derive(Debug, Deserialize)]
pub struct ClassificationResponse {
    pub classifications: Vec<Classification>,
    pub confidence: f64,
    pub processing_time_ms: u32,
}

#[derive(Debug, Deserialize)]
pub struct Classification {
    pub category: String,
    pub confidence: f64,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Cache statistics response
#[derive(Debug, Deserialize)]
pub struct CacheStatsResponse {
    pub total_entries: u32,
    pub hit_rate: f64,
    pub storage_efficiency: StorageEfficiency,
    pub performance_metrics: CachePerformanceMetrics,
    pub by_research_type: HashMap<String, CacheTypeStats>,
}

#[derive(Debug, Deserialize)]
pub struct StorageEfficiency {
    pub compression_ratio: f64,
    pub deduplication_savings: f64,
    pub total_size_bytes: u64,
    pub compressed_size_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct CachePerformanceMetrics {
    pub average_read_time_ms: f64,
    pub average_write_time_ms: f64,
    pub recent_operations: RecentOperations,
}

#[derive(Debug, Deserialize)]
pub struct RecentOperations {
    pub hits: u32,
    pub misses: u32,
    pub writes: u32,
    pub time_window_minutes: u32,
}

#[derive(Debug, Deserialize)]
pub struct CacheTypeStats {
    pub entries: u32,
    pub size_bytes: u64,
    pub hit_rate: f64,
    pub hits: u32,
    pub misses: u32,
    pub average_quality: f64,
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: env::var("FORTITUDE_API_KEY").unwrap_or_default(),
            base_url: env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            timeout: Duration::from_secs(
                env::var("FORTITUDE_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30)
            ),
            max_retries: env::var("FORTITUDE_MAX_RETRIES")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            user_agent: "Fortitude-Rust-Client/1.0.0".to_string(),
        }
    }
}

/// Fortitude API client
#[derive(Clone)]
pub struct FortitudeClient {
    client: Arc<Client>,
    config: ClientConfig,
}

impl FortitudeClient {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self, FortitudeError> {
        let config = ClientConfig::default();
        Self::with_config(config)
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self, FortitudeError> {
        if config.api_key.is_empty() {
            return Err(FortitudeError::ConfigError(
                "API key is required. Set FORTITUDE_API_KEY environment variable.".to_string()
            ));
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-API-Key",
            reqwest::header::HeaderValue::from_str(&config.api_key)
                .map_err(|e| FortitudeError::ConfigError(format!("Invalid API key: {}", e)))?
        );
        headers.insert(
            "User-Agent",
            reqwest::header::HeaderValue::from_str(&config.user_agent)
                .map_err(|e| FortitudeError::ConfigError(format!("Invalid user agent: {}", e)))?
        );

        let client = ClientBuilder::new()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()
            .map_err(FortitudeError::HttpError)?;

        info!("Initialized Fortitude client for {}", config.base_url);

        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }

    /// Create a client builder for custom configuration
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Make an HTTP request with retries and error handling
    async fn make_request<T, R>(&self, method: reqwest::Method, endpoint: &str, body: Option<&T>) -> Result<ApiResponse<R>, FortitudeError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.config.base_url, endpoint);
        
        for attempt in 0..=self.config.max_retries {
            let mut request = self.client.request(method.clone(), &url);
            
            if let Some(data) = body {
                request = request.json(data);
            }

            debug!("Making request: {} {} (attempt {})", method, endpoint, attempt + 1);
            
            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    if status.is_success() {
                        let result: ApiResponse<R> = response.json().await?;
                        debug!("Request successful: {} {}", method, endpoint);
                        return Ok(result);
                    }
                    
                    // Handle retryable errors
                    if (status == 429 || status.is_server_error()) && attempt < self.config.max_retries {
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt)));
                        warn!("Request failed with {}, retrying in {:?}...", status, delay);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    
                    // Handle client errors and final server errors
                    let error_response: Result<ErrorResponse, _> = response.json().await;
                    
                    match error_response {
                        Ok(err) => {
                            error!("API error: {} - {}", err.error_code, err.message);
                            return Err(FortitudeError::ApiError {
                                status_code: status.as_u16(),
                                code: err.error_code,
                                message: err.message,
                                request_id: err.request_id,
                                details: err.details,
                            });
                        }
                        Err(_) => {
                            let text = response.text().await.unwrap_or_default();
                            return Err(FortitudeError::ApiError {
                                status_code: status.as_u16(),
                                code: "UNKNOWN_ERROR".to_string(),
                                message: format!("HTTP {}: {}", status, text),
                                request_id: None,
                                details: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt)));
                        warn!("Request error: {}, retrying in {:?}...", e, delay);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    
                    error!("Request failed after {} attempts: {}", self.config.max_retries + 1, e);
                    return Err(FortitudeError::HttpError(e));
                }
            }
        }
        
        Err(FortitudeError::ConfigError("Max retries exceeded".to_string()))
    }

    // Health endpoints

    /// Get public health status
    pub async fn get_health(&self) -> Result<HealthResponse, FortitudeError> {
        let response: ApiResponse<HealthResponse> = self.make_request(reqwest::Method::GET, "/health", None::<&()>).await?;
        Ok(response.data)
    }

    /// Get detailed health status (requires authentication)
    pub async fn get_protected_health(&self) -> Result<HealthResponse, FortitudeError> {
        let response: ApiResponse<HealthResponse> = self.make_request(reqwest::Method::GET, "/api/v1/health/protected", None::<&()>).await?;
        Ok(response.data)
    }

    // Research endpoints

    /// Perform a research query
    pub async fn research(&self, query: &str) -> Result<ResearchResponse, FortitudeError> {
        let request = ResearchRequest {
            query: query.to_string(),
            context: None,
            priority: Some("medium".to_string()),
            audience_context: None,
            domain_context: None,
        };
        
        let response: ApiResponse<ResearchResponse> = self.make_request(reqwest::Method::POST, "/api/v1/research", Some(&request)).await?;
        Ok(response.data)
    }

    /// Perform a detailed research query with context
    pub async fn research_detailed(&self, request: ResearchRequest) -> Result<ResearchResponse, FortitudeError> {
        let response: ApiResponse<ResearchResponse> = self.make_request(reqwest::Method::POST, "/api/v1/research", Some(&request)).await?;
        Ok(response.data)
    }

    /// Get a specific research result by ID
    pub async fn get_research_result(&self, research_id: &Uuid) -> Result<ResearchResult, FortitudeError> {
        let endpoint = format!("/api/v1/research/{}", research_id);
        let response: ApiResponse<ResearchResult> = self.make_request(reqwest::Method::GET, &endpoint, None::<&()>).await?;
        Ok(response.data)
    }

    /// List research results with pagination
    pub async fn list_research_results(&self, limit: Option<u32>, offset: Option<u32>, query: Option<&str>) -> Result<ResearchListResponse, FortitudeError> {
        let mut endpoint = "/api/v1/research".to_string();
        let mut params = Vec::new();
        
        if let Some(limit) = limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = offset {
            params.push(format!("offset={}", offset));
        }
        if let Some(query) = query {
            params.push(format!("query={}", urlencoding::encode(query)));
        }
        
        if !params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&params.join("&"));
        }
        
        let response: ApiResponse<ResearchListResponse> = self.make_request(reqwest::Method::GET, &endpoint, None::<&()>).await?;
        Ok(response.data)
    }

    // Classification endpoints

    /// Classify content
    pub async fn classify(&self, content: &str) -> Result<ClassificationResponse, FortitudeError> {
        let request = ClassificationRequest {
            content: content.to_string(),
            categories: None,
            context_preferences: None,
        };
        
        let response: ApiResponse<ClassificationResponse> = self.make_request(reqwest::Method::POST, "/api/v1/classify", Some(&request)).await?;
        Ok(response.data)
    }

    /// Classify content with detailed options
    pub async fn classify_detailed(&self, request: ClassificationRequest) -> Result<ClassificationResponse, FortitudeError> {
        let response: ApiResponse<ClassificationResponse> = self.make_request(reqwest::Method::POST, "/api/v1/classify", Some(&request)).await?;
        Ok(response.data)
    }

    // Cache endpoints

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStatsResponse, FortitudeError> {
        let response: ApiResponse<CacheStatsResponse> = self.make_request(reqwest::Method::GET, "/api/v1/cache/stats", None::<&()>).await?;
        Ok(response.data)
    }

    /// Test API connectivity
    pub async fn test_connection(&self) -> Result<bool, FortitudeError> {
        match self.get_health().await {
            Ok(_) => {
                info!("API connection test successful");
                Ok(true)
            }
            Err(e) => {
                error!("API connection test failed: {}", e);
                Err(e)
            }
        }
    }
}