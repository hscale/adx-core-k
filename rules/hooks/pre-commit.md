# 🔍 Pre-Commit Quality Gates

## Validation Mission

> **"Nothing reaches the repository without passing world-class quality checks."**

These pre-commit rules ensure every change meets ADX Core's enterprise standards.

## 🎯 Automated Quality Checks

### 1. Code Quality Validation
```bash
#!/bin/bash
# Pre-commit hook script: .git/hooks/pre-commit

set -e  # Exit on any failure

echo "🔍 Running ADX Core pre-commit quality checks..."

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${BLUE}[ADX]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Get list of changed Rust files
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)

if [ -z "$CHANGED_FILES" ]; then
    print_status "No Rust files changed, skipping Rust-specific checks"
else
    print_status "Found changed Rust files:"
    echo "$CHANGED_FILES" | sed 's/^/  - /'
fi

# 1. Rust formatting check
print_status "Checking Rust formatting..."
if ! cargo fmt --check; then
    print_error "Code formatting check failed!"
    print_warning "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi
print_success "Code formatting check passed"

# 2. Clippy linting (only on workspace with Rust files)
if [ -n "$CHANGED_FILES" ]; then
    print_status "Running Clippy linting..."
    if ! cargo clippy --workspace --all-targets --all-features -- -D warnings; then
        print_error "Clippy linting failed!"
        print_warning "Fix all clippy warnings before committing"
        exit 1
    fi
    print_success "Clippy linting passed"
fi

# 3. Security audit
print_status "Running security audit..."
if ! cargo audit; then
    print_error "Security audit failed!"
    print_warning "Address security vulnerabilities before committing"
    exit 1
fi
print_success "Security audit passed"

# 4. Test execution (quick tests only)
if [ -n "$CHANGED_FILES" ]; then
    print_status "Running unit tests..."
    if ! cargo test --workspace --lib; then
        print_error "Unit tests failed!"
        print_warning "All tests must pass before committing"
        exit 1
    fi
    print_success "Unit tests passed"
fi

# 5. Documentation check
print_status "Checking documentation..."
if [ -n "$CHANGED_FILES" ]; then
    # Check for missing documentation on public items
    if ! cargo doc --workspace --no-deps --document-private-items; then
        print_error "Documentation build failed!"
        exit 1
    fi
fi
print_success "Documentation check passed"

# 6. Check for TODO/FIXME in production code
print_status "Checking for TODO/FIXME markers..."
TODO_COUNT=$(git diff --cached --diff-filter=A | grep -E "TODO|FIXME|XXX|HACK" | wc -l || true)
if [ "$TODO_COUNT" -gt 0 ]; then
    print_warning "Found $TODO_COUNT TODO/FIXME markers in new code:"
    git diff --cached --diff-filter=A | grep -n -E "TODO|FIXME|XXX|HACK" || true
    print_warning "Consider addressing these before committing"
    # Don't fail, just warn
fi

# 7. Check for hardcoded secrets
print_status "Scanning for potential secrets..."
SECRET_PATTERNS=(
    "password\s*=\s*['\"][^'\"]*['\"]"
    "secret\s*=\s*['\"][^'\"]*['\"]"
    "api_key\s*=\s*['\"][^'\"]*['\"]"
    "token\s*=\s*['\"][^'\"]*['\"]"
    "['\"][A-Za-z0-9]{32,}['\"]"  # Long strings that might be tokens
)

for pattern in "${SECRET_PATTERNS[@]}"; do
    if git diff --cached | grep -iE "$pattern" >/dev/null; then
        print_error "Potential secret detected in commit!"
        git diff --cached | grep -iE "$pattern" --color=always
        print_warning "Remove secrets before committing. Use environment variables or config files."
        exit 1
    fi
done
print_success "No secrets detected"

# 8. Check commit size
COMMIT_SIZE=$(git diff --cached --numstat | awk '{sum += $1 + $2} END {print sum}')
if [ "$COMMIT_SIZE" -gt 1000 ]; then
    print_warning "Large commit detected ($COMMIT_SIZE lines changed)"
    print_warning "Consider breaking this into smaller commits"
    # Don't fail, just warn
fi

# 9. Architecture compliance check
print_status "Checking architecture compliance..."

# Check for direct database access outside of activities
if echo "$CHANGED_FILES" | xargs grep -l "sqlx::" | grep -v "activities/" | grep -v "src/database" >/dev/null 2>&1; then
    print_error "Direct database access detected outside of activities!"
    echo "$CHANGED_FILES" | xargs grep -l "sqlx::" | grep -v "activities/" | grep -v "src/database"
    print_warning "Database access should only occur in activities or dedicated database modules"
    exit 1
fi

# Check for missing tenant_id in database queries
if echo "$CHANGED_FILES" | xargs grep -l "SELECT.*FROM.*WHERE" | xargs grep -L "tenant_id" >/dev/null 2>&1; then
    print_warning "Database queries without tenant_id found - verify multi-tenant isolation"
    echo "$CHANGED_FILES" | xargs grep -l "SELECT.*FROM.*WHERE" | xargs grep -L "tenant_id"
fi

print_success "Architecture compliance check passed"

# 10. Performance check
print_status "Checking for performance anti-patterns..."

# Check for potential N+1 queries
if echo "$CHANGED_FILES" | xargs grep -n "for.*in.*{" | xargs grep -l "query\|fetch" >/dev/null 2>&1; then
    print_warning "Potential N+1 query pattern detected"
    echo "$CHANGED_FILES" | xargs grep -n "for.*in.*{" | xargs grep -A 5 -B 5 "query\|fetch"
    print_warning "Consider using batch queries or joins instead"
fi

# Check for large Vec allocations
if echo "$CHANGED_FILES" | xargs grep -n "Vec::with_capacity.*[0-9]{4,}" >/dev/null 2>&1; then
    print_warning "Large Vec allocation detected"
    echo "$CHANGED_FILES" | xargs grep -n "Vec::with_capacity.*[0-9]{4,}"
    print_warning "Consider streaming or chunked processing for large datasets"
fi

print_success "Performance check completed"

print_success "All pre-commit checks passed! 🚀"
echo ""
print_status "Commit quality summary:"
echo "  ✅ Code formatting"
echo "  ✅ Linting (Clippy)"
echo "  ✅ Security audit"
echo "  ✅ Unit tests"
echo "  ✅ Documentation"
echo "  ✅ Secret scanning"
echo "  ✅ Architecture compliance"
echo "  ✅ Performance patterns"
echo ""
```

### 2. Temporal Workflow Validation
```bash
# Workflow-specific validation
validate_temporal_workflows() {
    print_status "Validating Temporal workflows..."
    
    # Check workflow naming convention
    WORKFLOW_FILES=$(echo "$CHANGED_FILES" | grep "workflows/" || true)
    
    for file in $WORKFLOW_FILES; do
        # Check for proper workflow attribute
        if ! grep -q "#\[workflow\]" "$file"; then
            if grep -q "async fn.*workflow" "$file"; then
                print_error "Workflow function found without #[workflow] attribute in $file"
                exit 1
            fi
        fi
        
        # Check for proper error handling
        if grep -q "#\[workflow\]" "$file"; then
            if ! grep -q "WorkflowResult" "$file"; then
                print_error "Workflow function should return WorkflowResult in $file"
                exit 1
            fi
            
            # Check for tenant validation
            if ! grep -q "validate_tenant_access" "$file"; then
                print_warning "Workflow missing tenant validation in $file"
            fi
            
            # Check for audit logging
            if ! grep -q "audit_log" "$file"; then
                print_warning "Workflow missing audit logging in $file"
            fi
        fi
    done
    
    # Check activity naming convention
    ACTIVITY_FILES=$(echo "$CHANGED_FILES" | grep "activities/" || true)
    
    for file in $ACTIVITY_FILES; do
        # Check for proper activity attribute
        if ! grep -q "#\[activity\]" "$file"; then
            if grep -q "async fn.*activity" "$file"; then
                print_error "Activity function found without #[activity] attribute in $file"
                exit 1
            fi
        fi
        
        # Check for proper error handling
        if grep -q "#\[activity\]" "$file"; then
            if ! grep -q "Result.*ActivityError" "$file"; then
                print_error "Activity should return Result<T, ActivityError> in $file"
                exit 1
            fi
        fi
    done
    
    print_success "Temporal workflow validation passed"
}

# Call the validation
if [ -n "$CHANGED_FILES" ]; then
    validate_temporal_workflows
fi
```

### 3. Security Validation
```bash
# Security-specific validation
validate_security_patterns() {
    print_status "Validating security patterns..."
    
    # Check for proper authentication middleware usage
    HANDLER_FILES=$(echo "$CHANGED_FILES" | grep "handlers/" || true)
    
    for file in $HANDLER_FILES; do
        # Check if handler needs authentication
        if grep -q "pub async fn" "$file" && ! grep -q "health" "$file"; then
            if ! grep -q "auth_middleware\|AuthClaims\|authenticate" "$file"; then
                print_warning "Handler may be missing authentication in $file"
            fi
        fi
        
        # Check for tenant isolation in handlers
        if grep -q "tenant_id" "$file"; then
            if ! grep -q "claims.tenant_id" "$file"; then
                print_warning "Handler accessing tenant_id but not validating from claims in $file"
            fi
        fi
    done
    
    # Check for password handling
    if echo "$CHANGED_FILES" | xargs grep -n "password" >/dev/null 2>&1; then
        if echo "$CHANGED_FILES" | xargs grep -n "password" | grep -v "hash\|bcrypt\|argon2" >/dev/null 2>&1; then
            print_warning "Password handling detected - ensure proper hashing is used"
            echo "$CHANGED_FILES" | xargs grep -n "password" | grep -v "hash\|bcrypt\|argon2"
        fi
    fi
    
    # Check for SQL injection prevention
    if echo "$CHANGED_FILES" | xargs grep -n "format!\|&format" | grep -E "SELECT|INSERT|UPDATE|DELETE" >/dev/null 2>&1; then
        print_error "Potential SQL injection vulnerability detected!"
        echo "$CHANGED_FILES" | xargs grep -n "format!\|&format" | grep -E "SELECT|INSERT|UPDATE|DELETE"
        print_warning "Use parameterized queries (sqlx::query!) instead of string formatting"
        exit 1
    fi
    
    print_success "Security validation passed"
}

# Call the validation
if [ -n "$CHANGED_FILES" ]; then
    validate_security_patterns
fi
```

### 4. Database Migration Validation
```bash
# Database migration validation
validate_database_changes() {
    print_status "Checking for database migrations..."
    
    # Check if database schema files changed
    MIGRATION_FILES=$(git diff --cached --name-only | grep -E "migrations/|schema\.sql|\.sql$" || true)
    
    if [ -n "$MIGRATION_FILES" ]; then
        print_status "Database changes detected:"
        echo "$MIGRATION_FILES" | sed 's/^/  - /'
        
        # Check for proper migration structure
        for file in $MIGRATION_FILES; do
            if [[ $file == *"migrations/"* ]]; then
                # Check if migration has both up and down
                if [[ $file == *"up.sql" ]]; then
                    DOWN_FILE=${file/up.sql/down.sql}
                    if [ ! -f "$DOWN_FILE" ]; then
                        print_error "Migration $file missing corresponding down migration"
                        exit 1
                    fi
                fi
                
                # Check for dangerous operations
                if grep -iE "DROP TABLE|DROP COLUMN|ALTER TABLE.*DROP" "$file" >/dev/null 2>&1; then
                    print_warning "Potentially destructive migration detected in $file"
                    grep -inE "DROP TABLE|DROP COLUMN|ALTER TABLE.*DROP" "$file"
                    print_warning "Ensure this is intentional and backed up"
                fi
                
                # Check for missing indexes on foreign keys
                if grep -i "REFERENCES" "$file" >/dev/null 2>&1; then
                    if ! grep -i "INDEX\|KEY" "$file" >/dev/null 2>&1; then
                        print_warning "Foreign key without index detected in $file"
                        print_warning "Consider adding indexes for foreign key columns"
                    fi
                fi
            fi
        done
        
        print_success "Database migration validation completed"
    fi
}

# Call the validation
validate_database_changes
```

## 🔧 Performance Validation

### Response Time Checks
```bash
# Performance validation
validate_performance_patterns() {
    print_status "Checking performance patterns..."
    
    # Check for proper async usage
    if echo "$CHANGED_FILES" | xargs grep -n "\.await" | grep -v "async fn\|async move" >/dev/null 2>&1; then
        SYNC_FUNCTIONS=$(echo "$CHANGED_FILES" | xargs grep -l "\.await" | xargs grep -L "async fn")
        if [ -n "$SYNC_FUNCTIONS" ]; then
            print_warning "Using .await in non-async functions detected"
            echo "$SYNC_FUNCTIONS"
        fi
    fi
    
    # Check for blocking operations in async context
    BLOCKING_PATTERNS=(
        "std::thread::sleep"
        "std::fs::"
        "std::net::"
        "reqwest::blocking"
    )
    
    for pattern in "${BLOCKING_PATTERNS[@]}"; do
        if echo "$CHANGED_FILES" | xargs grep -n "$pattern" >/dev/null 2>&1; then
            print_warning "Potentially blocking operation detected: $pattern"
            echo "$CHANGED_FILES" | xargs grep -n "$pattern"
            print_warning "Consider using async alternatives"
        fi
    done
    
    # Check for large allocations
    if echo "$CHANGED_FILES" | xargs grep -nE "vec!\[.*;\s*[0-9]{4,}" >/dev/null 2>&1; then
        print_warning "Large vector allocation detected"
        echo "$CHANGED_FILES" | xargs grep -nE "vec!\[.*;\s*[0-9]{4,}"
        print_warning "Consider lazy loading or streaming"
    fi
    
    print_success "Performance pattern check completed"
}

# Call the validation
if [ -n "$CHANGED_FILES" ]; then
    validate_performance_patterns
fi
```

## 📊 Quality Metrics

### Code Coverage Check
```bash
# Test coverage validation
validate_test_coverage() {
    print_status "Checking test coverage..."
    
    # Only check coverage for significant changes
    if [ "$COMMIT_SIZE" -gt 100 ]; then
        # Run coverage analysis on changed files
        if command -v cargo-tarpaulin >/dev/null 2>&1; then
            COVERAGE=$(cargo tarpaulin --workspace --timeout 60 --out Xml --output-dir target/coverage 2>/dev/null | grep "Coverage" | awk '{print $2}' | sed 's/%//')
            
            if [ -n "$COVERAGE" ]; then
                if [ "$(echo "$COVERAGE < 80" | bc -l)" -eq 1 ]; then
                    print_warning "Test coverage is $COVERAGE% (target: 80%+)"
                    print_warning "Consider adding more tests for new code"
                else
                    print_success "Test coverage: $COVERAGE%"
                fi
            fi
        else
            print_warning "cargo-tarpaulin not installed, skipping coverage check"
            print_warning "Install with: cargo install cargo-tarpaulin"
        fi
    fi
}

# Call the validation (only for larger commits)
validate_test_coverage
```

## 🎯 Integration Checks

### API Compatibility
```bash
# API compatibility check
validate_api_compatibility() {
    print_status "Checking API compatibility..."
    
    # Check for breaking changes in public APIs
    API_FILES=$(echo "$CHANGED_FILES" | grep -E "handlers/|lib\.rs|main\.rs" || true)
    
    for file in $API_FILES; do
        # Check for removed public functions
        REMOVED_FUNCTIONS=$(git diff --cached "$file" | grep "^-.*pub.*fn" || true)
        if [ -n "$REMOVED_FUNCTIONS" ]; then
            print_warning "Public function removal detected in $file:"
            echo "$REMOVED_FUNCTIONS"
            print_warning "This may be a breaking API change"
        fi
        
        # Check for changed function signatures
        CHANGED_SIGNATURES=$(git diff --cached "$file" | grep -E "^[-+].*pub.*fn.*\(" || true)
        if [ -n "$CHANGED_SIGNATURES" ]; then
            print_warning "Public function signature changes detected in $file:"
            echo "$CHANGED_SIGNATURES"
            print_warning "Verify backward compatibility"
        fi
    done
    
    print_success "API compatibility check completed"
}

# Call the validation
if [ -n "$CHANGED_FILES" ]; then
    validate_api_compatibility
fi
```

---

**Quality is non-negotiable. Every commit must meet these standards!** 🔍
