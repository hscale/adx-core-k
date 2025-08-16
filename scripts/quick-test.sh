#!/bin/bash

# ADX Core Quick Test Script
# Runs essential tests without requiring full Docker infrastructure

set -e

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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test compilation
test_compilation() {
    log_info "Testing Rust compilation..."
    cd adx-core
    if cargo check --workspace; then
        log_success "Rust compilation successful"
    else
        log_error "Rust compilation failed"
        return 1
    fi
    cd ..
}

# Test basic functionality
test_basic() {
    log_info "Running basic tests..."
    
    # Test tools
    if rustc --version >/dev/null 2>&1; then
        log_success "Rust toolchain available"
    else
        log_error "Rust toolchain missing"
        return 1
    fi
    
    if node --version >/dev/null 2>&1; then
        log_success "Node.js available"
    else
        log_error "Node.js missing"
        return 1
    fi
    
    return 0
}

# Main execution
main() {
    echo "🚀 ADX Core Quick Test"
    
    if test_basic && test_compilation; then
        log_success "🎉 Quick tests passed!"
        echo "Next steps:"
        echo "  1. Run full validation: ./scripts/validate-setup.sh"
        echo "  2. Run comprehensive tests: ./scripts/test-all.sh"
        exit 0
    else
        log_error "❌ Quick tests failed"
        exit 1
    fi
}

main "$@"