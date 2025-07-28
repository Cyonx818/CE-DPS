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

// ABOUTME: Classification system modules for multi-dimensional research classification

// Basic classification engine
mod basic_classifier;
pub use basic_classifier::*;

// Advanced classification engine
pub mod advanced_classifier;
pub use advanced_classifier::*;

// Context detection system
pub mod context_detector;
pub use context_detector::*;

// Classification rules
pub mod rules;
pub use rules::*;

// Scoring and signal composition
pub mod scoring;
pub use scoring::*;
