use sqlx::PgPool;
use std::path::Path;
use crate::{Error, Result};
use tracing::{info, warn, error};

pub struct DatabaseSeeder {
    pool: PgPool,
}

impl DatabaseSeeder {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run all seed files in the seeds directory
    pub async fn run_all_seeds(&self) -> Result<()> {
        info!("Starting database seeding process");

        // Check if we're in a development or test environment
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        match environment.as_str() {
            "development" => {
                self.run_development_seeds().await?;
            }
            "test" => {
                self.run_test_seeds().await?;
            }
            "production" => {
                warn!("Skipping database seeding in production environment");
                return Ok(());
            }
            _ => {
                warn!("Unknown environment '{}', running development seeds", environment);
                self.run_development_seeds().await?;
            }
        }

        info!("Database seeding completed successfully");
        Ok(())
    }

    /// Run development seed data
    pub async fn run_development_seeds(&self) -> Result<()> {
        info!("Running development seed data");
        
        // Check if data already exists
        if self.has_existing_data().await? {
            info!("Development data already exists, skipping seeding");
            return Ok(());
        }

        self.run_seed_file("seeds/001_development_data.sql").await?;
        info!("Development seed data loaded successfully");
        Ok(())
    }

    /// Run test seed data
    pub async fn run_test_seeds(&self) -> Result<()> {
        info!("Running test seed data");
        
        // Always clean and reseed test data
        self.clean_test_data().await?;
        self.run_seed_file("seeds/002_test_data.sql").await?;
        info!("Test seed data loaded successfully");
        Ok(())
    }

    /// Run a specific seed file
    pub async fn run_seed_file(&self, file_path: &str) -> Result<()> {
        let full_path = Path::new("services/shared").join(file_path);
        
        if !full_path.exists() {
            return Err(Error::Internal(format!("Seed file not found: {}", full_path.display())));
        }

        let sql_content = std::fs::read_to_string(&full_path)
            .map_err(|e| Error::Internal(format!("Failed to read seed file: {}", e)))?;

        info!("Executing seed file: {}", file_path);
        
        // Split the SQL content by semicolons and execute each statement
        let statements: Vec<&str> = sql_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();

        for (i, statement) in statements.iter().enumerate() {
            if statement.trim().is_empty() {
                continue;
            }

            match sqlx::query(statement).execute(&self.pool).await {
                Ok(_) => {
                    if i % 10 == 0 {
                        info!("Executed {} statements from {}", i + 1, file_path);
                    }
                }
                Err(e) => {
                    // Log the error but continue with other statements
                    // Some statements might fail due to existing data (ON CONFLICT clauses)
                    if !e.to_string().contains("duplicate key") && 
                       !e.to_string().contains("already exists") {
                        error!("Failed to execute statement in {}: {}", file_path, e);
                        error!("Statement: {}", statement);
                    }
                }
            }
        }

        info!("Completed executing seed file: {}", file_path);
        Ok(())
    }

    /// Check if development data already exists
    async fn has_existing_data(&self) -> Result<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants WHERE name LIKE '%Acme%' OR name LIKE '%TechStart%'")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    /// Clean test data before reseeding
    async fn clean_test_data(&self) -> Result<()> {
        info!("Cleaning existing test data");

        // Delete test data in reverse dependency order
        let cleanup_queries = vec![
            "DELETE FROM audit_logs WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM query_performance_log WHERE query_hash LIKE 'test_query_%'",
            "DELETE FROM api_keys WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM user_profiles WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM tenant_usage WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM tenant_billing WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM workflow_executions WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM file_permissions WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM files WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM user_sessions WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM users WHERE tenant_id IN (SELECT id FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%')",
            "DELETE FROM tenants WHERE name LIKE 'Test Tenant%' OR name LIKE '%Test%'",
        ];

        for query in cleanup_queries {
            match sqlx::query(query).execute(&self.pool).await {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        info!("Cleaned {} rows with query: {}", result.rows_affected(), query);
                    }
                }
                Err(e) => {
                    warn!("Failed to execute cleanup query: {} - Error: {}", query, e);
                }
            }
        }

        info!("Test data cleanup completed");
        Ok(())
    }

    /// Seed specific tenant data for testing
    pub async fn seed_tenant_data(&self, tenant_name: &str, admin_email: &str) -> Result<String> {
        info!("Seeding data for tenant: {}", tenant_name);

        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let session_id = uuid::Uuid::new_v4();

        // Create tenant
        sqlx::query(
            r#"
            INSERT INTO tenants (id, name, slug, admin_email, subscription_tier, isolation_level, is_active)
            VALUES ($1, $2, $3, $4, 'professional', 'row', true)
            "#
        )
        .bind(tenant_id)
        .bind(tenant_name)
        .bind(tenant_name.to_lowercase().replace(" ", "-"))
        .bind(admin_email)
        .execute(&self.pool)
        .await?;

        // Create admin user
        sqlx::query(
            r#"
            INSERT INTO users (id, tenant_id, email, password_hash, first_name, last_name, status, roles, permissions, email_verified_at)
            VALUES ($1, $2, $3, $4, 'Test', 'Admin', 'active', $5, $6, NOW())
            "#
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(admin_email)
        .bind("$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa") // password: test123
        .bind(&vec!["admin", "user"])
        .bind(&vec!["tenant:admin", "user:admin", "file:admin", "workflow:admin"])
        .execute(&self.pool)
        .await?;

        // Create active session
        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, tenant_id, session_token, refresh_token, status, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, 'active', '127.0.0.1', 'Test Agent', NOW() + INTERVAL '7 days')
            "#
        )
        .bind(session_id)
        .bind(user_id)
        .bind(tenant_id)
        .bind(format!("test_session_{}", tenant_id))
        .bind(format!("test_refresh_{}", tenant_id))
        .execute(&self.pool)
        .await?;

        info!("Successfully seeded tenant: {} with ID: {}", tenant_name, tenant_id);
        Ok(tenant_id.to_string())
    }

    /// Get seeding statistics
    pub async fn get_seeding_stats(&self) -> Result<SeedingStats> {
        let tenants_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
            .fetch_one(&self.pool)
            .await?;

        let users_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        let files_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
            .fetch_one(&self.pool)
            .await?;

        let workflows_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workflow_executions")
            .fetch_one(&self.pool)
            .await?;

        let active_sessions_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE status = 'active'")
            .fetch_one(&self.pool)
            .await?;

        Ok(SeedingStats {
            tenants: tenants_count.0,
            users: users_count.0,
            files: files_count.0,
            workflows: workflows_count.0,
            active_sessions: active_sessions_count.0,
        })
    }

    /// Validate seeded data integrity
    pub async fn validate_seeded_data(&self) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for users without tenants
        let orphaned_users: Vec<(String,)> = sqlx::query_as(
            "SELECT u.email FROM users u LEFT JOIN tenants t ON u.tenant_id = t.id WHERE t.id IS NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        for (email,) in orphaned_users {
            issues.push(ValidationIssue {
                issue_type: "orphaned_user".to_string(),
                description: format!("User {} has no associated tenant", email),
                severity: "high".to_string(),
            });
        }

        // Check for files without users
        let orphaned_files: Vec<(String,)> = sqlx::query_as(
            "SELECT f.filename FROM files f LEFT JOIN users u ON f.user_id = u.id WHERE u.id IS NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        for (filename,) in orphaned_files {
            issues.push(ValidationIssue {
                issue_type: "orphaned_file".to_string(),
                description: format!("File {} has no associated user", filename),
                severity: "medium".to_string(),
            });
        }

        // Check for expired sessions that are still active
        let expired_active_sessions: Vec<(String,)> = sqlx::query_as(
            "SELECT session_token FROM user_sessions WHERE status = 'active' AND expires_at < NOW()"
        )
        .fetch_all(&self.pool)
        .await?;

        for (token,) in expired_active_sessions {
            issues.push(ValidationIssue {
                issue_type: "expired_active_session".to_string(),
                description: format!("Session {} is marked active but has expired", token),
                severity: "low".to_string(),
            });
        }

        Ok(issues)
    }
}

#[derive(Debug)]
pub struct SeedingStats {
    pub tenants: i64,
    pub users: i64,
    pub files: i64,
    pub workflows: i64,
    pub active_sessions: i64,
}

#[derive(Debug)]
pub struct ValidationIssue {
    pub issue_type: String,
    pub description: String,
    pub severity: String,
}

impl std::fmt::Display for SeedingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database Seeding Stats:\n  Tenants: {}\n  Users: {}\n  Files: {}\n  Workflows: {}\n  Active Sessions: {}",
            self.tenants, self.users, self.files, self.workflows, self.active_sessions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn setup_test_pool() -> PgPool {
        // This would be set up with a test database
        // For now, we'll skip the actual database tests
        todo!("Set up test database connection")
    }

    #[tokio::test]
    #[ignore] // Ignore until we have a test database setup
    async fn test_run_test_seeds() {
        let pool = setup_test_pool().await;
        let seeder = DatabaseSeeder::new(pool);
        
        let result = seeder.run_test_seeds().await;
        assert!(result.is_ok());
        
        let stats = seeder.get_seeding_stats().await.unwrap();
        assert!(stats.tenants > 0);
        assert!(stats.users > 0);
    }

    #[tokio::test]
    #[ignore] // Ignore until we have a test database setup
    async fn test_validate_seeded_data() {
        let pool = setup_test_pool().await;
        let seeder = DatabaseSeeder::new(pool);
        
        seeder.run_test_seeds().await.unwrap();
        let issues = seeder.validate_seeded_data().await.unwrap();
        
        // Should have no validation issues with properly seeded test data
        assert_eq!(issues.len(), 0);
    }
}