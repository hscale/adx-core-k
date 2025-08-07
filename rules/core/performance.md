# ⚡ Performance Rules - Sub-100ms Excellence

## Performance Mandate

> **"Every API response under 100ms. Every database query under 10ms. No exceptions."**

ADX Core is built for enterprise scale with strict performance requirements.

## 🎯 Performance Targets

### Response Time Requirements
- **API Gateway**: < 50ms (p95)
- **Authentication**: < 30ms (p95)
- **User Operations**: < 100ms (p95)
- **File Operations**: < 200ms (p95)
- **Workflow Triggers**: < 150ms (p95)
- **Database Queries**: < 10ms (p95)

### Throughput Requirements
- **Concurrent Users**: 10,000+ per service
- **Requests per Second**: 1,000+ per service
- **Database Connections**: 100+ per service
- **Memory Usage**: < 512MB per service
- **CPU Usage**: < 50% under normal load

## 🚀 Database Performance

### 1. Query Optimization
```rust
// ✅ REQUIRED: Use proper indexing strategy
// Database schema with performance indexes
CREATE INDEX CONCURRENTLY idx_users_tenant_email 
ON users (tenant_id, email);

CREATE INDEX CONCURRENTLY idx_users_tenant_active 
ON users (tenant_id, is_active) 
WHERE is_active = true;

CREATE INDEX CONCURRENTLY idx_files_tenant_owner_created 
ON files (tenant_id, owner_id, created_at DESC);

// ✅ REQUIRED: Optimized queries with LIMIT
pub async fn get_users_paginated(
    db: &PgPool,
    tenant_id: Uuid,
    page: u32,
    page_size: u32,
) -> Result<Vec<User>, DatabaseError> {
    // Validate page size (prevent large queries)
    let page_size = page_size.min(100); // Max 100 items per page
    let offset = page * page_size;
    
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, tenant_id, email, name, is_active, created_at, updated_at
        FROM users 
        WHERE tenant_id = $1 
        AND is_active = true
        ORDER BY created_at DESC 
        LIMIT $2 OFFSET $3
        "#,
        tenant_id,
        page_size as i64,
        offset as i64
    )
    .fetch_all(db)
    .await?;
    
    Ok(users)
}

// ✅ REQUIRED: Use prepared statements for repeated queries
pub struct UserRepository {
    get_user_by_id: sqlx::query::QueryAs<'static, Postgres, User, (Uuid, Uuid)>,
    create_user: sqlx::query::Query<'static, Postgres, (String, String, Uuid)>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            get_user_by_id: sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE id = $1 AND tenant_id = $2"
            ),
            create_user: sqlx::query!(
                "INSERT INTO users (email, name, tenant_id) VALUES ($1, $2, $3) RETURNING id"
            ),
        }
    }
    
    pub async fn get_user(&self, db: &PgPool, user_id: Uuid, tenant_id: Uuid) -> Result<User, DatabaseError> {
        let start = Instant::now();
        
        let user = self.get_user_by_id
            .bind(user_id)
            .bind(tenant_id)
            .fetch_optional(db)
            .await?
            .ok_or(DatabaseError::NotFound)?;
        
        let duration = start.elapsed();
        if duration > Duration::from_millis(10) {
            tracing::warn!(
                duration_ms = duration.as_millis(),
                query = "get_user_by_id",
                "Slow database query detected"
            );
        }
        
        Ok(user)
    }
}
```

### 2. Connection Pool Optimization
```rust
// ✅ REQUIRED: Optimized connection pool configuration
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,        // High concurrency
            min_connections: 10,         // Always ready connections
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),    // 10 minutes
            max_lifetime: Duration::from_secs(1800),   // 30 minutes
        }
    }
}

// ✅ REQUIRED: Connection pool monitoring
pub async fn monitor_database_performance(pool: &PgPool) {
    let stats = pool.options();
    
    metrics::gauge!("db_connections_active", stats.get_max_connections() as f64);
    metrics::gauge!("db_connections_idle", stats.get_min_connections() as f64);
    
    // Alert if connection pool is under pressure
    if stats.get_max_connections() > 80 {
        tracing::warn!(
            active_connections = stats.get_max_connections(),
            "Database connection pool under pressure"
        );
    }
}
```

## 💾 Caching Strategy

### 1. Multi-Layer Caching
```rust
// ✅ REQUIRED: Intelligent caching hierarchy
pub struct CacheManager {
    // L1: In-memory cache (fastest)
    memory_cache: Arc<RwLock<LruCache<String, CachedValue>>>,
    
    // L2: Redis cache (fast, shared)
    redis: Arc<RedisManager>,
    
    // L3: Database (source of truth)
    database: Arc<DatabaseManager>,
}

impl CacheManager {
    // ✅ Cache-aside pattern with performance monitoring
    pub async fn get_user_permissions(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Vec<Permission>, CacheError> {
        let cache_key = format!("permissions:{}:{}", user_id, tenant_id);
        let start = Instant::now();
        
        // L1: Check memory cache (< 1ms)
        {
            let cache = self.memory_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired() {
                    metrics::counter!("cache_hits", "layer" => "memory").increment(1);
                    return Ok(cached.value.clone());
                }
            }
        }
        
        // L2: Check Redis cache (< 5ms)
        if let Ok(cached) = self.redis.get::<Vec<Permission>>(&cache_key).await {
            // Store in L1 for next time
            {
                let mut cache = self.memory_cache.write().await;
                cache.put(cache_key.clone(), CachedValue::new(cached.clone(), Duration::from_secs(300)));
            }
            
            metrics::counter!("cache_hits", "layer" => "redis").increment(1);
            return Ok(cached);
        }
        
        // L3: Database fallback
        let permissions = self.database.get_user_permissions(user_id, tenant_id).await?;
        
        // Store in both caches
        self.redis.set_with_ttl(&cache_key, &permissions, 600).await?; // 10 minutes
        {
            let mut cache = self.memory_cache.write().await;
            cache.put(cache_key, CachedValue::new(permissions.clone(), Duration::from_secs(300)));
        }
        
        let duration = start.elapsed();
        metrics::histogram!("cache_miss_duration", duration.as_millis() as f64);
        metrics::counter!("cache_misses").increment(1);
        
        Ok(permissions)
    }
}

// ✅ REQUIRED: Cache invalidation strategy
pub async fn invalidate_user_cache(user_id: Uuid, tenant_id: Uuid) -> Result<(), CacheError> {
    let patterns = vec![
        format!("user:{}:*", user_id),
        format!("permissions:{}:*", user_id),
        format!("roles:{}:*", user_id),
        format!("tenant:{}:users", tenant_id),
    ];
    
    for pattern in patterns {
        // Invalidate Redis
        REDIS.delete_pattern(&pattern).await?;
        
        // Invalidate memory cache
        {
            let mut cache = MEMORY_CACHE.write().await;
            cache.retain(|key, _| !key.starts_with(&pattern.replace("*", "")));
        }
    }
    
    Ok(())
}
```

### 2. Smart Cache Warming
```rust
// ✅ REQUIRED: Proactive cache warming for hot data
#[workflow]
pub async fn cache_warming_workflow(
    ctx: &mut WfContext,
    _: (),
) -> WorkflowResult<()> {
    // Warm up frequently accessed permissions
    let hot_users = ctx.activity(get_hot_users_activity, ()).await?;
    
    for user in hot_users {
        ctx.activity(
            warm_user_permissions_activity,
            (user.id, user.tenant_id)
        ).await?;
    }
    
    // Warm up tenant configurations
    let active_tenants = ctx.activity(get_active_tenants_activity, ()).await?;
    
    for tenant in active_tenants {
        ctx.activity(warm_tenant_config_activity, tenant.id).await?;
    }
    
    // Schedule next warming cycle (every 15 minutes)
    ctx.timer(Duration::from_secs(900)).await?;
    ctx.continue_as_new(cache_warming_workflow, ()).await?;
    
    Ok(())
}

#[activity]
pub async fn warm_user_permissions_activity(
    input: (Uuid, Uuid),
) -> Result<(), ActivityError> {
    let (user_id, tenant_id) = input;
    
    // Pre-load permissions into cache
    let _permissions = CACHE_MANAGER
        .get_user_permissions(user_id, tenant_id)
        .await?;
    
    // Pre-load user roles
    let _roles = CACHE_MANAGER
        .get_user_roles(user_id, tenant_id)
        .await?;
    
    Ok(())
}
```

## 🔄 Async Performance

### 1. Concurrent Processing
```rust
// ✅ REQUIRED: Use async concurrency for I/O operations
pub async fn process_batch_operations(
    operations: Vec<Operation>,
) -> Result<Vec<OperationResult>, ProcessingError> {
    // Process operations concurrently (but limit concurrency)
    let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent operations
    
    let tasks: Vec<_> = operations
        .into_iter()
        .map(|operation| {
            let semaphore = semaphore.clone();
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                process_single_operation(operation).await
            })
        })
        .collect();
    
    // Wait for all operations to complete
    let results = futures::future::join_all(tasks).await;
    
    // Collect successful results
    let mut operation_results = Vec::new();
    for result in results {
        match result {
            Ok(Ok(op_result)) => operation_results.push(op_result),
            Ok(Err(e)) => return Err(ProcessingError::OperationFailed(e)),
            Err(e) => return Err(ProcessingError::TaskFailed(e)),
        }
    }
    
    Ok(operation_results)
}

// ✅ REQUIRED: Streaming for large datasets
pub async fn stream_large_dataset(
    tenant_id: Uuid,
    filters: DataFilters,
) -> Result<impl Stream<Item = Result<DataRow, DatabaseError>>, DatabaseError> {
    let query = sqlx::query_as!(
        DataRow,
        r#"
        SELECT * FROM large_table 
        WHERE tenant_id = $1 
        AND created_at >= $2 
        ORDER BY created_at
        "#,
        tenant_id,
        filters.start_date
    );
    
    // Use fetch() instead of fetch_all() for streaming
    let stream = query.fetch(&*DATABASE);
    
    Ok(stream)
}
```

### 2. Memory-Efficient Processing
```rust
// ✅ REQUIRED: Process large files without loading into memory
pub async fn process_large_file(
    file_id: Uuid,
    tenant_id: Uuid,
) -> Result<ProcessingResult, FileError> {
    // Stream file processing
    let file_stream = get_file_stream(file_id, tenant_id).await?;
    let mut processor = FileProcessor::new();
    
    // Process in chunks to limit memory usage
    const CHUNK_SIZE: usize = 8192; // 8KB chunks
    let mut buffer = vec![0; CHUNK_SIZE];
    let mut total_processed = 0;
    
    let mut reader = tokio::io::BufReader::new(file_stream);
    
    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break; // EOF
        }
        
        // Process chunk
        processor.process_chunk(&buffer[..bytes_read]).await?;
        total_processed += bytes_read;
        
        // Yield control to prevent blocking
        if total_processed % (CHUNK_SIZE * 100) == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    Ok(processor.finish().await?)
}

// ✅ REQUIRED: Efficient JSON streaming
pub async fn stream_json_response(
    data: impl Stream<Item = Result<serde_json::Value, DatabaseError>>,
) -> Result<impl Stream<Item = Result<Bytes, std::io::Error>>, ApiError> {
    let stream = data.map(|item| {
        match item {
            Ok(value) => {
                let json = serde_json::to_string(&value)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                Ok(Bytes::from(json + "\n"))
            }
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    });
    
    Ok(stream)
}
```

## 📊 Performance Monitoring

### 1. Metrics Collection
```rust
// ✅ REQUIRED: Comprehensive performance metrics
use metrics::{counter, histogram, gauge};

// ✅ HTTP request metrics
pub async fn request_metrics_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    
    // Process request
    let response = next.run(req).await;
    
    // Record metrics
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    histogram!("http_request_duration_ms", duration.as_millis() as f64)
        .with_tag("method", method)
        .with_tag("path", path)
        .with_tag("status", status.to_string());
    
    counter!("http_requests_total")
        .with_tag("method", method)
        .with_tag("status", status.to_string())
        .increment(1);
    
    if duration > Duration::from_millis(100) {
        tracing::warn!(
            duration_ms = duration.as_millis(),
            path = path,
            method = method,
            "Slow HTTP request detected"
        );
    }
    
    Ok(response)
}

// ✅ REQUIRED: Database query metrics
pub async fn query_with_metrics<T>(
    query_name: &str,
    query_fn: impl Future<Output = Result<T, sqlx::Error>>,
) -> Result<T, sqlx::Error> {
    let start = Instant::now();
    
    let result = query_fn.await;
    
    let duration = start.elapsed();
    histogram!("db_query_duration_ms", duration.as_millis() as f64)
        .with_tag("query", query_name);
    
    match &result {
        Ok(_) => counter!("db_queries_success").with_tag("query", query_name).increment(1),
        Err(_) => counter!("db_queries_error").with_tag("query", query_name).increment(1),
    }
    
    if duration > Duration::from_millis(10) {
        tracing::warn!(
            duration_ms = duration.as_millis(),
            query = query_name,
            "Slow database query detected"
        );
    }
    
    result
}

// ✅ REQUIRED: Memory usage monitoring
pub async fn monitor_memory_usage() {
    loop {
        let memory_usage = get_memory_usage().await;
        gauge!("memory_usage_bytes", memory_usage as f64);
        
        if memory_usage > 400_000_000 { // 400MB warning threshold
            tracing::warn!(
                memory_usage_mb = memory_usage / 1_000_000,
                "High memory usage detected"
            );
        }
        
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
```

### 2. Performance Alerting
```rust
// ✅ REQUIRED: Performance threshold monitoring
pub struct PerformanceMonitor {
    alert_thresholds: PerformanceThresholds,
    metrics_collector: MetricsCollector,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: u64,
    pub max_database_query_ms: u64,
    pub max_memory_usage_mb: u64,
    pub max_error_rate_percent: f64,
}

impl PerformanceMonitor {
    pub async fn check_performance_thresholds(&self) -> Result<(), PerformanceError> {
        // Check response times
        let avg_response_time = self.metrics_collector.get_avg_response_time().await?;
        if avg_response_time > Duration::from_millis(self.alert_thresholds.max_response_time_ms) {
            self.send_alert(PerformanceAlert::SlowResponses {
                current: avg_response_time,
                threshold: Duration::from_millis(self.alert_thresholds.max_response_time_ms),
            }).await?;
        }
        
        // Check database performance
        let avg_db_time = self.metrics_collector.get_avg_database_time().await?;
        if avg_db_time > Duration::from_millis(self.alert_thresholds.max_database_query_ms) {
            self.send_alert(PerformanceAlert::SlowDatabase {
                current: avg_db_time,
                threshold: Duration::from_millis(self.alert_thresholds.max_database_query_ms),
            }).await?;
        }
        
        // Check error rates
        let error_rate = self.metrics_collector.get_error_rate().await?;
        if error_rate > self.alert_thresholds.max_error_rate_percent {
            self.send_alert(PerformanceAlert::HighErrorRate {
                current: error_rate,
                threshold: self.alert_thresholds.max_error_rate_percent,
            }).await?;
        }
        
        Ok(())
    }
}
```

## 🔧 Performance Testing

### 1. Load Testing Requirements
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use tokio::time::Instant;
    
    #[tokio::test]
    async fn test_api_response_time() {
        let test_env = setup_performance_test_env().await;
        
        // Test 100 concurrent requests
        let tasks: Vec<_> = (0..100)
            .map(|i| {
                let client = test_env.client.clone();
                tokio::spawn(async move {
                    let start = Instant::now();
                    let response = client.get("/api/v1/health").send().await.unwrap();
                    let duration = start.elapsed();
                    
                    assert_eq!(response.status(), 200);
                    assert!(duration < Duration::from_millis(100), 
                        "Response time {}ms exceeds 100ms threshold", duration.as_millis());
                    
                    duration
                })
            })
            .collect();
        
        let durations = futures::future::join_all(tasks).await;
        let avg_duration: Duration = durations.iter()
            .map(|r| r.as_ref().unwrap())
            .sum::<Duration>() / durations.len() as u32;
        
        assert!(avg_duration < Duration::from_millis(50), 
            "Average response time {}ms exceeds 50ms threshold", avg_duration.as_millis());
    }
    
    #[tokio::test]
    async fn test_database_query_performance() {
        let test_env = setup_database_test_env().await;
        
        // Create test data
        test_env.create_test_users(1000).await;
        
        // Test query performance
        let start = Instant::now();
        let users = test_env.get_users_paginated(Uuid::new_v4(), 0, 50).await.unwrap();
        let duration = start.elapsed();
        
        assert_eq!(users.len(), 50);
        assert!(duration < Duration::from_millis(10), 
            "Database query time {}ms exceeds 10ms threshold", duration.as_millis());
    }
    
    #[tokio::test]
    async fn test_memory_usage_under_load() {
        let test_env = setup_memory_test_env().await;
        
        let initial_memory = get_memory_usage().await;
        
        // Simulate high load
        let tasks: Vec<_> = (0..1000)
            .map(|_| {
                tokio::spawn(async {
                    // Simulate work
                    let _ = process_test_request().await;
                })
            })
            .collect();
        
        futures::future::join_all(tasks).await;
        
        let final_memory = get_memory_usage().await;
        let memory_increase = final_memory - initial_memory;
        
        assert!(memory_increase < 100_000_000, // 100MB increase limit
            "Memory usage increased by {}MB, exceeds 100MB threshold", 
            memory_increase / 1_000_000);
    }
}
```

---

**Performance is not negotiable. Every millisecond counts!** ⚡
