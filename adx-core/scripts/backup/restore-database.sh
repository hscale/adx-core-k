#!/bin/bash

# ADX Core Database Restore Script
# This script restores encrypted database backups

set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/backups/database}"
POSTGRES_HOST="${POSTGRES_HOST:-postgresql}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-adx_core}"
ENCRYPTION_KEY="${BACKUP_ENCRYPTION_KEY}"
S3_BUCKET="${BACKUP_S3_BUCKET}"
S3_PREFIX="${BACKUP_S3_PREFIX:-database-backups}"

# Logging
LOG_FILE="/var/log/adx-core/restore-database.log"
exec 1> >(tee -a "$LOG_FILE")
exec 2>&1

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to show usage
usage() {
    echo "Usage: $0 [OPTIONS] BACKUP_FILE"
    echo "Options:"
    echo "  -s, --from-s3     Download backup from S3"
    echo "  -f, --force       Force restore without confirmation"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 adx_core_backup_20240115_120000.sql.enc.gz"
    echo "  $0 -s adx_core_backup_20240115_120000.sql.enc.gz"
    exit 1
}

# Parse command line arguments
FROM_S3=false
FORCE=false
BACKUP_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--from-s3)
            FROM_S3=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [ -z "$BACKUP_FILE" ]; then
                BACKUP_FILE="$1"
            else
                echo "ERROR: Multiple backup files specified"
                usage
            fi
            shift
            ;;
    esac
done

if [ -z "$BACKUP_FILE" ]; then
    echo "ERROR: No backup file specified"
    usage
fi

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Download from S3 if requested
if [ "$FROM_S3" = true ]; then
    log "Downloading backup from S3: $BACKUP_FILE"
    aws s3 cp "s3://$S3_BUCKET/$S3_PREFIX/$BACKUP_FILE" "$BACKUP_DIR/$BACKUP_FILE"
    
    if [ $? -ne 0 ]; then
        log "ERROR: Failed to download backup from S3"
        exit 1
    fi
fi

# Check if backup file exists
if [ ! -f "$BACKUP_DIR/$BACKUP_FILE" ]; then
    log "ERROR: Backup file not found: $BACKUP_DIR/$BACKUP_FILE"
    exit 1
fi

# Verify backup integrity
log "Verifying backup integrity..."
gunzip -t "$BACKUP_DIR/$BACKUP_FILE"
if [ $? -ne 0 ]; then
    log "ERROR: Backup file is corrupted"
    exit 1
fi

# Confirmation prompt
if [ "$FORCE" = false ]; then
    echo ""
    echo "WARNING: This will completely replace the current database!"
    echo "Database: $POSTGRES_DB"
    echo "Host: $POSTGRES_HOST"
    echo "Backup file: $BACKUP_FILE"
    echo ""
    read -p "Are you sure you want to continue? (yes/no): " -r
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log "Restore cancelled by user"
        exit 0
    fi
fi

log "Starting database restore from: $BACKUP_FILE"

# Create a pre-restore backup
PRE_RESTORE_BACKUP="pre_restore_backup_$(date '+%Y%m%d_%H%M%S').sql"
log "Creating pre-restore backup: $PRE_RESTORE_BACKUP"

PGPASSWORD="$POSTGRES_PASSWORD" pg_dump \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    --verbose \
    --no-password \
    --format=custom \
    --compress=9 \
    --file="$BACKUP_DIR/$PRE_RESTORE_BACKUP"

if [ $? -ne 0 ]; then
    log "WARNING: Pre-restore backup failed, continuing anyway..."
fi

# Decompress backup
log "Decompressing backup..."
DECOMPRESSED_FILE="${BACKUP_FILE%.gz}"
gunzip -c "$BACKUP_DIR/$BACKUP_FILE" > "$BACKUP_DIR/$DECOMPRESSED_FILE"

# Decrypt backup
log "Decrypting backup..."
DECRYPTED_FILE="${DECOMPRESSED_FILE%.enc}"
openssl enc -aes-256-cbc -d -in "$BACKUP_DIR/$DECOMPRESSED_FILE" -out "$BACKUP_DIR/$DECRYPTED_FILE" -k "$ENCRYPTION_KEY"

if [ $? -ne 0 ]; then
    log "ERROR: Backup decryption failed"
    rm -f "$BACKUP_DIR/$DECOMPRESSED_FILE"
    exit 1
fi

# Remove compressed and encrypted files
rm -f "$BACKUP_DIR/$DECOMPRESSED_FILE"

# Stop all services that use the database
log "Stopping services..."
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml stop \
    api-gateway auth-service user-service file-service workflow-service tenant-service \
    auth-bff tenant-bff file-bff user-bff workflow-bff

# Drop existing database connections
log "Terminating existing database connections..."
PGPASSWORD="$POSTGRES_PASSWORD" psql \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d postgres \
    -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$POSTGRES_DB' AND pid <> pg_backend_pid();"

# Drop and recreate database
log "Dropping and recreating database..."
PGPASSWORD="$POSTGRES_PASSWORD" psql \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d postgres \
    -c "DROP DATABASE IF EXISTS $POSTGRES_DB;"

PGPASSWORD="$POSTGRES_PASSWORD" psql \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d postgres \
    -c "CREATE DATABASE $POSTGRES_DB;"

# Restore database
log "Restoring database..."
PGPASSWORD="$POSTGRES_PASSWORD" pg_restore \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    --verbose \
    --no-password \
    --clean \
    --if-exists \
    "$BACKUP_DIR/$DECRYPTED_FILE"

if [ $? -ne 0 ]; then
    log "ERROR: Database restore failed"
    
    # Attempt to restore pre-restore backup
    if [ -f "$BACKUP_DIR/$PRE_RESTORE_BACKUP" ]; then
        log "Attempting to restore pre-restore backup..."
        PGPASSWORD="$POSTGRES_PASSWORD" pg_restore \
            -h "$POSTGRES_HOST" \
            -p "$POSTGRES_PORT" \
            -U "$POSTGRES_USER" \
            -d "$POSTGRES_DB" \
            --verbose \
            --no-password \
            --clean \
            --if-exists \
            "$BACKUP_DIR/$PRE_RESTORE_BACKUP"
    fi
    
    rm -f "$BACKUP_DIR/$DECRYPTED_FILE"
    exit 1
fi

# Clean up decrypted file
rm -f "$BACKUP_DIR/$DECRYPTED_FILE"

# Run database migrations to ensure schema is up to date
log "Running database migrations..."
cd adx-core
sqlx migrate run --database-url "postgresql://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB"

# Start services
log "Starting services..."
docker-compose -f adx-core/infrastructure/production/docker-compose.prod.yml start \
    api-gateway auth-service user-service file-service workflow-service tenant-service \
    auth-bff tenant-bff file-bff user-bff workflow-bff

# Wait for services to be healthy
log "Waiting for services to be healthy..."
sleep 30

# Verify restore
log "Verifying database restore..."
PGPASSWORD="$POSTGRES_PASSWORD" psql \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" \
    -t | tr -d ' '

if [ $? -eq 0 ]; then
    log "Database restore completed successfully"
else
    log "ERROR: Database restore verification failed"
    exit 1
fi

# Send notification
if [ -n "${SLACK_WEBHOOK_URL:-}" ]; then
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"✅ Database restore completed successfully from: $BACKUP_FILE\"}" \
        "$SLACK_WEBHOOK_URL"
fi

log "Restore process completed"
exit 0