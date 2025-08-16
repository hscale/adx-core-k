#!/usr/bin/env bash

# ADX Core Environment Validation Script
# Validates development environment setup and configuration

set -e

# Ensure we're running in bash with associative array support
if [[ -z "$BASH_VERSION" ]] || [[ ${BASH_VERSION%%.*} -lt 4 ]]; then
    echo "Error: This script requires Bash 4.0 or later"
    echo "Current shell: $0"
    echo "Please run with: bash $0"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
VERBOSE=${VERBOSE:-false}
FIX_ISSUES=${FIX_ISSUES:-false}
SKIP_OPTIONAL=${SKIP_OPTIONAL:-false}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ADX_CORE_DIR="$PROJECT_ROOT/adx-core"

# Validation results
declare -A VALIDATION_RESULTS
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

# Required versions
REQUIRED_NODE_VERSION="18.0.0"
REQUIRED_RUST_VERSION="1.70.0"
REQUIRED_DOCKER_VERSION="20.0.0"

# Utility functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✅ PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[❌ FAIL]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠️  WARN]${NC} $1"
}

log_info() {
    echo -e "${CYAN}[ℹ️  INFO]${NC} $1"
}

run_check() {
    local check_name="$1"
    local check_function="$2"
    local is_optional="${3:-false}"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    log "Checking $check_name..."
    
    if $check_function; then
        log_success "$check_name"
        VALIDATION_RESULTS["$check_name"]="PASS"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        if [[ "$is_optional" == "true" ]]; then
            log_warning "$check_name (optional)"
            VALIDATION_RESULTS["$check_name"]="WARN"
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            return 0
        else
            log_error "$check_name"
            VALIDATION_RESULTS["$check_name"]="FAIL"
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
            return 1
        fi
    fi
}

version_compare() {
    local version1="$1"
    local version2="$2"
    
    # Remove 'v' prefix if present
    version1=${version1#v}
    version2=${version2#v}
    
    printf '%s\n%s\n' "$version1" "$version2" | sort -V -C
}

# Validation functions
check_project_structure() {
    [[ -f "$PROJECT_ROOT/package.json" ]] && \
    [[ -d "$ADX_CORE_DIR" ]] && \
    [[ -f "$ADX_CORE_DIR/Cargo.toml" ]] && \
    [[ -d "$PROJECT_ROOT/apps" ]] && \
    [[ -d "$PROJECT_ROOT/packages" ]] && \
    [[ -d "$PROJECT_ROOT/scripts" ]]
}

check_node_version() {
    if ! command -v node &> /dev/null; then
        return 1
    fi
    
    local node_version=$(node --version | sed 's/v//')
    version_compare "$REQUIRED_NODE_VERSION" "$node_version"
}

check_npm_version() {
    if ! command -v npm &> /dev/null; then
        return 1
    fi
    
    # npm should be available if node is installed
    npm --version > /dev/null 2>&1
}

check_rust_version() {
    if ! command -v rustc &> /dev/null; then
        return 1
    fi
    
    local rust_version=$(rustc --version | awk '{print $2}')
    version_compare "$REQUIRED_RUST_VERSION" "$rust_version"
}

check_cargo_version() {
    if ! command -v cargo &> /dev/null; then
        return 1
    fi
    
    cargo --version > /dev/null 2>&1
}

check_docker_version() {
    if ! command -v docker &> /dev/null; then
        return 1
    fi
    
    local docker_version=$(docker --version | awk '{print $3}' | sed 's/,//')
    version_compare "$REQUIRED_DOCKER_VERSION" "$docker_version"
}

check_docker_compose() {
    if command -v docker-compose &> /dev/null; then
        docker-compose --version > /dev/null 2>&1
        return $?
    elif docker-compose version &> /dev/null; then
        return 0
    else
        return 1
    fi
}

check_git_version() {
    if ! command -v git &> /dev/null; then
        return 1
    fi
    
    git --version > /dev/null 2>&1
}

check_root_dependencies() {
    [[ -f "$PROJECT_ROOT/package.json" ]] && \
    [[ -d "$PROJECT_ROOT/node_modules" ]] && \
    [[ -f "$PROJECT_ROOT/package-lock.json" ]]
}

check_rust_workspace() {
    cd "$ADX_CORE_DIR"
    
    # Check if workspace compiles
    cargo check --workspace > /dev/null 2>&1
}

check_frontend_apps() {
    local apps=("shell" "auth" "tenant" "file" "user" "workflow")
    
    for app in "${apps[@]}"; do
        if [[ -d "$PROJECT_ROOT/apps/$app" ]]; then
            if [[ ! -f "$PROJECT_ROOT/apps/$app/package.json" ]]; then
                return 1
            fi
        fi
    done
    
    return 0
}

check_shared_packages() {
    local packages=("design-system" "i18n" "shared-context" "event-bus")
    
    for package in "${packages[@]}"; do
        if [[ -d "$PROJECT_ROOT/packages/$package" ]]; then
            if [[ ! -f "$PROJECT_ROOT/packages/$package/package.json" ]]; then
                return 1
            fi
        fi
    done
    
    return 0
}

check_environment_files() {
    [[ -f "$ADX_CORE_DIR/.env" ]] || [[ -f "$ADX_CORE_DIR/.env.example" ]]
}

check_database_config() {
    local db_url="${DATABASE_URL:-}"
    
    if [[ -z "$db_url" ]]; then
        # Check if .env file has DATABASE_URL
        if [[ -f "$ADX_CORE_DIR/.env" ]]; then
            grep -q "DATABASE_URL" "$ADX_CORE_DIR/.env"
        else
            return 1
        fi
    else
        # Validate URL format
        [[ "$db_url" =~ ^postgres:// ]]
    fi
}

check_redis_config() {
    local redis_url="${REDIS_URL:-}"
    
    if [[ -z "$redis_url" ]]; then
        # Check if .env file has REDIS_URL
        if [[ -f "$ADX_CORE_DIR/.env" ]]; then
            grep -q "REDIS_URL" "$ADX_CORE_DIR/.env"
        else
            return 1
        fi
    else
        # Validate URL format
        [[ "$redis_url" =~ ^redis:// ]]
    fi
}

check_temporal_config() {
    local temporal_url="${TEMPORAL_SERVER_URL:-}"
    
    if [[ -z "$temporal_url" ]]; then
        # Check if .env file has TEMPORAL_SERVER_URL
        if [[ -f "$ADX_CORE_DIR/.env" ]]; then
            grep -q "TEMPORAL_SERVER_URL" "$ADX_CORE_DIR/.env"
        else
            return 1
        fi
    else
        # Validate URL format (should be host:port)
        [[ "$temporal_url" =~ ^[a-zA-Z0-9.-]+:[0-9]+$ ]] || [[ "$temporal_url" =~ ^localhost:[0-9]+$ ]]
    fi
}

check_docker_infrastructure() {
    [[ -f "$ADX_CORE_DIR/infrastructure/docker/docker-compose.dev.yml" ]] && \
    [[ -f "$ADX_CORE_DIR/infrastructure/docker/docker-compose.prod.yml" ]]
}

check_database_migrations() {
    cd "$ADX_CORE_DIR"
    
    # Check if migrations directory exists
    [[ -d "services/shared/migrations" ]] || [[ -d "migrations" ]]
}

check_test_configuration() {
    [[ -f "$PROJECT_ROOT/vitest.config.ts" ]] && \
    [[ -f "$PROJECT_ROOT/playwright.config.desktop.ts" ]] && \
    [[ -f "$PROJECT_ROOT/playwright.config.performance.ts" ]]
}

check_linting_configuration() {
    [[ -f "$PROJECT_ROOT/.eslintrc.json" ]] && \
    [[ -f "$PROJECT_ROOT/tsconfig.json" ]]
}

# Optional checks
check_tauri_cli() {
    command -v cargo-tauri &> /dev/null
}

check_playwright() {
    command -v playwright &> /dev/null || \
    npx playwright --version &> /dev/null
}

check_rust_tools() {
    command -v cargo-clippy &> /dev/null && \
    command -v rustfmt &> /dev/null
}

check_additional_tools() {
    command -v jq &> /dev/null && \
    command -v curl &> /dev/null && \
    command -v nc &> /dev/null
}

# Service connectivity checks
check_postgres_connectivity() {
    local db_url="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/adx_core_dev}"
    
    # Try to connect using psql if available
    if command -v psql &> /dev/null; then
        echo "SELECT 1;" | psql "$db_url" > /dev/null 2>&1
    else
        # Check if port is open
        nc -z localhost 5432 2>/dev/null
    fi
}

check_redis_connectivity() {
    # Check if Redis port is open
    nc -z localhost 6379 2>/dev/null
}

check_temporal_connectivity() {
    # Check if Temporal server port is open
    nc -z localhost 7233 2>/dev/null
}

# Fix functions
fix_root_dependencies() {
    if [[ "$FIX_ISSUES" == "true" ]]; then
        log_info "Installing root dependencies..."
        cd "$PROJECT_ROOT"
        npm ci
    fi
}

fix_environment_files() {
    if [[ "$FIX_ISSUES" == "true" ]]; then
        log_info "Creating .env file from example..."
        if [[ -f "$ADX_CORE_DIR/.env.example" ]] && [[ ! -f "$ADX_CORE_DIR/.env" ]]; then
            cp "$ADX_CORE_DIR/.env.example" "$ADX_CORE_DIR/.env"
        fi
    fi
}

generate_validation_report() {
    log "=== Generating Validation Report ==="
    
    local report_file="$PROJECT_ROOT/validation-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# ADX Core Environment Validation Report

**Generated:** $(date)
**Total Checks:** $TOTAL_CHECKS
**Passed:** $PASSED_CHECKS
**Failed:** $FAILED_CHECKS
**Warnings:** $WARNING_CHECKS

## Validation Results

EOF

    for check_name in "${!VALIDATION_RESULTS[@]}"; do
        local status="${VALIDATION_RESULTS[$check_name]}"
        local icon="❌"
        case "$status" in
            "PASS") icon="✅" ;;
            "WARN") icon="⚠️" ;;
            "FAIL") icon="❌" ;;
        esac
        echo "- $icon **$check_name**: $status" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## System Information

- **OS:** $(uname -s) $(uname -r)
- **Architecture:** $(uname -m)
- **Node.js:** $(node --version 2>/dev/null || echo "Not installed")
- **npm:** $(npm --version 2>/dev/null || echo "Not installed")
- **Rust:** $(rustc --version 2>/dev/null || echo "Not installed")
- **Cargo:** $(cargo --version 2>/dev/null || echo "Not installed")
- **Docker:** $(docker --version 2>/dev/null || echo "Not installed")
- **Git:** $(git --version 2>/dev/null || echo "Not installed")

## Environment Variables

- **DATABASE_URL:** ${DATABASE_URL:-"Not set"}
- **REDIS_URL:** ${REDIS_URL:-"Not set"}
- **TEMPORAL_SERVER_URL:** ${TEMPORAL_SERVER_URL:-"Not set"}
- **NODE_ENV:** ${NODE_ENV:-"Not set"}
- **RUST_LOG:** ${RUST_LOG:-"Not set"}

## Recommendations

EOF

    # Add recommendations based on failed checks
    if [[ $FAILED_CHECKS -gt 0 ]]; then
        echo "### Critical Issues to Fix" >> "$report_file"
        echo "" >> "$report_file"
        
        for check_name in "${!VALIDATION_RESULTS[@]}"; do
            if [[ "${VALIDATION_RESULTS[$check_name]}" == "FAIL" ]]; then
                case "$check_name" in
                    "Node.js Version")
                        echo "- 🔧 **Install Node.js $REQUIRED_NODE_VERSION or later**: Visit https://nodejs.org/" >> "$report_file"
                        ;;
                    "Rust Version")
                        echo "- 🔧 **Install Rust $REQUIRED_RUST_VERSION or later**: Run \`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\`" >> "$report_file"
                        ;;
                    "Docker Version")
                        echo "- 🔧 **Install Docker $REQUIRED_DOCKER_VERSION or later**: Visit https://docs.docker.com/get-docker/" >> "$report_file"
                        ;;
                    "Root Dependencies")
                        echo "- 🔧 **Install dependencies**: Run \`npm ci\` in project root" >> "$report_file"
                        ;;
                    "Environment Files")
                        echo "- 🔧 **Create .env file**: Copy \`.env.example\` to \`.env\` in adx-core directory" >> "$report_file"
                        ;;
                    *)
                        echo "- 🔧 **Fix $check_name**: Check the specific requirements for this component" >> "$report_file"
                        ;;
                esac
            fi
        done
    else
        echo "✅ **All critical checks passed!** Your environment is properly configured." >> "$report_file"
    fi
    
    if [[ $WARNING_CHECKS -gt 0 ]]; then
        echo "" >> "$report_file"
        echo "### Optional Improvements" >> "$report_file"
        echo "" >> "$report_file"
        
        for check_name in "${!VALIDATION_RESULTS[@]}"; do
            if [[ "${VALIDATION_RESULTS[$check_name]}" == "WARN" ]]; then
                case "$check_name" in
                    "Tauri CLI")
                        echo "- 💡 **Install Tauri CLI**: Run \`cargo install tauri-cli\` for desktop app development" >> "$report_file"
                        ;;
                    "Playwright")
                        echo "- 💡 **Install Playwright**: Run \`npx playwright install\` for E2E testing" >> "$report_file"
                        ;;
                    "Rust Tools")
                        echo "- 💡 **Install Rust tools**: Run \`rustup component add clippy rustfmt\`" >> "$report_file"
                        ;;
                    *)
                        echo "- 💡 **Consider installing $check_name** for enhanced development experience" >> "$report_file"
                        ;;
                esac
            fi
        done
    fi
    
    cat >> "$report_file" << EOF

## Next Steps

1. **Fix Critical Issues**: Address all failed checks above
2. **Start Infrastructure**: Run \`./scripts/dev-start-all.sh\`
3. **Run Tests**: Execute \`./scripts/test-all.sh\`
4. **Start Development**: Begin coding with \`npm run dev:all\`

## Quick Setup Commands

\`\`\`bash
# Install dependencies
npm ci

# Start infrastructure
cd adx-core && docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d

# Run database migrations
cd adx-core && cargo run --bin db-manager -- migrate

# Start all services
./scripts/dev-start-all.sh
\`\`\`

EOF

    log_success "Validation report generated: $report_file"
    
    return $report_file
}

main() {
    log "Starting ADX Core Environment Validation"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --fix)
                FIX_ISSUES=true
                shift
                ;;
            --skip-optional)
                SKIP_OPTIONAL=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --verbose       Enable verbose output"
                echo "  --fix           Attempt to fix issues automatically"
                echo "  --skip-optional Skip optional checks"
                echo "  --help          Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    echo
    echo "🔍 ADX Core Environment Validation"
    echo "=================================="
    echo
    
    # Core requirements
    run_check "Project Structure" check_project_structure
    run_check "Node.js Version" check_node_version
    run_check "npm Version" check_npm_version
    run_check "Rust Version" check_rust_version
    run_check "Cargo Version" check_cargo_version
    run_check "Docker Version" check_docker_version
    run_check "docker-compose" check_docker_compose
    run_check "Git Version" check_git_version
    
    # Project configuration
    run_check "Root Dependencies" check_root_dependencies
    run_check "Rust Workspace" check_rust_workspace
    run_check "Frontend Apps" check_frontend_apps
    run_check "Shared Packages" check_shared_packages
    run_check "Environment Files" check_environment_files
    run_check "Database Config" check_database_config
    run_check "Redis Config" check_redis_config
    run_check "Temporal Config" check_temporal_config
    run_check "Docker Infrastructure" check_docker_infrastructure
    run_check "Database Migrations" check_database_migrations
    run_check "Test Configuration" check_test_configuration
    run_check "Linting Configuration" check_linting_configuration
    
    # Service connectivity (optional)
    run_check "PostgreSQL Connectivity" check_postgres_connectivity true
    run_check "Redis Connectivity" check_redis_connectivity true
    run_check "Temporal Connectivity" check_temporal_connectivity true
    
    # Optional tools
    if [[ "$SKIP_OPTIONAL" != "true" ]]; then
        run_check "Tauri CLI" check_tauri_cli true
        run_check "Playwright" check_playwright true
        run_check "Rust Tools" check_rust_tools true
        run_check "Additional Tools" check_additional_tools true
    fi
    
    # Generate report
    generate_validation_report
    
    # Display summary
    echo
    echo "=================================="
    echo "      VALIDATION SUMMARY"
    echo "=================================="
    echo "Total Checks: $TOTAL_CHECKS"
    echo "Passed: $PASSED_CHECKS"
    echo "Failed: $FAILED_CHECKS"
    echo "Warnings: $WARNING_CHECKS"
    echo "=================================="
    
    if [[ $FAILED_CHECKS -gt 0 ]]; then
        echo
        log_error "Environment validation failed. Check the report for details: $report_file"
        
        if [[ "$FIX_ISSUES" == "true" ]]; then
            log_info "Attempting to fix issues..."
            # Add fix attempts here
        else
            log_info "Run with --fix to attempt automatic fixes"
        fi
        
        exit 1
    else
        echo
        log_success "Environment validation passed! Your setup is ready for development."
        
        if [[ $WARNING_CHECKS -gt 0 ]]; then
            log_info "Consider addressing $WARNING_CHECKS optional warnings for enhanced development experience"
        fi
        
        echo
        log_info "Next steps:"
        echo "  1. Start infrastructure: ./scripts/dev-start-all.sh"
        echo "  2. Run tests: ./scripts/test-all.sh"
        echo "  3. Start development: npm run dev:all"
        
        exit 0
    fi
}

# Run main function
main "$@"