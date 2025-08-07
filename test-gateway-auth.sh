#!/bin/bash

# ============================================================================
# Test API Gateway Auth Routing
# ============================================================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_fail() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

echo -e "${BLUE}🔗 Testing API Gateway Auth Routing${NC}"
echo "===================================="
echo ""

# Test 1: Direct Auth Service
print_info "Testing direct auth service..."
direct_response=$(curl -s -w "%{http_code}" -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}')

direct_code="${direct_response: -3}"
if [ "$direct_code" = "200" ]; then
    print_success "Direct auth service working (HTTP $direct_code)"
else
    print_fail "Direct auth service failed (HTTP $direct_code)"
fi

echo ""

# Test 2: API Gateway Routing
print_info "Testing API Gateway auth routing..."
gateway_response=$(curl -s -w "%{http_code}" -X POST http://localhost:8080/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}')

gateway_code="${gateway_response: -3}"
if [ "$gateway_code" = "200" ]; then
    print_success "API Gateway auth routing working (HTTP $gateway_code)"
else
    print_fail "API Gateway auth routing failed (HTTP $gateway_code)"
    echo "Response: ${gateway_response%???}"
fi

echo ""

# Test 3: Compare Responses
if [ "$direct_code" = "200" ] && [ "$gateway_code" = "200" ]; then
    print_info "Comparing response structures..."
    
    direct_body="${direct_response%???}"
    gateway_body="${gateway_response%???}"
    
    if [[ "$direct_body" == *"access_token"* ]] && [[ "$gateway_body" == *"access_token"* ]]; then
        print_success "Both responses contain access tokens"
    else
        print_fail "Response structure mismatch"
    fi
fi

echo ""

# Test 4: Frontend API Configuration
print_info "Testing frontend API configuration..."
if [ -f "frontend/.env" ]; then
    api_url=$(grep VITE_API_URL frontend/.env | cut -d'=' -f2)
    print_info "Frontend configured to use: $api_url"
    
    if [ "$api_url" = "http://localhost:8080" ]; then
        print_success "Frontend correctly configured for API Gateway"
    else
        print_fail "Frontend API URL misconfigured"
    fi
else
    print_fail "Frontend .env file not found"
fi

echo ""

# Test 5: CORS Headers
print_info "Testing CORS headers..."
cors_response=$(curl -s -I -X OPTIONS http://localhost:8080/api/v1/auth/login \
  -H 'Origin: http://localhost:1420' \
  -H 'Access-Control-Request-Method: POST' \
  -H 'Access-Control-Request-Headers: Content-Type')

if [[ "$cors_response" == *"Access-Control-Allow-Origin"* ]]; then
    print_success "CORS headers present"
else
    print_info "CORS headers may need configuration"
fi

echo ""
echo "📊 Summary"
echo "=========="
echo "• Direct Auth Service: http://localhost:8081/api/v1/auth/login"
echo "• API Gateway Route:   http://localhost:8080/api/v1/auth/login"
echo "• Frontend URL:        http://localhost:1420"
echo ""
print_success "API Gateway auth routing test complete!"