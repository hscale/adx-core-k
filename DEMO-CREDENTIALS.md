# ADX CORE Demo Credentials

## 🔐 Demo Login Credentials

### Default Admin User
- **Email**: `admin@example.com`
- **Password**: `password`
- **Tenant ID**: `550e8400-e29b-41d4-a716-446655440000`
- **Role**: Admin
- **Tenant**: Demo Tenant (demo.adx-core.com)

## 🌐 Frontend Login

### Access the Frontend
1. **URL**: http://localhost:1420
2. **Login Page**: http://localhost:1420/auth/login (if redirected)

### Login Steps
1. Open http://localhost:1420 in your browser
2. If not automatically redirected, navigate to the login page
3. Enter credentials:
   - **Email**: `admin@example.com`
   - **Password**: `password`
4. Click "Sign In" or "Login"

## 🧪 API Testing

### Direct API Authentication
```bash
# Test login via API
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "admin@example.com",
    "password": "password",
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

### Expected Response
```json
{
  "data": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "...",
    "user": {
      "id": "...",
      "email": "admin@example.com",
      "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
    },
    "tenant": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Demo Tenant",
      "domain": "demo.adx-core.com"
    }
  }
}
```

## 🏢 Tenant Information

### Demo Tenant Details
- **ID**: `550e8400-e29b-41d4-a716-446655440000`
- **Name**: Demo Tenant
- **Domain**: demo.adx-core.com
- **Status**: Active

## 🔧 Creating Additional Demo Users

### Via API (after logging in as admin)
```bash
# Get auth token first
TOKEN=$(curl -s -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}' \
  | jq -r '.data.access_token')

# Create a regular user
curl -X POST http://localhost:8080/api/v1/users \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "email": "user@example.com",
    "password": "password",
    "firstName": "Demo",
    "lastName": "User",
    "role": "user",
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }'

# Create a viewer user
curl -X POST http://localhost:8080/api/v1/users \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "email": "viewer@example.com",
    "password": "password",
    "firstName": "Demo",
    "lastName": "Viewer",
    "role": "viewer",
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

### Via Database (for development)
```sql
-- Connect to PostgreSQL
-- psql -h localhost -p 5432 -U adx_user -d adx_core

-- Insert additional users
INSERT INTO users (tenant_id, email, password_hash, profile, is_active) VALUES 
    ('550e8400-e29b-41d4-a716-446655440000', 'user@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcSAg/9qK', '{"firstName":"Demo","lastName":"User","role":"user"}', true),
    ('550e8400-e29b-41d4-a716-446655440000', 'viewer@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcSAg/9qK', '{"firstName":"Demo","lastName":"Viewer","role":"viewer"}', true);
```

## 🎭 User Roles & Permissions

### Admin User (`admin@example.com`)
- **Full system access**
- Can manage users, tenants, workflows
- Access to all API endpoints
- Can view system metrics and logs

### Regular User (`user@example.com` - if created)
- **Standard user access**
- Can manage own files and workflows
- Limited administrative functions
- Cannot manage other users

### Viewer (`viewer@example.com` - if created)
- **Read-only access**
- Can view files and workflows
- Cannot create, edit, or delete resources
- Limited to viewing permissions

## 🚀 Quick Demo Flow

### 1. Start Services
```bash
./quick-dev.sh
```

### 2. Test API Authentication
```bash
./test-api.sh
```

### 3. Access Frontend
1. Open browser to http://localhost:1420
2. Login with `admin@example.com` / `password`
3. Explore the dashboard and features

### 4. Test Different Features
- **Dashboard**: Overview of system stats
- **Users**: User management (admin only)
- **Files**: File upload and management
- **Workflows**: Business process automation
- **Settings**: System configuration

## 🔍 Troubleshooting Login Issues

### Frontend Login Problems
```bash
# Check if frontend is running
curl http://localhost:1420

# Check frontend logs
tail -f logs/frontend.log

# Restart frontend
pkill -f "npm run dev"
cd frontend && npm run dev &
```

### API Gateway Routing Issues
```bash
# Test API Gateway auth routing
./test-gateway-auth.sh

# Check if API Gateway is running
curl http://localhost:8080/health

# Restart API Gateway
pkill -f "api-gateway"
cd adx-core && cargo run -p api-gateway &
```

### API Authentication Problems
```bash
# Test auth service directly
curl http://localhost:8081/health

# Check auth service logs
tail -f logs/auth-service.log

# Test database connection
docker exec docker-postgres-1 psql -U adx_user -d adx_core -c "SELECT * FROM users;"
```

### Database Issues
```bash
# Check if PostgreSQL is running
docker exec docker-postgres-1 pg_isready -U adx_user -d adx_core

# View database logs
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml logs postgres

# Reset database (WARNING: This will delete all data)
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down -v
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d
```

## 📱 Multi-Platform Access

### Web Browser
- **URL**: http://localhost:1420
- **Supported**: Chrome, Firefox, Safari, Edge

### Desktop App (Tauri - if built)
```bash
cd frontend
npm run tauri:dev
```

### Mobile (Tauri - if configured)
```bash
cd frontend
npm run dev:mobile
```

---

**Quick Reference:**
- **Frontend**: http://localhost:1420
- **API Gateway**: http://localhost:8080
- **Admin Email**: admin@example.com
- **Password**: password
- **Tenant ID**: 550e8400-e29b-41d4-a716-446655440000