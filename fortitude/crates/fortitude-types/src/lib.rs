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
