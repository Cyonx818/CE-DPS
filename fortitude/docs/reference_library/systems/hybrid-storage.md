# Hybrid Storage System Reference

<purpose>
Production-ready hybrid storage implementation scaling from file-based MVP to enterprise storage with caching, deduplication, and backup capabilities.
</purpose>

## Quick Start

<code-example purpose="minimal-mvp">
```rust
// MVP: Simple file storage
let storage = FileStorage::new("./data").await?;
storage.put(b"key", b"value").await?;
let value = storage.get(b"key").await?;
```
</code-example>

<code-example purpose="production-setup">
```rust
// Production: Full hybrid system
let config = StorageConfig {
    data_path: "./data".into(),
    redis_url: "redis://localhost".into(),
    sqlite_url: "sqlite://metadata.db".into(),
    memory_cache_size: 1000,
    cache_ttl: 3600,
    backup_schedule: BackupSchedule {
        interval_seconds: 3600,
        retention_days: 7,
        incremental: true,
    },
};
let system = HybridStorageSystem::new(config).await?;
```
</code-example>

## Core Architecture

<concept name="storage-trait">
All storage layers implement a common trait enabling composition and migration:

```rust
pub trait StorageLayer: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    async fn delete(&self, key: &[u8]) -> Result<(), StorageError>;
    async fn scan(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError>;
}
```
</concept>

## Storage Layers

### File Storage (MVP)

<implementation name="file-storage">
**Purpose**: Filesystem-based storage suitable for MVP and small-scale deployments.

**Key Features**:
- Content-addressed storage with hex-encoded keys
- Two-level directory structure to avoid filesystem limits
- JSON metadata tracking
- Atomic operations via temporary files

**Implementation Details**:
```rust
pub struct FileStorage {
    base_path: PathBuf,
    metadata: HashMap<Vec<u8>, FileMetadata>,
}

// Key mapping: key -> hex -> aa/bbccdd...
fn key_to_path(&self, key: &[u8]) -> PathBuf {
    let hex_key = hex::encode(key);
    let (prefix, suffix) = hex_key.split_at(2);
    self.base_path.join(prefix).join(suffix)
}
```

**Usage Pattern**: Start here for rapid prototyping, migrate to hybrid when scaling.
</implementation>

### Cache Layer

<implementation name="hybrid-cache">
**Purpose**: Multi-tier caching with in-memory LRU and Redis for distributed systems.

**Architecture**:
```
Request → Memory Cache (L1) → Redis Cache (L2) → Storage Layer
```

**Key Features**:
- Automatic cache promotion between tiers
- Configurable TTL and size limits
- Thread-safe concurrent access
- Graceful degradation on cache failure

**Critical Configuration**:
- `memory_limit`: Number of items in LRU cache
- `ttl_seconds`: Redis key expiration time
- Cache aside pattern for consistency
</implementation>

### Deduplication System

<implementation name="content-deduplication">
**Purpose**: Content-defined chunking for storage efficiency and bandwidth optimization.

**Algorithm**:
1. **Rolling Hash Chunking**: Variable-size chunks based on content patterns
2. **SHA-256 Fingerprinting**: Identify duplicate chunks
3. **Reference Counting**: Track chunk usage for safe deletion
4. **Manifest Storage**: Reconstruct original data from chunks

**Configuration**:
```rust
const MIN_CHUNK_SIZE: usize = 4096;    // 4KB minimum
const MAX_CHUNK_SIZE: usize = 65536;   // 64KB maximum
const WINDOW_SIZE: usize = 64;         // Rolling hash window
```

**Performance Impact**: 
- Storage savings: 30-70% for similar content
- CPU overhead: ~10-15% for chunking
- Best for: Large files, versioned content, backups
</implementation>

## Database Integration

### SQLite Metadata Store

<pattern name="metadata-management">
**Purpose**: Efficient metadata queries and deduplication tracking.

**Schema**:
```sql
CREATE TABLE storage_metadata (
    key BLOB PRIMARY KEY,
    size INTEGER NOT NULL,
    created INTEGER NOT NULL,
    modified INTEGER NOT NULL,
    checksum TEXT NOT NULL,
    content_type TEXT,
    dedup_hash BLOB,
    INDEX idx_checksum (checksum),
    INDEX idx_dedup_hash (dedup_hash)
);
```

**Query Patterns**:
- Find duplicates by checksum
- List keys by prefix
- Track access patterns
- Manage retention policies
</pattern>

### DuckDB Analytics

<pattern name="analytics-layer">
**Purpose**: OLAP queries on storage access patterns and performance metrics.

**Use Cases**:
- Access frequency analysis
- Cost optimization insights
- Performance bottleneck identification
- Capacity planning

**Integration**: Asynchronous ETL from access logs to analytics tables.
</pattern>

## Backup & Recovery

<implementation name="backup-system">
**Strategy**: Configurable full and incremental backups with verification.

**Backup Flow**:
1. Create backup manifest with checksums
2. Copy data to backup storage (can be remote)
3. Verify integrity via checksums
4. Maintain retention policy
5. Support point-in-time recovery

**Critical Features**:
- Backup verification before deletion
- Parallel backup for large datasets
- Compression support
- Cross-region replication ready
</implementation>

## Migration Strategy

<pattern name="zero-downtime-migration">
**From File to Hybrid**:

```rust
// 1. Initialize new system alongside old
let old_storage = FileStorage::new("./data").await?;
let new_system = HybridStorageSystem::new(config).await?;

// 2. Migrate with verification
let migrator = StorageMigrator::new(old_storage, new_system);
migrator.migrate_data().await?;
assert!(migrator.verify_migration().await?);

// 3. Switch traffic to new system
// 4. Keep old system as fallback
```

**Key Principles**:
- Read from both, write to new
- Verify data integrity
- Gradual traffic migration
- Rollback capability
</pattern>

## Performance Optimization

<optimization-guide>
**Caching Strategy**:
- Hot data in memory (microseconds)
- Warm data in Redis (milliseconds)
- Cold data in storage (tens of milliseconds)

**Deduplication Tuning**:
- Adjust chunk sizes based on data patterns
- Skip dedup for small files (<1KB)
- Batch dedup operations

**Concurrency**:
- Async/await throughout
- Connection pooling for databases
- Parallel chunk processing
- Lock-free read paths where possible
</optimization-guide>

## Error Handling

<error-patterns>
```rust
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

**Recovery Strategies**:
- Cache failures: Fallback to storage
- Database failures: Queue writes, serve from cache
- Storage failures: Serve from cache, alert operators
- Deduplication failures: Store without dedup
</error-patterns>

## Production Checklist

<checklist>
- [ ] Configure monitoring and alerts
- [ ] Set up backup schedule and verify restores
- [ ] Tune cache sizes based on workload
- [ ] Configure rate limiting for API access
- [ ] Implement access control and encryption
- [ ] Set up replication for high availability
- [ ] Plan capacity based on growth projections
- [ ] Document runbooks for common operations
</checklist>

## Common Pitfalls

<pitfalls>
1. **Cache Stampede**: Use cache warming and request coalescing
2. **Memory Leaks**: Implement cache eviction and connection limits
3. **Backup Failures**: Monitor and test restore procedures regularly
4. **Dedup Overhead**: Profile and skip for small files
5. **Lock Contention**: Use read-write locks and sharding
</pitfalls>

## See Also

<references>
- RocksDB documentation for LSM-tree concepts
- Redis persistence documentation
- Content-defined chunking papers (Rabin fingerprinting)
- Backup best practices for distributed systems
</references>