use prometheus::{Counter, Gauge, Histogram, Opts, Registry};
use std::collections::HashMap;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

pub struct ObservabilityManager {
    metrics_registry: Arc<Registry>,
    request_counter: Counter,
    request_duration: Histogram,
    active_connections: Gauge,
    error_counter: Counter,
}

impl ObservabilityManager {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let request_counter = Counter::with_opts(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .const_labels(HashMap::new()),
        )
        .unwrap();

        let request_duration = Histogram::with_opts(prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
        ))
        .unwrap();

        let active_connections = Gauge::with_opts(Opts::new(
            "active_connections",
            "Number of active connections",
        ))
        .unwrap();

        let error_counter =
            Counter::with_opts(Opts::new("errors_total", "Total number of errors")).unwrap();

        registry
            .register(Box::new(request_counter.clone()))
            .unwrap();
        registry
            .register(Box::new(request_duration.clone()))
            .unwrap();
        registry
            .register(Box::new(active_connections.clone()))
            .unwrap();
        registry.register(Box::new(error_counter.clone())).unwrap();

        Self {
            metrics_registry: registry,
            request_counter,
            request_duration,
            active_connections,
            error_counter,
        }
    }

    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        self.request_counter.inc();
        self.request_duration.observe(duration);

        if status >= 400 {
            self.error_counter.inc();
        }

        tracing::info!(
            method = method,
            path = path,
            status = status,
            duration = duration,
            "HTTP request completed"
        );
    }

    pub fn set_active_connections(&self, count: i64) {
        self.active_connections.set(count as f64);
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.metrics_registry.clone()
    }
}

impl Default for ObservabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;
    fn name(&self) -> &str;
}

pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    pub async fn check_all(&self) -> HashMap<String, HealthStatus> {
        let mut results = HashMap::new();

        for check in &self.checks {
            let status = check.check().await;
            results.insert(check.name().to_string(), status);
        }

        results
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
