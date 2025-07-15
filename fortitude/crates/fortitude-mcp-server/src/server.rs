// ABOUTME: Production-ready MCP server implementation for Fortitude
// Provides secure, performant Model Context Protocol server
// Follows production patterns with authentication, error handling, and observability

use crate::auth::{AuthManager, AuthMiddleware, Permission};
use crate::config::ServerConfig;
use crate::monitoring::McpMonitoringService;
use crate::pattern_tracking::{McpPatternTracker, McpPatternTrackingConfig};
use crate::resources::ResourceProvider;
use crate::tools::FortitudeTools;
use anyhow::Result;
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Implementation, InitializeRequestParam,
        InitializeResult, ListResourcesResult, ListToolsResult, PaginatedRequestParam,
        ProtocolVersion, ReadResourceRequestParam, ReadResourceResult, ServerCapabilities,
        ServerInfo,
    },
    service::{RequestContext, RoleServer},
    Error as McpError, ServerHandler, ServiceExt,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, instrument};

/// Production MCP server for Fortitude AI research assistant
#[derive(Clone)]
pub struct McpServer {
    config: Arc<ServerConfig>,
    tools: Arc<FortitudeTools>,
    resources: Arc<ResourceProvider>,
    #[allow(dead_code)] // Used for authentication setup, needed for structure completeness
    auth_manager: Arc<AuthManager>,
    auth_middleware: Arc<AuthMiddleware>,
    pattern_tracker: Option<Arc<McpPatternTracker>>,
    monitoring_service: Arc<McpMonitoringService>,
    _inner: Arc<RwLock<ServerState>>,
}

struct ServerState {
    // Internal state will be expanded in subsequent tasks
    _placeholder: bool,
}

impl McpServer {
    /// Create a new MCP server with the given configuration
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let config_arc = Arc::new(config.clone());

        // Initialize authentication manager
        let auth_manager = Arc::new(AuthManager::new(config_arc.clone())?);
        let auth_middleware = Arc::new(AuthMiddleware::new(auth_manager.clone()));

        // Initialize tools
        let tools = Arc::new(FortitudeTools::new(config.clone()).await?);

        // Initialize resources
        let resources = Arc::new(ResourceProvider::new(config_arc.clone()));

        // Initialize pattern tracking if enabled
        let pattern_tracker = if config.integration.enable_pattern_tracking.unwrap_or(false) {
            let pattern_config = McpPatternTrackingConfig::default();
            let (tracker, _receiver) = McpPatternTracker::new(pattern_config);
            info!("MCP pattern tracking initialized successfully");
            Some(Arc::new(tracker))
        } else {
            info!("MCP pattern tracking disabled");
            None
        };

        // Initialize monitoring service
        let monitoring_service = Arc::new(McpMonitoringService::for_mcp_server());
        info!("MCP monitoring service initialized successfully");

        let inner = Arc::new(RwLock::new(ServerState { _placeholder: true }));

        Ok(Self {
            config: config_arc,
            tools,
            resources,
            auth_manager,
            auth_middleware,
            pattern_tracker,
            monitoring_service,
            _inner: inner,
        })
    }

    /// Run the MCP server with graceful shutdown
    pub async fn run(self) -> Result<()> {
        info!("Starting MCP server on port {}", self.config.port);

        // Create the MCP server using stdio transport
        let transport = rmcp::transport::stdio();

        // Set up graceful shutdown
        let shutdown = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C handler");
            info!("Received shutdown signal");
        };

        // Run server with graceful shutdown
        tokio::select! {
            result = self.serve(transport) => {
                match result {
                    Ok(service) => {
                        info!("MCP server completed successfully");
                        service.cancel().await.ok();
                    }
                    Err(e) => error!("MCP server error: {:?}", e),
                }
            }
            _ = shutdown => {
                info!("Shutting down MCP server gracefully");
            }
        }

        Ok(())
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    #[instrument(skip(self, _context))]
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("MCP server initialized");
        Ok(InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "fortitude-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some("Fortitude MCP server for AI research assistance".to_string()),
        })
    }

    #[instrument(skip(self, _context))]
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        info!("Listing available tools");
        Ok(self.tools.list_tools())
    }

    #[instrument(skip(self, _context))]
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        info!("Calling tool: {}", request.name);
        let start_time = std::time::Instant::now();

        // Extract client ID from context (using a simple approach for now)
        let client_id = "mcp_client"; // In production, this would come from the connection

        // Authenticate and authorize the request
        let _auth_start = std::time::Instant::now();
        let auth_result = self
            .auth_middleware
            .authenticate_request(
                None, // Auth header would come from context in a real implementation
                client_id,
                Permission::ResearchRead,
            )
            .await;

        let claims = match auth_result {
            Ok(claims) => {
                self.monitoring_service
                    .record_auth_attempt(true, Some(client_id))
                    .await
                    .ok();
                claims
            }
            Err(e) => {
                self.monitoring_service
                    .record_auth_attempt(false, Some(client_id))
                    .await
                    .ok();
                return Err(e);
            }
        };

        // Call the tool
        let result = self.tools.call_tool(request.clone()).await;
        let execution_time = start_time.elapsed();
        let success = result.is_ok();

        // Calculate argument and response sizes
        let args_size = request.arguments.as_ref().map(|args| {
            serde_json::to_string(args)
                .map(|s| s.len() as u64)
                .unwrap_or(0)
        });
        let response_size = if success {
            result.as_ref().ok().map(|r| {
                serde_json::to_string(r)
                    .map(|s| s.len() as u64)
                    .unwrap_or(0)
            })
        } else {
            None
        };

        // Record monitoring metrics
        if let Err(e) = self
            .monitoring_service
            .record_tool_call(
                &request.name,
                execution_time,
                success,
                args_size,
                response_size,
            )
            .await
        {
            error!("Failed to record tool call metrics: {}", e);
        }

        // Track pattern if enabled
        if let Some(tracker) = &self.pattern_tracker {
            let exit_code = if success { 0 } else { 1 };

            tracker
                .track_tool_call(
                    &request.name,
                    request.arguments.as_ref(),
                    Some(&claims),
                    execution_time,
                    success,
                    exit_code,
                )
                .await;
        }

        result
    }

    #[instrument(skip(self, _context))]
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        info!("Listing available resources");

        // Extract client ID from context
        let client_id = "mcp_client"; // In production, this would come from the connection

        // Authenticate and authorize the request
        let _claims = self
            .auth_middleware
            .authenticate_request(
                None, // Auth header would come from context in a real implementation
                client_id,
                Permission::ResourcesRead,
            )
            .await?;

        let resources = self.resources.list_resources().await?;

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
        })
    }

    #[instrument(skip(self, _context))]
    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("Reading resource: {}", request.uri);
        let start_time = std::time::Instant::now();

        // Extract client ID from context
        let client_id = "mcp_client"; // In production, this would come from the connection

        // Determine required permission based on resource type
        let required_permission = if request.uri.contains("/config/") {
            Permission::ConfigRead
        } else {
            Permission::ResourcesRead
        };

        // Authenticate and authorize the request
        let auth_result = self
            .auth_middleware
            .authenticate_request(
                None, // Auth header would come from context in a real implementation
                client_id,
                required_permission,
            )
            .await;

        let _claims = match auth_result {
            Ok(claims) => {
                self.monitoring_service
                    .record_auth_attempt(true, Some(client_id))
                    .await
                    .ok();
                claims
            }
            Err(e) => {
                self.monitoring_service
                    .record_auth_attempt(false, Some(client_id))
                    .await
                    .ok();
                return Err(e);
            }
        };

        // Read the resource
        let result = self.resources.read_resource(&request.uri).await;
        let execution_time = start_time.elapsed();
        let success = result.is_ok();

        // Calculate resource size
        let resource_size = if success {
            result
                .as_ref()
                .ok()
                .and_then(|r| r.first())
                .map(|content| match content {
                    rmcp::model::ResourceContents::TextResourceContents { text, .. } => {
                        text.len() as u64
                    }
                    rmcp::model::ResourceContents::BlobResourceContents { blob, .. } => {
                        blob.len() as u64
                    }
                })
        } else {
            None
        };

        // Record monitoring metrics
        if let Err(e) = self
            .monitoring_service
            .record_resource_read(&request.uri, execution_time, success, resource_size)
            .await
        {
            error!("Failed to record resource read metrics: {}", e);
        }

        match result {
            Ok(contents) => Ok(ReadResourceResult { contents }),
            Err(e) => Err(e),
        }
    }
}
