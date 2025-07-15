// ABOUTME: Main HTTP server implementation for Fortitude API server
// Provides production-ready Axum-based server with middleware stack and graceful shutdown

use crate::config::ApiServerConfig;
use crate::middleware::{
    auth::{AuthManager, AuthState},
    cors, logging, monitoring, pattern_tracking,
};
use crate::models::errors::ApiError;
use crate::routes::{
    cache, classification, health, learning, monitoring as routes_monitoring, proactive, providers,
    research,
};
use anyhow::Result;
use axum::{
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
};
use tracing::{error, info, instrument};
use utoipa::OpenApi;

/// OpenAPI documentation definition
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health endpoints
        health::health_check,
        health::protected_health_check,
        // Research endpoints
        research::submit_research,
        research::get_research_by_id,
        research::list_research_results,
        // Classification endpoints
        classification::submit_classification,
        classification::get_classification_by_id,
        classification::list_classification_results,
        classification::get_classification_types,
        // Cache endpoints
        cache::get_cache_stats,
        cache::search_cache,
        cache::get_cache_item,
        cache::delete_cache_item,
        cache::invalidate_cache,
        cache::cleanup_cache,
        // Proactive research endpoints
        proactive::start_proactive_research,
        proactive::stop_proactive_research,
        proactive::get_proactive_status,
        proactive::get_proactive_config,
        proactive::update_proactive_config,
        proactive::list_proactive_tasks,
        proactive::list_proactive_notifications,
        // Learning system endpoints
        learning::get_learning_dashboard_data,
        learning::get_learning_metrics,
        learning::get_learning_health,
        learning::get_learning_performance_summary,
        // Monitoring dashboard endpoints
        routes_monitoring::get_monitoring_dashboard,
        routes_monitoring::get_monitoring_metrics,
        routes_monitoring::get_monitoring_health,
        routes_monitoring::get_monitoring_alerts,
        routes_monitoring::get_monitoring_performance_summary,
    ),
    components(schemas()),
    tags(
        (name = "Health", description = "Health monitoring and status endpoints"),
        (name = "Research", description = "AI-powered research and analysis operations"),
        (name = "Classification", description = "Content classification and categorization"),
        (name = "Cache", description = "Cache management and statistics"),
        (name = "Proactive Research", description = "Automated proactive research and gap detection"),
        (name = "Learning", description = "Learning system metrics and dashboard monitoring"),
        (name = "Monitoring", description = "System monitoring dashboard and observability endpoints")
    ),
    info(
        title = "Fortitude API Server",
        version = "0.1.0",
        description = "Production-ready JSON API server for the Fortitude research system",
        contact(name = "Fortitude Team", email = "fortitude@example.com"),
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.fortitude.example.com", description = "Production server")
    )
)]
struct ApiDoc;

/// Production-ready API server
pub struct ApiServer {
    pub config: ApiServerConfig,
    pub app: Router,
    pub auth_manager: Option<std::sync::Arc<AuthManager>>,
    pub research_state: Option<research::ResearchState>,
    pub classification_state: Option<classification::ClassificationState>,
    pub cache_state: Option<cache::CacheState>,
    pub proactive_state: Option<proactive::ProactiveState>,
    pub learning_state: Option<learning::LearningState>,
    pub monitoring_state: Option<routes_monitoring::MonitoringState>,
    pub provider_state: Option<providers::ProviderState>,
    pub pattern_tracker: Option<pattern_tracking::PatternTracker>,
    pub monitoring_service: Option<std::sync::Arc<monitoring::ApiMonitoringService>>,
}

impl ApiServer {
    /// Create new API server with configuration
    #[instrument(skip(config))]
    pub async fn new(config: ApiServerConfig) -> Result<Self> {
        info!(
            "Initializing API server with config: {}:{}",
            config.host, config.port
        );

        // Create authentication manager if auth is enabled
        let auth_manager = if config.auth.enabled {
            Some(std::sync::Arc::new(AuthManager::new(std::sync::Arc::new(
                config.clone(),
            ))?))
        } else {
            None
        };

        // Initialize research state
        let research_state = match research::ResearchState::new().await {
            Ok(state) => {
                info!("Research pipeline initialized successfully");
                Some(state)
            }
            Err(e) => {
                error!("Failed to initialize research pipeline: {}", e);
                info!("Research endpoints will be unavailable");
                None
            }
        };

        // Initialize classification state
        let classification_state = match classification::ClassificationState::new().await {
            Ok(state) => {
                info!("Classification system initialized successfully");
                Some(state)
            }
            Err(e) => {
                error!("Failed to initialize classification system: {}", e);
                info!("Classification endpoints will be unavailable");
                None
            }
        };

        // Initialize cache state
        let cache_state = match cache::CacheState::new(&config).await {
            Ok(state) => {
                info!("Cache system initialized successfully");
                Some(state)
            }
            Err(e) => {
                error!("Failed to initialize cache system: {}", e);
                info!("Cache endpoints will be unavailable");
                None
            }
        };

        // Initialize proactive state
        let proactive_state = match proactive::ProactiveState::new().await {
            Ok(state) => {
                info!("Proactive research system initialized successfully");
                Some(state)
            }
            Err(e) => {
                error!("Failed to initialize proactive research system: {}", e);
                info!("Proactive research endpoints will be unavailable");
                None
            }
        };

        // Initialize learning state
        let learning_state = match learning::LearningState::new().await {
            Ok(state) => {
                info!("Learning system initialized successfully");
                Some(state)
            }
            Err(e) => {
                error!("Failed to initialize learning system: {}", e);
                info!("Learning endpoints will be unavailable");
                None
            }
        };

        // Initialize monitoring state
        let monitoring_state = match routes_monitoring::MonitoringState::new().await {
            Ok(state) => {
                info!("Monitoring system initialized successfully");
                Some(state)
            }
            Err(e) => {
                error!("Failed to initialize monitoring system: {}", e);
                info!("Monitoring endpoints will be unavailable");
                None
            }
        };

        // Initialize provider state
        let provider_state = Some(providers::ProviderState::default());
        info!("Provider management system initialized successfully");

        // Initialize pattern tracking
        let pattern_tracker = if *config.features.get("pattern_tracking").unwrap_or(&false) {
            let pattern_config = pattern_tracking::PatternTrackingConfig::default();
            let (tracker, _receiver) = pattern_tracking::PatternTracker::new(pattern_config);
            info!("Pattern tracking initialized successfully");
            Some(tracker)
        } else {
            info!("Pattern tracking disabled");
            None
        };

        // Initialize monitoring service
        let monitoring_service = Some(std::sync::Arc::new(
            monitoring::ApiMonitoringService::for_api_server(),
        ));
        info!("API monitoring service initialized successfully");

        // Build the application router
        let app = Self::build_router(
            &config,
            auth_manager.as_ref(),
            research_state.as_ref(),
            classification_state.as_ref(),
            cache_state.as_ref(),
            proactive_state.as_ref(),
            learning_state.as_ref(),
            monitoring_state.as_ref(),
            provider_state.as_ref(),
            pattern_tracker.as_ref(),
            monitoring_service.as_ref(),
        )
        .await?;

        Ok(Self {
            config,
            app,
            auth_manager,
            research_state,
            classification_state,
            cache_state,
            proactive_state,
            learning_state,
            monitoring_state,
            provider_state,
            pattern_tracker,
            monitoring_service,
        })
    }

    /// Build the main application router with middleware
    #[allow(clippy::too_many_arguments)]
    async fn build_router(
        _config: &ApiServerConfig,
        auth_manager: Option<&std::sync::Arc<AuthManager>>,
        research_state: Option<&research::ResearchState>,
        classification_state: Option<&classification::ClassificationState>,
        cache_state: Option<&cache::CacheState>,
        proactive_state: Option<&proactive::ProactiveState>,
        learning_state: Option<&learning::LearningState>,
        monitoring_state: Option<&routes_monitoring::MonitoringState>,
        provider_state: Option<&providers::ProviderState>,
        pattern_tracker: Option<&pattern_tracking::PatternTracker>,
        monitoring_service: Option<&std::sync::Arc<monitoring::ApiMonitoringService>>,
    ) -> Result<Router> {
        // Note: Using manual Swagger UI implementation instead of utoipa_swagger_ui crate integration

        // Create the basic router with health check and documentation
        let mut app = Router::new()
            .route("/health", get(health::health_check))
            // Serve OpenAPI spec directly
            .route("/openapi.yaml", get(Self::serve_openapi_yaml))
            .route("/api-docs/openapi.json", get(Self::serve_openapi_json))
            // Add Swagger UI routes manually
            .route("/docs", get(Self::redirect_to_docs))
            .route("/docs/", get(Self::serve_swagger_ui))
            .route("/docs/{*tail}", get(Self::serve_swagger_ui))
            // Fallback for unknown routes
            .fallback(Self::handle_404);

        // Add protected routes
        if let Some(auth_mgr) = auth_manager {
            // With authentication enabled, add middleware
            let auth_state = AuthState {
                auth_manager: auth_mgr.clone(),
            };

            let mut protected_routes = Router::new().route(
                "/api/v1/health/protected",
                get(health::protected_health_check),
            );

            // Add research routes if available
            if let Some(research_state) = research_state {
                let research_routes = Router::new()
                    .route("/api/v1/research", post(research::submit_research))
                    .route("/api/v1/research/{id}", get(research::get_research_by_id))
                    .route("/api/v1/research", get(research::list_research_results))
                    .with_state(research_state.clone());

                protected_routes = protected_routes.merge(research_routes);
            }

            // Add classification routes if available
            if let Some(classification_state) = classification_state {
                let classification_routes = Router::new()
                    .route(
                        "/api/v1/classify",
                        post(classification::submit_classification),
                    )
                    .route(
                        "/api/v1/classify/{id}",
                        get(classification::get_classification_by_id),
                    )
                    .route(
                        "/api/v1/classify",
                        get(classification::list_classification_results),
                    )
                    .route(
                        "/api/v1/classify/types",
                        get(classification::get_classification_types),
                    )
                    .with_state(classification_state.clone());

                protected_routes = protected_routes.merge(classification_routes);
            }

            // Add cache routes if available
            if let Some(cache_state) = cache_state {
                use crate::middleware::auth::{require_permission, Permission};

                // Cache read operations - require ResourcesRead permission
                let cache_read_routes = Router::new()
                    .route("/api/v1/cache/stats", get(cache::get_cache_stats))
                    .route("/api/v1/cache/search", get(cache::search_cache))
                    .route("/api/v1/cache/{id}", get(cache::get_cache_item))
                    .route_layer(axum::middleware::from_fn(require_permission(
                        Permission::ResourcesRead,
                    )))
                    .with_state(cache_state.clone());

                // Cache admin operations - require Admin permission
                let cache_admin_routes = Router::new()
                    .route("/api/v1/cache/{id}", delete(cache::delete_cache_item))
                    .route("/api/v1/cache/invalidate", post(cache::invalidate_cache))
                    .route("/api/v1/cache/cleanup", post(cache::cleanup_cache))
                    .route_layer(axum::middleware::from_fn(require_permission(
                        Permission::Admin,
                    )))
                    .with_state(cache_state.clone());

                protected_routes = protected_routes
                    .merge(cache_read_routes)
                    .merge(cache_admin_routes);
            }

            // Add proactive research routes if available
            if let Some(proactive_state) = proactive_state {
                let proactive_routes = Router::new()
                    .route(
                        "/api/v1/proactive/start",
                        post(proactive::start_proactive_research),
                    )
                    .route(
                        "/api/v1/proactive/stop",
                        post(proactive::stop_proactive_research),
                    )
                    .route(
                        "/api/v1/proactive/status",
                        get(proactive::get_proactive_status),
                    )
                    .route(
                        "/api/v1/proactive/config",
                        get(proactive::get_proactive_config),
                    )
                    .route(
                        "/api/v1/proactive/config",
                        axum::routing::put(proactive::update_proactive_config),
                    )
                    .route(
                        "/api/v1/proactive/tasks",
                        get(proactive::list_proactive_tasks),
                    )
                    .route(
                        "/api/v1/proactive/notifications",
                        get(proactive::list_proactive_notifications),
                    )
                    .with_state(proactive_state.clone());

                protected_routes = protected_routes.merge(proactive_routes);
            }

            // Add learning system routes if available
            if let Some(learning_state) = learning_state {
                let learning_routes = Router::new()
                    .route(
                        "/api/v1/learning/dashboard",
                        get(learning::get_learning_dashboard_data),
                    )
                    .route(
                        "/api/v1/learning/metrics",
                        get(learning::get_learning_metrics),
                    )
                    .route(
                        "/api/v1/learning/health",
                        get(learning::get_learning_health),
                    )
                    .route(
                        "/api/v1/learning/performance",
                        get(learning::get_learning_performance_summary),
                    )
                    .with_state(Arc::new((*learning_state).clone()));

                protected_routes = protected_routes.merge(learning_routes);
            }

            // Add monitoring system routes if available
            if let Some(monitoring_state) = monitoring_state {
                let monitoring_routes = Router::new()
                    .route(
                        "/api/v1/monitoring/dashboard",
                        get(routes_monitoring::get_monitoring_dashboard),
                    )
                    .route(
                        "/api/v1/monitoring/metrics",
                        get(routes_monitoring::get_monitoring_metrics),
                    )
                    .route(
                        "/api/v1/monitoring/health",
                        get(routes_monitoring::get_monitoring_health),
                    )
                    .route(
                        "/api/v1/monitoring/alerts",
                        get(routes_monitoring::get_monitoring_alerts),
                    )
                    .route(
                        "/api/v1/monitoring/performance",
                        get(routes_monitoring::get_monitoring_performance_summary),
                    )
                    .with_state(Arc::new((*monitoring_state).clone()));

                protected_routes = protected_routes.merge(monitoring_routes);
            }

            // Add provider management routes if available
            if let Some(provider_state) = provider_state {
                let provider_routes =
                    providers::create_router().with_state(Arc::new(provider_state.clone()));

                protected_routes = protected_routes.merge(provider_routes);
            }

            protected_routes = protected_routes.layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                crate::middleware::auth::jwt_auth_middleware,
            ));

            app = app.merge(protected_routes);
        } else {
            // Even with authentication disabled, add the protected route (without middleware)
            let mut protected_routes = Router::new().route(
                "/api/v1/health/protected",
                get(health::protected_health_check),
            );

            // Add research routes if available (without auth middleware)
            if let Some(research_state) = research_state {
                let research_routes = Router::new()
                    .route("/api/v1/research", post(research::submit_research))
                    .route("/api/v1/research/{id}", get(research::get_research_by_id))
                    .route("/api/v1/research", get(research::list_research_results))
                    .with_state(research_state.clone());

                protected_routes = protected_routes.merge(research_routes);
            }

            // Add classification routes if available (without auth middleware)
            if let Some(classification_state) = classification_state {
                let classification_routes = Router::new()
                    .route(
                        "/api/v1/classify",
                        post(classification::submit_classification),
                    )
                    .route(
                        "/api/v1/classify/{id}",
                        get(classification::get_classification_by_id),
                    )
                    .route(
                        "/api/v1/classify",
                        get(classification::list_classification_results),
                    )
                    .route(
                        "/api/v1/classify/types",
                        get(classification::get_classification_types),
                    )
                    .with_state(classification_state.clone());

                protected_routes = protected_routes.merge(classification_routes);
            }

            // Add cache routes if available (without auth middleware)
            if let Some(cache_state) = cache_state {
                let cache_routes = Router::new()
                    .route("/api/v1/cache/stats", get(cache::get_cache_stats))
                    .route("/api/v1/cache/search", get(cache::search_cache))
                    .route("/api/v1/cache/{id}", get(cache::get_cache_item))
                    .route("/api/v1/cache/{id}", delete(cache::delete_cache_item))
                    .route("/api/v1/cache/invalidate", post(cache::invalidate_cache))
                    .route("/api/v1/cache/cleanup", post(cache::cleanup_cache))
                    .with_state(cache_state.clone());

                protected_routes = protected_routes.merge(cache_routes);
            }

            // Add proactive research routes if available (without auth middleware)
            if let Some(proactive_state) = proactive_state {
                let proactive_routes = Router::new()
                    .route(
                        "/api/v1/proactive/start",
                        post(proactive::start_proactive_research),
                    )
                    .route(
                        "/api/v1/proactive/stop",
                        post(proactive::stop_proactive_research),
                    )
                    .route(
                        "/api/v1/proactive/status",
                        get(proactive::get_proactive_status),
                    )
                    .route(
                        "/api/v1/proactive/config",
                        get(proactive::get_proactive_config),
                    )
                    .route(
                        "/api/v1/proactive/config",
                        axum::routing::put(proactive::update_proactive_config),
                    )
                    .route(
                        "/api/v1/proactive/tasks",
                        get(proactive::list_proactive_tasks),
                    )
                    .route(
                        "/api/v1/proactive/notifications",
                        get(proactive::list_proactive_notifications),
                    )
                    .with_state(proactive_state.clone());

                protected_routes = protected_routes.merge(proactive_routes);
            }

            // Add learning system routes if available (without auth middleware)
            if let Some(learning_state) = learning_state {
                let learning_routes = Router::new()
                    .route(
                        "/api/v1/learning/dashboard",
                        get(learning::get_learning_dashboard_data),
                    )
                    .route(
                        "/api/v1/learning/metrics",
                        get(learning::get_learning_metrics),
                    )
                    .route(
                        "/api/v1/learning/health",
                        get(learning::get_learning_health),
                    )
                    .route(
                        "/api/v1/learning/performance",
                        get(learning::get_learning_performance_summary),
                    )
                    .with_state(Arc::new((*learning_state).clone()));

                protected_routes = protected_routes.merge(learning_routes);
            }

            // Add monitoring system routes if available (without auth middleware)
            if let Some(monitoring_state) = monitoring_state {
                let monitoring_routes = Router::new()
                    .route(
                        "/api/v1/monitoring/dashboard",
                        get(routes_monitoring::get_monitoring_dashboard),
                    )
                    .route(
                        "/api/v1/monitoring/metrics",
                        get(routes_monitoring::get_monitoring_metrics),
                    )
                    .route(
                        "/api/v1/monitoring/health",
                        get(routes_monitoring::get_monitoring_health),
                    )
                    .route(
                        "/api/v1/monitoring/alerts",
                        get(routes_monitoring::get_monitoring_alerts),
                    )
                    .route(
                        "/api/v1/monitoring/performance",
                        get(routes_monitoring::get_monitoring_performance_summary),
                    )
                    .with_state(Arc::new((*monitoring_state).clone()));

                protected_routes = protected_routes.merge(monitoring_routes);
            }

            // Add provider management routes if available (without auth middleware)
            if let Some(provider_state) = provider_state {
                let provider_routes =
                    providers::create_router().with_state(Arc::new(provider_state.clone()));

                protected_routes = protected_routes.merge(provider_routes);
            }

            app = app.merge(protected_routes);
        }

        // Add middleware stack with comprehensive configuration
        // Note: Middleware is applied in reverse order (last = innermost)
        let mut app = app
            // Individual layers to avoid type compatibility issues
            .layer(CompressionLayer::new())
            .layer(cors::create_cors_layer())
            .layer(logging::create_trace_layer::<axum::body::Body>())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(CatchPanicLayer::custom(Self::handle_panic));

        // Add pattern tracking middleware if enabled
        if let Some(tracker) = pattern_tracker {
            app = app
                .layer(axum::Extension(tracker.clone()))
                .layer(axum::middleware::from_fn(
                    pattern_tracking::pattern_tracking_middleware,
                ));
        }

        // Add monitoring middleware if enabled
        if let Some(monitoring_svc) = monitoring_service {
            app = app.layer(axum::middleware::from_fn_with_state(
                monitoring_svc.clone(),
                monitoring::monitoring_middleware,
            ));
        }

        let app = app;

        Ok(app)
    }

    /// Handle 404 errors for unknown routes
    async fn handle_404() -> impl IntoResponse {
        let error = ApiError::NotFound {
            resource: "The requested endpoint was not found".to_string(),
        };
        error.into_response()
    }

    /// Serve the OpenAPI specification in YAML format
    async fn serve_openapi_yaml() -> impl IntoResponse {
        use axum::http::header;

        // Read the OpenAPI YAML file from the project root
        let openapi_content = std::fs::read_to_string("openapi.yaml")
            .unwrap_or_else(|_| {
                // Fallback: just return a minimal spec since we have the full spec in the YAML file
                "openapi: 3.0.3\ninfo:\n  title: Fortitude API\n  version: 0.1.0\n  description: API documentation at /docs".to_string()
            });

        (
            [(header::CONTENT_TYPE, "application/x-yaml")],
            openapi_content,
        )
    }

    /// Serve the OpenAPI specification in JSON format
    async fn serve_openapi_json() -> impl IntoResponse {
        use axum::http::header;

        // Generate JSON from the OpenAPI struct
        let openapi_json = serde_json::to_string_pretty(&ApiDoc::openapi()).unwrap_or_else(|_| {
            r#"{"openapi":"3.0.3","info":{"title":"Fortitude API","version":"0.1.0"}}"#.to_string()
        });

        ([(header::CONTENT_TYPE, "application/json")], openapi_json)
    }

    /// Redirect to docs index
    async fn redirect_to_docs() -> impl IntoResponse {
        use axum::http::{header, StatusCode};

        (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, "/docs/")],
            "Redirecting to Swagger UI",
        )
    }

    /// Serve Swagger UI
    async fn serve_swagger_ui() -> impl IntoResponse {
        use axum::http::header;

        // Return a simple HTML page that loads the Swagger UI
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Fortitude API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui.css" />
    <style>
        html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin:0; background: #fafafa; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: '/api-docs/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>
        "#;

        ([(header::CONTENT_TYPE, "text/html")], html)
    }

    /// Handle panic recovery
    fn handle_panic(_err: Box<dyn std::any::Any + Send + 'static>) -> Response {
        error!("Service panic occurred");
        let error = ApiError::InternalError {
            message: "Internal server error occurred".to_string(),
        };
        error.into_response()
    }

    /// Run the server with graceful shutdown
    #[instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        let bind_addr = self.config.bind_address();
        info!("Starting server on {}", bind_addr);

        // Create TCP listener
        let listener = TcpListener::bind(&bind_addr).await?;
        info!("Server listening on {}", bind_addr);

        // Set up graceful shutdown signal
        let shutdown_signal = async {
            let ctrl_c = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install CTRL+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => {
                    info!("Received SIGINT (Ctrl+C) signal");
                }
                _ = terminate => {
                    info!("Received SIGTERM signal");
                }
            }
        };

        // Run server with graceful shutdown
        axum::serve(listener, self.app)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        info!("Server shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = ApiServerConfig::default();
        let server = ApiServer::new(config).await.unwrap();
        // Should not panic
        assert!(!server.config.bind_address().is_empty());
    }

    #[tokio::test]
    async fn test_router_building() {
        let config = ApiServerConfig::default();
        let router = ApiServer::build_router(
            &config, None, None, None, None, None, None, None, None, None, None,
        )
        .await
        .unwrap();
        // Should not panic
        assert!(!format!("{router:?}").is_empty());
    }

    #[tokio::test]
    async fn test_server_config_validation() {
        let config = ApiServerConfig::default();
        assert!(!config.host.is_empty());
        assert!(config.port > 0);
        // Port is u16, so it's automatically within valid range

        let server = ApiServer::new(config).await.unwrap();
        assert!(!server.config.bind_address().is_empty());
    }
}
