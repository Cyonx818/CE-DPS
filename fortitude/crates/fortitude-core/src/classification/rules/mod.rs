// ABOUTME: Context detection rules modules for multi-dimensional classification

pub mod audience_rules;
pub mod domain_rules;
pub mod urgency_rules;

pub use audience_rules::*;
pub use domain_rules::*;
pub use urgency_rules::*;
