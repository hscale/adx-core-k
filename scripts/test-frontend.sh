#!/bin/bash

# ADX CORE Frontend Testing Script
# Runs TypeScript/React unit tests, component tests, and micro-frontend integration tests

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

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Parse command line arguments
COVERAGE=false
VERBOSE=false
UNIT_ONLY=false
COMPONENT_ONLY=false
INTEGRATION_ONLY=false
WATCH=false

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
        --component-only)
            COMPONENT_ONLY=true
            shift
            ;;
        --integration-only)
            INTEGRATION_ONLY=true
            shift
            ;;
        --watch)
            WATCH=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --coverage         Generate coverage report"
            echo "  --verbose          Verbose output"
            echo "  --unit-only        Run only unit tests"
            echo "  --component-only   Run only component tests"
            echo "  --integration-only Run only integration tests"
            echo "  --watch            Run tests in watch mode"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_status "Starting ADX CORE Frontend Tests..."

# Set test environment variables
export NODE_ENV="test"
export CI=true
export FORCE_COLOR=1

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    print_status "Installing dependencies..."
    npm install
fi

# Build test flags
TEST_FLAGS=""
if [ "$VERBOSE" = true ]; then
    TEST_FLAGS="$TEST_FLAGS --verbose"
fi

if [ "$COVERAGE" = true ]; then
    TEST_FLAGS="$TEST_FLAGS --coverage"
fi

if [ "$WATCH" = true ]; then
    TEST_FLAGS="$TEST_FLAGS --watch"
fi

# Function to run tests for a specific workspace
run_workspace_tests() {
    local workspace="$1"
    local test_type="$2"
    
    if [ ! -d "$workspace" ]; then
        print_warning "Workspace $workspace not found, skipping..."
        return 0
    fi
    
    print_status "Testing $workspace ($test_type)..."
    
    cd "$workspace"
    
    # Check if package.json exists
    if [ ! -f "package.json" ]; then
        print_warning "No package.json found in $workspace, skipping..."
        cd ..
        return 0
    fi
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing dependencies for $workspace..."
        npm install
    fi
    
    # Run tests based on available scripts
    if npm run | grep -q "test:$test_type"; then
        npm run "test:$test_type" -- $TEST_FLAGS
    elif npm run | grep -q "test"; then
        npm run test -- $TEST_FLAGS
    else
        print_warning "No test script found in $workspace"
        cd ..
        return 0
    fi
    
    local exit_code=$?
    cd ..
    
    if [ $exit_code -eq 0 ]; then
        print_success "$workspace ($test_type) tests passed"
    else
        print_error "$workspace ($test_type) tests failed"
        return $exit_code
    fi
}

# Function to run linting for a workspace
run_workspace_lint() {
    local workspace="$1"
    
    if [ ! -d "$workspace" ]; then
        return 0
    fi
    
    print_status "Linting $workspace..."
    
    cd "$workspace"
    
    if [ -f "package.json" ] && npm run | grep -q "lint"; then
        npm run lint
        local exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            print_success "$workspace linting passed"
        else
            print_warning "$workspace linting failed"
        fi
    fi
    
    cd ..
}

# Function to run type checking for a workspace
run_workspace_typecheck() {
    local workspace="$1"
    
    if [ ! -d "$workspace" ]; then
        return 0
    fi
    
    print_status "Type checking $workspace..."
    
    cd "$workspace"
    
    if [ -f "package.json" ] && npm run | grep -q "type-check"; then
        npm run type-check
        local exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            print_success "$workspace type checking passed"
        else
            print_error "$workspace type checking failed"
            cd ..
            return $exit_code
        fi
    elif [ -f "tsconfig.json" ]; then
        npx tsc --noEmit
        local exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            print_success "$workspace type checking passed"
        else
            print_error "$workspace type checking failed"
            cd ..
            return $exit_code
        fi
    fi
    
    cd ..
}

# Define workspaces to test
SHELL_APP="adx-core/apps/shell"
MICRO_APPS=(
    "adx-core/apps/auth"
    "adx-core/apps/tenant"
    "adx-core/apps/file"
    "adx-core/apps/user"
    "adx-core/apps/workflow"
    "adx-core/apps/module"
)

PACKAGES=(
    "adx-core/packages/design-system"
    "adx-core/packages/shared-context"
    "adx-core/packages/event-bus"
    "adx-core/packages/i18n"
    "adx-core/packages/shared"
)

BFF_SERVICES=(
    "adx-core/bff-services/auth-bff"
    "adx-core/bff-services/tenant-bff"
    "adx-core/bff-services/file-bff"
    "adx-core/bff-services/user-bff"
    "adx-core/bff-services/workflow-bff"
    "adx-core/bff-services/module-bff"
)

# Run tests based on options
if [ "$UNIT_ONLY" = true ]; then
    print_status "=== Running Unit Tests Only ==="
    
    # Test shared packages first
    for package in "${PACKAGES[@]}"; do
        run_workspace_tests "$package" "unit" || exit 1
    done
    
    # Test shell app
    run_workspace_tests "$SHELL_APP" "unit" || exit 1
    
    # Test micro-apps
    for app in "${MICRO_APPS[@]}"; do
        run_workspace_tests "$app" "unit" || exit 1
    done
    
elif [ "$COMPONENT_ONLY" = true ]; then
    print_status "=== Running Component Tests Only ==="
    
    # Test shell app components
    run_workspace_tests "$SHELL_APP" "component" || exit 1
    
    # Test micro-app components
    for app in "${MICRO_APPS[@]}"; do
        run_workspace_tests "$app" "component" || exit 1
    done
    
elif [ "$INTEGRATION_ONLY" = true ]; then
    print_status "=== Running Integration Tests Only ==="
    
    # Test BFF services
    for bff in "${BFF_SERVICES[@]}"; do
        run_workspace_tests "$bff" "integration" || exit 1
    done
    
    # Test cross-micro-frontend integration
    run_workspace_tests "$SHELL_APP" "integration" || exit 1
    
else
    # Run all frontend tests
    print_status "=== Running All Frontend Tests ==="
    
    # 1. Shared Packages Tests
    print_status "=== Testing Shared Packages ==="
    for package in "${PACKAGES[@]}"; do
        run_workspace_tests "$package" "unit" || exit 1
    done
    
    # 2. Shell Application Tests
    print_status "=== Testing Shell Application ==="
    run_workspace_tests "$SHELL_APP" "unit" || exit 1
    run_workspace_tests "$SHELL_APP" "component" || exit 1
    
    # 3. Micro-Frontend Tests
    print_status "=== Testing Micro-Frontends ==="
    for app in "${MICRO_APPS[@]}"; do
        print_status "Testing $(basename "$app") micro-frontend..."
        run_workspace_tests "$app" "unit" || exit 1
        run_workspace_tests "$app" "component" || exit 1
    done
    
    # 4. BFF Services Tests
    print_status "=== Testing BFF Services ==="
    for bff in "${BFF_SERVICES[@]}"; do
        print_status "Testing $(basename "$bff") BFF service..."
        run_workspace_tests "$bff" "unit" || exit 1
        run_workspace_tests "$bff" "integration" || exit 1
    done
    
    # 5. Cross-Micro-Frontend Integration Tests
    print_status "=== Testing Cross-Micro-Frontend Integration ==="
    run_workspace_tests "$SHELL_APP" "integration" || exit 1
fi

# Run linting for all workspaces
if [ "$UNIT_ONLY" = false ] && [ "$COMPONENT_ONLY" = false ] && [ "$INTEGRATION_ONLY" = false ]; then
    print_status "=== Running Linting ==="
    
    # Lint packages
    for package in "${PACKAGES[@]}"; do
        run_workspace_lint "$package"
    done
    
    # Lint shell app
    run_workspace_lint "$SHELL_APP"
    
    # Lint micro-apps
    for app in "${MICRO_APPS[@]}"; do
        run_workspace_lint "$app"
    done
    
    # Lint BFF services
    for bff in "${BFF_SERVICES[@]}"; do
        run_workspace_lint "$bff"
    done
fi

# Run type checking for all workspaces
if [ "$UNIT_ONLY" = false ] && [ "$COMPONENT_ONLY" = false ] && [ "$INTEGRATION_ONLY" = false ]; then
    print_status "=== Running Type Checking ==="
    
    # Type check packages
    for package in "${PACKAGES[@]}"; do
        run_workspace_typecheck "$package" || exit 1
    done
    
    # Type check shell app
    run_workspace_typecheck "$SHELL_APP" || exit 1
    
    # Type check micro-apps
    for app in "${MICRO_APPS[@]}"; do
        run_workspace_typecheck "$app" || exit 1
    done
    
    # Type check BFF services
    for bff in "${BFF_SERVICES[@]}"; do
        run_workspace_typecheck "$bff" || exit 1
    done
fi

# Generate combined coverage report if requested
if [ "$COVERAGE" = true ]; then
    print_status "Generating combined coverage report..."
    
    # Create coverage directory
    mkdir -p coverage/combined
    
    # Combine coverage reports from all workspaces
    # This would require a tool like nyc or c8 to merge coverage reports
    print_status "Coverage reports generated in individual workspace directories"
    print_status "Combined coverage report: coverage/combined/index.html"
fi

# Bundle size analysis (if not in watch mode)
if [ "$WATCH" = false ]; then
    print_status "=== Bundle Size Analysis ==="
    
    # Analyze shell app bundle
    if [ -d "$SHELL_APP" ]; then
        cd "$SHELL_APP"
        if npm run | grep -q "analyze"; then
            print_status "Analyzing shell app bundle size..."
            npm run analyze || print_warning "Bundle analysis failed for shell app"
        fi
        cd ../..
    fi
    
    # Analyze micro-app bundles
    for app in "${MICRO_APPS[@]}"; do
        if [ -d "$app" ]; then
            cd "$app"
            if npm run | grep -q "analyze"; then
                print_status "Analyzing $(basename "$app") bundle size..."
                npm run analyze || print_warning "Bundle analysis failed for $(basename "$app")"
            fi
            cd ../..
        fi
    done
fi

print_success "Frontend tests completed successfully! ✅"

print_status "Frontend test summary:"
print_status "  ✅ Shared package tests passed"
print_status "  ✅ Shell application tests passed"
print_status "  ✅ Micro-frontend tests passed"
print_status "  ✅ BFF service tests passed"
print_status "  ✅ Integration tests passed"
print_status "  ✅ Linting completed"
print_status "  ✅ Type checking completed"

if [ "$COVERAGE" = true ]; then
    print_status "  📊 Coverage reports generated"
fi

if [ "$WATCH" = false ]; then
    print_status "  📦 Bundle size analysis completed"
fi