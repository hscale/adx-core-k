#!/bin/bash

# ============================================================================
# ADX CORE API Testing Script
# Quick API endpoint testing for development
# ============================================================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_test() {
    echo -e "${BLUE}🧪 Testing: $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_fail() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

echo "🧪 ADX CORE API Testing"
echo "======================="
echo ""

# Test 1: Health Checks
print_test "Health Checks"
echo ""

services=(
    "8080:API Gateway"
    "8081:Auth Service"
    "8082:User Service"
    "8083:File Service"
    "8084:Workflow Service"
    "8085:Tenant Service"
)

for service in "${services[@]}"; do
    port=$(echo $service | cut -d: -f1)
    name=$(echo $service | cut -d: -f2)
    
    response=$(curl -s -w "%{http_code}" http://localhost:$port/health)
    http_code="${response: -3}"
    
    if [ "$http_code" = "200" ]; then
        print_success "$name health check passed"
    else
        print_fail "$name health check failed (HTTP $http_code)"
    fi
done

echo ""

# Test 2: Authentication
print_test "Authentication Flow"
echo ""

print_info "Testing login endpoint..."
auth_response=$(curl -s -w "%{http_code}" -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}')

http_code="${auth_response: -3}"
response_body="${auth_response%???}"

if [ "$http_code" = "200" ] && [[ "$response_body" == *"access_token"* ]]; then
    print_success "Authentication login successful"
    # Extract token for further tests
    TOKEN=$(echo $response_body | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
    print_info "Token extracted: ${TOKEN:0:20}..."
else
    print_fail "Authentication login failed (HTTP $http_code)"
    echo "Response: $response_body"
fi

echo ""

# Test 3: API Gateway Routing
print_test "API Gateway Routing"
echo ""

print_info "Testing user service routing through gateway..."
user_response=$(curl -s -w "%{http_code}" http://localhost:8080/api/v1/users)
http_code="${user_response: -3}"

if [ "$http_code" = "200" ] || [ "$http_code" = "401" ]; then
    print_success "API Gateway routing to user service working"
else
    print_fail "API Gateway routing failed (HTTP $http_code)"
fi

print_info "Testing auth service routing through gateway..."
auth_gateway_response=$(curl -s -w "%{http_code}" http://localhost:8080/api/v1/auth/health)
http_code="${auth_gateway_response: -3}"

if [ "$http_code" = "200" ] || [ "$http_code" = "404" ]; then
    print_success "API Gateway routing to auth service accessible"
else
    print_fail "API Gateway routing to auth service failed (HTTP $http_code)"
fi

echo ""

# Test 4: Infrastructure Services
print_test "Infrastructure Services"
echo ""

print_info "Testing PostgreSQL connection..."
if docker exec docker-postgres-1 pg_isready -U adx_user -d adx_core > /dev/null 2>&1; then
    print_success "PostgreSQL is ready"
else
    print_fail "PostgreSQL connection failed"
fi

print_info "Testing Redis connection..."
if docker exec docker-redis-1 redis-cli ping | grep -q PONG; then
    print_success "Redis is ready"
else
    print_fail "Redis connection failed"
fi

print_info "Testing Temporal server..."
temporal_response=$(curl -s -w "%{http_code}" http://localhost:7233/api/v1/namespaces)
http_code="${temporal_response: -3}"

if [ "$http_code" = "200" ]; then
    print_success "Temporal server is ready"
else
    print_fail "Temporal server connection failed (HTTP $http_code)"
fi

print_info "Testing Temporal UI..."
temporal_ui_response=$(curl -s -w "%{http_code}" http://localhost:8088)
http_code="${temporal_ui_response: -3}"

if [ "$http_code" = "200" ]; then
    print_success "Temporal UI is accessible"
else
    print_fail "Temporal UI connection failed (HTTP $http_code)"
fi

echo ""

# Test 5: Service Metrics (if available)
print_test "Service Metrics"
echo ""

print_info "Testing auth service metrics..."
metrics_response=$(curl -s -w "%{http_code}" http://localhost:8081/metrics)
http_code="${metrics_response: -3}"

if [ "$http_code" = "200" ]; then
    print_success "Auth service metrics available"
else
    print_info "Auth service metrics not configured (HTTP $http_code)"
fi

echo ""

# Summary
echo "📊 Test Summary"
echo "==============="
echo ""
echo "🌐 Service URLs:"
echo "  • API Gateway:      http://localhost:8080"
echo "  • Auth Service:     http://localhost:8081"
echo "  • User Service:     http://localhost:8082"
echo "  • File Service:     http://localhost:8083"
echo "  • Workflow Service: http://localhost:8084"
echo "  • Tenant Service:   http://localhost:8085"
echo "  • Temporal UI:      http://localhost:8088"
echo ""
echo "🔧 Infrastructure:"
echo "  • PostgreSQL:       localhost:5432"
echo "  • Redis:            localhost:6379"
echo "  • Temporal:         localhost:7233"
echo ""
echo "📋 Useful Commands:"
echo "  • View logs:        tail -f logs/auth-service.log"
echo "  • Stop services:    pkill -f 'cargo run'"
echo "  • Restart infra:    docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml restart"
echo ""

print_success "API testing complete!"