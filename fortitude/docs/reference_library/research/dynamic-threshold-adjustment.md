# Dynamic Threshold Adjustment for Classification Systems

<meta>
  <title>Dynamic Threshold Adjustment for Classification Systems</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Real-time threshold optimization for classification systems based on performance feedback
- **Key Approach**: Adaptive threshold manager with multiple adjustment strategies and continuous monitoring
- **Core Benefits**: Self-improving classification accuracy, automatic threshold tuning, performance monitoring
- **When to use**: Production classification systems requiring continuous performance optimization
- **Related docs**: [Rule Composition Algorithms](rule-composition-algorithms.md), [Semantic Analysis Integration](semantic-analysis-integration.md)

## <implementation>Core Implementation Pattern</implementation>

### <pattern>Adaptive Threshold Manager</pattern>

The foundation provides real-time threshold adjustment based on classification performance feedback:

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub confidence: f64,
    pub predicted_class: bool,
    pub actual_class: Option<bool>,
    pub timestamp: Instant,
}

pub struct AdaptiveThresholdManager {
    threshold_config: ThresholdConfig,
    current_threshold: f64,
    performance_history: Vec<PerformanceMetrics>,
    adjustment_strategy: String,
    monitoring_active: bool,
}

#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    pub initial_threshold: f64,
    pub min_threshold: f64,
    pub max_threshold: f64,
    pub adjustment_rate: f64,
    pub evaluation_window: usize,
    pub min_samples_for_adjustment: usize,
    pub target_metric: String,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            initial_threshold: 0.5,
            min_threshold: 0.1,
            max_threshold: 0.9,
            adjustment_rate: 0.05,
            evaluation_window: 50,
            min_samples_for_adjustment: 10,
            target_metric: "f1".to_string(),
        }
    }
}

impl AdaptiveThresholdManager {
    pub fn new(config: ThresholdConfig, strategy: &str) -> Self {
        Self {
            current_threshold: config.initial_threshold,
            threshold_config: config,
            performance_history: Vec::new(),
            adjustment_strategy: strategy.to_string(),
            monitoring_active: false,
        }
    }

    pub fn process_result(&mut self, result: ClassificationResult) -> Result<(), ThresholdError> {
        // Update performance metrics
        self.update_performance_metrics(result)?;
        
        // Check if adjustment is needed
        if self.should_adjust_threshold()? {
            self.adjust_threshold()?;
        }
        
        Ok(())
    }

    pub fn get_current_threshold(&self) -> f64 {
        self.current_threshold
    }

    pub fn start_monitoring(&mut self, check_interval: Duration) -> Result<(), ThresholdError> {
        self.monitoring_active = true;
        Ok(())
    }

    pub fn stop_monitoring(&mut self) -> Result<(), ThresholdError> {
        self.monitoring_active = false;
        Ok(())
    }
}
```

### <pattern>Classification Pipeline Integration</pattern>

Production-ready pipeline that integrates threshold adjustment with classification processing:

```rust
pub struct ClassificationPipeline {
    threshold_manager: Arc<Mutex<AdaptiveThresholdManager>>,
    prediction_sender: Sender<ModelPrediction>,
    feedback_sender: Sender<ClassificationFeedback>,
    pipeline_active: Arc<Mutex<bool>>,
    stats: Arc<Mutex<PipelineStats>>,
}

#[derive(Debug, Clone)]
pub struct ModelPrediction {
    pub features: Vec<f64>,
    pub confidence_scores: Vec<f64>,
    pub primary_confidence: f64,
    pub model_id: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct ClassificationFeedback {
    pub prediction_id: String,
    pub actual_class: bool,
    pub confidence: f64,
    pub timestamp: Instant,
}

impl ClassificationPipeline {
    pub fn new(threshold_config: ThresholdConfig, strategy: &str) -> Result<Self, ThresholdError> {
        let mut manager = AdaptiveThresholdManager::new(threshold_config, strategy);
        manager.enable_auto_strategy_switching();
        manager.start_monitoring(Duration::from_secs(10))?;
        
        let threshold_manager = Arc::new(Mutex::new(manager));
        let (prediction_sender, _) = mpsc::channel();
        let (feedback_sender, _) = mpsc::channel();
        let pipeline_active = Arc::new(Mutex::new(false));
        let stats = Arc::new(Mutex::new(PipelineStats::default()));

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

        // Start processing threads
        self.start_prediction_processor(pred_rx)?;
        self.start_feedback_processor(feedback_rx)?;

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
}
```

## <examples>Usage Examples</examples>

### <template>Pipeline Simulation</template>

```rust
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

        while start_time.elapsed() < duration && self.simulation_active {
            // Generate synthetic prediction
            let confidence = self.generate_realistic_confidence();
            let prediction = ModelPrediction {
                features: vec![0.1, 0.2, 0.3],
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

        self.print_final_stats()?;
        self.pipeline.stop_pipeline()?;
        
        Ok(())
    }

    fn generate_realistic_confidence(&self) -> f64 {
        use std::f64::consts::PI;
        
        let random_val: f64 = (Instant::now().elapsed().as_nanos() % 1000) as f64 / 1000.0;
        let base_confidence = 0.5 + 0.3 * (2.0 * PI * random_val).sin();
        
        let noise = 0.1 * (random_val - 0.5);
        (base_confidence + noise).max(0.0).min(1.0)
    }
}
```

### <template>Configuration Examples</template>

```rust
pub fn run_example() -> Result<(), ThresholdError> {
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
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### <issue>Threshold Oscillation</issue>
- **Problem**: Threshold continuously adjusts without stabilizing
- **Solution**: Increase `min_samples_for_adjustment` and reduce `adjustment_rate`
- **Pattern**: Use exponential decay for adjustment rate over time

### <issue>Performance Degradation</issue>
- **Problem**: System performance drops with threshold adjustment enabled
- **Solution**: Implement asynchronous processing and batch feedback processing
- **Pattern**: Use separate threads for prediction and feedback processing

### <issue>Memory Leaks in Long-Running Systems</issue>
- **Problem**: Performance history grows indefinitely
- **Solution**: Implement sliding window for performance metrics
- **Pattern**: Use bounded collections with automatic cleanup

## <references>See Also</references>

- [Rule Composition Algorithms](rule-composition-algorithms.md) - Multi-dimensional signal combination
- [Semantic Analysis Integration](semantic-analysis-integration.md) - Context-aware classification
- [Classification Patterns](../patterns/classification-algorithms.md) - Basic classification implementation
- [Performance Monitoring](../patterns/performance-monitoring.md) - System monitoring patterns

## <integration>Integration with Fortitude</integration>

### <fortitude-integration>Sprint 004 Application</fortitude-integration>

For Sprint 004 (Advanced Classification), this research enables:

1. **Confidence Score Adaptation**: Dynamic adjustment of classification confidence thresholds
2. **Performance Monitoring**: Real-time tracking of classification accuracy
3. **Strategy Switching**: Automatic fallback between classification strategies
4. **Feedback Loop**: Continuous improvement based on classification results

```rust
// Integration pattern for Fortitude classification system
use fortitude_core::classification::AdvancedClassifier;

impl AdvancedClassifier {
    pub fn with_adaptive_thresholds(mut self, config: ThresholdConfig) -> Self {
        self.threshold_manager = Some(AdaptiveThresholdManager::new(config, "balanced"));
        self
    }

    pub fn process_with_feedback(&mut self, request: ClassifiedRequest) -> Result<ClassificationResult, ClassificationError> {
        let result = self.classify_request(request)?;
        
        // Update threshold based on result confidence
        if let Some(ref mut manager) = self.threshold_manager {
            manager.process_result(ClassificationResult {
                confidence: result.confidence,
                predicted_class: result.research_type != ResearchType::Learning,
                actual_class: None, // Will be provided via feedback
                timestamp: Instant::now(),
            })?;
        }
        
        Ok(result)
    }
}
```

**Success Metrics**:
- Classification accuracy improvement >15%
- Threshold convergence within 100 samples
- Performance monitoring overhead <5ms per classification
- Memory usage bounded to <10MB for 24-hour operation