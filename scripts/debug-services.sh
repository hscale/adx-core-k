#!/bin/bash

# ADX Core Service Debugging Tool
# Comprehensive debugging and diagnostics for all ADX Core services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
VERBOSE=${VERBOSE:-false}
FOLLOW_LOGS=${FOLLOW_LOGS:-false}
SERVICE_FILTER=${SERVICE_FILTER:-""}
DEBUG_LEVEL=${DEBUG_LEVEL:-"debug"}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADX_CORE_DIR="$PROJECT_ROOT/adx-core"

# Log file
DEBUG_LOG="$PROJECT_ROOT/debug-session-$(date +%Y%m%d-%H%M%S).log"

# Service definitions
declare -A BACKEND_SERVICES=(
    ["api-gateway"]="8080"
    ["auth-service"]="8081"
    ["user-service"]="8082"
    ["file-service"]="8083"
    ["workflow-service"]="8084"
    ["tenant-service"]="8085"
    ["module-service"]="8086"
    ["license-service"]="8087"
)

declare -A FRONTEND_SERVICES=(
    ["shell"]="3000"
    ["auth"]="3001"
    ["tenant"]="3002"
    ["file"]="3003"
    ["user"]="3004"
    ["workflow"]="3005"
    ["module"]="3006"
)

declare -A BFF_SERVICES=(
    ["auth-bff"]="4001"
    ["tenant-bff"]="4002"
    ["file-bff"]="4003"
    ["user-bff"]="4004"
    ["workflow-bff"]="4005"
    ["module-bff"]="4006"
)

declare -A INFRASTRUCTURE_SERVICES=(
    ["postgres"]="5432"
    ["redis"]="6379"
    ["temporal"]="7233"
    ["temporal-ui"]="8088"
)

# Utility functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$DEBUG_LOG"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$DEBUG_LOG"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$DEBUG_LOG"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$DEBUG_LOG"
}

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1" | tee -a "$DEBUG_LOG"
}

check_service_health() {
    local service_name="$1"
    local port="$2"
    local health_endpoint="${3:-/health}"
    
    log_info "Checking health of $service_name on port $port..."
    
    # Check if port is open
    if ! nc -z localhost "$port" 2>/dev/null; then
        log_error "$service_name is not listening on port $port"
        return 1
    fi
    
    # Try HTTP health check
    local health_url="http://localhost:$port$health_endpoint"
    if curl -s -f "$health_url" > /dev/null 2>&1; then
        log_success "$service_name is healthy"
        return 0
    else
        log_warning "$service_name port is open but health check failed"
        return 1
    fi
}

get_service_logs() {
    local service_name="$1"
    local lines="${2:-50}"
    
    log_info "Getting logs for $service_name (last $lines lines)..."
    
    # Try docker-compose logs first
    if docker-compose -f "$ADX_CORE_DIR/infrastructure/docker/docker-compose.dev.yml" logs --tail="$lines" "$service_name" 2>/dev/null; then
        return 0
    fi
    
    # Try systemd logs
    if systemctl is-active --quiet "$service_name" 2>/dev/null; then
        journalctl -u "$service_name" -n "$lines" --no-pager
        return 0
    fi
    
    # Try process logs
    local pid=$(pgrep -f "$service_name" | head -1)
    if [[ -n "$pid" ]]; then
        log_info "Service $service_name running with PID $pid"
        ps -p "$pid" -o pid,ppid,cmd,etime,pcpu,pmem
    else
        log_warning "No logs found for $service_name"
    fi
}

check_database_connectivity() {
    log "=== Database Connectivity Check ==="
    
    local db_url="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/adx_core_dev}"
    
    log_info "Testing database connection: $db_url"
    
    cd "$ADX_CORE_DIR"
    
    if cargo run --bin db-manager -- health --database-url "$db_url" 2>/dev/null; then
        log_success "Database connection successful"
        
        # Get database statistics
        log_info "Database statistics:"
        cargo run --bin db-manager -- stats --database-url "$db_url" 2>/dev/null || true
        
        # Check connection pool
        log_info "Connection pool status:"
        cargo run --bin db-manager -- monitor-pool --database-url "$db_url" 2>/dev/null || true
        
    else
        log_error "Database connection failed"
        
        # Check if PostgreSQL is running
        if check_service_health "postgres" "5432"; then
            log_info "PostgreSQL is running, checking configuration..."
            
            # Try to connect with psql
            if command -v psql &> /dev/null; then
                log_info "Testing direct psql connection..."
                if echo "SELECT version();" | psql "$db_url" 2>/dev/null; then
                    log_success "Direct psql connection works"
                else
                    log_error "Direct psql connection failed"
                fi
            fi
        else
            log_error "PostgreSQL is not running or not accessible"
        fi
    fi
}

check_redis_connectivity() {
    log "=== Redis Connectivity Check ==="
    
    local redis_url="${REDIS_URL:-redis://localhost:6379}"
    
    log_info "Testing Redis connection: $redis_url"
    
    if check_service_health "redis" "6379"; then
        log_success "Redis is accessible"
        
        # Test Redis operations
        if command -v redis-cli &> /dev/null; then
            log_info "Testing Redis operations..."
            
            # Test basic operations
            if redis-cli ping | grep -q "PONG"; then
                log_success "Redis PING successful"
                
                # Get Redis info
                log_info "Redis info:"
                redis-cli info server | head -10
                
                # Check memory usage
                log_info "Redis memory usage:"
                redis-cli info memory | grep "used_memory_human"
                
                # Check connected clients
                log_info "Connected clients:"
                redis-cli info clients | grep "connected_clients"
                
            else
                log_error "Redis PING failed"
            fi
        else
            log_warning "redis-cli not available for detailed testing"
        fi
    else
        log_error "Redis is not accessible"
    fi
}

check_temporal_connectivity() {
    log "=== Temporal Connectivity Check ==="
    
    local temporal_url="${TEMPORAL_SERVER_URL:-localhost:7233}"
    local temporal_ui_url="http://localhost:8088"
    
    log_info "Testing Temporal server: $temporal_url"
    
    if check_service_health "temporal" "7233"; then
        log_success "Temporal server is accessible"
        
        # Check Temporal UI
        if check_service_health "temporal-ui" "8088" "/api/v1/namespaces"; then
            log_success "Temporal UI is accessible at $temporal_ui_url"
            
            # Get namespace information
            log_info "Temporal namespaces:"
            curl -s "$temporal_ui_url/api/v1/namespaces" | jq -r '.namespaces[].namespaceInfo.name' 2>/dev/null || \
                curl -s "$temporal_ui_url/api/v1/namespaces" 2>/dev/null || \
                log_warning "Could not retrieve namespace information"
            
            # Check workflow executions
            log_info "Recent workflow executions:"
            curl -s "$temporal_ui_url/api/v1/namespaces/default/workflows" | \
                jq -r '.executions[0:5][] | "\(.workflowExecutionInfo.execution.workflowId) - \(.workflowExecutionInfo.type.name)"' 2>/dev/null || \
                log_info "No recent workflows or unable to parse"
                
        else
            log_warning "Temporal UI is not accessible"
        fi
    else
        log_error "Temporal server is not accessible"
    fi
}

debug_backend_services() {
    log "=== Backend Services Debug ==="
    
    cd "$ADX_CORE_DIR"
    
    for service in "${!BACKEND_SERVICES[@]}"; do
        if [[ -n "$SERVICE_FILTER" ]] && [[ "$service" != *"$SERVICE_FILTER"* ]]; then
            continue
        fi
        
        local port="${BACKEND_SERVICES[$service]}"
        
        echo
        log_info "Debugging $service (port $port)..."
        
        # Check if service directory exists
        if [[ -d "services/$service" ]]; then
            log_info "$service directory exists"
            
            # Check if service is in workspace
            if grep -q "services/$service" Cargo.toml; then
                log_success "$service is in workspace"
                
                # Try to build the service
                log_info "Testing $service build..."
                if cargo check --package "${service//-/_}" 2>/dev/null; then
                    log_success "$service builds successfully"
                else
                    log_error "$service has build errors"
                    cargo check --package "${service//-/_}" 2>&1 | tail -10
                fi
            else
                log_warning "$service is not in workspace (commented out)"
            fi
        else
            log_error "$service directory not found"
        fi
        
        # Check service health
        check_service_health "$service" "$port"
        
        # Get service logs
        get_service_logs "$service" 20
    done
}

debug_frontend_services() {
    log "=== Frontend Services Debug ==="
    
    cd "$PROJECT_ROOT"
    
    for service in "${!FRONTEND_SERVICES[@]}"; do
        if [[ -n "$SERVICE_FILTER" ]] && [[ "$service" != *"$SERVICE_FILTER"* ]]; then
            continue
        fi
        
        local port="${FRONTEND_SERVICES[$service]}"
        
        echo
        log_info "Debugging $service frontend (port $port)..."
        
        # Check if app directory exists
        if [[ -d "apps/$service" ]]; then
            log_info "$service app directory exists"
            
            # Check package.json
            if [[ -f "apps/$service/package.json" ]]; then
                log_success "$service has package.json"
                
                # Check dependencies
                cd "apps/$service"
                if [[ -d "node_modules" ]]; then
                    log_success "$service dependencies installed"
                else
                    log_warning "$service dependencies not installed"
                fi
                
                # Check if service can build
                log_info "Testing $service build..."
                if npm run build > /dev/null 2>&1; then
                    log_success "$service builds successfully"
                else
                    log_error "$service has build errors"
                fi
                
                cd "$PROJECT_ROOT"
            else
                log_error "$service missing package.json"
            fi
        else
            log_error "$service app directory not found"
        fi
        
        # Check service health
        check_service_health "$service" "$port"
    done
}

debug_bff_services() {
    log "=== BFF Services Debug ==="
    
    cd "$PROJECT_ROOT"
    
    for service in "${!BFF_SERVICES[@]}"; do
        if [[ -n "$SERVICE_FILTER" ]] && [[ "$service" != *"$SERVICE_FILTER"* ]]; then
            continue
        fi
        
        local port="${BFF_SERVICES[$service]}"
        
        echo
        log_info "Debugging $service (port $port)..."
        
        # Check if BFF directory exists
        if [[ -d "bff-services/$service" ]]; then
            log_info "$service BFF directory exists"
            
            # Check Cargo.toml for Rust BFF or package.json for Node.js BFF
            if [[ -f "bff-services/$service/Cargo.toml" ]]; then
                log_success "$service is a Rust BFF service"
                
                cd "bff-services/$service"
                if cargo check 2>/dev/null; then
                    log_success "$service builds successfully"
                else
                    log_error "$service has build errors"
                fi
                cd "$PROJECT_ROOT"
                
            elif [[ -f "bff-services/$service/package.json" ]]; then
                log_success "$service is a Node.js BFF service"
                
                cd "bff-services/$service"
                if [[ -d "node_modules" ]]; then
                    log_success "$service dependencies installed"
                else
                    log_warning "$service dependencies not installed"
                fi
                cd "$PROJECT_ROOT"
            else
                log_warning "$service BFF configuration not found"
            fi
        else
            log_warning "$service BFF directory not found"
        fi
        
        # Check service health
        check_service_health "$service" "$port"
    done
}

debug_infrastructure_services() {
    log "=== Infrastructure Services Debug ==="
    
    for service in "${!INFRASTRUCTURE_SERVICES[@]}"; do
        if [[ -n "$SERVICE_FILTER" ]] && [[ "$service" != *"$SERVICE_FILTER"* ]]; then
            continue
        fi
        
        local port="${INFRASTRUCTURE_SERVICES[$service]}"
        
        echo
        log_info "Debugging $service infrastructure (port $port)..."
        
        # Check service health
        check_service_health "$service" "$port"
        
        # Get Docker container status
        local container_name="adx-core-$service"
        if docker ps --format "table {{.Names}}" | grep -q "$service"; then
            log_success "$service container is running"
            
            # Get container stats
            log_info "$service container stats:"
            docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep "$service" || true
            
            # Get container logs
            log_info "$service recent logs:"
            docker logs --tail 10 $(docker ps -q -f name="$service") 2>/dev/null || true
        else
            log_error "$service container is not running"
            
            # Check if container exists but is stopped
            if docker ps -a --format "table {{.Names}}" | grep -q "$service"; then
                log_warning "$service container exists but is stopped"
                docker ps -a --format "table {{.Names}}\t{{.Status}}" | grep "$service"
            else
                log_error "$service container does not exist"
            fi
        fi
    done
}

analyze_system_resources() {
    log "=== System Resource Analysis ==="
    
    # CPU usage
    log_info "CPU usage:"
    top -l 1 -n 0 | grep "CPU usage" || \
    grep 'cpu ' /proc/stat | awk '{usage=($2+$4)*100/($2+$4+$5)} END {print "CPU Usage: " usage "%"}' 2>/dev/null || \
    log_warning "Could not determine CPU usage"
    
    # Memory usage
    log_info "Memory usage:"
    if command -v free &> /dev/null; then
        free -h
    elif [[ -f /proc/meminfo ]]; then
        grep -E "MemTotal|MemAvailable|MemFree" /proc/meminfo
    else
        vm_stat | head -5 2>/dev/null || log_warning "Could not determine memory usage"
    fi
    
    # Disk usage
    log_info "Disk usage:"
    df -h . 2>/dev/null || log_warning "Could not determine disk usage"
    
    # Network connections
    log_info "Network connections on ADX Core ports:"
    netstat -an 2>/dev/null | grep -E ":(3000|3001|3002|3003|3004|3005|4001|4002|4003|4004|4005|5432|6379|7233|8080|8081|8082|8083|8084|8085|8086|8087|8088)" | head -20 || \
    ss -tuln 2>/dev/null | grep -E ":(3000|3001|3002|3003|3004|3005|4001|4002|4003|4004|4005|5432|6379|7233|8080|8081|8082|8083|8084|8085|8086|8087|8088)" | head -20 || \
    log_warning "Could not check network connections"
    
    # Docker resource usage
    if command -v docker &> /dev/null; then
        log_info "Docker system info:"
        docker system df 2>/dev/null || log_warning "Could not get Docker system info"
    fi
}

generate_debug_report() {
    log "=== Generating Debug Report ==="
    
    local report_file="$PROJECT_ROOT/debug-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX Core Debug Report

**Generated:** $(date)
**Debug Session:** $DEBUG_LOG

## System Information

- **OS:** $(uname -s) $(uname -r)
- **Architecture:** $(uname -m)
- **Hostname:** $(hostname)
- **User:** $(whoami)

## Service Status Summary

### Backend Services
EOF

    for service in "${!BACKEND_SERVICES[@]}"; do
        local port="${BACKEND_SERVICES[$service]}"
        if nc -z localhost "$port" 2>/dev/null; then
            echo "- ✅ **$service** (port $port): Running" >> "$report_file"
        else
            echo "- ❌ **$service** (port $port): Not accessible" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

### Frontend Services
EOF

    for service in "${!FRONTEND_SERVICES[@]}"; do
        local port="${FRONTEND_SERVICES[$service]}"
        if nc -z localhost "$port" 2>/dev/null; then
            echo "- ✅ **$service** (port $port): Running" >> "$report_file"
        else
            echo "- ❌ **$service** (port $port): Not accessible" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

### Infrastructure Services
EOF

    for service in "${!INFRASTRUCTURE_SERVICES[@]}"; do
        local port="${INFRASTRUCTURE_SERVICES[$service]}"
        if nc -z localhost "$port" 2>/dev/null; then
            echo "- ✅ **$service** (port $port): Running" >> "$report_file"
        else
            echo "- ❌ **$service** (port $port): Not accessible" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

## Environment Variables

- **DATABASE_URL:** ${DATABASE_URL:-"Not set"}
- **REDIS_URL:** ${REDIS_URL:-"Not set"}
- **TEMPORAL_SERVER_URL:** ${TEMPORAL_SERVER_URL:-"Not set"}
- **NODE_ENV:** ${NODE_ENV:-"Not set"}
- **RUST_LOG:** ${RUST_LOG:-"Not set"}

## Debug Log

Full debug session log: \`$DEBUG_LOG\`

## Recommendations

EOF

    # Add recommendations based on findings
    local recommendations_added=false
    
    # Check for common issues
    if ! nc -z localhost 5432 2>/dev/null; then
        echo "- 🔧 **Start PostgreSQL**: Run \`docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d postgres\`" >> "$report_file"
        recommendations_added=true
    fi
    
    if ! nc -z localhost 6379 2>/dev/null; then
        echo "- 🔧 **Start Redis**: Run \`docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d redis\`" >> "$report_file"
        recommendations_added=true
    fi
    
    if ! nc -z localhost 7233 2>/dev/null; then
        echo "- 🔧 **Start Temporal**: Run \`docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d temporal\`" >> "$report_file"
        recommendations_added=true
    fi
    
    if [[ "$recommendations_added" == "false" ]]; then
        echo "- ✅ **All infrastructure services appear to be running**" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "Generated by ADX Core Debug Tool" >> "$report_file"
    
    log_success "Debug report generated: $report_file"
}

show_service_urls() {
    log "=== Service URLs ==="
    
    echo
    echo "🌐 ADX Core Service URLs:"
    echo "========================="
    
    echo
    echo "Backend Services:"
    for service in "${!BACKEND_SERVICES[@]}"; do
        local port="${BACKEND_SERVICES[$service]}"
        local status="❌"
        if nc -z localhost "$port" 2>/dev/null; then
            status="✅"
        fi
        printf "  %s %-20s http://localhost:%s\n" "$status" "$service" "$port"
    done
    
    echo
    echo "Frontend Services:"
    for service in "${!FRONTEND_SERVICES[@]}"; do
        local port="${FRONTEND_SERVICES[$service]}"
        local status="❌"
        if nc -z localhost "$port" 2>/dev/null; then
            status="✅"
        fi
        printf "  %s %-20s http://localhost:%s\n" "$status" "$service" "$port"
    done
    
    echo
    echo "BFF Services:"
    for service in "${!BFF_SERVICES[@]}"; do
        local port="${BFF_SERVICES[$service]}"
        local status="❌"
        if nc -z localhost "$port" 2>/dev/null; then
            status="✅"
        fi
        printf "  %s %-20s http://localhost:%s\n" "$status" "$service" "$port"
    done
    
    echo
    echo "Infrastructure:"
    for service in "${!INFRASTRUCTURE_SERVICES[@]}"; do
        local port="${INFRASTRUCTURE_SERVICES[$service]}"
        local status="❌"
        if nc -z localhost "$port" 2>/dev/null; then
            status="✅"
        fi
        
        local url="localhost:$port"
        if [[ "$service" == "temporal-ui" ]]; then
            url="http://localhost:$port"
        fi
        
        printf "  %s %-20s %s\n" "$status" "$service" "$url"
    done
    
    echo
}

main() {
    log "Starting ADX Core Service Debug Session"
    log "Debug log: $DEBUG_LOG"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --follow)
                FOLLOW_LOGS=true
                shift
                ;;
            --service)
                SERVICE_FILTER="$2"
                shift 2
                ;;
            --level)
                DEBUG_LEVEL="$2"
                shift 2
                ;;
            --urls-only)
                show_service_urls
                exit 0
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --verbose       Enable verbose output"
                echo "  --follow        Follow logs in real-time"
                echo "  --service NAME  Filter to specific service"
                echo "  --level LEVEL   Set debug level (debug/info/warn/error)"
                echo "  --urls-only     Show service URLs and exit"
                echo "  --help          Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Show service URLs first
    show_service_urls
    
    # Run debug phases
    check_database_connectivity
    check_redis_connectivity
    check_temporal_connectivity
    
    debug_infrastructure_services
    debug_backend_services
    debug_frontend_services
    debug_bff_services
    
    analyze_system_resources
    generate_debug_report
    
    echo
    log_success "Debug session completed. Check $DEBUG_LOG for detailed logs."
    
    if [[ "$FOLLOW_LOGS" == "true" ]]; then
        log "Following logs... (Press Ctrl+C to exit)"
        tail -f "$DEBUG_LOG"
    fi
}

# Run main function
main "$@"