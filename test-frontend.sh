#!/bin/bash

# ============================================================================
# ADX CORE Frontend Testing Script
# Quick frontend testing for development
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

echo "🌐 ADX CORE Frontend Testing"
echo "============================"
echo ""

# Test 1: Frontend Accessibility
print_test "Frontend Accessibility"
echo ""

print_info "Testing frontend server..."
response=$(curl -s -w "%{http_code}" http://localhost:1420)
http_code="${response: -3}"

if [ "$http_code" = "200" ]; then
    print_success "Frontend server is accessible"
    
    # Check if it's actually serving HTML
    if [[ "$response" == *"<html"* ]] || [[ "$response" == *"<!DOCTYPE"* ]]; then
        print_success "Frontend is serving HTML content"
    else
        print_info "Frontend is responding but may not be fully loaded"
    fi
else
    print_fail "Frontend server is not responding (HTTP $http_code)"
fi

echo ""

# Test 2: Static Assets
print_test "Static Assets"
echo ""

print_info "Testing Vite dev server assets..."
assets_response=$(curl -s -w "%{http_code}" http://localhost:1420/@vite/client)
assets_http_code="${assets_response: -3}"

if [ "$assets_http_code" = "200" ]; then
    print_success "Vite client assets are loading"
else
    print_info "Vite assets may not be ready yet (HTTP $assets_http_code)"
fi

echo ""

# Test 3: API Connection from Frontend
print_test "Frontend to Backend Connection"
echo ""

print_info "Testing if frontend can reach backend API..."
# This would typically be done through the frontend, but we can test the API endpoint
api_response=$(curl -s -w "%{http_code}" http://localhost:8080/health)
api_http_code="${api_response: -3}"

if [ "$api_http_code" = "200" ]; then
    print_success "Backend API is reachable from frontend network"
else
    print_fail "Backend API is not reachable (HTTP $api_http_code)"
fi

echo ""

# Test 4: Development Tools
print_test "Development Tools"
echo ""

print_info "Checking if React DevTools can connect..."
if pgrep -f "npm run dev" > /dev/null; then
    print_success "Frontend development server is running"
else
    print_fail "Frontend development server is not running"
fi

print_info "Checking for hot reload capability..."
if [ -f "frontend/vite.config.ts" ]; then
    print_success "Vite configuration found - hot reload should work"
else
    print_info "Vite configuration not found"
fi

echo ""

# Summary
echo "📊 Frontend Test Summary"
echo "========================"
echo ""
echo "🌐 Frontend URL:        http://localhost:1420"
echo "🔧 Development Server:  Vite"
echo "⚛️  Framework:          React + TypeScript"
echo "🎨 Styling:             TailwindCSS"
echo "🌍 Internationalization: react-i18next"
echo ""
echo "📋 Useful Commands:"
echo "  • View frontend logs:   tail -f logs/frontend.log"
echo "  • Restart frontend:     pkill -f 'npm run dev' && cd frontend && npm run dev &"
echo "  • Install dependencies: cd frontend && npm install"
echo "  • Build for production: cd frontend && npm run build"
echo ""

print_success "Frontend testing complete!"
echo ""
echo "💡 Next Steps:"
echo "  1. Open http://localhost:1420 in your browser"
echo "  2. Check browser console for any errors"
echo "  3. Test authentication flow with backend"
echo "  4. Verify responsive design on different screen sizes"