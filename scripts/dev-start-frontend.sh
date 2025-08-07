#!/bin/bash

# ADX CORE - Frontend Microservices Development Startup Script
# This script starts all frontend microservices and BFF services for development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_service() {
    echo -e "${BLUE}[SERVICE]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "frontend" ]; then
    print_error "Please run this script from the ADX CORE root directory"
    exit 1
fi

print_status "Starting ADX CORE Frontend Microservices Development Environment"
print_status "================================================================"

# Function to check if port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 1
    else
        return 0
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local name=$2
    local max_attempts=30
    local attempt=1
    
    print_status "Waiting for $name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            print_status "$name is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    print_error "$name failed to start within $max_attempts seconds"
    return 1
}

# Check required ports
REQUIRED_PORTS=(3000 3001 3002 3003 3004 3005 4001 4002 4003 4004 4005)
OCCUPIED_PORTS=()

print_status "Checking port availability..."
for port in "${REQUIRED_PORTS[@]}"; do
    if ! check_port $port; then
        OCCUPIED_PORTS+=($port)
    fi
done

if [ ${#OCCUPIED_PORTS[@]} -gt 0 ]; then
    print_error "The following ports are already in use: ${OCCUPIED_PORTS[*]}"
    print_error "Please stop the services using these ports and try again"
    print_status "You can use: lsof -ti:PORT | xargs kill -9"
    exit 1
fi

# Create logs directory
mkdir -p logs/frontend

# Function to start a service in background
start_service() {
    local name=$1
    local command=$2
    local log_file="logs/frontend/${name}.log"
    
    print_service "Starting $name..."
    
    # Start the service in background and redirect output to log file
    eval "$command" > "$log_file" 2>&1 &
    local pid=$!
    
    # Store PID for cleanup
    echo $pid > "logs/frontend/${name}.pid"
    
    print_service "$name started (PID: $pid, Log: $log_file)"
}

# Cleanup function
cleanup() {
    print_status "Shutting down services..."
    
    # Kill all background processes
    for pid_file in logs/frontend/*.pid; do
        if [ -f "$pid_file" ]; then
            local pid=$(cat "$pid_file")
            local name=$(basename "$pid_file" .pid)
            
            if kill -0 $pid 2>/dev/null; then
                print_service "Stopping $name (PID: $pid)..."
                kill $pid 2>/dev/null || true
            fi
            
            rm -f "$pid_file"
        fi
    done
    
    print_status "All services stopped"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

print_status "Starting Backend Services (if not already running)..."

# Check if backend services are running
if ! curl -s http://localhost:8080/health >/dev/null 2>&1; then
    print_warning "Backend services not detected. Starting them..."
    if [ -f "adx-core/scripts/dev-start.sh" ]; then
        cd adx-core && ./scripts/dev-start.sh &
        cd ..
        print_status "Waiting for backend services to start..."
        sleep 10
    else
        print_error "Backend startup script not found. Please start backend services manually."
        exit 1
    fi
fi

print_status "Starting BFF Services..."

# Start BFF services (these need to be created)
if [ -d "bff-services/auth-bff" ]; then
    start_service "auth-bff" "cd bff-services/auth-bff && npm run dev"
fi

if [ -d "bff-services/tenant-bff" ]; then
    start_service "tenant-bff" "cd bff-services/tenant-bff && npm run dev"
fi

if [ -d "bff-services/file-bff" ]; then
    start_service "file-bff" "cd bff-services/file-bff && cargo run"
fi

if [ -d "bff-services/user-bff" ]; then
    start_service "user-bff" "cd bff-services/user-bff && cargo run"
fi

if [ -d "bff-services/workflow-bff" ]; then
    start_service "workflow-bff" "cd bff-services/workflow-bff && cargo run"
fi

print_status "Starting Frontend Microservices..."

# Start Shell Application
if [ -d "micro-frontends/shell" ]; then
    start_service "shell" "cd micro-frontends/shell && npm run dev"
else
    print_warning "Shell application not found. Starting legacy frontend..."
    start_service "legacy-frontend" "cd frontend && npm run dev"
fi

# Start Micro-Frontends
MICRO_FRONTENDS=("auth" "tenant" "file" "user" "workflow" "dashboard")

for mf in "${MICRO_FRONTENDS[@]}"; do
    if [ -d "micro-frontends/${mf}-micro-app" ]; then
        start_service "${mf}-micro-app" "cd micro-frontends/${mf}-micro-app && npm run dev"
    else
        print_warning "${mf}-micro-app not found, skipping..."
    fi
done

# Wait a bit for services to start
sleep 5

print_status "Checking service health..."

# Check Shell/Frontend
if curl -s http://localhost:3000 >/dev/null 2>&1; then
    print_service "✅ Shell Application: http://localhost:3000"
else
    print_service "❌ Shell Application: http://localhost:3000 (not responding)"
fi

# Check Micro-Frontends
PORTS=(3001 3002 3003 3004 3005 3006)
NAMES=("Auth" "Tenant" "File" "User" "Workflow" "Dashboard")

for i in "${!PORTS[@]}"; do
    local port=${PORTS[$i]}
    local name=${NAMES[$i]}
    
    if curl -s "http://localhost:$port" >/dev/null 2>&1; then
        print_service "✅ $name Micro-App: http://localhost:$port"
    else
        print_service "❌ $name Micro-App: http://localhost:$port (not responding)"
    fi
done

# Check BFF Services
BFF_PORTS=(4001 4002 4003 4004 4005 4006)
BFF_NAMES=("Auth BFF" "Tenant BFF" "File BFF" "User BFF" "Workflow BFF" "Dashboard BFF")

for i in "${!BFF_PORTS[@]}"; do
    local port=${BFF_PORTS[$i]}
    local name=${BFF_NAMES[$i]}
    
    if curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
        print_service "✅ $name: http://localhost:$port"
    else
        print_service "❌ $name: http://localhost:$port (not responding)"
    fi
done

print_status ""
print_status "🚀 Frontend Development Environment Started!"
print_status "=========================================="
print_status ""
print_status "Main Application: http://localhost:3000"
print_status ""
print_status "Micro-Frontends:"
print_status "  Auth:      http://localhost:3001"
print_status "  Tenant:    http://localhost:3002"
print_status "  File:      http://localhost:3003"
print_status "  User:      http://localhost:3004"
print_status "  Workflow:  http://localhost:3005"
print_status "  Dashboard: http://localhost:3006"
print_status ""
print_status "BFF Services:"
print_status "  Auth BFF:      http://localhost:4001"
print_status "  Tenant BFF:    http://localhost:4002"
print_status "  File BFF:      http://localhost:4003"
print_status "  User BFF:      http://localhost:4004"
print_status "  Workflow BFF:  http://localhost:4005"
print_status "  Dashboard BFF: http://localhost:4006"
print_status ""
print_status "Logs are available in: logs/frontend/"
print_status ""
print_status "Press Ctrl+C to stop all services"

# Keep the script running and wait for interrupt
while true; do
    sleep 1
done