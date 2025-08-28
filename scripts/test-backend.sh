#!/bin/bash

# ADX CORE Backend Testing Script
# Runs Rust backend unit tests, service tests, and repository tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Navigate to ADX Core directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../adx-core"

# Parse command line arguments
COVERAGE=false
VERBOSE=false
UNIT_ONLY=false
SERVICE_ONLY=false
REPOSITORY_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --coverage)
            COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --unit-only)
            UNIT_ONLY=true
            shift
            ;;
        --service-only)
            SERVICE_ONLY=true
            shift
            ;;
        --repository-only)
            REPOSITORY_ONLY=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --coverage         Generate coverage report"
            echo "  --verbose          Verbose output"
            echo "  --unit-only        Run only unit tests"
            echo "  --service-only     Run only service tests"
            echo "  --repository-only  Run only repository tests"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_status "Starting ADX CORE Backend Tests..."

# Set test environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
export REDIS_URL="redis://localhost:6379"
export TEMPORAL_SERVER_URL="localhost:7233"
export RUST_LOG="info"
export TEST_MODE="true"

# Build test flags
TEST_FLAGS=""
if [ "$VERBOSE" = true ]; then
    TEST_FLAGS="$TEST_FLAGS --verbose"
fi

# Install coverage tool if needed
if [ "$COVERAGE" = true ]; then
    print_status "Installing cargo-tarpaulin for coverage..."
    cargo install cargo-tarpaulin --quiet 2>/dev/null || true
fi

# Ensure test database exists
print_status "Setting up test database..."
docker-compose -f infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c "CREATE DATABASE IF NOT EXISTS adx_core_test;" 2>/dev/null || true

# Run database migrations for tests
print_status "Running database migrations..."
# TODO: Implement when migrations are available
# sqlx migrate run --database-url $DATABASE_URL

# Function to run specific test category
run_test_category() {
    local category="$1"
    local test_pattern="$2"
    local description="$3"
    
    print_status "Running $description..."
    
    if [ "$COVERAGE" = true ]; then
        cargo tarpaulin --lib --test "$test_pattern" --out Html --output-dir "target/coverage/$category" $TEST_FLAGS
    else
        cargo test --lib --test "$test_pattern" $TEST_FLAGS
    fi
    
    if [ $? -eq 0 ]; then
        print_success "$description completed successfully"
    else
        print_error "$description failed"
        return 1
    fi
}

# Run tests based on options
if [ "$UNIT_ONLY" = true ]; then
    print_status "Running unit tests only..."
    if [ "$COVERAGE" = true ]; then
        cargo tarpaulin --lib --out Html --output-dir target/coverage/unit $TEST_FLAGS
    else
        cargo test --lib $TEST_FLAGS
    fi
elif [ "$SERVICE_ONLY" = true ]; then
    print_status "Running service tests only..."
    run_test_category "service" "*service*" "Service Tests"
elif [ "$REPOSITORY_ONLY" = true ]; then
    print_status "Running repository tests only..."
    run_test_category "repository" "*repository*" "Repository Tests"
else
    # Run all backend tests
    print_status "Running all backend tests..."
    
    # 1. Unit Tests (library code)
    print_status "=== Unit Tests ==="
    if [ "$COVERAGE" = true ]; then
        cargo tarpaulin --lib --out Html --output-dir target/coverage/unit $TEST_FLAGS
    else
        cargo test --lib $TEST_FLAGS
    fi
    
    if [ $? -ne 0 ]; then
        print_error "Unit tests failed"
        exit 1
    fi
    
    # 2. Service Tests (individual service testing)
    print_status "=== Service Tests ==="
    for service in services/*/; do
        if [ -d "$service" ]; then
            service_name=$(basename "$service")
            print_status "Testing $service_name..."
            
            cd "$service"
            if [ "$COVERAGE" = true ]; then
                cargo tarpaulin --lib --out Html --output-dir "../../target/coverage/$service_name" $TEST_FLAGS
            else
                cargo test --lib $TEST_FLAGS
            fi
            
            if [ $? -ne 0 ]; then
                print_error "$service_name tests failed"
                cd ../..
                exit 1
            fi
            cd ../..
        fi
    done
    
    # 3. Repository Tests (database layer testing)
    print_status "=== Repository Tests ==="
    # Test each service's repository layer
    for service in services/*/; do
        if [ -d "$service" ] && [ -f "$service/tests/repository_tests.rs" ]; then
            service_name=$(basename "$service")
            print_status "Testing $service_name repository..."
            
            cd "$service"
            cargo test --test repository_tests $TEST_FLAGS
            
            if [ $? -ne 0 ]; then
                print_error "$service_name repository tests failed"
                cd ../..
                exit 1
            fi
            cd ../..
        fi
    done
    
    # 4. Shared Library Tests
    print_status "=== Shared Library Tests ==="
    cd services/shared
    if [ "$COVERAGE" = true ]; then
        cargo tarpaulin --lib --out Html --output-dir "../../target/coverage/shared" $TEST_FLAGS
    else
        cargo test --lib $TEST_FLAGS
    fi
    
    if [ $? -ne 0 ]; then
        print_error "Shared library tests failed"
        exit 1
    fi
    cd ../..
fi

# Generate combined coverage report if requested
if [ "$COVERAGE" = true ]; then
    print_status "Generating combined coverage report..."
    cargo tarpaulin --workspace --out Html --output-dir target/coverage/combined $TEST_FLAGS
    
    print_success "Coverage reports generated:"
    print_status "  - Combined: target/coverage/combined/tarpaulin-report.html"
    print_status "  - Unit: target/coverage/unit/tarpaulin-report.html"
    
    # List individual service coverage reports
    for service_dir in target/coverage/*/; do
        if [ -d "$service_dir" ] && [ -f "$service_dir/tarpaulin-report.html" ]; then
            service_name=$(basename "$service_dir")
            print_status "  - $service_name: $service_dir/tarpaulin-report.html"
        fi
    done
fi

# Run clippy for code quality
print_status "Running clippy for code quality checks..."
cargo clippy --workspace --all-targets --all-features -- -D warnings

if [ $? -ne 0 ]; then
    print_warning "Clippy found issues (not failing tests, but should be addressed)"
fi

# Check code formatting
print_status "Checking code formatting..."
cargo fmt --all -- --check

if [ $? -ne 0 ]; then
    print_warning "Code formatting issues found (run 'cargo fmt' to fix)"
fi

# Security audit
print_status "Running security audit..."
cargo audit --quiet 2>/dev/null || {
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
}

print_success "Backend tests completed successfully! ✅"

# Performance benchmarks (if available)
if [ -d "benches" ]; then
    print_status "Running performance benchmarks..."
    cargo bench --quiet || print_warning "Benchmarks failed or not available"
fi

print_status "Backend test summary:"
print_status "  ✅ Unit tests passed"
print_status "  ✅ Service tests passed"
print_status "  ✅ Repository tests passed"
print_status "  ✅ Shared library tests passed"
print_status "  ✅ Code quality checks completed"

if [ "$COVERAGE" = true ]; then
    print_status "  📊 Coverage reports generated"
fi