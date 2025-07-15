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
