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

// ABOUTME: Health check endpoints for API server monitoring and status verification

use crate::middleware::auth::Claims;
use crate::models::responses::{ComponentHealth, HealthResponse};
use axum::{http::StatusCode, response::Json, Extension};
use chrono::Utc;
use std::collections::HashMap;
use tracing::instrument;
use utoipa;

/// Simple health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse),
    ),
    tag = "Health"
)]
#[instrument]
pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    let mut components = HashMap::new();

    // Check basic system health
    components.insert(
        "system".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            details: Some("System is operational".to_string()),
        },
    );

    // Check database health (placeholder - would be actual checks)
    components.insert(
        "database".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            details: Some("Database connections available".to_string()),
        },
    );

    // Check cache health (placeholder - would be actual checks)
    components.insert(
        "cache".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            details: Some("Cache system operational".to_string()),
        },
    );

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // In production, this would track actual uptime
        components,
    };

    (StatusCode::OK, Json(response))
}

/// Protected health check endpoint that requires authentication
#[utoipa::path(
    get,
    path = "/api/v1/health/protected",
    responses(
        (status = 200, description = "Protected health check successful", body = HealthResponse),
        (status = 401, description = "Unauthorized - JWT token required"),
    ),
    tag = "Health",
    security(("jwt_auth" = []))
)]
#[instrument(skip(claims))]
pub async fn protected_health_check(
    claims: Option<Extension<Claims>>,
) -> (StatusCode, Json<HealthResponse>) {
    let mut components = HashMap::new();

    // Check basic system health
    components.insert(
        "system".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            details: Some("System is operational".to_string()),
        },
    );

    // Check authentication health
    if let Some(Extension(claims)) = claims {
        components.insert(
            "authentication".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                last_check: Utc::now(),
                details: Some(format!("Authenticated user: {}", claims.sub)),
            },
        );

        // Check user permissions
        components.insert(
            "permissions".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                last_check: Utc::now(),
                details: Some(format!("User permissions: {:?}", claims.permissions)),
            },
        );
    } else {
        components.insert(
            "authentication".to_string(),
            ComponentHealth {
                status: "disabled".to_string(),
                last_check: Utc::now(),
                details: Some("Authentication is disabled".to_string()),
            },
        );

        components.insert(
            "permissions".to_string(),
            ComponentHealth {
                status: "open".to_string(),
                last_check: Utc::now(),
                details: Some("No authentication required".to_string()),
            },
        );
    }

    // Check database health (placeholder - would be actual checks)
    components.insert(
        "database".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            details: Some("Database connections available".to_string()),
        },
    );

    // Check cache health (placeholder - would be actual checks)
    components.insert(
        "cache".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            details: Some("Cache system operational".to_string()),
        },
    );

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // In production, this would track actual uptime
        components,
    };

    (StatusCode::OK, Json(response))
}
