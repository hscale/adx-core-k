use crate::error::{AIError, AIResult};
use crate::providers::AIProviderManager;
use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct HealthMonitor {
    provider_manager: Arc<AIProviderManager>,
    health_history: Arc<tokio::sync::RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
    check_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub timestamp: DateTime<Utc>,
    pub provider: AIProvider,
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

impl HealthMonitor {
    pub fn new(provider_manager: Arc<AIProviderManager>, check_interval_seconds: u64) -> Self {
        Self {
            provider_manager,
            health_history: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(check_interval_seconds),
        }
    }
    
    pub async fn start_monitoring(&self) {
        let provider_manager = self.provider_manager.clone();
        let health_history = self.health_history.clone();
        let check_interval = self.check_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(check_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::perform_health_checks(
                    &provider_manager,
                    &health_history,
                ).await {
                    tracing::error!("Health check failed: {}", e);
                }
            }
        });
    }
    
    async fn perform_health_checks(
        provider_manager: &AIProviderManager,
        health_history: &Arc<tokio::sync::RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
    ) -> AIResult<()> {
        let health_results = provider_manager.health_check_all().await?;
        let mut history = health_history.write().await;
        
        for (provider_type, health) in health_results {
            let provider_key = format!("{:?}", provider_type);
            let result = HealthCheckResult {
                timestamp: Utc::now(),
                provider: provider_type,
                status: health.status,
                response_time_ms: health.response_time_ms,
                error: health.last_error,
            };
            
            let provider_history = history.entry(provider_key).or_insert_with(Vec::new);
            provider_history.push(result);
            
            // Keep only last 100 health checks per provider
            if provider_history.len() > 100 {
                provider_history.remove(0);
            }
        }
        
        Ok(())
    }
    
    pub async fn get_current_health(&self) -> AIResult<AIServiceHealth> {
        let provider_health = self.provider_manager.health_check_all().await?;
        
        // Determine overall status
        let overall_status = if provider_health.values().all(|h| matches!(h.status, HealthStatus::Healthy)) {
            HealthStatus::Healthy
        } else if provider_health.values().any(|h| matches!(h.status, HealthStatus::Healthy)) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        // Create model health based on provider health
        let mut model_health = HashMap::new();
        
        // This would normally iterate through actual models from a registry
        let sample_models = vec![
            ("gpt-3.5-turbo", AIProvider::OpenAI),
            ("gpt-4", AIProvider::OpenAI),
            ("claude-3-sonnet-20240229", AIProvider::Anthropic),
            ("llama2-7b", AIProvider::Local),
        ];
        
        for (model_id, provider_type) in sample_models {
            if let Some(provider_health_info) = provider_health.get(&provider_type) {
                model_health.insert(model_id.to_string(), ModelHealth {
                    status: provider_health_info.status.clone(),
                    availability: match provider_health_info.status {
                        HealthStatus::Healthy => 1.0,
                        HealthStatus::Degraded => 0.5,
                        HealthStatus::Unhealthy => 0.0,
                    },
                    avg_response_time_ms: provider_health_info.response_time_ms.unwrap_or(0) as f64,
                    error_rate: provider_health_info.error_rate,
                    last_check: provider_health_info.last_check,
                });
            }
        }
        
        Ok(AIServiceHealth {
            status: overall_status,
            providers: provider_health,
            models: model_health,
            last_check: Utc::now(),
        })
    }
    
    pub async fn get_health_history(
        &self,
        provider: Option<AIProvider>,
        hours: u32,
    ) -> AIResult<Vec<HealthCheckResult>> {
        let history = self.health_history.read().await;
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours as i64);
        
        let mut results = Vec::new();
        
        if let Some(provider_type) = provider {
            let provider_key = format!("{:?}", provider_type);
            if let Some(provider_history) = history.get(&provider_key) {
                results.extend(
                    provider_history
                        .iter()
                        .filter(|r| r.timestamp >= cutoff_time)
                        .cloned()
                );
            }
        } else {
            // Return history for all providers
            for provider_history in history.values() {
                results.extend(
                    provider_history
                        .iter()
                        .filter(|r| r.timestamp >= cutoff_time)
                        .cloned()
                );
            }
        }
        
        // Sort by timestamp
        results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        Ok(results)
    }
    
    pub async fn get_availability_metrics(
        &self,
        provider: AIProvider,
        hours: u32,
    ) -> AIResult<AvailabilityMetrics> {
        let history = self.get_health_history(Some(provider), hours).await?;
        
        if history.is_empty() {
            return Ok(AvailabilityMetrics {
                availability_percentage: 0.0,
                total_checks: 0,
                successful_checks: 0,
                failed_checks: 0,
                avg_response_time_ms: 0.0,
                max_response_time_ms: 0,
                min_response_time_ms: 0,
                uptime_periods: Vec::new(),
                downtime_periods: Vec::new(),
            });
        }
        
        let total_checks = history.len();
        let successful_checks = history
            .iter()
            .filter(|r| matches!(r.status, HealthStatus::Healthy))
            .count();
        let failed_checks = total_checks - successful_checks;
        
        let availability_percentage = (successful_checks as f64 / total_checks as f64) * 100.0;
        
        let response_times: Vec<u64> = history
            .iter()
            .filter_map(|r| r.response_time_ms)
            .collect();
        
        let avg_response_time_ms = if response_times.is_empty() {
            0.0
        } else {
            response_times.iter().sum::<u64>() as f64 / response_times.len() as f64
        };
        
        let max_response_time_ms = response_times.iter().max().copied().unwrap_or(0);
        let min_response_time_ms = response_times.iter().min().copied().unwrap_or(0);
        
        // Calculate uptime and downtime periods
        let (uptime_periods, downtime_periods) = self.calculate_uptime_downtime_periods(&history);
        
        Ok(AvailabilityMetrics {
            availability_percentage,
            total_checks,
            successful_checks,
            failed_checks,
            avg_response_time_ms,
            max_response_time_ms,
            min_response_time_ms,
            uptime_periods,
            downtime_periods,
        })
    }
    
    fn calculate_uptime_downtime_periods(
        &self,
        history: &[HealthCheckResult],
    ) -> (Vec<TimePeriod>, Vec<TimePeriod>) {
        let mut uptime_periods = Vec::new();
        let mut downtime_periods = Vec::new();
        
        if history.is_empty() {
            return (uptime_periods, downtime_periods);
        }
        
        let mut current_period_start = history[0].timestamp;
        let mut current_status = &history[0].status;
        
        for window in history.windows(2) {
            let current = &window[0];
            let next = &window[1];
            
            if std::mem::discriminant(&current.status) != std::mem::discriminant(&next.status) {
                // Status changed, close current period
                let period = TimePeriod {
                    start: current_period_start,
                    end: current.timestamp,
                    duration_seconds: (current.timestamp - current_period_start).num_seconds() as u64,
                };
                
                match current_status {
                    HealthStatus::Healthy => uptime_periods.push(period),
                    _ => downtime_periods.push(period),
                }
                
                current_period_start = next.timestamp;
                current_status = &next.status;
            }
        }
        
        // Close the last period
        if let Some(last) = history.last() {
            let period = TimePeriod {
                start: current_period_start,
                end: last.timestamp,
                duration_seconds: (last.timestamp - current_period_start).num_seconds() as u64,
            };
            
            match current_status {
                HealthStatus::Healthy => uptime_periods.push(period),
                _ => downtime_periods.push(period),
            }
        }
        
        (uptime_periods, downtime_periods)
    }
    
    pub async fn get_alert_conditions(&self) -> AIResult<Vec<AlertCondition>> {
        let current_health = self.get_current_health().await?;
        let mut alerts = Vec::new();
        
        // Check for unhealthy providers
        for (provider, health) in &current_health.providers {
            if matches!(health.status, HealthStatus::Unhealthy) {
                alerts.push(AlertCondition {
                    severity: AlertSeverity::Critical,
                    message: format!("Provider {:?} is unhealthy", provider),
                    provider: Some(provider.clone()),
                    model: None,
                    timestamp: Utc::now(),
                    details: health.last_error.clone(),
                });
            } else if matches!(health.status, HealthStatus::Degraded) {
                alerts.push(AlertCondition {
                    severity: AlertSeverity::Warning,
                    message: format!("Provider {:?} is degraded", provider),
                    provider: Some(provider.clone()),
                    model: None,
                    timestamp: Utc::now(),
                    details: health.last_error.clone(),
                });
            }
        }
        
        // Check for high response times
        for (provider, health) in &current_health.providers {
            if let Some(response_time) = health.response_time_ms {
                if response_time > 5000 { // 5 seconds
                    alerts.push(AlertCondition {
                        severity: AlertSeverity::Warning,
                        message: format!("Provider {:?} has high response time: {}ms", provider, response_time),
                        provider: Some(provider.clone()),
                        model: None,
                        timestamp: Utc::now(),
                        details: Some(format!("Response time: {}ms", response_time)),
                    });
                }
            }
        }
        
        Ok(alerts)
    }
}

#[derive(Debug, Clone)]
pub struct AvailabilityMetrics {
    pub availability_percentage: f64,
    pub total_checks: usize,
    pub successful_checks: usize,
    pub failed_checks: usize,
    pub avg_response_time_ms: f64,
    pub max_response_time_ms: u64,
    pub min_response_time_ms: u64,
    pub uptime_periods: Vec<TimePeriod>,
    pub downtime_periods: Vec<TimePeriod>,
}

#[derive(Debug, Clone)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct AlertCondition {
    pub severity: AlertSeverity,
    pub message: String,
    pub provider: Option<AIProvider>,
    pub model: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}