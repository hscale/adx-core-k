#!/bin/bash

# ADX CORE - Frontend Microservices Status Check
# This script checks the health of all frontend microservices and BFF services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_service() {
    echo -e "${BLUE}[SERVICE]${NC} $1"
}

# Function to check service health
check_service() {
    local url=$1
    local name=$2
    local timeout=${3:-5}
    
    if curl -s --max-time $timeout "$url" >/dev/null 2>&1; then
        print_service "✅ $name: $url"
        return 0
    else
        print_service "❌ $name: $url (not responding)"
        return 1
    fi
}

# Function to check if port is in use
check_port() {
    local port=$1
    local name=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        print_service "✅ $name: Port $port (in use)"
        return 0
    else
        print_service "❌ $name: Port $port (not in use)"
        return 1
    fi
}

print_status "ADX CORE Frontend Microservices Status Check"
print_status "============================================="

# Check Backend Services
print_status ""
print_status "Backend Services:"
print_status "----------------"

BACKEND_SERVICES=(
    "http://localhost:8080/health|API Gateway"
    "http://localhost:8081/health|Auth Service"
    "http://localhost:8082/health|User Service"
    "http://localhost:8083/health|File Service"
    "http://localhost:8084/health|Workflow Service"
    "http://localhost:8085/health|Tenant Service"
)

backend_healthy=0
for service in "${BACKEND_SERVICES[@]}"; do
    IFS='|' read -r url name <<< "$service"
    if check_service "$url" "$name"; then
        ((backend_healthy++))
    fi
done

# Check Frontend Services
print_status ""
print_status "Frontend Services:"
print_status "-----------------"

FRONTEND_SERVICES=(
    "http://localhost:3000|Shell Application"
    "http://localhost:3001|Auth Micro-App"
    "http://localhost:3002|Tenant Micro-App"
    "http://localhost:3003|File Micro-App"
    "http://localhost:3004|User Micro-App"
    "http://localhost:3005|Workflow Micro-App"
    "http://localhost:3006|Dashboard Micro-App"
)

frontend_healthy=0
for service in "${FRONTEND_SERVICES[@]}"; do
    IFS='|' read -r url name <<< "$service"
    if check_service "$url" "$name"; then
        ((frontend_healthy++))
    fi
done

# Check BFF Services
print_status ""
print_status "BFF Services:"
print_status "------------"

BFF_SERVICES=(
    "http://localhost:4001/health|Auth BFF"
    "http://localhost:4002/health|Tenant BFF"
    "http://localhost:4003/health|File BFF"
    "http://localhost:4004/health|User BFF"
    "http://localhost:4005/health|Workflow BFF"
    "http://localhost:4006/health|Dashboard BFF"
)

bff_healthy=0
for service in "${BFF_SERVICES[@]}"; do
    IFS='|' read -r url name <<< "$service"
    if check_service "$url" "$name"; then
        ((bff_healthy++))
    fi
done

# Check Infrastructure Services
print_status ""
print_status "Infrastructure Services:"
print_status "-----------------------"

INFRA_SERVICES=(
    "http://localhost:5432|PostgreSQL"
    "http://localhost:6379|Redis"
    "http://localhost:8088|Temporal UI"
)

infra_healthy=0
for service in "${INFRA_SERVICES[@]}"; do
    IFS='|' read -r url name <<< "$service"
    # For infrastructure services, just check if port is in use
    port=$(echo "$url" | sed 's/.*://')
    if check_port "$port" "$name"; then
        ((infra_healthy++))
    fi
done

# Summary
print_status ""
print_status "Summary:"
print_status "-------"
print_service "Backend Services: $backend_healthy/6 healthy"
print_service "Frontend Services: $frontend_healthy/7 healthy"
print_service "BFF Services: $bff_healthy/6 healthy"
print_service "Infrastructure: $infra_healthy/3 healthy"

total_services=$((6 + 7 + 6 + 3))
total_healthy=$((backend_healthy + frontend_healthy + bff_healthy + infra_healthy))

print_status ""
if [ $total_healthy -eq $total_services ]; then
    print_status "🎉 All services are healthy! ($total_healthy/$total_services)"
elif [ $total_healthy -gt $((total_services / 2)) ]; then
    print_warning "⚠️  Most services are healthy ($total_healthy/$total_services)"
else
    print_error "🚨 Many services are down ($total_healthy/$total_services)"
fi

# Check for common issues
print_status ""
print_status "Troubleshooting:"
print_status "---------------"

if [ $backend_healthy -lt 6 ]; then
    print_warning "Some backend services are down. Try: cd adx-core && ./scripts/dev-start.sh"
fi

if [ $frontend_healthy -lt 7 ]; then
    print_warning "Some frontend services are down. Try: ./scripts/dev-start-frontend.sh"
fi

if [ $infra_healthy -lt 3 ]; then
    print_warning "Infrastructure services are down. Try: cd adx-core && docker compose -f infrastructure/docker/docker-compose.dev.yml up -d"
fi

# Check log files
print_status ""
print_status "Recent Logs:"
print_status "-----------"

if [ -d "logs/frontend" ]; then
    for log_file in logs/frontend/*.log; do
        if [ -f "$log_file" ]; then
            service_name=$(basename "$log_file" .log)
            last_line=$(tail -n 1 "$log_file" 2>/dev/null || echo "No logs")
            print_service "$service_name: $last_line"
        fi
    done
else
    print_warning "No frontend logs found. Services may not be running."
fi

# Performance check
print_status ""
print_status "Performance Check:"
print_status "-----------------"

if command -v curl >/dev/null 2>&1; then
    if curl -s http://localhost:3000 >/dev/null 2>&1; then
        response_time=$(curl -o /dev/null -s -w '%{time_total}' http://localhost:3000)
        if (( $(echo "$response_time < 2.0" | bc -l) )); then
            print_service "✅ Shell app response time: ${response_time}s (good)"
        else
            print_service "⚠️  Shell app response time: ${response_time}s (slow)"
        fi
    fi
fi

print_status ""
print_status "For more details, check individual service logs in logs/frontend/"
print_status "To start all services: ./scripts/dev-start-frontend.sh"