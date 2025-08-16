#!/bin/bash

# ADX Core Deployment Rollback Script
# This script handles automated rollback of failed deployments

set -e

# Configuration
ENVIRONMENT=${1:-staging}
ROLLBACK_TARGET=${2:-previous}  # previous, version, or specific commit SHA
COMPONENT=${3:-all}  # all, backend, frontend, or specific service/app name
DRY_RUN=${4:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
NAMESPACE="adx-core-${ENVIRONMENT}"
BACKUP_NAMESPACE="adx-core-${ENVIRONMENT}-backup"
S3_BUCKET="adx-core-deployments-${ENVIRONMENT}"
REGISTRY="ghcr.io/adxcore/adx-core"

# Rollback functions
get_deployment_history() {
    local component=$1
    log_info "Getting deployment history for $component..."
    
    if [[ "$component" == "backend" ]] || [[ "$component" == "all" ]]; then
        # Get Kubernetes deployment history
        local services=(
            "api-gateway"
            "auth-service"
            "user-service"
            "file-service"
            "tenant-service"
            "workflow-service"
            "module-service"
            "license-service"
        )
        
        for service in "${services[@]}"; do
            if kubectl get deployment "$service" -n "$NAMESPACE" >/dev/null 2>&1; then
                log_info "Deployment history for $service:"
                kubectl rollout history deployment/"$service" -n "$NAMESPACE"
            fi
        done
    fi
    
    if [[ "$component" == "frontend" ]] || [[ "$component" == "all" ]]; then
        # Get S3 deployment history
        log_info "Frontend deployment history:"
        aws s3 ls "s3://$S3_BUCKET/deployments/" --recursive | grep -E "\.(json)$" | tail -10
    fi
}

backup_current_deployment() {
    log_info "Creating backup of current deployment..."
    
    local timestamp=$(date -u +"%Y%m%d-%H%M%S")
    local backup_dir="backup-${timestamp}"
    
    # Create backup namespace if it doesn't exist
    kubectl create namespace "$BACKUP_NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    # Backup Kubernetes resources
    log_info "Backing up Kubernetes resources..."
    kubectl get all -n "$NAMESPACE" -o yaml > "${backup_dir}-k8s-resources.yaml"
    
    # Backup ConfigMaps and Secrets
    kubectl get configmaps -n "$NAMESPACE" -o yaml > "${backup_dir}-configmaps.yaml"
    kubectl get secrets -n "$NAMESPACE" -o yaml > "${backup_dir}-secrets.yaml"
    
    # Upload backups to S3
    aws s3 cp "${backup_dir}-k8s-resources.yaml" "s3://$S3_BUCKET/backups/"
    aws s3 cp "${backup_dir}-configmaps.yaml" "s3://$S3_BUCKET/backups/"
    aws s3 cp "${backup_dir}-secrets.yaml" "s3://$S3_BUCKET/backups/"
    
    # Store backup metadata
    cat > "${backup_dir}-metadata.json" << EOF
{
  "timestamp": "$timestamp",
  "environment": "$ENVIRONMENT",
  "namespace": "$NAMESPACE",
  "backupType": "pre-rollback",
  "components": ["$COMPONENT"],
  "createdBy": "$(whoami)",
  "gitCommit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')"
}
EOF
    
    aws s3 cp "${backup_dir}-metadata.json" "s3://$S3_BUCKET/backups/"
    
    log_success "Backup created: $backup_dir"
    echo "$backup_dir"
}

rollback_backend_services() {
    local target_revision=$1
    log_info "Rolling back backend services to revision $target_revision..."
    
    local services=(
        "api-gateway"
        "auth-service"
        "user-service"
        "file-service"
        "tenant-service"
        "workflow-service"
        "module-service"
        "license-service"
    )
    
    local failed_rollbacks=()
    
    for service in "${services[@]}"; do
        if kubectl get deployment "$service" -n "$NAMESPACE" >/dev/null 2>&1; then
            log_info "Rolling back $service..."
            
            if [[ "$DRY_RUN" == "true" ]]; then
                log_info "[DRY RUN] Would rollback $service to revision $target_revision"
            else
                if kubectl rollout undo deployment/"$service" --to-revision="$target_revision" -n "$NAMESPACE"; then
                    log_success "Successfully initiated rollback for $service"
                    
                    # Wait for rollback to complete
                    if kubectl rollout status deployment/"$service" -n "$NAMESPACE" --timeout=300s; then
                        log_success "Rollback completed for $service"
                    else
                        log_error "Rollback timed out for $service"
                        failed_rollbacks+=("$service")
                    fi
                else
                    log_error "Failed to initiate rollback for $service"
                    failed_rollbacks+=("$service")
                fi
            fi
        else
            log_warning "$service deployment not found, skipping..."
        fi
    done
    
    if [[ ${#failed_rollbacks[@]} -gt 0 ]]; then
        log_error "Failed to rollback services: ${failed_rollbacks[*]}"
        return 1
    fi
    
    log_success "All backend services rolled back successfully"
    return 0
}

rollback_frontend_apps() {
    local target_version=$1
    log_info "Rolling back frontend apps to version $target_version..."
    
    local apps=("shell" "auth" "tenant" "file" "user" "workflow" "module")
    local failed_rollbacks=()
    
    # Get target deployment info
    local target_deployment_info
    if [[ "$target_version" == "previous" ]]; then
        # Find the previous deployment
        target_deployment_info=$(aws s3 ls "s3://$S3_BUCKET/deployments/" | grep -E "\.json$" | tail -2 | head -1 | awk '{print $4}')
    else
        target_deployment_info="${target_version}.json"
    fi
    
    if [[ -z "$target_deployment_info" ]]; then
        log_error "Could not find target deployment info"
        return 1
    fi
    
    # Download target deployment info
    aws s3 cp "s3://$S3_BUCKET/deployments/$target_deployment_info" "./target-deployment.json"
    local target_commit=$(jq -r '.commit.sha' ./target-deployment.json)
    
    log_info "Rolling back to commit: $target_commit"
    
    for app in "${apps[@]}"; do
        log_info "Rolling back $app..."
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "[DRY RUN] Would rollback $app to commit $target_commit"
        else
            # Restore files from S3 backup
            local s3_source_path
            if [[ "$app" == "shell" ]]; then
                s3_source_path="s3://$S3_BUCKET/versions/$target_commit/"
            else
                s3_source_path="s3://$S3_BUCKET/versions/$target_commit/apps/$app/"
            fi
            
            local s3_target_path
            if [[ "$app" == "shell" ]]; then
                s3_target_path="s3://$S3_BUCKET/$ENVIRONMENT/"
            else
                s3_target_path="s3://$S3_BUCKET/$ENVIRONMENT/apps/$app/"
            fi
            
            if aws s3 sync "$s3_source_path" "$s3_target_path" --delete; then
                log_success "Successfully rolled back $app files"
                
                # Invalidate CloudFront cache
                local invalidation_paths
                if [[ "$app" == "shell" ]]; then
                    invalidation_paths="/$ENVIRONMENT/*"
                else
                    invalidation_paths="/$ENVIRONMENT/apps/$app/*"
                fi
                
                aws cloudfront create-invalidation \
                    --distribution-id "$CLOUDFRONT_DISTRIBUTION_ID" \
                    --paths "$invalidation_paths" >/dev/null
                
                log_success "CloudFront cache invalidated for $app"
            else
                log_error "Failed to rollback $app files"
                failed_rollbacks+=("$app")
            fi
        fi
    done
    
    if [[ ${#failed_rollbacks[@]} -gt 0 ]]; then
        log_error "Failed to rollback apps: ${failed_rollbacks[*]}"
        return 1
    fi
    
    log_success "All frontend apps rolled back successfully"
    return 0
}

rollback_database_migrations() {
    log_info "Checking for database migration rollbacks..."
    
    # This is a placeholder for database migration rollbacks
    # In a real scenario, you would need to implement proper migration rollback logic
    log_warning "Database migration rollbacks are not implemented in this script"
    log_warning "Please manually review and rollback database migrations if necessary"
    
    return 0
}

verify_rollback() {
    log_info "Verifying rollback success..."
    
    # Run health checks
    if ./scripts/deployment-health-check.sh "$ENVIRONMENT" 60 5; then
        log_success "Rollback verification passed"
        return 0
    else
        log_error "Rollback verification failed"
        return 1
    fi
}

send_rollback_notification() {
    local status=$1
    local backup_id=$2
    
    local message
    if [[ "$status" == "success" ]]; then
        message="✅ Rollback completed successfully for $ENVIRONMENT environment"
    else
        message="❌ Rollback failed for $ENVIRONMENT environment"
    fi
    
    # Send Slack notification
    if [[ -n "$SLACK_WEBHOOK_URL" ]]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"$message\n\nComponent: $COMPONENT\nTarget: $ROLLBACK_TARGET\nBackup ID: $backup_id\nExecuted by: $(whoami)\"}" \
            "$SLACK_WEBHOOK_URL"
    fi
    
    # Send email notification
    if [[ -n "$NOTIFICATION_EMAIL" ]]; then
        echo "$message" | mail -s "ADX Core Rollback Notification" "$NOTIFICATION_EMAIL"
    fi
}

cleanup_rollback_artifacts() {
    log_info "Cleaning up rollback artifacts..."
    
    # Remove temporary files
    rm -f target-deployment.json
    rm -f backup-*-metadata.json
    rm -f backup-*-k8s-resources.yaml
    rm -f backup-*-configmaps.yaml
    rm -f backup-*-secrets.yaml
    
    log_success "Cleanup completed"
}

show_usage() {
    cat << EOF
Usage: $0 [ENVIRONMENT] [ROLLBACK_TARGET] [COMPONENT] [DRY_RUN]

Arguments:
  ENVIRONMENT     Target environment (staging, production) [default: staging]
  ROLLBACK_TARGET Rollback target (previous, version, or commit SHA) [default: previous]
  COMPONENT       Component to rollback (all, backend, frontend, or specific name) [default: all]
  DRY_RUN         Perform dry run without actual changes (true, false) [default: false]

Examples:
  $0 staging previous all false          # Rollback everything to previous version
  $0 production abc123 backend true      # Dry run rollback of backend to commit abc123
  $0 staging previous auth-service false # Rollback only auth-service to previous version

Environment Variables:
  SLACK_WEBHOOK_URL      Slack webhook for notifications
  NOTIFICATION_EMAIL     Email address for notifications
  CLOUDFRONT_DISTRIBUTION_ID  CloudFront distribution ID for cache invalidation
EOF
}

# Main execution
main() {
    if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
        show_usage
        exit 0
    fi
    
    log_info "Starting ADX Core deployment rollback..."
    log_info "Environment: $ENVIRONMENT"
    log_info "Rollback target: $ROLLBACK_TARGET"
    log_info "Component: $COMPONENT"
    log_info "Dry run: $DRY_RUN"
    
    # Validate environment
    if [[ "$ENVIRONMENT" != "staging" ]] && [[ "$ENVIRONMENT" != "production" ]]; then
        log_error "Invalid environment: $ENVIRONMENT. Must be 'staging' or 'production'"
        exit 1
    fi
    
    # Show deployment history
    get_deployment_history "$COMPONENT"
    
    # Confirm rollback (unless dry run)
    if [[ "$DRY_RUN" != "true" ]]; then
        echo
        read -p "Are you sure you want to proceed with the rollback? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Rollback cancelled by user"
            exit 0
        fi
    fi
    
    local overall_success=true
    local backup_id=""
    
    # Create backup
    if [[ "$DRY_RUN" != "true" ]]; then
        backup_id=$(backup_current_deployment)
    fi
    
    # Perform rollbacks based on component
    case "$COMPONENT" in
        "all")
            if ! rollback_backend_services "$ROLLBACK_TARGET"; then
                overall_success=false
            fi
            if ! rollback_frontend_apps "$ROLLBACK_TARGET"; then
                overall_success=false
            fi
            ;;
        "backend")
            if ! rollback_backend_services "$ROLLBACK_TARGET"; then
                overall_success=false
            fi
            ;;
        "frontend")
            if ! rollback_frontend_apps "$ROLLBACK_TARGET"; then
                overall_success=false
            fi
            ;;
        *)
            # Specific service or app
            if kubectl get deployment "$COMPONENT" -n "$NAMESPACE" >/dev/null 2>&1; then
                # It's a backend service
                if ! rollback_backend_services "$ROLLBACK_TARGET"; then
                    overall_success=false
                fi
            else
                # Assume it's a frontend app
                if ! rollback_frontend_apps "$ROLLBACK_TARGET"; then
                    overall_success=false
                fi
            fi
            ;;
    esac
    
    # Check for database migration rollbacks
    rollback_database_migrations
    
    # Verify rollback
    if [[ "$DRY_RUN" != "true" ]] && [[ "$overall_success" == "true" ]]; then
        if ! verify_rollback; then
            overall_success=false
        fi
    fi
    
    # Send notifications
    if [[ "$DRY_RUN" != "true" ]]; then
        if [[ "$overall_success" == "true" ]]; then
            send_rollback_notification "success" "$backup_id"
            log_success "Rollback completed successfully!"
        else
            send_rollback_notification "failure" "$backup_id"
            log_error "Rollback completed with errors!"
        fi
    else
        log_info "Dry run completed. No actual changes were made."
    fi
    
    # Cleanup
    cleanup_rollback_artifacts
    
    if [[ "$overall_success" == "true" ]]; then
        exit 0
    else
        exit 1
    fi
}

# Check dependencies
command -v kubectl >/dev/null 2>&1 || { log_error "kubectl is required but not installed. Aborting."; exit 1; }
command -v aws >/dev/null 2>&1 || { log_error "aws CLI is required but not installed. Aborting."; exit 1; }
command -v jq >/dev/null 2>&1 || { log_error "jq is required but not installed. Aborting."; exit 1; }

# Run main function
main "$@"