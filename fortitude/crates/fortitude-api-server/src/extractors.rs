// ABOUTME: Custom Axum extractors that handle deserialization errors gracefully
// Provides safe query parameter parsing that converts errors to proper 400 responses

use crate::models::errors::ApiError;
use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// Safe query parameter extractor that catches deserialization errors
/// and converts them to proper 400 Bad Request responses
#[derive(Debug)]
pub struct SafeQuery<T>(pub T);

impl<T> SafeQuery<T> {
    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, S> FromRequestParts<S> for SafeQuery<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Try to extract query parameters using Axum's standard Query extractor
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(value)) => {
                // Validate the extracted parameters
                if let Err(validation_error) = value.validate() {
                    let api_error = ApiError::ValidationError {
                        message: format!("Query parameter validation failed: {validation_error}"),
                    };
                    return Err(api_error.into_response());
                }

                Ok(SafeQuery(value))
            }
            Err(_extraction_error) => {
                // Convert any extraction/deserialization error to a ValidationError
                // This handles cases like "offset=abc" where abc can't be parsed as usize
                let api_error = ApiError::ValidationError {
                    message: "Invalid query parameters. Please check that numeric values are valid numbers and all required fields are provided.".to_string(),
                };
                Err(api_error.into_response())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Request, StatusCode, Uri};
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestQuery {
        #[validate(range(min = 1, max = 100))]
        limit: Option<usize>,
        #[validate(range(min = 0))]
        offset: Option<usize>,
    }

    #[tokio::test]
    async fn test_safe_query_valid_parameters() {
        let uri: Uri = "/test?limit=10&offset=0".parse().unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();

        let result = SafeQuery::<TestQuery>::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok());

        let SafeQuery(query) = result.unwrap();
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(0));
    }

    #[tokio::test]
    async fn test_safe_query_invalid_numeric_parameters() {
        let uri: Uri = "/test?limit=abc&offset=def".parse().unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();

        let result = SafeQuery::<TestQuery>::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());

        // Since we get a Response back, we can check that it has a 400 status
        let response = result.unwrap_err();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_safe_query_validation_failure() {
        let uri: Uri = "/test?limit=150&offset=0".parse().unwrap(); // limit too high
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();

        let result = SafeQuery::<TestQuery>::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());

        // Since we get a Response back, we can check that it has a 400 status
        let response = result.unwrap_err();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_safe_query_empty_parameters() {
        let uri: Uri = "/test".parse().unwrap();
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();

        let result = SafeQuery::<TestQuery>::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok());

        let SafeQuery(query) = result.unwrap();
        assert_eq!(query.limit, None);
        assert_eq!(query.offset, None);
    }
}
