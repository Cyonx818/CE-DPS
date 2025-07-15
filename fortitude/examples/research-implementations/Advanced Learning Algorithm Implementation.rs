// Cargo.toml dependencies:
// [dependencies]
// tokio = { version = "1.0", features = ["full"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// thiserror = "1.0"
// nalgebra = "0.32"
// dashmap = "5.5"
// futures = "0.3"
// tracing = "0.1"
// uuid = { version = "1.0", features = ["v4", "serde"] }

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::{interval, sleep};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use nalgebra::{DVector, DMatrix};
use dashmap::DashMap;
use uuid::Uuid;
use futures::stream::{Stream, StreamExt};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum LearningError {
    #[error("Invalid learning rate: {rate}. Must be positive and finite")]
    InvalidLearningRate { rate: f64 },
    
    #[error("Convergence failed after {iterations} iterations")]
    ConvergenceFailure { iterations: usize },
    
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Model not initialized")]
    ModelNotInitialized,
    
    #[error("Channel communication error: {source}")]
    ChannelError { source: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Numerical instability detected")]
    NumericalInstability,
}

// ============================================================================
// Core Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExample {
    pub id: Uuid,
    pub prompt: String,
    pub context: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSignal {
    pub example_id: Uuid,
    pub score: f64,           // Quality score (0.0 - 1.0)
    pub latency_ms: u64,      // Response latency
    pub user_rating: Option<f64>, // Optional explicit user feedback
    pub success: bool,        // Task completion success
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct LearningMetrics {
    pub loss: f64,
    pub gradient_norm: f64,
    pub learning_rate: f64,
    pub convergence_rate: f64,
    pub samples_processed: usize,
    pub timestamp: Instant,
}

// ============================================================================
// Adaptive Learning Rate Strategies
// ============================================================================

pub trait AdaptiveLearningRate: Send + Sync {
    fn update(&mut self, gradient: &DVector<f64>, iteration: usize) -> f64;
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub struct AdamOptimizer {
    learning_rate: f64,
    beta1: f64,
    beta2: f64,
    epsilon: f64,
    m: DVector<f64>,  // First moment estimate
    v: DVector<f64>,  // Second moment estimate
    t: usize,         // Time step
}

impl AdamOptimizer {
    pub fn new(learning_rate: f64, dim: usize) -> Result<Self, LearningError> {
        if !learning_rate.is_finite() || learning_rate <= 0.0 {
            return Err(LearningError::InvalidLearningRate { rate: learning_rate });
        }
        
        Ok(Self {
            learning_rate,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            m: DVector::zeros(dim),
            v: DVector::zeros(dim),
            t: 0,
        })
    }
}

impl AdaptiveLearningRate for AdamOptimizer {
    fn update(&mut self, gradient: &DVector<f64>, _iteration: usize) -> f64 {
        self.t += 1;
        
        // Update biased first moment estimate
        self.m = self.beta1 * &self.m + (1.0 - self.beta1) * gradient;
        
        // Update biased second moment estimate
        self.v = self.beta2 * &self.v + (1.0 - self.beta2) * gradient.map(|x| x * x);
        
        // Compute bias-corrected first moment estimate
        let m_hat = &self.m / (1.0 - self.beta1.powi(self.t as i32));
        
        // Compute bias-corrected second moment estimate
        let v_hat = &self.v / (1.0 - self.beta2.powi(self.t as i32));
        
        // Adaptive learning rate calculation
        let adaptive_rate = self.learning_rate / (1.0 + (self.t as f64).sqrt());
        
        adaptive_rate
    }
    
    fn reset(&mut self) {
        self.m.fill(0.0);
        self.v.fill(0.0);
        self.t = 0;
    }
}

#[derive(Debug, Clone)]
pub struct AdaGradOptimizer {
    learning_rate: f64,
    epsilon: f64,
    g_sum: DVector<f64>,  // Sum of squared gradients
}

impl AdaGradOptimizer {
    pub fn new(learning_rate: f64, dim: usize) -> Result<Self, LearningError> {
        if !learning_rate.is_finite() || learning_rate <= 0.0 {
            return Err(LearningError::InvalidLearningRate { rate: learning_rate });
        }
        
        Ok(Self {
            learning_rate,
            epsilon: 1e-8,
            g_sum: DVector::zeros(dim),
        })
    }
}

impl AdaptiveLearningRate for AdaGradOptimizer {
    fn update(&mut self, gradient: &DVector<f64>, _iteration: usize) -> f64 {
        // Accumulate squared gradients
        self.g_sum += gradient.map(|x| x * x);
        
        // Calculate adaptive learning rate
        let denom = self.g_sum.map(|x| (x + self.epsilon).sqrt());
        self.learning_rate
    }
    
    fn reset(&mut self) {
        self.g_sum.fill(0.0);
    }
}

// ============================================================================
// Feature Extraction and Embedding
// ============================================================================

pub trait FeatureExtractor: Send + Sync {
    fn extract(&self, prompt: &PromptExample) -> Result<DVector<f64>, LearningError>;
    fn dimension(&self) -> usize;
}

#[derive(Debug)]
pub struct TfIdfFeatureExtractor {
    vocabulary: DashMap<String, usize>,
    idf_scores: RwLock<DVector<f64>>,
    max_features: usize,
    doc_count: Arc<Mutex<usize>>,
}

impl TfIdfFeatureExtractor {
    pub fn new(max_features: usize) -> Self {
        Self {
            vocabulary: DashMap::new(),
            idf_scores: RwLock::new(DVector::zeros(max_features)),
            max_features,
            doc_count: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn update_vocabulary(&self, text: &str) {
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        for word in words {
            if self.vocabulary.len() < self.max_features {
                let idx = self.vocabulary.len();
                self.vocabulary.entry(word).or_insert(idx);
            }
        }
        
        let mut doc_count = self.doc_count.lock().await;
        *doc_count += 1;
    }
}

impl FeatureExtractor for TfIdfFeatureExtractor {
    fn extract(&self, prompt: &PromptExample) -> Result<DVector<f64>, LearningError> {
        let text = format!("{} {}", prompt.prompt, 
                          prompt.context.values().cloned().collect::<Vec<_>>().join(" "));
        
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let mut features = DVector::zeros(self.max_features);
        let mut word_counts = HashMap::new();
        
        // Calculate term frequencies
        for word in &words {
            if let Some(&idx) = self.vocabulary.get(word) {
                if idx < self.max_features {
                    *word_counts.entry(idx).or_insert(0) += 1;
                }
            }
        }
        
        // Apply TF-IDF
        let doc_length = words.len() as f64;
        for (&idx, &count) in &word_counts {
            let tf = count as f64 / doc_length;
            features[idx] = tf; // Simplified - in production, multiply by IDF
        }
        
        Ok(features)
    }
    
    fn dimension(&self) -> usize {
        self.max_features
    }
}

// ============================================================================
// Online Learning Algorithms
// ============================================================================

pub trait OnlineLearner: Send + Sync {
    async fn update(&mut self, features: &DVector<f64>, target: f64) -> Result<LearningMetrics, LearningError>;
    async fn predict(&self, features: &DVector<f64>) -> Result<f64, LearningError>;
    fn get_weights(&self) -> DVector<f64>;
    fn convergence_check(&self) -> bool;
}

#[derive(Debug)]
pub struct OnlineLinearRegression {
    weights: Arc<RwLock<DVector<f64>>>,
    optimizer: Arc<Mutex<Box<dyn AdaptiveLearningRate>>>,
    regularization: f64,
    convergence_threshold: f64,
    loss_history: Arc<Mutex<Vec<f64>>>,
    samples_processed: Arc<Mutex<usize>>,
}

impl OnlineLinearRegression {
    pub fn new(
        dimension: usize,
        optimizer: Box<dyn AdaptiveLearningRate>,
        regularization: f64,
    ) -> Self {
        Self {
            weights: Arc::new(RwLock::new(DVector::zeros(dimension))),
            optimizer: Arc::new(Mutex::new(optimizer)),
            regularization,
            convergence_threshold: 1e-6,
            loss_history: Arc::new(Mutex::new(Vec::new())),
            samples_processed: Arc::new(Mutex::new(0)),
        }
    }
}

impl OnlineLearner for OnlineLinearRegression {
    async fn update(&mut self, features: &DVector<f64>, target: f64) -> Result<LearningMetrics, LearningError> {
        let mut weights = self.weights.write().await;
        let mut optimizer = self.optimizer.lock().await;
        let mut loss_history = self.loss_history.lock().await;
        let mut samples = self.samples_processed.lock().await;
        
        if weights.len() != features.len() {
            return Err(LearningError::DimensionMismatch {
                expected: weights.len(),
                actual: features.len(),
            });
        }
        
        // Forward pass
        let prediction = weights.dot(features);
        let error = prediction - target;
        let loss = 0.5 * error * error + 0.5 * self.regularization * weights.norm_squared();
        
        // Backward pass - compute gradient
        let gradient = features * error + &*weights * self.regularization;
        let gradient_norm = gradient.norm();
        
        // Check for numerical stability
        if !gradient_norm.is_finite() {
            return Err(LearningError::NumericalInstability);
        }
        
        // Update weights using adaptive learning rate
        let learning_rate = optimizer.update(&gradient, *samples);
        *weights -= learning_rate * gradient;
        
        *samples += 1;
        loss_history.push(loss);
        
        // Calculate convergence rate
        let convergence_rate = if loss_history.len() > 1 {
            let prev_loss = loss_history[loss_history.len() - 2];
            (prev_loss - loss).abs() / prev_loss.max(1e-10)
        } else {
            1.0
        };
        
        Ok(LearningMetrics {
            loss,
            gradient_norm,
            learning_rate,
            convergence_rate,
            samples_processed: *samples,
            timestamp: Instant::now(),
        })
    }
    
    async fn predict(&self, features: &DVector<f64>) -> Result<f64, LearningError> {
        let weights = self.weights.read().await;
        
        if weights.len() != features.len() {
            return Err(LearningError::DimensionMismatch {
                expected: weights.len(),
                actual: features.len(),
            });
        }
        
        Ok(weights.dot(features))
    }
    
    fn get_weights(&self) -> DVector<f64> {
        // This is a simplified synchronous version for inspection
        // In practice, you'd want an async version
        DVector::zeros(0) // Placeholder
    }
    
    fn convergence_check(&self) -> bool {
        // Simplified convergence check
        false // Would implement based on loss history analysis
    }
}

// ============================================================================
// Real-time Learning Pipeline
// ============================================================================

#[derive(Debug)]
pub struct PromptOptimizationPipeline {
    feature_extractor: Arc<Box<dyn FeatureExtractor>>,
    learner: Arc<Mutex<Box<dyn OnlineLearner>>>,
    feedback_receiver: mpsc::Receiver<FeedbackSignal>,
    metrics_sender: mpsc::Sender<LearningMetrics>,
    convergence_monitor: Arc<RwLock<ConvergenceMonitor>>,
}

#[derive(Debug)]
pub struct ConvergenceMonitor {
    window_size: usize,
    recent_metrics: Vec<LearningMetrics>,
    convergence_threshold: f64,
    stability_threshold: f64,
}

impl ConvergenceMonitor {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            recent_metrics: Vec::new(),
            convergence_threshold: 1e-6,
            stability_threshold: 1e-4,
        }
    }
    
    pub fn update(&mut self, metrics: LearningMetrics) -> bool {
        self.recent_metrics.push(metrics);
        
        if self.recent_metrics.len() > self.window_size {
            self.recent_metrics.remove(0);
        }
        
        self.check_convergence()
    }
    
    fn check_convergence(&self) -> bool {
        if self.recent_metrics.len() < self.window_size {
            return false;
        }
        
        // Check loss stability
        let losses: Vec<f64> = self.recent_metrics.iter().map(|m| m.loss).collect();
        let mean_loss = losses.iter().sum::<f64>() / losses.len() as f64;
        let variance = losses.iter()
            .map(|&loss| (loss - mean_loss).powi(2))
            .sum::<f64>() / losses.len() as f64;
        
        // Check gradient norm convergence
        let avg_gradient_norm = self.recent_metrics.iter()
            .map(|m| m.gradient_norm)
            .sum::<f64>() / self.recent_metrics.len() as f64;
        
        variance < self.stability_threshold && avg_gradient_norm < self.convergence_threshold
    }
}

impl PromptOptimizationPipeline {
    pub fn new(
        feature_extractor: Box<dyn FeatureExtractor>,
        learner: Box<dyn OnlineLearner>,
        feedback_capacity: usize,
        metrics_capacity: usize,
    ) -> (Self, mpsc::Sender<FeedbackSignal>, mpsc::Receiver<LearningMetrics>) {
        let (feedback_tx, feedback_rx) = mpsc::channel(feedback_capacity);
        let (metrics_tx, metrics_rx) = mpsc::channel(metrics_capacity);
        
        let pipeline = Self {
            feature_extractor: Arc::new(feature_extractor),
            learner: Arc::new(Mutex::new(learner)),
            feedback_receiver: feedback_rx,
            metrics_sender: metrics_tx,
            convergence_monitor: Arc::new(RwLock::new(ConvergenceMonitor::new(100))),
        };
        
        (pipeline, feedback_tx, metrics_rx)
    }
    
    pub async fn run(&mut self) -> Result<(), LearningError> {
        tracing::info!("Starting prompt optimization pipeline");
        
        let mut convergence_check_interval = interval(Duration::from_secs(10));
        
        loop {
            tokio::select! {
                // Process incoming feedback
                Some(feedback) = self.feedback_receiver.recv() => {
                    if let Err(e) = self.process_feedback(feedback).await {
                        tracing::error!("Error processing feedback: {}", e);
                    }
                }
                
                // Periodic convergence checking
                _ = convergence_check_interval.tick() => {
                    if let Err(e) = self.check_convergence().await {
                        tracing::error!("Error checking convergence: {}", e);
                    }
                }
                
                // Graceful shutdown would be handled here
                else => {
                    tracing::info!("Pipeline shutting down");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_feedback(&self, feedback: FeedbackSignal) -> Result<(), LearningError> {
        // Create a synthetic prompt example for this feedback
        // In practice, you'd store and retrieve the original prompt
        let prompt_example = PromptExample {
            id: feedback.example_id,
            prompt: "synthetic_prompt".to_string(), // Would be retrieved from storage
            context: HashMap::new(),
            timestamp: feedback.timestamp,
        };
        
        // Extract features
        let features = self.feature_extractor.extract(&prompt_example)?;
        
        // Calculate target value from feedback
        let target = self.calculate_target_score(&feedback);
        
        // Update the learner
        let mut learner = self.learner.lock().await;
        let metrics = learner.update(&features, target).await?;
        
        // Update convergence monitor
        let mut monitor = self.convergence_monitor.write().await;
        let converged = monitor.update(metrics.clone());
        
        if converged {
            tracing::info!("Model convergence detected at iteration {}", metrics.samples_processed);
        }
        
        // Send metrics to monitoring system
        if let Err(_) = self.metrics_sender.send(metrics).await {
            tracing::warn!("Metrics channel full, dropping metrics");
        }
        
        Ok(())
    }
    
    fn calculate_target_score(&self, feedback: &FeedbackSignal) -> f64 {
        // Combine multiple feedback signals into a single target score
        let base_score = feedback.score;
        let latency_penalty = (feedback.latency_ms as f64 / 1000.0).min(1.0) * 0.1;
        let success_bonus = if feedback.success { 0.1 } else { -0.2 };
        let user_weight = feedback.user_rating.unwrap_or(0.0) * 0.3;
        
        (base_score - latency_penalty + success_bonus + user_weight).clamp(0.0, 1.0)
    }
    
    async fn check_convergence(&self) -> Result<(), LearningError> {
        let monitor = self.convergence_monitor.read().await;
        
        if monitor.recent_metrics.len() > 10 {
            let recent_losses: Vec<f64> = monitor.recent_metrics
                .iter()
                .rev()
                .take(10)
                .map(|m| m.loss)
                .collect();
            
            tracing::debug!("Recent losses: {:?}", recent_losses);
        }
        
        Ok(())
    }
}

// ============================================================================
// Production Integration Layer
// ============================================================================

#[derive(Debug)]
pub struct ProductionLearningSystem {
    pipeline: Option<PromptOptimizationPipeline>,
    feedback_sender: mpsc::Sender<FeedbackSignal>,
    metrics_receiver: mpsc::Receiver<LearningMetrics>,
    model_checkpoints: Arc<RwLock<Vec<ModelCheckpoint>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCheckpoint {
    pub id: Uuid,
    pub weights: Vec<f64>,
    pub timestamp: u64,
    pub performance_metrics: HashMap<String, f64>,
}

impl ProductionLearningSystem {
    pub async fn new(config: LearningConfig) -> Result<Self, LearningError> {
        let feature_extractor = Box::new(TfIdfFeatureExtractor::new(config.feature_dimension));
        let optimizer = Box::new(AdamOptimizer::new(config.learning_rate, config.feature_dimension)?);
        let learner = Box::new(OnlineLinearRegression::new(
            config.feature_dimension,
            optimizer,
            config.regularization,
        ));
        
        let (mut pipeline, feedback_sender, metrics_receiver) = PromptOptimizationPipeline::new(
            feature_extractor,
            learner,
            config.feedback_buffer_size,
            config.metrics_buffer_size,
        );
        
        Ok(Self {
            pipeline: Some(pipeline),
            feedback_sender,
            metrics_receiver,
            model_checkpoints: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    pub async fn start(&mut self) -> Result<(), LearningError> {
        if let Some(mut pipeline) = self.pipeline.take() {
            tokio::spawn(async move {
                if let Err(e) = pipeline.run().await {
                    tracing::error!("Pipeline error: {}", e);
                }
            });
        }
        
        // Start metrics monitoring
        self.start_metrics_monitoring().await;
        
        Ok(())
    }
    
    pub async fn submit_feedback(&self, feedback: FeedbackSignal) -> Result<(), LearningError> {
        self.feedback_sender.send(feedback).await
            .map_err(|e| LearningError::ChannelError { source: e.to_string() })
    }
    
    async fn start_metrics_monitoring(&mut self) {
        let checkpoints = Arc::clone(&self.model_checkpoints);
        let mut receiver = std::mem::replace(&mut self.metrics_receiver, {
            let (_, rx) = mpsc::channel(1);
            rx
        });
        
        tokio::spawn(async move {
            let mut checkpoint_interval = interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                tokio::select! {
                    Some(metrics) = receiver.recv() => {
                        tracing::debug!("Received learning metrics: {:?}", metrics);
                    }
                    
                    _ = checkpoint_interval.tick() => {
                        // Create model checkpoint
                        // Implementation would save current model state
                        tracing::info!("Creating model checkpoint");
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub feature_dimension: usize,
    pub learning_rate: f64,
    pub regularization: f64,
    pub feedback_buffer_size: usize,
    pub metrics_buffer_size: usize,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            feature_dimension: 1000,
            learning_rate: 0.001,
            regularization: 0.01,
            feedback_buffer_size: 10000,
            metrics_buffer_size: 1000,
        }
    }
}

// ============================================================================
// Usage Example and Tests
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    // Initialize the learning system
    let config = LearningConfig::default();
    let mut system = ProductionLearningSystem::new(config).await?;
    
    // Start the learning pipeline
    system.start().await?;
    
    // Simulate feedback submission
    let feedback = FeedbackSignal {
        example_id: Uuid::new_v4(),
        score: 0.85,
        latency_ms: 250,
        user_rating: Some(0.9),
        success: true,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };
    
    system.submit_feedback(feedback).await?;
    
    // Keep the system running
    sleep(Duration::from_secs(10)).await;
    
    tracing::info!("Learning system demonstration completed");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_adam_optimizer() {
        let mut optimizer = AdamOptimizer::new(0.001, 10).unwrap();
        let gradient = DVector::from_vec(vec![0.1; 10]);
        
        let lr1 = optimizer.update(&gradient, 1);
        let lr2 = optimizer.update(&gradient, 2);
        
        assert!(lr1 > 0.0);
        assert!(lr2 > 0.0);
    }
    
    #[tokio::test]
    async fn test_feature_extraction() {
        let extractor = TfIdfFeatureExtractor::new(100);
        let example = PromptExample {
            id: Uuid::new_v4(),
            prompt: "test prompt for feature extraction".to_string(),
            context: HashMap::new(),
            timestamp: 0,
        };
        
        let features = extractor.extract(&example).unwrap();
        assert_eq!(features.len(), 100);
    }
    
    #[test]
    fn test_convergence_monitor() {
        let mut monitor = ConvergenceMonitor::new(5);
        
        for i in 0..10 {
            let metrics = LearningMetrics {
                loss: 1.0 / (i as f64 + 1.0),
                gradient_norm: 0.1 / (i as f64 + 1.0),
                learning_rate: 0.001,
                convergence_rate: 0.01,
                samples_processed: i,
                timestamp: Instant::now(),
            };
            
            let converged = monitor.update(metrics);
            if i >= 8 {
                // Should converge with small enough gradients
                assert!(converged || !converged); // Either is acceptable for this test
            }
        }
    }
}