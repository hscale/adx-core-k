#!/bin/bash

# ADX CORE Development Environment Startup Script
# This script starts the complete development environment including infrastructure and services

set -e

echo "🚀 Starting ADX CORE Development Environment..."

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

# Check if Docker Compose is available
if ! command -v docker-compose > /dev/null 2>&1; then
    print_error "Docker Compose is not installed. Please install Docker Compose and try again."
    exit 1
fi

# Navigate to the infrastructure directory
cd "$(dirname "$0")/../infrastructure/docker"

print_status "Starting infrastructure services (PostgreSQL, Redis, Temporal)..."

# Start infrastructure services
docker-compose -f docker-compose.dev.yml up -d

print_status "Waiting for services to be ready..."

# Wait for PostgreSQL to be ready
print_status "Waiting for PostgreSQL..."
until docker-compose -f docker-compose.dev.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
done
print_success "PostgreSQL is ready"

# Wait for Redis to be ready
print_status "Waiting for Redis..."
until docker-compose -f docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1; do
    sleep 2
done
print_success "Redis is ready"

# Wait for Temporal to be ready
print_status "Waiting for Temporal..."
until curl -f http://localhost:7233/api/v1/namespaces > /dev/null 2>&1; do
    sleep 5
done
print_success "Temporal is ready"

print_success "Infrastructure services are running!"
print_status "Services available at:"
echo "  - PostgreSQL: localhost:5432"
echo "  - Redis: localhost:6379"
echo "  - Temporal Server: localhost:7233"
echo "  - Temporal UI: http://localhost:8088"

# Navigate back to workspace root
cd "$(dirname "$0")/.."

print_status "Building Rust workspace..."
if cargo build; then
    print_success "Rust workspace built successfully"
else
    print_error "Failed to build Rust workspace"
    exit 1
fi

print_status "Development environment is ready!"
print_status "To start individual services, use:"
echo "  cargo run --bin auth-service"
echo "  cargo run --bin user-service"
echo "  cargo run --bin file-service"
echo "  cargo run --bin tenant-service"
echo "  cargo run --bin workflow-service"

print_status "To run services in worker mode:"
echo "  cargo run --bin auth-service -- --mode worker"
echo "  cargo run --bin user-service -- --mode worker"

print_warning "Remember to set up your environment variables in .env files for each service"