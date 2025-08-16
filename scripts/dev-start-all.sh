#!/bin/bash

# ADX CORE Complete Development Environment Startup Script
# This script starts ALL components for live testing including:
# - Infrastructure services (PostgreSQL, Redis, Temporal)
# - Backend microservices (Rust)
# - Frontend micro-apps (React Module Federation)
# - BFF services (Backend for Frontend)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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
    docker compose -f docker-compose.dev.yml down
    
    print_success "Cleanup completed"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Store PIDs of background processes
PIDS=""

# Check prerequisites
print_section "🔍 Checking Prerequisites"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker Desktop and try again."
    exit 1
fi
print_success "Docker is running"

# Check if Node.js is installed
if ! command -v node > /dev/null 2>&1; then
    print_error "Node.js is not installed. Please install Node.js and try again."
    exit 1
fi
print_success "Node.js is available: $(node --version)"

# Check if Rust is installed
if ! command -v cargo > /dev/null 2>&1; then
    print_error "Rust is not installed. Please install Rust and try again."
    exit 1
fi
print_success "Rust is available: $(cargo --version)"

# Check if npm is installed
if ! command -v npm > /dev/null 2>&1; then
    print_error "npm is not installed. Please install npm and try again."
    exit 1
fi
print_success "npm is available: $(npm --version)"

print_section "🏗️  Setting Up Infrastructure Services"

# Navigate to infrastructure directory
cd adx-core/infrastructure/docker

# Start infrastructure services
print_status "Starting PostgreSQL, Redis, Temporal, and monitoring services..."
docker compose -f docker-compose.dev.yml up -d

# Wait for services to be ready
print_status "Waiting for infrastructure services to be ready..."

# Wait for PostgreSQL
print_status "Waiting for PostgreSQL..."
timeout=60
counter=0
until docker compose -f docker-compose.dev.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 2
    counter=$((counter + 2))
    if [ $counter -ge $timeout ]; then
        print_error "PostgreSQL failed to start within $timeout seconds"
        exit 1
    fi
done
print_success "PostgreSQL is ready"

# Wait for Redis
print_status "Waiting for Redis..."
counter=0
until docker compose -f docker-compose.dev.yml exec -T redis redis-cli ping > /dev/null 2>&1; do
    sleep 2
    counter=$((counter + 2))
    if [ $counter -ge $timeout ]; then
        print_error "Redis failed to start within $timeout seconds"
        exit 1
    fi
done
print_success "Redis is ready"

# Wait for Temporal
# print_status "Waiting for Temporal..."
# counter=0
# until docker exec adx-temporal tctl cluster health > /dev/null 2>&1; do
#     sleep 5
#     counter=$((counter + 5))
#     if [ $counter -ge 120 ]; then
#         print_error "Temporal failed to start within 120 seconds"
#         exit 1
#     fi
# done
# print_success "Temporal is ready"

# Navigate back to workspace root
cd ../../..

print_section "🔧 Building Backend Services"

# Build Rust workspace
print_status "Building Rust workspace..."
cd adx-core
if cargo build --workspace; then
    print_success "Rust workspace built successfully"
else
    print_error "Failed to build Rust workspace"
    exit 1
fi
cd ..

print_section "📦 Installing Frontend Dependencies"

# Install root dependencies
print_status "Installing root dependencies..."
if npm install; then
    print_success "Root dependencies installed"
else
    print_error "Failed to install root dependencies"
    exit 1
fi

# Install dependencies for each micro-frontend
for app in shell auth tenant file user workflow module; do
    if [ -d "apps/$app" ]; then
        print_status "Installing dependencies for $app micro-frontend..."
        cd "apps/$app"
        if npm install; then
            print_success "$app dependencies installed"
        else
            print_error "Failed to install $app dependencies"
            exit 1
        fi
        cd ../..
    fi
done

# Install dependencies for BFF services
for bff in auth-bff tenant-bff file-bff user-bff workflow-bff module-bff; do
    if [ -d "bff-services/$bff" ]; then
        print_status "Installing dependencies for $bff..."
        cd "bff-services/$bff"
        if [ -f "package.json" ]; then
            if npm install; then
                print_success "$bff dependencies installed"
            else
                print_error "Failed to install $bff dependencies"
                exit 1
            fi
        fi
        cd ../..
    fi
done

print_section "🚀 Starting Backend Microservices"

# Start backend services in server mode
services=("api-gateway" "auth-service" "tenant-service" "file-service" "user-service" "workflow-service" "module-service")
ports=(8080 8081 8085 8083 8082 8084 8086)

for i in "${!services[@]}"; do
    service="${services[$i]}"
    port="${ports[$i]}"
    
    if [ -d "adx-core/services/$service" ]; then
        print_service "Starting $service on port $port..."
        cd adx-core
        cargo run --bin "$service" > "../logs/${service}.log" 2>&1 &
        pid=$!
        PIDS="$PIDS $pid"
        print_success "$service started (PID: $pid)"
        cd ..
        
        # Give service time to start
        sleep 3
    else
        print_warning "$service directory not found, skipping..."
    fi
done

# print_section "⚡ Starting Temporal Workers"

# # Start Temporal workers for each service
# for service in "${services[@]}"; do
#     if [ -d "adx-core/services/$service" ] && [ "$service" != "api-gateway" ]; then
#         print_service "Starting $service Temporal worker..."
#         cd adx-core
#         cargo run --bin "$service" -- --mode worker > "../logs/${service}-worker.log" 2>&1 &
#         pid=$!
#         PIDS="$PIDS $pid"
#         print_success "$service worker started (PID: $pid)"
#         cd ..
        
#         # Give worker time to start
#         sleep 2
#     fi
# done

print_section "🌐 Starting BFF Services"

# Start BFF services
bff_services=("auth-bff" "tenant-bff" "file-bff" "user-bff" "workflow-bff" "module-bff")
bff_ports=(4001 4002 4003 4004 4005 4006)

for i in "${!bff_services[@]}"; do
    bff="${bff_services[$i]}"
    port="${bff_ports[$i]}"
    
    if [ -d "bff-services/$bff" ]; then
        print_service "Starting $bff on port $port..."
        cd "bff-services/$bff"
        
        if [ -f "Cargo.toml" ]; then
            # Rust BFF service
            cargo run > "../../logs/${bff}.log" 2>&1 &
        elif [ -f "package.json" ]; then
            # Node.js BFF service
            npm run dev > "../../logs/${bff}.log" 2>&1 &
        fi
        
        pid=$!
        PIDS="$PIDS $pid"
        print_success "$bff started (PID: $pid)"
        cd ../..
        
        # Give BFF time to start
        sleep 2
    else
        print_warning "$bff directory not found, skipping..."
    fi
done

print_section "🎨 Starting Frontend Micro-Apps"

# Create logs directory if it doesn't exist
mkdir -p logs

# Start micro-frontends
micro_apps=("shell" "auth" "tenant" "file" "user" "workflow" "module")
frontend_ports=(3000 3001 3002 3003 3004 3005 3006)

for i in "${!micro_apps[@]}"; do
    app="${micro_apps[$i]}"
    port="${frontend_ports[$i]}"
    
    if [ -d "apps/$app" ]; then
        print_service "Starting $app micro-frontend on port $port..."
        cd "apps/$app"
        npm run dev > "../../logs/${app}-frontend.log" 2>&1 &
        pid=$!
        PIDS="$PIDS $pid"
        print_success "$app micro-frontend started (PID: $pid)"
        cd ../..
        
        # Give frontend time to start
        sleep 3
    else
        print_warning "$app directory not found, skipping..."
    fi
done

print_section "🎯 Development Environment Ready!"

print_success "All services are starting up. Please wait a moment for everything to be ready..."

# Wait a bit for all services to fully start
sleep 10

print_success "🎉 ADX CORE Development Environment is now running!"
echo ""
print_section "📋 Service URLs:"
echo ""
echo "🏗️  Infrastructure Services:"
echo "  - PostgreSQL: localhost:5432"
echo "  - Redis: localhost:6379"
echo "  - Temporal Server: localhost:7233"
echo "  - Temporal UI: http://localhost:8088"
echo "  - Prometheus: http://localhost:9090"
echo "  - Grafana: http://localhost:3001 (admin/admin)"
echo "  - Jaeger: http://localhost:16686"
echo ""
echo "🔧 Backend Microservices:"
echo "  - API Gateway: http://localhost:8080"
echo "  - Auth Service: http://localhost:8081"
echo "  - User Service: http://localhost:8082"
echo "  - File Service: http://localhost:8083"
echo "  - Workflow Service: http://localhost:8084"
echo "  - Tenant Service: http://localhost:8085"
echo "  - Module Service: http://localhost:8086"
echo ""
echo "🌐 BFF Services:"
echo "  - Auth BFF: http://localhost:4001"
echo "  - Tenant BFF: http://localhost:4002"
echo "  - File BFF: http://localhost:4003"
echo "  - User BFF: http://localhost:4004"
echo "  - Workflow BFF: http://localhost:4005"
echo "  - Module BFF: http://localhost:4006"
echo ""
echo "🎨 Frontend Micro-Apps:"
echo "  - Shell Application: http://localhost:3000"
echo "  - Auth Micro-App: http://localhost:3001"
echo "  - Tenant Micro-App: http://localhost:3002"
echo "  - File Micro-App: http://localhost:3003"
echo "  - User Micro-App: http://localhost:3004"
echo "  - Workflow Micro-App: http://localhost:3005"
echo "  - Module Micro-App: http://localhost:3006"
echo ""
print_section "📊 Monitoring & Logs:"
echo "  - Service logs are in ./logs/ directory"
echo "  - Use 'tail -f logs/[service-name].log' to follow logs"
echo ""
print_section "🛑 To Stop All Services:"
echo "  - Press Ctrl+C in this terminal"
echo "  - Or run: docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down"
echo ""

# Keep the script running and monitor services
print_status "Monitoring services... Press Ctrl+C to stop all services"

# Function to check if a service is still running
check_service() {
    local pid=$1
    local service_name=$2
    
    if ! kill -0 $pid 2>/dev/null; then
        print_error "$service_name (PID: $pid) has stopped unexpectedly"
        return 1
    fi
    return 0
}

# Monitor services
while true; do
    sleep 30
    
    # Check if any services have died
    failed_services=0
    for pid in $PIDS; do
        if ! kill -0 $pid 2>/dev/null; then
            failed_services=$((failed_services + 1))
        fi
    done
    
    if [ $failed_services -gt 0 ]; then
        print_warning "$failed_services service(s) have stopped. Check logs for details."
    fi
    
    # Print a heartbeat every 5 minutes
    current_time=$(date '+%H:%M:%S')
    print_status "Services running at $current_time (Press Ctrl+C to stop)"
done