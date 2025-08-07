# ADX CORE - Performance and Deployment Architecture

## Overview

ADX CORE is designed for high performance, scalability, and reliable deployment across various environments. The architecture leverages Rust's performance advantages, modern deployment practices, and comprehensive monitoring to deliver enterprise-grade reliability.

## Performance Architecture

### Performance Targets
- **API Response Time**: < 200ms for 95th percentile
- **System Availability**: 99.9% uptime SLA
- **Concurrent Users**: Support for 100K+ concurrent users
- **Database Query Time**: < 100ms for 95th percentile
- **Workflow Execution**: < 1 second for simple workflows
- **File Upload Speed**: > 10MB/s for large files
- **Memory Usage**: < 512MB per service instance
- **CPU Utilization**: < 70% under normal load

### Rust Performance Advantages
```rust
// High-performance async runtime with Tokio
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure runtime for maximum performance
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name("adx-worker")
        .thread_stack_size(3 * 1024 * 1024) // 3MB stack
        .enable_all()
        .build()?;
    
    rt.block_on(async {
        // Start all services
        let services = start_services().await?;
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        
        // Graceful shutdown
        shutdown_services(services).await?;
        
        Ok(())
    })
}

// Zero-copy serialization with serde
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

// Memory-efficient connection pooling
pub struct DatabasePool {
    pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
    metrics: Arc<PoolMetrics>,
}

impl DatabasePool {
    pub fn new(database_url: &str, max_connections: u32) -> Result<Self, Error> {
        let manager = PostgresConnectionManager::new_from_stringlike(
            database_url,
            NoTls,
        )?;
        
        let pool = Pool::builder()
            .max_size(max_connections)
            .min_idle(Some(5))
            .max_lifetime(Some(Duration::from_secs(1800))) // 30 minutes
            .idle_timeout(Some(Duration::from_secs(600)))  // 10 minutes
            .connection_timeout(Duration::from_secs(30))
            .build(manager)?;
        
        Ok(Self {
            pool: Arc::new(pool),
            metrics: Arc::new(PoolMetrics::new()),
        })
    }
    
    pub async fn get_connection(&self) -> Result<PooledConnection<PostgresConnectionManager<NoTls>>, Error> {
        let start = Instant::now();
        let conn = self.pool.get().await?;
        self.metrics.record_connection_time(start.elapsed());
        Ok(conn)
    }
}
```

### Caching Strategy
```rust
// Multi-layer caching system
pub struct CacheManager {
    l1_cache: Arc<RwLock<LruCache<String, CacheEntry>>>, // In-memory
    l2_cache: Arc<RedisPool>,                            // Redis
    l3_cache: Arc<DatabasePool>,                         // Database
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Bytes,
    pub expires_at: DateTime<Utc>,
    pub etag: String,
    pub content_type: String,
}

impl CacheManager {
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: DeserializeOwned,
    {
        // L1 Cache (In-memory)
        if let Some(entry) = self.l1_cache.read().await.get(key) {
            if entry.expires_at > Utc::now() {
                let data: T = bincode::deserialize(&entry.data)?;
                return Ok(Some(data));
            }
        }
        
        // L2 Cache (Redis)
        let mut redis = self.l2_cache.get().await?;
        if let Some(data) = redis.get::<_, Vec<u8>>(key).await? {
            let entry: CacheEntry = bincode::deserialize(&data)?;
            if entry.expires_at > Utc::now() {
                // Update L1 cache
                self.l1_cache.write().await.put(key.to_string(), entry.clone());
                
                let data: T = bincode::deserialize(&entry.data)?;
                return Ok(Some(data));
            }
        }
        
        Ok(None)
    }
    
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let data = bincode::serialize(value)?;
        let expires_at = Utc::now() + chrono::Duration::from_std(ttl)?;
        let etag = format!("{:x}", md5::compute(&data));
        
        let entry = CacheEntry {
            data: Bytes::from(data),
            expires_at,
            etag,
            content_type: "application/octet-stream".to_string(),
        };
        
        // Update all cache layers
        self.l1_cache.write().await.put(key.to_string(), entry.clone());
        
        let mut redis = self.l2_cache.get().await?;
        let serialized_entry = bincode::serialize(&entry)?;
        redis.setex(key, ttl.as_secs() as usize, serialized_entry).await?;
        
        Ok(())
    }
    
    pub async fn invalidate(&self, pattern: &str) -> Result<(), CacheError> {
        // Invalidate L1 cache
        let mut l1 = self.l1_cache.write().await;
        let keys_to_remove: Vec<String> = l1.iter()
            .filter(|(k, _)| k.contains(pattern))
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in keys_to_remove {
            l1.pop(&key);
        }
        
        // Invalidate L2 cache
        let mut redis = self.l2_cache.get().await?;
        let keys: Vec<String> = redis.keys(format!("*{}*", pattern)).await?;
        if !keys.is_empty() {
            redis.del(keys).await?;
        }
        
        Ok(())
    }
}
```

### Load Balancing and Auto-Scaling
```rust
// Service discovery and load balancing
pub struct ServiceDiscovery {
    consul_client: Arc<ConsulClient>,
    health_checker: Arc<HealthChecker>,
    load_balancer: Arc<LoadBalancer>,
}

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub health_status: HealthStatus,
    pub metadata: HashMap<String, String>,
    pub last_heartbeat: DateTime<Utc>,
}

impl ServiceDiscovery {
    pub async fn register_service(&self, service: &ServiceInstance) -> Result<(), ServiceError> {
        // Register with Consul
        self.consul_client.register_service(service).await?;
        
        // Start health checking
        self.health_checker.start_monitoring(service.id.clone()).await?;
        
        Ok(())
    }
    
    pub async fn discover_services(&self, service_name: &str) -> Result<Vec<ServiceInstance>, ServiceError> {
        let services = self.consul_client.get_healthy_services(service_name).await?;
        
        // Filter by health status
        let healthy_services: Vec<ServiceInstance> = services
            .into_iter()
            .filter(|s| s.health_status == HealthStatus::Healthy)
            .collect();
        
        Ok(healthy_services)
    }
    
    pub async fn get_service_endpoint(&self, service_name: &str) -> Result<String, ServiceError> {
        let services = self.discover_services(service_name).await?;
        
        if services.is_empty() {
            return Err(ServiceError::NoHealthyInstances(service_name.to_string()));
        }
        
        // Use load balancer to select instance
        let selected = self.load_balancer.select_instance(&services).await?;
        
        Ok(format!("http://{}:{}", selected.address, selected.port))
    }
}

// Auto-scaling based on metrics
pub struct AutoScaler {
    metrics_collector: Arc<MetricsCollector>,
    kubernetes_client: Arc<KubernetesClient>,
    scaling_policies: Arc<RwLock<HashMap<String, ScalingPolicy>>>,
}

#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    pub service_name: String,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
}

impl AutoScaler {
    pub async fn evaluate_scaling(&self) -> Result<(), ScalingError> {
        let policies = self.scaling_policies.read().await;
        
        for policy in policies.values() {
            let metrics = self.metrics_collector.get_service_metrics(&policy.service_name).await?;
            let current_replicas = self.kubernetes_client.get_replica_count(&policy.service_name).await?;
            
            let cpu_utilization = metrics.cpu_utilization;
            let memory_utilization = metrics.memory_utilization;
            
            let desired_replicas = self.calculate_desired_replicas(
                current_replicas,
                cpu_utilization,
                memory_utilization,
                policy,
            );
            
            if desired_replicas != current_replicas {
                self.scale_service(&policy.service_name, desired_replicas).await?;
            }
        }
        
        Ok(())
    }
    
    fn calculate_desired_replicas(
        &self,
        current: u32,
        cpu_util: f64,
        memory_util: f64,
        policy: &ScalingPolicy,
    ) -> u32 {
        let cpu_ratio = cpu_util / policy.target_cpu_utilization;
        let memory_ratio = memory_util / policy.target_memory_utilization;
        
        // Use the higher ratio for scaling decisions
        let scaling_ratio = cpu_ratio.max(memory_ratio);
        
        let desired = (current as f64 * scaling_ratio).ceil() as u32;
        
        // Apply min/max constraints
        desired.max(policy.min_replicas).min(policy.max_replicas)
    }
}
```

## Deployment Architecture

### Container Strategy
```dockerfile
# Multi-stage Rust build for minimal production images
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build optimized release binary
RUN cargo build --release --bin adx-core

# Production image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /app adx

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/adx-core ./
COPY --chown=adx:adx config ./config

# Set ownership and permissions
RUN chown -R adx:adx /app && chmod +x adx-core

USER adx

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./adx-core"]
```

### Kubernetes Deployment
```yaml
# Kubernetes deployment configuration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: adx-core-api
  labels:
    app: adx-core
    component: api
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: adx-core
      component: api
  template:
    metadata:
      labels:
        app: adx-core
        component: api
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: adx-core
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: api
        image: adxcore/api:1.0.0
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: adx-core-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: adx-core-secrets
              key: redis-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: adx-core-secrets
              key: jwt-secret
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: temp
          mountPath: /tmp
      volumes:
      - name: config
        configMap:
          name: adx-core-config
      - name: temp
        emptyDir: {}
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - adx-core
                - key: component
                  operator: In
                  values:
                  - api
              topologyKey: kubernetes.io/hostname

---
apiVersion: v1
kind: Service
metadata:
  name: adx-core-api
  labels:
    app: adx-core
    component: api
spec:
  selector:
    app: adx-core
    component: api
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: adx-core-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: adx-core-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

### GitOps Deployment Pipeline
```yaml
# ArgoCD Application configuration
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: adx-core
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: default
  source:
    repoURL: https://github.com/adxcore/infrastructure
    targetRevision: HEAD
    path: kubernetes/overlays/production
  destination:
    server: https://kubernetes.default.svc
    namespace: adx-core
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    syncOptions:
    - CreateNamespace=true
    - PrunePropagationPolicy=foreground
    - PruneLast=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
  revisionHistoryLimit: 10

---
# Blue-Green deployment strategy
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: adx-core-api
spec:
  replicas: 5
  strategy:
    blueGreen:
      activeService: adx-core-api-active
      previewService: adx-core-api-preview
      autoPromotionEnabled: false
      scaleDownDelaySeconds: 30
      prePromotionAnalysis:
        templates:
        - templateName: success-rate
        args:
        - name: service-name
          value: adx-core-api-preview
      postPromotionAnalysis:
        templates:
        - templateName: success-rate
        args:
        - name: service-name
          value: adx-core-api-active
  selector:
    matchLabels:
      app: adx-core
      component: api
  template:
    metadata:
      labels:
        app: adx-core
        component: api
    spec:
      containers:
      - name: api
        image: adxcore/api:1.0.0
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

### Database Migration Strategy
```rust
// Database migration system
pub struct MigrationManager {
    database_pool: Arc<DatabasePool>,
    migration_lock: Arc<Mutex<()>>,
    migration_history: Arc<MigrationHistory>,
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
    pub checksum: String,
    pub applied_at: Option<DateTime<Utc>>,
}

impl MigrationManager {
    pub async fn run_migrations(&self) -> Result<(), MigrationError> {
        let _lock = self.migration_lock.lock().await;
        
        // Get pending migrations
        let pending_migrations = self.get_pending_migrations().await?;
        
        if pending_migrations.is_empty() {
            info!("No pending migrations");
            return Ok(());
        }
        
        info!("Running {} pending migrations", pending_migrations.len());
        
        for migration in pending_migrations {
            self.apply_migration(&migration).await?;
        }
        
        Ok(())
    }
    
    async fn apply_migration(&self, migration: &Migration) -> Result<(), MigrationError> {
        let mut conn = self.database_pool.get_connection().await?;
        let tx = conn.transaction().await?;
        
        // Apply migration
        tx.batch_execute(&migration.up_sql).await?;
        
        // Record migration
        tx.execute(
            "INSERT INTO schema_migrations (version, name, checksum, applied_at) VALUES ($1, $2, $3, $4)",
            &[&migration.version, &migration.name, &migration.checksum, &Utc::now()],
        ).await?;
        
        tx.commit().await?;
        
        info!("Applied migration: {} - {}", migration.version, migration.name);
        
        Ok(())
    }
    
    pub async fn rollback_migration(&self, target_version: &str) -> Result<(), MigrationError> {
        let _lock = self.migration_lock.lock().await;
        
        let applied_migrations = self.get_applied_migrations().await?;
        let target_index = applied_migrations
            .iter()
            .position(|m| m.version == target_version)
            .ok_or(MigrationError::VersionNotFound(target_version.to_string()))?;
        
        // Rollback migrations in reverse order
        for migration in applied_migrations[target_index..].iter().rev() {
            self.rollback_single_migration(migration).await?;
        }
        
        Ok(())
    }
    
    async fn rollback_single_migration(&self, migration: &Migration) -> Result<(), MigrationError> {
        let mut conn = self.database_pool.get_connection().await?;
        let tx = conn.transaction().await?;
        
        // Apply rollback
        tx.batch_execute(&migration.down_sql).await?;
        
        // Remove migration record
        tx.execute(
            "DELETE FROM schema_migrations WHERE version = $1",
            &[&migration.version],
        ).await?;
        
        tx.commit().await?;
        
        info!("Rolled back migration: {} - {}", migration.version, migration.name);
        
        Ok(())
    }
}
```

## Monitoring and Observability

### Metrics Collection
```rust
// Comprehensive metrics collection
pub struct MetricsCollector {
    registry: Arc<Registry>,
    http_requests: Arc<CounterVec>,
    http_duration: Arc<HistogramVec>,
    database_connections: Arc<Gauge>,
    active_users: Arc<Gauge>,
    workflow_executions: Arc<CounterVec>,
    memory_usage: Arc<Gauge>,
    cpu_usage: Arc<Gauge>,
}

impl MetricsCollector {
    pub fn new() -> Result<Self, MetricsError> {
        let registry = Registry::new();
        
        let http_requests = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "path", "status"],
        )?;
        
        let http_duration = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            &["method", "path"],
        )?;
        
        let database_connections = Gauge::new(
            "database_connections_active",
            "Number of active database connections",
        )?;
        
        let active_users = Gauge::new(
            "active_users_total",
            "Number of active users",
        )?;
        
        let workflow_executions = CounterVec::new(
            Opts::new("workflow_executions_total", "Total number of workflow executions"),
            &["workflow_id", "status", "ai_enhanced"],
        )?;
        
        let memory_usage = Gauge::new(
            "memory_usage_bytes",
            "Memory usage in bytes",
        )?;
        
        let cpu_usage = Gauge::new(
            "cpu_usage_percent",
            "CPU usage percentage",
        )?;
        
        // Register metrics
        registry.register(Box::new(http_requests.clone()))?;
        registry.register(Box::new(http_duration.clone()))?;
        registry.register(Box::new(database_connections.clone()))?;
        registry.register(Box::new(active_users.clone()))?;
        registry.register(Box::new(workflow_executions.clone()))?;
        registry.register(Box::new(memory_usage.clone()))?;
        registry.register(Box::new(cpu_usage.clone()))?;
        
        Ok(Self {
            registry: Arc::new(registry),
            http_requests: Arc::new(http_requests),
            http_duration: Arc::new(http_duration),
            database_connections: Arc::new(database_connections),
            active_users: Arc::new(active_users),
            workflow_executions: Arc::new(workflow_executions),
            memory_usage: Arc::new(memory_usage),
            cpu_usage: Arc::new(cpu_usage),
        })
    }
    
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        self.http_requests
            .with_label_values(&[method, path, &status.to_string()])
            .inc();
        
        self.http_duration
            .with_label_values(&[method, path])
            .observe(duration.as_secs_f64());
    }
    
    pub fn record_workflow_execution(&self, workflow_id: &str, status: &str, ai_enhanced: bool) {
        self.workflow_executions
            .with_label_values(&[workflow_id, status, &ai_enhanced.to_string()])
            .inc();
    }
    
    pub fn update_system_metrics(&self, memory_bytes: u64, cpu_percent: f64) {
        self.memory_usage.set(memory_bytes as f64);
        self.cpu_usage.set(cpu_percent);
    }
    
    pub fn get_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families).unwrap_or_default()
    }
}
```

### Distributed Tracing
```rust
// OpenTelemetry distributed tracing
pub struct TracingService {
    tracer: Arc<BoxedTracer>,
    span_processor: Arc<BatchSpanProcessor>,
}

impl TracingService {
    pub fn new() -> Result<Self, TracingError> {
        // Configure OTLP exporter
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("http://jaeger:14250");
        
        // Configure tracer
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::TraceIdRatioBased(1.0))
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", "adx-core"),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])),
            )
            .install_batch(opentelemetry::runtime::Tokio)?;
        
        Ok(Self {
            tracer: Arc::new(tracer),
            span_processor: Arc::new(BatchSpanProcessor::builder(
                opentelemetry_stdout::SpanExporter::default(),
                opentelemetry::runtime::Tokio,
            ).build()),
        })
    }
    
    pub fn start_span(&self, name: &str) -> Span {
        self.tracer.start(name)
    }
    
    pub fn start_span_with_context(&self, name: &str, parent_context: &Context) -> Span {
        self.tracer.start_with_context(name, parent_context)
    }
    
    pub async fn trace_async_operation<F, T>(&self, name: &str, operation: F) -> T
    where
        F: Future<Output = T>,
    {
        let span = self.start_span(name);
        let _guard = span.enter();
        
        let result = operation.await;
        
        span.end();
        result
    }
}

// Tracing middleware for HTTP requests
pub async fn tracing_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, Infallible> {
    let tracer = get_tracer();
    let span = tracer.start(format!("{} {}", req.method(), req.uri().path()));
    
    // Add request attributes
    span.set_attribute(KeyValue::new("http.method", req.method().to_string()));
    span.set_attribute(KeyValue::new("http.url", req.uri().to_string()));
    span.set_attribute(KeyValue::new("http.scheme", req.uri().scheme_str().unwrap_or("http")));
    
    if let Some(user_agent) = req.headers().get("user-agent") {
        span.set_attribute(KeyValue::new("http.user_agent", user_agent.to_str().unwrap_or("")));
    }
    
    let _guard = span.enter();
    
    let response = next.run(req).await;
    
    // Add response attributes
    span.set_attribute(KeyValue::new("http.status_code", response.status().as_u16() as i64));
    
    if response.status().is_server_error() {
        span.set_status(Status::error("Server error"));
    } else if response.status().is_client_error() {
        span.set_status(Status::error("Client error"));
    }
    
    span.end();
    
    Ok(response)
}
```

### Health Checks and Monitoring
```rust
// Comprehensive health checking system
pub struct HealthChecker {
    database_pool: Arc<DatabasePool>,
    redis_pool: Arc<RedisPool>,
    temporal_client: Arc<TemporalClient>,
    file_storage: Arc<dyn FileStorage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ServiceStatus,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime: Duration,
    pub services: HashMap<String, ServiceHealth>,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: ServiceStatus,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub last_check: DateTime<Utc>,
}

impl HealthChecker {
    pub async fn check_health(&self) -> HealthStatus {
        let start_time = Instant::now();
        let mut services = HashMap::new();
        
        // Check database
        let db_health = self.check_database_health().await;
        services.insert("database".to_string(), db_health);
        
        // Check Redis
        let redis_health = self.check_redis_health().await;
        services.insert("redis".to_string(), redis_health);
        
        // Check Temporal
        let temporal_health = self.check_temporal_health().await;
        services.insert("temporal".to_string(), temporal_health);
        
        // Check file storage
        let storage_health = self.check_storage_health().await;
        services.insert("file_storage".to_string(), storage_health);
        
        // Determine overall status
        let overall_status = self.determine_overall_status(&services);
        
        // Get system metrics
        let metrics = self.get_system_metrics().await;
        
        HealthStatus {
            status: overall_status,
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: start_time.elapsed(),
            services,
            metrics,
        }
    }
    
    async fn check_database_health(&self) -> ServiceHealth {
        let start = Instant::now();
        
        match self.database_pool.get_connection().await {
            Ok(mut conn) => {
                match conn.execute("SELECT 1", &[]).await {
                    Ok(_) => ServiceHealth {
                        status: ServiceStatus::Healthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        error_message: None,
                        last_check: Utc::now(),
                    },
                    Err(e) => ServiceHealth {
                        status: ServiceStatus::Unhealthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        error_message: Some(e.to_string()),
                        last_check: Utc::now(),
                    },
                }
            }
            Err(e) => ServiceHealth {
                status: ServiceStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                last_check: Utc::now(),
            },
        }
    }
    
    async fn check_redis_health(&self) -> ServiceHealth {
        let start = Instant::now();
        
        match self.redis_pool.get().await {
            Ok(mut conn) => {
                match conn.ping().await {
                    Ok(_) => ServiceHealth {
                        status: ServiceStatus::Healthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        error_message: None,
                        last_check: Utc::now(),
                    },
                    Err(e) => ServiceHealth {
                        status: ServiceStatus::Unhealthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        error_message: Some(e.to_string()),
                        last_check: Utc::now(),
                    },
                }
            }
            Err(e) => ServiceHealth {
                status: ServiceStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                last_check: Utc::now(),
            },
        }
    }
    
    fn determine_overall_status(&self, services: &HashMap<String, ServiceHealth>) -> ServiceStatus {
        let unhealthy_count = services.values()
            .filter(|s| matches!(s.status, ServiceStatus::Unhealthy))
            .count();
        
        let degraded_count = services.values()
            .filter(|s| matches!(s.status, ServiceStatus::Degraded))
            .count();
        
        if unhealthy_count > 0 {
            ServiceStatus::Unhealthy
        } else if degraded_count > 0 {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        }
    }
}
```

## Disaster Recovery and Backup

### Backup Strategy
```rust
// Automated backup system
pub struct BackupManager {
    database_pool: Arc<DatabasePool>,
    file_storage: Arc<dyn FileStorage>,
    backup_storage: Arc<dyn BackupStorage>,
    encryption_service: Arc<EncryptionService>,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub schedule: BackupSchedule,
    pub retention_policy: RetentionPolicy,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub cross_region_replication: bool,
}

impl BackupManager {
    pub async fn create_full_backup(&self, tenant_id: Option<TenantId>) -> Result<BackupResult, BackupError> {
        let backup_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        info!("Starting full backup: {}", backup_id);
        
        // Create database backup
        let db_backup = self.backup_database(tenant_id).await?;
        
        // Create file backup
        let file_backup = self.backup_files(tenant_id).await?;
        
        // Combine backups
        let combined_backup = self.combine_backups(db_backup, file_backup).await?;
        
        // Encrypt if enabled
        let final_backup = if self.config.encryption_enabled {
            self.encryption_service.encrypt_backup(&combined_backup).await?
        } else {
            combined_backup
        };
        
        // Store backup
        let backup_location = self.backup_storage.store_backup(&final_backup).await?;
        
        // Record backup metadata
        let backup_result = BackupResult {
            backup_id,
            tenant_id,
            backup_type: BackupType::Full,
            size_bytes: final_backup.len() as u64,
            location: backup_location,
            created_at: start_time,
            completed_at: Utc::now(),
            checksum: self.calculate_checksum(&final_backup),
        };
        
        self.record_backup(&backup_result).await?;
        
        info!("Completed full backup: {} in {:?}", backup_id, backup_result.completed_at - start_time);
        
        Ok(backup_result)
    }
    
    pub async fn restore_backup(&self, backup_id: Uuid, target_tenant_id: Option<TenantId>) -> Result<(), BackupError> {
        info!("Starting backup restore: {}", backup_id);
        
        // Get backup metadata
        let backup_info = self.get_backup_info(backup_id).await?;
        
        // Download backup
        let backup_data = self.backup_storage.retrieve_backup(&backup_info.location).await?;
        
        // Decrypt if necessary
        let decrypted_data = if backup_info.encrypted {
            self.encryption_service.decrypt_backup(&backup_data).await?
        } else {
            backup_data
        };
        
        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&decrypted_data);
        if calculated_checksum != backup_info.checksum {
            return Err(BackupError::ChecksumMismatch);
        }
        
        // Extract backup components
        let (db_backup, file_backup) = self.extract_backup_components(&decrypted_data).await?;
        
        // Restore database
        self.restore_database(&db_backup, target_tenant_id).await?;
        
        // Restore files
        self.restore_files(&file_backup, target_tenant_id).await?;
        
        info!("Completed backup restore: {}", backup_id);
        
        Ok(())
    }
    
    pub async fn cleanup_old_backups(&self) -> Result<(), BackupError> {
        let retention_policy = &self.config.retention_policy;
        let cutoff_date = Utc::now() - retention_policy.max_age;
        
        let old_backups = self.get_backups_before(cutoff_date).await?;
        
        for backup in old_backups {
            // Delete backup file
            self.backup_storage.delete_backup(&backup.location).await?;
            
            // Remove metadata
            self.delete_backup_record(backup.backup_id).await?;
            
            info!("Deleted old backup: {}", backup.backup_id);
        }
        
        Ok(())
    }
}
```

This comprehensive performance and deployment architecture provides:

1. **High-performance Rust implementation** with async/await and zero-copy optimizations
2. **Multi-layer caching strategy** for optimal response times
3. **Auto-scaling capabilities** based on real-time metrics
4. **Container-first deployment** with Kubernetes orchestration
5. **GitOps deployment pipeline** with blue-green strategies
6. **Database migration system** with rollback capabilities
7. **Comprehensive monitoring** with metrics, tracing, and health checks
8. **Disaster recovery** with automated backups and restoration
9. **Load balancing** with service discovery
10. **Production-ready configuration** for enterprise deployment