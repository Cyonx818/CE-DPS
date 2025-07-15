# Rule Composition Algorithms for Multi-Dimensional Classification

<meta>
  <title>Rule Composition Algorithms for Multi-Dimensional Classification</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Algorithms for intelligently combining multiple classification signals with weighted scoring and conflict resolution
- **Key Approach**: Multi-dimensional signal composition with confidence weighting, conflict resolution, and rule prioritization
- **Core Benefits**: Improved classification accuracy, intelligent signal combination, conflict resolution, configurable weighting
- **When to use**: Classification systems requiring multiple input signals and context-aware decision making
- **Related docs**: [Dynamic Threshold Adjustment](dynamic-threshold-adjustment.md), [Semantic Analysis Integration](semantic-analysis-integration.md)

## <implementation>Core Implementation Pattern</implementation>

### <pattern>Classification Signal System</pattern>

Foundation for multi-dimensional classification with weighted signals:

```rust
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClassificationError {
    #[error("Invalid weight: {0} (must be between 0.0 and 1.0)")]
    InvalidWeight(f64),
    #[error("No signals provided for classification")]
    NoSignals,
    #[error("Conflicting signals with equal confidence: {0}")]
    UnresolvableConflict(String),
    #[error("Signal type mismatch: expected {expected}, got {actual}")]
    SignalTypeMismatch { expected: String, actual: String },
    #[error("Missing required dimension: {0}")]
    MissingDimension(String),
}

type Result<T> = std::result::Result<T, ClassificationError>;

// Core signal types for multi-dimensional classification
#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    ResearchType,
    Audience,
    Domain,
    Urgency,
}

#[derive(Debug, Clone)]
pub struct ClassificationSignal {
    pub signal_type: SignalType,
    pub value: String,
    pub confidence: f64,
    pub source: String,
    pub timestamp: u64,
}

impl ClassificationSignal {
    pub fn new(
        signal_type: SignalType,
        value: String,
        confidence: f64,
        source: String,
    ) -> Result<Self> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(ClassificationError::InvalidWeight(confidence));
        }
        
        Ok(Self {
            signal_type,
            value,
            confidence,
            source,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}
```

### <pattern>Weighted Rule System</pattern>

Configurable rule weighting and composition for signal combination:

```rust
// Weighted rule for signal combination
#[derive(Debug, Clone)]
pub struct WeightedRule {
    pub dimension: SignalType,
    pub weight: f64,
    pub required: bool,
    pub boost_factor: f64,
}

impl WeightedRule {
    pub fn new(dimension: SignalType, weight: f64, required: bool) -> Result<Self> {
        if !(0.0..=1.0).contains(&weight) {
            return Err(ClassificationError::InvalidWeight(weight));
        }
        
        Ok(Self {
            dimension,
            weight,
            required,
            boost_factor: 1.0,
        })
    }
    
    pub fn with_boost(mut self, boost_factor: f64) -> Self {
        self.boost_factor = boost_factor;
        self
    }
}

// Configuration for rule composition
#[derive(Debug, Clone)]
pub struct CompositionConfig {
    pub conflict_resolution: ConflictResolution,
    pub minimum_confidence: f64,
    pub decay_factor: f64,
    pub max_age_seconds: u64,
}

#[derive(Debug, Clone)]
pub enum ConflictResolution {
    HighestConfidence,
    WeightedAverage,
    MostRecent,
    ConsensusRequired(f64), // threshold for consensus
}

impl Default for CompositionConfig {
    fn default() -> Self {
        Self {
            conflict_resolution: ConflictResolution::HighestConfidence,
            minimum_confidence: 0.1,
            decay_factor: 0.95,
            max_age_seconds: 3600, // 1 hour
        }
    }
}
```

### <pattern>Rule Composition Engine</pattern>

Main engine for combining multiple classification signals:

```rust
// Main rule composition engine
pub struct RuleCompositionEngine {
    rules: Vec<WeightedRule>,
    config: CompositionConfig,
}

impl RuleCompositionEngine {
    pub fn new(config: CompositionConfig) -> Self {
        Self {
            rules: Vec::new(),
            config,
        }
    }
    
    pub fn add_rule(&mut self, rule: WeightedRule) -> Result<()> {
        self.rules.push(rule);
        Ok(())
    }
    
    pub fn compose_classification(
        &self,
        signals: Vec<ClassificationSignal>,
    ) -> Result<ClassificationResult> {
        if signals.is_empty() {
            return Err(ClassificationError::NoSignals);
        }
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Filter and age signals
        let valid_signals = self.filter_and_age_signals(signals, current_time)?;
        
        // Group signals by dimension
        let grouped_signals = self.group_signals_by_dimension(valid_signals)?;
        
        // Validate required dimensions
        self.validate_required_dimensions(&grouped_signals)?;
        
        // Resolve conflicts within each dimension
        let resolved_signals = self.resolve_dimensional_conflicts(grouped_signals)?;
        
        // Apply weighted scoring
        let weighted_results = self.apply_weighted_scoring(resolved_signals)?;
        
        // Compose final result
        self.compose_final_result(weighted_results, current_time)
    }
}
```

## <examples>Advanced Conflict Resolution</examples>

### <template>Consensus-Based Resolution</template>

```rust
impl RuleCompositionEngine {
    fn require_consensus(
        &self,
        signals: Vec<ClassificationSignal>,
        threshold: f64,
    ) -> Result<ClassificationSignal> {
        // Group by value and check if any value has enough consensus
        let mut value_groups: HashMap<String, Vec<ClassificationSignal>> = HashMap::new();
        
        for signal in signals {
            value_groups
                .entry(signal.value.clone())
                .or_insert_with(Vec::new)
                .push(signal);
        }
        
        for (value, group) in value_groups {
            let total_confidence: f64 = group.iter().map(|s| s.confidence).sum();
            let avg_confidence = total_confidence / group.len() as f64;
            
            if avg_confidence >= threshold {
                // Return the signal with highest confidence from this group
                return Ok(group.into_iter()
                    .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                    .unwrap());
            }
        }
        
        Err(ClassificationError::UnresolvableConflict(
            format!("No consensus above threshold {}", threshold)
        ))
    }

    fn compute_weighted_average(
        &self,
        signals: Vec<ClassificationSignal>,
    ) -> Result<ClassificationSignal> {
        if signals.is_empty() {
            return Err(ClassificationError::NoSignals);
        }
        
        // For simplicity, take the signal with highest confidence
        // In a real implementation, you might blend values more intelligently
        let best_signal = signals.into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .unwrap();
        
        Ok(best_signal)
    }
}
```

### <template>Age-Based Signal Filtering</template>

```rust
impl RuleCompositionEngine {
    fn filter_and_age_signals(
        &self,
        signals: Vec<ClassificationSignal>,
        current_time: u64,
    ) -> Result<Vec<ClassificationSignal>> {
        let mut aged_count = 0;
        let mut valid_signals = Vec::new();
        
        for mut signal in signals {
            let age = current_time.saturating_sub(signal.timestamp);
            
            // Skip signals that are too old
            if age > self.config.max_age_seconds {
                continue;
            }
            
            // Apply age decay to confidence
            if age > 0 {
                let decay = self.config.decay_factor.powf(age as f64 / 3600.0); // per hour
                signal.confidence *= decay;
                aged_count += 1;
            }
            
            // Skip signals below minimum confidence
            if signal.confidence < self.config.minimum_confidence {
                continue;
            }
            
            valid_signals.push(signal);
        }
        
        Ok(valid_signals)
    }
    
    fn apply_weighted_scoring(
        &self,
        resolved_signals: HashMap<SignalType, ClassificationSignal>,
    ) -> Result<HashMap<SignalType, (ClassificationSignal, f64)>> {
        let mut weighted_results = HashMap::new();
        
        for (signal_type, signal) in resolved_signals {
            // Find the rule for this signal type
            let rule = self.rules.iter()
                .find(|r| r.dimension == signal_type)
                .cloned()
                .unwrap_or_else(|| WeightedRule {
                    dimension: signal_type.clone(),
                    weight: 0.5, // Default weight
                    required: false,
                    boost_factor: 1.0,
                });
            
            let weighted_confidence = signal.confidence * rule.weight * rule.boost_factor;
            weighted_results.insert(signal_type, (signal, weighted_confidence));
        }
        
        Ok(weighted_results)
    }
}
```

## <examples>Predefined Rule Configurations</examples>

### <template>Balanced Rule Configuration</template>

```rust
impl RuleCompositionEngine {
    pub fn with_balanced_rules() -> Result<Self> {
        let mut engine = Self::new(CompositionConfig::default());
        
        engine.add_rule(WeightedRule::new(SignalType::ResearchType, 0.3, true)?)?;
        engine.add_rule(WeightedRule::new(SignalType::Audience, 0.25, false)?)?;
        engine.add_rule(WeightedRule::new(SignalType::Domain, 0.25, false)?)?;
        engine.add_rule(WeightedRule::new(SignalType::Urgency, 0.2, false)?
            .with_boost(1.5))?; // Boost urgency signals
        
        Ok(engine)
    }
    
    pub fn with_research_focused_rules() -> Result<Self> {
        let mut engine = Self::new(CompositionConfig {
            conflict_resolution: ConflictResolution::ConsensusRequired(0.7),
            ..CompositionConfig::default()
        });
        
        engine.add_rule(WeightedRule::new(SignalType::ResearchType, 0.5, true)?)?;
        engine.add_rule(WeightedRule::new(SignalType::Domain, 0.3, true)?)?;
        engine.add_rule(WeightedRule::new(SignalType::Audience, 0.15, false)?)?;
        engine.add_rule(WeightedRule::new(SignalType::Urgency, 0.05, false)?)?;
        
        Ok(engine)
    }
}
```

### <template>Usage Example</template>

```rust
fn main() -> Result<()> {
    // Create a classification engine with balanced rules
    let engine = RuleCompositionEngine::with_balanced_rules()?;
    
    // Create some classification signals
    let signals = vec![
        ClassificationSignal::new(
            SignalType::ResearchType,
            "empirical".to_string(),
            0.9,
            "research_classifier_v2".to_string(),
        )?,
        ClassificationSignal::new(
            SignalType::Audience,
            "academic".to_string(),
            0.8,
            "audience_detector_v1".to_string(),
        )?,
        ClassificationSignal::new(
            SignalType::Domain,
            "artificial_intelligence".to_string(),
            0.85,
            "domain_classifier_v3".to_string(),
        )?,
        ClassificationSignal::new(
            SignalType::Urgency,
            "high".to_string(),
            0.95,
            "urgency_detector_v1".to_string(),
        )?,
    ];
    
    // Compose the classification
    let result = engine.compose_classification(signals)?;
    
    println!("Classification Result:");
    println!("  Research Type: {:?}", result.research_type);
    println!("  Audience: {:?}", result.audience);
    println!("  Domain: {:?}", result.domain);
    println!("  Urgency: {:?}", result.urgency);
    println!("  Overall Confidence: {:.3}", result.overall_confidence);
    
    Ok(())
}
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### <issue>Signal Conflicts</issue>
- **Problem**: Multiple signals with equal confidence cause decision deadlock
- **Solution**: Use `ConflictResolution::MostRecent` or implement tie-breaking logic
- **Pattern**: Add timestamp-based or source-priority-based resolution

### <issue>Missing Required Dimensions</issue>
- **Problem**: Required signals are missing, causing classification failure
- **Solution**: Implement graceful degradation with default values
- **Pattern**: Use optional rules with fallback mechanisms

### <issue>Performance with Large Signal Sets</issue>
- **Problem**: Processing becomes slow with many signals
- **Solution**: Implement signal filtering and pre-grouping optimizations
- **Pattern**: Use early filtering based on confidence thresholds

## <references>See Also</references>

- [Dynamic Threshold Adjustment](dynamic-threshold-adjustment.md) - Confidence threshold optimization
- [Semantic Analysis Integration](semantic-analysis-integration.md) - Context-aware signal generation
- [Classification Patterns](../patterns/classification-algorithms.md) - Basic classification approaches
- [Performance Optimization](../patterns/performance-optimization.md) - System performance patterns

## <integration>Integration with Fortitude</integration>

### <fortitude-integration>Sprint 004 Application</fortitude-integration>

For Sprint 004 (Advanced Classification), this research enables:

1. **Multi-Dimensional Classification**: Combine research type, audience, domain, and urgency signals
2. **Conflict Resolution**: Handle conflicting classification signals intelligently
3. **Weighted Scoring**: Configure relative importance of different signal types
4. **Signal Aging**: Handle temporal aspects of classification signals

```rust
// Integration pattern for Fortitude classification system
use fortitude_core::classification::AdvancedClassifier;
use fortitude_types::research::{ClassifiedRequest, ResearchType, AudienceContext, DomainContext};

impl AdvancedClassifier {
    pub fn classify_with_composition(&self, request: &str) -> Result<ClassifiedRequest, ClassificationError> {
        let mut signals = Vec::new();
        
        // Generate signals from different detectors
        if let Some(research_signal) = self.detect_research_type(request)? {
            signals.push(research_signal);
        }
        
        if let Some(audience_signal) = self.detect_audience_context(request)? {
            signals.push(audience_signal);
        }
        
        if let Some(domain_signal) = self.detect_domain_context(request)? {
            signals.push(domain_signal);
        }
        
        if let Some(urgency_signal) = self.detect_urgency_level(request)? {
            signals.push(urgency_signal);
        }
        
        // Compose the final classification
        let composition_result = self.rule_engine.compose_classification(signals)?;
        
        // Convert to Fortitude types
        Ok(ClassifiedRequest {
            original_request: request.to_string(),
            research_type: composition_result.research_type
                .and_then(|rt| rt.parse().ok())
                .unwrap_or(ResearchType::Learning),
            audience_context: composition_result.audience
                .and_then(|ac| ac.parse().ok())
                .unwrap_or(AudienceContext::General),
            domain_context: composition_result.domain
                .and_then(|dc| dc.parse().ok())
                .unwrap_or(DomainContext::General),
            confidence: composition_result.overall_confidence,
            timestamp: std::time::SystemTime::now(),
        })
    }
}
```

**Success Metrics**:
- Multi-dimensional classification accuracy >80%
- Conflict resolution success rate >95%
- Signal processing time <10ms per request
- Memory usage <1MB per 1000 signals