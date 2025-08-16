#!/bin/bash

# ADX Core Database Backup Script
# This script creates encrypted backups of PostgreSQL databases

set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/backups/database}"
POSTGRES_HOST="${POSTGRES_HOST:-postgresql}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-adx_core}"
ENCRYPTION_KEY="${BACKUP_ENCRYPTION_KEY}"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"
S3_BUCKET="${BACKUP_S3_BUCKET}"
S3_PREFIX="${BACKUP_S3_PREFIX:-database-backups}"

# Logging
LOG_FILE="/var/log/adx-core/backup-database.log"
exec 1> >(tee -a "$LOG_FILE")
exec 2>&1

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Generate backup filename
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
BACKUP_FILE="adx_core_backup_${TIMESTAMP}.sql"
ENCRYPTED_FILE="${BACKUP_FILE}.enc"
COMPRESSED_FILE="${ENCRYPTED_FILE}.gz"

log "Starting database backup: $BACKUP_FILE"

# Create database dump
log "Creating database dump..."
PGPASSWORD="$POSTGRES_PASSWORD" pg_dump \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DB" \
    --verbose \
    --no-password \
    --format=custom \
    --compress=9 \
    --file="$BACKUP_DIR/$BACKUP_FILE"

if [ $? -ne 0 ]; then
    log "ERROR: Database dump failed"
    exit 1
fi

log "Database dump completed: $(du -h "$BACKUP_DIR/$BACKUP_FILE" | cut -f1)"

# Encrypt backup
log "Encrypting backup..."
openssl enc -aes-256-cbc -salt -in "$BACKUP_DIR/$BACKUP_FILE" -out "$BACKUP_DIR/$ENCRYPTED_FILE" -k "$ENCRYPTION_KEY"

if [ $? -ne 0 ]; then
    log "ERROR: Backup encryption failed"
    rm -f "$BACKUP_DIR/$BACKUP_FILE"
    exit 1
fi

# Remove unencrypted backup
rm -f "$BACKUP_DIR/$BACKUP_FILE"

# Compress encrypted backup
log "Compressing backup..."
gzip "$BACKUP_DIR/$ENCRYPTED_FILE"

log "Backup encrypted and compressed: $(du -h "$BACKUP_DIR/$COMPRESSED_FILE" | cut -f1)"

# Upload to S3
if [ -n "$S3_BUCKET" ]; then
    log "Uploading backup to S3..."
    aws s3 cp "$BACKUP_DIR/$COMPRESSED_FILE" "s3://$S3_BUCKET/$S3_PREFIX/$COMPRESSED_FILE" \
        --storage-class STANDARD_IA \
        --server-side-encryption AES256
    
    if [ $? -eq 0 ]; then
        log "Backup uploaded to S3 successfully"
        # Remove local copy after successful upload
        rm -f "$BACKUP_DIR/$COMPRESSED_FILE"
    else
        log "WARNING: S3 upload failed, keeping local copy"
    fi
fi

# Cleanup old backups
log "Cleaning up old backups..."
find "$BACKUP_DIR" -name "adx_core_backup_*.sql.enc.gz" -mtime +$RETENTION_DAYS -delete

# Cleanup old S3 backups
if [ -n "$S3_BUCKET" ]; then
    CUTOFF_DATE=$(date -d "$RETENTION_DAYS days ago" '+%Y-%m-%d')
    aws s3 ls "s3://$S3_BUCKET/$S3_PREFIX/" | while read -r line; do
        FILE_DATE=$(echo "$line" | awk '{print $1}')
        FILE_NAME=$(echo "$line" | awk '{print $4}')
        
        if [[ "$FILE_DATE" < "$CUTOFF_DATE" ]]; then
            log "Deleting old S3 backup: $FILE_NAME"
            aws s3 rm "s3://$S3_BUCKET/$S3_PREFIX/$FILE_NAME"
        fi
    done
fi

# Verify backup integrity
log "Verifying backup integrity..."
if [ -f "$BACKUP_DIR/$COMPRESSED_FILE" ]; then
    gunzip -t "$BACKUP_DIR/$COMPRESSED_FILE"
    if [ $? -eq 0 ]; then
        log "Backup integrity verified"
    else
        log "ERROR: Backup integrity check failed"
        exit 1
    fi
fi

# Create backup metadata
METADATA_FILE="$BACKUP_DIR/backup_metadata_${TIMESTAMP}.json"
cat > "$METADATA_FILE" << EOF
{
    "backup_file": "$COMPRESSED_FILE",
    "timestamp": "$TIMESTAMP",
    "database": "$POSTGRES_DB",
    "host": "$POSTGRES_HOST",
    "size_bytes": $(stat -c%s "$BACKUP_DIR/$COMPRESSED_FILE" 2>/dev/null || echo "0"),
    "checksum": "$(sha256sum "$BACKUP_DIR/$COMPRESSED_FILE" 2>/dev/null | cut -d' ' -f1 || echo 'unknown')",
    "encrypted": true,
    "compressed": true,
    "s3_location": "s3://$S3_BUCKET/$S3_PREFIX/$COMPRESSED_FILE"
}
EOF

log "Backup completed successfully: $COMPRESSED_FILE"
log "Metadata saved: $METADATA_FILE"

# Send notification
if [ -n "${SLACK_WEBHOOK_URL:-}" ]; then
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"✅ Database backup completed successfully: $COMPRESSED_FILE\"}" \
        "$SLACK_WEBHOOK_URL"
fi

exit 0