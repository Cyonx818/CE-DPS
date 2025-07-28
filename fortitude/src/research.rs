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
