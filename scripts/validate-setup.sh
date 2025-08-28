#!/bin/bash

# ADX CORE Setup Validation Script
# Validates the development environment and all dependencies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

print_check() {
    echo -e "${PURPLE}[CHECK]${NC} $1"
}

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Validation results tracking
VALIDATION_ERRORS=0
VALIDATION_WARNINGS=0

# Function to check command availability
check_command() {
    local cmd="$1"
    local description="$2"
    local required="$3"
    
    print_check "Checking $description..."
    
    if command -v "$cmd" &> /dev/null; then
        local version=$($cmd --version 2>/dev/null | head -1 || echo "unknown")
        print_success "$description is available: $version"
        return 0
    else
        if [ "$required" = "true" ]; then
            print_error "$description is not installed (required)"
            VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
        else
            print_warning "$description is not installed (optional)"
            VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
        fi
        return 1
    fi
}

# Function to check file/directory existence
check_path() {
    local path="$1"
    local description="$2"
    local required="$3"
    
    print_check "Checking $description..."
    
    if [ -e "$path" ]; then
        print_success "$description exists: $path"
        return 0
    else
        if [ "$required" = "true" ]; then
            print_error "$description not found: $path (required)"
            VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
        else
            print_warning "$description not found: $path (optional)"
            VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
        fi
        return 1
    fi
}

# Function to check port availability
check_port() {
    local port="$1"
    local service="$2"
    local required="$3"
    
    print_check "Checking port $port for $service..."
    
    if netstat -tuln 2>/dev/null | grep -q ":$port "; then
        print_warning "Port $port is already in use (may conflict with $service)"
        if [ "$required" = "true" ]; then
            VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
        fi
        return 1
    else
        print_success "Port $port is available for $service"
        return 0
    fi
}

# Function to check service connectivity
check_service() {
    local url="$1"
    local service="$2"
    local required="$3"
    
    print_check "Checking $service connectivity..."
    
    if curl -s --connect-timeout 5 "$url" > /dev/null 2>&1; then
        print_success "$service is accessible at $url"
        return 0
    else
        if [ "$required" = "true" ]; then
            print_error "$service is not accessible at $url (required)"
            VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
        else
            print_warning "$service is not accessible at $url (optional)"
            VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
        fi
        return 1
    fi
}

print_status "ADX CORE Environment Validation"
print_status "==============================="

# System Requirements
print_status "Checking System Requirements..."
check_command "uname" "Operating System" "true"

# Check OS compatibility
OS=$(uname -s)
case $OS in
    "Linux")
        print_success "Running on Linux (supported)"
        ;;
    "Darwin")
        print_success "Running on macOS (supported)"
        ;;
    "MINGW"*|"CYGWIN"*|"MSYS"*)
        print_success "Running on Windows (supported)"
        ;;
    *)
        print_warning "Running on $OS (may not be fully supported)"
        VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
        ;;
esac

# Development Tools
print_status "Checking Development Tools..."
check_command "git" "Git" "true"
check_command "curl" "cURL" "true"
check_command "docker" "Docker" "true"
check_command "docker-compose" "Docker Compose" "true"

# Backend Dependencies
print_status "Checking Backend Dependencies..."
check_command "rustc" "Rust Compiler" "true"
check_command "cargo" "Cargo Package Manager" "true"

if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_check "Checking Rust version..."
    
    # Check if Rust version is 1.70 or higher
    if [ "$(printf '%s\n' "1.70.0" "$RUST_VERSION" | sort -V | head -n1)" = "1.70.0" ]; then
        print_success "Rust version $RUST_VERSION is compatible"
    else
        print_error "Rust version $RUST_VERSION is too old (minimum: 1.70.0)"
        VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
    fi
fi

# Frontend Dependencies
print_status "Checking Frontend Dependencies..."
check_command "node" "Node.js" "true"
check_command "npm" "NPM" "true"

if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version | cut -d'v' -f2)
    print_check "Checking Node.js version..."
    
    # Check if Node.js version is 18 or higher
    if [ "$(printf '%s\n' "18.0.0" "$NODE_VERSION" | sort -V | head -n1)" = "18.0.0" ]; then
        print_success "Node.js version $NODE_VERSION is compatible"
    else
        print_error "Node.js version $NODE_VERSION is too old (minimum: 18.0.0)"
        VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
    fi
fi

# Optional Tools
print_status "Checking Optional Tools..."
check_command "yarn" "Yarn Package Manager" "false"
check_command "pnpm" "PNPM Package Manager" "false"
check_command "sqlx" "SQLx CLI" "false"
check_command "cargo-tarpaulin" "Cargo Tarpaulin (Coverage)" "false"
check_command "cargo-audit" "Cargo Audit (Security)" "false"
check_command "playwright" "Playwright (E2E Testing)" "false"

# Project Structure
print_status "Checking Project Structure..."
check_path "adx-core" "ADX Core Directory" "true"
check_path "adx-core/Cargo.toml" "Workspace Cargo.toml" "true"
check_path "adx-core/services" "Services Directory" "true"
check_path "adx-core/apps" "Apps Directory" "true"
check_path "adx-core/packages" "Packages Directory" "true"
check_path "adx-core/infrastructure" "Infrastructure Directory" "true"
check_path "package.json" "Root Package.json" "true"

# Configuration Files
print_status "Checking Configuration Files..."
check_path "adx-core/infrastructure/docker/docker-compose.dev.yml" "Docker Compose Config" "true"
check_path "adx-core/.env.example" "Environment Example" "false"
check_path ".gitignore" "Git Ignore File" "true"

# Service Directories
print_status "Checking Service Structure..."
SERVICES=("shared" "api-gateway" "auth-service" "user-service" "file-service" "tenant-service" "workflow-service")
for service in "${SERVICES[@]}"; do
    check_path "adx-core/services/$service" "$service Directory" "true"
    check_path "adx-core/services/$service/Cargo.toml" "$service Cargo.toml" "true"
done

# Frontend Applications
print_status "Checking Frontend Structure..."
APPS=("shell" "auth" "tenant" "file" "user" "workflow" "module")
for app in "${APPS[@]}"; do
    check_path "adx-core/apps/$app" "$app App Directory" "true"
    check_path "adx-core/apps/$app/package.json" "$app Package.json" "true"
done

# Shared Packages
print_status "Checking Shared Packages..."
PACKAGES=("design-system" "shared-context" "event-bus" "i18n" "shared")
for package in "${PACKAGES[@]}"; do
    check_path "adx-core/packages/$package" "$package Package Directory" "true"
done

# Port Availability
print_status "Checking Port Availability..."
# Infrastructure ports
check_port "5432" "PostgreSQL" "false"
check_port "6379" "Redis" "false"
check_port "7233" "Temporal Server" "false"
check_port "8088" "Temporal UI" "false"

# Backend service ports
check_port "8080" "API Gateway" "false"
check_port "8081" "Auth Service" "false"
check_port "8082" "User Service" "false"
check_port "8083" "File Service" "false"
check_port "8084" "Workflow Service" "false"
check_port "8085" "Tenant Service" "false"

# Frontend application ports
check_port "3000" "Shell Application" "false"
check_port "3001" "Auth Micro-Frontend" "false"
check_port "3002" "Tenant Micro-Frontend" "false"
check_port "3003" "File Micro-Frontend" "false"
check_port "3004" "User Micro-Frontend" "false"
check_port "3005" "Workflow Micro-Frontend" "false"
check_port "3006" "Module Micro-Frontend" "false"

# BFF service ports
check_port "4001" "Auth BFF" "false"
check_port "4002" "Tenant BFF" "false"
check_port "4003" "File BFF" "false"
check_port "4004" "User BFF" "false"
check_port "4005" "Workflow BFF" "false"
check_port "4006" "Module BFF" "false"

# Docker Infrastructure
print_status "Checking Docker Infrastructure..."
if command -v docker &> /dev/null; then
    print_check "Checking Docker daemon..."
    if docker info > /dev/null 2>&1; then
        print_success "Docker daemon is running"
        
        # Check Docker Compose services
        print_check "Checking Docker Compose services..."
        if [ -f "adx-core/infrastructure/docker/docker-compose.dev.yml" ]; then
            docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml config > /dev/null 2>&1
            if [ $? -eq 0 ]; then
                print_success "Docker Compose configuration is valid"
            else
                print_error "Docker Compose configuration is invalid"
                VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
            fi
        fi
    else
        print_error "Docker daemon is not running"
        VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
    fi
fi

# Database Connectivity (if running)
print_status "Checking Service Connectivity..."
check_service "http://localhost:5432" "PostgreSQL" "false"
check_service "http://localhost:6379" "Redis" "false"
check_service "http://localhost:8088/health" "Temporal UI" "false"

# Rust Compilation Check
print_status "Checking Rust Compilation..."
if [ -f "adx-core/Cargo.toml" ]; then
    print_check "Testing Rust workspace compilation..."
    cd adx-core
    if cargo check --workspace > /dev/null 2>&1; then
        print_success "Rust workspace compiles successfully"
    else
        print_error "Rust workspace compilation failed"
        VALIDATION_ERRORS=$((VALIDATION_ERRORS + 1))
    fi
    cd ..
fi

# Node.js Dependencies Check
print_status "Checking Node.js Dependencies..."
if [ -f "package.json" ]; then
    print_check "Checking root package.json dependencies..."
    if npm list --depth=0 > /dev/null 2>&1; then
        print_success "Root dependencies are satisfied"
    else
        print_warning "Root dependencies may need installation (run: npm install)"
        VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
    fi
fi

# Environment Variables
print_status "Checking Environment Variables..."
ENV_VARS=("DATABASE_URL" "REDIS_URL" "TEMPORAL_SERVER_URL")
for var in "${ENV_VARS[@]}"; do
    if [ -n "${!var}" ]; then
        print_success "Environment variable $var is set"
    else
        print_warning "Environment variable $var is not set (will use defaults)"
        VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
    fi
done

# Disk Space Check
print_status "Checking Disk Space..."
if command -v df &> /dev/null; then
    AVAILABLE_SPACE=$(df . | tail -1 | awk '{print $4}')
    AVAILABLE_GB=$((AVAILABLE_SPACE / 1024 / 1024))
    
    if [ $AVAILABLE_GB -gt 5 ]; then
        print_success "Sufficient disk space available: ${AVAILABLE_GB}GB"
    else
        print_warning "Low disk space: ${AVAILABLE_GB}GB (recommended: >5GB)"
        VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
    fi
fi

# Memory Check
print_status "Checking Memory..."
if command -v free &> /dev/null; then
    AVAILABLE_MEM=$(free -m | awk 'NR==2{printf "%.0f", $7}')
    
    if [ $AVAILABLE_MEM -gt 2048 ]; then
        print_success "Sufficient memory available: ${AVAILABLE_MEM}MB"
    else
        print_warning "Low memory: ${AVAILABLE_MEM}MB (recommended: >2GB)"
        VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
    fi
elif command -v vm_stat &> /dev/null; then
    # macOS memory check
    FREE_PAGES=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    FREE_MB=$((FREE_PAGES * 4096 / 1024 / 1024))
    
    if [ $FREE_MB -gt 2048 ]; then
        print_success "Sufficient memory available: ${FREE_MB}MB"
    else
        print_warning "Low memory: ${FREE_MB}MB (recommended: >2GB)"
        VALIDATION_WARNINGS=$((VALIDATION_WARNINGS + 1))
    fi
fi

# Generate validation report
print_status "Generating Validation Report..."
mkdir -p target/validation-reports
REPORT_FILE="target/validation-reports/validation_report_$(date +%Y%m%d_%H%M%S).md"

cat > "$REPORT_FILE" << EOF
# ADX CORE Environment Validation Report

**Generated:** $(date)  
**System:** $(uname -a)  
**User:** $(whoami)

## Validation Summary
- **Errors:** $VALIDATION_ERRORS
- **Warnings:** $VALIDATION_WARNINGS
- **Status:** $([ $VALIDATION_ERRORS -eq 0 ] && echo "✅ PASSED" || echo "❌ FAILED")

## System Information
- **OS:** $(uname -s)
- **Architecture:** $(uname -m)
- **Kernel:** $(uname -r)

## Tool Versions
- **Git:** $(git --version 2>/dev/null || echo "Not installed")
- **Docker:** $(docker --version 2>/dev/null || echo "Not installed")
- **Rust:** $(rustc --version 2>/dev/null || echo "Not installed")
- **Node.js:** $(node --version 2>/dev/null || echo "Not installed")
- **NPM:** $(npm --version 2>/dev/null || echo "Not installed")

## Recommendations
EOF

# Add recommendations based on validation results
if [ $VALIDATION_ERRORS -gt 0 ]; then
    echo "### Critical Issues (Must Fix)" >> "$REPORT_FILE"
    echo "- Fix all validation errors before proceeding with development" >> "$REPORT_FILE"
    echo "- Run this validation script again after fixing issues" >> "$REPORT_FILE"
fi

if [ $VALIDATION_WARNINGS -gt 0 ]; then
    echo "### Warnings (Should Fix)" >> "$REPORT_FILE"
    echo "- Address validation warnings for optimal development experience" >> "$REPORT_FILE"
fi

if ! command -v docker &> /dev/null; then
    echo "- Install Docker: https://docs.docker.com/get-docker/" >> "$REPORT_FILE"
fi

if ! command -v rustc &> /dev/null; then
    echo "- Install Rust: https://rustup.rs/" >> "$REPORT_FILE"
fi

if ! command -v node &> /dev/null; then
    echo "- Install Node.js: https://nodejs.org/" >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"
echo "## Next Steps" >> "$REPORT_FILE"
if [ $VALIDATION_ERRORS -eq 0 ]; then
    echo "1. Start infrastructure: \`docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d\`" >> "$REPORT_FILE"
    echo "2. Install dependencies: \`npm install\`" >> "$REPORT_FILE"
    echo "3. Build backend: \`cd adx-core && cargo build\`" >> "$REPORT_FILE"
    echo "4. Start development: \`./scripts/dev-start-all.sh\`" >> "$REPORT_FILE"
else
    echo "1. Fix all validation errors listed above" >> "$REPORT_FILE"
    echo "2. Re-run validation: \`./scripts/validate-setup.sh\`" >> "$REPORT_FILE"
fi

print_success "Validation report generated: $REPORT_FILE"

# Final summary
print_status "Validation Summary"
print_status "=================="
if [ $VALIDATION_ERRORS -eq 0 ]; then
    print_success "Environment validation PASSED ✅"
    print_status "Errors: $VALIDATION_ERRORS, Warnings: $VALIDATION_WARNINGS"
    print_status "Your development environment is ready!"
    exit 0
else
    print_error "Environment validation FAILED ❌"
    print_status "Errors: $VALIDATION_ERRORS, Warnings: $VALIDATION_WARNINGS"
    print_status "Please fix the errors above before proceeding."
    exit 1
fi