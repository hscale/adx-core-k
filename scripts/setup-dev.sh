#!/bin/bash

# ADX CORE Development Setup Script
# This script installs all dependencies and prepares the development environment

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

echo "🛠️  ADX CORE Development Environment Setup"
echo "=========================================="

# Check prerequisites
print_status "Checking prerequisites..."

# Check Node.js
if ! command -v node > /dev/null 2>&1; then
    print_error "Node.js is not installed. Please install Node.js 18+ and try again."
    exit 1
fi
print_success "Node.js $(node --version) is installed"

# Check npm
if ! command -v npm > /dev/null 2>&1; then
    print_error "npm is not installed. Please install npm and try again."
    exit 1
fi
print_success "npm $(npm --version) is installed"

# Check Rust
if ! command -v cargo > /dev/null 2>&1; then
    print_error "Rust is not installed. Please install Rust and try again."
    print_status "Visit: https://rustup.rs/"
    exit 1
fi
print_success "Rust $(rustc --version) is installed"

# Check Docker
if ! command -v docker > /dev/null 2>&1; then
    print_error "Docker is not installed. Please install Docker Desktop and try again."
    exit 1
fi
print_success "Docker $(docker --version) is installed"

# Install root dependencies
print_status "Installing root dependencies..."
if npm install; then
    print_success "Root dependencies installed"
else
    print_error "Failed to install root dependencies"
    exit 1
fi

# Install Rust dependencies and build
print_status "Building Rust workspace..."
cd adx-core
if cargo build --workspace; then
    print_success "Rust workspace built successfully"
else
    print_error "Failed to build Rust workspace"
    exit 1
fi
cd ..

# Install frontend dependencies
print_status "Installing frontend dependencies..."

# Install dependencies for each micro-frontend
for app in shell auth tenant file user workflow module; do
    if [ -d "apps/$app" ]; then
        print_status "Installing dependencies for $app micro-frontend..."
        cd "apps/$app"
        if npm install; then
            print_success "$app dependencies installed"
        else
            print_error "Failed to install $app dependencies"
            exit 1
        fi
        cd ../..
    else
        print_warning "$app directory not found, skipping..."
    fi
done

# Install BFF dependencies
print_status "Installing BFF service dependencies..."
for bff in auth-bff tenant-bff file-bff user-bff workflow-bff module-bff; do
    if [ -d "bff-services/$bff" ]; then
        print_status "Installing dependencies for $bff..."
        cd "bff-services/$bff"
        if [ -f "package.json" ]; then
            if npm install; then
                print_success "$bff dependencies installed"
            else
                print_error "Failed to install $bff dependencies"
                exit 1
            fi
        elif [ -f "Cargo.toml" ]; then
            if cargo build; then
                print_success "$bff built successfully"
            else
                print_error "Failed to build $bff"
                exit 1
            fi
        fi
        cd ../..
    else
        print_warning "$bff directory not found, skipping..."
    fi
done

# Create necessary directories
print_status "Creating necessary directories..."
mkdir -p logs
mkdir -p uploads
mkdir -p adx-core/uploads
print_success "Directories created"

# Set up environment files
print_status "Setting up environment files..."
if [ ! -f "adx-core/.env" ]; then
    if [ -f "adx-core/.env.example" ]; then
        cp adx-core/.env.example adx-core/.env
        print_success "Environment file created from example"
    else
        print_warning "No .env.example found, .env file already exists or will be created by startup script"
    fi
else
    print_success "Environment file already exists"
fi

# Install additional tools if needed
print_status "Checking for additional tools..."

# Check if sqlx-cli is installed
if ! command -v sqlx > /dev/null 2>&1; then
    print_status "Installing sqlx-cli for database migrations..."
    cargo install sqlx-cli --no-default-features --features postgres
    print_success "sqlx-cli installed"
else
    print_success "sqlx-cli is already installed"
fi

# Check if concurrently is installed globally
if ! command -v concurrently > /dev/null 2>&1; then
    print_status "Installing concurrently for running multiple services..."
    npm install -g concurrently
    print_success "concurrently installed globally"
else
    print_success "concurrently is already installed"
fi

print_success "🎉 Development environment setup completed!"
echo ""
print_status "Next steps:"
echo "  1. Start Docker Desktop if not already running"
echo "  2. Run: ./scripts/dev-start-all.sh"
echo "  3. Wait for all services to start (may take a few minutes)"
echo "  4. Run: ./scripts/health-check.sh to verify everything is working"
echo "  5. Open http://localhost:3000 in your browser"
echo ""
print_status "Useful commands:"
echo "  - Start all services: ./scripts/dev-start-all.sh"
echo "  - Check service health: ./scripts/health-check.sh"
echo "  - View logs: tail -f logs/[service-name].log"
echo "  - Stop all services: Ctrl+C in the terminal running dev-start-all.sh"