#!/bin/bash

# ADX Core Disaster Recovery Script
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRODUCTION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKUP_DIR="/opt/adx-core/backups"
RECOVERY_LOG="/var/log/adx-core-recovery.log"

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}" | tee -a "$RECOVERY_LOG"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}" | tee -a "$RECOVERY_LOG"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}" | tee -a "$RECOVERY_LOG"
    exit 1
}

list_backups() {
    log "Available backups:"
    
    if [[ ! -d "$BACKUP_DIR" ]]; then
        error "Backup directory $BACKUP_DIR does not exist"
    fi
    
    local backups=($(ls -1t "$BACKUP_DIR" | grep -E "(deployment_|automated_)" | head -20))
    
    if [[ ${#backups[@]} -eq 0 ]]; then
        error "No backups found in $BACKUP_DIR"
    fi
    
    for i in "${!backups[@]}"; do
        local backup="${backups[$i]}"
        local backup_path="$BACKUP_DIR/$backup"
        local backup_date=$(stat -c %y "$backup_path" | cut -d' ' -f1)
        local backup_size=$(du -sh "$backup_path" | cut -f1)
        
        echo "  $((i+1)). $backup (Date: $backup_date, Size: $backup_size)"
    done
}

select_backup() {
    list_backups
    
    echo -n "Select backup number to restore (1-20): "
    read -r backup_number
    
    if ! [[ "$backup_number" =~ ^[0-9]+$ ]] || [[ "$backup_number" -lt 1 ]] || [[ "$backup_number" -gt 20 ]]; then
        error "Invalid backup number"
    fi
    
    local backups=($(ls -1t "$BACKUP_DIR" | grep -E "(deployment_|automated_)" | head -20))
    local selected_backup="${backups[$((backup_number-1))]}"
    
    if [[ -z "$selected_backup" ]]; then
        error "Backup not found"
    fi
    
    echo "$BACKUP_DIR/$selected_backup"
}

verify_backup() {
    local backup_path="$1"
    
    log "Verifying backup integrity: $backup_path"
    
    # Check if backup directory exists
    if [[ ! -d "$backup_path" ]]; then
        error "Backup directory does not exist: $backup_path"
    fi
    
    # Check for required backup files
    local required_files=("database.sql" "config")
    for file in "${required_files[@]}"; do
        if [[ ! -e "$backup_path/$file" ]] && [[ ! -e "$backup_path/${file}.gz" ]]; then
            error "Required backup file missing: $file"
        fi
    done
    
    # Verify database backup
    if [[ -f "$backup_path/database.sql" ]]; then
        if ! head -n 1 "$backup_path/database.sql" | grep -q "PostgreSQL database dump"; then
            error "Database backup appears to be corrupted"
        fi
    elif [[ -f "$backup_path/database.sql.gz" ]]; then
        if ! zcat "$backup_path/database.sql.gz" | head -n 1 | grep -q "PostgreSQL database dump"; then
            error "Database backup appears to be corrupted"
        fi
    fi
    
    log "Backup verification passed"
}

stop_services() {
    log "Stopping ADX Core services..."
    
    cd "$PRODUCTION_DIR"
    
    # Stop all services gracefully
    docker-compose -f docker-compose.prod.yml down --timeout 30
    
    # Force stop if any containers are still running
    local running_containers=$(docker ps -q --filter "name=adx-core")
    if [[ -n "$running_containers" ]]; then
        warn "Force stopping remaining containers..."
        docker stop $running_containers
    fi
    
    log "Services stopped"
}

restore_database() {
    local backup_path="$1"
    
    log "Restoring database from backup..."
    
    # Start only PostgreSQL
    cd "$PRODUCTION_DIR"
    docker-compose -f docker-compose.prod.yml up -d postgres
    
    # Wait for PostgreSQL to be ready
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if docker exec adx-core-postgres-prod pg_isready -U "$POSTGRES_USER" > /dev/null 2>&1; then
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            error "PostgreSQL failed to start after $max_attempts attempts"
        fi
        
        log "Waiting for PostgreSQL to be ready... (attempt $attempt/$max_attempts)"
        sleep 5
        ((attempt++))
    done
    
    # Drop and recreate database
    log "Recreating database..."
    docker exec adx-core-postgres-prod psql -U "$POSTGRES_USER" -c "DROP DATABASE IF EXISTS adx_core_prod;"
    docker exec adx-core-postgres-prod psql -U "$POSTGRES_USER" -c "CREATE DATABASE adx_core_prod;"
    
    # Restore database
    if [[ -f "$backup_path/database.sql" ]]; then
        log "Restoring from uncompressed backup..."
        docker exec -i adx-core-postgres-prod psql -U "$POSTGRES_USER" -d adx_core_prod < "$backup_path/database.sql"
    elif [[ -f "$backup_path/database.sql.gz" ]]; then
        log "Restoring from compressed backup..."
        zcat "$backup_path/database.sql.gz" | docker exec -i adx-core-postgres-prod psql -U "$POSTGRES_USER" -d adx_core_prod
    else
        error "No database backup found"
    fi
    
    log "Database restored successfully"
}

restore_redis() {
    local backup_path="$1"
    
    if [[ ! -f "$backup_path/redis.rdb" ]] && [[ ! -f "$backup_path/redis.rdb.gz" ]]; then
        warn "No Redis backup found, skipping Redis restore"
        return
    fi
    
    log "Restoring Redis from backup..."
    
    # Start Redis
    cd "$PRODUCTION_DIR"
    docker-compose -f docker-compose.prod.yml up -d redis
    
    # Wait for Redis to be ready
    sleep 10
    
    # Clear existing data
    docker exec adx-core-redis-prod redis-cli FLUSHALL
    
    # Restore Redis data
    if [[ -f "$backup_path/redis.rdb" ]]; then
        log "Restoring from uncompressed Redis backup..."
        docker cp "$backup_path/redis.rdb" adx-core-redis-prod:/data/dump.rdb
    elif [[ -f "$backup_path/redis.rdb.gz" ]]; then
        log "Restoring from compressed Redis backup..."
        zcat "$backup_path/redis.rdb.gz" > /tmp/redis_restore.rdb
        docker cp /tmp/redis_restore.rdb adx-core-redis-prod:/data/dump.rdb
        rm /tmp/redis_restore.rdb
    fi
    
    # Restart Redis to load the data
    docker-compose -f docker-compose.prod.yml restart redis
    
    log "Redis restored successfully"
}

restore_configuration() {
    local backup_path="$1"
    
    log "Restoring configuration from backup..."
    
    # Backup current configuration
    local current_config_backup="/tmp/adx-core-config-$(date +%Y%m%d_%H%M%S)"
    cp -r "$PRODUCTION_DIR" "$current_config_backup"
    log "Current configuration backed up to: $current_config_backup"
    
    # Restore configuration
    if [[ -d "$backup_path/config" ]]; then
        log "Restoring from uncompressed config backup..."
        cp -r "$backup_path/config/"* "$PRODUCTION_DIR/"
    elif [[ -f "$backup_path/config.tar.gz" ]]; then
        log "Restoring from compressed config backup..."
        tar -xzf "$backup_path/config.tar.gz" -C "$PRODUCTION_DIR"
    else
        error "No configuration backup found"
    fi
    
    log "Configuration restored successfully"
}

start_services() {
    log "Starting ADX Core services..."
    
    cd "$PRODUCTION_DIR"
    
    # Start infrastructure services first
    log "Starting infrastructure services..."
    docker-compose -f docker-compose.prod.yml up -d postgres redis temporal
    
    # Wait for infrastructure to be ready
    log "Waiting for infrastructure services..."
    sleep 60
    
    # Start application services
    log "Starting application services..."
    docker-compose -f docker-compose.prod.yml up -d
    
    log "Services started"
}

verify_recovery() {
    log "Verifying recovery..."
    
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
            
            log "Health check attempt $attempt/$max_attempts failed, retrying in 10 seconds..."
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
    
    log "Recovery verification completed successfully"
}

full_recovery() {
    local backup_path
    
    if [[ -n "$1" ]]; then
        backup_path="$1"
    else
        backup_path=$(select_backup)
    fi
    
    log "Starting full disaster recovery from: $backup_path"
    
    # Verify backup integrity
    verify_backup "$backup_path"
    
    # Confirm recovery
    echo -n "Are you sure you want to restore from this backup? This will overwrite all current data. (yes/no): "
    read -r confirmation
    
    if [[ "$confirmation" != "yes" ]]; then
        log "Recovery cancelled by user"
        exit 0
    fi
    
    # Stop services
    stop_services
    
    # Restore components
    restore_configuration "$backup_path"
    restore_database "$backup_path"
    restore_redis "$backup_path"
    
    # Start services
    start_services
    
    # Wait for services to stabilize
    log "Waiting for services to stabilize..."
    sleep 120
    
    # Verify recovery
    verify_recovery
    
    log "Disaster recovery completed successfully!"
    log "System has been restored from backup: $backup_path"
}

partial_recovery() {
    local component="$1"
    local backup_path
    
    if [[ -n "$2" ]]; then
        backup_path="$2"
    else
        backup_path=$(select_backup)
    fi
    
    log "Starting partial recovery of $component from: $backup_path"
    
    verify_backup "$backup_path"
    
    case "$component" in
        "database")
            echo -n "Are you sure you want to restore the database? This will overwrite all current data. (yes/no): "
            read -r confirmation
            if [[ "$confirmation" == "yes" ]]; then
                restore_database "$backup_path"
                log "Database recovery completed"
            else
                log "Database recovery cancelled"
            fi
            ;;
        "redis")
            echo -n "Are you sure you want to restore Redis data? This will overwrite all current cache data. (yes/no): "
            read -r confirmation
            if [[ "$confirmation" == "yes" ]]; then
                restore_redis "$backup_path"
                log "Redis recovery completed"
            else
                log "Redis recovery cancelled"
            fi
            ;;
        "config")
            echo -n "Are you sure you want to restore configuration? This will overwrite current configuration. (yes/no): "
            read -r confirmation
            if [[ "$confirmation" == "yes" ]]; then
                restore_configuration "$backup_path"
                log "Configuration recovery completed"
            else
                log "Configuration recovery cancelled"
            fi
            ;;
        *)
            error "Unknown component: $component. Valid options: database, redis, config"
            ;;
    esac
}

create_emergency_backup() {
    log "Creating emergency backup before recovery..."
    
    local emergency_backup_path="$BACKUP_DIR/emergency_$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$emergency_backup_path"
    
    # Backup database if running
    if docker ps | grep -q "adx-core-postgres-prod"; then
        log "Backing up current database..."
        docker exec adx-core-postgres-prod pg_dump -U "$POSTGRES_USER" adx_core_prod > "$emergency_backup_path/database.sql"
    fi
    
    # Backup Redis if running
    if docker ps | grep -q "adx-core-redis-prod"; then
        log "Backing up current Redis data..."
        docker exec adx-core-redis-prod redis-cli --rdb - > "$emergency_backup_path/redis.rdb"
    fi
    
    # Backup current configuration
    cp -r "$PRODUCTION_DIR" "$emergency_backup_path/config"
    
    log "Emergency backup created at: $emergency_backup_path"
}

# Main execution
main() {
    case "${1:-help}" in
        "full")
            create_emergency_backup
            full_recovery "$2"
            ;;
        "partial")
            if [[ -z "$2" ]]; then
                error "Component required for partial recovery. Usage: $0 partial {database|redis|config} [backup_path]"
            fi
            create_emergency_backup
            partial_recovery "$2" "$3"
            ;;
        "list")
            list_backups
            ;;
        "verify")
            if [[ -z "$2" ]]; then
                backup_path=$(select_backup)
            else
                backup_path="$2"
            fi
            verify_backup "$backup_path"
            ;;
        "help"|*)
            echo "ADX Core Disaster Recovery Script"
            echo ""
            echo "Usage: $0 {full|partial|list|verify|help}"
            echo ""
            echo "Commands:"
            echo "  full [backup_path]              - Full system recovery from backup"
            echo "  partial {component} [backup_path] - Partial recovery of specific component"
            echo "    Components: database, redis, config"
            echo "  list                            - List available backups"
            echo "  verify [backup_path]            - Verify backup integrity"
            echo "  help                            - Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 full                         - Interactive full recovery"
            echo "  $0 partial database             - Interactive database recovery"
            echo "  $0 full /opt/adx-core/backups/deployment_20240115_120000"
            echo "  $0 verify /opt/adx-core/backups/automated_20240115_020000"
            exit 1
            ;;
    esac
}

# Create log directory
mkdir -p "$(dirname "$RECOVERY_LOG")"

# Run main function
main "$@"