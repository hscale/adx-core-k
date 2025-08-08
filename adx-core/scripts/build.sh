#!/bin/bash

# ADX CORE Build Script
# This script builds all services with different profiles and targets

set -e

echo "🔨 Building ADX CORE Services..."

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
BUILD_PROFILE="dev"
TARGET=""
CLEAN=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_PROFILE="release"
            shift
            ;;
        --dev)
            BUILD_PROFILE="dev"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --release      Build in release mode (optimized)"
            echo "  --dev          Build in development mode (default)"
            echo "  --target       Specify target architecture"
            echo "  --clean        Clean before building"
            echo "  --verbose      Verbose output"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    print_status "Cleaning previous builds..."
    cargo clean
    print_success "Clean completed"
fi

# Build flags
BUILD_FLAGS=""
if [ "$BUILD_PROFILE" = "release" ]; then
    BUILD_FLAGS="$BUILD_FLAGS --release"
fi

if [ -n "$TARGET" ]; then
    BUILD_FLAGS="$BUILD_FLAGS --target $TARGET"
fi

if [ "$VERBOSE" = true ]; then
    BUILD_FLAGS="$BUILD_FLAGS --verbose"
fi

# Check Rust toolchain
print_status "Checking Rust toolchain..."
if ! command -v cargo > /dev/null 2>&1; then
    print_error "Cargo is not installed. Please install Rust and try again."
    exit 1
fi

RUST_VERSION=$(rustc --version)
print_status "Using Rust: $RUST_VERSION"

# Build workspace
print_status "Building workspace with profile: $BUILD_PROFILE"
if cargo build $BUILD_FLAGS; then
    print_success "Workspace build completed successfully"
else
    print_error "Workspace build failed"
    exit 1
fi

# List built binaries
print_status "Built binaries:"
if [ "$BUILD_PROFILE" = "release" ]; then
    BINARY_DIR="target/release"
else
    BINARY_DIR="target/debug"
fi

if [ -n "$TARGET" ]; then
    BINARY_DIR="target/$TARGET/$BUILD_PROFILE"
fi

if [ -d "$BINARY_DIR" ]; then
    ls -la "$BINARY_DIR" | grep -E "(auth-service|user-service|file-service|tenant-service|workflow-service)" || true
fi

# Build Docker images if requested
if [ "$1" = "--docker" ]; then
    print_status "Building Docker images..."
    
    # Build base image
    docker build -t adx-core-base -f infrastructure/docker/Dockerfile.base .
    
    # Build service images
    for service in auth-service user-service file-service tenant-service workflow-service; do
        print_status "Building Docker image for $service..."
        docker build -t "adx-core-$service" -f "services/$service/Dockerfile" .
    done
    
    print_success "Docker images built successfully"
fi

print_success "Build process completed! 🎉"

# Show next steps
print_status "Next steps:"
echo "  - Run services: cargo run --bin <service-name>"
echo "  - Run tests: ./scripts/test.sh"
echo "  - Start dev environment: ./scripts/dev-start.sh"