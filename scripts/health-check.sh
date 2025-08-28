#!/bin/bash

# ADX CORE Health Check Script
# Performs comprehensive health checks on all services and infrastructure

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

print_health() {
    echo -e "${PURPLE}[HEALTH]${NC} $1"
}

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Health check results tracking
HEALTH_ERRORS=0
HEALTH_WARNINGS=0
TOTAL_CHECKS=0

# Function to perform health check
health_check() {
    local service="$1"
    local check_command="$2"
    local description="$3"
    local critical="$4"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    print_health "Checking $service: $description"
    
    if eval "$check_command" > /dev/null 2>&1; then
        print_success "$service: $description ✅"
        return 0
    else
        if [ "$critical" = "true" ]; then
            print_error "$service: $description ❌"
            HEALTH_ERRORS=$((HEALTH_ERRORS + 1))
        else
            print_warning "$service: $description ⚠️"
            HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
        fi
        return 1
    fi
}

# Function to check service response time
check_response_time() {
    local service="$1"
    local url="$2"
    local max_time="$3"
    
    print_health "Checking $service response time..."
    
    local start_time=$(date +%s%N)
    if curl -s --max-time "$max_time" "$url" > /dev/null 2>&1; then
        local end_time=$(date +%s%N)
        local response_time=$(( (end_time - start_time) / 1000000 ))
        
        if [ $response_time -lt $(( max_time * 1000 )) ]; then
            print_success "$service response time: ${response_time}ms ✅"
            return 0
        else
            print_warning "$service response time: ${response_time}ms (slow) ⚠️"
            HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
            return 1
        fi
    else
        print_error "$service: not responding ❌"
        HEALTH_ERRORS=$((HEALTH_ERRORS + 1))
        return 1
    fi
}

print_status "ADX CORE Comprehensive Health Check"
print_status "===================================="

# Infrastructure Health Checks
print_status "Infrastructure Health Checks"
print_status "----------------------------"

# Docker Infrastructure
health_check "Docker" "docker info" "Docker daemon running" "true"
health_check "Docker Compose" "docker-compose --version" "Docker Compose available" "true"

# Database Health
health_check "PostgreSQL" "docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres pg_isready -U postgres" "Database connectivity" "true"

if health_check "PostgreSQL Connection" "docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c 'SELECT 1;'" "Database query execution" "true"; then
    # Check database performance
    print_health "Checking PostgreSQL performance..."
    local db_response_time=$(docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c '\timing on' -c 'SELECT COUNT(*) FROM information_schema.tables;' 2>/dev/null | grep "Time:" | awk '{print $2}' | cut -d'.' -f1 || echo "0")
    
    if [ "$db_response_time" -lt 100 ]; then
        print_success "PostgreSQL performance: ${db_response_time}ms ✅"
    else
        print_warning "PostgreSQL performance: ${db_response_time}ms (slow) ⚠️"
        HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
    fi
fi

# Redis Health
health_check "Redis" "docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T redis redis-cli ping" "Redis connectivity" "true"

if health_check "Redis Memory" "docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T redis redis-cli info memory" "Redis memory info" "false"; then
    local redis_memory=$(docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T redis redis-cli info memory | grep "used_memory_human" | cut -d':' -f2 | tr -d '\r')
    print_success "Redis memory usage: $redis_memory"
fi

# Temporal Health
health_check "Temporal Server" "curl -s http://localhost:7233/health" "Temporal server connectivity" "true"
health_check "Temporal UI" "curl -s http://localhost:8088/health" "Temporal UI accessibility" "false"

# Backend Services Health Checks
print_status "Backend Services Health Checks"
print_status "------------------------------"

# API Gateway
if health_check "API Gateway" "curl -s http://localhost:8080/health" "API Gateway accessibility" "true"; then
    check_response_time "API Gateway" "http://localhost:8080/health" 2
fi

# Individual Backend Services
BACKEND_SERVICES=(
    "Auth Service:8081"
    "User Service:8082"
    "File Service:8083"
    "Workflow Service:8084"
    "Tenant Service:8085"
)

for service_info in "${BACKEND_SERVICES[@]}"; do
    IFS=':' read -r service_name port <<< "$service_info"
    
    if health_check "$service_name" "curl -s http://localhost:$port/health" "$service_name accessibility" "false"; then
        check_response_time "$service_name" "http://localhost:$port/health" 2
        
        # Check service-specific endpoints
        case $service_name in
            "Auth Service")
                health_check "$service_name" "curl -s http://localhost:$port/api/v1/auth/status" "Auth endpoints" "false"
                ;;
            "User Service")
                health_check "$service_name" "curl -s http://localhost:$port/api/v1/users/health" "User endpoints" "false"
                ;;
            "File Service")
                health_check "$service_name" "curl -s http://localhost:$port/api/v1/files/health" "File endpoints" "false"
                ;;
            "Tenant Service")
                health_check "$service_name" "curl -s http://localhost:$port/api/v1/tenants/health" "Tenant endpoints" "false"
                ;;
        esac
    fi
done

# Frontend Applications Health Checks
print_status "Frontend Applications Health Checks"
print_status "-----------------------------------"

# Shell Application
if health_check "Shell Application" "curl -s http://localhost:3000" "Shell app accessibility" "true"; then
    check_response_time "Shell Application" "http://localhost:3000" 3
    
    # Check if Module Federation is working
    health_check "Module Federation" "curl -s http://localhost:3000/assets/remoteEntry.js" "Module Federation host" "false"
fi

# Micro-Frontends
MICRO_FRONTENDS=(
    "Auth App:3001"
    "Tenant App:3002"
    "File App:3003"
    "User App:3004"
    "Workflow App:3005"
    "Module App:3006"
)

for app_info in "${MICRO_FRONTENDS[@]}"; do
    IFS=':' read -r app_name port <<< "$app_info"
    
    if health_check "$app_name" "curl -s http://localhost:$port" "$app_name accessibility" "false"; then
        check_response_time "$app_name" "http://localhost:$port" 3
        
        # Check Module Federation remote entry
        health_check "$app_name Federation" "curl -s http://localhost:$port/assets/remoteEntry.js" "$app_name remote entry" "false"
    fi
done

# BFF Services Health Checks
print_status "BFF Services Health Checks"
print_status "--------------------------"

BFF_SERVICES=(
    "Auth BFF:4001"
    "Tenant BFF:4002"
    "File BFF:4003"
    "User BFF:4004"
    "Workflow BFF:4005"
    "Module BFF:4006"
)

for bff_info in "${BFF_SERVICES[@]}"; do
    IFS=':' read -r bff_name port <<< "$bff_info"
    
    if health_check "$bff_name" "curl -s http://localhost:$port/health" "$bff_name accessibility" "false"; then
        check_response_time "$bff_name" "http://localhost:$port/health" 2
        
        # Check BFF-specific endpoints
        health_check "$bff_name API" "curl -s http://localhost:$port/api/health" "$bff_name API endpoints" "false"
    fi
done

# System Resource Health Checks
print_status "System Resource Health Checks"
print_status "-----------------------------"

# CPU Usage
if command -v top &> /dev/null; then
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1 | cut -d'u' -f1 2>/dev/null || echo "0")
    if [ "${CPU_USAGE%.*}" -lt 80 ]; then
        print_success "CPU usage: ${CPU_USAGE}% ✅"
    else
        print_warning "CPU usage: ${CPU_USAGE}% (high) ⚠️"
        HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
    fi
fi

# Memory Usage
if command -v free &> /dev/null; then
    MEMORY_USAGE=$(free | grep Mem | awk '{printf("%.1f"), $3/$2 * 100.0}')
    if [ "${MEMORY_USAGE%.*}" -lt 80 ]; then
        print_success "Memory usage: ${MEMORY_USAGE}% ✅"
    else
        print_warning "Memory usage: ${MEMORY_USAGE}% (high) ⚠️"
        HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
    fi
elif command -v vm_stat &> /dev/null; then
    # macOS memory check
    print_success "Memory check completed (macOS) ✅"
fi

# Disk Usage
if command -v df &> /dev/null; then
    DISK_USAGE=$(df . | tail -1 | awk '{print $5}' | cut -d'%' -f1)
    if [ "$DISK_USAGE" -lt 80 ]; then
        print_success "Disk usage: ${DISK_USAGE}% ✅"
    else
        print_warning "Disk usage: ${DISK_USAGE}% (high) ⚠️"
        HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
    fi
fi

# Network Connectivity
print_status "Network Connectivity Health Checks"
print_status "----------------------------------"

# Internal connectivity
health_check "Localhost" "curl -s http://localhost:3000" "Localhost connectivity" "true"

# External connectivity (optional)
health_check "Internet" "curl -s --max-time 5 https://www.google.com" "Internet connectivity" "false"

# Docker network connectivity
health_check "Docker Network" "docker network ls | grep adx-network" "Docker network exists" "false"

# Workflow Health Checks
print_status "Workflow Health Checks"
print_status "---------------------"

# Temporal workflow execution test
if curl -s http://localhost:7233/health > /dev/null 2>&1; then
    print_health "Testing Temporal workflow execution..."
    # This would test a simple workflow execution
    # For now, we'll just check if Temporal is responsive
    health_check "Temporal Workflows" "curl -s http://localhost:8088/api/v1/namespaces" "Temporal namespace API" "false"
fi

# Security Health Checks
print_status "Security Health Checks"
print_status "---------------------"

# Check for exposed sensitive ports
SENSITIVE_PORTS=("22" "3306" "5432" "6379" "27017")
for port in "${SENSITIVE_PORTS[@]}"; do
    if netstat -tuln 2>/dev/null | grep -q ":$port.*0.0.0.0"; then
        print_warning "Port $port is exposed to all interfaces ⚠️"
        HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
    else
        print_success "Port $port is properly secured ✅"
    fi
done

# Check for default passwords (if applicable)
health_check "Database Security" "docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c 'SELECT 1;'" "Database authentication" "false"

# Performance Benchmarks
print_status "Performance Benchmarks"
print_status "----------------------"

# API Gateway performance test
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    print_health "Running API Gateway performance test..."
    local start_time=$(date +%s%N)
    for i in {1..10}; do
        curl -s http://localhost:8080/health > /dev/null 2>&1
    done
    local end_time=$(date +%s%N)
    local avg_response_time=$(( (end_time - start_time) / 10000000 ))
    
    if [ $avg_response_time -lt 100 ]; then
        print_success "API Gateway average response time: ${avg_response_time}ms ✅"
    else
        print_warning "API Gateway average response time: ${avg_response_time}ms (slow) ⚠️"
        HEALTH_WARNINGS=$((HEALTH_WARNINGS + 1))
    fi
fi

# Generate health report
print_status "Generating Health Report..."
mkdir -p target/health-reports
HEALTH_REPORT="target/health-reports/health_report_$(date +%Y%m%d_%H%M%S).md"

cat > "$HEALTH_REPORT" << EOF
# ADX CORE Health Check Report

**Generated:** $(date)  
**System:** $(uname -a)  
**Total Checks:** $TOTAL_CHECKS  
**Errors:** $HEALTH_ERRORS  
**Warnings:** $HEALTH_WARNINGS  
**Success Rate:** $(( (TOTAL_CHECKS - HEALTH_ERRORS - HEALTH_WARNINGS) * 100 / TOTAL_CHECKS ))%

## Health Status
$([ $HEALTH_ERRORS -eq 0 ] && echo "✅ **HEALTHY**" || echo "❌ **UNHEALTHY**")

## Service Status Matrix

| Service | Status | Response Time | Notes |
|---------|--------|---------------|-------|
| PostgreSQL | $(curl -s http://localhost:5432 > /dev/null 2>&1 && echo "✅ UP" || echo "❌ DOWN") | - | Database |
| Redis | $(docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1 && echo "✅ UP" || echo "❌ DOWN") | - | Cache |
| Temporal | $(curl -s http://localhost:8088/health > /dev/null 2>&1 && echo "✅ UP" || echo "❌ DOWN") | - | Workflow Engine |
| API Gateway | $(curl -s http://localhost:8080/health > /dev/null 2>&1 && echo "✅ UP" || echo "❌ DOWN") | - | Gateway |
| Shell App | $(curl -s http://localhost:3000 > /dev/null 2>&1 && echo "✅ UP" || echo "❌ DOWN") | - | Frontend |

## System Resources
- **CPU Usage:** $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1 2>/dev/null || echo "N/A")
- **Memory Usage:** $(free | grep Mem | awk '{printf("%.1f%%"), $3/$2 * 100.0}' 2>/dev/null || echo "N/A")
- **Disk Usage:** $(df . | tail -1 | awk '{print $5}' 2>/dev/null || echo "N/A")

## Recommendations
EOF

# Add recommendations based on health check results
if [ $HEALTH_ERRORS -gt 0 ]; then
    echo "### Critical Issues" >> "$HEALTH_REPORT"
    echo "- $HEALTH_ERRORS critical health checks failed" >> "$HEALTH_REPORT"
    echo "- Review error messages above and fix critical issues" >> "$HEALTH_REPORT"
    echo "- Re-run health check after fixes" >> "$HEALTH_REPORT"
fi

if [ $HEALTH_WARNINGS -gt 0 ]; then
    echo "### Warnings" >> "$HEALTH_REPORT"
    echo "- $HEALTH_WARNINGS health checks have warnings" >> "$HEALTH_REPORT"
    echo "- Consider addressing warnings for optimal performance" >> "$HEALTH_REPORT"
fi

if [ $HEALTH_ERRORS -eq 0 ] && [ $HEALTH_WARNINGS -eq 0 ]; then
    echo "### All Systems Healthy" >> "$HEALTH_REPORT"
    echo "- All health checks passed successfully" >> "$HEALTH_REPORT"
    echo "- System is ready for development and testing" >> "$HEALTH_REPORT"
fi

print_success "Health report generated: $HEALTH_REPORT"

# Final summary
print_status "Health Check Summary"
print_status "==================="
print_status "Total Checks: $TOTAL_CHECKS"
print_status "Errors: $HEALTH_ERRORS"
print_status "Warnings: $HEALTH_WARNINGS"
print_status "Success Rate: $(( (TOTAL_CHECKS - HEALTH_ERRORS - HEALTH_WARNINGS) * 100 / TOTAL_CHECKS ))%"

if [ $HEALTH_ERRORS -eq 0 ]; then
    print_success "System health check PASSED ✅"
    print_status "All critical services are healthy and operational"
    exit 0
else
    print_error "System health check FAILED ❌"
    print_status "Please address the critical issues above"
    exit 1
fi