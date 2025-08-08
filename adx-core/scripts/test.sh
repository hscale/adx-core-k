#!/bin/bash

# ADX CORE Testing Script
# This script runs comprehensive tests across all services

set -e

echo "🧪 Running ADX CORE Test Suite..."

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

# Navigate to workspace root
cd "$(dirname "$0")/.."

# Parse command line arguments
TEST_TYPE="all"
COVERAGE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit)
            TEST_TYPE="unit"
            shift
            ;;
        --integration)
            TEST_TYPE="integration"
            shift
            ;;
        --workflow)
            TEST_TYPE="workflow"
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --unit         Run only unit tests"
            echo "  --integration  Run only integration tests"
            echo "  --workflow     Run only workflow tests"
            echo "  --coverage     Generate coverage report"
            echo "  --verbose      Verbose output"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Set up test environment
print_status "Setting up test environment..."

# Check if test infrastructure is running
if ! docker-compose -f infrastructure/docker/docker-compose.dev.yml ps | grep -q "Up"; then
    print_warning "Test infrastructure not running. Starting minimal test services..."
    docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d postgres redis
    
    # Wait for services
    print_status "Waiting for test services..."
    sleep 10
fi

# Set test environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
export REDIS_URL="redis://localhost:6379"
export TEMPORAL_SERVER_URL="localhost:7233"
export RUST_LOG="debug"
export TEST_MODE="true"

# Create test database if it doesn't exist
print_status "Setting up test database..."
docker-compose -f infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c "CREATE DATABASE adx_core_test;" 2>/dev/null || true

# Run database migrations for tests
print_status "Running database migrations..."
# Note: This would run migrations when they're implemented
# sqlx migrate run --database-url $DATABASE_URL

# Build test flags
TEST_FLAGS=""
if [ "$VERBOSE" = true ]; then
    TEST_FLAGS="$TEST_FLAGS --verbose"
fi

if [ "$COVERAGE" = true ]; then
    print_status "Installing cargo-tarpaulin for coverage..."
    cargo install cargo-tarpaulin --quiet || true
fi

# Run tests based on type
case $TEST_TYPE in
    "unit")
        print_status "Running unit tests..."
        if [ "$COVERAGE" = true ]; then
            cargo tarpaulin --lib --out Html --output-dir target/coverage $TEST_FLAGS
        else
            cargo test --lib $TEST_FLAGS
        fi
        ;;
    "integration")
        print_status "Running integration tests..."
        cargo test --test integration_tests $TEST_FLAGS
        ;;
    "workflow")
        print_status "Running workflow tests..."
        cargo test --test workflow_tests $TEST_FLAGS
        ;;
    "all")
        print_status "Running all tests..."
        if [ "$COVERAGE" = true ]; then
            cargo tarpaulin --out Html --output-dir target/coverage $TEST_FLAGS
        else
            cargo test $TEST_FLAGS
        fi
        ;;
esac

# Check test results
if [ $? -eq 0 ]; then
    print_success "All tests passed! ✅"
    
    if [ "$COVERAGE" = true ]; then
        print_status "Coverage report generated at: target/coverage/tarpaulin-report.html"
    fi
else
    print_error "Some tests failed! ❌"
    exit 1
fi

print_status "Test run completed."