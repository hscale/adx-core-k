#!/bin/bash

# ADX CORE Comprehensive Testing Script
# Runs all test suites: backend, frontend, workflows, integration, and e2e tests

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
    echo -e "\n${PURPLE}=== $1 ===${NC}\n"
}

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Parse command line arguments
BACKEND_TESTS=true
FRONTEND_TESTS=true
WORKFLOW_TESTS=true
INTEGRATION_TESTS=true
E2E_TESTS=true
COVERAGE=false
VERBOSE=false
PARALLEL=true
FAIL_FAST=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --backend-only)
            FRONTEND_TESTS=false
            WORKFLOW_TESTS=false
            INTEGRATION_TESTS=false
            E2E_TESTS=false
            shift
            ;;
        --frontend-only)
            BACKEND_TESTS=false
            WORKFLOW_TESTS=false
            INTEGRATION_TESTS=false
            E2E_TESTS=false
            shift
            ;;
        --workflow-only)
            BACKEND_TESTS=false
            FRONTEND_TESTS=false
            INTEGRATION_TESTS=false
            E2E_TESTS=false
            shift
            ;;
        --integration-only)
            BACKEND_TESTS=false
            FRONTEND_TESTS=false
            WORKFLOW_TESTS=false
            E2E_TESTS=false
            shift
            ;;
        --e2e-only)
            BACKEND_TESTS=false
            FRONTEND_TESTS=false
            WORKFLOW_TESTS=false
            INTEGRATION_TESTS=false
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
        --sequential)
            PARALLEL=false
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --backend-only     Run only backend tests"
            echo "  --frontend-only    Run only frontend tests"
            echo "  --workflow-only    Run only workflow tests"
            echo "  --integration-only Run only integration tests"
            echo "  --e2e-only         Run only end-to-end tests"
            echo "  --coverage         Generate coverage reports"
            echo "  --verbose          Verbose output"
            echo "  --sequential       Run tests sequentially (not in parallel)"
            echo "  --fail-fast        Stop on first test failure"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Test results tracking
BACKEND_RESULT=0
FRONTEND_RESULT=0
WORKFLOW_RESULT=0
INTEGRATION_RESULT=0
E2E_RESULT=0

# Create test results directory
mkdir -p target/test-results
TEST_TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
TEST_REPORT_DIR="target/test-results/run_$TEST_TIMESTAMP"
mkdir -p "$TEST_REPORT_DIR"

print_section "ADX CORE Comprehensive Test Suite"
print_status "Test run started at: $(date)"
print_status "Test report directory: $TEST_REPORT_DIR"

# Validate environment
print_section "Environment Validation"
./scripts/validate-setup.sh || {
    print_error "Environment validation failed. Please fix issues before running tests."
    exit 1
}

# Start test infrastructure if needed
print_section "Test Infrastructure Setup"
if ! docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml ps | grep -q "Up"; then
    print_status "Starting test infrastructure..."
    docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 30
    
    # Verify services are healthy
    ./scripts/health-check.sh || {
        print_error "Test infrastructure health check failed"
        exit 1
    }
fi

# Set test environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
export REDIS_URL="redis://localhost:6379"
export TEMPORAL_SERVER_URL="localhost:7233"
export RUST_LOG="info"
export TEST_MODE="true"
export NODE_ENV="test"

# Create test database
print_status "Setting up test database..."
docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c "DROP DATABASE IF EXISTS adx_core_test;" 2>/dev/null || true
docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml exec -T postgres psql -U postgres -c "CREATE DATABASE adx_core_test;" 2>/dev/null || true

# Function to run tests with proper error handling
run_test_suite() {
    local test_name="$1"
    local test_command="$2"
    local result_var="$3"
    
    print_section "$test_name"
    print_status "Running $test_name..."
    
    local start_time=$(date +%s)
    
    if [ "$VERBOSE" = true ]; then
        eval "$test_command" 2>&1 | tee "$TEST_REPORT_DIR/${test_name,,}_output.log"
        local exit_code=${PIPESTATUS[0]}
    else
        eval "$test_command" > "$TEST_REPORT_DIR/${test_name,,}_output.log" 2>&1
        local exit_code=$?
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [ $exit_code -eq 0 ]; then
        print_success "$test_name completed successfully in ${duration}s"
        echo "PASS" > "$TEST_REPORT_DIR/${test_name,,}_result.txt"
    else
        print_error "$test_name failed after ${duration}s"
        echo "FAIL" > "$TEST_REPORT_DIR/${test_name,,}_result.txt"
        
        if [ "$FAIL_FAST" = true ]; then
            print_error "Fail-fast mode enabled. Stopping test execution."
            exit $exit_code
        fi
    fi
    
    eval "$result_var=$exit_code"
}

# Run test suites
if [ "$PARALLEL" = true ] && [ "$BACKEND_TESTS" = true ] && [ "$FRONTEND_TESTS" = true ]; then
    print_section "Running Backend and Frontend Tests in Parallel"
    
    # Run backend tests in background
    if [ "$BACKEND_TESTS" = true ]; then
        (run_test_suite "Backend Tests" "./scripts/test-backend.sh $([ "$COVERAGE" = true ] && echo "--coverage") $([ "$VERBOSE" = true ] && echo "--verbose")" "BACKEND_RESULT") &
        BACKEND_PID=$!
    fi
    
    # Run frontend tests in background
    if [ "$FRONTEND_TESTS" = true ]; then
        (run_test_suite "Frontend Tests" "./scripts/test-frontend.sh $([ "$COVERAGE" = true ] && echo "--coverage") $([ "$VERBOSE" = true ] && echo "--verbose")" "FRONTEND_RESULT") &
        FRONTEND_PID=$!
    fi
    
    # Wait for parallel tests to complete
    if [ "$BACKEND_TESTS" = true ]; then
        wait $BACKEND_PID
        BACKEND_RESULT=$?
    fi
    
    if [ "$FRONTEND_TESTS" = true ]; then
        wait $FRONTEND_PID
        FRONTEND_RESULT=$?
    fi
else
    # Run tests sequentially
    if [ "$BACKEND_TESTS" = true ]; then
        run_test_suite "Backend Tests" "./scripts/test-backend.sh $([ "$COVERAGE" = true ] && echo "--coverage") $([ "$VERBOSE" = true ] && echo "--verbose")" "BACKEND_RESULT"
    fi
    
    if [ "$FRONTEND_TESTS" = true ]; then
        run_test_suite "Frontend Tests" "./scripts/test-frontend.sh $([ "$COVERAGE" = true ] && echo "--coverage") $([ "$VERBOSE" = true ] && echo "--verbose")" "FRONTEND_RESULT"
    fi
fi

# Run workflow tests
if [ "$WORKFLOW_TESTS" = true ]; then
    run_test_suite "Workflow Tests" "./scripts/test-workflows.sh $([ "$VERBOSE" = true ] && echo "--verbose")" "WORKFLOW_RESULT"
fi

# Run integration tests
if [ "$INTEGRATION_TESTS" = true ]; then
    run_test_suite "Integration Tests" "./scripts/test-integration.sh $([ "$VERBOSE" = true ] && echo "--verbose")" "INTEGRATION_RESULT"
fi

# Run end-to-end tests
if [ "$E2E_TESTS" = true ]; then
    run_test_suite "End-to-End Tests" "./scripts/test-e2e.sh $([ "$VERBOSE" = true ] && echo "--verbose")" "E2E_RESULT"
fi

# Generate comprehensive test report
print_section "Test Results Summary"

# Create summary report
cat > "$TEST_REPORT_DIR/summary.md" << EOF
# ADX CORE Test Run Summary

**Test Run:** $TEST_TIMESTAMP  
**Date:** $(date)  
**Duration:** $(($(date +%s) - $(date -d "$(stat -c %y "$TEST_REPORT_DIR")" +%s)))s

## Test Results

| Test Suite | Status | Details |
|------------|--------|---------|
EOF

# Add results to summary
if [ "$BACKEND_TESTS" = true ]; then
    if [ $BACKEND_RESULT -eq 0 ]; then
        echo "| Backend Tests | ✅ PASS | All backend unit and service tests passed |" >> "$TEST_REPORT_DIR/summary.md"
        print_success "Backend Tests: PASSED"
    else
        echo "| Backend Tests | ❌ FAIL | Backend tests failed - check backend_tests_output.log |" >> "$TEST_REPORT_DIR/summary.md"
        print_error "Backend Tests: FAILED"
    fi
fi

if [ "$FRONTEND_TESTS" = true ]; then
    if [ $FRONTEND_RESULT -eq 0 ]; then
        echo "| Frontend Tests | ✅ PASS | All frontend unit and component tests passed |" >> "$TEST_REPORT_DIR/summary.md"
        print_success "Frontend Tests: PASSED"
    else
        echo "| Frontend Tests | ❌ FAIL | Frontend tests failed - check frontend_tests_output.log |" >> "$TEST_REPORT_DIR/summary.md"
        print_error "Frontend Tests: FAILED"
    fi
fi

if [ "$WORKFLOW_TESTS" = true ]; then
    if [ $WORKFLOW_RESULT -eq 0 ]; then
        echo "| Workflow Tests | ✅ PASS | All Temporal workflow tests passed |" >> "$TEST_REPORT_DIR/summary.md"
        print_success "Workflow Tests: PASSED"
    else
        echo "| Workflow Tests | ❌ FAIL | Workflow tests failed - check workflow_tests_output.log |" >> "$TEST_REPORT_DIR/summary.md"
        print_error "Workflow Tests: FAILED"
    fi
fi

if [ "$INTEGRATION_TESTS" = true ]; then
    if [ $INTEGRATION_RESULT -eq 0 ]; then
        echo "| Integration Tests | ✅ PASS | All cross-service integration tests passed |" >> "$TEST_REPORT_DIR/summary.md"
        print_success "Integration Tests: PASSED"
    else
        echo "| Integration Tests | ❌ FAIL | Integration tests failed - check integration_tests_output.log |" >> "$TEST_REPORT_DIR/summary.md"
        print_error "Integration Tests: FAILED"
    fi
fi

if [ "$E2E_TESTS" = true ]; then
    if [ $E2E_RESULT -eq 0 ]; then
        echo "| End-to-End Tests | ✅ PASS | All end-to-end tests passed |" >> "$TEST_REPORT_DIR/summary.md"
        print_success "End-to-End Tests: PASSED"
    else
        echo "| End-to-End Tests | ❌ FAIL | E2E tests failed - check end-to-end_tests_output.log |" >> "$TEST_REPORT_DIR/summary.md"
        print_error "End-to-End Tests: FAILED"
    fi
fi

# Calculate overall result
OVERALL_RESULT=$((BACKEND_RESULT + FRONTEND_RESULT + WORKFLOW_RESULT + INTEGRATION_RESULT + E2E_RESULT))

echo "" >> "$TEST_REPORT_DIR/summary.md"
if [ $OVERALL_RESULT -eq 0 ]; then
    echo "## Overall Result: ✅ ALL TESTS PASSED" >> "$TEST_REPORT_DIR/summary.md"
    print_success "🎉 ALL TESTS PASSED! 🎉"
else
    echo "## Overall Result: ❌ SOME TESTS FAILED" >> "$TEST_REPORT_DIR/summary.md"
    print_error "❌ SOME TESTS FAILED"
fi

# Add coverage information if generated
if [ "$COVERAGE" = true ]; then
    echo "" >> "$TEST_REPORT_DIR/summary.md"
    echo "## Coverage Reports" >> "$TEST_REPORT_DIR/summary.md"
    echo "- Backend Coverage: \`target/coverage/tarpaulin-report.html\`" >> "$TEST_REPORT_DIR/summary.md"
    echo "- Frontend Coverage: \`coverage/lcov-report/index.html\`" >> "$TEST_REPORT_DIR/summary.md"
fi

# Display summary
print_section "Test Summary"
cat "$TEST_REPORT_DIR/summary.md"

print_status "Detailed test reports available in: $TEST_REPORT_DIR"
print_status "Test run completed at: $(date)"

# Exit with appropriate code
exit $OVERALL_RESULT