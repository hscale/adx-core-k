#!/bin/bash

# ADX Core Backend Test Suite
# Comprehensive testing for Rust backend services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VERBOSE=${VERBOSE:-false}
COVERAGE=${COVERAGE:-false}
PARALLEL=${PARALLEL:-true}
TEST_TIMEOUT=${TEST_TIMEOUT:-600}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADX_CORE_DIR="$PROJECT_ROOT/adx-core"

# Log file
LOG_FILE="$PROJECT_ROOT/backend-test-results-$(date +%Y%m%d-%H%M%S).log"

# Test results
declare -A TEST_RESULTS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Utility functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    local timeout="${3:-$TEST_TIMEOUT}"
    
    log "Running $test_name..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if timeout "$timeout" bash -c "$test_command" >> "$LOG_FILE" 2>&1; then
        log_success "$test_name completed"
        TEST_RESULTS["$test_name"]="PASSED"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$test_name failed"
        TEST_RESULTS["$test_name"]="FAILED"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

setup_backend_environment() {
    log "Setting up backend test environment..."
    
    cd "$ADX_CORE_DIR"
    
    # Set environment variables
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
    export REDIS_URL="redis://localhost:6379"
    export TEMPORAL_SERVER_URL="localhost:7233"
    
    # Start required services
    log "Starting infrastructure services..."
    docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d postgres redis temporal
    
    # Wait for services
    log "Waiting for services to be ready..."
    sleep 30
    
    # Verify database connection
    if ! cargo run --bin db-manager -- health --database-url "$DATABASE_URL"; then
        log_error "Database connection failed"
        return 1
    fi
    
    # Run migrations
    log "Running database migrations..."
    cargo run --bin db-manager -- migrate --database-url "$DATABASE_URL"
    
    # Seed test data
    log "Seeding test data..."
    cargo run --bin db-manager -- seed --environment test --database-url "$DATABASE_URL"
    
    log_success "Backend environment setup completed"
}

run_unit_tests() {
    log "=== Running Unit Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test shared library
    run_test "Shared Library Unit Tests" \
        "cargo test --package adx-shared --lib --verbose"
    
    # Test individual services (if they exist and compile)
    local services=("auth-service" "user-service" "tenant-service" "file-service" "workflow-service")
    
    for service in "${services[@]}"; do
        if [[ -d "services/$service" ]] && [[ -f "services/$service/Cargo.toml" ]]; then
            # Check if service is in workspace
            if grep -q "services/$service" Cargo.toml; then
                run_test "$service Unit Tests" \
                    "cargo test --package ${service//-/_} --lib --verbose"
            else
                log_warning "Skipping $service (not in workspace)"
            fi
        else
            log_warning "Service $service not found, skipping"
        fi
    done
    
    # Test all workspace members
    run_test "All Workspace Unit Tests" \
        "cargo test --workspace --lib --verbose"
}

run_integration_tests() {
    log "=== Running Integration Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Database integration tests
    run_test "Database Integration Tests" \
        "cargo test --test database_integration --verbose"
    
    # Service integration tests
    run_test "Service Integration Tests" \
        "cargo test --test service_integration --verbose"
    
    # Multi-tenant integration tests
    run_test "Multi-Tenant Integration Tests" \
        "cargo test --test multi_tenant_integration --verbose"
    
    # API integration tests
    run_test "API Integration Tests" \
        "cargo test --test api_integration --verbose"
}

run_workflow_tests() {
    log "=== Running Temporal Workflow Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Workflow unit tests
    run_test "Workflow Unit Tests" \
        "cargo test --test workflow_tests --verbose"
    
    # Activity tests
    run_test "Activity Tests" \
        "cargo test --test activity_tests --verbose"
    
    # Workflow integration tests
    run_test "Workflow Integration Tests" \
        "cargo test --test workflow_integration --verbose"
    
    # Compensation tests
    run_test "Workflow Compensation Tests" \
        "cargo test --test compensation_tests --verbose"
}

run_cross_service_tests() {
    log "=== Running Cross-Service Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Cross-service communication tests
    run_test "Cross-Service Communication Tests" \
        "cargo test --manifest-path tests/Cargo.toml cross_service_tests --verbose"
    
    # End-to-end workflow tests
    run_test "End-to-End Workflow Tests" \
        "cargo test --manifest-path tests/Cargo.toml e2e_workflow_tests --verbose"
    
    # Multi-tenant cross-service tests
    run_test "Multi-Tenant Cross-Service Tests" \
        "cargo test --manifest-path tests/Cargo.toml multi_tenant_cross_service --verbose"
}

run_security_tests() {
    log "=== Running Security Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Authentication tests
    run_test "Authentication Security Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --manifest-path tests/Cargo.toml auth_security_tests --verbose"
    
    # Authorization tests
    run_test "Authorization Security Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --manifest-path tests/Cargo.toml authz_security_tests --verbose"
    
    # Input validation tests
    run_test "Input Validation Security Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --manifest-path tests/Cargo.toml input_validation_tests --verbose"
    
    # SQL injection tests
    run_test "SQL Injection Security Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --manifest-path tests/Cargo.toml sql_injection_tests --verbose"
}

run_performance_tests() {
    log "=== Running Performance Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Database performance tests
    run_test "Database Performance Tests" \
        "RUN_PERFORMANCE_TESTS=1 cargo test --manifest-path tests/Cargo.toml --release db_performance_tests --verbose" \
        900
    
    # API performance tests
    run_test "API Performance Tests" \
        "RUN_PERFORMANCE_TESTS=1 cargo test --manifest-path tests/Cargo.toml --release api_performance_tests --verbose" \
        900
    
    # Workflow performance tests
    run_test "Workflow Performance Tests" \
        "RUN_PERFORMANCE_TESTS=1 cargo test --manifest-path tests/Cargo.toml --release workflow_performance_tests --verbose" \
        900
    
    # Load testing
    run_test "Load Testing" \
        "RUN_PERFORMANCE_TESTS=1 cargo test --manifest-path tests/Cargo.toml --release load_testing --verbose" \
        1200
}

run_code_quality_checks() {
    log "=== Running Code Quality Checks ==="
    
    cd "$ADX_CORE_DIR"
    
    # Clippy linting
    run_test "Clippy Linting" \
        "cargo clippy --workspace --all-targets --all-features -- -D warnings"
    
    # Format checking
    run_test "Format Checking" \
        "cargo fmt --all -- --check"
    
    # Audit dependencies
    if command -v cargo-audit &> /dev/null; then
        run_test "Security Audit" \
            "cargo audit"
    else
        log_warning "cargo-audit not installed, skipping security audit"
    fi
    
    # Check for unused dependencies
    if command -v cargo-udeps &> /dev/null; then
        run_test "Unused Dependencies Check" \
            "cargo +nightly udeps --workspace"
    else
        log_warning "cargo-udeps not installed, skipping unused dependencies check"
    fi
}

generate_coverage_report() {
    if [[ "$COVERAGE" != "true" ]]; then
        return 0
    fi
    
    log "=== Generating Coverage Report ==="
    
    cd "$ADX_CORE_DIR"
    
    if command -v cargo-tarpaulin &> /dev/null; then
        run_test "Coverage Report Generation" \
            "cargo tarpaulin --workspace --out Html --output-dir ../coverage-report" \
            1200
        
        log_success "Coverage report generated in coverage-report/"
    else
        log_warning "cargo-tarpaulin not installed, skipping coverage report"
    fi
}

validate_database_state() {
    log "=== Validating Database State ==="
    
    cd "$ADX_CORE_DIR"
    
    # Database integrity check
    run_test "Database Integrity Check" \
        "cargo run --bin db-manager -- validate --database-url '$DATABASE_URL'"
    
    # Connection pool monitoring
    run_test "Connection Pool Health" \
        "cargo run --bin db-manager -- monitor-pool --database-url '$DATABASE_URL'"
    
    # Index performance analysis
    run_test "Index Performance Analysis" \
        "cargo run --bin db-manager -- analyze-indexes --database-url '$DATABASE_URL'"
}

cleanup_backend_environment() {
    log "Cleaning up backend test environment..."
    
    cd "$ADX_CORE_DIR"
    
    # Clean test data
    if [[ "$DATABASE_URL" == *"test"* ]]; then
        cargo run --bin db-manager -- clean --database-url "$DATABASE_URL" || true
    fi
    
    # Stop services
    docker-compose -f infrastructure/docker/docker-compose.dev.yml down || true
    
    log_success "Backend cleanup completed"
}

generate_test_report() {
    log "=== Generating Backend Test Report ==="
    
    local report_file="$PROJECT_ROOT/backend-test-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX Core Backend Test Report

**Generated:** $(date)
**Total Tests:** $TOTAL_TESTS
**Passed:** $PASSED_TESTS
**Failed:** $FAILED_TESTS
**Success Rate:** $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%

## Test Results

EOF

    for test_name in "${!TEST_RESULTS[@]}"; do
        local status="${TEST_RESULTS[$test_name]}"
        local icon="❌"
        if [[ "$status" == "PASSED" ]]; then
            icon="✅"
        fi
        echo "- $icon **$test_name**: $status" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Environment Information

- **Rust Version:** $(rustc --version)
- **Cargo Version:** $(cargo --version)
- **Database:** PostgreSQL (Test)
- **Cache:** Redis
- **Workflow Engine:** Temporal
- **Git Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "N/A")

## Log File

Full test logs: \`$LOG_FILE\`

EOF

    log_success "Backend test report generated: $report_file"
    
    # Display summary
    echo
    echo "=================================="
    echo "      BACKEND TEST SUMMARY"
    echo "=================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo "=================================="
    
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some backend tests failed. Check log: $LOG_FILE"
        return 1
    else
        log_success "All backend tests passed!"
        return 0
    fi
}

main() {
    log "Starting ADX Core Backend Test Suite"
    log "Log file: $LOG_FILE"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --coverage)
                COVERAGE=true
                shift
                ;;
            --no-parallel)
                PARALLEL=false
                shift
                ;;
            --timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --verbose      Enable verbose output"
                echo "  --coverage     Generate coverage report"
                echo "  --no-parallel  Disable parallel testing"
                echo "  --timeout SEC  Set test timeout (default: 600)"
                echo "  --help         Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Trap cleanup
    trap cleanup_backend_environment EXIT
    
    # Run test phases
    setup_backend_environment
    
    run_unit_tests
    run_integration_tests
    run_workflow_tests
    run_cross_service_tests
    run_security_tests
    validate_database_state
    run_code_quality_checks
    
    # Performance tests (optional, time-consuming)
    if [[ "$CI" != "true" ]]; then
        run_performance_tests
    fi
    
    generate_coverage_report
    
    # Generate report and exit
    if generate_test_report; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"