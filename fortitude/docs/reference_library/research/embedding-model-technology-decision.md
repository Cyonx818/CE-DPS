# Embedding Model Technology Decision

<meta>
  <title>Embedding Model Technology Decision</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Select optimal embedding model for technical documentation semantic search
- **Key Approach**: Balance performance, cost, and integration complexity
- **Core Benefits**: Superior technical content understanding, scalable architecture
- **Recommendation**: **OpenAI text-embedding-3-large** for start, **Stella-400m** for scale
- **Related docs**: [Vector Database Technology Decision](vector-database-technology-decision.md)

## <implementation>Model Comparison Matrix</implementation>

### <pattern>Performance Rankings</pattern>

| Model | MTEB Score | Technical Content | Cost/1M tokens | Dimensions | Recommendation |
|-------|------------|-------------------|----------------|------------|----------------|
| **OpenAI-3-large** | 64.6% | Excellent | $130 | 3072 | ⭐ Start Here |
| **OpenAI-3-small** | 62.3% | Excellent | $20 | 1536 | Budget Option |
| **Stella-400m** | Competitive | Good* | $0 (+hosting) | 1024 | ⭐ Scale Here |
| **Cohere v4** | Lower | Good | $120 | 1024 | ❌ Avoid |

*Excellent with fine-tuning

### <concept>Cost Analysis</concept>

```python
# Annual cost projection (10M tokens/month)
costs = {
    "OpenAI-large": 10_000_000 * 12 * 0.00013,     # $15,600/year
    "OpenAI-small": 10_000_000 * 12 * 0.00002,     # $2,400/year
    "Local-hosting": 200 * 12,                      # $2,400/year (GPU)
    "Cohere-v4": 10_000_000 * 12 * 0.00012,       # $14,400/year
}
```

### <pattern>Technical Content Specialization</pattern>

**Code Documentation**
1. OpenAI-3-large (GitHub-trained)
2. CodeBERT (code-specific)
3. Stella-400m (generalizable)

**API References**
1. OpenAI-3-large (semantic understanding)
2. Stella-400m (cost-effective)
3. BGE-large (open source)

**Architecture Docs**
1. OpenAI-3-large (complex relationships)
2. BGE-large (structured content)
3. Stella-400m (balanced)

## <examples>Implementation Patterns</examples>

### <template>OpenAI Quick Start</template>

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

async fn get_embedding(text: &str, api_key: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let request = EmbeddingRequest {
        input: text.to_string(),
        model: "text-embedding-3-large".to_string(),
    };
    
    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json::<EmbeddingResponse>()
        .await?;
    
    Ok(response.data[0].embedding.clone())
}
```

### <template>Local Model Setup</template>

```rust
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;

struct LocalEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
}

impl LocalEmbedder {
    async fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load Stella-400m or similar
        let config = Config::from_file(&format!("{}/config.json", model_path))?;
        let model = BertModel::load(&format!("{}/model.safetensors", model_path), &config)?;
        let tokenizer = Tokenizer::from_file(&format!("{}/tokenizer.json", model_path))?;
        
        Ok(Self { model, tokenizer })
    }
    
    async fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Tokenize and generate embeddings
        let encoding = self.tokenizer.encode(text, true)?;
        // Model inference logic here
        Ok(vec![])  // Placeholder
    }
}
```

### <template>Hybrid Architecture</template>

```rust
enum EmbeddingStrategy {
    RealTime,    // Use API for low-latency
    Batch,       // Use local for bulk
    HighQuality, // Always use best model
}

async fn get_optimal_embedding(
    text: &str,
    strategy: EmbeddingStrategy,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    match strategy {
        EmbeddingStrategy::RealTime => {
            // OpenAI API for immediate response
            get_openai_embedding(text).await
        }
        EmbeddingStrategy::Batch => {
            // Local model for cost efficiency
            get_local_embedding(text).await
        }
        EmbeddingStrategy::HighQuality => {
            // Always use OpenAI-3-large
            get_openai_embedding_large(text).await
        }
    }
}
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>High Latency</issue>
**Problem**: API calls taking >500ms
**Solution**: Implement request pooling and caching:
```rust
use lru::LruCache;
use std::sync::Mutex;

struct EmbeddingCache {
    cache: Mutex<LruCache<String, Vec<f32>>>,
}

impl EmbeddingCache {
    fn get_or_compute(&self, text: &str) -> Vec<f32> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(embedding) = cache.get(text) {
            return embedding.clone();
        }
        // Compute and cache
    }
}
```

### <issue>Storage Costs</issue>
**Problem**: 3072-dim vectors expensive
**Solution**: Dimension reduction:
```rust
// Use OpenAI's dimension parameter
let request = json!({
    "input": text,
    "model": "text-embedding-3-large",
    "dimensions": 1536  // Reduces to half size
});
```

### <issue>Rate Limits</issue>
**Problem**: OpenAI API throttling
**Solution**: Implement exponential backoff:
```rust
use tokio::time::{sleep, Duration};

async fn embed_with_retry(text: &str, max_retries: u32) -> Result<Vec<f32>, Error> {
    for attempt in 0..max_retries {
        match get_embedding(text).await {
            Ok(embedding) => return Ok(embedding),
            Err(e) if e.is_rate_limit() => {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

## <constraints>Selection Constraints</constraints>

### <constraint>OpenAI Limitations</constraint>
- Data leaves premises (privacy concern)
- API dependency (availability risk)
- High-volume costs escalate quickly
- 8192 token limit per request

### <constraint>Local Model Limitations</constraint>
- GPU infrastructure required
- Setup complexity higher
- Fine-tuning expertise needed
- Model updates manual process

### <constraint>Storage Implications</constraint>
```
Vector Storage Costs (1M documents):
- OpenAI-large (3072): 12GB + overhead
- OpenAI-small (1536): 6GB + overhead  
- Stella/BGE (1024): 4GB + overhead
```

## <implementation>Phased Rollout Strategy</implementation>

### <phase>Week 1-2: Quick Start</phase>
```rust
// Start with OpenAI for immediate value
let embedder = OpenAIEmbedder::new(api_key);
let vectors = embedder.embed_batch(documents).await?;
vector_db.insert_batch(vectors).await?;
```

### <phase>Month 1-2: Measure & Optimize</phase>
```rust
// Track performance metrics
struct EmbeddingMetrics {
    latency_p95: Duration,
    cost_per_doc: f64,
    retrieval_accuracy: f64,
}

// A/B test configurations
let configs = vec![
    ("openai-large", 3072),
    ("openai-small", 1536),
    ("openai-large-reduced", 1536),
];
```

### <phase>Month 2-3: Local Deployment</phase>
```rust
// Deploy Stella-400m for batch processing
let local_embedder = LocalEmbedder::new("stella-400m").await?;

// Hybrid routing logic
if batch_size > 1000 || latency_tolerance > Duration::from_secs(60) {
    local_embedder.process_batch(documents).await
} else {
    openai_embedder.process_realtime(documents).await
}
```

## <references>See Also</references>
- [Vector Database Technology Decision](vector-database-technology-decision.md)
- [Hybrid Storage System Implementation](hybrid-storage-system-implementation.md)
- [Multi-LLM Provider System](multi-llm-provider-system.md)
- [Production-Ready Rust API System](production-ready-rust-api-system.md)