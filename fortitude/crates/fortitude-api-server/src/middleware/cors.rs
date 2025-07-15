// ABOUTME: CORS middleware configuration for API server cross-origin requests

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

/// Create CORS layer for the API server
///
/// This configures cross-origin resource sharing to allow requests from
/// various origins during development and production.
pub fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_credentials(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_creation() {
        // This test ensures the CORS layer can be created without panic
        let _cors_layer = create_cors_layer();
        // If we reach here, the layer was created successfully
    }
}
