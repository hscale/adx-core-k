#!/bin/bash

# ADX Core Temporal Workflow Test Suite
# Comprehensive testing for Temporal workflows and activities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VERBOSE=${VERBOSE:-false}
TEMPORAL_UI=${TEMPORAL_UI:-true}
TEST_TIMEOUT=${TEST_TIMEOUT:-900}
WORKFLOW_TIMEOUT=${WORKFLOW_TIMEOUT:-300}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADX_CORE_DIR="$PROJECT_ROOT/adx-core"

# Log file
LOG_FILE="$PROJECT_ROOT/workflow-test-results-$(date +%Y%m%d-%H%M%S).log"

# Test results
declare -A TEST_RESULTS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Workflow test scenarios
declare -A WORKFLOW_SCENARIOS=(
    ["create_tenant"]="Tenant Creation Workflow"
    ["switch_tenant"]="Tenant Switching Workflow"
    ["user_onboarding"]="User Onboarding Workflow"
    ["file_upload"]="File Upload Workflow"
    ["bulk_import"]="Bulk Data Import Workflow"
    ["module_installation"]="Module Installation Workflow"
    ["data_migration"]="Data Migration Workflow"
    ["backup_restore"]="Backup and Restore Workflow"
)

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

setup_temporal_environment() {
    log "Setting up Temporal workflow test environment..."
    
    cd "$ADX_CORE_DIR"
    
    # Set environment variables
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
    export REDIS_URL="redis://localhost:6379"
    export TEMPORAL_SERVER_URL="localhost:7233"
    export TEMPORAL_NAMESPACE="adx-core-test"
    
    # Start Temporal and dependencies
    log "Starting Temporal infrastructure..."
    docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d postgres redis temporal
    
    # Wait for services to be ready
    log "Waiting for Temporal server to be ready..."
    local max_attempts=30
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if curl -s http://localhost:8088/api/v1/namespaces > /dev/null 2>&1; then
            log_success "Temporal server is ready"
            break
        fi
        
        attempt=$((attempt + 1))
        log "Waiting for Temporal server... (attempt $attempt/$max_attempts)"
        sleep 10
    done
    
    if [[ $attempt -eq $max_attempts ]]; then
        log_error "Temporal server failed to start within timeout"
        return 1
    fi
    
    # Setup test namespace
    log "Setting up test namespace..."
    if ! curl -X POST http://localhost:8088/api/v1/namespaces \
        -H "Content-Type: application/json" \
        -d '{"name":"adx-core-test","description":"ADX Core test namespace"}' > /dev/null 2>&1; then
        log_warning "Test namespace might already exist"
    fi
    
    # Run database migrations and seeding
    log "Preparing database..."
    cargo run --bin db-manager -- migrate --database-url "$DATABASE_URL"
    cargo run --bin db-manager -- seed --environment test --database-url "$DATABASE_URL"
    
    log_success "Temporal environment setup completed"
}

run_workflow_unit_tests() {
    log "=== Running Workflow Unit Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test workflow definitions
    run_test "Workflow Definition Tests" \
        "cargo test --test workflow_definitions --verbose"
    
    # Test activity implementations
    run_test "Activity Implementation Tests" \
        "cargo test --test activity_implementations --verbose"
    
    # Test workflow state management
    run_test "Workflow State Management Tests" \
        "cargo test --test workflow_state --verbose"
    
    # Test workflow versioning
    run_test "Workflow Versioning Tests" \
        "cargo test --test workflow_versioning --verbose"
    
    # Test workflow replay
    run_test "Workflow Replay Tests" \
        "cargo test --test workflow_replay --verbose"
}

run_activity_tests() {
    log "=== Running Activity Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test tenant activities
    run_test "Tenant Activity Tests" \
        "cargo test --test tenant_activities --verbose"
    
    # Test user activities
    run_test "User Activity Tests" \
        "cargo test --test user_activities --verbose"
    
    # Test file activities
    run_test "File Activity Tests" \
        "cargo test --test file_activities --verbose"
    
    # Test notification activities
    run_test "Notification Activity Tests" \
        "cargo test --test notification_activities --verbose"
    
    # Test external service activities
    run_test "External Service Activity Tests" \
        "cargo test --test external_activities --verbose"
    
    # Test activity retry logic
    run_test "Activity Retry Logic Tests" \
        "cargo test --test activity_retry --verbose"
    
    # Test activity timeout handling
    run_test "Activity Timeout Tests" \
        "cargo test --test activity_timeout --verbose"
}

run_workflow_integration_tests() {
    log "=== Running Workflow Integration Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test complete workflow execution
    for workflow_key in "${!WORKFLOW_SCENARIOS[@]}"; do
        local workflow_name="${WORKFLOW_SCENARIOS[$workflow_key]}"
        run_test "$workflow_name Integration Test" \
            "cargo test --test workflow_integration test_${workflow_key}_workflow --verbose" \
            "$WORKFLOW_TIMEOUT"
    done
    
    # Test workflow cancellation
    run_test "Workflow Cancellation Tests" \
        "cargo test --test workflow_cancellation --verbose"
    
    # Test workflow signals
    run_test "Workflow Signal Tests" \
        "cargo test --test workflow_signals --verbose"
    
    # Test workflow queries
    run_test "Workflow Query Tests" \
        "cargo test --test workflow_queries --verbose"
}

run_compensation_tests() {
    log "=== Running Compensation Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test saga pattern implementation
    run_test "Saga Pattern Tests" \
        "cargo test --test saga_pattern --verbose"
    
    # Test compensation activities
    run_test "Compensation Activity Tests" \
        "cargo test --test compensation_activities --verbose"
    
    # Test rollback scenarios
    run_test "Rollback Scenario Tests" \
        "cargo test --test rollback_scenarios --verbose"
    
    # Test partial failure recovery
    run_test "Partial Failure Recovery Tests" \
        "cargo test --test partial_failure_recovery --verbose"
}

run_multi_tenant_workflow_tests() {
    log "=== Running Multi-Tenant Workflow Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test tenant isolation in workflows
    run_test "Tenant Isolation Workflow Tests" \
        "cargo test --test tenant_isolation_workflows --verbose"
    
    # Test cross-tenant operations
    run_test "Cross-Tenant Operation Tests" \
        "cargo test --test cross_tenant_workflows --verbose"
    
    # Test tenant-specific configurations
    run_test "Tenant Configuration Workflow Tests" \
        "cargo test --test tenant_config_workflows --verbose"
    
    # Test tenant data migration workflows
    run_test "Tenant Migration Workflow Tests" \
        "cargo test --test tenant_migration_workflows --verbose"
}

run_performance_workflow_tests() {
    log "=== Running Workflow Performance Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test concurrent workflow execution
    run_test "Concurrent Workflow Execution Tests" \
        "cargo test --test concurrent_workflows --release --verbose" \
        1200
    
    # Test workflow throughput
    run_test "Workflow Throughput Tests" \
        "cargo test --test workflow_throughput --release --verbose" \
        1200
    
    # Test large payload handling
    run_test "Large Payload Workflow Tests" \
        "cargo test --test large_payload_workflows --verbose" \
        600
    
    # Test long-running workflows
    run_test "Long-Running Workflow Tests" \
        "cargo test --test long_running_workflows --verbose" \
        1800
}

run_error_handling_tests() {
    log "=== Running Error Handling Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test activity failures
    run_test "Activity Failure Handling Tests" \
        "cargo test --test activity_failures --verbose"
    
    # Test network failures
    run_test "Network Failure Handling Tests" \
        "cargo test --test network_failures --verbose"
    
    # Test database failures
    run_test "Database Failure Handling Tests" \
        "cargo test --test database_failures --verbose"
    
    # Test external service failures
    run_test "External Service Failure Tests" \
        "cargo test --test external_service_failures --verbose"
    
    # Test timeout scenarios
    run_test "Timeout Scenario Tests" \
        "cargo test --test timeout_scenarios --verbose"
}

run_workflow_monitoring_tests() {
    log "=== Running Workflow Monitoring Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test workflow metrics
    run_test "Workflow Metrics Tests" \
        "cargo test --test workflow_metrics --verbose"
    
    # Test workflow tracing
    run_test "Workflow Tracing Tests" \
        "cargo test --test workflow_tracing --verbose"
    
    # Test workflow logging
    run_test "Workflow Logging Tests" \
        "cargo test --test workflow_logging --verbose"
    
    # Test workflow health checks
    run_test "Workflow Health Check Tests" \
        "cargo test --test workflow_health --verbose"
}

run_workflow_security_tests() {
    log "=== Running Workflow Security Tests ==="
    
    cd "$ADX_CORE_DIR"
    
    # Test workflow authorization
    run_test "Workflow Authorization Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --test workflow_authorization --verbose"
    
    # Test activity permissions
    run_test "Activity Permission Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --test activity_permissions --verbose"
    
    # Test data encryption in workflows
    run_test "Workflow Data Encryption Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --test workflow_encryption --verbose"
    
    # Test audit logging
    run_test "Workflow Audit Logging Tests" \
        "RUN_SECURITY_TESTS=1 cargo test --test workflow_audit --verbose"
}

validate_temporal_health() {
    log "=== Validating Temporal Health ==="
    
    # Check Temporal server health
    run_test "Temporal Server Health Check" \
        "curl -f http://localhost:8088/api/v1/namespaces" \
        30
    
    # Check namespace health
    run_test "Temporal Namespace Health Check" \
        "curl -f http://localhost:8088/api/v1/namespaces/adx-core-test" \
        30
    
    # Check workflow history
    run_test "Workflow History Validation" \
        "cargo test --test workflow_history_validation --verbose"
    
    # Check worker connectivity
    run_test "Worker Connectivity Check" \
        "cargo test --test worker_connectivity --verbose"
}

generate_workflow_report() {
    log "=== Generating Workflow Test Report ==="
    
    local report_file="$PROJECT_ROOT/workflow-test-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX Core Workflow Test Report

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

## Workflow Scenarios Tested

EOF

    for workflow_key in "${!WORKFLOW_SCENARIOS[@]}"; do
        local workflow_name="${WORKFLOW_SCENARIOS[$workflow_key]}"
        echo "- **$workflow_name** (\`$workflow_key\`)" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Environment Information

- **Temporal Server:** http://localhost:8088
- **Temporal Namespace:** adx-core-test
- **Database:** PostgreSQL (Test)
- **Cache:** Redis
- **Rust Version:** $(rustc --version)
- **Git Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "N/A")

## Temporal UI

Access Temporal UI at: http://localhost:8088

## Log File

Full test logs: \`$LOG_FILE\`

EOF

    # Add workflow execution statistics if available
    if curl -s http://localhost:8088/api/v1/namespaces/adx-core-test/workflows > /dev/null 2>&1; then
        cat >> "$report_file" << EOF

## Workflow Execution Statistics

$(curl -s http://localhost:8088/api/v1/namespaces/adx-core-test/workflows | jq -r '.workflows | length') workflows executed during testing.

EOF
    fi
    
    log_success "Workflow test report generated: $report_file"
    
    # Display summary
    echo
    echo "=================================="
    echo "     WORKFLOW TEST SUMMARY"
    echo "=================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo "=================================="
    
    if [[ "$TEMPORAL_UI" == "true" ]]; then
        echo
        log "Temporal UI available at: http://localhost:8088"
        echo "Namespace: adx-core-test"
    fi
    
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some workflow tests failed. Check log: $LOG_FILE"
        return 1
    else
        log_success "All workflow tests passed!"
        return 0
    fi
}

cleanup_temporal_environment() {
    log "Cleaning up Temporal test environment..."
    
    cd "$ADX_CORE_DIR"
    
    # Clean test data
    if [[ "$DATABASE_URL" == *"test"* ]]; then
        cargo run --bin db-manager -- clean --database-url "$DATABASE_URL" || true
    fi
    
    # Stop Temporal services (but keep them running if requested)
    if [[ "$KEEP_TEMPORAL" != "true" ]]; then
        docker-compose -f infrastructure/docker/docker-compose.dev.yml down temporal || true
    fi
    
    log_success "Temporal cleanup completed"
}

main() {
    log "Starting ADX Core Temporal Workflow Test Suite"
    log "Log file: $LOG_FILE"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --no-ui)
                TEMPORAL_UI=false
                shift
                ;;
            --keep-temporal)
                KEEP_TEMPORAL=true
                shift
                ;;
            --timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            --workflow-timeout)
                WORKFLOW_TIMEOUT="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --verbose           Enable verbose output"
                echo "  --no-ui             Don't mention Temporal UI"
                echo "  --keep-temporal     Keep Temporal running after tests"
                echo "  --timeout SEC       Set general test timeout (default: 900)"
                echo "  --workflow-timeout SEC  Set workflow test timeout (default: 300)"
                echo "  --help              Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Trap cleanup
    trap cleanup_temporal_environment EXIT
    
    # Run test phases
    setup_temporal_environment
    
    run_workflow_unit_tests
    run_activity_tests
    run_workflow_integration_tests
    run_compensation_tests
    run_multi_tenant_workflow_tests
    run_error_handling_tests
    run_workflow_monitoring_tests
    run_workflow_security_tests
    validate_temporal_health
    
    # Performance tests (optional, time-consuming)
    if [[ "$CI" != "true" ]] && [[ "$SKIP_PERFORMANCE" != "true" ]]; then
        run_performance_workflow_tests
    else
        log "Skipping performance tests"
    fi
    
    # Generate report and exit
    if generate_workflow_report; then
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"