use std::sync::Arc;
use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use sqlx::PgPool;
use adx_shared::{
    config::AppConfig,
    database::DatabasePool,
    middleware::{tenant_context_middleware, auth_middleware},
};
use crate::{
    handlers::FileHandlers,
    repositories::*,
    services::FileService,
    storage::{StorageManager, LocalStorageProvider, LocalConfig},
};

pub struct FileServer {
    config: AppConfig,
    pool: PgPool,
}

impl FileServer {
    pub fn new(config: AppConfig, pool: PgPool) -> Self {
        Self { config, pool }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let port = self.config.server.port + 2; // File service runs on port 8083
        let addr = format!("0.0.0.0:{}", port);

        // Initialize repositories
        let file_repo = Arc::new(PostgresFileRepository::new(self.pool.clone()));
        let permission_repo = Arc::new(PostgresFilePermissionRepository::new(self.pool.clone()));
        let share_repo = Arc::new(PostgresFileShareRepository::new(self.pool.clone()));

        // Initialize storage manager
        let mut storage_manager = StorageManager::new();
        
        // Add local storage provider as default
        let local_config = LocalConfig {
            base_path: self.config.file_storage.local_path.clone().unwrap_or_else(|| "./storage".to_string()),
            url_prefix: format!("http://localhost:{}/files", port),
        };
        storage_manager.add_provider(
            "local".to_string(),
            Box::new(LocalStorageProvider::new(local_config))
        );
        storage_manager.set_default_provider("local".to_string());

        let storage_manager = Arc::new(storage_manager);

        // Initialize services
        let file_service = Arc::new(FileService::new(
            file_repo,
            permission_repo,
            share_repo,
            storage_manager,
        ));

        // Initialize handlers
        let handlers = Arc::new(FileHandlers::new(file_service));

        // Build the application
        let app = self.create_router(handlers);

        tracing::info!("File Service HTTP server starting on {}", addr);

        // Start the server
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    fn create_router(&self, handlers: Arc<FileHandlers>) -> Router {
        Router::new()
            // Health check endpoint (no auth required)
            .route("/health", get(FileHandlers::health_check))
            
            // File management endpoints (auth required)
            .route("/api/v1/files", post(FileHandlers::create_file))
            .route("/api/v1/files", get(FileHandlers::list_files))
            .route("/api/v1/files/:file_id", get(FileHandlers::get_file))
            .route("/api/v1/files/:file_id", put(FileHandlers::update_file))
            .route("/api/v1/files/:file_id", delete(FileHandlers::delete_file))
            
            // File upload/download endpoints
            .route("/api/v1/files/:file_id/upload", post(FileHandlers::upload_file_data))
            .route("/api/v1/files/:file_id/download", get(FileHandlers::download_file))
            
            // File sharing endpoints
            .route("/api/v1/files/:file_id/shares", post(FileHandlers::create_file_share))
            .route("/api/v1/files/:file_id/shares", get(FileHandlers::get_file_shares))
            
            // File permission endpoints
            .route("/api/v1/files/:file_id/permissions", post(FileHandlers::grant_file_permission))
            .route("/api/v1/files/:file_id/permissions", get(FileHandlers::get_file_permissions))
            
            // Public share access endpoint (no auth required)
            .route("/api/v1/shares/:share_token", post(FileHandlers::access_shared_file))
            
            // Apply middleware
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
                    .layer(TimeoutLayer::from_secs(30))
                    .layer(middleware::from_fn(tenant_context_middleware))
                    .layer(middleware::from_fn(auth_middleware))
            )
            .with_state(handlers)
    }
}

pub async fn start_server(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database connection
    let database_pool = DatabasePool::new(&config.database).await?;
    let pool = database_pool.get_pool();

    // Create and run server
    let server = FileServer::new(config, pool.clone());
    server.run().await
}