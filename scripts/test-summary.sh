#!/bin/bash

# ADX CORE Test Summary Script
# Provides a comprehensive overview of all testing capabilities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

print_header() {
    echo -e "${PURPLE}$1${NC}"
}

print_section() {
    echo -e "\n${CYAN}=== $1 ===${NC}\n"
}

# Navigate to workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

print_header "ADX CORE Testing Infrastructure Summary"
print_header "======================================="

print_section "Available Testing Scripts"

echo "📋 Core Testing Scripts:"
echo "  ./scripts/test-all.sh           - Run all test suites (backend, frontend, workflows, integration, e2e)"
echo "  ./scripts/test-backend.sh       - Run Rust backend tests (unit, service, repository)"
echo "  ./scripts/test-frontend.sh      - Run TypeScript frontend tests (unit, component, integration)"
echo "  ./scripts/test-workflows.sh     - Run Temporal workflow tests (activities, workflows, replay)"
echo "  ./scripts/test-integration.sh   - Run cross-service integration tests"
echo "  ./scripts/test-e2e.sh           - Run end-to-end tests with Playwright"

echo ""
echo "🔧 Utility Scripts:"
echo "  ./scripts/validate-setup.sh     - Validate development environment"
echo "  ./scripts/health-check.sh       - Check system and service health"
echo "  ./scripts/debug-services.sh     - Debug and troubleshoot services"

print_section "Test Categories"

echo "🦀 Backend Tests (Rust):"
echo "  • Unit Tests          - Individual function and module testing"
echo "  • Service Tests       - HTTP endpoint and business logic testing"
echo "  • Repository Tests    - Database layer testing with test containers"
echo "  • Integration Tests   - Cross-service communication testing"
echo "  • Performance Tests   - Load and stress testing"

echo ""
echo "⚛️ Frontend Tests (TypeScript/React):"
echo "  • Unit Tests          - Individual component and hook testing"
echo "  • Component Tests     - React component integration testing"
echo "  • Integration Tests   - Cross-micro-frontend communication testing"
echo "  • Accessibility Tests - WCAG compliance and screen reader testing"
echo "  • Performance Tests   - Bundle size and rendering performance"

echo ""
echo "🔄 Workflow Tests (Temporal):"
echo "  • Activity Tests      - Individual Temporal activity testing"
echo "  • Workflow Tests      - Complete workflow execution testing"
echo "  • Replay Tests        - Workflow versioning and compatibility testing"
echo "  • Error Handling      - Compensation and retry logic testing"
echo "  • Cross-Service       - Multi-service workflow orchestration testing"

echo ""
echo "🌐 End-to-End Tests (Playwright):"
echo "  • User Journey Tests  - Complete user workflows"
echo "  • Cross-Platform      - Web, desktop, and mobile testing"
echo "  • Performance Tests   - Page load and interaction performance"
echo "  • Accessibility Tests - Full application accessibility testing"

print_section "Testing Features"

echo "✨ Key Features:"
echo "  • Parallel Execution     - Run tests in parallel for faster feedback"
echo "  • Coverage Reports       - Generate comprehensive coverage reports"
echo "  • Multi-Platform         - Test across different operating systems"
echo "  • Multi-Browser          - Test across Chrome, Firefox, Safari"
echo "  • Multi-Tenant Testing   - Validate tenant isolation and switching"
echo "  • Temporal Integration   - Test workflow execution and monitoring"
echo "  • Module Federation      - Test micro-frontend loading and communication"
echo "  • Performance Monitoring - Track response times and resource usage"
echo "  • Security Testing       - Validate authentication and authorization"
echo "  • Database Testing       - Test with real database connections"

print_section "Quick Start Guide"

echo "🚀 Getting Started:"
echo ""
echo "1. Validate your environment:"
echo "   ./scripts/validate-setup.sh"
echo ""
echo "2. Start infrastructure services:"
echo "   docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d"
echo ""
echo "3. Run all tests:"
echo "   ./scripts/test-all.sh"
echo ""
echo "4. Run specific test categories:"
echo "   ./scripts/test-backend.sh --coverage"
echo "   ./scripts/test-frontend.sh --verbose"
echo "   ./scripts/test-workflows.sh"
echo "   ./scripts/test-integration.sh"
echo "   ./scripts/test-e2e.sh --headed"

print_section "Test Configuration"

echo "🔧 Environment Variables:"
echo "  DATABASE_URL              - PostgreSQL connection string"
echo "  REDIS_URL                 - Redis connection string"
echo "  TEMPORAL_SERVER_URL       - Temporal server address"
echo "  NODE_ENV                  - Node.js environment (test)"
echo "  RUST_LOG                  - Rust logging level"
echo "  TEST_MODE                 - Enable test mode"
echo "  SKIP_DB_TESTS            - Skip database-dependent tests"
echo "  SKIP_TEMPORAL_TESTS      - Skip Temporal-dependent tests"

echo ""
echo "📁 Test Directories:"
echo "  tests/                    - Root test directory"
echo "  tests/e2e/               - End-to-end test files"
echo "  tests/integration/       - Integration test files"
echo "  tests/fixtures/          - Test data and fixtures"
echo "  adx-core/services/*/tests/ - Service-specific tests"
echo "  adx-core/apps/*/src/**/__tests__/ - Frontend component tests"

print_section "Test Reports"

echo "📊 Generated Reports:"
echo "  target/test-results/     - All test results and reports"
echo "  target/coverage/         - Backend coverage reports (HTML)"
echo "  coverage/                - Frontend coverage reports"
echo "  target/debug-reports/    - Debug and diagnostic reports"
echo "  target/health-reports/   - System health check reports"
echo "  target/validation-reports/ - Environment validation reports"

print_section "Continuous Integration"

echo "🔄 CI/CD Integration:"
echo "  • GitHub Actions ready   - Pre-configured workflows"
echo "  • Docker support         - Containerized test execution"
echo "  • Parallel execution     - Optimized for CI environments"
echo "  • Artifact generation    - Test reports and coverage data"
echo "  • Failure notifications  - Detailed error reporting"

print_section "Best Practices"

echo "💡 Testing Best Practices:"
echo "  • Test-Driven Development - Write tests before implementation"
echo "  • Temporal-First Testing  - Test workflows as primary business logic"
echo "  • Multi-Tenant Testing    - Validate tenant isolation at all levels"
echo "  • Performance Testing     - Include performance assertions"
echo "  • Security Testing        - Test authentication and authorization"
echo "  • Accessibility Testing   - Ensure WCAG compliance"
echo "  • Cross-Platform Testing  - Validate across different environments"

print_section "Troubleshooting"

echo "🔍 Common Issues:"
echo "  • Port conflicts         - Use debug-services.sh to check port usage"
echo "  • Database connectivity  - Ensure PostgreSQL is running and accessible"
echo "  • Temporal connectivity  - Verify Temporal server is running"
echo "  • Memory issues          - Monitor system resources during tests"
echo "  • Test timeouts          - Adjust timeout values for slower systems"

echo ""
echo "🛠️ Debug Commands:"
echo "  ./scripts/debug-services.sh --service all --health"
echo "  ./scripts/health-check.sh"
echo "  ./scripts/validate-setup.sh"

print_section "Example Usage"

echo "📝 Example Test Commands:"
echo ""
echo "# Run all tests with coverage"
echo "./scripts/test-all.sh --coverage --verbose"
echo ""
echo "# Run only backend unit tests"
echo "./scripts/test-backend.sh --unit-only --coverage"
echo ""
echo "# Run frontend tests in watch mode"
echo "./scripts/test-frontend.sh --watch"
echo ""
echo "# Run workflow tests with replay testing"
echo "./scripts/test-workflows.sh --replay-only"
echo ""
echo "# Run integration tests for API only"
echo "./scripts/test-integration.sh --api-only"
echo ""
echo "# Run E2E tests in headed mode"
echo "./scripts/test-e2e.sh --headed --browser firefox"
echo ""
echo "# Debug specific service"
echo "./scripts/debug-services.sh --service api-gateway --logs"

print_section "Next Steps"

echo "🎯 Recommended Actions:"
echo ""
echo "1. Run environment validation:"
echo "   ./scripts/validate-setup.sh"
echo ""
echo "2. Start with a quick test run:"
echo "   ./scripts/test-all.sh --backend-only"
echo ""
echo "3. Set up your IDE for testing:"
echo "   - Configure test runners"
echo "   - Set up debugging"
echo "   - Install recommended extensions"
echo ""
echo "4. Explore the test files:"
echo "   - Review example tests"
echo "   - Understand testing patterns"
echo "   - Add your own tests"

print_success "Testing infrastructure is ready! 🚀"
print_status "For detailed help on any script, run: <script> --help"
print_status "Happy testing! 🧪"