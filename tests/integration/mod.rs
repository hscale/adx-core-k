// Integration test module for ADX CORE
// This module provides comprehensive end-to-end testing infrastructure

pub mod circuit_breakers;
pub mod cross_service;
pub mod load_testing;
pub mod micro_frontend;
pub mod multi_tenant;
pub mod user_workflows;
pub mod test_environment;
pub mod performance;

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestEnvironmentConfig {
    pub database_url: String,
    pub redis_url: String,
    pub temporal_url: String,
    pub api_gateway_url: String,
    pub frontend_shell_url: String,
    pub enable_load_testing: bool,
    pub max_concurrent_users: u32,
    pub test_duration_seconds: u64,
}

impl Default for TestEnvironmentConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/adx_core_test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            temporal_url: "http://localhost:7233".to_string(),
            api_gateway_url: "http://localhost:8080".to_string(),
            frontend_shell_url: "http://localhost:3000".to_string(),
            enable_load_testing: false,
            max_concurrent_users: 100,
            test_duration_seconds: 300,
        }
    }
}

/// Test result aggregation
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationTestResults {
    pub test_suite: String,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
    pub execution_time_ms: u64,
    pub test_details: Vec<TestResult>,
    pub performance_metrics: PerformanceMetrics,
    pub errors: Vec<TestError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub assertions: Vec<AssertionResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssertionResult {
    pub description: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_requests_per_second: f64,
    pub error_rate_percentage: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestError {
    pub test_name: String,
    pub error_type: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Global test state manager
pub struct TestStateManager {
    pub config: TestEnvironmentConfig,
    pub results: Arc<RwLock<Vec<IntegrationTestResults>>>,
    pub active_tests: Arc<RwLock<Vec<String>>>,
}

impl TestStateManager {
    pub fn new(config: TestEnvironmentConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(Vec::new())),
            active_tests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_result(&self, result: IntegrationTestResults) {
        let mut results = self.results.write().await;
        results.push(result);
    }

    pub async fn get_summary(&self) -> TestSummary {
        let results = self.results.read().await;
        let total_tests: u32 = results.iter().map(|r| r.total_tests).sum();
        let passed_tests: u32 = results.iter().map(|r| r.passed_tests).sum();
        let failed_tests: u32 = results.iter().map(|r| r.failed_tests).sum();
        let total_execution_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();

        TestSummary {
            total_test_suites: results.len() as u32,
            total_tests,
            passed_tests,
            failed_tests,
            success_rate: if total_tests > 0 { (passed_tests as f64 / total_tests as f64) * 100.0 } else { 0.0 },
            total_execution_time_ms: total_execution_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_test_suites: u32,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub success_rate: f64,
    pub total_execution_time_ms: u64,
}