# Hybrid Storage System Implementation

<meta>
  <title>Hybrid Storage System Implementation</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Scalable hybrid storage system from file-based MVP to production multi-layer architecture
- **Key Approach**: LSM-tree concepts, content-addressable storage, hybrid caching strategies
- **Core Benefits**: Deduplication, distributed caching, atomic operations, backup/recovery
- **When to use**: Applications requiring high-performance storage with advanced features
- **Related docs**: [Vector Database Technology Decision](vector-database-technology-decision.md)

## <implementation>Core Architecture</implementation>

### <pattern>Storage Layer Trait</pattern>

```rust
use async_trait::async_trait;

#[async_trait]
pub trait StorageLayer: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    async fn delete(&self, key: &[u8]) -> Result<(), StorageError>;
    async fn scan(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Deduplication error: {0}")]
    Deduplication(String),
}
```

### <pattern>MVP File Storage</pattern>

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub created: u64,
    pub modified: u64,
    pub checksum: String,
    pub content_type: Option<String>,
}

pub struct FileStorage {
    base_path: PathBuf,
    metadata: HashMap<Vec<u8>, FileMetadata>,
    metadata_path: PathBuf,
}

impl FileStorage {
    pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, StorageError> {
        let base_path = base_path.as_ref().to_path_buf();
        let metadata_path = base_path.join("metadata.json");
        
        tokio::fs::create_dir_all(&base_path).await?;
        
        let metadata = if metadata_path.exists() {
            let content = tokio::fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };
        
        Ok(FileStorage {
            base_path,
            metadata,
            metadata_path,
        })
    }
    
    fn key_to_path(&self, key: &[u8]) -> PathBuf {
        let hex_key = hex::encode(key);
        let (prefix, suffix) = hex_key.split_at(2);
        self.base_path.join(prefix).join(suffix)
    }
}

#[async_trait]
impl StorageLayer for FileStorage {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let path = self.key_to_path(key);
        match tokio::fs::read(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e)),
        }
    }
    
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let path = self.key_to_path(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&path, value).await?;
        
        // Update metadata atomically
        let metadata = FileMetadata {
            size: value.len() as u64,
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            checksum: hex::encode(sha2::Sha256::digest(value)),
            content_type: None,
        };
        
        self.save_metadata_atomic(key, metadata).await?;
        Ok(())
    }
}
```

## <implementation>Hybrid Caching System</implementation>

### <pattern>Multi-Level Cache</pattern>

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;

pub struct HybridCache {
    // L1: In-memory LRU cache
    memory_cache: Arc<RwLock<LruCache<Vec<u8>, Vec<u8>>>>,
    // L2: Redis distributed cache
    redis_cache: Arc<RedisCache>,
    // L3: Persistent storage
    storage: Arc<dyn StorageLayer>,
}

impl HybridCache {
    pub fn new(
        memory_capacity: usize,
        redis_cache: Arc<RedisCache>,
        storage: Arc<dyn StorageLayer>,
    ) -> Self {
        Self {
            memory_cache: Arc::new(RwLock::new(LruCache::new(memory_capacity))),
            redis_cache,
            storage,
        }
    }
}

#[async_trait]
impl StorageLayer for HybridCache {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        // L1: Check memory cache first
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(value) = cache.get(key) {
                return Ok(Some(value.clone()));
            }
        }
        
        // L2: Check Redis cache
        if let Some(value) = self.redis_cache.get(key).await? {
            // Promote to memory cache
            let mut cache = self.memory_cache.write().await;
            cache.put(key.to_vec(), value.clone());
            return Ok(Some(value));
        }
        
        // L3: Check storage layer
        if let Some(value) = self.storage.get(key).await? {
            // Populate caches
            let mut cache = self.memory_cache.write().await;
            cache.put(key.to_vec(), value.clone());
            self.redis_cache.put(key, &value, Some(Duration::from_secs(3600))).await?;
            return Ok(Some(value));
        }
        
        Ok(None)
    }
    
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        // Write to all layers
        self.storage.put(key, value).await?;
        self.redis_cache.put(key, value, Some(Duration::from_secs(3600))).await?;
        
        let mut cache = self.memory_cache.write().await;
        cache.put(key.to_vec(), value.to_vec());
        
        Ok(())
    }
}
```

### <pattern>Redis Cache Implementation</pattern>

```rust
use redis::{Client, Commands, Connection};
use std::time::Duration;

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(url: &str) -> Result<Self, StorageError> {
        let client = Client::open(url)
            .map_err(|e| StorageError::Cache(e.to_string()))?;
        
        Ok(RedisCache { client })
    }
    
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Cache(e.to_string()))?;
        
        let result: Option<Vec<u8>> = conn.get(key)
            .map_err(|e| StorageError::Cache(e.to_string()))?;
        
        Ok(result)
    }
    
    pub async fn put(
        &self,
        key: &[u8],
        value: &[u8],
        ttl: Option<Duration>,
    ) -> Result<(), StorageError> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Cache(e.to_string()))?;
        
        if let Some(ttl) = ttl {
            conn.set_ex(key, value, ttl.as_secs() as usize)
                .map_err(|e| StorageError::Cache(e.to_string()))?;
        } else {
            conn.set(key, value)
                .map_err(|e| StorageError::Cache(e.to_string()))?;
        }
        
        Ok(())
    }
}
```

## <implementation>Content-Addressable Storage</implementation>

### <pattern>Deduplication System</pattern>

```rust
use sha2::{Sha256, Digest};
use std::collections::HashMap;

pub struct ContentAddressableStorage {
    chunk_store: Arc<dyn StorageLayer>,
    manifest_store: Arc<dyn StorageLayer>,
    chunk_size_target: usize,
    min_chunk_size: usize,
    max_chunk_size: usize,
}

#[derive(Serialize, Deserialize)]
struct ChunkManifest {
    chunks: Vec<ChunkInfo>,
    total_size: usize,
    checksum: String,
}

#[derive(Serialize, Deserialize)]
struct ChunkInfo {
    hash: String,
    size: usize,
    offset: usize,
}

impl ContentAddressableStorage {
    pub async fn store_with_deduplication(
        &self,
        key: &[u8],
        data: &[u8],
    ) -> Result<(), StorageError> {
        // Create chunks using content-defined chunking
        let chunks = self.create_chunks(data);
        let mut manifest = ChunkManifest {
            chunks: Vec::new(),
            total_size: data.len(),
            checksum: hex::encode(Sha256::digest(data)),
        };
        
        let mut offset = 0;
        
        for chunk in chunks {
            let chunk_hash = hex::encode(Sha256::digest(&chunk));
            let chunk_key = chunk_hash.as_bytes();
            
            // Only store if chunk doesn't exist
            if self.chunk_store.get(chunk_key).await?.is_none() {
                self.chunk_store.put(chunk_key, &chunk).await?;
            }
            
            manifest.chunks.push(ChunkInfo {
                hash: chunk_hash,
                size: chunk.len(),
                offset,
            });
            
            offset += chunk.len();
        }
        
        // Store manifest
        let manifest_data = serde_json::to_vec(&manifest)?;
        self.manifest_store.put(key, &manifest_data).await?;
        
        Ok(())
    }
    
    pub async fn retrieve_deduplicated(
        &self,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, StorageError> {
        // Get manifest
        let manifest_data = match self.manifest_store.get(key).await? {
            Some(data) => data,
            None => return Ok(None),
        };
        
        let manifest: ChunkManifest = serde_json::from_slice(&manifest_data)?;
        
        // Reconstruct data from chunks
        let mut data = Vec::with_capacity(manifest.total_size);
        
        for chunk_info in manifest.chunks {
            let chunk_key = chunk_info.hash.as_bytes();
            let chunk_data = self.chunk_store.get(chunk_key).await?
                .ok_or_else(|| StorageError::Deduplication(
                    format!("Missing chunk: {}", chunk_info.hash)
                ))?;
            
            data.extend_from_slice(&chunk_data);
        }
        
        // Verify integrity
        let actual_checksum = hex::encode(Sha256::digest(&data));
        if actual_checksum != manifest.checksum {
            return Err(StorageError::Deduplication(
                "Checksum mismatch during reconstruction".to_string()
            ));
        }
        
        Ok(Some(data))
    }
    
    fn create_chunks(&self, data: &[u8]) -> Vec<Vec<u8>> {
        // Simplified content-defined chunking
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < data.len() {
            let end = std::cmp::min(start + self.chunk_size_target, data.len());
            chunks.push(data[start..end].to_vec());
            start = end;
        }
        
        chunks
    }
}
```

## <implementation>Backup and Recovery</implementation>

### <pattern>Backup System</pattern>

```rust
use std::path::Path;
use chrono::{DateTime, Utc};

pub struct BackupSystem {
    storage: Arc<dyn StorageLayer>,
    backup_storage: Arc<dyn StorageLayer>,
}

#[derive(Serialize, Deserialize)]
struct BackupManifest {
    timestamp: DateTime<Utc>,
    backup_type: BackupType,
    entries: Vec<BackupEntry>,
    checksum: String,
}

#[derive(Serialize, Deserialize)]
enum BackupType {
    Full,
    Incremental { since: DateTime<Utc> },
}

#[derive(Serialize, Deserialize)]
struct BackupEntry {
    key: Vec<u8>,
    checksum: String,
    size: usize,
    timestamp: DateTime<Utc>,
}

impl BackupSystem {
    pub async fn create_backup(
        &self,
        backup_type: BackupType,
    ) -> Result<String, StorageError> {
        let backup_id = format!("backup_{}", Utc::now().format("%Y%m%d_%H%M%S"));
        let mut entries = Vec::new();
        
        // Collect all entries for backup
        let all_entries = self.storage.scan(&[]).await?;
        
        for (key, value) in all_entries {
            let entry = BackupEntry {
                key: key.clone(),
                checksum: hex::encode(Sha256::digest(&value)),
                size: value.len(),
                timestamp: Utc::now(),
            };
            
            // Store data in backup storage
            let backup_key = format!("{}_{}", backup_id, hex::encode(&key));
            self.backup_storage.put(backup_key.as_bytes(), &value).await?;
            
            entries.push(entry);
        }
        
        // Create manifest
        let manifest = BackupManifest {
            timestamp: Utc::now(),
            backup_type,
            entries,
            checksum: String::new(), // Calculate from entries
        };
        
        let manifest_data = serde_json::to_vec(&manifest)?;
        let manifest_key = format!("{}_manifest", backup_id);
        self.backup_storage.put(manifest_key.as_bytes(), &manifest_data).await?;
        
        Ok(backup_id)
    }
    
    pub async fn restore_backup(
        &self,
        backup_id: &str,
    ) -> Result<(), StorageError> {
        // Load manifest
        let manifest_key = format!("{}_manifest", backup_id);
        let manifest_data = self.backup_storage.get(manifest_key.as_bytes()).await?
            .ok_or_else(|| StorageError::Database(
                format!("Backup manifest not found: {}", backup_id)
            ))?;
        
        let manifest: BackupManifest = serde_json::from_slice(&manifest_data)?;
        
        // Restore each entry
        for entry in manifest.entries {
            let backup_key = format!("{}_{}", backup_id, hex::encode(&entry.key));
            let value = self.backup_storage.get(backup_key.as_bytes()).await?
                .ok_or_else(|| StorageError::Database(
                    format!("Backup entry not found: {}", hex::encode(&entry.key))
                ))?;
            
            // Verify checksum
            let actual_checksum = hex::encode(Sha256::digest(&value));
            if actual_checksum != entry.checksum {
                return Err(StorageError::Database(
                    format!("Checksum mismatch for key: {}", hex::encode(&entry.key))
                ));
            }
            
            // Restore to main storage
            self.storage.put(&entry.key, &value).await?;
        }
        
        Ok(())
    }
}
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Cache Invalidation</issue>
**Problem**: Stale data in cache layers
**Solution**: Implement cache invalidation on writes:
```rust
async fn invalidate_cache(&self, key: &[u8]) -> Result<(), StorageError> {
    // Remove from memory cache
    let mut cache = self.memory_cache.write().await;
    cache.pop(key);
    
    // Remove from Redis cache
    self.redis_cache.delete(key).await?;
    
    Ok(())
}
```

### <issue>Chunk Corruption</issue>
**Problem**: Corrupted chunks during deduplication
**Solution**: Integrity verification during reconstruction:
```rust
fn verify_chunk_integrity(&self, chunk: &[u8], expected_hash: &str) -> Result<(), StorageError> {
    let actual_hash = hex::encode(Sha256::digest(chunk));
    if actual_hash != expected_hash {
        return Err(StorageError::Deduplication(
            format!("Chunk integrity check failed: expected {}, got {}", 
                   expected_hash, actual_hash)
        ));
    }
    Ok(())
}
```

## <constraints>Performance Considerations</constraints>

### <constraint>Memory Usage</constraint>
- LRU cache size should be tuned based on available memory
- Chunk size affects memory usage during deduplication
- Redis connection pooling prevents connection exhaustion

### <constraint>Disk I/O</constraint>
- Directory structure affects filesystem performance
- Async I/O prevents blocking on large operations
- Batch operations reduce I/O overhead

### <constraint>Network Latency</constraint>
- Redis cache reduces network calls to storage
- Connection pooling amortizes connection overhead
- Compression can reduce network bandwidth

## <references>See Also</references>
- [Vector Database Technology Decision](vector-database-technology-decision.md)
- [Production-Ready Rust API System](production-ready-rust-api-system.md)
- [Observability System Implementation](observability-system-implementation.md)
- [Multi-LLM Provider System](multi-llm-provider-system.md)