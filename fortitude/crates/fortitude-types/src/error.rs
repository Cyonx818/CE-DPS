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

// ABOUTME: Error types for the Fortitude research system
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FortitudeError {
    #[error("Research error: {0}")]
    Research(#[from] ResearchError),

    #[error("Classification error: {0}")]
    Classification(#[from] ClassificationError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Pipeline error: {0}")]
    Pipeline(#[from] PipelineError),

    #[error("API error: {0}")]
    Api(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ResearchError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Research topic not found: {0}")]
    TopicNotFound(String),

    #[error("Invalid research type: {0}")]
    InvalidType(String),

    #[error("Research timeout")]
    Timeout,

    #[error("Research quota exceeded")]
    QuotaExceeded,
}

#[derive(Error, Debug)]
pub enum ClassificationError {
    #[error("Classification failed: {0}")]
    Failed(String),

    #[error("Confidence threshold not met: {actual} < {threshold}")]
    LowConfidence { actual: f64, threshold: f64 },

    #[error("No classification rules matched")]
    NoRulesMatched,

    #[error("Invalid classification input: {0}")]
    InvalidInput(String),

    #[error("Processing timeout: classification took too long")]
    ProcessingTimeout,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Storage full")]
    StorageFull,

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Index error: {0}")]
    Index(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Pipeline configuration error: {0}")]
    Config(String),

    #[error("Pipeline processing failed: {0}")]
    Processing(String),

    #[error("Pipeline stage failed: {stage} - {error}")]
    StageFailed { stage: String, error: String },

    #[error("Pipeline timeout")]
    Timeout,

    #[error("Pipeline aborted")]
    Aborted,
}

pub type Result<T> = std::result::Result<T, FortitudeError>;
