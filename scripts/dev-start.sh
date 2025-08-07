#!/bin/bash

echo "🚀 Starting ADX CORE Development Environment"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Navigate to the adx-core project directory
cd "$(dirname "$0")/../adx-core"

# Start infrastructure services
echo "📦 Starting infrastructure services..."
cd infrastructure/docker
docker compose -f docker-compose.dev.yml up -d

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 15

# Check service health
echo "🔍 Checking service health..."
docker compose -f docker-compose.dev.yml ps

# Go back to adx-core root
cd ../../

# Build all services
echo "🔨 Building all services..."
cargo build --workspace

# Start API Gateway
echo "🌐 Starting API Gateway..."
cargo run --bin api-gateway &
API_GATEWAY_PID=$!

# Start Auth Service
echo "🔐 Starting Auth Service..."
cargo run --bin auth-service &
AUTH_SERVICE_PID=$!

# Start User Service
echo "👥 Starting User Service..."
cargo run --bin user-service &
USER_SERVICE_PID=$!

# Start File Service  
echo "📁 Starting File Service..."
cargo run --bin file-service &
FILE_SERVICE_PID=$!

# Start Workflow Service
echo "⚡ Starting Workflow Service..."
cargo run --bin workflow-service &
WORKFLOW_SERVICE_PID=$!

# Start Tenant Service
echo "🏢 Starting Tenant Service..."
cargo run --bin tenant-service &
TENANT_SERVICE_PID=$!

echo ""
echo "✅ Development environment ready!"
echo "📊 Services:"
echo "  - API Gateway: http://localhost:8080"
echo "  - Auth Service: http://localhost:8081"
echo "  - User Service: http://localhost:8082"
echo "  - File Service: http://localhost:8083"
echo "  - Workflow Service: http://localhost:8084"
echo "  - Tenant Service: http://localhost:8085"
echo "  - Temporal UI: http://localhost:8088"
echo "  - Database: postgresql://adx_user:dev_password@localhost:5432/adx_core"
echo "  - Redis: redis://localhost:6379"
# Health check
curl http://localhost:8080/health

# User service
curl http://localhost:8080/api/v1/users

# Auth endpoints  
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}'

echo ""
echo "🧪 Test endpoints:"
echo "  - Health: curl http://localhost:8080/health"
echo "  - Login: curl -X POST http://localhost:8081/api/v1/auth/login -H 'Content-Type: application/json' -d '{\"email\":\"admin@example.com\",\"password\":\"password\",\"tenant_id\":\"550e8400-e29b-41d4-a716-446655440000\"}'"
echo "  - Users: curl http://localhost:8080/api/v1/users"
echo "  - Create User: curl -X POST http://localhost:8080/api/v1/users -H 'Content-Type: application/json' -d '{\"email\":\"test@example.com\",\"password\":\"password\"}'"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for interrupt
trap "echo '🛑 Stopping services...'; kill $API_GATEWAY_PID $AUTH_SERVICE_PID $USER_SERVICE_PID $FILE_SERVICE_PID $WORKFLOW_SERVICE_PID $TENANT_SERVICE_PID 2>/dev/null; cd infrastructure/docker; docker compose -f docker-compose.dev.yml down; echo '✅ All services stopped'" EXIT
wait