use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResearchError {
    #[error("API request failed: {0}")]
    ApiError(String),
    #[error("Research topic not found: {0}")]
    TopicNotFound(String),
}

pub struct ResearchPipeline {
    #[allow(dead_code)] // TODO: Will be used when research_topic is implemented
    api_key: String,
}

impl ResearchPipeline {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn research_topic(&self, _topic: &str) -> Result<String, ResearchError> {
        todo!("Implement research functionality")
    }
}
