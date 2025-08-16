#!/bin/bash

# ADX Core Comprehensive Test Suite
# This script runs all tests across the entire ADX Core platform

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
PARALLEL_TESTS=${PARALLEL_TESTS:-true}
VERBOSE=${VERBOSE:-false}
SKIP_SETUP=${SKIP_SETUP:-false}
TEST_TIMEOUT=${TEST_TIMEOUT:-300}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADX_CORE_DIR="$PROJECT_ROOT/adx-core"

# Log file
LOG_FILE="$PROJECT_ROOT/test-results-$(date +%Y%m%d-%H%M%S).log"

# Test results tracking
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

run_test_suite() {
    local suite_name="$1"
    local test_command="$2"
    local timeout="${3:-$TEST_TIMEOUT}"
    
    log "Running $suite_name..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if timeout "$timeout" bash -c "$test_command" >> "$LOG_FILE" 2>&1; then
        log_success "$suite_name completed successfully"
        TEST_RESULTS["$suite_name"]="PASSED"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        log_error "$suite_name failed"
        TEST_RESULTS["$suite_name"]="FAILED"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/package.json" ]] || [[ ! -d "$ADX_CORE_DIR" ]]; then
        log_error "Not in ADX Core project root directory"
        exit 1
    fi
    
    # Check required tools
    local required_tools=("cargo" "node" "npm" "docker" "docker-compose")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool '$tool' is not installed"
            exit 1
        fi
    done
    
    # Check Rust toolchain
    if ! cargo --version | grep -q "1."; then
        log_error "Rust toolchain not properly installed"
        exit 1
    fi
    
    # Check Node.js version
    local node_version=$(node --version | sed 's/v//')
    local required_node_version="18.0.0"
    if ! printf '%s\n%s\n' "$required_node_version" "$node_version" | sort -V -C; then
        log_error "Node.js version $node_version is below required $required_node_version"
        exit 1
    fi
    
    log_success "All prerequisites met"
}

setup_test_environment() {
    if [[ "$SKIP_SETUP" == "true" ]]; then
        log "Skipping test environment setup"
        return 0
    fi
    
    log "Setting up test environment..."
    
    # Set test environment variables
    export NODE_ENV=test
    export RUST_LOG=debug
    export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
    export REDIS_URL="redis://localhost:6379"
    export TEMPORAL_SERVER_URL="localhost:7233"
    
    # Start infrastructure services
    log "Starting infrastructure services..."
    cd "$ADX_CORE_DIR"
    
    if ! docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d postgres redis temporal; then
        log_error "Failed to start infrastructure services"
        return 1
    fi
    
    # Wait for services to be ready
    log "Waiting for services to be ready..."
    sleep 30
    
    # Run database migrations
    log "Running database migrations..."
    if ! cargo run --bin db-manager -- migrate --database-url "$DATABASE_URL"; then
        log_error "Database migrations failed"
        return 1
    fi
    
    # Seed test data
    log "Seeding test data..."
    if ! cargo run --bin db-manager -- seed --environment test --database-url "$DATABASE_URL"; then
        log_error "Database seeding failed"
        return 1
    fi
    
    cd "$PROJECT_ROOT"
    
    # Install frontend dependencies
    log "Installing frontend dependencies..."
    if ! npm ci; then
        log_error "Frontend dependency installation failed"
        return 1
    fi
    
    log_success "Test environment setup completed"
}

run_backend_tests() {
    log "=== Running Backend Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Unit tests
    run_test_suite "Backend Unit Tests" \
        "cargo test --workspace --lib --verbose" \
        600
    
    # Integration tests
    run_test_suite "Backend Integration Tests" \
        "cargo test --workspace --test integration_tests --verbose" \
        900
    
    # Workflow tests
    run_test_suite "Temporal Workflow Tests" \
        "cargo test --workspace --test workflow_tests --verbose" \
        600
    
    # Cross-service tests
    run_test_suite "Cross-Service Tests" \
        "cargo test --manifest-path ../tests/Cargo.toml cross_service_tests --verbose" \
        900
    
    # Security tests
    run_test_suite "Security Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --manifest-path ../tests/Cargo.toml security_tests --verbose" \
        600
    
    cd "$PROJECT_ROOT"
}

run_frontend_tests() {
    log "=== Running Frontend Tests ==="
    
    # TypeScript unit tests
    run_test_suite "Frontend Unit Tests" \
        "npm run test:unit" \
        300
    
    # Frontend integration tests
    run_test_suite "Frontend Integration Tests" \
        "npm run test:integration" \
        600
    
    # Micro-frontend tests
    run_test_suite "Micro-Frontend Tests" \
        "npm run test:e2e:microfrontends" \
        900
    
    # i18n tests
    run_test_suite "Internationalization Tests" \
        "cd packages/i18n && npm test" \
        300
}

run_e2e_tests() {
    log "=== Running End-to-End Tests ==="
    
    # Start all services for E2E tests
    log "Starting all services for E2E tests..."
    if ! ./scripts/dev-start-all.sh --test-mode; then
        log_warning "Failed to start all services, skipping E2E tests"
        return 0
    fi
    
    # Wait for services to be ready
    sleep 60
    
    # Cross-platform E2E tests
    run_test_suite "Cross-Platform E2E Tests" \
        "npm run test:e2e" \
        1200
    
    # Desktop integration tests
    if command -v cargo-tauri &> /dev/null; then
        run_test_suite "Desktop Integration Tests" \
            "npm run test:desktop:integration" \
            900
    else
        log_warning "Tauri not installed, skipping desktop tests"
    fi
    
    # Mobile compatibility tests (if available)
    if [[ "$CI" != "true" ]]; then
        run_test_suite "Mobile Compatibility Tests" \
            "npm run test:cross-platform:compatibility" \
            600
    else
        log "Skipping mobile tests in CI environment"
    fi
}

run_performance_tests() {
    log "=== Running Performance Tests ==="
    
    # Backend performance tests
    run_test_suite "Backend Performance Tests" \
        "RUN_PERFORMANCE_TESTS=1 cargo test --manifest-path tests/Cargo.toml performance_tests --release" \
        1800
    
    # Frontend performance tests
    run_test_suite "Frontend Performance Tests" \
        "npm run test:performance:cross-platform" \
        900
    
    # Load testing
    run_test_suite "Load Testing" \
        "cd adx-core && cargo test --release load_testing" \
        1200
}

validate_system_health() {
    log "=== Validating System Health ==="
    
    # Database health
    run_test_suite "Database Health Check" \
        "cd adx-core && cargo run --bin db-manager -- health-check --database-url '$DATABASE_URL'" \
        60
    
    # Service connectivity
    run_test_suite "Service Connectivity Check" \
        "./scripts/health-check.sh" \
        120
    
    # API Gateway health
    run_test_suite "API Gateway Health" \
        "curl -f http://localhost:8080/health || exit 1" \
        30
    
    # Temporal health
    run_test_suite "Temporal Health Check" \
        "curl -f http://localhost:8088/api/v1/namespaces || exit 1" \
        30
}

generate_test_report() {
    log "=== Generating Test Report ==="
    
    local report_file="$PROJECT_ROOT/test-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX Core Test Report

**Generated:** $(date)
**Total Tests:** $TOTAL_TESTS
**Passed:** $PASSED_TESTS
**Failed:** $FAILED_TESTS
**Success Rate:** $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%

## Test Results

EOF

    for test_suite in "${!TEST_RESULTS[@]}"; do
        local status="${TEST_RESULTS[$test_suite]}"
        local icon="❌"
        if [[ "$status" == "PASSED" ]]; then
            icon="✅"
        fi
        echo "- $icon **$test_suite**: $status" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## System Information

- **OS:** $(uname -s) $(uname -r)
- **Rust Version:** $(rustc --version)
- **Node.js Version:** $(node --version)
- **Docker Version:** $(docker --version)
- **Git Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "N/A")

## Log File

Full test logs available at: \`$LOG_FILE\`

EOF

    log_success "Test report generated: $report_file"
    
    # Display summary
    echo
    echo "=================================="
    echo "         TEST SUMMARY"
    echo "=================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo "=================================="
    
    if [[ $FAILED_TESTS -gt 0 ]]; then
        echo
        log_error "Some tests failed. Check the log file for details: $LOG_FILE"
        return 1
    else
        echo
        log_success "All tests passed!"
        return 0
    fi
}

cleanup_test_environment() {
    log "Cleaning up test environment..."
    
    # Stop services
    cd "$ADX_CORE_DIR"
    docker-compose -f infrastructure/docker/docker-compose.dev.yml down || true
    
    # Clean test data
    if [[ "$DATABASE_URL" == *"test"* ]]; then
        cargo run --bin db-manager -- clean --database-url "$DATABASE_URL" || true
    fi
    
    cd "$PROJECT_ROOT"
    
    log_success "Cleanup completed"
}

main() {
    log "Starting ADX Core Comprehensive Test Suite"
    log "Log file: $LOG_FILE"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-setup)
                SKIP_SETUP=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --parallel)
                PARALLEL_TESTS=true
                shift
                ;;
            --timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --skip-setup    Skip test environment setup"
                echo "  --verbose       Enable verbose output"
                echo "  --parallel      Run tests in parallel (default: true)"
                echo "  --timeout SEC   Set test timeout in seconds (default: 300)"
                echo "  --help          Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Trap cleanup on exit
    trap cleanup_test_environment EXIT
    
    # Run test phases
    check_prerequisites
    setup_test_environment
    
    # Run test suites
    run_backend_tests
    run_frontend_tests
    validate_system_health
    
    # Run E2E and performance tests if not in CI
    if [[ "$CI" != "true" ]]; then
        run_e2e_tests
        run_performance_tests
    else
        log "Skipping E2E and performance tests in CI environment"
    fi
    
    # Generate report and exit with appropriate code
    if generate_test_report; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"