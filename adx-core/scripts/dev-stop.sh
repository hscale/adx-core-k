#!/bin/bash

# ADX CORE Development Environment Shutdown Script
# This script stops all development services and cleans up resources

set -e

echo "🛑 Stopping ADX CORE Development Environment..."

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

print_status "Stopping infrastructure services..."

# Stop all services
docker-compose -f docker-compose.dev.yml down

# Optional: Remove volumes (uncomment if you want to reset data)
# print_warning "Removing volumes (this will delete all data)..."
# docker-compose -f docker-compose.dev.yml down -v

print_success "All services stopped successfully!"

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

print_status "Development environment shutdown complete."
print_status "To restart, run: ./scripts/dev-start.sh"