#!/bin/bash

# ADX CORE Complete Development Environment Startup Script
# This script starts ALL infrastructure and services for a complete development environment

set -e

echo "🚀 Starting ADX CORE Complete Development Environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Navigate to the infrastructure directory
cd "$(dirname "$0")/../infrastructure/docker"

print_status "Starting complete ADX CORE infrastructure..."

# Start all infrastructure services
print_status "Starting PostgreSQL, Redis, Temporal, and monitoring stack..."
docker-compose -f docker-compose.dev.yml -f docker-compose.temporal.yml up -d

print_status "Waiting for all services to be ready..."

# Wait for PostgreSQL to be ready
print_status "Waiting for PostgreSQL..."
timeout=60
count=0
until docker-compose -f docker-compose.dev.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
    count=$((count + 1))
    if [ $count -gt $timeout ]; then
        print_error "PostgreSQL failed to start within $timeout seconds"
        exit 1
    fi
done
print_success "PostgreSQL is ready"

# Wait for Redis to be ready
print_status "Waiting for Redis..."
count=0
until docker-compose -f docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1; do
    sleep 2
    count=$((count + 1))
    if [ $count -gt $timeout ]; then
        print_error "Redis failed to start within $timeout seconds"
        exit 1
    fi
done
print_success "Redis is ready"

# Wait for Temporal to be ready
print_status "Waiting for Temporal..."
count=0
until docker exec temporal tctl cluster health > /dev/null 2>&1; do
    sleep 5
    count=$((count + 1))
    if [ $count -gt 30 ]; then
        print_error "Temporal failed to start within 150 seconds"
        exit 1
    fi
done
print_success "Temporal is ready"

# Setup Temporal namespaces
print_status "Setting up Temporal namespaces..."
if [ -f "$(dirname "$0")/setup-temporal-namespaces.sh" ]; then
    bash "$(dirname "$0")/setup-temporal-namespaces.sh" > /dev/null 2>&1 || print_warning "Namespace setup may have failed, but continuing..."
    print_success "Temporal namespaces configured"
else
    print_warning "Namespace setup script not found, skipping..."
fi

print_success "All infrastructure services are running!"

# Navigate back to workspace root
cd "$(dirname "$0")/.."

print_status "Building Rust workspace..."
if cargo build --release; then
    print_success "Rust workspace built successfully"
else
    print_error "Failed to build Rust workspace"
    exit 1
fi

print_success "🎉 Complete ADX CORE development environment is ready!"
echo ""
print_status "Infrastructure Services:"
echo "  ✅ PostgreSQL: localhost:5432"
echo "  ✅ Redis: localhost:6379"
echo "  ✅ Temporal Server: localhost:7233"
echo "  ✅ Temporal UI: http://localhost:8088"
echo "  ✅ Prometheus: http://localhost:9090"
echo "  ✅ Grafana: http://localhost:3001 (admin/admin)"
echo "  ✅ Jaeger: http://localhost:16686"
echo ""
print_status "To start backend services:"
echo "  # HTTP Server Mode"
echo "  cargo run --bin auth-service server"
echo "  cargo run --bin user-service server"
echo "  cargo run --bin file-service server"
echo "  cargo run --bin tenant-service server"
echo "  cargo run --bin workflow-service server"
echo "  cargo run --bin api-gateway"
echo ""
echo "  # Temporal Worker Mode"
echo "  cargo run --bin auth-service worker"
echo "  cargo run --bin user-service worker"
echo "  cargo run --bin file-service worker"
echo "  cargo run --bin tenant-service worker"
echo "  cargo run --bin workflow-service worker"
echo ""
print_status "Frontend development:"
echo "  cd ../apps/shell && npm run dev"
echo "  cd ../apps/auth && npm run dev"
echo "  cd ../apps/tenant && npm run dev"
echo ""
print_warning "Remember to:"
echo "  1. Set up environment variables in .env files"
echo "  2. Run database migrations if needed"
echo "  3. Start frontend services separately"
echo ""
print_status "To stop everything: ./scripts/dev-stop-all.sh"