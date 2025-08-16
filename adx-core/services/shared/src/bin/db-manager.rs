use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, error, Level};
use tracing_subscriber;
use adx_shared::database::{create_database_pool, run_migrations, check_database_health, seeder::DatabaseSeeder};

#[derive(Parser)]
#[command(name = "db-manager")]
#[command(about = "ADX Core Database Management Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long)]
    database_url: Option<String>,
    
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run database migrations
    Migrate,
    /// Seed database with development or test data
    Seed {
        #[arg(long, default_value = "development")]
        environment: String,
    },
    /// Check database health
    Health,
    /// Validate database schema and data integrity
    Validate,
    /// Show database statistics
    Stats,
    /// Clean test data from database
    Clean,
    /// Create a new tenant with sample data
    CreateTenant {
        #[arg(long)]
        name: String,
        #[arg(long)]
        admin_email: String,
    },
    /// Run enhanced database health check
    HealthCheck,
    /// Analyze index performance
    AnalyzeIndexes,
    /// Monitor connection pool
    MonitorPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let log_level = match cli.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    // Get database URL
    let database_url = cli.database_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| {
            error!("DATABASE_URL must be provided via --database-url or environment variable");
            std::process::exit(1);
        });

    // Create database connection pool
    let pool = create_database_pool(&database_url, 10).await?;
    
    match cli.command {
        Commands::Migrate => {
            info!("Running database migrations...");
            run_migrations(&*pool).await?;
            info!("Migrations completed successfully");
        }
        
        Commands::Seed { environment } => {
            info!("Seeding database with {} data...", environment);
            env::set_var("ENVIRONMENT", &environment);
            let seeder = DatabaseSeeder::new((*pool).clone());
            seeder.run_all_seeds().await?;
            
            let stats = seeder.get_seeding_stats().await?;
            info!("{}", stats);
        }
        
        Commands::Health => {
            info!("Checking database health...");
            match check_database_health(&*pool).await {
                Ok(_) => info!("Database is healthy"),
                Err(e) => {
                    error!("Database health check failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Validate => {
            info!("Validating database integrity...");
            let seeder = DatabaseSeeder::new((*pool).clone());
            let issues = seeder.validate_seeded_data().await?;
            
            if issues.is_empty() {
                info!("No validation issues found");
            } else {
                info!("Found {} validation issues:", issues.len());
                for issue in issues {
                    match issue.severity.as_str() {
                        "high" => error!("[{}] {}: {}", issue.severity.to_uppercase(), issue.issue_type, issue.description),
                        "medium" => tracing::warn!("[{}] {}: {}", issue.severity.to_uppercase(), issue.issue_type, issue.description),
                        _ => info!("[{}] {}: {}", issue.severity.to_uppercase(), issue.issue_type, issue.description),
                    }
                }
            }
        }
        
        Commands::Stats => {
            info!("Gathering database statistics...");
            let seeder = DatabaseSeeder::new((*pool).clone());
            let stats = seeder.get_seeding_stats().await?;
            println!("{}", stats);
        }
        
        Commands::Clean => {
            info!("Cleaning test data from database...");
            env::set_var("ENVIRONMENT", "test");
            let seeder = DatabaseSeeder::new((*pool).clone());
            seeder.run_test_seeds().await?; // This includes cleanup
            info!("Test data cleaned successfully");
        }
        
        Commands::CreateTenant { name, admin_email } => {
            info!("Creating tenant: {} with admin: {}", name, admin_email);
            let seeder = DatabaseSeeder::new((*pool).clone());
            let tenant_id = seeder.seed_tenant_data(&name, &admin_email).await?;
            info!("Tenant created successfully with ID: {}", tenant_id);
        }
        
        Commands::HealthCheck => {
            info!("Running enhanced database health check...");
            
            let health_results: Vec<(String, String, serde_json::Value, i32)> = sqlx::query_as(
                "SELECT check_name, status, details, response_time_ms FROM enhanced_database_health_check()"
            )
            .fetch_all(&*pool)
            .await?;
            
            println!("\n=== Database Health Check Results ===");
            for (check_name, status, details, response_time) in health_results {
                let status_icon = match status.as_str() {
                    "healthy" => "✅",
                    "warning" => "⚠️",
                    "critical" => "❌",
                    _ => "❓",
                };
                
                println!("{} {} ({}ms)", status_icon, check_name, response_time);
                if let Ok(details_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(details) {
                    for (key, value) in details_obj {
                        println!("   {}: {}", key, value);
                    }
                }
                println!();
            }
        }
        
        Commands::AnalyzeIndexes => {
            info!("Analyzing index performance...");
            
            let index_results: Vec<(String, String, String, i64, i64, i64, String)> = sqlx::query_as(
                "SELECT table_name, index_name, index_size, index_scans, tuples_read, tuples_fetched, recommendation FROM analyze_index_performance()"
            )
            .fetch_all(&*pool)
            .await?;
            
            println!("\n=== Index Performance Analysis ===");
            println!("{:<30} {:<25} {:<10} {:<10} {:<12} {:<12} {}", 
                     "Table", "Index", "Size", "Scans", "Tuples Read", "Tuples Fetched", "Recommendation");
            println!("{}", "-".repeat(120));
            
            for (table_name, index_name, index_size, index_scans, tuples_read, tuples_fetched, recommendation) in index_results {
                println!("{:<30} {:<25} {:<10} {:<10} {:<12} {:<12} {}", 
                         table_name, index_name, index_size, index_scans, tuples_read, tuples_fetched, recommendation);
            }
        }
        
        Commands::MonitorPool => {
            info!("Monitoring connection pool...");
            
            // Update connection pool stats
            sqlx::query("SELECT monitor_connection_pool()")
                .execute(&*pool)
                .await?;
            
            let pool_stats: Vec<(String, i32, i32, i32, i32, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
                "SELECT pool_name, active_connections, idle_connections, max_connections, total_connections, recorded_at 
                 FROM connection_pool_stats 
                 ORDER BY recorded_at DESC 
                 LIMIT 10"
            )
            .fetch_all(&*pool)
            .await?;
            
            println!("\n=== Connection Pool Statistics ===");
            println!("{:<15} {:<8} {:<8} {:<8} {:<8} {}", 
                     "Pool", "Active", "Idle", "Max", "Total", "Recorded At");
            println!("{}", "-".repeat(70));
            
            for (pool_name, active, idle, max_conn, total, recorded_at) in pool_stats {
                println!("{:<15} {:<8} {:<8} {:<8} {:<8} {}", 
                         pool_name, active, idle, max_conn, total, recorded_at.format("%Y-%m-%d %H:%M:%S"));
            }
        }
    }
    
    Ok(())
}