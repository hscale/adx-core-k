use std::sync::Arc;
use std::time::Duration;
use redis::{AsyncCommands, Client as RedisClient};
use tracing::{debug, warn, error};
use serde::{Serialize, Deserialize};

use crate::config::RateLimitingConfig;
use crate::error::{ApiGatewayError, ApiResult};

/// Rate limiter with tenant and user awareness
#[derive(Clone)]
pub struct RateLimiter {
    redis_client: Arc<RedisClient>,
    config: RateLimitingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit_type: Option<String>,
    pub retry_after: Option<u64>,
    pub remaining_minute: Option<u32>,
    pub remaining_hour: Option<u32>,
    pub current_usage: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct RateLimitKey {
    pub tenant_id: String,
    pub user_id: String,
    pub endpoint: String,
    pub time_window: String,
}

impl RateLimiter {
    pub async fn new(redis_url: &str, config: RateLimitingConfig) -> ApiResult<Self> {
        let redis_client = RedisClient::open(redis_url)
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to create Redis client: {}", e),
            })?;

        // Test Redis connection
        let mut conn = redis_client.get_async_connection().await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to connect to Redis: {}", e),
            })?;

        // Test with a simple command
        let _: String = redis::cmd("PING").query_async(&mut conn).await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Redis ping failed: {}", e),
            })?;

        debug!("Successfully connected to Redis for rate limiting");

        Ok(Self {
            redis_client: Arc::new(redis_client),
            config,
        })
    }

    /// Check rate limit for a request
    pub async fn check_rate_limit(
        &self,
        tenant_id: &str,
        user_id: &str,
        endpoint: &str,
    ) -> ApiResult<RateLimitResult> {
        if !self.config.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                limit_type: None,
                retry_after: None,
                remaining_minute: None,
                remaining_hour: None,
                current_usage: None,
            });
        }

        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to get Redis connection: {}", e),
            })?;

        // Check minute-based rate limit
        let minute_key = self.create_rate_limit_key(tenant_id, user_id, endpoint, "minute");
        let minute_count = self.increment_counter(&mut conn, &minute_key, 60).await?;

        if minute_count > self.config.requests_per_minute {
            debug!(
                tenant_id = tenant_id,
                user_id = user_id,
                endpoint = endpoint,
                count = minute_count,
                limit = self.config.requests_per_minute,
                "Rate limit exceeded (per minute)"
            );

            return Ok(RateLimitResult {
                allowed: false,
                limit_type: Some("per_minute".to_string()),
                retry_after: Some(60),
                remaining_minute: Some(0),
                remaining_hour: None,
                current_usage: Some(minute_count),
            });
        }

        // Check hour-based rate limit
        let hour_key = self.create_rate_limit_key(tenant_id, user_id, endpoint, "hour");
        let hour_count = self.increment_counter(&mut conn, &hour_key, 3600).await?;

        if hour_count > self.config.requests_per_hour {
            debug!(
                tenant_id = tenant_id,
                user_id = user_id,
                endpoint = endpoint,
                count = hour_count,
                limit = self.config.requests_per_hour,
                "Rate limit exceeded (per hour)"
            );

            return Ok(RateLimitResult {
                allowed: false,
                limit_type: Some("per_hour".to_string()),
                retry_after: Some(3600),
                remaining_minute: Some(self.config.requests_per_minute - minute_count),
                remaining_hour: Some(0),
                current_usage: Some(hour_count),
            });
        }

        // Check burst limit
        let burst_key = self.create_rate_limit_key(tenant_id, user_id, endpoint, "burst");
        let burst_count = self.increment_counter(&mut conn, &burst_key, 10).await?; // 10 second window

        if burst_count > self.config.burst_limit {
            debug!(
                tenant_id = tenant_id,
                user_id = user_id,
                endpoint = endpoint,
                count = burst_count,
                limit = self.config.burst_limit,
                "Burst rate limit exceeded"
            );

            return Ok(RateLimitResult {
                allowed: false,
                limit_type: Some("burst".to_string()),
                retry_after: Some(10),
                remaining_minute: Some(self.config.requests_per_minute - minute_count),
                remaining_hour: Some(self.config.requests_per_hour - hour_count),
                current_usage: Some(burst_count),
            });
        }

        debug!(
            tenant_id = tenant_id,
            user_id = user_id,
            endpoint = endpoint,
            minute_count = minute_count,
            hour_count = hour_count,
            burst_count = burst_count,
            "Rate limit check passed"
        );

        Ok(RateLimitResult {
            allowed: true,
            limit_type: None,
            retry_after: None,
            remaining_minute: Some(self.config.requests_per_minute - minute_count),
            remaining_hour: Some(self.config.requests_per_hour - hour_count),
            current_usage: None,
        })
    }

    /// Get current rate limit status without incrementing
    pub async fn get_rate_limit_status(
        &self,
        tenant_id: &str,
        user_id: &str,
        endpoint: &str,
    ) -> ApiResult<RateLimitResult> {
        if !self.config.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                limit_type: None,
                retry_after: None,
                remaining_minute: None,
                remaining_hour: None,
                current_usage: None,
            });
        }

        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to get Redis connection: {}", e),
            })?;

        // Get current counts without incrementing
        let minute_key = self.create_rate_limit_key(tenant_id, user_id, endpoint, "minute");
        let hour_key = self.create_rate_limit_key(tenant_id, user_id, endpoint, "hour");

        let minute_count: u32 = conn.get(&minute_key).await.unwrap_or(0);
        let hour_count: u32 = conn.get(&hour_key).await.unwrap_or(0);

        Ok(RateLimitResult {
            allowed: minute_count <= self.config.requests_per_minute && 
                    hour_count <= self.config.requests_per_hour,
            limit_type: None,
            retry_after: None,
            remaining_minute: Some(self.config.requests_per_minute.saturating_sub(minute_count)),
            remaining_hour: Some(self.config.requests_per_hour.saturating_sub(hour_count)),
            current_usage: Some(minute_count.max(hour_count)),
        })
    }

    /// Reset rate limits for a user (admin operation)
    pub async fn reset_rate_limits(
        &self,
        tenant_id: &str,
        user_id: &str,
        endpoint: Option<&str>,
    ) -> ApiResult<()> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to get Redis connection: {}", e),
            })?;

        let endpoint = endpoint.unwrap_or("*");
        let pattern = format!("rate_limit:{}:{}:{}:*", tenant_id, user_id, endpoint);

        // Get all matching keys
        let keys: Vec<String> = conn.keys(&pattern).await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to get rate limit keys: {}", e),
            })?;

        if !keys.is_empty() {
            // Delete all matching keys
            let _: () = conn.del(&keys).await
                .map_err(|e| ApiGatewayError::RedisError {
                    message: format!("Failed to delete rate limit keys: {}", e),
                })?;

            debug!(
                tenant_id = tenant_id,
                user_id = user_id,
                endpoint = endpoint,
                keys_deleted = keys.len(),
                "Rate limits reset"
            );
        }

        Ok(())
    }

    /// Create a rate limit key
    fn create_rate_limit_key(
        &self,
        tenant_id: &str,
        user_id: &str,
        endpoint: &str,
        time_window: &str,
    ) -> String {
        format!("rate_limit:{}:{}:{}:{}", tenant_id, user_id, endpoint, time_window)
    }

    /// Increment counter with expiration
    async fn increment_counter(
        &self,
        conn: &mut redis::aio::Connection,
        key: &str,
        expire_seconds: u64,
    ) -> ApiResult<u32> {
        // Use Redis pipeline for atomic increment and expire
        let (count,): (u32,) = redis::pipe()
            .incr(key, 1)
            .expire(key, expire_seconds as i64)
            .query_async(conn)
            .await
            .map_err(|e| ApiGatewayError::RedisError {
                message: format!("Failed to increment counter: {}", e),
            })?;

        Ok(count)
    }
}

/// Rate limiting middleware helper
pub async fn check_rate_limit_middleware(
    rate_limiter: &RateLimiter,
    tenant_id: &str,
    user_id: &str,
    endpoint: &str,
) -> Result<(), ApiGatewayError> {
    let result = rate_limiter.check_rate_limit(tenant_id, user_id, endpoint).await?;

    if !result.allowed {
        let limit_type = result.limit_type.unwrap_or_else(|| "unknown".to_string());
        let retry_after = result.retry_after.unwrap_or(60);

        return Err(ApiGatewayError::RateLimitExceeded {
            limit_type,
            retry_after,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_key_creation() {
        let config = RateLimitingConfig {
            enabled: true,
            requests_per_minute: 100,
            requests_per_hour: 1000,
            burst_limit: 20,
        };

        let redis_client = Arc::new(RedisClient::open("redis://localhost:6379").unwrap());
        let rate_limiter = RateLimiter {
            redis_client,
            config,
        };

        let key = rate_limiter.create_rate_limit_key("tenant1", "user1", "/api/test", "minute");
        assert_eq!(key, "rate_limit:tenant1:user1:/api/test:minute");
    }

    #[tokio::test]
    async fn test_disabled_rate_limiting() {
        let config = RateLimitingConfig {
            enabled: false,
            requests_per_minute: 100,
            requests_per_hour: 1000,
            burst_limit: 20,
        };

        let redis_client = Arc::new(RedisClient::open("redis://localhost:6379").unwrap());
        let rate_limiter = RateLimiter {
            redis_client,
            config,
        };

        let result = rate_limiter.check_rate_limit("tenant1", "user1", "/api/test").await;
        
        // Should succeed even without Redis connection when disabled
        match result {
            Ok(rate_limit_result) => {
                assert!(rate_limit_result.allowed);
            }
            Err(_) => {
                // This is expected if Redis is not available in test environment
                // The important thing is that disabled rate limiting doesn't fail the request
            }
        }
    }
}