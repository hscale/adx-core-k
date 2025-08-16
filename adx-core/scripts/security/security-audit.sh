#!/bin/bash

# ADX Core Security Audit Script
# Comprehensive security audit for production environment

set -euo pipefail

# Configuration
AUDIT_DIR="/var/log/adx-core/security-audit"
REPORT_FILE="$AUDIT_DIR/security-audit-$(date '+%Y%m%d_%H%M%S').json"
TEMP_DIR="/tmp/security-audit-$$"

# Logging
LOG_FILE="/var/log/adx-core/security-audit.log"
exec 1> >(tee -a "$LOG_FILE")
exec 2>&1

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Create directories
mkdir -p "$AUDIT_DIR" "$TEMP_DIR"

log "Starting security audit..."

# Initialize audit report
cat > "$REPORT_FILE" << EOF
{
    "audit_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "audit_version": "1.0",
    "environment": "production",
    "findings": [],
    "summary": {
        "total_checks": 0,
        "passed": 0,
        "failed": 0,
        "warnings": 0
    }
}
EOF

# Function to add finding to report
add_finding() {
    local severity="$1"
    local category="$2"
    local title="$3"
    local description="$4"
    local recommendation="$5"
    
    jq --arg severity "$severity" \
       --arg category "$category" \
       --arg title "$title" \
       --arg description "$description" \
       --arg recommendation "$recommendation" \
       '.findings += [{
           "severity": $severity,
           "category": $category,
           "title": $title,
           "description": $description,
           "recommendation": $recommendation,
           "timestamp": now | strftime("%Y-%m-%dT%H:%M:%SZ")
       }]' "$REPORT_FILE" > "$TEMP_DIR/report.tmp" && mv "$TEMP_DIR/report.tmp" "$REPORT_FILE"
}

# Function to update summary
update_summary() {
    local status="$1"
    jq --arg status "$status" '
        .summary.total_checks += 1 |
        if $status == "passed" then .summary.passed += 1
        elif $status == "failed" then .summary.failed += 1
        else .summary.warnings += 1
        end' "$REPORT_FILE" > "$TEMP_DIR/report.tmp" && mv "$TEMP_DIR/report.tmp" "$REPORT_FILE"
}

# 1. Container Security Audit
log "Auditing container security..."

# Check for running containers with privileged mode
PRIVILEGED_CONTAINERS=$(docker ps --format "table {{.Names}}\t{{.Status}}" --filter "label=privileged=true" | tail -n +2)
if [ -n "$PRIVILEGED_CONTAINERS" ]; then
    add_finding "HIGH" "Container Security" "Privileged containers detected" \
        "Found containers running in privileged mode: $PRIVILEGED_CONTAINERS" \
        "Remove privileged mode unless absolutely necessary and implement proper capability management"
    update_summary "failed"
else
    update_summary "passed"
fi

# Check for containers running as root
ROOT_CONTAINERS=$(docker ps --format "{{.Names}}" | xargs -I {} docker exec {} whoami 2>/dev/null | grep -c "^root$" || true)
if [ "$ROOT_CONTAINERS" -gt 0 ]; then
    add_finding "MEDIUM" "Container Security" "Containers running as root" \
        "Found $ROOT_CONTAINERS containers running as root user" \
        "Configure containers to run as non-root users"
    update_summary "failed"
else
    update_summary "passed"
fi

# 2. Network Security Audit
log "Auditing network security..."

# Check for open ports
OPEN_PORTS=$(nmap -sT localhost | grep "open" | wc -l)
if [ "$OPEN_PORTS" -gt 20 ]; then
    add_finding "MEDIUM" "Network Security" "Too many open ports" \
        "Found $OPEN_PORTS open ports on localhost" \
        "Review and close unnecessary ports, implement proper firewall rules"
    update_summary "warning"
else
    update_summary "passed"
fi

# Check SSL/TLS configuration
SSL_CHECK=$(curl -s -I https://localhost:443 2>/dev/null | grep -i "strict-transport-security" || echo "")
if [ -z "$SSL_CHECK" ]; then
    add_finding "MEDIUM" "Network Security" "Missing HSTS header" \
        "HTTPS endpoints do not include Strict-Transport-Security header" \
        "Configure HSTS headers on all HTTPS endpoints"
    update_summary "failed"
else
    update_summary "passed"
fi

# 3. Database Security Audit
log "Auditing database security..."

# Check PostgreSQL configuration
if command -v psql >/dev/null 2>&1; then
    # Check for default passwords
    DEFAULT_PASS_CHECK=$(PGPASSWORD="postgres" psql -h localhost -U postgres -c "SELECT 1;" 2>/dev/null || echo "failed")
    if [ "$DEFAULT_PASS_CHECK" != "failed" ]; then
        add_finding "CRITICAL" "Database Security" "Default database password" \
            "PostgreSQL is using default password" \
            "Change default passwords immediately and use strong passwords"
        update_summary "failed"
    else
        update_summary "passed"
    fi
    
    # Check for SSL enforcement
    SSL_SETTING=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h localhost -U "$POSTGRES_USER" -d "$POSTGRES_DB" -t -c "SHOW ssl;" 2>/dev/null | tr -d ' ' || echo "off")
    if [ "$SSL_SETTING" = "off" ]; then
        add_finding "HIGH" "Database Security" "Database SSL disabled" \
            "PostgreSQL SSL is disabled" \
            "Enable SSL for all database connections"
        update_summary "failed"
    else
        update_summary "passed"
    fi
fi

# 4. Authentication and Authorization Audit
log "Auditing authentication and authorization..."

# Check JWT secret strength
if [ -n "${JWT_SECRET:-}" ]; then
    JWT_LENGTH=${#JWT_SECRET}
    if [ "$JWT_LENGTH" -lt 32 ]; then
        add_finding "HIGH" "Authentication" "Weak JWT secret" \
            "JWT secret is too short ($JWT_LENGTH characters)" \
            "Use a JWT secret of at least 32 characters with high entropy"
        update_summary "failed"
    else
        update_summary "passed"
    fi
else
    add_finding "CRITICAL" "Authentication" "Missing JWT secret" \
        "JWT_SECRET environment variable is not set" \
        "Configure a strong JWT secret"
    update_summary "failed"
fi

# Check for hardcoded credentials in configuration
HARDCODED_CREDS=$(grep -r -i "password\|secret\|key" adx-core/infrastructure/production/ | grep -v ".env" | grep -E "(password|secret|key)\s*[:=]\s*['\"][^'\"]{1,}" || true)
if [ -n "$HARDCODED_CREDS" ]; then
    add_finding "CRITICAL" "Authentication" "Hardcoded credentials" \
        "Found hardcoded credentials in configuration files" \
        "Remove hardcoded credentials and use environment variables or secrets management"
    update_summary "failed"
else
    update_summary "passed"
fi

# 5. File System Security Audit
log "Auditing file system security..."

# Check file permissions
WORLD_WRITABLE=$(find /var/log/adx-core -type f -perm -002 2>/dev/null | wc -l)
if [ "$WORLD_WRITABLE" -gt 0 ]; then
    add_finding "MEDIUM" "File System" "World-writable files" \
        "Found $WORLD_WRITABLE world-writable files in log directory" \
        "Fix file permissions to prevent unauthorized access"
    update_summary "failed"
else
    update_summary "passed"
fi

# Check for sensitive files with weak permissions
SENSITIVE_FILES=("/etc/passwd" "/etc/shadow" "/etc/ssh/sshd_config")
for file in "${SENSITIVE_FILES[@]}"; do
    if [ -f "$file" ]; then
        PERMS=$(stat -c "%a" "$file")
        case "$file" in
            "/etc/shadow")
                if [ "$PERMS" != "640" ] && [ "$PERMS" != "600" ]; then
                    add_finding "HIGH" "File System" "Weak permissions on $file" \
                        "File $file has permissions $PERMS" \
                        "Set appropriate permissions (600 or 640) on sensitive files"
                    update_summary "failed"
                fi
                ;;
            *)
                if [ "${PERMS:2:1}" -gt 4 ]; then
                    add_finding "MEDIUM" "File System" "Weak permissions on $file" \
                        "File $file has world-readable permissions $PERMS" \
                        "Review and restrict permissions on sensitive files"
                    update_summary "warning"
                fi
                ;;
        esac
    fi
done

# 6. Application Security Audit
log "Auditing application security..."

# Check for security headers
SECURITY_HEADERS=("X-Frame-Options" "X-Content-Type-Options" "X-XSS-Protection" "Content-Security-Policy")
for header in "${SECURITY_HEADERS[@]}"; do
    HEADER_CHECK=$(curl -s -I http://localhost:8080 | grep -i "$header" || echo "")
    if [ -z "$HEADER_CHECK" ]; then
        add_finding "MEDIUM" "Application Security" "Missing security header: $header" \
            "HTTP response does not include $header header" \
            "Configure security headers in web server or application"
        update_summary "failed"
    else
        update_summary "passed"
    fi
done

# Check for exposed debug endpoints
DEBUG_ENDPOINTS=("/debug" "/admin" "/metrics" "/health")
for endpoint in "${DEBUG_ENDPOINTS[@]}"; do
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:8080$endpoint" || echo "000")
    if [ "$RESPONSE" = "200" ] && [ "$endpoint" != "/health" ]; then
        add_finding "HIGH" "Application Security" "Exposed debug endpoint: $endpoint" \
            "Debug endpoint $endpoint is publicly accessible" \
            "Restrict access to debug endpoints or disable in production"
        update_summary "failed"
    else
        update_summary "passed"
    fi
done

# 7. Dependency Security Audit
log "Auditing dependencies..."

# Check for known vulnerabilities in Rust dependencies
if command -v cargo-audit >/dev/null 2>&1; then
    cd adx-core
    RUST_VULNS=$(cargo audit --json 2>/dev/null | jq '.vulnerabilities.found | length' || echo "0")
    if [ "$RUST_VULNS" -gt 0 ]; then
        add_finding "HIGH" "Dependencies" "Vulnerable Rust dependencies" \
            "Found $RUST_VULNS vulnerable Rust dependencies" \
            "Update vulnerable dependencies to secure versions"
        update_summary "failed"
    else
        update_summary "passed"
    fi
    cd ..
fi

# Check for known vulnerabilities in Node.js dependencies
if command -v npm >/dev/null 2>&1; then
    NPM_AUDIT=$(npm audit --json 2>/dev/null | jq '.metadata.vulnerabilities.total' || echo "0")
    if [ "$NPM_AUDIT" -gt 0 ]; then
        add_finding "HIGH" "Dependencies" "Vulnerable Node.js dependencies" \
            "Found $NPM_AUDIT vulnerable Node.js dependencies" \
            "Run 'npm audit fix' to update vulnerable dependencies"
        update_summary "failed"
    else
        update_summary "passed"
    fi
fi

# 8. Logging and Monitoring Security
log "Auditing logging and monitoring..."

# Check log file permissions
LOG_PERMS=$(stat -c "%a" "$LOG_FILE" 2>/dev/null || echo "000")
if [ "$LOG_PERMS" != "640" ] && [ "$LOG_PERMS" != "600" ]; then
    add_finding "MEDIUM" "Logging" "Weak log file permissions" \
        "Log file has permissions $LOG_PERMS" \
        "Set appropriate permissions (640) on log files"
    update_summary "failed"
else
    update_summary "passed"
fi

# Check for sensitive data in logs
SENSITIVE_PATTERNS=("password" "secret" "token" "key" "credit.card" "ssn")
for pattern in "${SENSITIVE_PATTERNS[@]}"; do
    MATCHES=$(grep -i "$pattern" "$LOG_FILE" 2>/dev/null | wc -l || echo "0")
    if [ "$MATCHES" -gt 0 ]; then
        add_finding "HIGH" "Logging" "Sensitive data in logs" \
            "Found $MATCHES potential sensitive data entries matching '$pattern'" \
            "Review logging practices and sanitize sensitive data"
        update_summary "failed"
        break
    fi
done

# 9. Backup Security Audit
log "Auditing backup security..."

# Check backup encryption
if [ -d "/backups" ]; then
    UNENCRYPTED_BACKUPS=$(find /backups -name "*.sql" -o -name "*.dump" | wc -l)
    if [ "$UNENCRYPTED_BACKUPS" -gt 0 ]; then
        add_finding "HIGH" "Backup Security" "Unencrypted backups" \
            "Found $UNENCRYPTED_BACKUPS unencrypted backup files" \
            "Encrypt all backup files and secure backup storage"
        update_summary "failed"
    else
        update_summary "passed"
    fi
fi

# 10. Generate final report
log "Generating security audit report..."

# Calculate risk score
CRITICAL_COUNT=$(jq '.findings | map(select(.severity == "CRITICAL")) | length' "$REPORT_FILE")
HIGH_COUNT=$(jq '.findings | map(select(.severity == "HIGH")) | length' "$REPORT_FILE")
MEDIUM_COUNT=$(jq '.findings | map(select(.severity == "MEDIUM")) | length' "$REPORT_FILE")
LOW_COUNT=$(jq '.findings | map(select(.severity == "LOW")) | length' "$REPORT_FILE")

RISK_SCORE=$((CRITICAL_COUNT * 10 + HIGH_COUNT * 5 + MEDIUM_COUNT * 2 + LOW_COUNT * 1))

# Add risk assessment to report
jq --argjson risk_score "$RISK_SCORE" \
   --argjson critical "$CRITICAL_COUNT" \
   --argjson high "$HIGH_COUNT" \
   --argjson medium "$MEDIUM_COUNT" \
   --argjson low "$LOW_COUNT" \
   '.risk_assessment = {
       "risk_score": $risk_score,
       "risk_level": (if $risk_score > 50 then "CRITICAL" elif $risk_score > 20 then "HIGH" elif $risk_score > 10 then "MEDIUM" else "LOW" end),
       "severity_breakdown": {
           "critical": $critical,
           "high": $high,
           "medium": $medium,
           "low": $low
       }
   }' "$REPORT_FILE" > "$TEMP_DIR/report.tmp" && mv "$TEMP_DIR/report.tmp" "$REPORT_FILE"

# Generate summary
TOTAL_CHECKS=$(jq '.summary.total_checks' "$REPORT_FILE")
PASSED=$(jq '.summary.passed' "$REPORT_FILE")
FAILED=$(jq '.summary.failed' "$REPORT_FILE")
WARNINGS=$(jq '.summary.warnings' "$REPORT_FILE")

log "Security audit completed:"
log "  Total checks: $TOTAL_CHECKS"
log "  Passed: $PASSED"
log "  Failed: $FAILED"
log "  Warnings: $WARNINGS"
log "  Risk score: $RISK_SCORE"
log "  Report saved: $REPORT_FILE"

# Send notification
if [ -n "${SLACK_WEBHOOK_URL:-}" ]; then
    RISK_LEVEL=$(jq -r '.risk_assessment.risk_level' "$REPORT_FILE")
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"🔒 Security audit completed. Risk level: $RISK_LEVEL, Score: $RISK_SCORE. Report: $REPORT_FILE\"}" \
        "$SLACK_WEBHOOK_URL"
fi

# Cleanup
rm -rf "$TEMP_DIR"

# Exit with appropriate code
if [ "$CRITICAL_COUNT" -gt 0 ]; then
    exit 2
elif [ "$HIGH_COUNT" -gt 0 ]; then
    exit 1
else
    exit 0
fi