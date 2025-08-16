#!/bin/bash

# ADX Core Test Summary Script
# Provides a quick overview of system status and test results

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADX_CORE_DIR="$PROJECT_ROOT/adx-core"

# Logging functions
log_header() {
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${BLUE}$(printf '=%.0s' {1..60})${NC}"
}

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Function to check status
check_status() {
    local description="$1"
    local command="$2"
    local is_critical="${3:-true}"
    
    if eval "$command" >/dev/null 2>&1; then
        log_success "$description"
        return 0
    else
        if [[ "$is_critical" == "true" ]]; then
            log_error "$description"
        else
            log_warning "$description"
        fi
        return 1
    fi
}

# System overview
show_system_overview() {
    log_header "ADX CORE SYSTEM OVERVIEW"
    
    echo -e "${BOLD}Project Information:${NC}"
    echo "  Location: $PROJECT_ROOT"
    echo "  ADX Core: $ADX_CORE_DIR"
    echo "  Timestamp: $(date)"
    echo
    
    echo -e "${BOLD}System Information:${NC}"
    echo "  OS: $(uname -s) $(uname -m)"
    echo "  Rust: $(rustc --version 2>/dev/null || echo 'Not installed')"
    echo "  Node: $(node --version 2>/dev/null || echo 'Not installed')"
    echo "  Docker: $(docker --version 2>/dev/null | cut -d' ' -f3 | tr -d ',' || echo 'Not installed')"
    echo
}

# Quick health check
show_health_status() {
    log_header "HEALTH STATUS"
    
    # Core tools
    check_status "Rust toolchain available" "rustc --version"
    check_status "Cargo available" "cargo --version"
    check_status "Node.js available" "node --version"
    check_status "npm available" "npm --version"
    check_status "Docker available" "docker --version"
    
    # Project structure
    check_status "ADX Core directory exists" "test -d '$ADX_CORE_DIR'"
    check_status "Cargo workspace configured" "test -f '$ADX_CORE_DIR/Cargo.toml'"
    check_status "Package.json exists" "test -f '$PROJECT_ROOT/package.json'"
    
    echo
}

# Compilation status
show_compilation_status() {
    log_header "COMPILATION STATUS"
    
    cd "$ADX_CORE_DIR"
    
    log_info "Checking Rust workspace compilation..."
    if cargo check --workspace --quiet >/dev/null 2>&1; then
        log_success "Rust workspace compiles successfully"
    else
        log_error "Rust workspace compilation failed"
        echo "  Run 'cargo check --workspace' for details"
    fi
    
    # Check individual services
    local services=("shared" "module-service")
    for service in "${services[@]}"; do
        if [[ -d "services/$service" ]]; then
            cd "services/$service"
            if cargo check --quiet >/dev/null 2>&1; then
                log_success "Service '$service' compiles"
            else
                log_error "Service '$service' compilation failed"
            fi
            cd "$ADX_CORE_DIR"
        fi
    done
    
    echo
}

# Test status
show_test_status() {
    log_header "TEST STATUS"
    
    cd "$ADX_CORE_DIR"
    
    log_info "Running unit tests..."
    if cargo test --workspace --lib --quiet >/dev/null 2>&1; then
        # Get test results
        local test_output=$(cargo test --workspace --lib --quiet 2>&1)
        local passed=$(echo "$test_output" | grep -o '[0-9]* passed' | cut -d' ' -f1)
        local failed=$(echo "$test_output" | grep -o '[0-9]* failed' | cut -d' ' -f1)
        local ignored=$(echo "$test_output" | grep -o '[0-9]* ignored' | cut -d' ' -f1)
        
        log_success "Unit tests: ${passed:-0} passed, ${failed:-0} failed, ${ignored:-0} ignored"
    else
        log_error "Unit tests failed"
        echo "  Run 'cargo test --workspace --lib' for details"
    fi
    
    echo
}

# Available scripts
show_available_scripts() {
    log_header "AVAILABLE TEST SCRIPTS"
    
    local scripts=(
        "quick-test.sh:Quick validation without Docker"
        "validate-setup.sh:Complete environment validation"
        "test-all.sh:Comprehensive test suite"
        "test-backend.sh:Backend-specific tests"
        "test-frontend.sh:Frontend-specific tests"
        "test-workflows.sh:Temporal workflow tests"
        "debug-services.sh:Service debugging tools"
    )
    
    for script_info in "${scripts[@]}"; do
        local script_name=$(echo "$script_info" | cut -d':' -f1)
        local script_desc=$(echo "$script_info" | cut -d':' -f2)
        
        if [[ -f "$SCRIPT_DIR/$script_name" ]]; then
            log_success "$script_name - $script_desc"
        else
            log_warning "$script_name - Not found"
        fi
    done
    
    echo
}

# Recommendations
show_recommendations() {
    log_header "RECOMMENDATIONS"
    
    # Check for common issues and provide recommendations
    cd "$ADX_CORE_DIR"
    
    # Check for warnings
    local warnings=$(cargo check --workspace 2>&1 | grep -c "warning:" || echo "0")
    if [[ $warnings -gt 0 ]]; then
        log_warning "$warnings compiler warnings found"
        echo "  Consider running: cargo clippy --workspace --fix"
    else
        log_success "No compiler warnings"
    fi
    
    # Check Docker status
    if docker ps >/dev/null 2>&1; then
        log_success "Docker daemon is running"
    else
        log_warning "Docker daemon not running"
        echo "  Start Docker to run full integration tests"
    fi
    
    # Check for node_modules
    if [[ -d "$PROJECT_ROOT/node_modules" ]]; then
        log_success "Frontend dependencies installed"
    else
        log_warning "Frontend dependencies not installed"
        echo "  Run: npm ci"
    fi
    
    echo
    echo -e "${BOLD}Next Steps:${NC}"
    echo "  1. Fix any issues shown above"
    echo "  2. Run: ./scripts/validate-setup.sh (full validation)"
    echo "  3. Run: ./scripts/test-all.sh (comprehensive tests)"
    echo "  4. For debugging: ./scripts/debug-services.sh"
    echo
}

# Main execution
main() {
    clear
    echo -e "${BOLD}${GREEN}🚀 ADX CORE TEST SUMMARY${NC}"
    echo
    
    show_system_overview
    show_health_status
    show_compilation_status
    show_test_status
    show_available_scripts
    show_recommendations
    
    echo -e "${BOLD}${GREEN}Summary complete!${NC}"
    echo "For detailed testing, run: ${CYAN}./scripts/test-all.sh${NC}"
}

# Run main function
main "$@"