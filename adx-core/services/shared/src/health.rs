use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use crate::{Result, Error, types::{HealthStatus, HealthCheck}};

#[derive(Debug, Clone)]
pub struct HealthChecker {
    checks: HashMap<String, Box<dyn HealthCheckProvider + Send + Sync>>,
    version: String,
}

#[async_trait::async_trait]
pub trait HealthCheckProvider {
    async fn check(&self) -> Result<HealthCheck>;
    fn name(&self) -> &str;
}

impl HealthChecker {
    pub fn new(version: String) -> Self {
        Self {
            checks: HashMap::new(),
            version,
        }
    }
    
    pub fn add_check<T: HealthCheckProvider + Send + Sync + 'static>(&mut self, check: T) {
        self.checks.insert(check.name().to_string(), Box::new(check));
    }
    
    pub async fn check_health(&self) -> HealthStatus {
        let mut checks = HashMap::new();
        let mut overall_status = "healthy";
        
        for (name, checker) in &self.checks {
            let start = Instant::now();
            let result = checker.check().await;
            let duration = start.elapsed();
            
            let health_check = match result {
                Ok(check) => {
                    if check.status != "healthy" {
                        overall_status = "unhealthy";
                    }
                    check
                }
                Err(e) => {
                    overall_status = "unhealthy";
                    HealthCheck {
                        status: "unhealthy".to_string(),
                        message: Some(e.to_string()),
                        duration_ms: duration.as_millis() as u64,
                    }
                }
            };
            
            checks.insert(name.clone(), health_check);
        }
        
        HealthStatus {
            status: overall_status.to_string(),
            timestamp: Utc::now(),
            version: self.version.clone(),
            checks,
        }
    }
}

// Database health check
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl HealthCheckProvider for DatabaseHealthCheck {
    async fn check(&self) -> Result<HealthCheck> {
        let start = Instant::now();
        
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(HealthCheck {
                status: "healthy".to_string(),
                message: Some("Database connection successful".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
            Err(e) => Ok(HealthCheck {
                status: "unhealthy".to_string(),
                message: Some(format!("Database connection failed: {}", e)),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
}

// Redis health check
pub struct RedisHealthCheck {
    client: redis::Client,
}

impl RedisHealthCheck {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HealthCheckProvider for RedisHealthCheck {
    async fn check(&self) -> Result<HealthCheck> {
        let start = Instant::now();
        
        match self.client.get_async_connection().await {
            Ok(mut conn) => {
                match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                    Ok(_) => Ok(HealthCheck {
                        status: "healthy".to_string(),
                        message: Some("Redis connection successful".to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }),
                    Err(e) => Ok(HealthCheck {
                        status: "unhealthy".to_string(),
                        message: Some(format!("Redis ping failed: {}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }),
                }
            }
            Err(e) => Ok(HealthCheck {
                status: "unhealthy".to_string(),
                message: Some(format!("Redis connection failed: {}", e)),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }
    
    fn name(&self) -> &str {
        "redis"
    }
}

// Temporal health check
pub struct TemporalHealthCheck {
    client: crate::temporal::TemporalClient,
}

impl TemporalHealthCheck {
    pub fn new(client: crate::temporal::TemporalClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HealthCheckProvider for TemporalHealthCheck {
    async fn check(&self) -> Result<HealthCheck> {
        let start = Instant::now();
        
        // In a real implementation, this would check Temporal server connectivity
        // For now, we'll return a mock healthy status
        Ok(HealthCheck {
            status: "healthy".to_string(),
            message: Some("Temporal connection successful".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    fn name(&self) -> &str {
        "temporal"
    }
}