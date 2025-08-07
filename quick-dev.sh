#!/bin/bash

# ============================================================================
# ADX CORE Quick Development Script
# Fast startup and testing for development environment
# ============================================================================

set -e

echo "🚀 ADX CORE Quick Development Environment"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker first."
    exit 1
fi

# Start infrastructure if not running
echo ""
print_info "Starting infrastructure services..."
cd adx-core/infrastructure/docker
docker compose -f docker-compose.dev.yml up -d
cd ../../..

# Wait for infrastructure
print_info "Waiting for infrastructure to be ready..."
sleep 5

# Check infrastructure health
print_info "Checking infrastructure health..."
if docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml ps | grep -q "healthy"; then
    print_status "Infrastructure services are healthy"
else
    print_warning "Some infrastructure services may not be fully ready"
fi

# Create logs directory
mkdir -p logs

# Build backend services
echo ""
print_info "Building backend services..."
cd adx-core
if cargo build --workspace; then
    print_status "Backend services built successfully"
else
    print_error "Backend build failed"
    exit 1
fi

# Start backend services
echo ""
print_info "Starting backend services..."

# Kill any existing services
pkill -f "cargo run" 2>/dev/null || true

# Start services in background
RUST_LOG=info cargo run -p auth-service > ../logs/auth-service.log 2>&1 &
AUTH_PID=$!

RUST_LOG=info cargo run -p user-service > ../logs/user-service.log 2>&1 &
USER_PID=$!

RUST_LOG=info cargo run -p file-service > ../logs/file-service.log 2>&1 &
FILE_PID=$!

RUST_LOG=info cargo run -p workflow-service > ../logs/workflow-service.log 2>&1 &
WORKFLOW_PID=$!

RUST_LOG=info cargo run -p tenant-service > ../logs/tenant-service.log 2>&1 &
TENANT_PID=$!

RUST_LOG=info cargo run -p api-gateway > ../logs/api-gateway.log 2>&1 &
GATEWAY_PID=$!

cd ..

# Wait for services to start
print_info "Waiting for backend services to start..."
sleep 15

# Test backend services
echo ""
print_info "Testing backend services..."

# Test each service
services=(
    "8081:Auth Service"
    "8082:User Service" 
    "8083:File Service"
    "8084:Workflow Service"
    "8085:Tenant Service"
    "8080:API Gateway"
)

all_healthy=true
for service in "${services[@]}"; do
    port=$(echo $service | cut -d: -f1)
    name=$(echo $service | cut -d: -f2)
    
    if curl -s http://localhost:$port/health > /dev/null; then
        print_status "$name (port $port) is healthy"
    else
        print_error "$name (port $port) is not responding"
        all_healthy=false
    fi
done

if [ "$all_healthy" = true ]; then
    print_status "All backend services are running!"
else
    print_warning "Some backend services are not ready. Check logs in ./logs/"
fi

# Test API endpoints
echo ""
print_info "Testing API endpoints..."

# Test authentication
print_info "Testing authentication endpoint..."
auth_response=$(curl -s -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}' || echo "failed")

if [[ "$auth_response" == *"access_token"* ]]; then
    print_status "Authentication endpoint working"
else
    print_warning "Authentication endpoint may not be fully configured"
fi

# Test user service through API Gateway
print_info "Testing user service routing..."
user_response=$(curl -s http://localhost:8080/api/v1/users || echo "failed")

if [[ "$user_response" != "failed" ]]; then
    print_status "API Gateway routing working"
else
    print_warning "API Gateway routing may need configuration"
fi

# Display service information
echo ""
echo "🌐 ADX CORE Development Environment Status"
echo "=========================================="
echo ""
echo "📊 Backend Services:"
echo "  • API Gateway:      http://localhost:8080"
echo "  • Auth Service:     http://localhost:8081"
echo "  • User Service:     http://localhost:8082"
echo "  • File Service:     http://localhost:8083"
echo "  • Workflow Service: http://localhost:8084"
echo "  • Tenant Service:   http://localhost:8085"
echo ""
echo "🔧 Infrastructure:"
echo "  • PostgreSQL:       postgresql://adx_user:dev_password@localhost:5432/adx_core"
echo "  • Redis:            redis://localhost:6379"
echo "  • Temporal Server:  http://localhost:7233"
echo "  • Temporal UI:      http://localhost:8088"
echo ""
echo "📋 Logs:"
echo "  • Service logs:     ./logs/"
echo "  • Infrastructure:   docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml logs"
echo ""

# Quick test commands
echo "🧪 Quick Test Commands:"
echo "=========================================="
echo ""
echo "# Health Checks:"
echo "curl http://localhost:8080/health"
echo "curl http://localhost:8081/health"
echo ""
echo "# Authentication Test:"
echo 'curl -X POST http://localhost:8081/api/v1/auth/login \'
echo '  -H "Content-Type: application/json" \'
echo '  -d '"'"'{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}'"'"
echo ""
echo "# User Management:"
echo "curl http://localhost:8080/api/v1/users"
echo ""
echo "# Stop Services:"
echo "pkill -f 'cargo run'"
echo "docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down"
echo ""

# Cleanup function
cleanup() {
    echo ""
    print_info "Shutting down services..."
    pkill -f "cargo run" 2>/dev/null || true
    docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down --remove-orphans
    print_status "All services stopped"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

print_status "Development environment is ready!"
print_info "Press Ctrl+C to stop all services"

# Start Frontend
echo ""
print_info "Starting frontend development server..."
cd frontend
npm run dev > ../logs/frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..

# Wait for frontend to start
sleep 5

# Test frontend
if curl -s http://localhost:1420 > /dev/null; then
    print_status "Frontend is running at http://localhost:1420"
else
    print_warning "Frontend may not be fully ready yet"
fi

echo ""
print_status "🎉 Complete development environment is ready!"
echo ""
echo "🌐 Access Points:"
echo "  • Frontend:         http://localhost:1420"
echo "  • API Gateway:      http://localhost:8080"
echo "  • Temporal UI:      http://localhost:8088"
echo ""

# Keep script alive
while true; do
    sleep 30
    # Optional: Add service health monitoring here
done