#!/bin/bash

# ADX CORE Health Check Script
# This script checks if all services are running and healthy

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[CHECK]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Function to check HTTP endpoint
check_http() {
    local url=$1
    local service_name=$2
    local timeout=${3:-5}
    
    if curl -s --max-time $timeout "$url" > /dev/null 2>&1; then
        print_success "$service_name is responding at $url"
        return 0
    else
        print_error "$service_name is not responding at $url"
        return 1
    fi
}

# Function to check TCP port
check_tcp() {
    local host=$1
    local port=$2
    local service_name=$3
    
    if nc -z "$host" "$port" 2>/dev/null; then
        print_success "$service_name is listening on $host:$port"
        return 0
    else
        print_error "$service_name is not listening on $host:$port"
        return 1
    fi
}

echo "🏥 ADX CORE Health Check"
echo "========================"

failed_checks=0

# Check infrastructure services
echo ""
echo "🏗️  Infrastructure Services:"
check_tcp localhost 5432 "PostgreSQL" || failed_checks=$((failed_checks + 1))
check_tcp localhost 6379 "Redis" || failed_checks=$((failed_checks + 1))
check_tcp localhost 7233 "Temporal Server" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8088" "Temporal UI" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:9090" "Prometheus" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3001" "Grafana" || failed_checks=$((failed_checks + 1))

# Check backend microservices
echo ""
echo "🔧 Backend Microservices:"
check_http "http://localhost:8080/health" "API Gateway" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8081/health" "Auth Service" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8082/health" "User Service" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8083/health" "File Service" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8084/health" "Workflow Service" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8085/health" "Tenant Service" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:8086/health" "Module Service" || failed_checks=$((failed_checks + 1))

# Check BFF services
echo ""
echo "🌐 BFF Services:"
check_http "http://localhost:4001/health" "Auth BFF" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:4002/health" "Tenant BFF" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:4003/health" "File BFF" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:4004/health" "User BFF" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:4005/health" "Workflow BFF" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:4006/health" "Module BFF" || failed_checks=$((failed_checks + 1))

# Check frontend micro-apps
echo ""
echo "🎨 Frontend Micro-Apps:"
check_http "http://localhost:3000" "Shell Application" || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3001" "Auth Micro-App" 10 || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3002" "Tenant Micro-App" 10 || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3003" "File Micro-App" 10 || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3004" "User Micro-App" 10 || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3005" "Workflow Micro-App" 10 || failed_checks=$((failed_checks + 1))
check_http "http://localhost:3006" "Module Micro-App" 10 || failed_checks=$((failed_checks + 1))

echo ""
echo "========================"

if [ $failed_checks -eq 0 ]; then
    print_success "All services are healthy! 🎉"
    echo ""
    echo "🌟 You can now access:"
    echo "  - Main Application: http://localhost:3000"
    echo "  - Temporal UI: http://localhost:8088"
    echo "  - Grafana: http://localhost:3001 (admin/admin)"
    echo "  - Prometheus: http://localhost:9090"
    exit 0
else
    print_error "$failed_checks service(s) failed health checks"
    echo ""
    echo "💡 Troubleshooting tips:"
    echo "  - Check service logs in ./logs/ directory"
    echo "  - Ensure Docker Desktop is running"
    echo "  - Wait a few more minutes for services to fully start"
    echo "  - Run 'docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml ps' to check container status"
    exit 1
fi