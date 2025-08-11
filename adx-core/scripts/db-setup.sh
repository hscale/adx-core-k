#!/bin/bash

# ADX Core Database Setup Script
# This script handles database migrations, seeding, and health checks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DATABASE_URL=${DATABASE_URL:-"postgresql://postgres:postgres@localhost:5432/adx_core"}
ENVIRONMENT=${ENVIRONMENT:-"development"}
COMMAND=""

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "ADX Core Database Setup Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  migrate         Run database migrations"
    echo "  seed            Seed database with sample data"
    echo "  health          Check database health"
    echo "  setup           Run migrations and seed data (full setup)"
    echo "  clean           Clean test data from database"
    echo "  stats           Show database statistics"
    echo "  validate        Validate database integrity"
    echo "  create-tenant   Create a new tenant"
    echo "  health-check    Run enhanced health check"
    echo "  analyze-indexes Analyze index performance"
    echo "  monitor-pool    Monitor connection pool"
    echo ""
    echo "Options:"
    echo "  --database-url URL    Database connection URL"
    echo "  --environment ENV     Environment (development, test, production)"
    echo "  --tenant-name NAME    Tenant name (for create-tenant command)"
    echo "  --admin-email EMAIL   Admin email (for create-tenant command)"
    echo "  --help               Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  DATABASE_URL          Database connection URL"
    echo "  ENVIRONMENT           Environment (development, test, production)"
    echo ""
    echo "Examples:"
    echo "  $0 setup                                    # Full database setup"
    echo "  $0 migrate --database-url postgresql://... # Run migrations with custom URL"
    echo "  $0 seed --environment test                  # Seed test data"
    echo "  $0 create-tenant --tenant-name \"Acme Corp\" --admin-email admin@acme.com"
}

# Function to check if database is accessible
check_database_connection() {
    print_status "Checking database connection..."
    
    if ! command -v psql &> /dev/null; then
        print_warning "psql not found, skipping connection test"
        return 0
    fi
    
    if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
        print_success "Database connection successful"
        return 0
    else
        print_error "Cannot connect to database: $DATABASE_URL"
        print_error "Please ensure the database is running and accessible"
        return 1
    fi
}

# Function to build the db-manager binary
build_db_manager() {
    print_status "Building database manager..."
    
    cd "$(dirname "$0")/.."
    
    if cargo build --bin db-manager --release; then
        print_success "Database manager built successfully"
        return 0
    else
        print_error "Failed to build database manager"
        return 1
    fi
}

# Function to run db-manager command
run_db_manager() {
    local cmd="$1"
    shift
    
    local db_manager_path="$(dirname "$0")/../target/release/db-manager"
    
    if [ ! -f "$db_manager_path" ]; then
        print_status "Database manager not found, building..."
        build_db_manager || return 1
    fi
    
    print_status "Running: $cmd"
    
    if "$db_manager_path" "$cmd" --database-url "$DATABASE_URL" "$@"; then
        print_success "Command completed: $cmd"
        return 0
    else
        print_error "Command failed: $cmd"
        return 1
    fi
}

# Function to wait for database to be ready
wait_for_database() {
    print_status "Waiting for database to be ready..."
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if check_database_connection &> /dev/null; then
            print_success "Database is ready"
            return 0
        fi
        
        print_status "Attempt $attempt/$max_attempts - Database not ready, waiting..."
        sleep 2
        ((attempt++))
    done
    
    print_error "Database did not become ready within $max_attempts attempts"
    return 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        migrate|seed|health|setup|clean|stats|validate|create-tenant|health-check|analyze-indexes|monitor-pool)
            COMMAND="$1"
            shift
            ;;
        --database-url)
            DATABASE_URL="$2"
            shift 2
            ;;
        --environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --tenant-name)
            TENANT_NAME="$2"
            shift 2
            ;;
        --admin-email)
            ADMIN_EMAIL="$2"
            shift 2
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Check if command was provided
if [ -z "$COMMAND" ]; then
    print_error "No command specified"
    show_usage
    exit 1
fi

# Export environment variables
export DATABASE_URL
export ENVIRONMENT

print_status "ADX Core Database Setup"
print_status "Database URL: $DATABASE_URL"
print_status "Environment: $ENVIRONMENT"
print_status "Command: $COMMAND"
echo ""

# Execute the requested command
case $COMMAND in
    migrate)
        wait_for_database || exit 1
        run_db_manager migrate || exit 1
        ;;
    seed)
        wait_for_database || exit 1
        run_db_manager seed --environment "$ENVIRONMENT" || exit 1
        ;;
    health)
        run_db_manager health || exit 1
        ;;
    setup)
        print_status "Running full database setup..."
        wait_for_database || exit 1
        run_db_manager migrate || exit 1
        run_db_manager seed --environment "$ENVIRONMENT" || exit 1
        run_db_manager health || exit 1
        run_db_manager stats || exit 1
        print_success "Database setup completed successfully!"
        ;;
    clean)
        run_db_manager clean || exit 1
        ;;
    stats)
        run_db_manager stats || exit 1
        ;;
    validate)
        run_db_manager validate || exit 1
        ;;
    create-tenant)
        if [ -z "$TENANT_NAME" ] || [ -z "$ADMIN_EMAIL" ]; then
            print_error "Both --tenant-name and --admin-email are required for create-tenant command"
            exit 1
        fi
        run_db_manager create-tenant --name "$TENANT_NAME" --admin-email "$ADMIN_EMAIL" || exit 1
        ;;
    health-check)
        run_db_manager health-check || exit 1
        ;;
    analyze-indexes)
        run_db_manager analyze-indexes || exit 1
        ;;
    monitor-pool)
        run_db_manager monitor-pool || exit 1
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac

print_success "All operations completed successfully!"