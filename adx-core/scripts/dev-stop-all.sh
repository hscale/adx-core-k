#!/bin/bash

# ADX CORE Complete Development Environment Shutdown Script
# This script stops ALL infrastructure and services

set -e

echo "🛑 Stopping ADX CORE Complete Development Environment..."

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

# Navigate to the infrastructure directory
cd "$(dirname "$0")/../infrastructure/docker"

print_status "Stopping all infrastructure services..."

# Stop all services
docker-compose -f docker-compose.dev.yml -f docker-compose.temporal.yml down

print_success "All infrastructure services stopped!"

# Optional cleanup
if [ "$1" = "--clean" ]; then
    print_status "Cleaning up Docker resources..."
    
    # Remove unused containers
    docker container prune -f
    
    # Remove unused images
    docker image prune -f
    
    # Remove unused networks
    docker network prune -f
    
    print_success "Docker cleanup completed!"
fi

if [ "$1" = "--reset" ]; then
    print_warning "Resetting all data (removing volumes)..."
    docker-compose -f docker-compose.dev.yml -f docker-compose.temporal.yml down -v
    print_success "All data reset!"
fi

print_success "🎉 ADX CORE development environment shutdown complete!"
print_status "To restart: ./scripts/dev-start-all.sh"