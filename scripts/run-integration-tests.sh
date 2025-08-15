#!/bin/bash

# ADX CORE Integration Test Runner
# This script sets up the environment and runs comprehensive end-to-end tests

set -e

echo "🚀 ADX CORE Integration Test Runner"
echo "===================================="

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DIR="$PROJECT_ROOT/tests"

# Default configuration
ENABLE_LOAD_TESTING=${ENABLE_LOAD_TESTING:-false}
MAX_CONCURRENT_USERS=${MAX_CONCURRENT_USERS:-100}
TEST_DURATION_SECONDS=${TEST_DURATION_SECONDS:-300}
CLEANUP_ON_EXIT=${CLEANUP_ON_EXIT:-true}
VERBOSE=${VERBOSE:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    if [ "$CLEANUP_ON_EXIT" = "true" ]; then
        log_info "Cleaning up test environment..."
        
        # Stop Docker containers
        docker-compose -f "$PROJECT_ROOT/adx-core/infrastructure/docker/docker-compose.dev.yml" down || true
        docker stop temporal-test 2>/dev/null || true
        docker rm temporal-test 2>/dev/null || true
        
        # Kill any remaining processes
        pkill -f "cargo run --bin" || true
        pkill -f "npm run dev" || true
        
        log_success "Cleanup completed"
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --enable-load-testing)
            ENABLE_LOAD_TESTING=true
            shift
            ;;
        --max-users)
            MAX_CONCURRENT_USERS="$2"
            shift 2
            ;;
        --test-duration)
            TEST_DURATION_SECONDS="$2"
            shift 2
            ;;
        --no-cleanup)
            CLEANUP_ON_EXIT=false
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --enable-load-testing    Enable load testing (default: false)"
            echo "  --max-users NUM         Maximum concurrent users for load testing (default: 100)"
            echo "  --test-duration SEC     Test duration in seconds (default: 300)"
            echo "  --no-cleanup            Don't cleanup on exit (default: cleanup)"
            echo "  --verbose               Enable verbose output (default: false)"
            echo "  --help                  Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Print configuration
log_info "Configuration:"
log_info "  Load Testing: $ENABLE_LOAD_TESTING"
log_info "  Max Concurrent Users: $MAX_CONCURRENT_USERS"
log_info "  Test Duration: ${TEST_DURATION_SECONDS}s"
log_info "  Cleanup on Exit: $CLEANUP_ON_EXIT"
log_info "  Verbose: $VERBOSE"
echo ""

# Check prerequisites
log_info "Checking prerequisites..."

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    log_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if required tools are installed
for tool in cargo npm node docker-compose; do
    if ! command -v $tool >/dev/null 2>&1; then
        log_error "$tool is not installed. Please install it and try again."
        exit 1
    fi
done

log_success "Prerequisites check passed"

# Set environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
export REDIS_URL="redis://localhost:6379"
export TEMPORAL_URL="http://localhost:7233"
export API_GATEWAY_URL="http://localhost:8080"
export FRONTEND_SHELL_URL="http://localhost:3000"
export ENABLE_LOAD_TESTING="$ENABLE_LOAD_TESTING"
export MAX_CONCURRENT_USERS="$MAX_CONCURRENT_USERS"
export TEST_DURATION_SECONDS="$TEST_DURATION_SECONDS"

# Start infrastructure services
log_info "Starting infrastructure services..."

# Start database and Redis
docker-compose -f "$PROJECT_ROOT/adx-core/infrastructure/docker/docker-compose.dev.yml" up -d postgres redis

# Wait for database to be ready
log_info "Waiting for database to be ready..."
for i in {1..30}; do
    if docker exec $(docker-compose -f "$PROJECT_ROOT/adx-core/infrastructure/docker/docker-compose.dev.yml" ps -q postgres) pg_isready -U postgres >/dev/null 2>&1; then
        break
    fi
    if [ $i -eq 30 ]; then
        log_error "Database failed to start within 30 seconds"
        exit 1
    fi
    sleep 1
done

# Start Temporal
log_info "Starting Temporal server..."
docker run -d --rm \
    --name temporal-test \
    -p 7233:7233 \
    -p 8088:8088 \
    temporalio/auto-setup:latest

# Wait for Temporal to be ready
log_info "Waiting for Temporal to be ready..."
for i in {1..60}; do
    if curl -f http://localhost:8088 >/dev/null 2>&1; then
        break
    fi
    if [ $i -eq 60 ]; then
        log_error "Temporal failed to start within 60 seconds"
        exit 1
    fi
    sleep 1
done

log_success "Infrastructure services started"

# Build and start backend services
log_info "Building and starting backend services..."

cd "$PROJECT_ROOT/adx-core"

# Build all services
if [ "$VERBOSE" = "true" ]; then
    cargo build --workspace
else
    cargo build --workspace >/dev/null 2>&1
fi

# Start services in background
services=("api-gateway:8080" "auth-service:8081" "user-service:8082" "file-service:8083" "workflow-service:8084" "tenant-service:8085")

for service_port in "${services[@]}"; do
    IFS=':' read -r service port <<< "$service_port"
    log_info "Starting $service on port $port..."
    
    if [ "$VERBOSE" = "true" ]; then
        cargo run --bin "$service" &
    else
        cargo run --bin "$service" >/dev/null 2>&1 &
    fi
    
    # Store PID for cleanup
    echo $! >> /tmp/adx_core_test_pids
done

# Wait for services to be ready
log_info "Waiting for backend services to be ready..."
for service_port in "${services[@]}"; do
    IFS=':' read -r service port <<< "$service_port"
    
    for i in {1..60}; do
        if curl -f "http://localhost:$port/health" >/dev/null 2>&1; then
            log_success "$service is ready"
            break
        fi
        if [ $i -eq 60 ]; then
            log_error "$service failed to start within 60 seconds"
            exit 1
        fi
        sleep 1
    done
done

# Start frontend services
log_info "Starting frontend services..."

cd "$PROJECT_ROOT"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    log_info "Installing frontend dependencies..."
    if [ "$VERBOSE" = "true" ]; then
        npm install
    else
        npm install >/dev/null 2>&1
    fi
fi

# Start micro-frontends
frontends=("shell:3000" "auth:3001" "tenant:3002" "file:3003" "user:3004" "workflow:3005")

for frontend_port in "${frontends[@]}"; do
    IFS=':' read -r frontend port <<< "$frontend_port"
    log_info "Starting $frontend micro-frontend on port $port..."
    
    cd "$PROJECT_ROOT/apps/$frontend"
    
    if [ "$VERBOSE" = "true" ]; then
        PORT=$port npm run dev &
    else
        PORT=$port npm run dev >/dev/null 2>&1 &
    fi
    
    # Store PID for cleanup
    echo $! >> /tmp/adx_core_test_pids
    
    cd "$PROJECT_ROOT"
done

# Wait for frontend services to be ready
log_info "Waiting for frontend services to be ready..."
for frontend_port in "${frontends[@]}"; do
    IFS=':' read -r frontend port <<< "$frontend_port"
    
    for i in {1..60}; do
        if curl -f "http://localhost:$port" >/dev/null 2>&1; then
            log_success "$frontend micro-frontend is ready"
            break
        fi
        if [ $i -eq 60 ]; then
            log_warning "$frontend micro-frontend failed to start within 60 seconds (continuing anyway)"
            break
        fi
        sleep 1
    done
done

log_success "All services started successfully"

# Run database migrations
log_info "Running database migrations..."
cd "$PROJECT_ROOT/adx-core"
if [ "$VERBOSE" = "true" ]; then
    sqlx migrate run
else
    sqlx migrate run >/dev/null 2>&1
fi

# Build and run integration tests
log_info "Building and running integration tests..."
cd "$TEST_DIR"

# Build tests
if [ "$VERBOSE" = "true" ]; then
    cargo build --bin integration_tests
else
    cargo build --bin integration_tests >/dev/null 2>&1
fi

# Run tests
log_info "Executing integration tests..."
echo ""

if [ "$VERBOSE" = "true" ]; then
    cargo run --bin integration_tests
else
    cargo run --bin integration_tests 2>&1
fi

TEST_EXIT_CODE=$?

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    log_success "All integration tests passed! 🎉"
else
    log_error "Some integration tests failed. Check the output above for details."
fi

# Display test reports
if [ -f "integration_test_report.html" ]; then
    log_info "Test reports generated:"
    log_info "  HTML Report: file://$(pwd)/integration_test_report.html"
    log_info "  JSON Report: $(pwd)/integration_test_report.json"
fi

# Cleanup will be handled by the trap
exit $TEST_EXIT_CODE