#!/bin/bash

# ADX Core Deployment Health Check Script
# This script performs comprehensive health checks after deployment

set -e

# Configuration
ENVIRONMENT=${1:-staging}
TIMEOUT=${2:-300}
CHECK_INTERVAL=${3:-10}
NAMESPACE="adx-core-${ENVIRONMENT}"

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

# Health check functions
check_kubernetes_services() {
    log_info "Checking Kubernetes services health..."
    
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
    
    local failed_services=()
    
    for service in "${services[@]}"; do
        log_info "Checking $service..."
        
        # Check if deployment exists and is ready
        if kubectl get deployment "$service" -n "$NAMESPACE" >/dev/null 2>&1; then
            local ready_replicas=$(kubectl get deployment "$service" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')
            local desired_replicas=$(kubectl get deployment "$service" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
            
            if [[ "$ready_replicas" == "$desired_replicas" ]] && [[ "$ready_replicas" -gt 0 ]]; then
                log_success "$service is healthy ($ready_replicas/$desired_replicas replicas ready)"
            else
                log_error "$service is not healthy ($ready_replicas/$desired_replicas replicas ready)"
                failed_services+=("$service")
            fi
        else
            log_error "$service deployment not found"
            failed_services+=("$service")
        fi
    done
    
    if [[ ${#failed_services[@]} -gt 0 ]]; then
        log_error "Failed services: ${failed_services[*]}"
        return 1
    fi
    
    log_success "All Kubernetes services are healthy"
    return 0
}

check_service_endpoints() {
    log_info "Checking service endpoints..."
    
    local services=(
        "api-gateway:8080:/health"
        "auth-service:8081:/health"
        "user-service:8082:/health"
        "file-service:8083:/health"
        "tenant-service:8085:/health"
        "workflow-service:8084:/health"
    )
    
    local failed_endpoints=()
    
    for service_info in "${services[@]}"; do
        IFS=':' read -r service port path <<< "$service_info"
        log_info "Checking $service endpoint..."
        
        # Port forward to the service
        kubectl port-forward "service/$service" "$port:80" -n "$NAMESPACE" &
        local pf_pid=$!
        
        # Wait for port forward to be ready
        sleep 3
        
        # Check health endpoint
        local http_status
        http_status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:$port$path" || echo "000")
        
        if [[ "$http_status" == "200" ]]; then
            log_success "$service endpoint is healthy"
        else
            log_error "$service endpoint returned HTTP $http_status"
            failed_endpoints+=("$service")
        fi
        
        # Clean up port forward
        kill $pf_pid 2>/dev/null || true
        wait $pf_pid 2>/dev/null || true
    done
    
    if [[ ${#failed_endpoints[@]} -gt 0 ]]; then
        log_error "Failed endpoints: ${failed_endpoints[*]}"
        return 1
    fi
    
    log_success "All service endpoints are healthy"
    return 0
}

check_database_connectivity() {
    log_info "Checking database connectivity..."
    
    # Get database credentials from secret
    local db_url
    db_url=$(kubectl get secret database-credentials -n "$NAMESPACE" -o jsonpath='{.data.url}' | base64 -d)
    
    if [[ -z "$db_url" ]]; then
        log_error "Database URL not found in secrets"
        return 1
    fi
    
    # Create a temporary pod to test database connectivity
    kubectl run db-test --rm -i --restart=Never --image=postgres:14 -n "$NAMESPACE" -- \
        psql "$db_url" -c "SELECT 1;" >/dev/null 2>&1
    
    if [[ $? -eq 0 ]]; then
        log_success "Database connectivity is healthy"
        return 0
    else
        log_error "Database connectivity failed"
        return 1
    fi
}

check_temporal_connectivity() {
    log_info "Checking Temporal connectivity..."
    
    # Port forward to Temporal service
    kubectl port-forward service/temporal-server 7233:7233 -n "$NAMESPACE" &
    local pf_pid=$!
    
    # Wait for port forward to be ready
    sleep 3
    
    # Check Temporal health
    local temporal_health
    temporal_health=$(curl -s "http://localhost:7233/api/v1/namespaces" | jq -r '.namespaces | length' 2>/dev/null || echo "0")
    
    # Clean up port forward
    kill $pf_pid 2>/dev/null || true
    wait $pf_pid 2>/dev/null || true
    
    if [[ "$temporal_health" -gt 0 ]]; then
        log_success "Temporal connectivity is healthy"
        return 0
    else
        log_error "Temporal connectivity failed"
        return 1
    fi
}

check_redis_connectivity() {
    log_info "Checking Redis connectivity..."
    
    # Port forward to Redis service
    kubectl port-forward service/redis 6379:6379 -n "$NAMESPACE" &
    local pf_pid=$!
    
    # Wait for port forward to be ready
    sleep 3
    
    # Check Redis health
    local redis_response
    redis_response=$(redis-cli -h localhost -p 6379 ping 2>/dev/null || echo "FAILED")
    
    # Clean up port forward
    kill $pf_pid 2>/dev/null || true
    wait $pf_pid 2>/dev/null || true
    
    if [[ "$redis_response" == "PONG" ]]; then
        log_success "Redis connectivity is healthy"
        return 0
    else
        log_error "Redis connectivity failed"
        return 1
    fi
}

check_microfrontend_availability() {
    log_info "Checking micro-frontend availability..."
    
    local base_url
    if [[ "$ENVIRONMENT" == "production" ]]; then
        base_url="https://app.adxcore.com"
    else
        base_url="https://staging.adxcore.com"
    fi
    
    local apps=("shell" "auth" "tenant" "file" "user" "workflow" "module")
    local failed_apps=()
    
    for app in "${apps[@]}"; do
        log_info "Checking $app micro-frontend..."
        
        local url
        if [[ "$app" == "shell" ]]; then
            url="$base_url/"
        else
            url="$base_url/apps/$app/"
        fi
        
        local http_status
        http_status=$(curl -s -o /dev/null -w "%{http_code}" "$url" || echo "000")
        
        if [[ "$http_status" == "200" ]]; then
            log_success "$app micro-frontend is available"
            
            # Check remoteEntry.js for micro-frontends (not shell)
            if [[ "$app" != "shell" ]]; then
                local remote_entry_url="$base_url/apps/$app/assets/remoteEntry.js"
                local remote_status
                remote_status=$(curl -s -o /dev/null -w "%{http_code}" "$remote_entry_url" || echo "000")
                
                if [[ "$remote_status" == "200" ]]; then
                    log_success "$app remoteEntry.js is available"
                else
                    log_error "$app remoteEntry.js returned HTTP $remote_status"
                    failed_apps+=("$app-remoteEntry")
                fi
            fi
        else
            log_error "$app micro-frontend returned HTTP $http_status"
            failed_apps+=("$app")
        fi
    done
    
    if [[ ${#failed_apps[@]} -gt 0 ]]; then
        log_error "Failed micro-frontends: ${failed_apps[*]}"
        return 1
    fi
    
    log_success "All micro-frontends are available"
    return 0
}

check_workflow_execution() {
    log_info "Checking workflow execution..."
    
    # Port forward to API Gateway
    kubectl port-forward service/api-gateway 8080:80 -n "$NAMESPACE" &
    local pf_pid=$!
    
    # Wait for port forward to be ready
    sleep 3
    
    # Test a simple workflow execution
    local workflow_response
    workflow_response=$(curl -s -X POST "http://localhost:8080/api/v1/workflows/health-check" \
        -H "Content-Type: application/json" \
        -d '{"test": true}' || echo "FAILED")
    
    # Clean up port forward
    kill $pf_pid 2>/dev/null || true
    wait $pf_pid 2>/dev/null || true
    
    if [[ "$workflow_response" != "FAILED" ]] && echo "$workflow_response" | jq -e '.operationId' >/dev/null 2>&1; then
        log_success "Workflow execution is healthy"
        return 0
    else
        log_error "Workflow execution failed"
        return 1
    fi
}

check_performance_metrics() {
    log_info "Checking performance metrics..."
    
    # Port forward to API Gateway for performance test
    kubectl port-forward service/api-gateway 8080:80 -n "$NAMESPACE" &
    local pf_pid=$!
    
    # Wait for port forward to be ready
    sleep 3
    
    # Measure response time
    local start_time=$(date +%s%N)
    local http_status
    http_status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:8080/health" || echo "000")
    local end_time=$(date +%s%N)
    
    # Clean up port forward
    kill $pf_pid 2>/dev/null || true
    wait $pf_pid 2>/dev/null || true
    
    local response_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    
    if [[ "$http_status" == "200" ]] && [[ "$response_time" -lt 2000 ]]; then
        log_success "Performance metrics are healthy (${response_time}ms response time)"
        return 0
    else
        log_error "Performance metrics failed (HTTP $http_status, ${response_time}ms response time)"
        return 1
    fi
}

generate_health_report() {
    local overall_status=$1
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    cat > "health-report-${ENVIRONMENT}-${timestamp}.json" << EOF
{
  "environment": "$ENVIRONMENT",
  "timestamp": "$timestamp",
  "overallStatus": "$overall_status",
  "checks": {
    "kubernetesServices": $kubernetes_status,
    "serviceEndpoints": $endpoints_status,
    "databaseConnectivity": $database_status,
    "temporalConnectivity": $temporal_status,
    "redisConnectivity": $redis_status,
    "microfrontendAvailability": $microfrontend_status,
    "workflowExecution": $workflow_status,
    "performanceMetrics": $performance_status
  },
  "summary": {
    "totalChecks": 8,
    "passedChecks": $passed_checks,
    "failedChecks": $failed_checks
  }
}
EOF
    
    log_info "Health report generated: health-report-${ENVIRONMENT}-${timestamp}.json"
}

# Main execution
main() {
    log_info "Starting ADX Core deployment health check for environment: $ENVIRONMENT"
    log_info "Timeout: ${TIMEOUT}s, Check interval: ${CHECK_INTERVAL}s"
    
    local overall_status="HEALTHY"
    local passed_checks=0
    local failed_checks=0
    
    # Run all health checks
    local checks=(
        "check_kubernetes_services:kubernetes_status"
        "check_service_endpoints:endpoints_status"
        "check_database_connectivity:database_status"
        "check_temporal_connectivity:temporal_status"
        "check_redis_connectivity:redis_status"
        "check_microfrontend_availability:microfrontend_status"
        "check_workflow_execution:workflow_status"
        "check_performance_metrics:performance_status"
    )
    
    for check_info in "${checks[@]}"; do
        IFS=':' read -r check_function status_var <<< "$check_info"
        
        if $check_function; then
            declare "$status_var=true"
            ((passed_checks++))
        else
            declare "$status_var=false"
            ((failed_checks++))
            overall_status="UNHEALTHY"
        fi
        
        echo # Add spacing between checks
    done
    
    # Generate health report
    generate_health_report "$overall_status"
    
    # Summary
    log_info "Health check summary:"
    log_info "  Passed: $passed_checks"
    log_info "  Failed: $failed_checks"
    log_info "  Overall status: $overall_status"
    
    if [[ "$overall_status" == "HEALTHY" ]]; then
        log_success "All health checks passed! Deployment is healthy."
        exit 0
    else
        log_error "Some health checks failed! Deployment may have issues."
        exit 1
    fi
}

# Check dependencies
command -v kubectl >/dev/null 2>&1 || { log_error "kubectl is required but not installed. Aborting."; exit 1; }
command -v curl >/dev/null 2>&1 || { log_error "curl is required but not installed. Aborting."; exit 1; }
command -v jq >/dev/null 2>&1 || { log_error "jq is required but not installed. Aborting."; exit 1; }

# Run main function
main "$@"