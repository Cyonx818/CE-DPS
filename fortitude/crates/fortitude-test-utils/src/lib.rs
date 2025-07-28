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

// ABOUTME: Shared testing utilities for the Fortitude research system
//! This crate contains shared testing utilities, fixtures, and helpers
//! used across the Fortitude test suite.

pub mod classification_fixtures;
pub mod fixtures;
pub mod helpers;

pub use classification_fixtures::*;
pub use fixtures::*;
pub use helpers::*;
