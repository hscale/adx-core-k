#!/bin/bash

# ADX Core Frontend Test Suite
# Comprehensive testing for TypeScript frontend microservices

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
TEST_TIMEOUT=${TEST_TIMEOUT:-300}
BROWSER=${BROWSER:-chromium}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Log file
LOG_FILE="$PROJECT_ROOT/frontend-test-results-$(date +%Y%m%d-%H%M%S).log"

# Test results
declare -A TEST_RESULTS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Micro-frontend apps
MICRO_FRONTENDS=("shell" "auth" "tenant" "file" "user" "workflow" "module")
PACKAGES=("design-system" "i18n" "shared-context" "event-bus")

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

setup_frontend_environment() {
    log "Setting up frontend test environment..."
    
    cd "$PROJECT_ROOT"
    
    # Set environment variables
    export NODE_ENV=test
    export CI=true
    export BROWSER=none
    
    # Install root dependencies
    log "Installing root dependencies..."
    if ! npm ci; then
        log_error "Failed to install root dependencies"
        return 1
    fi
    
    # Install dependencies for all micro-frontends
    for app in "${MICRO_FRONTENDS[@]}"; do
        if [[ -d "apps/$app" ]] && [[ -f "apps/$app/package.json" ]]; then
            log "Installing dependencies for $app..."
            cd "apps/$app"
            if ! npm ci; then
                log_warning "Failed to install dependencies for $app"
            fi
            cd "$PROJECT_ROOT"
        fi
    done
    
    # Install dependencies for packages
    for package in "${PACKAGES[@]}"; do
        if [[ -d "packages/$package" ]] && [[ -f "packages/$package/package.json" ]]; then
            log "Installing dependencies for $package..."
            cd "packages/$package"
            if ! npm ci; then
                log_warning "Failed to install dependencies for $package"
            fi
            cd "$PROJECT_ROOT"
        fi
    done
    
    log_success "Frontend environment setup completed"
}

run_package_tests() {
    log "=== Running Package Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Test shared packages
    for package in "${PACKAGES[@]}"; do
        if [[ -d "packages/$package" ]] && [[ -f "packages/$package/package.json" ]]; then
            # Check if package has test script
            if grep -q '"test"' "packages/$package/package.json"; then
                run_test "$package Package Tests" \
                    "cd packages/$package && npm test"
            else
                log_warning "No test script found for $package package"
            fi
        fi
    done
    
    # Test design system specifically
    if [[ -d "packages/design-system" ]]; then
        run_test "Design System Component Tests" \
            "cd packages/design-system && npm run test:components"
        
        run_test "Design System Visual Regression Tests" \
            "cd packages/design-system && npm run test:visual"
    fi
    
    # Test i18n package specifically
    if [[ -d "packages/i18n" ]]; then
        run_test "i18n Translation Tests" \
            "cd packages/i18n && npm test"
        
        run_test "i18n Format Validation" \
            "cd packages/i18n && npm run validate:translations"
    fi
}

run_microfrontend_tests() {
    log "=== Running Micro-Frontend Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Test each micro-frontend
    for app in "${MICRO_FRONTENDS[@]}"; do
        if [[ -d "apps/$app" ]] && [[ -f "apps/$app/package.json" ]]; then
            log "Testing $app micro-frontend..."
            
            # Unit tests
            if grep -q '"test"' "apps/$app/package.json"; then
                run_test "$app Unit Tests" \
                    "cd apps/$app && npm test"
            fi
            
            # Component tests
            if grep -q '"test:components"' "apps/$app/package.json"; then
                run_test "$app Component Tests" \
                    "cd apps/$app && npm run test:components"
            fi
            
            # Integration tests
            if grep -q '"test:integration"' "apps/$app/package.json"; then
                run_test "$app Integration Tests" \
                    "cd apps/$app && npm run test:integration"
            fi
            
            # Type checking
            run_test "$app Type Checking" \
                "cd apps/$app && npx tsc --noEmit"
            
            # Linting
            if grep -q '"lint"' "apps/$app/package.json"; then
                run_test "$app Linting" \
                    "cd apps/$app && npm run lint"
            fi
            
            # Build test
            run_test "$app Build Test" \
                "cd apps/$app && npm run build"
        else
            log_warning "Micro-frontend $app not found or missing package.json"
        fi
    done
}

run_cross_microfrontend_tests() {
    log "=== Running Cross Micro-Frontend Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Module Federation tests
    run_test "Module Federation Integration" \
        "npm run test:module-federation"
    
    # Event bus communication tests
    run_test "Event Bus Communication Tests" \
        "npm run test:event-bus"
    
    # Shared context tests
    run_test "Shared Context Tests" \
        "npm run test:shared-context"
    
    # Cross-app navigation tests
    run_test "Cross-App Navigation Tests" \
        "npm run test:navigation"
}

run_e2e_tests() {
    log "=== Running End-to-End Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Start development servers
    log "Starting development servers for E2E tests..."
    npm run dev:all &
    DEV_PID=$!
    
    # Wait for servers to start
    sleep 60
    
    # Basic E2E tests
    run_test "Basic E2E Tests" \
        "npx playwright test tests/e2e/basic.spec.ts --browser=$BROWSER" \
        600
    
    # Cross micro-frontend E2E tests
    run_test "Cross Micro-Frontend E2E Tests" \
        "npx playwright test tests/e2e/specs/cross-microfrontend.spec.ts --browser=$BROWSER" \
        900
    
    # User journey tests
    run_test "User Journey E2E Tests" \
        "npx playwright test tests/e2e/user-journeys.spec.ts --browser=$BROWSER" \
        900
    
    # Multi-tenant E2E tests
    run_test "Multi-Tenant E2E Tests" \
        "npx playwright test tests/e2e/multi-tenant.spec.ts --browser=$BROWSER" \
        600
    
    # Stop development servers
    kill $DEV_PID 2>/dev/null || true
    sleep 5
}

run_accessibility_tests() {
    log "=== Running Accessibility Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Start servers for accessibility testing
    npm run dev:all &
    DEV_PID=$!
    sleep 60
    
    # Accessibility tests for each micro-frontend
    for app in "${MICRO_FRONTENDS[@]}"; do
        if [[ -d "apps/$app" ]]; then
            run_test "$app Accessibility Tests" \
                "npx playwright test tests/accessibility/$app.spec.ts --browser=$BROWSER" \
                300
        fi
    done
    
    # WCAG compliance tests
    run_test "WCAG Compliance Tests" \
        "npx playwright test tests/accessibility/wcag.spec.ts --browser=$BROWSER" \
        600
    
    # Stop servers
    kill $DEV_PID 2>/dev/null || true
    sleep 5
}

run_performance_tests() {
    log "=== Running Performance Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Bundle size analysis
    run_test "Bundle Size Analysis" \
        "npm run analyze:bundles"
    
    # Performance benchmarks
    run_test "Performance Benchmarks" \
        "npm run test:performance" \
        900
    
    # Lighthouse tests
    if command -v lighthouse &> /dev/null; then
        # Start servers
        npm run dev:all &
        DEV_PID=$!
        sleep 60
        
        for app in "${MICRO_FRONTENDS[@]}"; do
            local port=$((3000 + $(printf "%s\n" "${MICRO_FRONTENDS[@]}" | grep -n "^$app$" | cut -d: -f1)))
            run_test "$app Lighthouse Performance" \
                "lighthouse http://localhost:$port --only-categories=performance --chrome-flags='--headless' --output=json --output-path=lighthouse-$app.json" \
                300
        done
        
        kill $DEV_PID 2>/dev/null || true
        sleep 5
    else
        log_warning "Lighthouse not installed, skipping performance tests"
    fi
}

run_cross_platform_tests() {
    log "=== Running Cross-Platform Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Browser compatibility tests
    local browsers=("chromium" "firefox" "webkit")
    
    # Start servers
    npm run dev:all &
    DEV_PID=$!
    sleep 60
    
    for browser in "${browsers[@]}"; do
        run_test "Cross-Platform Tests ($browser)" \
            "npx playwright test tests/cross-platform/compatibility.test.ts --browser=$browser" \
            600
    done
    
    # Mobile responsiveness tests
    run_test "Mobile Responsiveness Tests" \
        "npx playwright test tests/mobile/responsiveness.spec.ts" \
        600
    
    # Desktop app tests (if Tauri is available)
    if command -v cargo-tauri &> /dev/null; then
        run_test "Desktop App Tests" \
            "npm run test:desktop:integration" \
            900
    else
        log_warning "Tauri not available, skipping desktop tests"
    fi
    
    kill $DEV_PID 2>/dev/null || true
    sleep 5
}

run_internationalization_tests() {
    log "=== Running Internationalization Tests ==="
    
    cd "$PROJECT_ROOT"
    
    # Translation completeness tests
    run_test "Translation Completeness" \
        "npm run test:i18n:completeness"
    
    # Translation format validation
    run_test "Translation Format Validation" \
        "npm run test:i18n:format"
    
    # RTL layout tests
    run_test "RTL Layout Tests" \
        "npm run test:i18n:rtl"
    
    # Locale-specific tests
    local locales=("en" "es" "fr" "de" "ja" "zh" "ar" "he")
    
    # Start servers
    npm run dev:all &
    DEV_PID=$!
    sleep 60
    
    for locale in "${locales[@]}"; do
        run_test "Locale Tests ($locale)" \
            "LOCALE=$locale npx playwright test tests/i18n/locale.spec.ts" \
            300
    done
    
    kill $DEV_PID 2>/dev/null || true
    sleep 5
}

generate_coverage_report() {
    if [[ "$COVERAGE" != "true" ]]; then
        return 0
    fi
    
    log "=== Generating Coverage Report ==="
    
    cd "$PROJECT_ROOT"
    
    # Generate coverage for packages
    for package in "${PACKAGES[@]}"; do
        if [[ -d "packages/$package" ]] && grep -q '"test:coverage"' "packages/$package/package.json"; then
            run_test "$package Coverage Report" \
                "cd packages/$package && npm run test:coverage"
        fi
    done
    
    # Generate coverage for micro-frontends
    for app in "${MICRO_FRONTENDS[@]}"; do
        if [[ -d "apps/$app" ]] && grep -q '"test:coverage"' "apps/$app/package.json"; then
            run_test "$app Coverage Report" \
                "cd apps/$app && npm run test:coverage"
        fi
    done
    
    # Merge coverage reports
    run_test "Coverage Report Merge" \
        "npm run coverage:merge"
    
    log_success "Coverage reports generated in coverage/"
}

cleanup_frontend_environment() {
    log "Cleaning up frontend test environment..."
    
    # Kill any remaining processes
    pkill -f "vite" || true
    pkill -f "webpack" || true
    pkill -f "playwright" || true
    
    # Clean temporary files
    find "$PROJECT_ROOT" -name "*.log" -type f -mtime +7 -delete || true
    find "$PROJECT_ROOT" -name "lighthouse-*.json" -type f -delete || true
    
    log_success "Frontend cleanup completed"
}

generate_test_report() {
    log "=== Generating Frontend Test Report ==="
    
    local report_file="$PROJECT_ROOT/frontend-test-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX Core Frontend Test Report

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

- **Node.js Version:** $(node --version)
- **npm Version:** $(npm --version)
- **Browser:** $BROWSER
- **Platform:** $(uname -s) $(uname -m)
- **Git Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "N/A")

## Micro-Frontends Tested

EOF

    for app in "${MICRO_FRONTENDS[@]}"; do
        if [[ -d "apps/$app" ]]; then
            echo "- ✅ $app" >> "$report_file"
        else
            echo "- ❌ $app (not found)" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

## Packages Tested

EOF

    for package in "${PACKAGES[@]}"; do
        if [[ -d "packages/$package" ]]; then
            echo "- ✅ $package" >> "$report_file"
        else
            echo "- ❌ $package (not found)" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

## Log File

Full test logs: \`$LOG_FILE\`

EOF

    log_success "Frontend test report generated: $report_file"
    
    # Display summary
    echo
    echo "=================================="
    echo "     FRONTEND TEST SUMMARY"
    echo "=================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo "=================================="
    
    if [[ $FAILED_TESTS -gt 0 ]]; then
        log_error "Some frontend tests failed. Check log: $LOG_FILE"
        return 1
    else
        log_success "All frontend tests passed!"
        return 0
    fi
}

main() {
    log "Starting ADX Core Frontend Test Suite"
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
            --browser)
                BROWSER="$2"
                shift 2
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
                echo "  --browser NAME Set browser for E2E tests (chromium/firefox/webkit)"
                echo "  --no-parallel  Disable parallel testing"
                echo "  --timeout SEC  Set test timeout (default: 300)"
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
    trap cleanup_frontend_environment EXIT
    
    # Run test phases
    setup_frontend_environment
    
    run_package_tests
    run_microfrontend_tests
    run_cross_microfrontend_tests
    run_internationalization_tests
    
    # E2E and performance tests (if not in CI or explicitly requested)
    if [[ "$CI" != "true" ]] || [[ "$FORCE_E2E" == "true" ]]; then
        run_e2e_tests
        run_accessibility_tests
        run_performance_tests
        run_cross_platform_tests
    else
        log "Skipping E2E and performance tests in CI environment"
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