#!/bin/bash

# ADX CORE Frontend Setup Script
set -e

echo "🚀 Setting up ADX CORE Frontend..."

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ first."
    exit 1
fi

# Check Node.js version
NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js version 18+ is required. Current version: $(node -v)"
    exit 1
fi

echo "✅ Node.js $(node -v) detected"

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Copy environment file
if [ ! -f .env.development ]; then
    echo "📝 Creating environment file..."
    cp .env.example .env.development
    echo "✅ Created .env.development - please review and update as needed"
fi

# Check if Rust is installed for Tauri
if command -v rustc &> /dev/null; then
    echo "✅ Rust $(rustc --version | cut -d' ' -f2) detected"
    
    # Install Tauri CLI if not present
    if ! command -v cargo-tauri &> /dev/null; then
        echo "📦 Installing Tauri CLI..."
        cargo install tauri-cli@next
    fi
    echo "✅ Tauri CLI ready"
else
    echo "⚠️  Rust not found - desktop/mobile builds will not be available"
    echo "   Install Rust from https://rustup.rs/ to enable Tauri features"
fi

# Create necessary directories
mkdir -p src-tauri/icons
mkdir -p public/icons

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Available commands:"
echo "  npm run dev              - Start web development server"
echo "  npm run build            - Build for web production"
echo "  npm run tauri:dev        - Start desktop development (requires Rust)"
echo "  npm run tauri:build      - Build desktop app (requires Rust)"
echo "  npm run test             - Run tests"
echo "  npm run lint             - Run linting"
echo ""
echo "Next steps:"
echo "1. Review and update .env.development with your API endpoints"
echo "2. Start the backend services (see adx-core/README.md)"
echo "3. Run 'npm run dev' to start development"
echo ""
echo "📚 Documentation: frontend/README.md"
echo "🐛 Issues: https://github.com/adx-core/frontend/issues"