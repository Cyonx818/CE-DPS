// ABOUTME: API module for external service integrations
//! This module provides API client implementations for external services
//! including Claude API integration for research generation.

pub mod claude;
pub mod client;
pub mod error;

pub use claude::*;
pub use client::*;
pub use error::*;
