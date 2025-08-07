# Client Management Plugin - Base Plugin Specification

## Overview
The Client Management Plugin is a **first-party base plugin** that demonstrates comprehensive plugin development patterns while providing essential client and customer management functionality.

## Plugin Metadata
```rust
impl AdxPlugin for ClientManagementPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "adx-core/client-management".to_string(),
            name: "Client & Customer Management".to_string(),
            version: "1.0.0".to_string(),
            description: "Manage external clients and customers with branded portals".to_string(),
            author: "ADX CORE Team".to_string(),
            category: PluginCategory::Business,
            is_first_party: true,
            price: None, // Included with platform
            permissions: vec![
                Permission::ClientManagement,
                Permission::PortalAccess,
                Permission::FileSharing,
                Permission::ProjectManagement,
            ],
            dependencies: vec![], // No external dependencies
            temporal_workflows: vec![
                "client_onboarding_workflow".to_string(),
                "client_portal_setup_workflow".to_string(),
                "client_project_workflow".to_string(),
            ],
        }
    }
}
```

## Temporal Workflows (Plugin Examples)

### 1. Client Onboarding Workflow
```rust
#[workflow]
pub async fn client_onboarding_workflow(
    client_data: ClientOnboardingData,
) -> WorkflowResult<OnboardedClient> {
    // Step 1: Validate client data
    validate_client_data_activity(client_data.clone()).await?;
    
    // Step 2: Create client record
    let client = create_client_record_activity(client_data.clone()).await?;
    
    // Step 3: Set up client portal (if requested)
    let portal = if client_data.setup_portal {
        Some(setup_client_portal_activity(
            client.id,
            client_data.portal_config,
        ).await?)
    } else {
        None
    };
    
    // Step 4: Create initial project (if provided)
    let project = if let Some(project_data) = client_data.initial_project {
        Some(create_client_project_activity(
            client.id,
            project_data,
        ).await?)
    } else {
        None
    };
    
    // Step 5: Send welcome email with portal access
    send_client_welcome_email_activity(
        client.email.clone(),
        client.clone(),
        portal.clone(),
    ).await?;
    
    // Step 6: Notify team of new client
    notify_team_new_client_activity(
        client_data.assigned_team_id,
        client.clone(),
    ).await?;
    
    Ok(OnboardedClient {
        client,
        portal,
        project,
        onboarded_at: temporal_sdk::now(),
    })
}
```

### 2. Client Portal Setup Workflow
```rust
#[workflow]
pub async fn client_portal_setup_workflow(
    portal_request: ClientPortalRequest,
) -> WorkflowResult<ClientPortal> {
    // Step 1: Validate portal configuration
    validate_portal_config_activity(portal_request.config.clone()).await?;
    
    // Step 2: Create portal subdomain (if custom domain not provided)
    let domain_config = if let Some(custom_domain) = portal_request.custom_domain {
        setup_custom_domain_activity(
            portal_request.client_id,
            custom_domain,
        ).await?
    } else {
        create_subdomain_activity(
            portal_request.client_id,
            portal_request.subdomain_prefix,
        ).await?
    };
    
    // Step 3: Apply branding configuration
    let branding = apply_portal_branding_activity(
        portal_request.client_id,
        portal_request.branding,
    ).await?;
    
    // Step 4: Set up file access permissions
    configure_file_permissions_activity(
        portal_request.client_id,
        portal_request.file_access_config,
    ).await?;
    
    // Step 5: Create portal user accounts
    let portal_users = create_portal_users_activity(
        portal_request.client_id,
        portal_request.portal_users,
    ).await?;
    
    // Step 6: Generate portal access credentials
    let access_credentials = generate_portal_credentials_activity(
        portal_request.client_id,
        portal_users.clone(),
    ).await?;
    
    // Step 7: Send portal access information
    send_portal_access_email_activity(
        portal_request.client_id,
        portal_users,
        access_credentials,
        domain_config.portal_url.clone(),
    ).await?;
    
    Ok(ClientPortal {
        client_id: portal_request.client_id,
        domain_config,
        branding,
        access_credentials,
        created_at: temporal_sdk::now(),
    })
}
```

## Plugin Activities (Examples)

### Client Management Activities
```rust
#[activity]
pub async fn create_client_record_activity(
    client_data: ClientOnboardingData,
) -> Result<Client, ActivityError> {
    let client_repo = get_client_repository().await?;
    
    let client = Client {
        id: ClientId::new(),
        tenant_id: client_data.tenant_id,
        name: client_data.name,
        email: client_data.email,
        phone: client_data.phone,
        company_name: client_data.company_name,
        address: client_data.address,
        client_type: client_data.client_type,
        status: ClientStatus::Active,
        assigned_team_id: client_data.assigned_team_id,
        assigned_user_id: client_data.assigned_user_id,
        custom_fields: client_data.custom_fields,
        portal_access: PortalAccess::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    client_repo.create_client(client.clone()).await?;
    
    Ok(client)
}

#[activity]
pub async fn setup_client_portal_activity(
    client_id: ClientId,
    portal_config: PortalConfiguration,
) -> Result<ClientPortal, ActivityError> {
    let portal_service = get_portal_service().await?;
    
    // Create portal with custom branding
    let portal = portal_service
        .create_portal(client_id, portal_config)
        .await?;
    
    Ok(portal)
}

#[activity]
pub async fn grant_file_access_activity(
    client_id: ClientId,
    file_id: FileId,
    permission: PortalPermission,
) -> Result<(), ActivityError> {
    let file_service = get_file_service().await?;
    
    file_service
        .grant_client_access(client_id, file_id, permission)
        .await?;
    
    Ok(())
}
```

## Database Schema (Plugin-Specific)

### Client Tables
```sql
-- Plugin-specific tables (created during plugin installation)
CREATE TABLE plugin_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(50),
    company_name VARCHAR(255),
    address JSONB,
    client_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    assigned_team_id UUID,
    assigned_user_id UUID,
    custom_fields JSONB DEFAULT '{}',
    portal_access JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, email),
    INDEX idx_plugin_clients_tenant (tenant_id),
    INDEX idx_plugin_clients_status (status),
    INDEX idx_plugin_clients_assigned (assigned_user_id)
);

CREATE TABLE plugin_client_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES plugin_clients(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    start_date DATE,
    end_date DATE,
    budget_amount DECIMAL(10,2),
    budget_currency VARCHAR(3) DEFAULT 'USD',
    project_manager_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    INDEX idx_plugin_client_projects_client (client_id),
    INDEX idx_plugin_client_projects_status (status)
);

CREATE TABLE plugin_client_file_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES plugin_clients(id),
    file_id UUID NOT NULL, -- References files table in core system
    permission VARCHAR(50) NOT NULL,
    granted_by UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    
    UNIQUE(client_id, file_id),
    INDEX idx_plugin_client_file_access_client (client_id),
    INDEX idx_plugin_client_file_access_file (file_id)
);
```

## UI Components (Plugin Frontend)

### React Components
```typescript
// Client dashboard component
export const ClientDashboard: React.FC = () => {
  const { clients, loading } = useClients();
  const { execute: createClient } = useTemporalWorkflow('client-onboarding');

  return (
    <div className="client-dashboard">
      <div className="dashboard-header">
        <h1>Client Management</h1>
        <button 
          onClick={() => setShowCreateModal(true)}
          className="btn-primary"
        >
          Add New Client
        </button>
      </div>
      
      <div className="client-grid">
        {clients.map(client => (
          <ClientCard 
            key={client.id} 
            client={client}
            onUpdate={handleClientUpdate}
          />
        ))}
      </div>
      
      {showCreateModal && (
        <CreateClientModal
          onSubmit={handleCreateClient}
          onClose={() => setShowCreateModal(false)}
        />
      )}
    </div>
  );
};

// Client portal builder component
export const ClientPortalBuilder: React.FC<{ clientId: string }> = ({ clientId }) => {
  const { execute: setupPortal } = useTemporalWorkflow('client-portal-setup');
  const [portalConfig, setPortalConfig] = useState<PortalConfiguration>();

  const handleSetupPortal = async () => {
    await setupPortal({
      client_id: clientId,
      config: portalConfig,
      branding: {
        logo_url: portalConfig.logo_url,
        primary_color: portalConfig.primary_color,
        custom_css: portalConfig.custom_css,
      },
    });
  };

  return (
    <div className="portal-builder">
      <h2>Client Portal Setup</h2>
      
      <div className="portal-config-form">
        <BrandingConfiguration 
          config={portalConfig}
          onChange={setPortalConfig}
        />
        
        <FileAccessConfiguration
          clientId={clientId}
          config={portalConfig}
          onChange={setPortalConfig}
        />
        
        <button 
          onClick={handleSetupPortal}
          className="btn-primary"
        >
          Setup Client Portal
        </button>
      </div>
    </div>
  );
};
```

## API Endpoints (Plugin Routes)

### Client Management API
```rust
// Plugin-specific API routes
pub fn client_management_routes() -> Router<AppState> {
    Router::new()
        .route("/clients", get(list_clients).post(create_client))
        .route("/clients/:id", get(get_client).put(update_client).delete(delete_client))
        .route("/clients/:id/portal", post(setup_client_portal))
        .route("/clients/:id/projects", get(list_client_projects).post(create_client_project))
        .route("/clients/:id/files", get(list_client_files).post(grant_file_access))
        .route("/clients/:id/portal/access", post(generate_portal_access))
}

// API handlers
pub async fn create_client(
    State(state): State<AppState>,
    Json(request): Json<CreateClientRequest>,
) -> Result<Json<Client>, ApiError> {
    // Start client onboarding workflow
    let workflow_id = format!("client-onboarding-{}", Uuid::new_v4());
    
    let client = state.temporal_client
        .start_workflow(
            workflow_id,
            client_onboarding_workflow,
            ClientOnboardingData {
                tenant_id: request.tenant_id,
                name: request.name,
                email: request.email,
                phone: request.phone,
                company_name: request.company_name,
                address: request.address,
                client_type: request.client_type,
                assigned_team_id: request.assigned_team_id,
                assigned_user_id: request.assigned_user_id,
                custom_fields: request.custom_fields,
                setup_portal: request.setup_portal,
                portal_config: request.portal_config,
                initial_project: request.initial_project,
            }
        )
        .await
        .map_err(ApiError::from)?;
    
    Ok(Json(client.client))
}
```

## Plugin Installation & Configuration

### Installation Workflow
```rust
impl AdxPlugin for ClientManagementPlugin {
    async fn activate(&self, context: &PluginContext) -> Result<(), PluginError> {
        // Register UI components
        context.register_ui_component("client-dashboard", ClientDashboard).await?;
        context.register_ui_component("client-list", ClientListView).await?;
        context.register_ui_component("client-portal-builder", ClientPortalBuilder).await?;
        
        // Register API endpoints
        context.register_routes("/api/plugins/client-management", client_management_routes()).await?;
        
        // Register Temporal workflows
        context.register_workflow("client_onboarding_workflow", client_onboarding_workflow).await?;
        context.register_workflow("client_portal_setup_workflow", client_portal_setup_workflow).await?;
        
        // Register database migrations
        context.run_migrations(self.get_migrations()).await?;
        
        // Register event handlers
        context.register_event_handler("file.uploaded", on_file_uploaded).await?;
        context.register_event_handler("user.created", on_user_created).await?;
        
        Ok(())
    }
    
    fn register_database_migrations(&self) -> Vec<Migration> {
        vec![
            Migration {
                version: "001_create_plugin_clients_table".to_string(),
                sql: include_str!("../migrations/001_create_plugin_clients_table.sql").to_string(),
            },
            Migration {
                version: "002_create_plugin_client_projects_table".to_string(),
                sql: include_str!("../migrations/002_create_plugin_client_projects_table.sql").to_string(),
            },
            Migration {
                version: "003_create_plugin_client_file_access_table".to_string(),
                sql: include_str!("../migrations/003_create_plugin_client_file_access_table.sql").to_string(),
            },
        ]
    }
}
```

## Key Benefits as Base Plugin

### 1. **Demonstrates Best Practices**
- **Temporal-First**: All complex operations use workflows
- **Multi-Tenant**: Proper tenant isolation and context
- **Security**: Input validation and permission checking
- **Testing**: Comprehensive test coverage including workflow replay

### 2. **Provides Real Business Value**
- **Client Management**: Essential for service businesses
- **Portal Creation**: Branded client access portals
- **File Sharing**: Secure client file access
- **Project Tracking**: Client project management

### 3. **Template for Developers**
- **Complete Example**: Full plugin implementation
- **Documentation**: Comprehensive development guide
- **Patterns**: Reusable code patterns and structures
- **Integration**: Shows how to integrate with core platform

This Client Management Plugin serves as both a **valuable business tool** and a **comprehensive development template** for the ADX CORE plugin ecosystem.