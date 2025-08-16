#!/bin/bash

# Cross-Platform Testing Script
# This script runs comprehensive cross-platform tests

set -e

# Configuration
TEST_TYPE=${1:-all}  # all, desktop, mobile, compatibility
PLATFORMS=${2:-all}  # all, windows, macos, linux, ios, android
HEADLESS=${3:-true}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Test functions
run_desktop_tests() {
    log_info "Running desktop tests..."
    
    # Build desktop app first
    if command -v cargo >/dev/null 2>&1; then
        log_info "Building Tauri desktop app..."
        cargo tauri build --debug
    else
        log_warning "Cargo not found, skipping Tauri build"
    fi
    
    # Run desktop integration tests
    log_info "Running desktop integration tests..."
    npm run test:desktop:integration
    
    # Run desktop E2E tests
    if [[ "$HEADLESS" == "true" ]]; then
        log_info "Running desktop E2E tests (headless)..."
        HEADLESS=true npm run test:desktop:e2e
    else
        log_info "Running desktop E2E tests (headed)..."
        npm run test:desktop:e2e
    fi
}

run_mobile_tests() {
    log_info "Running mobile tests..."
    
    # Check for mobile testing dependencies
    if [[ "$PLATFORMS" == "all" ]] || [[ "$PLATFORMS" == *"ios"* ]]; then
        if command -v xcrun >/dev/null 2>&1; then
            log_info "Running iOS tests..."
            npm run test:mobile:ios
        else
            log_warning "Xcode not found, skipping iOS tests"
        fi
    fi
    
    if [[ "$PLATFORMS" == "all" ]] || [[ "$PLATFORMS" == *"android"* ]]; then
        if command -v adb >/dev/null 2>&1; then
            log_info "Running Android tests..."
            npm run test:mobile:android
        else
            log_warning "Android SDK not found, skipping Android tests"
        fi
    fi
}

run_compatibility_tests() {
    log_info "Running cross-platform compatibility tests..."
    
    # Run compatibility test suite
    npm run test:cross-platform:compatibility
    
    # Generate compatibility report
    npm run generate:compatibility-report
}

run_performance_tests() {
    log_info "Running cross-platform performance tests..."
    
    # Run performance tests on different platforms
    npm run test:performance:cross-platform
}

# Main execution
main() {
    log_info "Starting cross-platform tests..."
    log_info "Test type: $TEST_TYPE"
    log_info "Platforms: $PLATFORMS"
    log_info "Headless: $HEADLESS"
    
    # Install dependencies
    log_info "Installing dependencies..."
    npm ci
    
    # Install Playwright browsers
    log_info "Installing Playwright browsers..."
    npx playwright install
    
    case "$TEST_TYPE" in
        "all")
            run_desktop_tests
            run_mobile_tests
            run_compatibility_tests
            run_performance_tests
            ;;
        "desktop")
            run_desktop_tests
            ;;
        "mobile")
            run_mobile_tests
            ;;
        "compatibility")
            run_compatibility_tests
            ;;
        "performance")
            run_performance_tests
            ;;
        *)
            log_error "Unknown test type: $TEST_TYPE"
            exit 1
            ;;
    esac
    
    log_success "Cross-platform tests completed!"
}

# Check dependencies
command -v npm >/dev/null 2>&1 || { log_error "npm is required but not installed. Aborting."; exit 1; }
command -v node >/dev/null 2>&1 || { log_error "node is required but not installed. Aborting."; exit 1; }

# Run main function
main "$@"