# ADX CORE Technology Stack

## Backend Architecture (Temporal-First)
- **Language**: Rust 1.88+ with async/await for high-performance concurrent processing
- **Framework**: Axum for HTTP services with middleware support
- **Workflow Engine**: Temporal.io as the PRIMARY orchestration mechanism for ALL multi-step operations
- **Architecture Pattern**: Workflow-driven microservices where complex operations are implemented as Temporal workflows
- **Service Pattern**: Dual-mode services providing both direct endpoints (simple operations) and Temporal activities (complex workflows)
- **Database**: PostgreSQL (primary) + Redis (caching/sessions) with database abstraction via Rust traits
- **Authentication**: JWT tokens with bcrypt password hashing, SSO (SAML, OAuth), Active Directory, MFA support
- **Multi-Tenancy**: Complete isolation at database, application, and workflow levels with tenant-aware contexts
- **Module System**: Hot-loadable modules with sandboxing, marketplace integration, and Temporal workflow support
- **Observability**: Structured logging (serde_json), OpenTelemetry tracing, Prometheus metrics, Temporal UI for workflow monitoring

## Frontend Architecture
- **Architecture**: Microservices with Module Federation (mirrors backend domain boundaries)
- **Shell Application**: React 18+ with TypeScript, Vite Module Federation
- **Micro-Frontends**: Domain-specific apps (Auth, Tenant, File, User, Workflow)
- **Framework Flexibility**: React, Vue, Svelte, Angular support per micro-frontend
- **Styling**: TailwindCSS with shared design system
- **Build Tool**: Vite with Module Federation for dynamic loading
- **Cross-Platform**: Tauri 2.0 for native desktop (Windows, macOS, Linux) and mobile (iOS, Android)
- **State Management**: Zustand, React Query (@tanstack/react-query)
- **UI Components**: Headless UI, Framer Motion, Lucide React icons
- **Internationalization**: react-i18next with browser language detection
- **Communication**: Event bus and BFF pattern for micro-frontend coordination

## Key Dependencies

### Rust Backend
```toml
tokio = "1.0"           # Async runtime
axum = "0.7"            # Web framework
sqlx = "0.8"            # Database toolkit with multi-tenant support
serde = "1.0"           # Serialization
uuid = "1.0"            # UUID generation
chrono = "0.4"          # Date/time handling
jsonwebtoken = "9.0"    # JWT handling
bcrypt = "0.15"         # Password hashing
redis = "0.24"          # Redis client
prometheus = "0.13"     # Metrics
tracing = "0.1"         # Logging/tracing
temporal-sdk = "0.1"    # Temporal Rust SDK
temporal-sdk-core = "0.1" # Core Temporal functionality
reqwest = "0.11"        # HTTP client for external services
tower = "0.4"           # Service abstractions
tower-http = "0.4"      # HTTP middleware
anyhow = "1.0"          # Error handling
thiserror = "1.0"       # Error derive macros
config = "0.13"         # Configuration management
clap = "4.0"            # CLI argument parsing
```

### Frontend (Shell + Micro-Frontends)
```json
"react": "^18.2.0"
"typescript": "^5.2.2"
"vite": "^5.0.0"
"@originjs/vite-plugin-federation": "^1.3.5"
"@tauri-apps/api": "^2.0.0-beta.0"
"tailwindcss": "^3.3.6"
"@tanstack/react-query": "^5.8.4"
"react-router-dom": "^6.20.1"
"react-i18next": "^13.5.0"
"zustand": "^4.4.7"
"@headlessui/react": "^1.7.17"
"framer-motion": "^10.16.4"
"lucide-react": "^0.292.0"
"@adx-core/design-system": "workspace:*"
"@adx-core/shared-context": "workspace:*"
"@adx-core/event-bus": "workspace:*"
"@adx-core/bff-client": "workspace:*"
"@adx-core/native-integration": "workspace:*"
```

### BFF Services (Optional Optimization Layer)
```json
// Node.js BFF with Temporal Client
"express": "^4.18.2"
"cors": "^2.8.5"
"@temporalio/client": "^1.8.0"
"@temporalio/worker": "^1.8.0"
"redis": "^4.6.0"

// Rust BFF with Temporal Client
"axum": "^0.7"
"reqwest": "^0.11"
"tower-http": "^0.4"
"temporal-sdk": "^0.1.0"
"redis": "^0.24"
```

### Temporal Workflow Dependencies
```toml
# Rust Backend Services
temporal-sdk = "0.1"        # Temporal Rust SDK
temporal-sdk-core = "0.1"   # Core Temporal functionality
uuid = "1.0"                # Workflow and activity IDs
serde = "1.0"               # Workflow state serialization
tokio = "1.0"               # Async runtime for workflows
```

## Development Environment

### Infrastructure
- **Containerization**: Docker + Docker Compose for development
- **Database**: PostgreSQL 14+ with automated migrations
- **Cache**: Redis 6+ for sessions and caching
- **Workflow Engine**: Temporal Server with UI at http://localhost:8088

### Service Ports

#### Backend Services (Temporal-Enabled)
- API Gateway: http://localhost:8080 (Workflow endpoints + Direct endpoints + Rate limiting)
- Auth Service: http://localhost:8081 (Activities + Direct endpoints + Workflow worker + SSO/MFA)
- User Service: http://localhost:8082 (Activities + Direct endpoints + Workflow worker + Multi-tenant)
- File Service: http://localhost:8083 (Activities + Direct endpoints + Workflow worker + Storage backends)
- Workflow Service: http://localhost:8084 (Cross-service workflow orchestration + AI integration)
- Tenant Service: http://localhost:8085 (Activities + Direct endpoints + Workflow worker + Isolation)
- Module Service: http://localhost:8086 (Module management + Sandbox + Marketplace + Hot-loading)
- License Service: http://localhost:8087 (License validation + Quota enforcement + Billing)

#### Temporal Infrastructure
- Temporal Server: http://localhost:7233
- Temporal UI: http://localhost:8088
- Temporal Database: PostgreSQL (shared with application data)

#### Frontend Micro-Services
- Shell Application: http://localhost:3000
- Auth Micro-App: http://localhost:3001
- Tenant Micro-App: http://localhost:3002
- File Micro-App: http://localhost:3003
- User Micro-App: http://localhost:3004
- Workflow Micro-App: http://localhost:3005
- Module Micro-App: http://localhost:3006

#### BFF Services (Optional Optimization Layer)
- Auth BFF: http://localhost:4001 (Temporal workflow client + caching)
- Tenant BFF: http://localhost:4002 (Temporal workflow client + caching)
- File BFF: http://localhost:4003 (Temporal workflow client + caching)
- User BFF: http://localhost:4004 (Temporal workflow client + caching)
- Workflow BFF: http://localhost:4005 (Temporal workflow client + caching)
- Module BFF: http://localhost:4006 (Temporal workflow client + caching)

#### Legacy Frontend (Deprecated)
- Monolithic Frontend: http://localhost:1420 (Vite dev server)

## Common Commands

### Development Setup
```bash
# Start entire development environment
./scripts/dev-start.sh

# Start from adx-core directory
cd adx-core && ./scripts/dev-start.sh

# Start infrastructure only
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d
```

### Backend Development (Temporal-First)
```bash
# Build all services with workflow support
cargo build --workspace

# Run services in different modes
cargo run --bin api-gateway                    # HTTP server + workflow client
cargo run --bin auth-service                   # HTTP server mode
cargo run --bin auth-service -- --mode worker  # Workflow worker mode
cargo run --bin user-service                   # HTTP server mode
cargo run --bin user-service -- --mode worker  # Workflow worker mode

# Start Temporal infrastructure
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up temporal -d

# Run workflow tests
cargo test --workspace --test workflow_tests
cargo test --test integration_workflows

# Test specific workflows
cargo test tenant_switch_workflow
cargo test file_upload_workflow
cargo test user_onboarding_workflow

# Check code
cargo clippy --workspace
cargo fmt --all
```

### Frontend Development

#### Micro-Frontend Development
```bash
# Start all micro-frontends
npm run dev:all

# Start individual micro-frontends
npm run dev:shell        # Shell application
npm run dev:auth         # Auth micro-frontend
npm run dev:tenant       # Tenant micro-frontend
npm run dev:file         # File micro-frontend
npm run dev:user         # User micro-frontend
npm run dev:workflow     # Workflow micro-frontend

# Start BFF services
npm run dev:bff          # All BFF services
npm run dev:auth-bff     # Auth BFF only
npm run dev:file-bff     # File BFF only

# Building
npm run build:all        # All micro-frontends
npm run build:shell      # Shell application
npm run build:auth       # Auth micro-frontend

# Cross-platform builds
npm run build:web        # Web builds
npm run build:desktop    # Desktop builds
npm run build:mobile     # Mobile builds

# Testing
npm run test:unit        # Unit tests
npm run test:integration # Integration tests
npm run test:e2e         # End-to-end tests
```

#### Legacy Frontend (Deprecated)
```bash
cd frontend

# Development servers
npm run dev              # Web development
npm run dev:desktop      # Desktop development
npm run dev:mobile       # Mobile development

# Tauri commands
npm run tauri:dev        # Desktop app development
npm run tauri:build      # Desktop app build
```

## Architecture Patterns
- **Temporal-First Backend**: ALL multi-step operations implemented as Temporal workflows for reliability and observability
- **Workflow-Driven Microservices**: Services provide both direct endpoints (simple operations) and Temporal activities (complex workflows)
- **Dual Integration Pattern**: Frontend can call API Gateway directly OR use BFF services as workflow clients
- **Frontend Microservices**: Domain-aligned micro-frontends with Module Federation
- **Multi-tenant**: Tenant isolation at database, application, and workflow levels
- **API Gateway**: Single entry point with workflow orchestration endpoints and direct service routing
- **BFF Pattern (Optional)**: Backend for Frontend services act as Temporal clients for data aggregation and caching
- **Workflow Orchestration**: Cross-service operations coordinated through Temporal workflows with automatic retry and compensation
- **Shared Libraries**: Common functionality in `adx-shared` crate including workflow utilities and activity interfaces
- **Database Abstraction**: Trait-based repository pattern for database independence
- **Team Autonomy**: Vertical slices owned by teams (backend service + workflows + frontend + optional BFF)