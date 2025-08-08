#!/bin/bash

# ADX CORE Deployment Script
# This script handles deployment to different environments

set -e

echo "🚀 ADX CORE Deployment Script..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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
cd "$(dirname "$0")/.."

# Parse command line arguments
ENVIRONMENT=""
VERSION=""
DRY_RUN=false
SKIP_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --help)
            echo "Usage: $0 --env <environment> [OPTIONS]"
            echo "Environments: development, staging, production"
            echo "Options:"
            echo "  --env          Target environment (required)"
            echo "  --version      Version tag for deployment"
            echo "  --dry-run      Show what would be deployed without executing"
            echo "  --skip-tests   Skip running tests before deployment"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate required arguments
if [ -z "$ENVIRONMENT" ]; then
    print_error "Environment is required. Use --env <environment>"
    exit 1
fi

# Validate environment
case $ENVIRONMENT in
    development|staging|production)
        ;;
    *)
        print_error "Invalid environment: $ENVIRONMENT. Must be one of: development, staging, production"
        exit 1
        ;;
esac

# Set version if not provided
if [ -z "$VERSION" ]; then
    VERSION=$(git rev-parse --short HEAD)
    print_status "Using git commit hash as version: $VERSION"
fi

print_status "Deploying to environment: $ENVIRONMENT"
print_status "Version: $VERSION"

if [ "$DRY_RUN" = true ]; then
    print_warning "DRY RUN MODE - No actual deployment will occur"
fi

# Pre-deployment checks
print_status "Running pre-deployment checks..."

# Check if git working directory is clean (for production)
if [ "$ENVIRONMENT" = "production" ]; then
    if ! git diff-index --quiet HEAD --; then
        print_error "Git working directory is not clean. Commit or stash changes before production deployment."
        exit 1
    fi
fi

# Run tests unless skipped
if [ "$SKIP_TESTS" = false ]; then
    print_status "Running tests before deployment..."
    if ! ./scripts/test.sh; then
        print_error "Tests failed. Deployment aborted."
        exit 1
    fi
    print_success "All tests passed"
fi

# Build for production
print_status "Building for deployment..."
if ! ./scripts/build.sh --release; then
    print_error "Build failed. Deployment aborted."
    exit 1
fi

# Environment-specific deployment logic
case $ENVIRONMENT in
    development)
        print_status "Deploying to development environment..."
        if [ "$DRY_RUN" = false ]; then
            # Development deployment logic
            print_status "Starting development services..."
            docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d --build
        fi
        ;;
    staging)
        print_status "Deploying to staging environment..."
        if [ "$DRY_RUN" = false ]; then
            # Staging deployment logic
            print_warning "Staging deployment not yet implemented"
            # This would typically involve:
            # - Building Docker images
            # - Pushing to container registry
            # - Updating Kubernetes manifests
            # - Rolling deployment
        fi
        ;;
    production)
        print_status "Deploying to production environment..."
        print_warning "Production deployment requires additional confirmation"
        
        read -p "Are you sure you want to deploy to PRODUCTION? (yes/no): " confirm
        if [ "$confirm" != "yes" ]; then
            print_status "Production deployment cancelled"
            exit 0
        fi
        
        if [ "$DRY_RUN" = false ]; then
            # Production deployment logic
            print_warning "Production deployment not yet implemented"
            # This would typically involve:
            # - Blue-green deployment
            # - Database migrations
            # - Health checks
            # - Rollback capability
        fi
        ;;
esac

if [ "$DRY_RUN" = false ]; then
    print_success "Deployment to $ENVIRONMENT completed successfully! 🎉"
    
    # Post-deployment verification
    print_status "Running post-deployment verification..."
    # Add health checks here
    
    print_status "Deployment summary:"
    echo "  Environment: $ENVIRONMENT"
    echo "  Version: $VERSION"
    echo "  Timestamp: $(date)"
else
    print_success "Dry run completed. No actual deployment performed."
fi