use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    ModuleResult, ModuleError, ModuleRepository as ModuleRepositoryTrait,
    ModuleMetadata, ModuleInstance, ModuleSearchQuery, ModuleSearchResult,
    ModuleStatus, SortBy,
};

/// PostgreSQL-based module repository implementation
pub struct PostgresModuleRepository {
    pool: PgPool,
}

impl PostgresModuleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize database tables for module storage
    pub async fn initialize(&self) -> ModuleResult<()> {
        // Create modules table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS modules (
                id VARCHAR PRIMARY KEY,
                name VARCHAR NOT NULL,
                version VARCHAR NOT NULL,
                description TEXT,
                long_description TEXT,
                author_name VARCHAR NOT NULL,
                author_email VARCHAR,
                author_website VARCHAR,
                author_organization VARCHAR,
                license VARCHAR NOT NULL,
                homepage VARCHAR,
                repository VARCHAR,
                documentation VARCHAR,
                keywords TEXT[],
                categories TEXT[],
                min_adx_version VARCHAR NOT NULL,
                max_adx_version VARCHAR,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create module instances table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS module_instances (
                id UUID PRIMARY KEY,
                module_id VARCHAR NOT NULL,
                tenant_id VARCHAR NOT NULL,
                version VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                configuration JSONB NOT NULL DEFAULT '{}',
                installation_path VARCHAR NOT NULL,
                installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                activated_at TIMESTAMPTZ,
                last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                memory_mb BIGINT NOT NULL DEFAULT 0,
                cpu_percent REAL NOT NULL DEFAULT 0.0,
                disk_mb BIGINT NOT NULL DEFAULT 0,
                network_in_mbps REAL NOT NULL DEFAULT 0.0,
                network_out_mbps REAL NOT NULL DEFAULT 0.0,
                active_connections INTEGER NOT NULL DEFAULT 0,
                is_healthy BOOLEAN NOT NULL DEFAULT true,
                last_health_check TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                error_count INTEGER NOT NULL DEFAULT 0,
                warning_count INTEGER NOT NULL DEFAULT 0,
                uptime_seconds BIGINT NOT NULL DEFAULT 0,
                response_time_ms BIGINT NOT NULL DEFAULT 0,
                FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_modules_name ON modules(name)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_modules_categories ON modules USING GIN(categories)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_module_instances_tenant ON module_instances(tenant_id)"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_module_instances_module ON module_instances(module_id)"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl ModuleRepositoryTrait for PostgresModuleRepository {
    async fn save_metadata(&self, metadata: &ModuleMetadata) -> ModuleResult<()> {
        let keywords: Vec<String> = metadata.keywords.clone();
        let categories: Vec<String> = metadata.categories.iter()
            .map(|c| format!("{:?}", c))
            .collect();

        sqlx::query!(
            r#"
            INSERT INTO modules (
                id, name, version, description, long_description,
                author_name, author_email, author_website, author_organization,
                license, homepage, repository, documentation,
                keywords, categories, min_adx_version, max_adx_version,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
            )
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                version = EXCLUDED.version,
                description = EXCLUDED.description,
                long_description = EXCLUDED.long_description,
                author_name = EXCLUDED.author_name,
                author_email = EXCLUDED.author_email,
                author_website = EXCLUDED.author_website,
                author_organization = EXCLUDED.author_organization,
                license = EXCLUDED.license,
                homepage = EXCLUDED.homepage,
                repository = EXCLUDED.repository,
                documentation = EXCLUDED.documentation,
                keywords = EXCLUDED.keywords,
                categories = EXCLUDED.categories,
                min_adx_version = EXCLUDED.min_adx_version,
                max_adx_version = EXCLUDED.max_adx_version,
                updated_at = EXCLUDED.updated_at
            "#,
            metadata.id,
            metadata.name,
            metadata.version.to_string(),
            metadata.description,
            metadata.long_description,
            metadata.author.name,
            metadata.author.email,
            metadata.author.website,
            metadata.author.organization,
            metadata.license,
            metadata.homepage,
            metadata.repository,
            metadata.documentation,
            &keywords,
            &categories,
            metadata.adx_core_version.min_version.to_string(),
            metadata.adx_core_version.max_version.as_ref().map(|v| v.to_string()),
            metadata.created_at,
            metadata.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_metadata(&self, module_id: &str) -> ModuleResult<Option<ModuleMetadata>> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, name, version, description, long_description,
                author_name, author_email, author_website, author_organization,
                license, homepage, repository, documentation,
                keywords, categories, min_adx_version, max_adx_version,
                created_at, updated_at
            FROM modules 
            WHERE id = $1
            "#,
            module_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let version = semver::Version::parse(&row.version)
                .map_err(|e| ModuleError::SerializationError(e.to_string()))?;
            
            let min_version = semver::Version::parse(&row.min_adx_version)
                .map_err(|e| ModuleError::SerializationError(e.to_string()))?;
            
            let max_version = if let Some(max_ver) = row.max_adx_version {
                Some(semver::Version::parse(&max_ver)
                    .map_err(|e| ModuleError::SerializationError(e.to_string()))?)
            } else {
                None
            };

            let categories = row.categories.into_iter()
                .filter_map(|c| match c.as_str() {
                    "BusinessManagement" => Some(crate::ModuleCategory::BusinessManagement),
                    "Analytics" => Some(crate::ModuleCategory::Analytics),
                    "Integration" => Some(crate::ModuleCategory::Integration),
                    "Workflow" => Some(crate::ModuleCategory::Workflow),
                    "Security" => Some(crate::ModuleCategory::Security),
                    "Communication" => Some(crate::ModuleCategory::Communication),
                    "FileManagement" => Some(crate::ModuleCategory::FileManagement),
                    "UserInterface" => Some(crate::ModuleCategory::UserInterface),
                    "Development" => Some(crate::ModuleCategory::Development),
                    "Utility" => Some(crate::ModuleCategory::Utility),
                    _ => Some(crate::ModuleCategory::Custom(c)),
                })
                .collect();

            let metadata = ModuleMetadata {
                id: row.id,
                name: row.name,
                version,
                description: row.description,
                long_description: row.long_description,
                author: crate::ModuleAuthor {
                    name: row.author_name,
                    email: row.author_email,
                    website: row.author_website,
                    organization: row.author_organization,
                },
                license: row.license,
                homepage: row.homepage,
                repository: row.repository,
                documentation: row.documentation,
                keywords: row.keywords,
                categories,
                adx_core_version: crate::VersionRequirement {
                    min_version,
                    max_version,
                    compatible_versions: vec![],
                },
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    async fn list_modules(&self) -> ModuleResult<Vec<ModuleMetadata>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, name, version, description, long_description,
                author_name, author_email, author_website, author_organization,
                license, homepage, repository, documentation,
                keywords, categories, min_adx_version, max_adx_version,
                created_at, updated_at
            FROM modules 
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut modules = Vec::new();
        for row in rows {
            let version = semver::Version::parse(&row.version)
                .map_err(|e| ModuleError::SerializationError(e.to_string()))?;
            
            let min_version = semver::Version::parse(&row.min_adx_version)
                .map_err(|e| ModuleError::SerializationError(e.to_string()))?;
            
            let max_version = if let Some(max_ver) = row.max_adx_version {
                Some(semver::Version::parse(&max_ver)
                    .map_err(|e| ModuleError::SerializationError(e.to_string()))?)
            } else {
                None
            };

            let categories = row.categories.into_iter()
                .filter_map(|c| match c.as_str() {
                    "BusinessManagement" => Some(crate::ModuleCategory::BusinessManagement),
                    "Analytics" => Some(crate::ModuleCategory::Analytics),
                    "Integration" => Some(crate::ModuleCategory::Integration),
                    "Workflow" => Some(crate::ModuleCategory::Workflow),
                    "Security" => Some(crate::ModuleCategory::Security),
                    "Communication" => Some(crate::ModuleCategory::Communication),
                    "FileManagement" => Some(crate::ModuleCategory::FileManagement),
                    "UserInterface" => Some(crate::ModuleCategory::UserInterface),
                    "Development" => Some(crate::ModuleCategory::Development),
                    "Utility" => Some(crate::ModuleCategory::Utility),
                    _ => Some(crate::ModuleCategory::Custom(c)),
                })
                .collect();

            let metadata = ModuleMetadata {
                id: row.id,
                name: row.name,
                version,
                description: row.description,
                long_description: row.long_description,
                author: crate::ModuleAuthor {
                    name: row.author_name,
                    email: row.author_email,
                    website: row.author_website,
                    organization: row.author_organization,
                },
                license: row.license,
                homepage: row.homepage,
                repository: row.repository,
                documentation: row.documentation,
                keywords: row.keywords,
                categories,
                adx_core_version: crate::VersionRequirement {
                    min_version,
                    max_version,
                    compatible_versions: vec![],
                },
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            modules.push(metadata);
        }

        Ok(modules)
    }

    async fn search_modules(&self, query: &ModuleSearchQuery) -> ModuleResult<ModuleSearchResult> {
        let mut sql = String::from(
            r#"
            SELECT 
                id, name, version, description, long_description,
                author_name, author_email, author_website, author_organization,
                license, homepage, repository, documentation,
                keywords, categories, min_adx_version, max_adx_version,
                created_at, updated_at
            FROM modules 
            WHERE 1=1
            "#
        );

        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        // Add search query filter
        if let Some(search_query) = &query.query {
            param_count += 1;
            sql.push_str(&format!(
                " AND (name ILIKE ${} OR description ILIKE ${})",
                param_count, param_count
            ));
            params.push(Box::new(format!("%{}%", search_query)));
        }

        // Add category filter
        if !query.categories.is_empty() {
            param_count += 1;
            let category_strings: Vec<String> = query.categories.iter()
                .map(|c| format!("{:?}", c))
                .collect();
            sql.push_str(&format!(" AND categories && ${}", param_count));
            params.push(Box::new(category_strings));
        }

        // Add author filter
        if let Some(author) = &query.author {
            param_count += 1;
            sql.push_str(&format!(" AND author_name ILIKE ${}", param_count));
            params.push(Box::new(format!("%{}%", author)));
        }

        // Add keywords filter
        if !query.keywords.is_empty() {
            param_count += 1;
            sql.push_str(&format!(" AND keywords && ${}", param_count));
            params.push(Box::new(query.keywords.clone()));
        }

        // Add sorting
        match query.sort_by {
            SortBy::Name => sql.push_str(" ORDER BY name"),
            SortBy::Version => sql.push_str(" ORDER BY version DESC"),
            SortBy::UpdatedAt => sql.push_str(" ORDER BY updated_at DESC"),
            SortBy::CreatedAt => sql.push_str(" ORDER BY created_at DESC"),
            _ => sql.push_str(" ORDER BY name"), // Default sorting
        }

        // Add pagination
        param_count += 1;
        sql.push_str(&format!(" LIMIT ${}", param_count));
        params.push(Box::new(query.limit as i64));

        param_count += 1;
        sql.push_str(&format!(" OFFSET ${}", param_count));
        params.push(Box::new(query.offset as i64));

        // Execute query (simplified - in real implementation would use dynamic query building)
        let modules = self.list_modules().await?; // Simplified for now
        
        // Filter and paginate results
        let filtered_modules: Vec<ModuleMetadata> = modules.into_iter()
            .filter(|module| {
                // Apply filters
                if let Some(search_query) = &query.query {
                    if !module.name.to_lowercase().contains(&search_query.to_lowercase()) &&
                       !module.description.to_lowercase().contains(&search_query.to_lowercase()) {
                        return false;
                    }
                }

                if !query.categories.is_empty() {
                    if !module.categories.iter().any(|c| query.categories.contains(c)) {
                        return false;
                    }
                }

                if let Some(author) = &query.author {
                    if !module.author.name.to_lowercase().contains(&author.to_lowercase()) {
                        return false;
                    }
                }

                true
            })
            .skip(query.offset as usize)
            .take(query.limit as usize)
            .collect();

        let total_count = filtered_modules.len() as u64;
        let has_more = total_count > query.limit as u64;

        // Build facets
        let mut category_facets = HashMap::new();
        let mut author_facets = HashMap::new();
        
        for module in &filtered_modules {
            for category in &module.categories {
                *category_facets.entry(category.clone()).or_insert(0) += 1;
            }
            *author_facets.entry(module.author.name.clone()).or_insert(0) += 1;
        }

        Ok(ModuleSearchResult {
            modules: filtered_modules,
            total_count,
            has_more,
            facets: crate::SearchFacets {
                categories: category_facets,
                authors: author_facets,
                versions: HashMap::new(),
            },
        })
    }

    async fn save_instance(&self, instance: &ModuleInstance) -> ModuleResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO module_instances (
                id, module_id, tenant_id, version, status, configuration,
                installation_path, installed_at, activated_at, last_updated,
                memory_mb, cpu_percent, disk_mb, network_in_mbps, network_out_mbps,
                active_connections, is_healthy, last_health_check, error_count,
                warning_count, uptime_seconds, response_time_ms
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
            )
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status,
                configuration = EXCLUDED.configuration,
                activated_at = EXCLUDED.activated_at,
                last_updated = EXCLUDED.last_updated,
                memory_mb = EXCLUDED.memory_mb,
                cpu_percent = EXCLUDED.cpu_percent,
                disk_mb = EXCLUDED.disk_mb,
                network_in_mbps = EXCLUDED.network_in_mbps,
                network_out_mbps = EXCLUDED.network_out_mbps,
                active_connections = EXCLUDED.active_connections,
                is_healthy = EXCLUDED.is_healthy,
                last_health_check = EXCLUDED.last_health_check,
                error_count = EXCLUDED.error_count,
                warning_count = EXCLUDED.warning_count,
                uptime_seconds = EXCLUDED.uptime_seconds,
                response_time_ms = EXCLUDED.response_time_ms
            "#,
            instance.id,
            instance.module_id,
            instance.tenant_id,
            instance.version.to_string(),
            format!("{:?}", instance.status),
            instance.configuration,
            instance.installation_path,
            instance.installed_at,
            instance.activated_at,
            instance.last_updated,
            instance.resource_usage.memory_mb as i64,
            instance.resource_usage.cpu_percent,
            instance.resource_usage.disk_mb as i64,
            instance.resource_usage.network_in_mbps,
            instance.resource_usage.network_out_mbps,
            instance.resource_usage.active_connections as i32,
            instance.health_status.is_healthy,
            instance.health_status.last_health_check,
            instance.health_status.error_count as i32,
            instance.health_status.warning_count as i32,
            instance.health_status.uptime_seconds as i64,
            instance.health_status.response_time_ms as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_instance(&self, instance_id: Uuid) -> ModuleResult<Option<ModuleInstance>> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, module_id, tenant_id, version, status, configuration,
                installation_path, installed_at, activated_at, last_updated,
                memory_mb, cpu_percent, disk_mb, network_in_mbps, network_out_mbps,
                active_connections, is_healthy, last_health_check, error_count,
                warning_count, uptime_seconds, response_time_ms
            FROM module_instances 
            WHERE id = $1
            "#,
            instance_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let version = semver::Version::parse(&row.version)
                .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

            let status = match row.status.as_str() {
                "Downloaded" => ModuleStatus::Downloaded,
                "Installing" => ModuleStatus::Installing,
                "Installed" => ModuleStatus::Installed,
                "Activating" => ModuleStatus::Activating,
                "Active" => ModuleStatus::Active,
                "Deactivating" => ModuleStatus::Deactivating,
                "Inactive" => ModuleStatus::Inactive,
                "Updating" => ModuleStatus::Updating,
                "Uninstalling" => ModuleStatus::Uninstalling,
                "Failed" => ModuleStatus::Failed,
                "Suspended" => ModuleStatus::Suspended,
                _ => ModuleStatus::Failed,
            };

            let instance = ModuleInstance {
                id: row.id,
                module_id: row.module_id,
                tenant_id: row.tenant_id,
                version,
                status,
                configuration: row.configuration,
                installation_path: row.installation_path,
                installed_at: row.installed_at,
                activated_at: row.activated_at,
                last_updated: row.last_updated,
                resource_usage: crate::ResourceUsage {
                    memory_mb: row.memory_mb as u64,
                    cpu_percent: row.cpu_percent,
                    disk_mb: row.disk_mb as u64,
                    network_in_mbps: row.network_in_mbps,
                    network_out_mbps: row.network_out_mbps,
                    active_connections: row.active_connections as u32,
                    last_measured: chrono::Utc::now(),
                },
                health_status: crate::HealthStatus {
                    is_healthy: row.is_healthy,
                    last_health_check: row.last_health_check,
                    error_count: row.error_count as u32,
                    warning_count: row.warning_count as u32,
                    uptime_seconds: row.uptime_seconds as u64,
                    response_time_ms: row.response_time_ms as u64,
                },
            };

            Ok(Some(instance))
        } else {
            Ok(None)
        }
    }

    async fn list_tenant_instances(&self, tenant_id: &str) -> ModuleResult<Vec<ModuleInstance>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, module_id, tenant_id, version, status, configuration,
                installation_path, installed_at, activated_at, last_updated,
                memory_mb, cpu_percent, disk_mb, network_in_mbps, network_out_mbps,
                active_connections, is_healthy, last_health_check, error_count,
                warning_count, uptime_seconds, response_time_ms
            FROM module_instances 
            WHERE tenant_id = $1
            ORDER BY installed_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut instances = Vec::new();
        for row in rows {
            let version = semver::Version::parse(&row.version)
                .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

            let status = match row.status.as_str() {
                "Downloaded" => ModuleStatus::Downloaded,
                "Installing" => ModuleStatus::Installing,
                "Installed" => ModuleStatus::Installed,
                "Activating" => ModuleStatus::Activating,
                "Active" => ModuleStatus::Active,
                "Deactivating" => ModuleStatus::Deactivating,
                "Inactive" => ModuleStatus::Inactive,
                "Updating" => ModuleStatus::Updating,
                "Uninstalling" => ModuleStatus::Uninstalling,
                "Failed" => ModuleStatus::Failed,
                "Suspended" => ModuleStatus::Suspended,
                _ => ModuleStatus::Failed,
            };

            let instance = ModuleInstance {
                id: row.id,
                module_id: row.module_id,
                tenant_id: row.tenant_id,
                version,
                status,
                configuration: row.configuration,
                installation_path: row.installation_path,
                installed_at: row.installed_at,
                activated_at: row.activated_at,
                last_updated: row.last_updated,
                resource_usage: crate::ResourceUsage {
                    memory_mb: row.memory_mb as u64,
                    cpu_percent: row.cpu_percent,
                    disk_mb: row.disk_mb as u64,
                    network_in_mbps: row.network_in_mbps,
                    network_out_mbps: row.network_out_mbps,
                    active_connections: row.active_connections as u32,
                    last_measured: chrono::Utc::now(),
                },
                health_status: crate::HealthStatus {
                    is_healthy: row.is_healthy,
                    last_health_check: row.last_health_check,
                    error_count: row.error_count as u32,
                    warning_count: row.warning_count as u32,
                    uptime_seconds: row.uptime_seconds as u64,
                    response_time_ms: row.response_time_ms as u64,
                },
            };

            instances.push(instance);
        }

        Ok(instances)
    }

    async fn update_instance_status(&self, instance_id: Uuid, status: ModuleStatus) -> ModuleResult<()> {
        sqlx::query!(
            "UPDATE module_instances SET status = $1, last_updated = NOW() WHERE id = $2",
            format!("{:?}", status),
            instance_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_instance(&self, instance_id: Uuid) -> ModuleResult<()> {
        sqlx::query!(
            "DELETE FROM module_instances WHERE id = $1",
            instance_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}