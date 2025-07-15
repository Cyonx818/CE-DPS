use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Processing failed: {0}")]
    ProcessingError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PipelineConfig {
    pub max_concurrent_requests: usize,
    pub cache_duration: u64,
    pub output_format: String,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 5,
            cache_duration: 3600,
            output_format: "markdown".to_string(),
        }
    }
}

pub struct KnowledgePipeline {
    #[allow(dead_code)] // TODO: Will be used when process method is implemented
    config: PipelineConfig,
}

impl KnowledgePipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    pub async fn process(&self) -> Result<(), PipelineError> {
        todo!("Implement pipeline processing")
    }
}
