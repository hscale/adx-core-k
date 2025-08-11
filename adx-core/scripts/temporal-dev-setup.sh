#!/bin/bash

# ADX Core Temporal Development Setup Script
# This script sets up the complete Temporal development environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCKER_COMPOSE_FILE="$PROJECT_ROOT/infrastructure/docker/docker-compose.temporal.yml"
NAMESPACE_SETUP_SCRIPT="$SCRIPT_DIR/setup-temporal-namespaces.sh"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    log_success "All dependencies are available"
}

setup_temporal_infrastructure() {
    log_info "Setting up Temporal infrastructure..."
    
    # Navigate to project root
    cd "$PROJECT_ROOT"
    
    # Stop any existing containers
    log_info "Stopping existing Temporal containers..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" down --remove-orphans || true
    
    # Pull latest images
    log_info "Pulling latest Temporal images..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" pull
    
    # Start Temporal infrastructure
    log_info "Starting Temporal infrastructure..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" up -d
    
    log_success "Temporal infrastructure started"
}

wait_for_temporal() {
    log_info "Waiting for Temporal server to be ready..."
    
    local max_attempts=60
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if docker exec temporal tctl cluster health &> /dev/null; then
            log_success "Temporal server is ready"
            return 0
        fi
        
        log_info "Attempt $attempt/$max_attempts - Temporal server not ready yet, waiting..."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    log_error "Temporal server failed to start within expected time"
    return 1
}

setup_namespaces() {
    log_info "Setting up Temporal namespaces..."
    
    if [ -f "$NAMESPACE_SETUP_SCRIPT" ]; then
        chmod +x "$NAMESPACE_SETUP_SCRIPT"
        "$NAMESPACE_SETUP_SCRIPT"
    else
        log_error "Namespace setup script not found at $NAMESPACE_SETUP_SCRIPT"
        return 1
    fi
    
    log_success "Temporal namespaces configured"
}

verify_setup() {
    log_info "Verifying Temporal setup..."
    
    # Check if containers are running
    local containers=(
        "temporal-postgresql"
        "temporal"
        "temporal-ui"
    )
    
    for container in "${containers[@]}"; do
        if docker ps --format "table {{.Names}}" | grep -q "^$container$"; then
            log_success "Container $container is running"
        else
            log_error "Container $container is not running"
            return 1
        fi
    done
    
    # Check if Temporal Web UI is accessible
    log_info "Checking Temporal Web UI accessibility..."
    if curl -f http://localhost:8088 &> /dev/null; then
        log_success "Temporal Web UI is accessible at http://localhost:8088"
    else
        log_warning "Temporal Web UI may not be ready yet. It should be available at http://localhost:8088"
    fi
    
    # List namespaces
    log_info "Listing available namespaces..."
    if docker exec temporal tctl namespace list; then
        log_success "Namespaces listed successfully"
    else
        log_warning "Could not list namespaces, but this may be normal during initial setup"
    fi
    
    log_success "Temporal setup verification completed"
}

show_usage_info() {
    echo ""
    echo "🎉 ADX Core Temporal Development Environment Setup Complete!"
    echo ""
    echo "📋 Available Services:"
    echo "  • Temporal Server: localhost:7233"
    echo "  • Temporal Web UI: http://localhost:8088"
    echo "  • PostgreSQL: localhost:5432 (user: temporal, password: temporal)"
    echo ""
    echo "🏢 Available Namespaces:"
    echo "  • adx-core-development (retention: 72h)"
    echo "  • adx-core-staging (retention: 168h)"
    echo "  • adx-core-production (retention: 8760h)"
    echo ""
    echo "🔧 Useful Commands:"
    echo "  • View logs: docker-compose -f $DOCKER_COMPOSE_FILE logs -f"
    echo "  • Stop services: docker-compose -f $DOCKER_COMPOSE_FILE down"
    echo "  • Restart services: docker-compose -f $DOCKER_COMPOSE_FILE restart"
    echo "  • Access Temporal CLI: docker exec -it temporal tctl"
    echo ""
    echo "📚 Next Steps:"
    echo "  1. Open Temporal Web UI: http://localhost:8088"
    echo "  2. Start developing your workflows in the services directory"
    echo "  3. Use the shared Temporal client in services/shared/src/temporal/"
    echo ""
    echo "🛠️  Development Tips:"
    echo "  • Use 'adx-core-development' namespace for local development"
    echo "  • Check workflow execution in the Web UI"
    echo "  • Monitor logs with: docker-compose logs -f temporal"
    echo ""
}

cleanup_on_error() {
    log_error "Setup failed. Cleaning up..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" down --remove-orphans || true
    exit 1
}

main() {
    echo "🚀 Starting ADX Core Temporal Development Environment Setup"
    echo ""
    
    # Set up error handling
    trap cleanup_on_error ERR
    
    # Run setup steps
    check_dependencies
    setup_temporal_infrastructure
    wait_for_temporal
    setup_namespaces
    verify_setup
    show_usage_info
    
    log_success "Setup completed successfully!"
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "ADX Core Temporal Development Setup"
        echo ""
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --clean        Clean up existing setup before starting"
        echo "  --no-verify    Skip verification steps"
        echo ""
        echo "This script sets up a complete Temporal development environment including:"
        echo "  • PostgreSQL database"
        echo "  • Temporal server"
        echo "  • Temporal Web UI"
        echo "  • Development namespaces"
        echo ""
        exit 0
        ;;
    --clean)
        log_info "Cleaning up existing setup..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" down --volumes --remove-orphans || true
        docker system prune -f || true
        log_success "Cleanup completed"
        main
        ;;
    --no-verify)
        log_info "Skipping verification steps..."
        check_dependencies
        setup_temporal_infrastructure
        wait_for_temporal
        setup_namespaces
        show_usage_info
        log_success "Setup completed (verification skipped)!"
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac