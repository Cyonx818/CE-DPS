//! Unit tests module for multi-LLM provider system
//! 
//! This module organizes all unit tests for the comprehensive test suite covering:
//! - Provider trait abstractions and implementations
//! - Individual provider implementations (OpenAI, Claude, Gemini)
//! - Fallback strategy engine
//! - Configuration management
//! - Research engine integration

pub mod provider_trait_tests;
pub mod openai_provider_tests;
pub mod claude_provider_tests;
pub mod gemini_provider_tests;
pub mod fallback_strategy_tests;
pub mod configuration_tests;
pub mod research_engine_integration_tests;