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
