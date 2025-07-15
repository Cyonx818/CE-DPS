use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};

// Import our threshold adjustment system
use crate::{
    AdaptiveThresholdManager, ClassificationResult, ThresholdConfig, ThresholdError,
    PerformanceMetrics
};

// Simulated ML model output
#[derive(Debug, Clone)]
pub struct ModelPrediction {
    pub features: Vec<f64>,
    pub confidence_scores: Vec<f64>, // Multiple class probabilities
    pub primary_confidence: f64,     // Confidence for binary classification
    pub model_id: String,
    pub timestamp: Instant,
}

// Classification pipeline that uses dynamic thresholds
pub struct ClassificationPipeline {
    threshold_manager: Arc<Mutex<AdaptiveThresholdManager>>,
    prediction_sender: Sender<ModelPrediction>,
    feedback_sender: Sender<ClassificationFeedback>,
    pipeline_active: Arc<Mutex<bool>>,
    stats: Arc<Mutex<PipelineStats>>,
}

#[derive(Debug, Clone)]
pub struct ClassificationFeedback {
    pub prediction_id: String,
    pub actual_class: bool,
    pub confidence: f64,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_predictions: u64,
    pub total_feedback: u64,
    pub current_threshold: f64,
    pub current_strategy: String,
    pub avg_confidence: f64,
    pub recent_accuracy: f64,
}

impl ClassificationPipeline {
    pub fn new(threshold_config: ThresholdConfig, strategy: &str) -> Result<Self, ThresholdError> {
        let mut manager = AdaptiveThresholdManager::new(threshold_config, strategy);
        manager.enable_auto_strategy_switching();
        
        // Start performance monitoring
        manager.start_monitoring(Duration::from_secs(10))?;
        
        let threshold_manager = Arc::new(Mutex::new(manager));
        let (prediction_sender, _) = mpsc::channel();
        let (feedback_sender, _) = mpsc::channel();
        let pipeline_active = Arc::new(Mutex::new(false));
        let stats = Arc::new(Mutex::new(PipelineStats {
            total_predictions: 0,
            total_feedback: 0,
            current_threshold: 0.5,
            current_strategy: strategy.to_string(),
            avg_confidence: 0.0,
            recent_accuracy: 0.0,
        }));

        Ok(Self {
            threshold_manager,
            prediction_sender,
            feedback_sender,
            pipeline_active,
            stats,
        })
    }

    pub fn start_pipeline(&mut self) -> Result<(), ThresholdError> {
        let mut active = self.pipeline_active.lock()
            .map_err(|e| ThresholdError::MonitoringError(format!("Lock error: {}", e)))?;
        
        if *active {
            return Err(ThresholdError::MonitoringError("Pipeline already active".to_string()));
        }
        
        *active = true;
        drop(active);

        // Create channels for pipeline communication
        let (pred_tx, pred_rx) = mpsc::channel::<ModelPrediction>();
        let (feedback_tx, feedback_rx) = mpsc::channel::<ClassificationFeedback>();
        
        self.prediction_sender = pred_tx;
        self.feedback_sender = feedback_tx;

        // Start prediction processing thread
        self.start_prediction_processor(pred_rx)?;
        
        // Start feedback processing thread
        self.start_feedback_processor(feedback_rx)?;

        println!("Classification pipeline started successfully");
        Ok(())
    }

    fn start_prediction_processor(&self, receiver: Receiver<ModelPrediction>) -> Result<(), ThresholdError> {
        let threshold_manager = Arc::clone(&self.threshold_manager);
        let stats = Arc::clone(&self.stats);
        let active = Arc::clone(&self.pipeline_active);

        thread::spawn(move || {
            let mut prediction_count = 0u64;
            let mut confidence_sum = 0.0;

            while let Ok(prediction) = receiver.recv() {
                // Check if pipeline is still active
                {
                    let is_active = active.lock().unwrap();
                    if !*is_active {
                        break;
                    }
                }

                // Get current threshold
                let current_threshold = {
                    let manager = threshold_manager.lock().unwrap();
                    manager.get_current_threshold()
                };

                // Make classification decision
                let classification = prediction.primary_confidence >= current_threshold;
                
                // Update statistics
                prediction_count += 1;
                confidence_sum += prediction.primary_confidence;
                
                {
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.total_predictions = prediction_count;
                    stats_guard.current_threshold = current_threshold;
                    stats_guard.avg_confidence = confidence_sum / prediction_count as f64;
                    
                    let manager = threshold_manager.lock().unwrap();
                    stats_guard.current_strategy = manager.get_current_strategy().to_string();
                }

                // Log prediction
                println!(
                    "Prediction {}: confidence={:.3}, threshold={:.3}, decision={}",
                    prediction_count,
                    prediction.primary_confidence,
                    current_threshold,
                    classification
                );

                // Simulate processing delay
                thread::sleep(Duration::from_millis(10));
            }
        });

        Ok(())
    }

    fn start_feedback_processor(&self, receiver: Receiver<ClassificationFeedback>) -> Result<(), ThresholdError> {
        let threshold_manager = Arc::clone(&self.threshold_manager);
        let stats = Arc::clone(&self.stats);
        let active = Arc::clone(&self.pipeline_active);

        thread::spawn(move || {
            let mut feedback_count = 0u64;
            let mut correct_predictions = 0u64;

            while let Ok(feedback) = receiver.recv() {
                // Check if pipeline is still active
                {
                    let is_active = active.lock().unwrap();
                    if !*is_active {
                        break;
                    }
                }

                // Create classification result
                let current_threshold = {
                    let manager = threshold_manager.lock().unwrap();
                    manager.get_current_threshold()
                };

                let predicted_class = feedback.confidence >= current_threshold;
                let result = ClassificationResult {
                    confidence: feedback.confidence,
                    predicted_class,
                    actual_class: Some(feedback.actual_class),
                    timestamp: feedback.timestamp,
                };

                // Update threshold manager
                {
                    let mut manager = threshold_manager.lock().unwrap();
                    if let Err(e) = manager.process_result(result) {
                        eprintln!("Error processing feedback: {}", e);
                        continue;
                    }
                }

                // Update accuracy statistics
                feedback_count += 1;
                if predicted_class == feedback.actual_class {
                    correct_predictions += 1;
                }

                let current_accuracy = correct_predictions as f64 / feedback_count as f64;

                {
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.total_feedback = feedback_count;
                    stats_guard.recent_accuracy = current_accuracy;
                }

                println!(
                    "Feedback {}: actual={}, predicted={}, accuracy={:.3}",
                    feedback_count,
                    feedback.actual_class,
                    predicted_class,
                    current_accuracy
                );
            }
        });

        Ok(())
    }

    pub fn process_prediction(&self, prediction: ModelPrediction) -> Result<(), ThresholdError> {
        self.prediction_sender.send(prediction)
            .map_err(|e| ThresholdError::MonitoringError(format!("Failed to send prediction: {}", e)))
    }

    pub fn submit_feedback(&self, feedback: ClassificationFeedback) -> Result<(), ThresholdError> {
        self.feedback_sender.send(feedback)
            .map_err(|e| ThresholdError::MonitoringError(format!("Failed to send feedback: {}", e)))
    }

    pub fn get_stats(&self) -> Result<PipelineStats, ThresholdError> {
        let stats = self.stats.lock()
            .map_err(|e| ThresholdError::MonitoringError(format!("Lock error: {}", e)))?;
        Ok(stats.clone())
    }

    pub fn get_performance_metrics(&self) -> Result<Option<PerformanceMetrics>, ThresholdError> {
        let manager = self.threshold_manager.lock()
            .map_err(|e| ThresholdError::MonitoringError(format!("Lock error: {}", e)))?;
        Ok(manager.get_performance_summary())
    }

    pub fn stop_pipeline(&mut self) -> Result<(), ThresholdError> {
        {
            let mut active = self.pipeline_active.lock()
                .map_err(|e| ThresholdError::MonitoringError(format!("Lock error: {}", e)))?;
            *active = false;
        }

        // Stop threshold manager monitoring
        {
            let mut manager = self.threshold_manager.lock()
                .map_err(|e| ThresholdError::MonitoringError(format!("Lock error: {}", e)))?;
            manager.stop_monitoring()?;
        }

        println!("Classification pipeline stopped");
        Ok(())
    }

    pub fn force_threshold_update(&self, new_threshold: f64) -> Result<(), ThresholdError> {
        if new_threshold < 0.0 || new_threshold > 1.0 {
            return Err(ThresholdError::InvalidThreshold(new_threshold));
        }

        // This would require extending the threshold manager to support manual overrides
        println!("Manual threshold override to {:.3} requested", new_threshold);
        Ok(())
    }
}

// Simulation utilities for testing
pub struct PipelineSimulator {
    pipeline: ClassificationPipeline,
    simulation_active: bool,
}

impl PipelineSimulator {
    pub fn new(config: ThresholdConfig) -> Result<Self, ThresholdError> {
        let pipeline = ClassificationPipeline::new(config, "moving_average")?;
        
        Ok(Self {
            pipeline,
            simulation_active: false,
        })
    }

    pub fn run_simulation(&mut self, duration: Duration, prediction_rate: f64) -> Result<(), ThresholdError> {
        self.pipeline.start_pipeline()?;
        self.simulation_active = true;

        let prediction_interval = Duration::from_millis((1000.0 / prediction_rate) as u64);
        let start_time = Instant::now();
        let mut prediction_id = 0u64;

        println!("Starting simulation for {:?} at {:.1} predictions/sec", duration, prediction_rate);

        while start_time.elapsed() < duration && self.simulation_active {
            // Generate synthetic prediction
            let confidence = self.generate_realistic_confidence();
            let prediction = ModelPrediction {
                features: vec![0.1, 0.2, 0.3], // Dummy features
                confidence_scores: vec![1.0 - confidence, confidence],
                primary_confidence: confidence,
                model_id: "test_model_v1".to_string(),
                timestamp: Instant::now(),
            };

            // Submit prediction
            self.pipeline.process_prediction(prediction)?;

            // Simulate delayed feedback (20% of predictions get feedback)
            if prediction_id % 5 == 0 {
                self.simulate_delayed_feedback(prediction_id, confidence)?;
            }

            prediction_id += 1;
            thread::sleep(prediction_interval);
        }

        // Print final statistics
        self.print_final_stats()?;
        self.pipeline.stop_pipeline()?;
        
        Ok(())
    }

    fn generate_realistic_confidence(&self) -> f64 {
        // Generate confidence scores that simulate real model behavior
        use std::f64::consts::PI;
        
        let random_val: f64 = (Instant::now().elapsed().as_nanos() % 1000) as f64 / 1000.0;
        let base_confidence = 0.5 + 0.3 * (2.0 * PI * random_val).sin();
        
        // Add some noise
        let noise = 0.1 * (random_val - 0.5);
        (base_confidence + noise).max(0.0).min(1.0)
    }

    fn simulate_delayed_feedback(&self, prediction_id: u64, confidence: f64) -> Result<(), ThresholdError> {
        let pipeline = &self.pipeline;
        
        // Simulate feedback delay
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(500 + (prediction_id % 1000)));
            
            // Generate ground truth (biased towards high confidence being correct)
            let actual_class = confidence > 0.6 || (confidence > 0.4 && prediction_id % 3 == 0);
            
            let feedback = ClassificationFeedback {
                prediction_id: format!("pred_{}", prediction_id),
                actual_class,
                confidence,
                timestamp: Instant::now(),
            };

            if let Err(e) = pipeline.submit_feedback(feedback) {
                eprintln!("Failed to submit feedback: {}", e);
            }
        });

        Ok(())
    }

    fn print_final_stats(&self) -> Result<(), ThresholdError> {
        let stats = self.pipeline.get_stats()?;
        let metrics = self.pipeline.get_performance_metrics()?;

        println!("\n=== SIMULATION RESULTS ===");
        println!("Total Predictions: {}", stats.total_predictions);
        println!("Total Feedback: {}", stats.total_feedback);
        println!("Final Threshold: {:.3}", stats.current_threshold);
        println!("Final Strategy: {}", stats.current_strategy);
        println!("Average Confidence: {:.3}", stats.avg_confidence);
        println!("Recent Accuracy: {:.3}", stats.recent_accuracy);

        if let Some(perf) = metrics {
            println!("=== PERFORMANCE METRICS ===");
            println!("Accuracy: {:.3}", perf.accuracy);
            println!("Precision: {:.3}", perf.precision);
            println!("Recall: {:.3}", perf.recall);
            println!("F1 Score: {:.3}", perf.f1_score);
            println!("True Positives: {}", perf.true_positives);
            println!("False Positives: {}", perf.false_positives);
            println!("True Negatives: {}", perf.true_negatives);
            println!("False Negatives: {}", perf.false_negatives);
        }

        Ok(())
    }

    pub fn stop_simulation(&mut self) {
        self.simulation_active = false;
    }
}

// Example main function demonstrating usage
pub fn run_example() -> Result<(), ThresholdError> {
    println!("Starting Dynamic Threshold Adjustment Example");

    // Configure threshold adjustment
    let config = ThresholdConfig {
        initial_threshold: 0.5,
        min_threshold: 0.2,
        max_threshold: 0.8,
        adjustment_rate: 0.05,
        evaluation_window: 50,
        min_samples_for_adjustment: 10,
        target_metric: "f1".to_string(),
    };

    // Create and run simulation
    let mut simulator = PipelineSimulator::new(config)?;
    
    // Run for 30 seconds at 5 predictions per second
    simulator.run_simulation(Duration::from_secs(30), 5.0)?;

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_pipeline_integration() {
        let config = ThresholdConfig::default();
        let mut pipeline = ClassificationPipeline::new(config, "moving_average").unwrap();
        
        pipeline.start_pipeline().unwrap();

        // Submit test predictions
        for i in 0..10 {
            let prediction = ModelPrediction {
                features: vec![0.1, 0.2],
                confidence_scores: vec![0.3, 0.7],
                primary_confidence: 0.6 + (i as f64 * 0.02),
                model_id: "test".to_string(),
                timestamp: Instant::now(),
            };
            
            pipeline.process_prediction(prediction).unwrap();
        }

        // Submit some feedback
        for i in 0..5 {
            let feedback = ClassificationFeedback {
                prediction_id: format!("test_{}", i),
                actual_class: i % 2 == 0,
                confidence: 0.6 + (i as f64 * 0.02),
                timestamp: Instant::now(),
            };
            
            pipeline.submit_feedback(feedback).unwrap();
        }

        // Wait for processing
        thread::sleep(Duration::from_millis(500));

        // Check stats
        let stats = pipeline.get_stats().unwrap();
        assert!(stats.total_predictions > 0);
        
        pipeline.stop_pipeline().unwrap();
    }

    #[test]
    fn test_simulator() {
        let config = ThresholdConfig::default();
        let mut simulator = PipelineSimulator::new(config).unwrap();
        
        // Run short simulation
        simulator.run_simulation(Duration::from_secs(2), 10.0).unwrap();
    }
}