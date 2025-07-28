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

// ABOUTME: Prompt template system for type-specific research generation
//! This module provides a template system for generating research prompts
//! based on research type, with support for progressive disclosure and
//! parameter substitution.

pub mod parameters;
pub mod registry;
pub mod substitution;
pub mod templates;
pub mod validation;

// Re-export key types for easier access
pub use parameters::*;
pub use registry::*;
pub use substitution::*;
pub use templates::*;
pub use validation::*;
