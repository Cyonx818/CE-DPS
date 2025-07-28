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

// ABOUTME: HTTP-specific models for Fortitude API server
// Defines request/response schemas and error handling for REST API

pub mod errors;
pub mod requests;
pub mod responses;

// Re-export common types for easier access
pub use responses::{HealthCheckRequest, LearningInsight, MonitoringMetricsQuery};
