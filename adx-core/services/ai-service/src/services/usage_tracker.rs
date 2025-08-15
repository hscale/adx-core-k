use crate::activities::CurrentUsage;
use crate::error::{AIError, AIResult};
use crate::types::*;
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, Client as RedisClient};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

pub struct UsageTracker {
    db_pool: Arc<PgPool>,
    redis_client: RedisClient,
}

impl UsageTracker {
    pub async fn new(database_url: &str, redis_url: &str) -> AIResult<Self> {
        let db_pool = Arc::new(
            PgPool::connect(database_url)
                .await
                .map_err(AIError::Database)?,
        );
        
        let redis_client = RedisClient::open(redis_url)
            .map_err(AIError::Redis)?;
        
        Ok(Self {
            db_pool,
            redis_client,
        })
    }
    
    pub async fn record_usage(&self, usage_record: AIUsageRecord) -> AIResult<()> {
        // Store in database for long-term tracking
        sqlx::query!(
            r#"
            INSERT INTO ai_usage_records (
                id, tenant_id, user_id, workflow_id, activity_id, model, capability,
                prompt_tokens, completion_tokens, total_tokens, estimated_cost,
                request_timestamp, response_timestamp, success, error_code
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            usage_record.id,
            usage_record.tenant_id,
            usage_record.user_id,
            usage_record.workflow_id,
            usage_record.activity_id,
            usage_record.model,
            serde_json::to_string(&usage_record.capability).unwrap(),
            usage_record.usage.prompt_tokens as i32,
            usage_record.usage.completion_tokens as i32,
            usage_record.usage.total_tokens as i32,
            usage_record.usage.estimated_cost,
            usage_record.request_timestamp,
            usage_record.response_timestamp,
            usage_record.success,
            usage_record.error_code
        )
        .execute(&*self.db_pool)
        .await
        .map_err(AIError::Database)?;
        
        // Update real-time counters in Redis
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(AIError::Redis)?;
        
        let now = Utc::now();
        let hour_key = format!("usage:{}:{}:hour:{}", 
            usage_record.tenant_id, 
            serde_json::to_string(&usage_record.capability).unwrap(),
            now.format("%Y%m%d%H")
        );
        let day_key = format!("usage:{}:{}:day:{}", 
            usage_record.tenant_id, 
            serde_json::to_string(&usage_record.capability).unwrap(),
            now.format("%Y%m%d")
        );
        
        // Increment counters
        let _: () = conn.hincrby(&hour_key, "requests", 1).await
            .map_err(AIError::Redis)?;
        let _: () = conn.hincrby(&hour_key, "tokens", usage_record.usage.total_tokens as i64).await
            .map_err(AIError::Redis)?;
        let _: () = conn.hincrbyfloat(&hour_key, "cost", usage_record.usage.estimated_cost).await
            .map_err(AIError::Redis)?;
        
        let _: () = conn.hincrby(&day_key, "requests", 1).await
            .map_err(AIError::Redis)?;
        let _: () = conn.hincrby(&day_key, "tokens", usage_record.usage.total_tokens as i64).await
            .map_err(AIError::Redis)?;
        let _: () = conn.hincrbyfloat(&day_key, "cost", usage_record.usage.estimated_cost).await
            .map_err(AIError::Redis)?;
        
        // Set expiration (keep hourly data for 7 days, daily data for 90 days)
        let _: () = conn.expire(&hour_key, 7 * 24 * 3600).await
            .map_err(AIError::Redis)?;
        let _: () = conn.expire(&day_key, 90 * 24 * 3600).await
            .map_err(AIError::Redis)?;
        
        Ok(())
    }
    
    pub async fn get_current_usage(&self, tenant_id: &str, capability: &AICapability) -> AIResult<CurrentUsage> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(AIError::Redis)?;
        
        let now = Utc::now();
        let hour_key = format!("usage:{}:{}:hour:{}", 
            tenant_id, 
            serde_json::to_string(capability).unwrap(),
            now.format("%Y%m%d%H")
        );
        
        let requests: u32 = conn.hget(&hour_key, "requests").await
            .unwrap_or(0);
        let tokens: u64 = conn.hget(&hour_key, "tokens").await
            .unwrap_or(0);
        
        Ok(CurrentUsage { requests, tokens })
    }
    
    pub async fn get_usage_stats(
        &self,
        tenant_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> AIResult<AIUsageStats> {
        let records = sqlx::query!(
            r#"
            SELECT 
                model,
                capability,
                COUNT(*) as request_count,
                SUM(total_tokens) as total_tokens,
                SUM(estimated_cost) as total_cost,
                AVG(EXTRACT(EPOCH FROM (response_timestamp - request_timestamp)) * 1000) as avg_response_time_ms,
                COUNT(*) FILTER (WHERE success = true) as successful_requests
            FROM ai_usage_records
            WHERE tenant_id = $1 
                AND request_timestamp >= $2 
                AND request_timestamp <= $3
            GROUP BY model, capability
            "#,
            tenant_id,
            period_start,
            period_end
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(AIError::Database)?;
        
        let mut usage_by_model = HashMap::new();
        let mut usage_by_capability = HashMap::new();
        let mut total_requests = 0u64;
        let mut successful_requests = 0u64;
        let mut total_tokens = 0u64;
        let mut total_cost = 0.0;
        
        for record in records {
            let requests = record.request_count.unwrap_or(0) as u64;
            let tokens = record.total_tokens.unwrap_or(0) as u64;
            let cost = record.total_cost.unwrap_or(0.0);
            let avg_response_time = record.avg_response_time_ms.unwrap_or(0.0);
            let successful = record.successful_requests.unwrap_or(0) as u64;
            
            total_requests += requests;
            successful_requests += successful;
            total_tokens += tokens;
            total_cost += cost;
            
            // Model stats
            usage_by_model.insert(record.model.clone(), ModelUsageStats {
                requests,
                tokens,
                cost,
                avg_response_time_ms: avg_response_time,
                success_rate: if requests > 0 { successful as f32 / requests as f32 } else { 0.0 },
            });
            
            // Capability stats
            let capability: AICapability = serde_json::from_str(&record.capability).unwrap_or(AICapability::TextGeneration);
            let capability_stats = usage_by_capability.entry(capability).or_insert(CapabilityUsageStats {
                requests: 0,
                tokens: 0,
                cost: 0.0,
                avg_quality_score: None,
            });
            
            capability_stats.requests += requests;
            capability_stats.tokens += tokens;
            capability_stats.cost += cost;
        }
        
        Ok(AIUsageStats {
            tenant_id: tenant_id.to_string(),
            period_start,
            period_end,
            total_requests,
            successful_requests,
            failed_requests: total_requests - successful_requests,
            total_tokens,
            total_cost,
            usage_by_model,
            usage_by_capability,
        })
    }
    
    pub async fn get_cost_breakdown(
        &self,
        tenant_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> AIResult<HashMap<String, f64>> {
        let records = sqlx::query!(
            r#"
            SELECT model, SUM(estimated_cost) as total_cost
            FROM ai_usage_records
            WHERE tenant_id = $1 
                AND request_timestamp >= $2 
                AND request_timestamp <= $3
            GROUP BY model
            "#,
            tenant_id,
            period_start,
            period_end
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(AIError::Database)?;
        
        let mut cost_breakdown = HashMap::new();
        for record in records {
            cost_breakdown.insert(record.model, record.total_cost.unwrap_or(0.0));
        }
        
        Ok(cost_breakdown)
    }
    
    pub async fn check_quota_limits(
        &self,
        tenant_id: &str,
        capability: &AICapability,
        request_tokens: u32,
    ) -> AIResult<bool> {
        let current_usage = self.get_current_usage(tenant_id, capability).await?;
        
        // Default quota limits (would normally be retrieved from database based on subscription)
        let (max_requests, max_tokens) = match capability {
            AICapability::TextGeneration => (1000, 100000),
            AICapability::TextClassification => (2000, 50000),
            AICapability::TextSummarization => (500, 200000),
            AICapability::EntityExtraction => (1000, 100000),
            _ => (100, 10000),
        };
        
        let would_exceed_requests = current_usage.requests >= max_requests;
        let would_exceed_tokens = current_usage.tokens + request_tokens as u64 > max_tokens;
        
        Ok(!would_exceed_requests && !would_exceed_tokens)
    }
    
    pub async fn get_top_users_by_usage(
        &self,
        tenant_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        limit: i64,
    ) -> AIResult<Vec<(String, u64, f64)>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                user_id,
                COUNT(*) as request_count,
                SUM(estimated_cost) as total_cost
            FROM ai_usage_records
            WHERE tenant_id = $1 
                AND request_timestamp >= $2 
                AND request_timestamp <= $3
            GROUP BY user_id
            ORDER BY total_cost DESC
            LIMIT $4
            "#,
            tenant_id,
            period_start,
            period_end,
            limit
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(AIError::Database)?;
        
        Ok(records
            .into_iter()
            .map(|r| (
                r.user_id,
                r.request_count.unwrap_or(0) as u64,
                r.total_cost.unwrap_or(0.0),
            ))
            .collect())
    }
    
    pub async fn cleanup_old_records(&self, days_to_keep: i32) -> AIResult<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep as i64);
        
        let result = sqlx::query!(
            "DELETE FROM ai_usage_records WHERE request_timestamp < $1",
            cutoff_date
        )
        .execute(&*self.db_pool)
        .await
        .map_err(AIError::Database)?;
        
        Ok(result.rows_affected())
    }
}