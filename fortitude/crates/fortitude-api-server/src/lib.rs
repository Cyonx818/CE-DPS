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

// ABOUTME: Fortitude API Server library providing production-ready JSON REST API
// Includes authentication, caching, research endpoints, and comprehensive testing

pub mod config;
pub mod extractors;
pub mod middleware;
pub mod models;
pub mod monitoring_types;
pub mod routes;
pub mod server;

pub use config::ApiServerConfig;
pub use models::{HealthCheckRequest, LearningInsight, MonitoringMetricsQuery};
pub use server::ApiServer;
