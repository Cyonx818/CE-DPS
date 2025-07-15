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

// Composed classification result
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub research_type: Option<String>,
    pub audience: Option<String>,
    pub domain: Option<String>,
    pub urgency: Option<String>,
    pub overall_confidence: f64,
    pub contributing_signals: Vec<ClassificationSignal>,
    pub composition_metadata: CompositionMetadata,
}

#[derive(Debug, Clone)]
pub struct CompositionMetadata {
    pub total_signals: usize,
    pub conflicts_resolved: usize,
    pub aged_signals: usize,
    pub rule_weights_applied: HashMap<String, f64>,
}

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
    
    fn group_signals_by_dimension(
        &self,
        signals: Vec<ClassificationSignal>,
    ) -> Result<HashMap<SignalType, Vec<ClassificationSignal>>> {
        let mut grouped = HashMap::new();
        
        for signal in signals {
            grouped
                .entry(signal.signal_type.clone())
                .or_insert_with(Vec::new)
                .push(signal);
        }
        
        Ok(grouped)
    }
    
    fn validate_required_dimensions(
        &self,
        grouped_signals: &HashMap<SignalType, Vec<ClassificationSignal>>,
    ) -> Result<()> {
        for rule in &self.rules {
            if rule.required && !grouped_signals.contains_key(&rule.dimension) {
                return Err(ClassificationError::MissingDimension(
                    format!("{:?}", rule.dimension)
                ));
            }
        }
        Ok(())
    }
    
    fn resolve_dimensional_conflicts(
        &self,
        grouped_signals: HashMap<SignalType, Vec<ClassificationSignal>>,
    ) -> Result<HashMap<SignalType, ClassificationSignal>> {
        let mut resolved = HashMap::new();
        let mut conflicts_resolved = 0;
        
        for (signal_type, signals) in grouped_signals {
            if signals.len() == 1 {
                resolved.insert(signal_type, signals.into_iter().next().unwrap());
                continue;
            }
            
            // Multiple signals - resolve conflict
            let resolved_signal = match &self.config.conflict_resolution {
                ConflictResolution::HighestConfidence => {
                    signals.into_iter()
                        .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                        .unwrap()
                }
                ConflictResolution::MostRecent => {
                    signals.into_iter()
                        .max_by_key(|s| s.timestamp)
                        .unwrap()
                }
                ConflictResolution::WeightedAverage => {
                    self.compute_weighted_average(signals)?
                }
                ConflictResolution::ConsensusRequired(threshold) => {
                    self.require_consensus(signals, *threshold)?
                }
            };
            
            resolved.insert(signal_type, resolved_signal);
            conflicts_resolved += 1;
        }
        
        Ok(resolved)
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
    
    fn compose_final_result(
        &self,
        weighted_results: HashMap<SignalType, (ClassificationSignal, f64)>,
        _current_time: u64,
    ) -> Result<ClassificationResult> {
        let mut research_type = None;
        let mut audience = None;
        let mut domain = None;
        let mut urgency = None;
        let mut contributing_signals = Vec::new();
        let mut rule_weights_applied = HashMap::new();
        
        let mut total_weighted_confidence = 0.0;
        let mut total_weights = 0.0;
        
        for (signal_type, (signal, weighted_confidence)) in weighted_results {
            contributing_signals.push(signal.clone());
            
            match signal_type {
                SignalType::ResearchType => research_type = Some(signal.value),
                SignalType::Audience => audience = Some(signal.value),
                SignalType::Domain => domain = Some(signal.value),
                SignalType::Urgency => urgency = Some(signal.value),
            }
            
            let rule_weight = self.rules.iter()
                .find(|r| r.dimension == signal_type)
                .map(|r| r.weight)
                .unwrap_or(0.5);
            
            rule_weights_applied.insert(format!("{:?}", signal_type), rule_weight);
            total_weighted_confidence += weighted_confidence;
            total_weights += rule_weight;
        }
        
        let overall_confidence = if total_weights > 0.0 {
            total_weighted_confidence / total_weights
        } else {
            0.0
        };
        
        let metadata = CompositionMetadata {
            total_signals: contributing_signals.len(),
            conflicts_resolved: 0, // Would be tracked during resolution
            aged_signals: 0, // Would be tracked during filtering
            rule_weights_applied,
        };
        
        Ok(ClassificationResult {
            research_type,
            audience,
            domain,
            urgency,
            overall_confidence,
            contributing_signals,
            composition_metadata: metadata,
        })
    }
}

// Utility functions for creating common rule sets
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

// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_classification() -> Result<()> {
        let mut engine = RuleCompositionEngine::with_balanced_rules()?;
        
        let signals = vec![
            ClassificationSignal::new(
                SignalType::ResearchType,
                "empirical".to_string(),
                0.9,
                "ml_classifier".to_string(),
            )?,
            ClassificationSignal::new(
                SignalType::Audience,
                "academic".to_string(),
                0.8,
                "audience_detector".to_string(),
            )?,
            ClassificationSignal::new(
                SignalType::Domain,
                "computer_science".to_string(),
                0.85,
                "domain_classifier".to_string(),
            )?,
        ];
        
        let result = engine.compose_classification(signals)?;
        
        assert_eq!(result.research_type, Some("empirical".to_string()));
        assert_eq!(result.audience, Some("academic".to_string()));
        assert_eq!(result.domain, Some("computer_science".to_string()));
        assert!(result.overall_confidence > 0.0);
        
        Ok(())
    }
    
    #[test]
    fn test_conflict_resolution() -> Result<()> {
        let mut engine = RuleCompositionEngine::new(CompositionConfig {
            conflict_resolution: ConflictResolution::HighestConfidence,
            ..CompositionConfig::default()
        });
        
        engine.add_rule(WeightedRule::new(SignalType::ResearchType, 1.0, true)?)?;
        
        let signals = vec![
            ClassificationSignal::new(
                SignalType::ResearchType,
                "theoretical".to_string(),
                0.7,
                "classifier_a".to_string(),
            )?,
            ClassificationSignal::new(
                SignalType::ResearchType,
                "empirical".to_string(),
                0.9,
                "classifier_b".to_string(),
            )?,
        ];
        
        let result = engine.compose_classification(signals)?;
        
        // Should pick "empirical" due to higher confidence
        assert_eq!(result.research_type, Some("empirical".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_missing_required_dimension() {
        let engine = RuleCompositionEngine::with_research_focused_rules().unwrap();
        
        let signals = vec![
            ClassificationSignal::new(
                SignalType::Audience,
                "academic".to_string(),
                0.8,
                "source".to_string(),
            ).unwrap(),
        ];
        
        let result = engine.compose_classification(signals);
        assert!(matches!(result, Err(ClassificationError::MissingDimension(_))));
    }
}

// Example usage
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
    println!("  Contributing Signals: {}", result.contributing_signals.len());
    
    Ok(())
}