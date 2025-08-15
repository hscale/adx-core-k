use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};

use crate::config::ModuleServiceConfig;
use crate::error::ModuleServiceError;
use crate::repositories::MarketplaceRepository;
use crate::services::MarketplaceService;
use crate::types::{MarketplaceListing, ModuleSearchRequest, ModuleSearchResponse};

/// Marketplace indexer for keeping module data synchronized
#[async_trait]
pub trait MarketplaceIndexerTrait {
    async fn start_indexing(&self) -> Result<(), ModuleServiceError>;
    async fn index_all_modules(&self) -> Result<IndexingResult, ModuleServiceError>;
    async fn index_module(&self, module_id: &str) -> Result<(), ModuleServiceError>;
    async fn remove_module_from_index(&self, module_id: &str) -> Result<(), ModuleServiceError>;
    async fn update_search_index(&self) -> Result<(), ModuleServiceError>;
    async fn rebuild_index(&self) -> Result<(), ModuleServiceError>;
}

pub struct MarketplaceIndexer {
    config: ModuleServiceConfig,
    marketplace_repo: Arc<MarketplaceRepository>,
    marketplace_service: Arc<MarketplaceService>,
    search_index: Arc<SearchIndex>,
}

impl MarketplaceIndexer {
    pub fn new(
        config: ModuleServiceConfig,
        marketplace_repo: Arc<MarketplaceRepository>,
        marketplace_service: Arc<MarketplaceService>,
    ) -> Self {
        Self {
            config,
            marketplace_repo,
            marketplace_service,
            search_index: Arc::new(SearchIndex::new()),
        }
    }
}

#[async_trait]
impl MarketplaceIndexerTrait for MarketplaceIndexer {
    async fn start_indexing(&self) -> Result<(), ModuleServiceError> {
        info!("Starting marketplace indexer");

        // Initial full index
        self.index_all_modules().await?;

        // Start periodic indexing
        let mut interval = interval(Duration::from_secs(3600)); // Every hour
        
        loop {
            interval.tick().await;
            
            match self.index_all_modules().await {
                Ok(result) => {
                    info!(
                        "Indexing completed: {} modules processed, {} updated, {} errors",
                        result.modules_processed,
                        result.modules_updated,
                        result.errors.len()
                    );
                }
                Err(e) => {
                    error!("Indexing failed: {}", e);
                }
            }
        }
    }

    async fn index_all_modules(&self) -> Result<IndexingResult, ModuleServiceError> {
        info!("Starting full marketplace indexing");

        let mut result = IndexingResult {
            modules_processed: 0,
            modules_updated: 0,
            modules_added: 0,
            modules_removed: 0,
            errors: Vec::new(),
        };

        // Fetch all modules from marketplace API
        let marketplace_modules = self.marketplace_service.fetch_all_modules().await
            .map_err(|e| ModuleServiceError::MarketplaceError(e.to_string()))?;

        // Get existing modules from database
        let existing_modules = self.marketplace_repo.list_all_modules().await?;
        let existing_ids: std::collections::HashSet<String> = existing_modules
            .into_iter()
            .map(|m| m.id)
            .collect();

        let marketplace_ids: std::collections::HashSet<String> = marketplace_modules
            .iter()
            .map(|m| m.id.clone())
            .collect();

        // Process each module from marketplace
        for module in marketplace_modules {
            result.modules_processed += 1;

            match self.process_module_update(&module, existing_ids.contains(&module.id)).await {
                Ok(action) => {
                    match action {
                        IndexAction::Added => result.modules_added += 1,
                        IndexAction::Updated => result.modules_updated += 1,
                        IndexAction::NoChange => {},
                    }
                }
                Err(e) => {
                    error!("Failed to process module {}: {}", module.id, e);
                    result.errors.push(format!("Module {}: {}", module.id, e));
                }
            }
        }

        // Remove modules that are no longer in marketplace
        for existing_id in &existing_ids {
            if !marketplace_ids.contains(existing_id) {
                match self.remove_module_from_index(existing_id).await {
                    Ok(_) => {
                        result.modules_removed += 1;
                        info!("Removed module {} from index", existing_id);
                    }
                    Err(e) => {
                        error!("Failed to remove module {}: {}", existing_id, e);
                        result.errors.push(format!("Remove {}: {}", existing_id, e));
                    }
                }
            }
        }

        // Update search index
        if let Err(e) = self.update_search_index().await {
            error!("Failed to update search index: {}", e);
            result.errors.push(format!("Search index update: {}", e));
        }

        info!("Marketplace indexing completed: {:?}", result);
        Ok(result)
    }

    async fn index_module(&self, module_id: &str) -> Result<(), ModuleServiceError> {
        info!("Indexing single module: {}", module_id);

        // Fetch module from marketplace
        let module = self.marketplace_service.fetch_module(module_id).await
            .map_err(|e| ModuleServiceError::MarketplaceError(e.to_string()))?;

        // Check if exists in database
        let exists = self.marketplace_repo.get_module(module_id).await?.is_some();

        // Process update
        self.process_module_update(&module, exists).await?;

        // Update search index for this module
        self.search_index.index_module(&module).await?;

        info!("Successfully indexed module: {}", module_id);
        Ok(())
    }

    async fn remove_module_from_index(&self, module_id: &str) -> Result<(), ModuleServiceError> {
        // Remove from database
        self.marketplace_repo.delete_module(module_id).await?;

        // Remove from search index
        self.search_index.remove_module(module_id).await?;

        info!("Removed module {} from index", module_id);
        Ok(())
    }

    async fn update_search_index(&self) -> Result<(), ModuleServiceError> {
        info!("Updating search index");

        // Get all modules from database
        let modules = self.marketplace_repo.list_all_modules().await?;

        // Rebuild search index
        self.search_index.rebuild_index(&modules).await?;

        info!("Search index updated with {} modules", modules.len());
        Ok(())
    }

    async fn rebuild_index(&self) -> Result<(), ModuleServiceError> {
        info!("Rebuilding entire marketplace index");

        // Clear existing data
        self.marketplace_repo.clear_all_modules().await?;
        self.search_index.clear_index().await?;

        // Perform full indexing
        self.index_all_modules().await?;

        info!("Marketplace index rebuild completed");
        Ok(())
    }
}

impl MarketplaceIndexer {
    async fn process_module_update(&self, module: &MarketplaceListing, exists: bool) -> Result<IndexAction, ModuleServiceError> {
        if exists {
            // Check if update is needed
            let existing = self.marketplace_repo.get_module(&module.id).await?
                .ok_or_else(|| ModuleServiceError::ModuleNotFound(module.id.clone()))?;

            if existing.updated_at < module.last_updated {
                // Update existing module
                self.marketplace_repo.update_module(module).await?;
                self.search_index.index_module(module).await?;
                return Ok(IndexAction::Updated);
            } else {
                return Ok(IndexAction::NoChange);
            }
        } else {
            // Add new module
            self.marketplace_repo.create_module(module).await?;
            self.search_index.index_module(module).await?;
            return Ok(IndexAction::Added);
        }
    }
}

// Search index implementation
pub struct SearchIndex {
    // In a real implementation, this would use Elasticsearch, Solr, or similar
    modules: tokio::sync::RwLock<HashMap<String, IndexedModule>>,
    text_index: tokio::sync::RwLock<HashMap<String, Vec<String>>>, // Simple text search
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            modules: tokio::sync::RwLock::new(HashMap::new()),
            text_index: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn index_module(&self, module: &MarketplaceListing) -> Result<(), ModuleServiceError> {
        let indexed_module = IndexedModule::from_listing(module);
        
        // Add to modules index
        let mut modules = self.modules.write().await;
        modules.insert(module.id.clone(), indexed_module);

        // Add to text index
        let mut text_index = self.text_index.write().await;
        let search_terms = self.extract_search_terms(module);
        
        for term in search_terms {
            text_index.entry(term.to_lowercase())
                .or_insert_with(Vec::new)
                .push(module.id.clone());
        }

        Ok(())
    }

    pub async fn remove_module(&self, module_id: &str) -> Result<(), ModuleServiceError> {
        // Remove from modules index
        let mut modules = self.modules.write().await;
        modules.remove(module_id);

        // Remove from text index
        let mut text_index = self.text_index.write().await;
        text_index.retain(|_, module_ids| {
            module_ids.retain(|id| id != module_id);
            !module_ids.is_empty()
        });

        Ok(())
    }

    pub async fn search(&self, request: &ModuleSearchRequest) -> Result<ModuleSearchResponse, ModuleServiceError> {
        let modules = self.modules.read().await;
        let text_index = self.text_index.read().await;

        let mut matching_ids = std::collections::HashSet::new();

        // Text search
        if let Some(query) = &request.query {
            let query_terms: Vec<&str> = query.to_lowercase().split_whitespace().collect();
            
            for term in query_terms {
                if let Some(module_ids) = text_index.get(term) {
                    for id in module_ids {
                        matching_ids.insert(id.clone());
                    }
                }
            }
        } else {
            // No query - include all modules
            matching_ids.extend(modules.keys().cloned());
        }

        // Apply filters
        let mut filtered_modules: Vec<&IndexedModule> = modules
            .values()
            .filter(|module| matching_ids.contains(&module.id))
            .collect();

        // Category filter
        if let Some(category) = &request.category {
            filtered_modules.retain(|module| &module.category == category);
        }

        // Author filter
        if let Some(author) = &request.author {
            filtered_modules.retain(|module| module.author_name.to_lowercase().contains(&author.to_lowercase()));
        }

        // Rating filter
        if let Some(min_rating) = request.rating_min {
            filtered_modules.retain(|module| module.rating >= min_rating);
        }

        // Sort results
        self.sort_modules(&mut filtered_modules, &request.sort_by, &request.sort_order);

        // Apply pagination
        let total_count = filtered_modules.len() as u64;
        let start_index = ((request.page.saturating_sub(1)) * request.page_size) as usize;
        let end_index = std::cmp::min(start_index + request.page_size as usize, filtered_modules.len());
        
        let page_modules = if start_index < filtered_modules.len() {
            &filtered_modules[start_index..end_index]
        } else {
            &[]
        };

        // Convert to marketplace listings
        let listings: Vec<MarketplaceListing> = page_modules
            .iter()
            .map(|indexed| indexed.to_listing())
            .collect();

        let total_pages = ((total_count + request.page_size as u64 - 1) / request.page_size as u64) as u32;

        Ok(ModuleSearchResponse {
            modules: listings,
            total_count,
            page: request.page,
            page_size: request.page_size,
            total_pages,
            facets: self.generate_facets(&filtered_modules).await,
        })
    }

    pub async fn rebuild_index(&self, modules: &[MarketplaceListing]) -> Result<(), ModuleServiceError> {
        // Clear existing index
        self.clear_index().await?;

        // Index all modules
        for module in modules {
            self.index_module(module).await?;
        }

        Ok(())
    }

    pub async fn clear_index(&self) -> Result<(), ModuleServiceError> {
        let mut modules = self.modules.write().await;
        let mut text_index = self.text_index.write().await;
        
        modules.clear();
        text_index.clear();

        Ok(())
    }

    fn extract_search_terms(&self, module: &MarketplaceListing) -> Vec<String> {
        let mut terms = Vec::new();
        
        // Add name words
        terms.extend(module.name.split_whitespace().map(|s| s.to_string()));
        
        // Add description words
        terms.extend(module.description.split_whitespace().map(|s| s.to_string()));
        
        // Add author name
        terms.extend(module.author.name.split_whitespace().map(|s| s.to_string()));
        
        // Add tags
        terms.extend(module.tags.clone());
        
        // Add category
        terms.push(format!("{:?}", module.category));

        terms
    }

    fn sort_modules(
        &self,
        modules: &mut Vec<&IndexedModule>,
        sort_by: &Option<crate::types::SortBy>,
        sort_order: &Option<crate::types::SortOrder>,
    ) {
        use crate::types::{SortBy, SortOrder};

        let ascending = matches!(sort_order, Some(SortOrder::Asc));

        match sort_by {
            Some(SortBy::Name) => {
                modules.sort_by(|a, b| {
                    if ascending {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
            }
            Some(SortBy::Rating) => {
                modules.sort_by(|a, b| {
                    if ascending {
                        a.rating.partial_cmp(&b.rating).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            Some(SortBy::Downloads) => {
                modules.sort_by(|a, b| {
                    if ascending {
                        a.downloads.cmp(&b.downloads)
                    } else {
                        b.downloads.cmp(&a.downloads)
                    }
                });
            }
            Some(SortBy::Updated) => {
                modules.sort_by(|a, b| {
                    if ascending {
                        a.last_updated.cmp(&b.last_updated)
                    } else {
                        b.last_updated.cmp(&a.last_updated)
                    }
                });
            }
            _ => {
                // Default sort by relevance (downloads * rating)
                modules.sort_by(|a, b| {
                    let score_a = a.downloads as f32 * a.rating;
                    let score_b = b.downloads as f32 * b.rating;
                    
                    if ascending {
                        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
        }
    }

    async fn generate_facets(&self, modules: &[&IndexedModule]) -> crate::types::SearchFacets {
        let mut categories = HashMap::new();
        let mut authors = HashMap::new();
        let mut tags = HashMap::new();
        let mut price_ranges = HashMap::new();
        let mut ratings = HashMap::new();

        for module in modules {
            // Count categories
            let category_key = format!("{:?}", module.category);
            *categories.entry(category_key).or_insert(0) += 1;

            // Count authors
            *authors.entry(module.author_name.clone()).or_insert(0) += 1;

            // Count tags
            for tag in &module.tags {
                *tags.entry(tag.clone()).or_insert(0) += 1;
            }

            // Count price ranges
            let price_range = if module.price.is_none() {
                "Free".to_string()
            } else {
                "Paid".to_string()
            };
            *price_ranges.entry(price_range).or_insert(0) += 1;

            // Count ratings
            let rating_range = format!("{:.0}-{:.0}", module.rating.floor(), module.rating.floor() + 1.0);
            *ratings.entry(rating_range).or_insert(0) += 1;
        }

        crate::types::SearchFacets {
            categories,
            authors,
            tags,
            price_ranges,
            ratings,
        }
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct IndexingResult {
    pub modules_processed: u32,
    pub modules_updated: u32,
    pub modules_added: u32,
    pub modules_removed: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum IndexAction {
    Added,
    Updated,
    NoChange,
}

#[derive(Debug, Clone)]
pub struct IndexedModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_name: String,
    pub category: crate::types::ModuleCategory,
    pub rating: f32,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub price: Option<crate::types::ModulePrice>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl IndexedModule {
    pub fn from_listing(listing: &MarketplaceListing) -> Self {
        Self {
            id: listing.id.clone(),
            name: listing.name.clone(),
            description: listing.description.clone(),
            author_name: listing.author.name.clone(),
            category: listing.category.clone(),
            rating: listing.rating,
            downloads: listing.downloads,
            tags: listing.tags.clone(),
            price: listing.price.clone(),
            last_updated: listing.last_updated,
        }
    }

    pub fn to_listing(&self) -> MarketplaceListing {
        // This would create a full MarketplaceListing from the indexed data
        // For brevity, I'll create a minimal version
        MarketplaceListing {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            long_description: "".to_string(),
            version: "1.0.0".to_string(),
            author: crate::types::ModuleAuthor {
                name: self.author_name.clone(),
                email: "".to_string(),
                website: None,
                organization: None,
            },
            category: self.category.clone(),
            subcategory: None,
            price: self.price.clone(),
            rating: self.rating,
            review_count: 0,
            downloads: self.downloads,
            active_installations: 0,
            screenshots: vec![],
            demo_url: None,
            documentation_url: "".to_string(),
            support_url: "".to_string(),
            tags: self.tags.clone(),
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
            last_updated: self.last_updated,
            changelog: vec![],
        }
    }
}

// Entry point for the indexer service
pub async fn start_indexer(config: ModuleServiceConfig) -> Result<(), ModuleServiceError> {
    info!("Starting marketplace indexer service");

    // Initialize dependencies (these would be properly injected)
    let marketplace_repo = Arc::new(MarketplaceRepository::new(/* pool */));
    let marketplace_service = Arc::new(MarketplaceService::new(/* dependencies */));

    let indexer = MarketplaceIndexer::new(config, marketplace_repo, marketplace_service);

    // Start indexing
    indexer.start_indexing().await
}