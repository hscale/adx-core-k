#!/bin/bash

# ADX CORE Frontend-Only Development Environment
# This script starts infrastructure and frontend components for live testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

print_section() {
    echo -e "${PURPLE}[SECTION]${NC} $1"
}

print_service() {
    echo -e "${CYAN}[SERVICE]${NC} $1"
}

# Cleanup function
cleanup() {
    print_warning "Shutting down services..."
    
    # Kill background processes
    if [ ! -z "$PIDS" ]; then
        for pid in $PIDS; do
            kill $pid 2>/dev/null || true
        done
    fi
    
    # Stop Docker services
    cd adx-core/infrastructure/docker
    docker-compose -f docker-compose.dev.yml down
    
    print_success "Cleanup completed"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Store PIDs of background processes
PIDS=""

echo "🎨 ADX CORE Frontend Development Environment"
echo "============================================"

# Check prerequisites
print_section "🔍 Checking Prerequisites"

if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker Desktop and try again."
    exit 1
fi
print_success "Docker is running"

if ! command -v node > /dev/null 2>&1; then
    print_error "Node.js is not installed."
    exit 1
fi
print_success "Node.js $(node --version) is available"

print_section "🏗️  Starting Infrastructure Services"

# Navigate to infrastructure directory
cd adx-core/infrastructure/docker

# Start infrastructure services
print_status "Starting PostgreSQL, Redis, Temporal..."
docker-compose -f docker-compose.dev.yml up -d

print_status "Waiting for services to be ready..."

# Wait for PostgreSQL
print_status "Waiting for PostgreSQL..."
timeout=60
counter=0
until docker-compose -f docker-compose.dev.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
    counter=$((counter + 2))
    if [ $counter -ge $timeout ]; then
        print_error "PostgreSQL failed to start"
        exit 1
    fi
done
print_success "PostgreSQL is ready"

# Wait for Redis
print_status "Waiting for Redis..."
counter=0
until docker-compose -f docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1; do
    sleep 2
    counter=$((counter + 2))
    if [ $counter -ge $timeout ]; then
        print_error "Redis failed to start"
        exit 1
    fi
done
print_success "Redis is ready"

# Navigate back to workspace root
cd ../../..

print_section "📦 Installing Frontend Dependencies"

# Install root dependencies if needed
if [ ! -d "node_modules" ]; then
    print_status "Installing root dependencies..."
    npm install
fi

# Install dependencies for micro-frontends
for app in shell auth tenant file user workflow module; do
    if [ -d "apps/$app" ] && [ ! -d "apps/$app/node_modules" ]; then
        print_status "Installing $app dependencies..."
        cd "apps/$app"
        npm install
        cd ../..
    fi
done

print_section "🎨 Starting Frontend Micro-Apps"

# Create logs directory
mkdir -p logs

# Start micro-frontends
micro_apps=("shell" "auth" "tenant" "file" "user" "workflow" "module")
frontend_ports=(3000 3001 3002 3003 3004 3005 3006)

for i in "${!micro_apps[@]}"; do
    app="${micro_apps[$i]}"
    port="${frontend_ports[$i]}"
    
    if [ -d "apps/$app" ]; then
        print_service "Starting $app on port $port..."
        cd "apps/$app"
        npm run dev > "../../logs/${app}.log" 2>&1 &
        pid=$!
        PIDS="$PIDS $pid"
        print_success "$app started (PID: $pid)"
        cd ../..
        sleep 3
    fi
done

print_success "🎉 Frontend Environment Ready!"
echo ""
echo "📋 Available Services:"
echo "  - Shell Application: http://localhost:3000"
echo "  - Auth Micro-App: http://localhost:3001"
echo "  - Tenant Micro-App: http://localhost:3002"
echo "  - File Micro-App: http://localhost:3003"
echo "  - User Micro-App: http://localhost:3004"
echo "  - Workflow Micro-App: http://localhost:3005"
echo "  - Module Micro-App: http://localhost:3006"
echo ""
echo "🏗️  Infrastructure:"
echo "  - Temporal UI: http://localhost:8088"
echo "  - Prometheus: http://localhost:9090"
echo "  - Grafana: http://localhost:3001"
echo ""
print_status "Press Ctrl+C to stop all services"

# Monitor services
while true; do
    sleep 30
    current_time=$(date '+%H:%M:%S')
    print_status "Services running at $current_time"
done