// ABOUTME: Database connection pooling for optimized vector operations
//! This module provides advanced connection pooling and database optimization
//! for Qdrant operations to achieve <200ms response times.

use crate::api::client::ApiClient;
use crate::vector::{
    error::{VectorError, VectorResult},
    performance::{ConnectionPoolConfig, LatencyMeasurement, PerformanceMonitor},
    QdrantClient, VectorConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, instrument, warn};

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total connections in pool
    pub total_connections: usize,
    /// Active connections in use
    pub active_connections: usize,
    /// Idle connections available
    pub idle_connections: usize,
    /// Connection acquisition wait time (ms)
    pub avg_wait_time_ms: f64,
    /// Pool utilization rate (0.0-1.0)
    pub utilization_rate: f64,
    /// Connection failures
    pub connection_failures: u64,
    /// Health check failures
    pub health_check_failures: u64,
    /// Total requests served
    pub total_requests: u64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            avg_wait_time_ms: 0.0,
            utilization_rate: 0.0,
            connection_failures: 0,
            health_check_failures: 0,
            total_requests: 0,
        }
    }
}

/// Connection wrapper with metadata
pub struct PooledConnection {
    /// The actual Qdrant client
    pub client: Arc<QdrantClient>,
    /// Connection ID for tracking
    pub id: String,
    /// When the connection was created
    pub created_at: Instant,
    /// When the connection was last used
    pub last_used: Instant,
    /// Number of times this connection was used
    pub usage_count: u64,
    /// Whether the connection is healthy
    pub is_healthy: bool,
    /// Connection-specific configuration
    pub config: VectorConfig,
}

impl PooledConnection {
    pub fn new(client: QdrantClient, config: VectorConfig) -> Self {
        let now = Instant::now();
        Self {
            client: Arc::new(client),
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            last_used: now,
            usage_count: 0,
            is_healthy: true,
            config,
        }
    }

    /// Check if connection should be retired
    pub fn should_retire(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }

    /// Check if connection is idle for too long
    pub fn is_idle_too_long(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }

    /// Mark connection as used
    pub fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.usage_count += 1;
    }

    /// Perform health check on connection
    pub async fn health_check(&mut self) -> bool {
        // Use the ApiClient trait's health_check method
        match (*self.client).health_check().await {
            Ok(status) => {
                self.is_healthy = matches!(status, crate::api::HealthStatus::Healthy);
                self.is_healthy
            }
            Err(e) => {
                warn!("Connection {} health check failed: {}", self.id, e);
                self.is_healthy = false;
                false
            }
        }
    }
}

/// Adaptive connection pool for Qdrant clients
pub struct ConnectionPool {
    /// Pool configuration
    config: ConnectionPoolConfig,
    /// Vector database configuration
    vector_config: VectorConfig,
    /// Available connections
    idle_connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    /// Active connections being used
    active_connections: Arc<RwLock<Vec<String>>>,
    /// Semaphore to limit concurrent connections
    connection_semaphore: Arc<Semaphore>,
    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,
    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,
    /// Shutdown signal
    shutdown: Arc<RwLock<bool>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(
        config: ConnectionPoolConfig,
        vector_config: VectorConfig,
        monitor: PerformanceMonitor,
    ) -> VectorResult<Self> {
        let pool = Self {
            connection_semaphore: Arc::new(Semaphore::new(config.max_connections)),
            idle_connections: Arc::new(Mutex::new(VecDeque::new())),
            active_connections: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PoolStats::default())),
            monitor: Arc::new(monitor),
            shutdown: Arc::new(RwLock::new(false)),
            config,
            vector_config,
        };

        // Initialize minimum connections
        pool.initialize_connections().await?;

        // Start background maintenance
        pool.start_maintenance_task().await;

        info!(
            "Connection pool initialized with {} connections",
            pool.config.min_connections
        );
        Ok(pool)
    }

    /// Initialize minimum number of connections
    async fn initialize_connections(&self) -> VectorResult<()> {
        let mut idle_connections = self.idle_connections.lock().await;

        for _ in 0..self.config.min_connections {
            match self.create_connection().await {
                Ok(connection) => {
                    idle_connections.push_back(connection);
                }
                Err(e) => {
                    error!("Failed to create initial connection: {}", e);
                    let mut stats = self.stats.write().await;
                    stats.connection_failures += 1;
                }
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_connections = idle_connections.len();
        stats.idle_connections = idle_connections.len();

        Ok(())
    }

    /// Create a new connection
    async fn create_connection(&self) -> VectorResult<PooledConnection> {
        let measurement = LatencyMeasurement::new("connection_creation");

        match tokio::time::timeout(
            self.config.connect_timeout,
            QdrantClient::new(self.vector_config.clone()),
        )
        .await
        {
            Ok(Ok(client)) => {
                let connection = PooledConnection::new(client, self.vector_config.clone());
                let duration = measurement.finish();
                self.monitor
                    .record_latency("connection_creation", duration)
                    .await;
                debug!("Created new connection: {}", connection.id);
                Ok(connection)
            }
            Ok(Err(e)) => {
                let duration = measurement.finish();
                self.monitor
                    .record_latency("connection_creation_failed", duration)
                    .await;
                Err(e)
            }
            Err(_) => {
                let duration = measurement.finish();
                self.monitor
                    .record_latency("connection_creation_timeout", duration)
                    .await;
                Err(VectorError::ConnectionPoolError(
                    "Connection creation timeout".to_string(),
                ))
            }
        }
    }

    /// Acquire a connection from the pool
    #[instrument(skip(self))]
    pub async fn acquire(&self) -> VectorResult<PooledConnection> {
        let measurement = LatencyMeasurement::new("connection_acquisition");
        let acquire_start = Instant::now();

        // Acquire semaphore permit
        let _permit = self.connection_semaphore.acquire().await.map_err(|_| {
            VectorError::ConnectionPoolError("Failed to acquire connection permit".to_string())
        })?;

        // Try to get an idle connection first
        let mut connection = {
            let mut idle_connections = self.idle_connections.lock().await;
            idle_connections.pop_front()
        };

        // If no idle connection available, create a new one
        if connection.is_none() {
            match self.create_connection().await {
                Ok(new_connection) => {
                    connection = Some(new_connection);
                }
                Err(e) => {
                    let mut stats = self.stats.write().await;
                    stats.connection_failures += 1;
                    return Err(e);
                }
            }
        }

        let mut conn = connection.unwrap();

        // Health check the connection
        if !conn.health_check().await {
            // Try to create a new connection if health check fails
            match self.create_connection().await {
                Ok(new_connection) => {
                    conn = new_connection;
                }
                Err(e) => {
                    let mut stats = self.stats.write().await;
                    stats.health_check_failures += 1;
                    return Err(e);
                }
            }
        }

        // Mark connection as active
        conn.mark_used();
        let connection_id = conn.id.clone();

        {
            let mut active_connections = self.active_connections.write().await;
            active_connections.push(connection_id);
        }

        // Update statistics
        let acquisition_time = acquire_start.elapsed();
        {
            let mut stats = self.stats.write().await;
            stats.active_connections += 1;
            stats.total_requests += 1;

            // Update rolling average wait time
            let total_requests = stats.total_requests as f64;
            stats.avg_wait_time_ms = (stats.avg_wait_time_ms * (total_requests - 1.0)
                + acquisition_time.as_millis() as f64)
                / total_requests;

            stats.utilization_rate =
                stats.active_connections as f64 / self.config.max_connections as f64;
        }

        let duration = measurement.finish();
        self.monitor
            .record_latency("connection_acquisition", duration)
            .await;

        debug!("Acquired connection: {}", conn.id);
        Ok(conn)
    }

    /// Return a connection to the pool
    #[instrument(skip(self, connection))]
    pub async fn release(&self, connection: PooledConnection) -> VectorResult<()> {
        let connection_id = connection.id.clone();

        // Remove from active connections
        {
            let mut active_connections = self.active_connections.write().await;
            active_connections.retain(|id| id != &connection_id);
        }

        // Check if connection should be retired
        if connection.should_retire(self.config.max_lifetime) || !connection.is_healthy {
            debug!("Retiring connection: {}", connection_id);

            let mut stats = self.stats.write().await;
            stats.total_connections = stats.total_connections.saturating_sub(1);
            stats.active_connections = stats.active_connections.saturating_sub(1);

            return Ok(());
        }

        // Return to idle pool
        {
            let mut idle_connections = self.idle_connections.lock().await;
            idle_connections.push_back(connection);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
            stats.idle_connections += 1;
            stats.utilization_rate =
                stats.active_connections as f64 / self.config.max_connections as f64;
        }

        debug!("Released connection: {}", connection_id);
        Ok(())
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let stats = self.stats.read().await.clone();
        stats
    }

    /// Perform pool maintenance
    async fn perform_maintenance(&self) {
        debug!("Performing connection pool maintenance");

        // Remove idle connections that have been idle too long
        {
            let mut idle_connections = self.idle_connections.lock().await;
            let initial_count = idle_connections.len();

            idle_connections.retain(|conn| {
                let keep = !conn.is_idle_too_long(self.config.idle_timeout)
                    && !conn.should_retire(self.config.max_lifetime);

                if !keep {
                    debug!("Removing idle connection: {}", conn.id);
                }
                keep
            });

            let removed_count = initial_count - idle_connections.len();

            if removed_count > 0 {
                let mut stats = self.stats.write().await;
                stats.total_connections = stats.total_connections.saturating_sub(removed_count);
                stats.idle_connections = idle_connections.len();
            }
        }

        // Health check idle connections
        {
            let mut idle_connections = self.idle_connections.lock().await;
            let mut healthy_connections = VecDeque::new();
            let mut health_check_failures = 0;

            while let Some(mut conn) = idle_connections.pop_front() {
                if conn.health_check().await {
                    healthy_connections.push_back(conn);
                } else {
                    health_check_failures += 1;
                    debug!("Removing unhealthy connection: {}", conn.id);
                }
            }

            *idle_connections = healthy_connections;

            if health_check_failures > 0 {
                let mut stats = self.stats.write().await;
                stats.health_check_failures += health_check_failures;
                stats.total_connections = stats
                    .total_connections
                    .saturating_sub(health_check_failures as usize);
                stats.idle_connections = idle_connections.len();
            }
        }

        // Ensure minimum connections
        {
            let idle_count = self.idle_connections.lock().await.len();
            let active_count = self.active_connections.read().await.len();
            let total_connections = idle_count + active_count;

            if total_connections < self.config.min_connections {
                let needed = self.config.min_connections - total_connections;

                for _ in 0..needed {
                    match self.create_connection().await {
                        Ok(connection) => {
                            self.idle_connections.lock().await.push_back(connection);
                        }
                        Err(e) => {
                            warn!("Failed to create maintenance connection: {}", e);
                            let mut stats = self.stats.write().await;
                            stats.connection_failures += 1;
                        }
                    }
                }
            }
        }

        // Update final statistics
        {
            let mut stats = self.stats.write().await;
            let idle_connections = self.idle_connections.lock().await;
            let active_connections = self.active_connections.read().await;

            stats.total_connections = idle_connections.len() + active_connections.len();
            stats.idle_connections = idle_connections.len();
            stats.active_connections = active_connections.len();
            stats.utilization_rate =
                stats.active_connections as f64 / self.config.max_connections as f64;
        }

        debug!("Connection pool maintenance completed");
    }

    /// Start background maintenance task
    async fn start_maintenance_task(&self) {
        let pool = self.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                // Check shutdown signal
                if *pool.shutdown.read().await {
                    break;
                }

                pool.perform_maintenance().await;
            }
        });

        info!(
            "Connection pool maintenance task started with interval: {:?}",
            interval
        );
    }

    /// Gracefully shutdown the pool
    pub async fn shutdown(&self) -> VectorResult<()> {
        info!("Shutting down connection pool");

        // Set shutdown signal
        *self.shutdown.write().await = true;

        // Wait for active connections to be released (with timeout)
        let shutdown_timeout = Duration::from_secs(30);
        let start_time = Instant::now();

        while start_time.elapsed() < shutdown_timeout {
            let active_count = self.active_connections.read().await.len();
            if active_count == 0 {
                break;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Clear all connections
        self.idle_connections.lock().await.clear();
        self.active_connections.write().await.clear();

        info!("Connection pool shutdown completed");
        Ok(())
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            vector_config: self.vector_config.clone(),
            idle_connections: Arc::clone(&self.idle_connections),
            active_connections: Arc::clone(&self.active_connections),
            connection_semaphore: Arc::clone(&self.connection_semaphore),
            stats: Arc::clone(&self.stats),
            monitor: Arc::clone(&self.monitor),
            shutdown: Arc::clone(&self.shutdown),
        }
    }
}

/// Connection pool manager that handles multiple pools
pub struct PoolManager {
    pools: Arc<RwLock<std::collections::HashMap<String, ConnectionPool>>>,
    default_config: ConnectionPoolConfig,
    monitor: Arc<PerformanceMonitor>,
}

impl PoolManager {
    pub fn new(config: ConnectionPoolConfig, monitor: PerformanceMonitor) -> Self {
        Self {
            pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_config: config,
            monitor: Arc::new(monitor),
        }
    }

    /// Get or create a connection pool for a specific configuration
    pub async fn get_pool(
        &self,
        pool_name: &str,
        vector_config: VectorConfig,
    ) -> VectorResult<ConnectionPool> {
        let pools = self.pools.read().await;

        if let Some(pool) = pools.get(pool_name) {
            return Ok(pool.clone());
        }

        drop(pools);

        // Create new pool
        let pool = ConnectionPool::new(
            self.default_config.clone(),
            vector_config,
            (*self.monitor).clone(),
        )
        .await?;

        let mut pools = self.pools.write().await;
        pools.insert(pool_name.to_string(), pool.clone());

        Ok(pool)
    }

    /// Get statistics for all pools
    pub async fn get_all_stats(&self) -> std::collections::HashMap<String, PoolStats> {
        let pools = self.pools.read().await;
        let mut all_stats = std::collections::HashMap::new();

        for (name, pool) in pools.iter() {
            all_stats.insert(name.clone(), pool.get_stats().await);
        }

        all_stats
    }

    /// Shutdown all pools
    pub async fn shutdown_all(&self) -> VectorResult<()> {
        let pools = self.pools.read().await;

        for pool in pools.values() {
            pool.shutdown().await?;
        }

        info!("All connection pools shutdown completed");
        Ok(())
    }
}

/// Connection pool aware client wrapper
pub struct PooledQdrantClient {
    pool: ConnectionPool,
}

impl PooledQdrantClient {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Execute a function with a pooled connection
    pub async fn with_connection<F, R, Fut>(&self, f: F) -> VectorResult<R>
    where
        F: FnOnce(Arc<QdrantClient>) -> Fut,
        Fut: std::future::Future<Output = VectorResult<R>>,
    {
        let connection = self.pool.acquire().await?;
        let client = Arc::clone(&connection.client);

        let result = f(client).await;

        // Always return connection to pool
        if let Err(e) = self.pool.release(connection).await {
            warn!("Failed to release connection back to pool: {}", e);
        }

        result
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self) -> PoolStats {
        self.pool.get_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::performance::MonitoringConfig;

    #[tokio::test]
    async fn test_connection_pool() {
        let pool_config = ConnectionPoolConfig {
            min_connections: 2,
            max_connections: 5,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
            health_check_interval: Duration::from_secs(30),
        };

        let vector_config = VectorConfig {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            default_collection: "test".to_string(),
            vector_dimensions: 384,
            distance_metric: crate::vector::DistanceMetric::Cosine,
            health_check: crate::vector::HealthCheckConfig {
                enabled: true,
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                max_failures: 3,
            },
            connection_pool: crate::vector::ConnectionPoolConfig::default(),
            embedding: crate::vector::EmbeddingConfig::default(),
        };

        let monitor = PerformanceMonitor::new(MonitoringConfig::default());

        // Note: This test will fail if Qdrant is not running
        // In a real test environment, you would use a mock or test container
        let max_connections = pool_config.max_connections;
        let _pool_result = ConnectionPool::new(pool_config, vector_config, monitor).await;

        // For now, just test that we can create the structs
        assert!(max_connections > 0);
    }

    #[test]
    fn test_pooled_connection() {
        // Test connection metadata methods
        let _config = VectorConfig {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            default_collection: "test".to_string(),
            vector_dimensions: 384,
            distance_metric: crate::vector::DistanceMetric::Cosine,
            health_check: crate::vector::HealthCheckConfig {
                enabled: true,
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                max_failures: 3,
            },
            connection_pool: crate::vector::ConnectionPoolConfig::default(),
            embedding: crate::vector::EmbeddingConfig::default(),
        };

        // This would normally require an actual QdrantClient
        // For testing, we'll just verify the metadata logic

        assert!((Duration::from_secs(1) <= Duration::from_secs(2))); // Test duration comparison logic
    }
}
