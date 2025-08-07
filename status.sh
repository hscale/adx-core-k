#!/bin/bash

# ============================================================================
# ADX CORE Status Check Script
# Quick status overview of all services
# ============================================================================

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔍 ADX CORE System Status${NC}"
echo "=========================="
echo ""

# Check backend services
echo -e "${BLUE}📊 Backend Services:${NC}"
services=(
    "8080:API Gateway"
    "8081:Auth Service"
    "8082:User Service"
    "8083:File Service"
    "8084:Workflow Service"
    "8085:Tenant Service"
)

for service in "${services[@]}"; do
    port=$(echo $service | cut -d: -f1)
    name=$(echo $service | cut -d: -f2)
    
    if curl -s http://localhost:$port/health > /dev/null; then
        echo -e "  ${GREEN}✅ $name${NC} - http://localhost:$port"
    else
        echo -e "  ${RED}❌ $name${NC} - http://localhost:$port (not responding)"
    fi
done

echo ""

# Check infrastructure
echo -e "${BLUE}🔧 Infrastructure:${NC}"

# PostgreSQL
if docker exec docker-postgres-1 pg_isready -U adx_user -d adx_core > /dev/null 2>&1; then
    echo -e "  ${GREEN}✅ PostgreSQL${NC} - localhost:5432"
else
    echo -e "  ${RED}❌ PostgreSQL${NC} - localhost:5432"
fi

# Redis
if docker exec docker-redis-1 redis-cli ping | grep -q PONG 2>/dev/null; then
    echo -e "  ${GREEN}✅ Redis${NC} - localhost:6379"
else
    echo -e "  ${RED}❌ Redis${NC} - localhost:6379"
fi

# Temporal UI
if curl -s http://localhost:8088 > /dev/null; then
    echo -e "  ${GREEN}✅ Temporal UI${NC} - http://localhost:8088"
else
    echo -e "  ${RED}❌ Temporal UI${NC} - http://localhost:8088"
fi

echo ""

# Check frontend
echo -e "${BLUE}🌐 Frontend:${NC}"
if curl -s http://localhost:1420 > /dev/null; then
    echo -e "  ${GREEN}✅ React Frontend${NC} - http://localhost:1420"
else
    echo -e "  ${YELLOW}⚠️  React Frontend${NC} - http://localhost:1420 (not running or has errors)"
fi

echo ""

# Process information
echo -e "${BLUE}🔄 Running Processes:${NC}"
if pgrep -f "cargo run" > /dev/null; then
    echo -e "  ${GREEN}✅ Backend services running${NC}"
    echo "     PIDs: $(pgrep -f 'cargo run' | tr '\n' ' ')"
else
    echo -e "  ${RED}❌ No backend services running${NC}"
fi

if pgrep -f "npm run dev" > /dev/null; then
    echo -e "  ${GREEN}✅ Frontend service running${NC}"
else
    echo -e "  ${YELLOW}⚠️  Frontend service not running${NC}"
fi

echo ""

# Quick actions
echo -e "${BLUE}⚡ Quick Actions:${NC}"
echo "  • Start all:     ./quick-dev.sh"
echo "  • Test APIs:     ./test-api.sh"
echo "  • Stop backend:  pkill -f 'cargo run'"
echo "  • View logs:     tail -f logs/auth-service.log"
echo "  • Restart infra: docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml restart"

echo ""