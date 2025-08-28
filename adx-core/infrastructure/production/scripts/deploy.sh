#!/bin/bash

# ADX Core Production Deployment Script
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PRODUCTION_DIR="$PROJECT_ROOT/infrastructure/production"
BACKUP_DIR="/opt/adx-core/backups"
DEPLOY_LOG="/var/log/adx-core-deploy.log"

# Functions
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}" | tee -a "$DEPLOY_LOG"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}" | tee -a "$DEPLOY_LOG"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}" | tee -a "$DEPLOY_LOG"
    exit 1
}

check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if running as root or with sudo
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root or with sudo"
    fi
    
    # Check required commands
    local required_commands=("docker" "docker-compose" "git" "curl" "openssl")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            error "Required command '$cmd' is not installed"
        fi
    done
    
    # Check if .env file exists
    if [[ ! -f "$PRODUCTION_DIR/.env" ]]; then
        error "Production .env file not found. Please copy .env.example to .env and configure it."
    fi
    
    log "Prerequisites check passed"
}

backup_current_deployment() {
    log "Creating backup of current deployment..."
    
    local backup_timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_path="$BACKUP_DIR/deployment_$backup_timestamp"
    
    mkdir -p "$backup_path"
    
    # Backup database
    if docker ps | grep -q "adx-core-postgres-prod"; then
        log "Backing up database..."
        docker exec adx-core-postgres-prod pg_dump -U "$POSTGRES_USER" adx_core_prod > "$backup_path/database.sql"
    fi
    
    # Backup Redis data
    if docker ps | grep -q "adx-core-redis-prod"; then
        log "Backing up Redis data..."
        docker exec adx-core-redis-prod redis-cli --rdb - > "$backup_path/redis.rdb"
    fi
    
    # Backup configuration files
    cp -r "$PRODUCTION_DIR" "$backup_path/config"
    
    log "Backup created at $backup_path"
    echo "$backup_path" > /tmp/adx-core-last-backup
}

build_services() {
    log "Building ADX Core services..."
    
    cd "$PROJECT_ROOT"
    
    # Build Rust services
    log "Building Rust backend services..."
    cargo build --release --workspace
    
    # Build Docker images
    log "Building Docker images..."
    docker-compose -f "$PRODUCTION_DIR/docker-compose.prod.yml" build --no-cache
    
    log "Services built successfully"
}

run_health_checks() {
    log "Running health checks..."
    
    local max_attempts=30
    local attempt=1
    
    # Check each service
    local services=("api-gateway:8080" "auth-service:8081" "user-service:8082" "file-service:8083" "workflow-service:8084" "tenant-service:8085")
    
    for service in "${services[@]}"; do
        local service_name=$(echo "$service" | cut -d':' -f1)
        local port=$(echo "$service" | cut -d':' -f2)
        
        log "Checking health of $service_name..."
        
        attempt=1
        while [[ $attempt -le $max_attempts ]]; do
            if curl -f -s "http://localhost:$port/health" > /dev/null; then
                log "$service_name is healthy"
                break
            fi
            
            if [[ $attempt -eq $max_attempts ]]; then
                error "$service_name failed health check after $max_attempts attempts"
            fi
            
            log "Attempt $attempt/$max_attempts failed, retrying in 10 seconds..."
            sleep 10
            ((attempt++))
        done
    done
    
    # Check Temporal
    log "Checking Temporal health..."
    attempt=1
    while [[ $attempt -le $max_attempts ]]; do
        if docker exec adx-core-temporal-prod tctl --address temporal:7233 cluster health > /dev/null 2>&1; then
            log "Temporal is healthy"
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            error "Temporal failed health check after $max_attempts attempts"
        fi
        
        log "Temporal health check attempt $attempt/$max_attempts failed, retrying in 10 seconds..."
        sleep 10
        ((attempt++))
    done
    
    log "All health checks passed"
}

deploy() {
    log "Starting ADX Core production deployment..."
    
    cd "$PRODUCTION_DIR"
    
    # Stop existing services
    log "Stopping existing services..."
    docker-compose -f docker-compose.prod.yml down --remove-orphans
    
    # Start infrastructure services first
    log "Starting infrastructure services..."
    docker-compose -f docker-compose.prod.yml up -d postgres redis temporal
    
    # Wait for infrastructure to be ready
    log "Waiting for infrastructure services to be ready..."
    sleep 30
    
    # Run database migrations
    log "Running database migrations..."
    cd "$PROJECT_ROOT"
    DATABASE_URL="postgresql://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:5432/adx_core_prod" \
        cargo run --bin migrate
    
    cd "$PRODUCTION_DIR"
    
    # Start application services
    log "Starting application services..."
    docker-compose -f docker-compose.prod.yml up -d
    
    # Wait for services to start
    log "Waiting for services to start..."
    sleep 60
    
    # Run health checks
    run_health_checks
    
    log "Deployment completed successfully"
}

rollback() {
    local backup_path
    if [[ -f /tmp/adx-core-last-backup ]]; then
        backup_path=$(cat /tmp/adx-core-last-backup)
    else
        error "No backup found for rollback"
    fi
    
    log "Rolling back to backup: $backup_path"
    
    cd "$PRODUCTION_DIR"
    
    # Stop current services
    docker-compose -f docker-compose.prod.yml down
    
    # Restore configuration
    cp -r "$backup_path/config/"* "$PRODUCTION_DIR/"
    
    # Restore database
    if [[ -f "$backup_path/database.sql" ]]; then
        log "Restoring database..."
        docker-compose -f docker-compose.prod.yml up -d postgres
        sleep 30
        docker exec -i adx-core-postgres-prod psql -U "$POSTGRES_USER" -d adx_core_prod < "$backup_path/database.sql"
    fi
    
    # Restore Redis
    if [[ -f "$backup_path/redis.rdb" ]]; then
        log "Restoring Redis data..."
        docker-compose -f docker-compose.prod.yml up -d redis
        sleep 10
        docker exec -i adx-core-redis-prod redis-cli --pipe < "$backup_path/redis.rdb"
    fi
    
    # Start all services
    docker-compose -f docker-compose.prod.yml up -d
    
    log "Rollback completed"
}

setup_ssl() {
    log "Setting up SSL certificates..."
    
    local ssl_dir="$PRODUCTION_DIR/nginx/ssl"
    mkdir -p "$ssl_dir"
    
    if [[ ! -f "$ssl_dir/cert.pem" ]] || [[ ! -f "$ssl_dir/key.pem" ]]; then
        warn "SSL certificates not found. Generating self-signed certificates for development."
        warn "For production, please replace with proper SSL certificates."
        
        openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout "$ssl_dir/key.pem" \
            -out "$ssl_dir/cert.pem" \
            -subj "/C=US/ST=State/L=City/O=Organization/CN=your-domain.com"
    fi
    
    log "SSL setup completed"
}

setup_monitoring() {
    log "Setting up monitoring..."
    
    # Create Grafana provisioning directories
    local grafana_dir="$PRODUCTION_DIR/monitoring/grafana"
    mkdir -p "$grafana_dir/provisioning/datasources"
    mkdir -p "$grafana_dir/provisioning/dashboards"
    mkdir -p "$grafana_dir/dashboards"
    
    # Create Grafana datasource configuration
    cat > "$grafana_dir/provisioning/datasources/prometheus.yml" << EOF
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
EOF
    
    # Create dashboard provisioning configuration
    cat > "$grafana_dir/provisioning/dashboards/dashboards.yml" << EOF
apiVersion: 1
providers:
  - name: 'ADX Core Dashboards'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /var/lib/grafana/dashboards
EOF
    
    log "Monitoring setup completed"
}

# Main execution
main() {
    case "${1:-deploy}" in
        "deploy")
            check_prerequisites
            backup_current_deployment
            setup_ssl
            setup_monitoring
            build_services
            deploy
            ;;
        "rollback")
            check_prerequisites
            rollback
            ;;
        "health")
            run_health_checks
            ;;
        "backup")
            check_prerequisites
            backup_current_deployment
            ;;
        *)
            echo "Usage: $0 {deploy|rollback|health|backup}"
            echo "  deploy  - Deploy ADX Core to production"
            echo "  rollback - Rollback to previous deployment"
            echo "  health  - Run health checks on current deployment"
            echo "  backup  - Create backup of current deployment"
            exit 1
            ;;
    esac
}

# Trap errors and provide cleanup
trap 'error "Deployment failed. Check logs at $DEPLOY_LOG"' ERR

# Create log directory
mkdir -p "$(dirname "$DEPLOY_LOG")"

# Run main function
main "$@"