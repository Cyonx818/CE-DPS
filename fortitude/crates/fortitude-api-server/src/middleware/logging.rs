// ABOUTME: Logging middleware for request/response tracing and monitoring

use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Create a tracing layer for request/response logging
///
/// This configures structured logging for all HTTP requests and responses,
/// including timing information and error details.
pub fn create_trace_layer<B>() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    DefaultMakeSpan,
    DefaultOnRequest,
    DefaultOnResponse,
> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_layer_creation() {
        // This test ensures the trace layer can be created without panic
        let _trace_layer = create_trace_layer::<axum::body::Body>();
        // If we reach here, the layer was created successfully
    }
}
