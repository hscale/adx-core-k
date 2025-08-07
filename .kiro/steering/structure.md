# ADX CORE Project Structure

## Root Directory Layout
```
adx-core/                    # Main project root
├── adx-core/               # Core platform services (Rust backend)
├── frontend/               # Legacy monolithic frontend (deprecated)
├── micro-frontends/        # Frontend microservices architecture
│   ├── shell/             # Shell application (Module Federation host)
│   ├── auth-micro-app/    # Authentication micro-frontend
│   ├── tenant-micro-app/  # Tenant management micro-frontend
│   ├── file-micro-app/    # File management micro-frontend
│   ├── user-micro-app/    # User management micro-frontend
│   ├── workflow-micro-app/# Workflow management micro-frontend
│   └── shared/            # Shared design system and utilities
├── bff-services/          # Backend for Frontend services
│   ├── auth-bff/          # Auth BFF (Node.js/TypeScript)
│   ├── tenant-bff/        # Tenant BFF (Node.js/TypeScript)
│   ├── file-bff/          # File BFF (Rust/Axum)
│   ├── user-bff/          # User BFF (Rust/Axum)
│   └── workflow-bff/      # Workflow BFF (Rust/Axum)
├── scripts/               # Root-level development scripts
├── rules/                 # Development rules and guidelines
├── .kiro/                 # Kiro IDE configuration and specs
└── src/                   # Legacy/additional source (minimal usage)
```

## Backend Structure (adx-core/) - Temporal-First
```
adx-core/
├── services/               # Temporal-enabled microservices
│   ├── shared/            # Shared libraries, workflow utilities, and activity traits
│   │   ├── workflows/     # Common workflow definitions and patterns
│   │   ├── activities/    # Shared activity trait definitions
│   │   └── temporal/      # Temporal client utilities and configuration
│   ├── api-gateway/       # API Gateway (port 8080) - Workflow endpoints + Direct routing
│   ├── auth-service/      # Auth service (port 8081) - Activities + Direct endpoints + Worker
│   ├── user-service/      # User service (port 8082) - Activities + Direct endpoints + Worker
│   ├── file-service/      # File service (port 8083) - Activities + Direct endpoints + Worker
│   ├── workflow-service/  # Cross-service workflow orchestration (port 8084)
│   └── tenant-service/    # Tenant service (port 8085) - Activities + Direct endpoints + Worker
├── infrastructure/        # Infrastructure as code
│   └── docker/           # Docker configurations
│       ├── docker-compose.dev.yml  # Includes Temporal server and workers
│       ├── init.sql
│       └── temporal-config/         # Temporal server configuration
├── scripts/              # Development scripts
│   ├── dev-start.sh      # Start all services + Temporal infrastructure
│   ├── start-workers.sh  # Start workflow workers only
│   └── test-workflows.sh # Run workflow integration tests
├── tests/                # Integration and workflow tests
│   ├── integration/      # Service integration tests
│   └── workflows/        # Workflow-specific tests
└── target/               # Rust build artifacts (gitignored)
```

## Frontend Microservices Structure (micro-frontends/)

### Shell Application (micro-frontends/shell/)
```
shell/
├── src/
│   ├── components/       # Shell-specific components
│   │   ├── MicroFrontendLoader.tsx
│   │   ├── NavigationShell.tsx
│   │   └── GlobalErrorBoundary.tsx
│   ├── providers/        # Global providers
│   │   ├── GlobalAuthProvider.tsx
│   │   ├── GlobalThemeProvider.tsx
│   │   └── EventBusProvider.tsx
│   ├── config/          # Configuration
│   │   ├── microFrontends.ts
│   │   └── i18n.ts
│   ├── types/           # Shell-specific types
│   └── utils/           # Shell utilities
├── vite.config.ts       # Module Federation host config
├── package.json         # Shell dependencies
└── tsconfig.json        # TypeScript configuration
```

### Micro-Frontend Structure Template
```
{micro-frontend-name}/
├── src/
│   ├── components/       # Domain-specific components
│   ├── pages/           # Domain pages
│   ├── hooks/           # Domain hooks
│   ├── services/        # Domain services (BFF integration)
│   ├── types/           # Domain types
│   ├── App.tsx          # Micro-frontend entry point
│   └── bootstrap.tsx    # Standalone development entry
├── vite.config.ts       # Module Federation remote config
├── package.json         # Micro-frontend dependencies
├── Dockerfile           # Container configuration
└── tsconfig.json        # TypeScript configuration
```

### Shared Design System (micro-frontends/shared/)
```
shared/
├── design-system/       # Shared UI components
│   ├── components/      # Button, Input, Modal, etc.
│   ├── tokens/          # Design tokens (colors, spacing)
│   ├── themes/          # Theme configurations
│   └── index.ts         # Design system exports
├── types/               # Common TypeScript types
├── utils/               # Shared utilities
├── hooks/               # Shared React hooks
└── package.json         # Shared package configuration
```

## BFF Services Structure (bff-services/)

### Node.js BFF Template
```
{service}-bff/
├── src/
│   ├── routes/          # API route handlers
│   ├── services/        # Business logic services
│   ├── middleware/      # Express middleware
│   ├── types/           # TypeScript types
│   ├── utils/           # Utility functions
│   └── server.ts        # Express server setup
├── package.json         # Dependencies
├── tsconfig.json        # TypeScript configuration
└── Dockerfile           # Container configuration
```

### Rust BFF Template
```
{service}-bff/
├── src/
│   ├── handlers/        # Request handlers
│   ├── services/        # Business logic
│   ├── types.rs         # Type definitions
│   ├── utils.rs         # Utility functions
│   └── main.rs          # Axum server setup
├── Cargo.toml           # Dependencies
└── Dockerfile           # Container configuration
```

## Legacy Frontend Structure (frontend/) - Deprecated
```
frontend/
├── src/
│   ├── components/       # Reusable UI components (being migrated)
│   ├── pages/           # Page components (being migrated)
│   ├── layouts/         # Layout components (being migrated)
│   ├── contexts/        # React contexts (being migrated)
│   ├── hooks/           # Custom React hooks (being migrated)
│   ├── services/        # API services (being migrated)
│   ├── types/           # TypeScript types (being migrated)
│   ├── utils/           # Utility functions (being migrated)
│   └── i18n/            # Internationalization (being migrated)
├── vite.config.ts       # Legacy Vite configuration
└── package.json         # Legacy dependencies
```

## Service Structure Pattern (Temporal-First)
Each microservice follows this dual-mode structure:
```
services/{service-name}/
├── Cargo.toml           # Service dependencies including Temporal SDK
└── src/
    ├── main.rs          # Service entry point with mode selection (server/worker)
    ├── server.rs        # HTTP server for direct endpoints
    ├── worker.rs        # Temporal workflow worker
    ├── service.rs       # Core business logic (shared by server and activities)
    ├── types.rs         # Service-specific types and workflow data models
    ├── activities.rs    # Temporal activity implementations
    ├── workflows.rs     # Service-specific workflow definitions
    ├── handlers.rs      # HTTP request handlers for direct endpoints
    └── compensation.rs  # Compensation logic for workflow rollbacks
```

## Cross-Service Workflow Structure
```
services/workflow-service/
├── Cargo.toml           # Dependencies for cross-service orchestration
└── src/
    ├── main.rs          # Workflow service entry point
    ├── workflows/       # Cross-service workflow definitions
    │   ├── tenant_switch.rs      # Tenant switching workflow
    │   ├── user_onboarding.rs    # User onboarding workflow
    │   ├── file_processing.rs    # File upload/processing workflow
    │   └── mod.rs               # Workflow module exports
    ├── activities/      # Cross-service activity orchestration
    ├── compensation/    # Cross-service compensation logic
    └── types.rs         # Cross-service workflow data models
```

## Shared Library Structure (adx-core/services/shared/) - Temporal-First
```
shared/src/
├── lib.rs               # Library entry point
├── database.rs          # Database abstractions and connections
├── temporal/            # Temporal utilities and patterns
│   ├── mod.rs          # Temporal module exports
│   ├── client.rs       # Temporal client configuration and utilities
│   ├── worker.rs       # Workflow worker setup and configuration
│   ├── activities.rs   # Common activity trait definitions
│   ├── workflows.rs    # Common workflow patterns and utilities
│   ├── errors.rs       # Workflow and activity error types
│   └── compensation.rs # Compensation pattern utilities
├── types.rs             # Common types (UserId, TenantId, WorkflowContext, etc.)
├── events.rs            # Event definitions and handling
├── auth.rs              # Authentication context for workflows
├── tenant.rs            # Tenant isolation utilities for workflows
└── observability.rs     # Logging, tracing, and metrics (including workflow tracing)
```

## Configuration and Rules (rules/)
```
rules/
├── core/                # Core development rules
│   ├── architecture.md  # Architecture guidelines
│   ├── coding-standards.md
│   ├── performance.md
│   └── security.md
├── services/            # Service-specific rules
├── workflows/           # Workflow patterns
├── hooks/               # Git hooks and automation
└── specs/               # Specification templates
```

## Kiro Configuration (.kiro/)
```
.kiro/
├── steering/            # AI assistant steering rules
│   ├── product.md       # Product overview
│   ├── tech.md          # Technology stack
│   ├── structure.md     # Project structure (this file)
│   └── frontend-microservices.md # Frontend microservices architecture
└── specs/               # Development specifications
    ├── adx-core/        # Core platform specs
    └── frontend-microservices/ # Frontend microservices specs
        ├── requirements.md
        ├── design.md
        └── tasks.md
```

## Naming Conventions

### Rust Services
- **Crate names**: `adx-{service-name}` (e.g., `adx-shared`, `adx-auth`)
- **Binary names**: `{service-name}` (e.g., `api-gateway`, `auth-service`)
- **Module files**: Snake case (e.g., `user_service.rs`, `auth_middleware.rs`)

### Frontend Microservices
- **Micro-Frontend Names**: kebab-case (e.g., `auth-micro-app`, `file-micro-app`)
- **Components**: PascalCase (e.g., `LoginPage.tsx`, `DashboardLayout.tsx`)
- **Hooks**: camelCase with `use` prefix (e.g., `useAuth.ts`, `usePlatform.ts`)
- **Services**: camelCase (e.g., `api.ts`, `auth.ts`)
- **Types**: PascalCase interfaces (e.g., `User`, `AuthContext`)
- **BFF Services**: kebab-case with `-bff` suffix (e.g., `auth-bff`, `file-bff`)

### Directories
- **Backend**: kebab-case (e.g., `auth-service`, `api-gateway`)
- **Frontend**: camelCase within micro-frontends (e.g., `components`, `contexts`, `services`)
- **Micro-Frontends**: kebab-case (e.g., `auth-micro-app`, `tenant-micro-app`)

## File Organization Principles

1. **Service Independence**: Each service is self-contained with its own dependencies
2. **Shared Code**: Common functionality goes in `adx-shared` crate
3. **Clear Separation**: Frontend and backend are completely separate with API boundaries
4. **Configuration Co-location**: Service-specific configs stay with the service
5. **Test Proximity**: Tests are organized near the code they test
6. **Documentation**: README files at each major directory level

## Import/Module Patterns

### Rust
```rust
// External crates first
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};

// Workspace crates
use adx_shared::{RequestContext, TenantId};

// Local modules
use crate::service::UserService;
```

### TypeScript (Micro-Frontends)
```typescript
// External libraries
import React from 'react'
import { useQuery } from '@tanstack/react-query'

// Shared design system
import { Button } from '@adx-core/design-system'
import type { User } from '@adx-core/shared-types'

// Internal modules with @ alias
import { useAuth } from '@/hooks/useAuth'
import { AuthService } from '@/services/auth'
import type { LoginRequest } from '@/types'
```

### TypeScript (Shell Application)
```typescript
// External libraries
import React from 'react'
import { BrowserRouter } from 'react-router-dom'

// Shared design system
import { ThemeProvider } from '@adx-core/design-system'

// Shell-specific modules
import { MicroFrontendLoader } from '@/components/MicroFrontendLoader'
import { EventBusProvider } from '@/providers/EventBusProvider'
import type { MicroFrontendConfig } from '@/types'
```