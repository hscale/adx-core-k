# ADX CORE Development Scripts

This directory contains automation scripts for the ADX CORE development workflow.

## Scripts Overview

### `dev-start.sh`
Starts the complete development environment including infrastructure services and builds the Rust workspace.

**Usage:**
```bash
./scripts/dev-start.sh
```

**What it does:**
- Starts Docker infrastructure (PostgreSQL, Redis, Temporal)
- Waits for all services to be ready
- Builds the Rust workspace
- Provides service URLs and next steps

### `dev-stop.sh`
Stops all development services and optionally cleans up Docker resources.

**Usage:**
```bash
./scripts/dev-stop.sh [--clean]
```

**Options:**
- `--clean`: Remove unused Docker containers, images, and networks

### `build.sh`
Builds all services with different profiles and targets.

**Usage:**
```bash
./scripts/build.sh [OPTIONS]
```

**Options:**
- `--release`: Build in release mode (optimized)
- `--dev`: Build in development mode (default)
- `--target <arch>`: Specify target architecture
- `--clean`: Clean before building
- `--verbose`: Verbose output
- `--docker`: Also build Docker images

### `test.sh`
Runs comprehensive tests across all services.

**Usage:**
```bash
./scripts/test.sh [OPTIONS]
```

**Options:**
- `--unit`: Run only unit tests
- `--integration`: Run only integration tests
- `--workflow`: Run only workflow tests
- `--coverage`: Generate coverage report
- `--verbose`: Verbose output

### `deploy.sh`
Handles deployment to different environments.

**Usage:**
```bash
./scripts/deploy.sh --env <environment> [OPTIONS]
```

**Environments:**
- `development`: Local development deployment
- `staging`: Staging environment deployment
- `production`: Production deployment (with additional safeguards)

**Options:**
- `--version <tag>`: Version tag for deployment
- `--dry-run`: Show what would be deployed without executing
- `--skip-tests`: Skip running tests before deployment

## Development Workflow

### Initial Setup
```bash
# Start development environment
./scripts/dev-start.sh

# Build all services
./scripts/build.sh
```

### Daily Development
```bash
# Run tests
./scripts/test.sh

# Build and test
./scripts/build.sh && ./scripts/test.sh

# Stop environment when done
./scripts/dev-stop.sh
```

### Before Committing
```bash
# Run full test suite with coverage
./scripts/test.sh --coverage

# Ensure clean build
./scripts/build.sh --clean --release
```

### Deployment
```bash
# Deploy to development
./scripts/deploy.sh --env development

# Deploy to staging (dry run first)
./scripts/deploy.sh --env staging --dry-run
./scripts/deploy.sh --env staging

# Deploy to production (with extra caution)
./scripts/deploy.sh --env production --dry-run
./scripts/deploy.sh --env production
```

## Prerequisites

- Docker and docker-compose
- Rust toolchain (latest stable)
- Git
- curl (for health checks)

## Environment Variables

The scripts use the following environment variables:

- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `TEMPORAL_SERVER_URL`: Temporal server URL
- `RUST_LOG`: Logging level for Rust applications

## Troubleshooting

### Docker Issues
```bash
# Check Docker status
docker info

# Restart Docker services
./scripts/dev-stop.sh --clean
./scripts/dev-start.sh
```

### Build Issues
```bash
# Clean build
./scripts/build.sh --clean

# Verbose build for debugging
./scripts/build.sh --verbose
```

### Test Issues
```bash
# Run specific test type
./scripts/test.sh --unit
./scripts/test.sh --integration

# Verbose test output
./scripts/test.sh --verbose
```

## Contributing

When adding new scripts:

1. Make them executable: `chmod +x script-name.sh`
2. Follow the existing pattern for argument parsing and output formatting
3. Include help text with `--help` option
4. Add error handling and validation
5. Update this README with the new script documentation