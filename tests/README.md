# ADX CORE End-to-End Integration Testing Framework

This comprehensive integration testing framework validates all aspects of the ADX CORE temporal-first microservices platform, including circuit breakers, user workflows, multi-tenant isolation, load testing, and cross-micro-frontend integration.

## Overview

The integration testing framework provides:

- **Complete Environment Setup**: Automatically starts all services, databases, and infrastructure
- **Circuit Breaker Testing**: Validates error handling and recovery across all services
- **User Workflow Testing**: Tests complete user journeys from registration to usage
- **Multi-Tenant Isolation**: Ensures complete data and security isolation between tenants
- **Load Testing**: Performance validation under realistic load conditions
- **Micro-Frontend Integration**: Tests Module Federation and cross-micro-frontend communication
- **Comprehensive Reporting**: Detailed HTML and JSON test reports

## Architecture

```
tests/
├── integration/
│   ├── mod.rs                    # Main integration test module
│   ├── test_environment.rs       # Test environment setup and management
│   ├── circuit_breakers.rs       # Circuit breaker integration tests
│   ├── user_workflows.rs         # Complete user workflow tests
│   ├── multi_tenant.rs           # Multi-tenant isolation tests
│   ├── load_testing.rs           # Load and performance tests
│   ├── micro_frontend.rs         # Micro-frontend integration tests
│   ├── cross_service.rs          # Cross-service integration tests
│   └── performance.rs            # Performance validation tests
├── integration_tests.rs          # Main test runner
├── Cargo.toml                    # Test dependencies
└── README.md                     # This file
```

## Test Suites

### 1. Circuit Breaker Tests
- **API Gateway Circuit Breakers**: Tests circuit breaker behavior for backend service failures
- **Service-to-Service Circuit Breakers**: Validates circuit breakers between microservices
- **BFF Circuit Breakers**: Tests BFF service circuit breakers and fallback mechanisms
- **Temporal Workflow Circuit Breakers**: Validates workflow activity circuit breakers
- **Database Circuit Breakers**: Tests database connection and query timeout circuit breakers
- **Redis Circuit Breakers**: Validates cache circuit breakers and fallback behavior

### 2. User Workflow Tests
- **User Registration Workflow**: Complete user registration with email verification
- **User Login Workflow**: Authentication, token validation, and session management
- **Tenant Switching Workflow**: Multi-tenant context switching with security validation
- **File Management Workflow**: File upload, processing, sharing, and management
- **User Profile Workflow**: Profile management and preference updates
- **Module Workflow**: Module installation, activation, and usage
- **Workflow Monitoring**: Workflow status tracking and management
- **User Deactivation Workflow**: Account deactivation and cleanup

### 3. Multi-Tenant Isolation Tests
- **Data Isolation**: Ensures complete data separation between tenants
- **User Access Control**: Validates tenant-based access control
- **Workflow Isolation**: Tests workflow isolation between tenants
- **File Storage Isolation**: Ensures file storage isolation
- **API Tenant Filtering**: Validates API endpoint tenant filtering
- **Cross-Tenant Leakage Prevention**: Tests for data leakage prevention
- **Tenant Switching Security**: Security validation for tenant switching
- **Quota Enforcement**: Per-tenant quota and rate limiting

### 4. Load Testing Suite
- **API Endpoint Load**: Concurrent API request handling
- **Workflow Execution Load**: Concurrent workflow execution performance
- **Concurrent User Simulation**: Realistic user session simulation
- **Database Performance Load**: Database performance under load
- **File Upload Load**: Concurrent file upload handling
- **Multi-Tenant Load Isolation**: Load isolation between tenants

### 5. Micro-Frontend Integration Tests
- **Module Federation Loading**: Tests Module Federation configuration and loading
- **Cross-Micro-Frontend Communication**: Event bus and shared state testing
- **Shared State Management**: Tenant context and authentication state
- **Micro-Frontend Isolation**: Error boundaries and failure isolation
- **BFF Integration**: BFF service integration with micro-frontends
- **Error Boundaries**: Graceful failure handling and fallbacks

## Prerequisites

### Required Software
- **Docker & Docker Compose**: For infrastructure services
- **Rust 1.88+**: For backend services and test runner
- **Node.js 18+**: For frontend micro-services
- **PostgreSQL**: Database (via Docker)
- **Redis**: Caching (via Docker)
- **Temporal**: Workflow engine (via Docker)

### System Requirements
- **Memory**: Minimum 8GB RAM (16GB recommended for load testing)
- **CPU**: Multi-core processor (4+ cores recommended)
- **Disk**: 10GB free space
- **Network**: Internet connection for Docker images

## Quick Start

### 1. Run All Tests
```bash
# Run complete integration test suite
./scripts/run-integration-tests.sh

# Run with load testing enabled
./scripts/run-integration-tests.sh --enable-load-testing

# Run with custom configuration
./scripts/run-integration-tests.sh \
  --enable-load-testing \
  --max-users 50 \
  --test-duration 180 \
  --verbose
```

### 2. Run Specific Test Suites
```bash
# Build and run tests manually
cd tests
cargo build --bin integration_tests
cargo run --bin integration_tests
```

### 3. Environment Variables
```bash
# Core configuration
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/adx_core_test"
export REDIS_URL="redis://localhost:6379"
export TEMPORAL_URL="http://localhost:7233"
export API_GATEWAY_URL="http://localhost:8080"
export FRONTEND_SHELL_URL="http://localhost:3000"

# Load testing configuration
export ENABLE_LOAD_TESTING="true"
export MAX_CONCURRENT_USERS="100"
export TEST_DURATION_SECONDS="300"
```

## Configuration Options

### Script Options
```bash
--enable-load-testing    # Enable load testing (default: false)
--max-users NUM         # Maximum concurrent users (default: 100)
--test-duration SEC     # Test duration in seconds (default: 300)
--no-cleanup           # Don't cleanup on exit (default: cleanup)
--verbose              # Enable verbose output (default: false)
--help                 # Show help message
```

### Environment Variables
- `ENABLE_LOAD_TESTING`: Enable/disable load testing
- `MAX_CONCURRENT_USERS`: Maximum concurrent users for load testing
- `TEST_DURATION_SECONDS`: Duration for load tests
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `TEMPORAL_URL`: Temporal server URL
- `API_GATEWAY_URL`: API Gateway URL
- `FRONTEND_SHELL_URL`: Frontend shell application URL

## Test Reports

The framework generates comprehensive test reports:

### HTML Report (`integration_test_report.html`)
- Interactive test results with visual indicators
- Detailed test suite breakdown
- Performance metrics and charts
- Failed test details with assertions
- Environment information

### JSON Report (`integration_test_report.json`)
- Machine-readable test results
- Complete test execution data
- Performance metrics
- Error details and stack traces
- Suitable for CI/CD integration

## Performance Criteria

### API Performance
- **Response Time**: <200ms for 95th percentile
- **Throughput**: >100 requests/second
- **Success Rate**: >95% under normal load
- **Error Rate**: <5% under load

### Workflow Performance
- **Initiation Time**: <1000ms for workflow start
- **Execution Time**: <5s for 90% of workflows, <30s for complex workflows
- **Success Rate**: >90% workflow completion
- **Temporal UI**: All workflows visible and debuggable

### Frontend Performance
- **Loading Time**: <2s initial load, <500ms micro-frontend switches
- **Module Federation**: <500KB per micro-frontend bundle
- **Cross-Communication**: <100ms event bus latency
- **Error Recovery**: Graceful degradation on micro-frontend failures

### Multi-Tenant Performance
- **Isolation**: 100% data isolation between tenants
- **Security**: 0% cross-tenant data leakage
- **Performance**: <10% performance impact from tenant switching
- **Quota Enforcement**: 100% quota compliance

## Troubleshooting

### Common Issues

#### Services Not Starting
```bash
# Check Docker status
docker info

# Check port availability
netstat -tulpn | grep :8080

# Check service logs
docker-compose -f adx-core/infrastructure/docker/docker-compose.dev.yml logs
```

#### Database Connection Issues
```bash
# Test database connection
docker exec -it $(docker-compose ps -q postgres) psql -U postgres -d adx_core_test

# Check database migrations
cd adx-core && sqlx migrate info
```

#### Frontend Build Issues
```bash
# Clear node modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Check individual micro-frontend builds
cd apps/shell && npm run build
```

#### Temporal Connection Issues
```bash
# Check Temporal server status
curl http://localhost:8088

# Check Temporal UI
open http://localhost:8088
```

### Debug Mode
```bash
# Run with verbose output
./scripts/run-integration-tests.sh --verbose

# Run individual test components
cd tests
RUST_LOG=debug cargo run --bin integration_tests
```

### Manual Service Testing
```bash
# Test API Gateway
curl http://localhost:8080/health

# Test individual services
curl http://localhost:8081/health  # Auth Service
curl http://localhost:8082/health  # User Service
curl http://localhost:8083/health  # File Service

# Test frontend services
curl http://localhost:3000  # Shell Application
curl http://localhost:3001  # Auth Micro-Frontend
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Integration Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
    
    - name: Run Integration Tests
      run: |
        chmod +x scripts/run-integration-tests.sh
        ./scripts/run-integration-tests.sh --enable-load-testing
    
    - name: Upload Test Reports
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: integration-test-reports
        path: |
          tests/integration_test_report.html
          tests/integration_test_report.json
```

## Contributing

### Adding New Tests
1. Create test module in `tests/integration/`
2. Implement test suite struct with `run_all_tests()` method
3. Add individual test methods returning `TestResult`
4. Update main test runner to include new suite
5. Update documentation

### Test Structure
```rust
pub struct NewTestSuite {
    env: Arc<IntegrationTestEnvironment>,
    test_data: TestData,
}

impl NewTestSuite {
    pub async fn run_all_tests(&self) -> Result<IntegrationTestResults, Box<dyn std::error::Error>> {
        // Implementation
    }
    
    async fn test_specific_functionality(&self) -> TestResult {
        // Individual test implementation
    }
}
```

### Best Practices
- Use descriptive test names and assertions
- Include comprehensive error messages
- Test both success and failure scenarios
- Validate performance criteria
- Ensure proper cleanup
- Document test purpose and expectations

## Support

For issues with the integration testing framework:

1. Check the troubleshooting section above
2. Review test logs and reports
3. Verify all prerequisites are installed
4. Check service health endpoints
5. Run tests with verbose output for debugging

The integration testing framework is designed to provide comprehensive validation of the ADX CORE platform's reliability, performance, and security across all components and user workflows.