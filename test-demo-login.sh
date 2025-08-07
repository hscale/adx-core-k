#!/bin/bash

# ============================================================================
# ADX CORE Demo Login Test Script
# Quick test of demo credentials
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

echo -e "${BLUE}🔐 ADX CORE Demo Login Test${NC}"
echo "============================"
echo ""

# Demo credentials
EMAIL="admin@example.com"
PASSWORD="password"
TENANT_ID="550e8400-e29b-41d4-a716-446655440000"

print_info "Testing demo credentials:"
echo "  Email: $EMAIL"
echo "  Password: $PASSWORD"
echo "  Tenant ID: $TENANT_ID"
echo ""

# Test authentication
print_info "Attempting login..."
response=$(curl -s -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\",\"tenant_id\":\"$TENANT_ID\"}")

# Check if login was successful
if [[ "$response" == *"access_token"* ]]; then
    print_success "Demo login successful!"
    
    # Extract token for display
    token=$(echo $response | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
    user_id=$(echo $response | grep -o '"user_id":"[^"]*' | cut -d'"' -f4)
    
    echo ""
    print_info "Login Details:"
    echo "  User ID: $user_id"
    echo "  Token: ${token:0:50}..."
    echo ""
    
    # Test a protected endpoint
    print_info "Testing protected endpoint..."
    user_response=$(curl -s -H "Authorization: Bearer $token" \
                         -H "X-Tenant-ID: $TENANT_ID" \
                         http://localhost:8080/api/v1/users)
    
    if [[ "$user_response" != *"error"* ]] && [[ "$user_response" != "" ]]; then
        print_success "Protected endpoint access successful!"
    else
        print_fail "Protected endpoint access failed"
    fi
    
else
    print_fail "Demo login failed!"
    echo "Response: $response"
    echo ""
    print_info "Troubleshooting steps:"
    echo "  1. Check if auth service is running: curl http://localhost:8081/health"
    echo "  2. Check if database is accessible: docker exec docker-postgres-1 pg_isready"
    echo "  3. View auth service logs: tail -f logs/auth-service.log"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Demo Login Test Complete!${NC}"
echo ""
echo "📋 Next Steps:"
echo "  1. Open frontend: http://localhost:1420"
echo "  2. Use these credentials to login:"
echo "     Email: $EMAIL"
echo "     Password: $PASSWORD"
echo "  3. Explore the dashboard and features"
echo ""
echo "🔗 Useful Links:"
echo "  • Frontend: http://localhost:1420"
echo "  • API Gateway: http://localhost:8080"
echo "  • Temporal UI: http://localhost:8088"
echo "  • Demo Credentials Guide: ./DEMO-CREDENTIALS.md"