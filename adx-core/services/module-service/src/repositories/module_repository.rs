use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::ModuleServiceError;
use crate::models::{ModuleRecord, ModuleDependencyRecord, ModuleVersionRecord};
use crate::types::{Module, ModuleSearchRequest, ModuleSearchResponse, SearchFacets};

#[async_trait]
pub trait ModuleRepositoryTrait {
    async fn create_module(&self, module: &Module) -> Result<Module, ModuleServiceError>;
    async fn get_module_by_id(&self, id: &str) -> Result<Option<Module>, ModuleServiceError>;
    async fn get_module_by_name_version(&self, name: &str, version: &str) -> Result<Option<Module>, ModuleServiceError>;
    async fn update_module(&self, id: &str, module: &Module) -> Result<Module, ModuleServiceError>;
    async fn delete_module(&self, id: &str) -> Result<(), ModuleServiceError>;
    async fn list_modules(&self, tenant_id: Option<&str>, page: u32, page_size: u32) -> Result<Vec<Module>, ModuleServiceError>;
    async fn search_modules(&self, request: &ModuleSearchRequest) -> Result<ModuleSearchResponse, ModuleServiceError>;
    async fn get_module_dependencies(&self, module_id: &str) -> Result<Vec<ModuleDependencyRecord>, ModuleServiceError>;
    async fn add_module_dependency(&self, module_id: &str, dependency_id: &str, version_requirement: &str, optional: bool) -> Result<(), ModuleServiceError>;
    async fn remove_module_dependency(&self, module_id: &str, dependency_id: &str) -> Result<(), ModuleServiceError>;
    async fn get_module_versions(&self, module_id: &str) -> Result<Vec<ModuleVersionRecord>, ModuleServiceError>;
    async fn create_module_version(&self, version: &ModuleVersionRecord) -> Result<ModuleVersionRecord, ModuleServiceError>;
    async fn get_module_stats(&self, module_id: &str) -> Result<ModuleStats, ModuleServiceError>;
}

pub struct ModuleRepository {
    pool: PgPool,
}

impl ModuleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModuleRepositoryTrait for ModuleRepository {
    async fn create_module(&self, module: &Module) -> Result<Module, ModuleServiceError> {
        let manifest_json = serde_json::to_value(&module.manifest)?;
        
        let record = sqlx::query_as::<_, ModuleRecord>(
            r#"
            INSERT INTO modules (
                id, name, version, description, author_name, author_email,
                author_website, author_organization, category, manifest_json,
                status, tenant_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&module.id)
        .bind(&module.name)
        .bind(&module.version)
        .bind(&module.description)
        .bind(&module.author.name)
        .bind(&module.author.email)
        .bind(&module.author.website)
        .bind(&module.author.organization)
        .bind(serde_json::to_string(&module.category)?.trim_matches('"'))
        .bind(&manifest_json)
        .bind(serde_json::to_string(&module.status)?.trim_matches('"'))
        .bind(&module.tenant_id)
        .fetch_one(&self.pool)
        .await?;

        record.to_module().map_err(|e| ModuleServiceError::SerializationError(e))
    }

    async fn get_module_by_id(&self, id: &str) -> Result<Option<Module>, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleRecord>(
            "SELECT * FROM modules WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(record) => Ok(Some(record.to_module().map_err(|e| ModuleServiceError::SerializationError(e))?)),
            None => Ok(None),
        }
    }

    async fn get_module_by_name_version(&self, name: &str, version: &str) -> Result<Option<Module>, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleRecord>(
            "SELECT * FROM modules WHERE name = $1 AND version = $2"
        )
        .bind(name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(record) => Ok(Some(record.to_module().map_err(|e| ModuleServiceError::SerializationError(e))?)),
            None => Ok(None),
        }
    }

    async fn update_module(&self, id: &str, module: &Module) -> Result<Module, ModuleServiceError> {
        let manifest_json = serde_json::to_value(&module.manifest)?;
        
        let record = sqlx::query_as::<_, ModuleRecord>(
            r#"
            UPDATE modules SET
                name = $2, version = $3, description = $4, author_name = $5,
                author_email = $6, author_website = $7, author_organization = $8,
                category = $9, manifest_json = $10, status = $11, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&module.name)
        .bind(&module.version)
        .bind(&module.description)
        .bind(&module.author.name)
        .bind(&module.author.email)
        .bind(&module.author.website)
        .bind(&module.author.organization)
        .bind(serde_json::to_string(&module.category)?.trim_matches('"'))
        .bind(&manifest_json)
        .bind(serde_json::to_string(&module.status)?.trim_matches('"'))
        .fetch_one(&self.pool)
        .await?;

        record.to_module().map_err(|e| ModuleServiceError::SerializationError(e))
    }

    async fn delete_module(&self, id: &str) -> Result<(), ModuleServiceError> {
        sqlx::query("DELETE FROM modules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn list_modules(&self, tenant_id: Option<&str>, page: u32, page_size: u32) -> Result<Vec<Module>, ModuleServiceError> {
        let offset = (page.saturating_sub(1)) * page_size;
        
        let query = match tenant_id {
            Some(tenant_id) => {
                sqlx::query_as::<_, ModuleRecord>(
                    "SELECT * FROM modules WHERE tenant_id = $1 OR tenant_id IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3"
                )
                .bind(tenant_id)
                .bind(page_size as i64)
                .bind(offset as i64)
            }
            None => {
                sqlx::query_as::<_, ModuleRecord>(
                    "SELECT * FROM modules WHERE tenant_id IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2"
                )
                .bind(page_size as i64)
                .bind(offset as i64)
            }
        };

        let records = query.fetch_all(&self.pool).await?;
        
        let mut modules = Vec::new();
        for record in records {
            modules.push(record.to_module().map_err(|e| ModuleServiceError::SerializationError(e))?);
        }
        
        Ok(modules)
    }

    async fn search_modules(&self, request: &ModuleSearchRequest) -> Result<ModuleSearchResponse, ModuleServiceError> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT m.*, mm.rating_average, mm.download_count FROM modules m LEFT JOIN module_marketplace mm ON m.id = mm.id WHERE 1=1"
        );
        
        let mut count_builder = sqlx::QueryBuilder::new(
            "SELECT COUNT(*) FROM modules m LEFT JOIN module_marketplace mm ON m.id = mm.id WHERE 1=1"
        );

        // Add search conditions
        if let Some(query) = &request.query {
            query_builder.push(" AND (m.name ILIKE ");
            query_builder.push_bind(format!("%{}%", query));
            query_builder.push(" OR m.description ILIKE ");
            query_builder.push_bind(format!("%{}%", query));
            query_builder.push(")");
            
            count_builder.push(" AND (m.name ILIKE ");
            count_builder.push_bind(format!("%{}%", query));
            count_builder.push(" OR m.description ILIKE ");
            count_builder.push_bind(format!("%{}%", query));
            count_builder.push(")");
        }

        if let Some(category) = &request.category {
            let category_str = serde_json::to_string(category)?.trim_matches('"').to_string();
            query_builder.push(" AND m.category = ");
            query_builder.push_bind(&category_str);
            count_builder.push(" AND m.category = ");
            count_builder.push_bind(&category_str);
        }

        if let Some(author) = &request.author {
            query_builder.push(" AND m.author_name ILIKE ");
            query_builder.push_bind(format!("%{}%", author));
            count_builder.push(" AND m.author_name ILIKE ");
            count_builder.push_bind(format!("%{}%", author));
        }

        if let Some(rating_min) = request.rating_min {
            query_builder.push(" AND mm.rating_average >= ");
            query_builder.push_bind(rating_min);
            count_builder.push(" AND mm.rating_average >= ");
            count_builder.push_bind(rating_min);
        }

        // Add sorting
        match (&request.sort_by, &request.sort_order) {
            (Some(sort_by), Some(sort_order)) => {
                let sort_column = match sort_by {
                    crate::types::SortBy::Name => "m.name",
                    crate::types::SortBy::Rating => "mm.rating_average",
                    crate::types::SortBy::Downloads => "mm.download_count",
                    crate::types::SortBy::Updated => "m.updated_at",
                    crate::types::SortBy::Price => "mm.price_amount",
                    crate::types::SortBy::Relevance => "m.created_at",
                };
                
                let order = match sort_order {
                    crate::types::SortOrder::Asc => "ASC",
                    crate::types::SortOrder::Desc => "DESC",
                };
                
                query_builder.push(" ORDER BY ");
                query_builder.push(sort_column);
                query_builder.push(" ");
                query_builder.push(order);
            }
            _ => {
                query_builder.push(" ORDER BY m.created_at DESC");
            }
        }

        // Add pagination
        let offset = (request.page.saturating_sub(1)) * request.page_size;
        query_builder.push(" LIMIT ");
        query_builder.push_bind(request.page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        // Execute queries
        let total_count: i64 = count_builder.build_query_scalar().fetch_one(&self.pool).await?;
        
        let rows = query_builder.build().fetch_all(&self.pool).await?;
        
        let mut modules = Vec::new();
        for row in rows {
            let record = ModuleRecord {
                id: row.get("id"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                author_name: row.get("author_name"),
                author_email: row.get("author_email"),
                author_website: row.get("author_website"),
                author_organization: row.get("author_organization"),
                category: row.get("category"),
                manifest_json: row.get("manifest_json"),
                status: row.get("status"),
                tenant_id: row.get("tenant_id"),
                package_url: row.get("package_url"),
                package_hash: row.get("package_hash"),
                installation_id: row.get("installation_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            
            // Convert to marketplace listing format
            let module = record.to_module().map_err(|e| ModuleServiceError::SerializationError(e))?;
            
            // Create marketplace listing from module
            let listing = crate::types::MarketplaceListing {
                id: module.id,
                name: module.name,
                description: module.description,
                long_description: "".to_string(), // Would be populated from marketplace table
                version: module.version,
                author: module.author,
                category: module.category,
                subcategory: None,
                price: None,
                rating: row.try_get::<f32, _>("rating_average").unwrap_or(0.0),
                review_count: 0,
                downloads: row.try_get::<i64, _>("download_count").unwrap_or(0) as u64,
                active_installations: 0,
                screenshots: vec![],
                demo_url: None,
                documentation_url: "".to_string(),
                support_url: "".to_string(),
                tags: vec![],
                supported_platforms: vec![],
                compatibility: crate::types::CompatibilityInfo {
                    adx_core_versions: vec![],
                    node_version: None,
                    browser_support: None,
                    os_support: None,
                },
                security_scan_results: crate::types::SecurityScanResults {
                    passed: true,
                    score: 100,
                    vulnerabilities: vec![],
                    scan_date: chrono::Utc::now(),
                    scanner_version: "1.0.0".to_string(),
                },
                performance_metrics: crate::types::PerformanceMetrics {
                    bundle_size_kb: 0,
                    load_time_ms: 0,
                    memory_usage_mb: 0,
                    cpu_usage_percent: 0.0,
                },
                last_updated: module.updated_at,
                changelog: vec![],
            };
            
            modules.push(listing);
        }

        // Generate facets (simplified)
        let facets = SearchFacets {
            categories: HashMap::new(),
            authors: HashMap::new(),
            tags: HashMap::new(),
            price_ranges: HashMap::new(),
            ratings: HashMap::new(),
        };

        let total_pages = ((total_count as u32) + request.page_size - 1) / request.page_size;

        Ok(ModuleSearchResponse {
            modules,
            total_count: total_count as u64,
            page: request.page,
            page_size: request.page_size,
            total_pages,
            facets,
        })
    }

    async fn get_module_dependencies(&self, module_id: &str) -> Result<Vec<ModuleDependencyRecord>, ModuleServiceError> {
        let records = sqlx::query_as::<_, ModuleDependencyRecord>(
            "SELECT * FROM module_dependencies WHERE module_id = $1 ORDER BY created_at"
        )
        .bind(module_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn add_module_dependency(&self, module_id: &str, dependency_id: &str, version_requirement: &str, optional: bool) -> Result<(), ModuleServiceError> {
        sqlx::query(
            r#"
            INSERT INTO module_dependencies (module_id, dependency_id, version_requirement, optional)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (module_id, dependency_id) 
            DO UPDATE SET version_requirement = $3, optional = $4
            "#
        )
        .bind(module_id)
        .bind(dependency_id)
        .bind(version_requirement)
        .bind(optional)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_module_dependency(&self, module_id: &str, dependency_id: &str) -> Result<(), ModuleServiceError> {
        sqlx::query("DELETE FROM module_dependencies WHERE module_id = $1 AND dependency_id = $2")
            .bind(module_id)
            .bind(dependency_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_module_versions(&self, module_id: &str) -> Result<Vec<ModuleVersionRecord>, ModuleServiceError> {
        let records = sqlx::query_as::<_, ModuleVersionRecord>(
            "SELECT * FROM module_versions WHERE module_id = $1 ORDER BY published_at DESC"
        )
        .bind(module_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn create_module_version(&self, version: &ModuleVersionRecord) -> Result<ModuleVersionRecord, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleVersionRecord>(
            r#"
            INSERT INTO module_versions (
                module_id, version, changelog, package_url, package_hash,
                package_size_bytes, manifest_json, security_scan_json,
                performance_metrics_json, compatibility_json, is_stable,
                is_deprecated, deprecation_reason, published_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(&version.module_id)
        .bind(&version.version)
        .bind(&version.changelog)
        .bind(&version.package_url)
        .bind(&version.package_hash)
        .bind(version.package_size_bytes)
        .bind(&version.manifest_json)
        .bind(&version.security_scan_json)
        .bind(&version.performance_metrics_json)
        .bind(&version.compatibility_json)
        .bind(version.is_stable)
        .bind(version.is_deprecated)
        .bind(&version.deprecation_reason)
        .bind(version.published_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_module_stats(&self, module_id: &str) -> Result<ModuleStats, ModuleServiceError> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(DISTINCT mi.tenant_id) as installation_count,
                AVG(mr.rating) as average_rating,
                COUNT(mr.id) as review_count,
                MAX(mi.last_used_at) as last_used_at,
                SUM(mu.usage_count) as total_usage
            FROM modules m
            LEFT JOIN module_installations mi ON m.id = mi.module_id
            LEFT JOIN module_reviews mr ON m.id = mr.module_id
            LEFT JOIN module_usage mu ON m.id = mu.module_id
            WHERE m.id = $1
            GROUP BY m.id
            "#
        )
        .bind(module_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ModuleStats {
            installation_count: row.get::<i64, _>("installation_count") as u64,
            average_rating: row.get::<Option<f64>, _>("average_rating").unwrap_or(0.0) as f32,
            review_count: row.get::<i64, _>("review_count") as u32,
            last_used_at: row.get("last_used_at"),
            total_usage: row.get::<Option<i64>, _>("total_usage").unwrap_or(0) as u64,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub installation_count: u64,
    pub average_rating: f32,
    pub review_count: u32,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_usage: u64,
}