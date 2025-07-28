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

// ABOUTME: Core types and errors for the Fortitude research system
//! This crate contains the shared types and error definitions used across
//! the Fortitude research pipeline. It defines the core domain types for
//! research classification, requests, and results.

pub mod classification;
pub mod classification_result;
pub mod error;
pub mod pattern_recognition;
pub mod research;
pub mod storage;

pub use classification::*;
pub use classification_result::*;
pub use error::*;
pub use pattern_recognition::*;
pub use research::*;
pub use storage::*;
