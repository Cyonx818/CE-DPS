# ML Framework Integration in Rust

<meta>
  <title>ML Framework Integration in Rust</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-10</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Production-ready ML model serving in Rust with ONNX Runtime and Candle framework integration
- **Key Approach**: Dual framework support + async inference + model quantization + GPU acceleration
- **Core Benefits**: Memory-efficient serving, cross-platform compatibility, production-grade performance
- **When to use**: Local inference pipelines, edge deployment, high-performance ML serving applications
- **Related docs**: [Vector Database Optimization](vector-database-performance-optimization.md), [Hybrid Search](hybrid-search-algorithm-implementation.md)

## <implementation>Core Architecture</implementation>

### <pattern>Multi-Framework Model Manager</pattern>
```rust
use ort::{Session, SessionBuilder, Value};
use candle_core::{Device, Tensor, Module};
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct MLFrameworkManager {
    onnx_models: Arc<RwLock<HashMap<String, OnnxModel>>>,
    candle_models: Arc<RwLock<HashMap<String, CandleModel>>>,
    memory_pool: Arc<MemoryPool>,
    device_manager: Arc<DeviceManager>,
    config: MLConfig,
}

#[derive(Debug, Clone)]
pub struct MLConfig {
    pub max_model_size: usize,
    pub memory_pool_size: usize,
    pub max_batch_size: usize,
    pub enable_gpu: bool,
    pub enable_quantization: bool,
    pub intra_op_threads: usize,
    pub inter_op_threads: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            max_model_size: 2_147_483_648, // 2GB
            memory_pool_size: 16,
            max_batch_size: 32,
            enable_gpu: true,
            enable_quantization: true,
            intra_op_threads: 8,
            inter_op_threads: 1,
        }
    }
}

impl MLFrameworkManager {
    pub async fn new(config: MLConfig) -> Result<Self, MLError> {
        let device_manager = Arc::new(DeviceManager::new(config.enable_gpu)?);
        let memory_pool = Arc::new(MemoryPool::new(config.memory_pool_size)?);
        
        Ok(Self {
            onnx_models: Arc::new(RwLock::new(HashMap::new())),
            candle_models: Arc::new(RwLock::new(HashMap::new())),
            memory_pool,
            device_manager,
            config,
        })
    }
}
```

### <pattern>ONNX Runtime Integration</pattern>
```rust
pub struct OnnxModel {
    session: Session,
    input_specs: Vec<InputSpec>,
    output_specs: Vec<OutputSpec>,
    quantized: bool,
    last_used: Instant,
}

#[derive(Debug, Clone)]
pub struct InputSpec {
    pub name: String,
    pub shape: Vec<i64>,
    pub data_type: ElementType,
}

impl OnnxModel {
    pub async fn load_from_file(
        path: &str,
        quantized: bool,
        execution_providers: &[&str],
    ) -> Result<Self, MLError> {
        let mut builder = SessionBuilder::new()?
            .with_optimization_level(ort::GraphOptimizationLevel::All)?
            .with_intra_threads(num_cpus::get())?
            .with_inter_threads(1)?;
        
        // Configure execution providers in order of preference
        for provider in execution_providers {
            match *provider {
                "CUDA" => {
                    builder = builder.with_cuda(0)?;
                }
                "TensorRT" => {
                    builder = builder.with_tensorrt(0)?;
                }
                "CPU" => {
                    // CPU is always available as fallback
                }
                _ => {
                    warn!("Unknown execution provider: {}", provider);
                }
            }
        }
        
        let session = builder.commit_from_file(path)?;
        
        // Extract input/output metadata
        let input_specs = session.inputs.iter().map(|input| {
            InputSpec {
                name: input.name.clone(),
                shape: input.input_type.tensor_dimensions().unwrap_or_default(),
                data_type: input.input_type.tensor_element_data_type().unwrap(),
            }
        }).collect();
        
        let output_specs = session.outputs.iter().map(|output| {
            OutputSpec {
                name: output.name.clone(),
                shape: output.output_type.tensor_dimensions().unwrap_or_default(),
                data_type: output.output_type.tensor_element_data_type().unwrap(),
            }
        }).collect();
        
        Ok(Self {
            session,
            input_specs,
            output_specs,
            quantized,
            last_used: Instant::now(),
        })
    }
    
    pub async fn infer(
        &mut self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, MLError> {
        self.last_used = Instant::now();
        
        // Validate inputs
        for (name, value) in &inputs {
            if !self.input_specs.iter().any(|spec| &spec.name == name) {
                return Err(MLError::InvalidInput(
                    format!("Unknown input: {}", name)
                ));
            }
        }
        
        // Run inference
        let outputs = self.session.run(inputs)?;
        
        Ok(outputs)
    }
    
    pub async fn infer_batch(
        &mut self,
        batch_inputs: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>, MLError> {
        if batch_inputs.is_empty() {
            return Ok(Vec::new());
        }
        
        // For ONNX, we typically need to batch inputs manually
        let batched_inputs = self.batch_inputs(batch_inputs)?;
        let batched_outputs = self.infer(batched_inputs).await?;
        let unbatched_outputs = self.unbatch_outputs(batched_outputs)?;
        
        Ok(unbatched_outputs)
    }
    
    fn batch_inputs(
        &self,
        inputs: Vec<HashMap<String, Value>>,
    ) -> Result<HashMap<String, Value>, MLError> {
        let mut batched = HashMap::new();
        
        // Get first input as template
        let template = inputs.first().unwrap();
        
        for (input_name, _) in template {
            let mut batch_values = Vec::new();
            
            for input_map in &inputs {
                if let Some(value) = input_map.get(input_name) {
                    batch_values.push(value.clone());
                } else {
                    return Err(MLError::BatchingError(
                        format!("Missing input {} in batch item", input_name)
                    ));
                }
            }
            
            // Stack values into batch dimension
            let batched_value = self.stack_values(batch_values)?;
            batched.insert(input_name.clone(), batched_value);
        }
        
        Ok(batched)
    }
}
```

### <pattern>Candle Framework Integration</pattern>
```rust
use candle_core::{Device, Tensor, DType, Module, Result as CandleResult};
use candle_nn::{Linear, VarBuilder};

pub struct CandleModel {
    model: Box<dyn CandleModelTrait + Send + Sync>,
    device: Device,
    dtype: DType,
    input_shape: Vec<usize>,
    output_shape: Vec<usize>,
    last_used: Instant,
}

pub trait CandleModelTrait {
    fn forward(&self, input: &Tensor) -> CandleResult<Tensor>;
    fn forward_batch(&self, inputs: &[Tensor]) -> CandleResult<Vec<Tensor>>;
}

// Example: BERT-like transformer model
pub struct BertEmbeddingModel {
    embeddings: candle_nn::Embedding,
    layers: Vec<TransformerLayer>,
    pooler: Linear,
    device: Device,
}

impl BertEmbeddingModel {
    pub fn load_from_safetensors(
        path: &str,
        device: &Device,
        dtype: DType,
    ) -> Result<Self, MLError> {
        let weights = unsafe { candle_core::safetensors::load(path, device)? };
        let vb = VarBuilder::from_tensors(weights, dtype, device);
        
        let config = BertConfig::default();
        
        let embeddings = candle_nn::embedding(
            config.vocab_size,
            config.hidden_size,
            vb.pp("embeddings.word_embeddings"),
        )?;
        
        let mut layers = Vec::new();
        for i in 0..config.num_hidden_layers {
            let layer = TransformerLayer::load(vb.pp(&format!("encoder.layer.{}", i)))?;
            layers.push(layer);
        }
        
        let pooler = candle_nn::linear(
            config.hidden_size,
            config.hidden_size,
            vb.pp("pooler.dense"),
        )?;
        
        Ok(Self {
            embeddings,
            layers,
            pooler,
            device: device.clone(),
        })
    }
}

impl CandleModelTrait for BertEmbeddingModel {
    fn forward(&self, input: &Tensor) -> CandleResult<Tensor> {
        // Token embeddings
        let mut hidden_states = self.embeddings.forward(input)?;
        
        // Apply transformer layers
        for layer in &self.layers {
            hidden_states = layer.forward(&hidden_states)?;
        }
        
        // Pool the sequence (take [CLS] token)
        let cls_token = hidden_states.i((.., 0, ..))?;
        let pooled_output = self.pooler.forward(&cls_token)?;
        
        // Apply tanh activation
        pooled_output.tanh()
    }
    
    fn forward_batch(&self, inputs: &[Tensor]) -> CandleResult<Vec<Tensor>> {
        let mut outputs = Vec::new();
        
        // Process each input in the batch
        for input in inputs {
            let output = self.forward(input)?;
            outputs.push(output);
        }
        
        Ok(outputs)
    }
}

impl CandleModel {
    pub async fn load_from_safetensors(
        path: &str,
        model_type: &str,
        device: Device,
        dtype: DType,
    ) -> Result<Self, MLError> {
        let model: Box<dyn CandleModelTrait + Send + Sync> = match model_type {
            "bert" => {
                let bert_model = BertEmbeddingModel::load_from_safetensors(path, &device, dtype)?;
                Box::new(bert_model)
            }
            "llama" => {
                let llama_model = LlamaModel::load_from_safetensors(path, &device, dtype)?;
                Box::new(llama_model)
            }
            _ => {
                return Err(MLError::UnsupportedModelType(model_type.to_string()));
            }
        };
        
        Ok(Self {
            model,
            device,
            dtype,
            input_shape: vec![1, 512], // Default sequence length
            output_shape: vec![768],   // Default embedding dimension
            last_used: Instant::now(),
        })
    }
    
    pub async fn infer(&mut self, input: &Tensor) -> Result<Tensor, MLError> {
        self.last_used = Instant::now();
        
        // Ensure input is on correct device
        let input = if input.device() != &self.device {
            input.to_device(&self.device)?
        } else {
            input.clone()
        };
        
        // Convert dtype if necessary
        let input = if input.dtype() != self.dtype {
            input.to_dtype(self.dtype)?
        } else {
            input
        };
        
        let output = self.model.forward(&input)?;
        Ok(output)
    }
    
    pub async fn infer_batch(&mut self, inputs: Vec<Tensor>) -> Result<Vec<Tensor>, MLError> {
        self.last_used = Instant::now();
        
        // Move all inputs to correct device and dtype
        let processed_inputs: Result<Vec<_>, _> = inputs.into_iter()
            .map(|input| {
                let input = if input.device() != &self.device {
                    input.to_device(&self.device)?
                } else {
                    input
                };
                
                if input.dtype() != self.dtype {
                    input.to_dtype(self.dtype)
                } else {
                    Ok(input)
                }
            })
            .collect();
        
        let processed_inputs = processed_inputs?;
        let outputs = self.model.forward_batch(&processed_inputs)?;
        
        Ok(outputs)
    }
}
```

## <examples>Production Integration</examples>

### <template>Async Inference Server</template>
```rust
use axum::{Router, routing::post, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct InferenceRequest {
    pub model_name: String,
    pub input_data: Vec<f32>,
    pub batch_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct InferenceResponse {
    pub output_data: Vec<f32>,
    pub latency_ms: f64,
    pub model_name: String,
}

pub struct MLInferenceServer {
    manager: Arc<MLFrameworkManager>,
    metrics: Arc<MetricsCollector>,
}

impl MLInferenceServer {
    pub async fn new(config: MLConfig) -> Result<Self, MLError> {
        let manager = Arc::new(MLFrameworkManager::new(config).await?);
        let metrics = Arc::new(MetricsCollector::new());
        
        Ok(Self { manager, metrics })
    }
    
    pub fn create_router(self) -> Router {
        Router::new()
            .route("/infer", post(inference_handler))
            .route("/health", axum::routing::get(health_handler))
            .route("/models", axum::routing::get(list_models_handler))
            .route("/metrics", axum::routing::get(metrics_handler))
            .layer(Extension(Arc::new(self)))
    }
}

async fn inference_handler(
    Extension(server): Extension<Arc<MLInferenceServer>>,
    Json(request): Json<InferenceRequest>,
) -> Result<Json<InferenceResponse>, MLError> {
    let start_time = Instant::now();
    
    // Validate request
    if request.input_data.is_empty() {
        return Err(MLError::InvalidInput("Empty input data".to_string()));
    }
    
    // Route to appropriate framework
    let output_data = if server.manager.has_onnx_model(&request.model_name).await {
        server.manager.infer_onnx(&request.model_name, &request.input_data).await?
    } else if server.manager.has_candle_model(&request.model_name).await {
        server.manager.infer_candle(&request.model_name, &request.input_data).await?
    } else {
        return Err(MLError::ModelNotFound(request.model_name.clone()));
    };
    
    let latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    
    // Update metrics
    server.metrics.record_inference(
        &request.model_name,
        latency_ms,
        request.input_data.len(),
        output_data.len(),
    ).await;
    
    Ok(Json(InferenceResponse {
        output_data,
        latency_ms,
        model_name: request.model_name,
    }))
}

async fn health_handler(
    Extension(server): Extension<Arc<MLInferenceServer>>,
) -> Json<serde_json::Value> {
    let models = server.manager.list_loaded_models().await;
    let metrics = server.metrics.get_summary().await;
    
    Json(serde_json::json!({
        "status": "healthy",
        "models": models,
        "metrics": metrics,
        "uptime_seconds": server.metrics.get_uptime().as_secs()
    }))
}
```

### <template>Memory Management and Optimization</template>
```rust
pub struct MemoryPool {
    pools: Vec<Arc<RwLock<Vec<Tensor>>>>,
    max_tensor_size: usize,
    gc_threshold: f64,
    allocated_bytes: AtomicUsize,
    max_allocated_bytes: usize,
}

impl MemoryPool {
    pub fn new(pool_size: usize) -> Result<Self, MLError> {
        let mut pools = Vec::new();
        for _ in 0..pool_size {
            pools.push(Arc::new(RwLock::new(Vec::new())));
        }
        
        Ok(Self {
            pools,
            max_tensor_size: 512 * 1024 * 1024, // 512MB
            gc_threshold: 0.8,
            allocated_bytes: AtomicUsize::new(0),
            max_allocated_bytes: 16 * 1024 * 1024 * 1024, // 16GB
        })
    }
    
    pub async fn get_tensor(&self, shape: &[usize], dtype: DType, device: &Device) -> Result<Tensor, MLError> {
        let tensor_size = shape.iter().product::<usize>() * dtype.size_in_bytes();
        
        if tensor_size > self.max_tensor_size {
            return Err(MLError::TensorTooLarge(tensor_size));
        }
        
        // Try to reuse from pool
        for pool in &self.pools {
            let mut pool_guard = pool.write().await;
            if let Some(tensor) = pool_guard.pop() {
                if tensor.shape().dims() == shape && tensor.dtype() == dtype && tensor.device() == device {
                    return Ok(tensor);
                }
                // Put back if not matching
                pool_guard.push(tensor);
            }
        }
        
        // Create new tensor if none available
        let tensor = Tensor::zeros(shape, dtype, device)?;
        
        // Update allocation tracking
        let current_allocated = self.allocated_bytes.fetch_add(tensor_size, Ordering::Relaxed);
        if current_allocated + tensor_size > self.max_allocated_bytes {
            self.trigger_gc().await?;
        }
        
        Ok(tensor)
    }
    
    pub async fn return_tensor(&self, tensor: Tensor) -> Result<(), MLError> {
        let tensor_size = tensor.elem_count() * tensor.dtype().size_in_bytes();
        
        // Find appropriate pool
        let pool_index = (tensor_size / 1024) % self.pools.len();
        let mut pool_guard = self.pools[pool_index].write().await;
        
        if pool_guard.len() < 100 { // Max 100 tensors per pool
            pool_guard.push(tensor);
        } else {
            // Pool is full, deallocate
            self.allocated_bytes.fetch_sub(tensor_size, Ordering::Relaxed);
        }
        
        Ok(())
    }
    
    async fn trigger_gc(&self) -> Result<(), MLError> {
        let mut total_freed = 0;
        
        for pool in &self.pools {
            let mut pool_guard = pool.write().await;
            let to_remove = pool_guard.len() / 2; // Remove half
            
            for _ in 0..to_remove {
                if let Some(tensor) = pool_guard.pop() {
                    total_freed += tensor.elem_count() * tensor.dtype().size_in_bytes();
                }
            }
        }
        
        self.allocated_bytes.fetch_sub(total_freed, Ordering::Relaxed);
        info!("Garbage collection freed {} bytes", total_freed);
        
        Ok(())
    }
}

// Device management for GPU acceleration
pub struct DeviceManager {
    devices: Vec<Device>,
    current_device: AtomicUsize,
    enable_gpu: bool,
}

impl DeviceManager {
    pub fn new(enable_gpu: bool) -> Result<Self, MLError> {
        let mut devices = vec![Device::Cpu];
        
        if enable_gpu {
            // Try to initialize CUDA devices
            match Device::new_cuda(0) {
                Ok(cuda_device) => {
                    devices.push(cuda_device);
                    info!("CUDA device 0 initialized successfully");
                }
                Err(e) => {
                    warn!("Failed to initialize CUDA device: {}", e);
                }
            }
        }
        
        Ok(Self {
            devices,
            current_device: AtomicUsize::new(0),
            enable_gpu,
        })
    }
    
    pub fn get_best_device(&self) -> &Device {
        if self.enable_gpu && self.devices.len() > 1 {
            &self.devices[1] // Use GPU if available
        } else {
            &self.devices[0] // Fallback to CPU
        }
    }
    
    pub fn get_device_for_model(&self, model_size: usize) -> &Device {
        // Use CPU for very large models to avoid GPU memory issues
        if model_size > 4 * 1024 * 1024 * 1024 { // 4GB
            &self.devices[0]
        } else {
            self.get_best_device()
        }
    }
}
```

## <troubleshooting>Performance Optimization</troubleshooting>

### <issue>High Memory Usage</issue>
**Problem**: Models consuming too much RAM/VRAM
**Solution**:
```rust
// 1. Enable model quantization
pub async fn quantize_onnx_model(
    input_path: &str,
    output_path: &str,
) -> Result<(), MLError> {
    use ort::quantization::{quantize_dynamic, QuantType};
    
    quantize_dynamic(
        input_path,
        output_path,
        &[QuantType::QInt8], // Use 8-bit quantization
        true, // Optimize model
    )?;
    
    Ok(())
}

// 2. Use half precision for Candle models
let dtype = DType::F16; // Instead of F32
let model = CandleModel::load_from_safetensors(path, "bert", device, dtype).await?;

// 3. Implement model swapping for limited memory
pub struct ModelSwapper {
    active_models: HashMap<String, Box<dyn ModelTrait>>,
    cold_storage: HashMap<String, ModelMetadata>,
    max_active_models: usize,
}
```

### <issue>Slow Inference Performance</issue>
**Problem**: Model inference taking too long
**Solution**:
```rust
// 1. Enable GPU acceleration with optimal settings
let mut builder = SessionBuilder::new()?
    .with_optimization_level(ort::GraphOptimizationLevel::All)?
    .with_intra_threads(num_cpus::get())?
    .with_cuda(0)? // GPU acceleration
    .with_tensorrt(0)?; // TensorRT optimization

// 2. Use batch processing
pub async fn batch_inference(
    &self,
    requests: Vec<InferenceRequest>,
) -> Result<Vec<InferenceResponse>, MLError> {
    let batch_size = self.config.max_batch_size.min(requests.len());
    let mut responses = Vec::new();
    
    for batch in requests.chunks(batch_size) {
        let batch_inputs: Vec<_> = batch.iter()
            .map(|req| req.input_data.clone())
            .collect();
        
        let batch_outputs = self.manager
            .infer_batch(&batch[0].model_name, batch_inputs)
            .await?;
        
        for (req, output) in batch.iter().zip(batch_outputs.iter()) {
            responses.push(InferenceResponse {
                output_data: output.clone(),
                latency_ms: 0.0, // Shared batch latency
                model_name: req.model_name.clone(),
            });
        }
    }
    
    Ok(responses)
}

// 3. Implement model caching and warm-up
pub async fn warm_up_model(&self, model_name: &str) -> Result<(), MLError> {
    // Create dummy input to warm up the model
    let dummy_input = vec![0.1_f32; 512];
    let _ = self.infer(model_name, &dummy_input).await?;
    info!("Model {} warmed up successfully", model_name);
    Ok(())
}
```

### <issue>Cross-Platform Compatibility</issue>
**Problem**: Model not working across different platforms
**Solution**:
```rust
// 1. Use conditional compilation for platform-specific features
#[cfg(target_os = "linux")]
use ort::CUDAExecutionProvider;

#[cfg(target_os = "windows")]
use ort::DMLExecutionProvider;

#[cfg(target_arch = "wasm32")]
use ort::WasmExecutionProvider;

// 2. Implement fallback execution providers
pub fn get_execution_providers() -> Vec<String> {
    let mut providers = Vec::new();
    
    #[cfg(feature = "cuda")]
    providers.push("CUDA".to_string());
    
    #[cfg(feature = "tensorrt")]
    providers.push("TensorRT".to_string());
    
    #[cfg(feature = "directml")]
    providers.push("DML".to_string());
    
    providers.push("CPU".to_string()); // Always available
    providers
}

// 3. Abstract device creation
pub fn create_device(prefer_gpu: bool) -> Result<Device, MLError> {
    if prefer_gpu {
        #[cfg(feature = "cuda")]
        if let Ok(device) = Device::new_cuda(0) {
            return Ok(device);
        }
        
        #[cfg(feature = "metal")]
        if let Ok(device) = Device::new_metal(0) {
            return Ok(device);
        }
    }
    
    Ok(Device::Cpu) // Fallback to CPU
}
```

## <references>Deployment and Configuration</references>

### <concept>Production Configuration Management</concept>
```toml
# config.toml
[models]
cache_dir = "./models"
max_model_size = 2147483648  # 2GB

[[models.onnx_models]]
name = "bert-base"
path = "./models/bert-base.onnx"
input_shape = [1, 512]
quantized = true
execution_providers = ["CUDA", "CPU"]

[[models.candle_models]]
name = "llama2-7b"
path = "./models/llama2-7b"
model_type = "llama"
device = "cuda"
dtype = "f16"

[memory]
pool_size = 16
max_tensor_size = 536870912  # 512MB
enable_memory_mapping = true
gc_threshold = 0.8

[performance]
max_batch_size = 32
intra_op_threads = 8
enable_gpu = true
enable_tensorrt = true
```

### <concept>Docker Deployment</concept>
```dockerfile
FROM nvidia/cuda:12.2-runtime-ubuntu22.04

# Install Rust and dependencies
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

# Build with GPU support
RUN cargo build --release --features cuda

# Runtime configuration
COPY config.toml /app/
EXPOSE 8080

CMD ["./target/release/rust-ml-inference"]
```

## <references>See Also</references>
- [Vector Database Performance Optimization](vector-database-performance-optimization.md)
- [Hybrid Search Algorithm Implementation](hybrid-search-algorithm-implementation.md)
- [Production API Patterns](../patterns/production-api-patterns.md)
- [Async Patterns](../patterns/async-patterns.md)