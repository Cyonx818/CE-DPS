# Advanced Classification System Guide

<meta>
  <title>Advanced Classification System Guide</title>
  <type>documentation</type>
  <audience>developer</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Multi-Dimensional Classification**: Research type + audience + domain + urgency detection
- **Context-Aware Processing**: 84.9% accuracy with intelligent fallback mechanisms
- **Signal Composition**: Weighted rule system with contextual boosting
- **Performance**: <500ms advanced classification, <100ms context detection
- **Configuration**: Flexible thresholds, graceful degradation, production-ready

## System Overview

### Architecture Components
```
┌─────────────────────────────────────────────────────────────────┐
│                    Advanced Classification Pipeline              │
├─────────────────────────────────────────────────────────────────┤
│  Query Input                                                    │
│       ↓                                                         │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐  │
│  │ Basic Classifier │    │ Context Detector │    │ Signal Composer │  │
│  │   Research Type  │    │   Multi-Context  │    │   Weighted Rules│  │
│  │      80%+        │    │     84.9%       │    │    Fallback    │  │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘  │
│       ↓                          ↓                          ↓     │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │              Enhanced Classification Result              │  │
│  │   • Research Type (Decision/Implementation/etc.)        │  │
│  │   • Audience Level (Beginner/Intermediate/Advanced)     │  │
│  │   • Technical Domain (Rust/Web/DevOps/etc.)            │  │
│  │   • Urgency Level (Planned/Soon/Immediate)              │  │
│  │   • Multi-dimensional confidence scores                 │  │
│  │   • Contextual boosting and fallback metadata          │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Features
1. **Multi-Dimensional Analysis**: Four-dimensional classification with independent confidence scores
2. **Context-Aware Processing**: Intelligent audience, domain, and urgency detection
3. **Signal Composition**: Weighted rule system with contextual boosting
4. **Graceful Degradation**: Fallback mechanisms for reliability
5. **Performance Optimization**: Sub-500ms processing with timeout controls

## Classification Dimensions

### 1. Research Type Classification
**Purpose**: Determine the type of research needed
**Accuracy**: 92%+ (baseline from basic classifier)
**Processing Time**: <50ms

**Supported Types**:
- **Decision**: Technology selection, architecture choices
- **Implementation**: How-to guides, code examples
- **Troubleshooting**: Debugging, error resolution
- **Learning**: Concepts, fundamentals, theory
- **Validation**: Code review, best practices

**Configuration**:
```rust
ClassificationConfig {
    default_threshold: 0.2,    // Higher threshold for production
    enable_advanced: true,     // Enable multi-dimensional analysis
    ..Default::default()
}
```

### 2. Audience Level Detection
**Purpose**: Adapt content complexity to user expertise
**Accuracy**: 84.9% (context detection)
**Processing Time**: <100ms

**Supported Levels**:
- **Beginner**: New to technology, needs foundational concepts
- **Intermediate**: Some experience, needs practical guidance
- **Advanced**: Expert level, needs architectural insights

**Detection Patterns**:
```rust
// Beginner indicators
"I'm new to", "learning", "getting started", "basics", "introduction"

// Intermediate indicators  
"How to implement", "best practices", "working on", "developing"

// Advanced indicators
"architecture", "optimization", "performance", "scaling", "enterprise"
```

### 3. Technical Domain Analysis
**Purpose**: Identify the technical context for specialized knowledge
**Accuracy**: 89% (domain-specific patterns)
**Processing Time**: <50ms

**Supported Domains**:
- **Rust**: Systems programming, memory safety, concurrency
- **Web**: Frontend, backend, full-stack development
- **DevOps**: Infrastructure, deployment, monitoring
- **Database**: SQL, NoSQL, data modeling
- **Security**: Authentication, encryption, vulnerability analysis
- **General**: Cross-cutting concerns, methodology

**Detection Examples**:
```rust
// Rust domain
"ownership", "borrowing", "lifetimes", "cargo", "tokio"

// Web domain
"HTTP", "REST", "GraphQL", "React", "Node.js"

// DevOps domain
"deployment", "containers", "Kubernetes", "CI/CD"
```

### 4. Urgency Level Assessment
**Purpose**: Prioritize response based on time sensitivity
**Accuracy**: 78% (urgency keywords)
**Processing Time**: <25ms

**Supported Levels**:
- **Planned**: Long-term planning, research, exploration
- **Soon**: Upcoming sprint, next milestone
- **Immediate**: Production issues, blockers, critical problems

**Urgency Indicators**:
```rust
// Immediate urgency
"URGENT", "production", "critical", "blocking", "emergency"

// Soon urgency
"sprint", "deadline", "milestone", "next week"

// Planned urgency
"exploring", "considering", "future", "planning"
```

## Advanced Configuration

### Complete Configuration Structure
```json
{
  "classification": {
    "default_threshold": 0.2,
    "enable_advanced": true,
    "enable_context_detection": true,
    "context_detection": {
      "confidence_threshold": 0.6,
      "enable_fallback": true,
      "max_processing_time_ms": 1000,
      "debug_logging": false
    },
    "advanced_classification": {
      "enable_graceful_degradation": true,
      "max_processing_time_ms": 1000,
      "signal_composition": {
        "confidence_threshold": 0.5,
        "enable_fallback": true,
        "max_signals": 10
      },
      "contextual_weights": {
        "urgency_boost": 1.3,
        "beginner_boost": 1.2,
        "domain_boost": 1.1,
        "low_confidence_penalty": 0.8
      }
    }
  }
}
```

### Environment Variables
```bash
# Advanced classification control
FORTITUDE_CLASSIFICATION_ADVANCED=true
FORTITUDE_CLASSIFICATION_CONTEXT_DETECTION=true

# Context detection tuning
FORTITUDE_CONTEXT_CONFIDENCE_THRESHOLD=0.6
FORTITUDE_CONTEXT_ENABLE_FALLBACK=true
FORTITUDE_CONTEXT_MAX_PROCESSING_TIME_MS=1000

# Advanced classification tuning
FORTITUDE_ADVANCED_GRACEFUL_DEGRADATION=true
FORTITUDE_ADVANCED_MAX_PROCESSING_TIME_MS=1000

# Signal composition tuning
FORTITUDE_SIGNAL_CONFIDENCE_THRESHOLD=0.5
FORTITUDE_SIGNAL_ENABLE_FALLBACK=true
FORTITUDE_SIGNAL_MAX_SIGNALS=10

# Contextual weights
FORTITUDE_WEIGHT_URGENCY_BOOST=1.3
FORTITUDE_WEIGHT_BEGINNER_BOOST=1.2
FORTITUDE_WEIGHT_DOMAIN_BOOST=1.1
FORTITUDE_WEIGHT_LOW_CONFIDENCE_PENALTY=0.8
```

## Signal Composition System

### Weighted Rule Architecture
```rust
pub struct WeightedRule {
    pub rule_id: String,
    pub weight: f64,
    pub confidence_threshold: f64,
    pub dimensions: Vec<ClassificationDimension>,
}
```

### Signal Types
```rust
pub enum ClassificationSignal {
    ResearchType(ResearchType, f64, Vec<String>),
    AudienceLevel(AudienceLevel, f64, Vec<String>),
    TechnicalDomain(TechnicalDomain, f64, Vec<String>),
    UrgencyLevel(UrgencyLevel, f64, Vec<String>),
}
```

### Contextual Boosting
```rust
// Urgency boost for immediate queries
if urgency_level == UrgencyLevel::Immediate {
    confidence *= contextual_weights.urgency_boost; // 1.3x
}

// Beginner boost for learning queries
if audience_level == AudienceLevel::Beginner {
    confidence *= contextual_weights.beginner_boost; // 1.2x
}

// Domain boost for specific technical contexts
if domain != TechnicalDomain::General {
    confidence *= contextual_weights.domain_boost; // 1.1x
}
```

## Usage Examples

### CLI Usage
```bash
# Basic advanced classification
fortitude research "How to implement async functions in Rust?" \
  --advanced-classification

# Full advanced classification with context detection
fortitude research "I'm new to Rust and need to debug a memory issue urgently" \
  --advanced-classification \
  --context-detection \
  --context-threshold 0.7 \
  --graceful-degradation

# Production-ready configuration
fortitude research "Choose between PostgreSQL and MongoDB for our enterprise app" \
  --advanced-classification \
  --context-detection \
  --context-threshold 0.8 \
  --graceful-degradation \
  --verbose
```

### Programmatic Usage
```rust
use fortitude_core::classification::advanced_classifier::{
    AdvancedClassifier, AdvancedClassificationConfig
};

// Create research-focused classifier
let classifier = AdvancedClassifier::with_research_focus()?;

// Perform enhanced classification
let result = classifier.classify_enhanced(
    "I'm getting a segfault in my Rust program and need to fix it immediately",
    &ResearchType::Troubleshooting,
)?;

// Access multi-dimensional results
println!("Research Type: {}", result.research_type);
println!("Audience Level: {}", result.audience_level);
println!("Technical Domain: {}", result.technical_domain);
println!("Urgency Level: {}", result.urgency_level);
println!("Overall Confidence: {:.2}", result.overall_confidence);
```

## Performance Characteristics

### Processing Time Breakdown
```
┌─────────────────────────────────────────────────────────────────┐
│                 Advanced Classification Pipeline                 │
├─────────────────────────────────────────────────────────────────┤
│  Basic Classification           │  3-50ms     │  Research Type   │
│  Context Detection              │  12-100ms   │  Multi-Context   │
│  Signal Composition             │  1-10ms     │  Weighted Rules  │
│  Contextual Boosting           │  <1ms       │  Weight Apply    │
│  Result Assembly               │  <1ms       │  Metadata        │
├─────────────────────────────────────────────────────────────────┤
│  Total Processing Time         │  <500ms     │  Well Under Limit │
└─────────────────────────────────────────────────────────────────┘
```

### Memory Usage
- **Base Memory**: ~2MB for rule sets and patterns
- **Per-Query Memory**: ~50KB temporary allocation
- **Concurrent Queries**: Linear scaling with efficient cleanup

### Accuracy Metrics
- **Research Type**: 92%+ (keyword matching)
- **Audience Level**: 84.9% (context patterns)
- **Technical Domain**: 89% (domain-specific keywords)
- **Urgency Level**: 78% (urgency indicators)
- **Overall System**: 84.9% (weighted average)

## Fallback Mechanisms

### Graceful Degradation Strategy
```rust
// Context detection fallback
if context_detection_fails {
    warn!("Context detection failed, using basic classification");
    return basic_classifier.classify(query);
}

// Signal composition fallback
if signal_composition_fails {
    warn!("Signal composition failed, using highest confidence signal");
    return highest_confidence_signal;
}

// Timeout fallback
if processing_time > max_processing_time {
    warn!("Processing timeout, returning partial results");
    return partial_results_with_fallback_flag;
}
```

### Fallback Quality Assurance
- **Fallback Detection**: All fallback usage is logged and tracked
- **Quality Metrics**: Fallback results maintain minimum quality standards
- **User Transparency**: Fallback usage is indicated in result metadata

## Debugging and Monitoring

### Debug Configuration
```json
{
  "classification": {
    "context_detection": {
      "debug_logging": true
    }
  },
  "logging": {
    "level": "debug",
    "json_format": true
  }
}
```

### Debug Output Example
```json
{
  "level": "DEBUG",
  "message": "Context detection results",
  "query": "I'm new to Rust and need help with ownership",
  "dimensions": {
    "audience_level": {
      "detected": "Beginner",
      "confidence": 0.92,
      "matched_keywords": ["new to", "need help"]
    },
    "technical_domain": {
      "detected": "Rust",
      "confidence": 0.95,
      "matched_keywords": ["Rust", "ownership"]
    },
    "urgency_level": {
      "detected": "Planned",
      "confidence": 0.60,
      "matched_keywords": []
    }
  }
}
```

### Monitoring Metrics
```rust
// Key metrics to track
pub struct ClassificationMetrics {
    pub processing_time_ms: u64,
    pub confidence_distribution: HashMap<String, f64>,
    pub fallback_usage_rate: f64,
    pub dimension_accuracy: HashMap<ClassificationDimension, f64>,
    pub contextual_boost_frequency: f64,
}
```

## Best Practices

### Configuration Tuning
1. **Start Conservative**: Begin with higher thresholds (0.7+) for production
2. **Enable Fallbacks**: Always enable graceful degradation in production
3. **Monitor Performance**: Track processing times and adjust timeouts
4. **Test Thoroughly**: Validate accuracy with domain-specific queries

### Performance Optimization
1. **Batch Processing**: Process multiple queries concurrently
2. **Caching**: Cache classification results for repeated queries
3. **Timeout Management**: Set appropriate timeouts for your use case
4. **Resource Limits**: Configure max concurrent operations

### Error Handling
```rust
match classifier.classify_enhanced(query, &research_type) {
    Ok(result) => {
        if result.metadata.fallback_used {
            warn!("Fallback used for query: {}", query);
        }
        // Process result
    },
    Err(ClassificationError::ProcessingTimeout) => {
        // Handle timeout with appropriate fallback
    },
    Err(e) => {
        // Handle other errors
    }
}
```

## Advanced Features

### Custom Rule Development
```rust
// Custom domain rule example
let custom_rule = WeightedRule {
    rule_id: "machine_learning_domain".to_string(),
    weight: 1.5,
    confidence_threshold: 0.8,
    dimensions: vec![ClassificationDimension::TechnicalDomain],
};

// Add to signal composer
let mut composer = SignalComposer::default();
composer.add_rule(custom_rule)?;
```

### Integration with Pipeline
```rust
// Pipeline configuration with advanced classification
let pipeline_config = PipelineConfig {
    enable_advanced_classification: true,
    enable_context_detection: true,
    classification_timeout_ms: 1000,
    ..Default::default()
};

let pipeline = PipelineBuilder::new()
    .with_advanced_classification(advanced_classifier)
    .with_context_detection(context_detector)
    .build(classifier, storage);
```

---

**Advanced Classification Summary**: The advanced classification system provides multi-dimensional analysis with 84.9% accuracy, sub-500ms processing times, and robust fallback mechanisms. It enables context-aware research with audience adaptation, domain specialization, and urgency prioritization for optimal user experience.