// ABOUTME: MCP resource providers for Fortitude reference library and system resources
// Exposes reference library files, cache statistics, and configuration state via MCP protocol
// Implements proper URI conventions and security for read-only access to docs/ directory

use crate::config::ServerConfig;
use anyhow::Result;
use rmcp::model::{RawResource, Resource, ResourceContents};
use rmcp::Error as McpError;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, instrument};

/// Resource provider for Fortitude MCP server
pub struct ResourceProvider {
    config: Arc<ServerConfig>,
    docs_base_path: PathBuf,
}

impl ResourceProvider {
    /// Create a new resource provider
    pub fn new(config: Arc<ServerConfig>) -> Self {
        // Determine docs base path relative to project root
        let docs_base_path = Self::find_docs_base_path();

        Self {
            config,
            docs_base_path,
        }
    }

    /// Find the docs base path by looking for CE-DPS reference/methodology or legacy docs/reference_library directory
    fn find_docs_base_path() -> PathBuf {
        // Start from current directory and work up to find CE-DPS methodology/reference
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Try CE-DPS locations first, then legacy fortitude locations
        let possible_paths = vec![
            // CE-DPS reference directory (preferred)
            current_path.join("../../../reference"),
            current_path.join("../../reference"), 
            current_path.join("../reference"),
            current_path.join("reference"),
            // CE-DPS methodology directory
            current_path.join("../../../methodology"),
            current_path.join("../../methodology"),
            current_path.join("../methodology"),
            current_path.join("methodology"),
            // Legacy fortitude docs (fallback)
            current_path.join("docs"),
            current_path.join("../docs"),
            current_path.join("../../docs"),
            current_path.join("../../../docs"),
        ];

        for path in possible_paths {
            // Check for CE-DPS reference structure
            if path.join("llm-documentation-quick-reference.md").exists() {
                return path;
            }
            // Check for CE-DPS methodology structure  
            if path.join("ai-implementation").exists() {
                return path;
            }
            // Check for legacy fortitude reference_library
            if path.join("reference_library").exists() {
                return path;
            }
        }

        // Fallback to CE-DPS reference directory
        PathBuf::from("reference")
    }

    /// List all available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        let mut resources = Vec::new();

        // Add reference library files
        self.add_reference_library_resources(&mut resources).await?;

        // Add cache statistics resource
        resources.push(Resource {
            raw: RawResource {
                uri: "mcp://fortitude/cache/statistics".to_string(),
                name: "Cache Statistics".to_string(),
                description: Some("Cache performance metrics and statistics".to_string()),
                mime_type: Some("application/json".to_string()),
                size: None,
            },
            annotations: None,
        });

        // Add configuration state resource
        resources.push(Resource {
            raw: RawResource {
                uri: "mcp://fortitude/config/current".to_string(),
                name: "Current Configuration".to_string(),
                description: Some("Current server configuration (sanitized)".to_string()),
                mime_type: Some("application/json".to_string()),
                size: None,
            },
            annotations: None,
        });

        // Add system metrics resource
        resources.push(Resource {
            raw: RawResource {
                uri: "mcp://fortitude/system/metrics".to_string(),
                name: "System Metrics".to_string(),
                description: Some("System performance and health metrics".to_string()),
                mime_type: Some("application/json".to_string()),
                size: None,
            },
            annotations: None,
        });

        Ok(resources)
    }

    /// Add reference library files to the resource list
    async fn add_reference_library_resources(
        &self,
        resources: &mut Vec<Resource>,
    ) -> Result<(), McpError> {
        // Try CE-DPS structure first, then legacy fortitude structure
        let possible_library_paths = vec![
            // CE-DPS reference directory
            self.docs_base_path.clone(),
            // CE-DPS methodology directory  
            self.docs_base_path.clone(),
            // Legacy fortitude reference_library
            self.docs_base_path.join("reference_library"),
        ];

        for library_path in possible_library_paths {
            if library_path.exists() {
                debug!(
                    "Found reference library path: {}",
                    library_path.display()
                );
                
                self.scan_directory_for_resources(
                    &library_path,
                    &library_path,
                    resources,
                )
                .await?;
                
                return Ok(());
            }
        }

        debug!(
            "No reference library path found, base path: {}",
            self.docs_base_path.display()
        );
        Ok(())
    }

    /// Recursively scan directory for files and add as resources
    #[instrument(skip(self, resources))]
    async fn scan_directory_for_resources(
        &self,
        dir_path: &Path,
        base_path: &Path,
        resources: &mut Vec<Resource>,
    ) -> Result<(), McpError> {
        let mut entries = fs::read_dir(dir_path).await.map_err(|e| {
            McpError::internal_error(format!("Failed to read directory: {e}"), None)
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            McpError::internal_error(format!("Failed to read directory entry: {e}"), None)
        })? {
            let path = entry.path();
            let metadata = entry.metadata().await.map_err(|e| {
                McpError::internal_error(format!("Failed to read metadata: {e}"), None)
            })?;

            if metadata.is_file() {
                // Create resource URI
                let relative_path = path
                    .strip_prefix(base_path)
                    .map_err(|e| McpError::internal_error(format!("Invalid path: {e}"), None))?;

                let uri = format!(
                    "mcp://fortitude/docs/reference_library/{}",
                    relative_path.to_string_lossy().replace('\\', "/")
                );

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let mime_type = Self::detect_mime_type(&path);

                resources.push(Resource {
                    raw: RawResource {
                        uri,
                        name: file_name.to_string(),
                        description: Some(format!(
                            "Reference library file: {}",
                            relative_path.display()
                        )),
                        mime_type: Some(mime_type),
                        size: Some(metadata.len().try_into().unwrap_or(u32::MAX)),
                    },
                    annotations: None,
                });
            } else if metadata.is_dir() {
                // Recursively scan subdirectories
                Box::pin(self.scan_directory_for_resources(&path, base_path, resources)).await?;
            }
        }

        Ok(())
    }

    /// Detect MIME type based on file extension
    fn detect_mime_type(path: &Path) -> String {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("md") => "text/markdown".to_string(),
            Some("txt") => "text/plain".to_string(),
            Some("json") => "application/json".to_string(),
            Some("yaml") | Some("yml") => "application/yaml".to_string(),
            Some("rs") => "text/x-rust".to_string(),
            Some("toml") => "application/toml".to_string(),
            _ => "text/plain".to_string(),
        }
    }

    /// Read resource content by URI
    #[instrument(skip(self))]
    pub async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContents>, McpError> {
        debug!("Reading resource: {}", uri);

        // Parse URI and validate
        let parsed_uri = self.parse_and_validate_uri(uri)?;

        match parsed_uri.resource_type.as_str() {
            "docs" => self.read_reference_library_file(&parsed_uri).await,
            "cache" => self.read_cache_resource(&parsed_uri).await,
            "config" => self.read_config_resource(&parsed_uri).await,
            "system" => self.read_system_resource(&parsed_uri).await,
            _ => Err(McpError::invalid_params(
                format!("Unknown resource type: {}", parsed_uri.resource_type),
                None,
            )),
        }
    }

    /// Parse and validate MCP URI
    fn parse_and_validate_uri(&self, uri: &str) -> Result<ParsedUri, McpError> {
        if !uri.starts_with("mcp://fortitude/") {
            return Err(McpError::invalid_params(
                format!("Invalid URI scheme: {uri}"),
                None,
            ));
        }

        let path_part = &uri[16..]; // Remove "mcp://fortitude/"
        let parts: Vec<&str> = path_part.split('/').collect();

        if parts.is_empty() {
            return Err(McpError::invalid_params(
                "Empty resource path".to_string(),
                None,
            ));
        }

        let resource_type = parts[0].to_string();
        let resource_path = if parts.len() > 1 {
            parts[1..].join("/")
        } else {
            String::new()
        };

        // Validate resource type
        match resource_type.as_str() {
            "docs" | "cache" | "config" | "system" => {}
            _ => {
                return Err(McpError::invalid_params(
                    format!("Invalid resource type: {resource_type}"),
                    None,
                ))
            }
        }

        // Path traversal protection
        if resource_path.contains("..") || resource_path.contains("~") {
            return Err(McpError::invalid_params(
                "Path traversal not allowed".to_string(),
                None,
            ));
        }

        Ok(ParsedUri {
            resource_type,
            resource_path,
        })
    }

    /// Read reference library file
    async fn read_reference_library_file(
        &self,
        parsed_uri: &ParsedUri,
    ) -> Result<Vec<ResourceContents>, McpError> {
        if !parsed_uri.resource_path.starts_with("reference_library/") {
            return Err(McpError::invalid_params(
                "Access denied: only reference_library files allowed".to_string(),
                None,
            ));
        }

        let file_path = self.docs_base_path.join(&parsed_uri.resource_path);

        // Additional security check: ensure file is within docs directory
        let canonical_docs = self.docs_base_path.canonicalize().map_err(|e| {
            McpError::internal_error(format!("Failed to canonicalize docs path: {e}"), None)
        })?;

        let canonical_file = file_path
            .canonicalize()
            .map_err(|_| McpError::invalid_params("File not found".to_string(), None))?;

        if !canonical_file.starts_with(&canonical_docs) {
            return Err(McpError::invalid_params(
                "Access denied: file outside docs directory".to_string(),
                None,
            ));
        }

        // Read file content
        let content = fs::read_to_string(&file_path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {e}"), None))?;

        let uri = format!("mcp://fortitude/docs/{}", parsed_uri.resource_path);
        Ok(vec![ResourceContents::text(content, uri)])
    }

    /// Read cache statistics resource
    async fn read_cache_resource(
        &self,
        parsed_uri: &ParsedUri,
    ) -> Result<Vec<ResourceContents>, McpError> {
        match parsed_uri.resource_path.as_str() {
            "statistics" => {
                // Mock cache statistics - in a real implementation this would come from the cache system
                let cache_stats = json!({
                    "total_entries": 0,
                    "hit_rate": 0.0,
                    "miss_rate": 0.0,
                    "total_size_bytes": 0,
                    "average_age_seconds": 0.0,
                    "expired_entries": 0,
                    "by_research_type": {},
                    "last_updated": chrono::Utc::now().to_rfc3339(),
                    "cache_enabled": true,
                    "max_size_bytes": 1024 * 1024 * 100, // 100MB
                    "ttl_seconds": 3600
                });

                let content = serde_json::to_string_pretty(&cache_stats).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize cache stats: {e}"), None)
                })?;

                Ok(vec![ResourceContents::text(
                    content,
                    "mcp://fortitude/cache/statistics".to_string(),
                )])
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown cache resource: {}", parsed_uri.resource_path),
                None,
            )),
        }
    }

    /// Read configuration resource
    async fn read_config_resource(
        &self,
        parsed_uri: &ParsedUri,
    ) -> Result<Vec<ResourceContents>, McpError> {
        match parsed_uri.resource_path.as_str() {
            "current" => {
                // Sanitize configuration - remove sensitive data
                let sanitized_config = json!({
                    "port": self.config.port,
                    "host": self.config.host,
                    "max_connections": self.config.max_connections,
                    "request_timeout": self.config.request_timeout,
                    "auth": {
                        "enabled": self.config.auth.enabled,
                        "token_expiration_hours": self.config.auth.token_expiration_hours,
                        "jwt_secret": "[REDACTED]"
                    },
                    "logging": {
                        "level": self.config.logging.level,
                        "structured": self.config.logging.structured,
                        "file_path": self.config.logging.file_path
                    },
                    "performance": {
                        "cache_size": self.config.performance.cache_size,
                        "cache_ttl": self.config.performance.cache_ttl,
                        "enable_deduplication": self.config.performance.enable_deduplication
                    }
                });

                let content = serde_json::to_string_pretty(&sanitized_config).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize config: {e}"), None)
                })?;

                Ok(vec![ResourceContents::text(
                    content,
                    "mcp://fortitude/config/current".to_string(),
                )])
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown config resource: {}", parsed_uri.resource_path),
                None,
            )),
        }
    }

    /// Read system metrics resource
    async fn read_system_resource(
        &self,
        parsed_uri: &ParsedUri,
    ) -> Result<Vec<ResourceContents>, McpError> {
        match parsed_uri.resource_path.as_str() {
            "metrics" => {
                // Mock system metrics - in a real implementation this would come from monitoring
                let system_metrics = json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "uptime_seconds": 0, // Would be calculated from server start time
                    "memory_usage": {
                        "total_bytes": 1024_u64 * 1024 * 64, // 64MB
                        "used_bytes": 1024_u64 * 1024 * 32,  // 32MB
                        "free_bytes": 1024_u64 * 1024 * 32   // 32MB
                    },
                    "cpu_usage": {
                        "percentage": 5.0,
                        "load_average": [0.1, 0.2, 0.3]
                    },
                    "network": {
                        "connections_active": 0,
                        "bytes_sent": 0,
                        "bytes_received": 0
                    },
                    "disk": {
                        "total_bytes": 1024_u64 * 1024 * 1024 * 10, // 10GB
                        "used_bytes": 1024_u64 * 1024 * 1024 * 3,   // 3GB
                        "free_bytes": 1024_u64 * 1024 * 1024 * 7    // 7GB
                    },
                    "process": {
                        "pid": std::process::id(),
                        "threads": 4,
                        "files_open": 10
                    }
                });

                let content = serde_json::to_string_pretty(&system_metrics).map_err(|e| {
                    McpError::internal_error(
                        format!("Failed to serialize system metrics: {e}"),
                        None,
                    )
                })?;

                Ok(vec![ResourceContents::text(
                    content,
                    "mcp://fortitude/system/metrics".to_string(),
                )])
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown system resource: {}", parsed_uri.resource_path),
                None,
            )),
        }
    }
}

/// Parsed MCP URI structure
#[derive(Debug)]
struct ParsedUri {
    resource_type: String,
    resource_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_config() -> Arc<ServerConfig> {
        Arc::new(ServerConfig::default())
    }

    #[test]
    fn test_mime_type_detection() {
        let md_path = Path::new("test.md");
        assert_eq!(ResourceProvider::detect_mime_type(md_path), "text/markdown");

        let rs_path = Path::new("test.rs");
        assert_eq!(ResourceProvider::detect_mime_type(rs_path), "text/x-rust");

        let json_path = Path::new("test.json");
        assert_eq!(
            ResourceProvider::detect_mime_type(json_path),
            "application/json"
        );

        let unknown_path = Path::new("test.unknown");
        assert_eq!(
            ResourceProvider::detect_mime_type(unknown_path),
            "text/plain"
        );
    }

    #[test]
    fn test_uri_parsing() {
        let config = create_test_config();
        let provider = ResourceProvider::new(config);

        // Valid URI
        let uri = "mcp://fortitude/docs/reference_library/README.md";
        let parsed = provider.parse_and_validate_uri(uri).unwrap();
        assert_eq!(parsed.resource_type, "docs");
        assert_eq!(parsed.resource_path, "reference_library/README.md");

        // Invalid scheme
        let invalid_uri = "http://example.com/test";
        assert!(provider.parse_and_validate_uri(invalid_uri).is_err());

        // Path traversal attempt
        let traversal_uri = "mcp://fortitude/docs/../../../etc/passwd";
        assert!(provider.parse_and_validate_uri(traversal_uri).is_err());
    }

    #[test]
    fn test_resource_type_validation() {
        let config = create_test_config();
        let provider = ResourceProvider::new(config);

        // Valid resource types
        assert!(provider
            .parse_and_validate_uri("mcp://fortitude/docs/test")
            .is_ok());
        assert!(provider
            .parse_and_validate_uri("mcp://fortitude/cache/statistics")
            .is_ok());
        assert!(provider
            .parse_and_validate_uri("mcp://fortitude/config/current")
            .is_ok());
        assert!(provider
            .parse_and_validate_uri("mcp://fortitude/system/metrics")
            .is_ok());

        // Invalid resource type
        assert!(provider
            .parse_and_validate_uri("mcp://fortitude/invalid/test")
            .is_err());
    }

    #[tokio::test]
    async fn test_cache_statistics_resource() {
        let config = create_test_config();
        let provider = ResourceProvider::new(config);

        let parsed_uri = ParsedUri {
            resource_type: "cache".to_string(),
            resource_path: "statistics".to_string(),
        };

        let contents = provider.read_cache_resource(&parsed_uri).await.unwrap();
        assert_eq!(contents.len(), 1);

        // Verify it's valid JSON
        let json_str = match &contents[0] {
            ResourceContents::TextResourceContents { text, .. } => text,
            _ => panic!("Expected text content"),
        };

        let _: serde_json::Value = serde_json::from_str(json_str).unwrap();
    }

    #[tokio::test]
    async fn test_config_resource() {
        let config = create_test_config();
        let provider = ResourceProvider::new(config);

        let parsed_uri = ParsedUri {
            resource_type: "config".to_string(),
            resource_path: "current".to_string(),
        };

        let contents = provider.read_config_resource(&parsed_uri).await.unwrap();
        assert_eq!(contents.len(), 1);

        // Verify it's valid JSON and JWT secret is redacted
        let json_str = match &contents[0] {
            ResourceContents::TextResourceContents { text, .. } => text,
            _ => panic!("Expected text content"),
        };

        let config_json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(config_json["auth"]["jwt_secret"], "[REDACTED]");
    }

    #[tokio::test]
    async fn test_system_metrics_resource() {
        let config = create_test_config();
        let provider = ResourceProvider::new(config);

        let parsed_uri = ParsedUri {
            resource_type: "system".to_string(),
            resource_path: "metrics".to_string(),
        };

        let contents = provider.read_system_resource(&parsed_uri).await.unwrap();
        assert_eq!(contents.len(), 1);

        // Verify it's valid JSON
        let json_str = match &contents[0] {
            ResourceContents::TextResourceContents { text, .. } => text,
            _ => panic!("Expected text content"),
        };

        let metrics_json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert!(metrics_json["timestamp"].is_string());
        assert!(metrics_json["process"]["pid"].is_number());
    }

    #[test]
    fn test_path_traversal_protection() {
        let config = create_test_config();
        let provider = ResourceProvider::new(config);

        // Test various path traversal attempts
        let malicious_uris = vec![
            "mcp://fortitude/docs/../../../etc/passwd",
            "mcp://fortitude/docs/reference_library/../../passwd",
            "mcp://fortitude/docs/~/secret",
            "mcp://fortitude/docs/reference_library/../../../config",
        ];

        for uri in malicious_uris {
            assert!(
                provider.parse_and_validate_uri(uri).is_err(),
                "Should reject: {uri}"
            );
        }
    }
}
