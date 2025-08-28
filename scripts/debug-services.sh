#!/bin/bash

# ADX CORE Service Debugging Script
# Provides debugging utilities for all services and infrastructure components

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

print_debug() {
    echo -e "${PURPLE}[DEBUG]${NC} $1"
}

print_service() {
    echo -e "${CYAN}[SERVICE]${NC} $1"
}

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Parse command line arguments
SERVICE=""
ACTION="status"
VERBOSE=false
LOGS=false
HEALTH_CHECK=false
RESTART=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --service)
            SERVICE="$2"
            shift 2
            ;;
        --status)
            ACTION="status"
            shift
            ;;
        --logs)
            ACTION="logs"
            LOGS=true
            shift
            ;;
        --health)
            ACTION="health"
            HEALTH_CHECK=true
            shift
            ;;
        --restart)
            ACTION="restart"
            RESTART=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --service SERVICE   Debug specific service (all, backend, frontend, infrastructure)"
            echo "  --status            Show service status (default)"
            echo "  --logs              Show service logs"
            echo "  --health            Run health checks"
            echo "  --restart           Restart services"
            echo "  --verbose           Verbose output"
            echo "  --help              Show this help message"
            echo ""
            echo "Available services:"
            echo "  all                 All services"
            echo "  infrastructure      Docker infrastructure (postgres, redis, temporal)"
            echo "  backend             All backend services"
            echo "  frontend            All frontend applications"
            echo "  api-gateway         API Gateway service"
            echo "  auth-service        Authentication service"
            echo "  user-service        User management service"
            echo "  file-service        File management service"
            echo "  tenant-service      Tenant management service"
            echo "  workflow-service    Workflow orchestration service"
            echo "  shell-app           Shell application"
            echo "  auth-app            Auth micro-frontend"
            echo "  tenant-app          Tenant micro-frontend"
            echo "  file-app            File micro-frontend"
            echo "  user-app            User micro-frontend"
            echo "  workflow-app        Workflow micro-frontend"
            echo "  module-app          Module micro-frontend"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Default to all services if none specified
if [ -z "$SERVICE" ]; then
    SERVICE="all"
fi

print_status "ADX CORE Service Debugger"
print_status "Service: $SERVICE"
print_status "Action: $ACTION"

# Function to check Docker infrastructure status
check_infrastructure_status() {
    print_service "Checking infrastructure services..."
    
    # Check Docker Compose services
    if command -v docker-compose &> /dev/null; then
        print_debug "Docker Compose services status:"
        docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml ps
        
        # Check individual service health
        local services=("postgres" "redis" "temporal" "temporal-ui")
        for service in "${services[@]}"; do
            local status=$(docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml ps -q $service)
            if [ -n "$status" ]; then
                local health=$(docker inspect --format='{{.State.Health.Status}}' $status 2>/dev/null || echo "unknown")
                print_service "$service: $health"
            else
                print_warning "$service: not running"
            fi
        done
    else
        print_error "Docker Compose not available"
    fi
}

# Function to check backend service status
check_backend_status() {
    print_service "Checking backend services..."
    
    local services=("api-gateway" "auth-service" "user-service" "file-service" "tenant-service" "workflow-service")
    local ports=(8080 8081 8082 8083 8085 8084)
    
    for i in "${!services[@]}"; do
        local service="${services[$i]}"
        local port="${ports[$i]}"
        
        print_debug "Checking $service on port $port..."
        
        if netstat -tuln | grep -q ":$port "; then
            print_success "$service: running on port $port"
            
            # Try health check endpoint
            if curl -s "http://localhost:$port/health" > /dev/null 2>&1; then
                print_success "$service: health check passed"
            else
                print_warning "$service: health check failed"
            fi
        else
            print_warning "$service: not running on port $port"
        fi
    done
}

# Function to check frontend application status
check_frontend_status() {
    print_service "Checking frontend applications..."
    
    local apps=("shell" "auth" "tenant" "file" "user" "workflow" "module")
    local ports=(3000 3001 3002 3003 3004 3005 3006)
    
    for i in "${!apps[@]}"; do
        local app="${apps[$i]}"
        local port="${ports[$i]}"
        
        print_debug "Checking $app app on port $port..."
        
        if netstat -tuln | grep -q ":$port "; then
            print_success "$app app: running on port $port"
            
            # Try to access the app
            if curl -s "http://localhost:$port" > /dev/null 2>&1; then
                print_success "$app app: accessible"
            else
                print_warning "$app app: not accessible"
            fi
        else
            print_warning "$app app: not running on port $port"
        fi
    done
    
    # Check BFF services
    print_service "Checking BFF services..."
    local bff_services=("auth-bff" "tenant-bff" "file-bff" "user-bff" "workflow-bff" "module-bff")
    local bff_ports=(4001 4002 4003 4004 4005 4006)
    
    for i in "${!bff_services[@]}"; do
        local bff="${bff_services[$i]}"
        local port="${bff_ports[$i]}"
        
        print_debug "Checking $bff on port $port..."
        
        if netstat -tuln | grep -q ":$port "; then
            print_success "$bff: running on port $port"
        else
            print_warning "$bff: not running on port $port"
        fi
    done
}

# Function to show service logs
show_service_logs() {
    local service_type="$1"
    
    case $service_type in
        "infrastructure")
            print_debug "Infrastructure logs:"
            docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml logs --tail=50
            ;;
        "backend")
            print_debug "Backend service logs:"
            # Show logs from running backend services
            for service in api-gateway auth-service user-service file-service tenant-service workflow-service; do
                if pgrep -f "$service" > /dev/null; then
                    print_service "Logs for $service:"
                    # In a real implementation, you'd show actual service logs
                    # For now, show process information
                    ps aux | grep "$service" | grep -v grep
                fi
            done
            ;;
        "frontend")
            print_debug "Frontend application logs:"
            # Show logs from running frontend processes
            for app in shell auth tenant file user workflow module; do
                if pgrep -f "vite.*$app" > /dev/null; then
                    print_service "Logs for $app app:"
                    ps aux | grep "vite.*$app" | grep -v grep
                fi
            done
            ;;
        *)
            print_warning "Unknown service type for logs: $service_type"
            ;;
    esac
}

# Function to run health checks
run_health_checks() {
    print_service "Running comprehensive health checks..."
    
    # Database connectivity
    print_debug "Testing database connectivity..."
    if docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then
        print_success "PostgreSQL: healthy"
    else
        print_error "PostgreSQL: unhealthy"
    fi
    
    # Redis connectivity
    print_debug "Testing Redis connectivity..."
    if docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1; then
        print_success "Redis: healthy"
    else
        print_error "Redis: unhealthy"
    fi
    
    # Temporal connectivity
    print_debug "Testing Temporal connectivity..."
    if curl -s http://localhost:8088/health > /dev/null 2>&1; then
        print_success "Temporal: healthy"
    else
        print_error "Temporal: unhealthy"
    fi
    
    # API Gateway health
    print_debug "Testing API Gateway health..."
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        print_success "API Gateway: healthy"
    else
        print_error "API Gateway: unhealthy"
    fi
    
    # Shell application health
    print_debug "Testing Shell application health..."
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        print_success "Shell application: healthy"
    else
        print_error "Shell application: unhealthy"
    fi
}

# Function to restart services
restart_services() {
    local service_type="$1"
    
    case $service_type in
        "infrastructure")
            print_service "Restarting infrastructure services..."
            docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml restart
            ;;
        "backend")
            print_service "Restarting backend services..."
            # Kill existing backend processes
            pkill -f "api-gateway\|auth-service\|user-service\|file-service\|tenant-service\|workflow-service" 2>/dev/null || true
            
            # Start backend services
            print_status "Starting backend services..."
            cd adx-core
            cargo run --bin api-gateway &
            cargo run --bin auth-service &
            cargo run --bin user-service &
            cargo run --bin file-service &
            cargo run --bin tenant-service &
            cargo run --bin workflow-service &
            cd ..
            ;;
        "frontend")
            print_service "Restarting frontend applications..."
            # Kill existing frontend processes
            pkill -f "vite\|npm.*dev" 2>/dev/null || true
            
            # Start frontend applications
            print_status "Starting frontend applications..."
            ./scripts/dev-start-frontend-only.sh &
            ;;
        *)
            print_warning "Unknown service type for restart: $service_type"
            ;;
    esac
}

# Function to debug specific service
debug_specific_service() {
    local service="$1"
    local action="$2"
    
    print_service "Debugging $service..."
    
    case $action in
        "status")
            case $service in
                "api-gateway")
                    if netstat -tuln | grep -q ":8080 "; then
                        print_success "API Gateway: running on port 8080"
                        curl -s http://localhost:8080/health || print_warning "Health check failed"
                    else
                        print_error "API Gateway: not running"
                    fi
                    ;;
                "auth-service")
                    if netstat -tuln | grep -q ":8081 "; then
                        print_success "Auth Service: running on port 8081"
                    else
                        print_error "Auth Service: not running"
                    fi
                    ;;
                "shell-app")
                    if netstat -tuln | grep -q ":3000 "; then
                        print_success "Shell App: running on port 3000"
                        curl -s http://localhost:3000 > /dev/null || print_warning "Not accessible"
                    else
                        print_error "Shell App: not running"
                    fi
                    ;;
                *)
                    print_warning "Status check not implemented for $service"
                    ;;
            esac
            ;;
        "logs")
            print_debug "Showing logs for $service..."
            # Implementation would show actual service logs
            ps aux | grep "$service" | grep -v grep || print_warning "Service not found in process list"
            ;;
        "health")
            print_debug "Running health check for $service..."
            # Implementation would run service-specific health checks
            ;;
        "restart")
            print_debug "Restarting $service..."
            # Implementation would restart specific service
            ;;
    esac
}

# Function to generate debug report
generate_debug_report() {
    print_status "Generating debug report..."
    
    mkdir -p target/debug-reports
    local report_file="target/debug-reports/debug_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX CORE Debug Report

**Generated:** $(date)  
**System:** $(uname -a)  
**User:** $(whoami)

## System Information
- **OS:** $(uname -s)
- **Architecture:** $(uname -m)
- **Kernel:** $(uname -r)
- **Docker:** $(docker --version 2>/dev/null || echo "Not installed")
- **Node.js:** $(node --version 2>/dev/null || echo "Not installed")
- **Rust:** $(rustc --version 2>/dev/null || echo "Not installed")

## Port Usage
\`\`\`
$(netstat -tuln | grep -E ":(3000|3001|3002|3003|3004|3005|3006|4001|4002|4003|4004|4005|4006|8080|8081|8082|8083|8084|8085|5432|6379|7233|8088)" || echo "No relevant ports in use")
\`\`\`

## Running Processes
\`\`\`
$(ps aux | grep -E "vite|cargo|node|npm|docker" | grep -v grep || echo "No relevant processes found")
\`\`\`

## Docker Containers
\`\`\`
$(docker ps -a 2>/dev/null || echo "Docker not available")
\`\`\`

## Disk Space
\`\`\`
$(df -h | head -5)
\`\`\`

## Memory Usage
\`\`\`
$(free -h 2>/dev/null || echo "Memory info not available")
\`\`\`

## Network Connectivity
- **Localhost connectivity:** $(curl -s http://localhost:3000 > /dev/null && echo "✅ OK" || echo "❌ Failed")
- **Database connectivity:** $(docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1 && echo "✅ OK" || echo "❌ Failed")
- **Redis connectivity:** $(docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1 && echo "✅ OK" || echo "❌ Failed")

## Recommendations
EOF

    # Add recommendations based on findings
    if ! netstat -tuln | grep -q ":3000 "; then
        echo "- Start the shell application: \`cd adx-core/apps/shell && npm run dev\`" >> "$report_file"
    fi
    
    if ! docker ps | grep -q postgres; then
        echo "- Start infrastructure services: \`docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d\`" >> "$report_file"
    fi
    
    print_success "Debug report generated: $report_file"
}

# Main execution logic
case $ACTION in
    "status")
        case $SERVICE in
            "all")
                check_infrastructure_status
                check_backend_status
                check_frontend_status
                ;;
            "infrastructure")
                check_infrastructure_status
                ;;
            "backend")
                check_backend_status
                ;;
            "frontend")
                check_frontend_status
                ;;
            *)
                debug_specific_service "$SERVICE" "status"
                ;;
        esac
        ;;
    "logs")
        case $SERVICE in
            "all")
                show_service_logs "infrastructure"
                show_service_logs "backend"
                show_service_logs "frontend"
                ;;
            *)
                show_service_logs "$SERVICE"
                ;;
        esac
        ;;
    "health")
        run_health_checks
        ;;
    "restart")
        case $SERVICE in
            "all")
                restart_services "infrastructure"
                restart_services "backend"
                restart_services "frontend"
                ;;
            *)
                restart_services "$SERVICE"
                ;;
        esac
        ;;
esac

# Generate debug report if verbose mode
if [ "$VERBOSE" = true ]; then
    generate_debug_report
fi

print_success "Service debugging completed"