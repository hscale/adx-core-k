# White-Label & Custom Domain Service - Temporal-First Design

## Overview

The White-Label Service uses Temporal workflows for all complex branding and domain operations, ensuring reliable customization and domain management for enterprise customers.

```
┌─────────────────────────────────────────────────────────────┐
│              Temporal-First White-Label Service            │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Domain         │   Branding      │    Multi-Level          │
│  Workflows      │   Workflows     │    Reseller             │
│                 │                 │                         │
│ • Domain Setup  │ • Brand Config  │ • Nested Tenants       │
│ • SSL Provision │ • Asset Upload  │ • Revenue Sharing       │
│ • DNS Verify    │ • Theme Apply   │ • Support Routing       │
│ • Certificate   │ • Email Template│ • Analytics Hierarchy   │
└─────────────────┴─────────────────┴─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Temporal    │    │   PostgreSQL  │    │      CDN      │
│   Workflows   │    │   (Config)    │    │  (Assets)     │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Core Temporal Workflows

### 1. Custom Domain Setup Workflow
```rust
#[workflow]
pub async fn custom_domain_setup_workflow(
    domain_request: CustomDomainRequest,
) -> WorkflowResult<DomainConfiguration> {
    // Step 1: Validate domain ownership
    let ownership_token = generate_domain_verification_token_activity(
        domain_request.domain.clone(),
        domain_request.tenant_id,
    ).await?;
    
    // Step 2: Wait for DNS verification (with timeout)
    let verified = temporal_sdk::select! {
        _ = wait_for_domain_verification_signal(ownership_token.clone()) => true,
        _ = temporal_sdk::sleep(Duration::from_hours(24)) => false,
    };
    
    if !verified {
        return Err(WorkflowError::DomainVerificationTimeout);
    }
    
    // Step 3: Configure DNS routing
    let dns_config = configure_dns_routing_activity(
        domain_request.domain.clone(),
        domain_request.tenant_id,
    ).await?;
    
    // Step 4: Provision SSL certificate
    let ssl_cert = provision_ssl_certificate_activity(
        domain_request.domain.clone(),
    ).await?;
    
    // Step 5: Update load balancer configuration
    update_load_balancer_activity(
        domain_request.domain.clone(),
        domain_request.tenant_id,
        ssl_cert.clone(),
    ).await?;
    
    // Step 6: Test domain routing
    let routing_test = test_domain_routing_activity(
        domain_request.domain.clone(),
    ).await?;
    
    if !routing_test.success {
        // Rollback configuration
        rollback_domain_setup_activity(
            domain_request.domain.clone(),
            dns_config,
            ssl_cert,
        ).await?;
        return Err(WorkflowError::DomainRoutingFailed);
    }
    
    // Step 7: Activate domain
    let domain_config = activate_custom_domain_activity(
        domain_request.domain,
        domain_request.tenant_id,
        ssl_cert,
    ).await?;
    
    Ok(domain_config)
}
```

### 2. White-Label Branding Workflow
```rust
#[workflow]
pub async fn white_label_branding_workflow(
    branding_request: BrandingRequest,
) -> WorkflowResult<BrandingConfiguration> {
    // Step 1: Validate branding assets
    let validation_result = validate_branding_assets_activity(
        branding_request.assets.clone(),
    ).await?;
    
    if !validation_result.is_valid {
        return Err(WorkflowError::InvalidBrandingAssets(validation_result.errors));
    }
    
    // Step 2: Upload assets to CDN in parallel
    let (logo_upload, theme_upload, email_templates) = temporal_sdk::join!(
        upload_logo_assets_activity(branding_request.logos),
        upload_theme_assets_activity(branding_request.theme),
        upload_email_templates_activity(branding_request.email_templates)
    );
    
    // Step 3: Generate custom CSS
    let custom_css = generate_custom_css_activity(
        branding_request.theme.clone(),
        theme_upload?,
    ).await?;
    
    // Step 4: Update frontend configuration
    let frontend_config = update_frontend_branding_activity(
        branding_request.tenant_id,
        BrandingAssets {
            logos: logo_upload?,
            theme: theme_upload?,
            custom_css,
            email_templates: email_templates?,
        },
    ).await?;
    
    // Step 5: Update mobile app configuration
    let mobile_config = update_mobile_branding_activity(
        branding_request.tenant_id,
        branding_request.mobile_assets.clone(),
    ).await?;
    
    // Step 6: Test branding application
    let branding_test = test_branding_application_activity(
        branding_request.tenant_id,
        frontend_config.clone(),
    ).await?;
    
    if !branding_test.success {
        // Rollback branding changes
        rollback_branding_activity(
            branding_request.tenant_id,
            frontend_config,
            mobile_config,
        ).await?;
        return Err(WorkflowError::BrandingApplicationFailed);
    }
    
    // Step 7: Activate branding
    let branding_config = activate_branding_activity(
        branding_request.tenant_id,
        frontend_config,
        mobile_config,
    ).await?;
    
    Ok(branding_config)
}
```

### 3. Multi-Level Reseller Setup Workflow
```rust
#[workflow]
pub async fn reseller_setup_workflow(
    reseller_request: ResellerSetupRequest,
) -> WorkflowResult<ResellerConfiguration> {
    // Step 1: Validate reseller hierarchy
    validate_reseller_hierarchy_activity(
        reseller_request.parent_tenant_id,
        reseller_request.reseller_level,
    ).await?;
    
    // Step 2: Create reseller tenant with inheritance
    let reseller_tenant = create_reseller_tenant_activity(
        reseller_request.clone(),
    ).await?;
    
    // Step 3: Set up branding inheritance
    let inherited_branding = setup_branding_inheritance_activity(
        reseller_request.parent_tenant_id,
        reseller_tenant.id,
        reseller_request.branding_overrides,
    ).await?;
    
    // Step 4: Configure revenue sharing
    let revenue_config = configure_revenue_sharing_activity(
        reseller_request.parent_tenant_id,
        reseller_tenant.id,
        reseller_request.revenue_share_percentage,
    ).await?;
    
    // Step 5: Set up support routing
    let support_config = configure_support_routing_activity(
        reseller_tenant.id,
        reseller_request.support_configuration,
    ).await?;
    
    // Step 6: Create reseller admin user
    let admin_user = create_reseller_admin_activity(
        reseller_tenant.id,
        reseller_request.admin_user,
    ).await?;
    
    // Step 7: Send reseller welcome package
    send_reseller_welcome_activity(
        admin_user.email,
        reseller_tenant.clone(),
        inherited_branding.clone(),
    ).await?;
    
    Ok(ResellerConfiguration {
        tenant: reseller_tenant,
        branding: inherited_branding,
        revenue_config,
        support_config,
        admin_user,
    })
}
```

### 4. SSL Certificate Management Workflow
```rust
#[workflow]
pub async fn ssl_certificate_management_workflow(
    cert_request: SSLCertificateRequest,
) -> WorkflowResult<SSLCertificate> {
    // Step 1: Check existing certificate
    let existing_cert = check_existing_certificate_activity(
        cert_request.domain.clone(),
    ).await?;
    
    // Step 2: Determine if renewal is needed
    let needs_renewal = if let Some(cert) = existing_cert {
        cert.expires_at < Utc::now() + Duration::from_days(30)
    } else {
        true
    };
    
    if !needs_renewal {
        return Ok(existing_cert.unwrap());
    }
    
    // Step 3: Request new certificate from Let's Encrypt
    let cert_challenge = request_ssl_certificate_activity(
        cert_request.domain.clone(),
    ).await?;
    
    // Step 4: Set up DNS challenge
    setup_dns_challenge_activity(
        cert_request.domain.clone(),
        cert_challenge.clone(),
    ).await?;
    
    // Step 5: Wait for certificate issuance
    let certificate = temporal_sdk::select! {
        cert = wait_for_certificate_issuance_activity(cert_challenge.clone()) => cert?,
        _ = temporal_sdk::sleep(Duration::from_minutes(10)) => {
            return Err(WorkflowError::CertificateIssuanceTimeout);
        }
    };
    
    // Step 6: Install certificate
    install_ssl_certificate_activity(
        cert_request.domain.clone(),
        certificate.clone(),
    ).await?;
    
    // Step 7: Schedule renewal
    schedule_certificate_renewal_activity(
        cert_request.domain,
        certificate.expires_at - Duration::from_days(30),
    ).await?;
    
    Ok(certificate)
}
```

## White-Label Activities

### Domain Management Activities
```rust
#[activity]
pub async fn generate_domain_verification_token_activity(
    domain: String,
    tenant_id: TenantId,
) -> Result<DomainVerificationToken, ActivityError> {
    let token = DomainVerificationToken {
        domain: domain.clone(),
        tenant_id,
        token: generate_secure_token(),
        expires_at: Utc::now() + Duration::from_hours(24),
        verification_type: VerificationType::TxtRecord,
    };
    
    // Store token for verification
    let domain_service = get_domain_service().await?;
    domain_service.store_verification_token(&token).await?;
    
    Ok(token)
}

#[activity]
pub async fn provision_ssl_certificate_activity(
    domain: String,
) -> Result<SSLCertificate, ActivityError> {
    let acme_client = get_acme_client().await?;
    
    // Request certificate from Let's Encrypt
    let certificate = acme_client
        .request_certificate(&domain)
        .await
        .map_err(|e| ActivityError::SSLProvisioningFailed(e.to_string()))?;
    
    Ok(certificate)
}

#[activity]
pub async fn configure_dns_routing_activity(
    domain: String,
    tenant_id: TenantId,
) -> Result<DNSConfiguration, ActivityError> {
    let dns_service = get_dns_service().await?;
    
    let dns_config = DNSConfiguration {
        domain: domain.clone(),
        tenant_id,
        cname_record: format!("{}.adxcore.com", tenant_id),
        a_records: vec!["1.2.3.4".to_string()], // Load balancer IPs
        txt_records: vec![format!("adx-tenant-id={}", tenant_id)],
    };
    
    dns_service.configure_routing(&dns_config).await?;
    
    Ok(dns_config)
}
```

### Branding Activities
```rust
#[activity]
pub async fn validate_branding_assets_activity(
    assets: BrandingAssets,
) -> Result<ValidationResult, ActivityError> {
    let mut errors = Vec::new();
    
    // Validate logo dimensions and format
    if let Some(logo) = &assets.logo {
        if logo.width > 500 || logo.height > 200 {
            errors.push("Logo dimensions exceed maximum (500x200)".to_string());
        }
        if !["png", "jpg", "svg"].contains(&logo.format.as_str()) {
            errors.push("Logo format must be PNG, JPG, or SVG".to_string());
        }
    }
    
    // Validate color scheme
    if let Some(colors) = &assets.colors {
        for color in &colors.palette {
            if !is_valid_hex_color(color) {
                errors.push(format!("Invalid color format: {}", color));
            }
        }
    }
    
    // Scan for malicious content
    let security_scan = scan_assets_for_malware(&assets).await?;
    if !security_scan.is_clean {
        errors.extend(security_scan.threats);
    }
    
    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    })
}

#[activity]
pub async fn upload_logo_assets_activity(
    logos: LogoAssets,
) -> Result<UploadedAssets, ActivityError> {
    let cdn_service = get_cdn_service().await?;
    
    let mut uploaded_assets = UploadedAssets::new();
    
    // Upload primary logo
    if let Some(primary_logo) = logos.primary {
        let url = cdn_service
            .upload_asset("logos/primary", &primary_logo)
            .await?;
        uploaded_assets.primary_logo_url = Some(url);
    }
    
    // Upload favicon
    if let Some(favicon) = logos.favicon {
        let url = cdn_service
            .upload_asset("logos/favicon", &favicon)
            .await?;
        uploaded_assets.favicon_url = Some(url);
    }
    
    // Upload mobile app icons
    for (size, icon) in logos.mobile_icons {
        let url = cdn_service
            .upload_asset(&format!("logos/mobile/{}", size), &icon)
            .await?;
        uploaded_assets.mobile_icons.insert(size, url);
    }
    
    Ok(uploaded_assets)
}

#[activity]
pub async fn generate_custom_css_activity(
    theme: ThemeConfiguration,
    uploaded_assets: UploadedAssets,
) -> Result<String, ActivityError> {
    let css_generator = get_css_generator().await?;
    
    let custom_css = css_generator
        .generate_theme_css(&theme, &uploaded_assets)
        .await?;
    
    // Minify CSS for production
    let minified_css = minify_css(&custom_css)?;
    
    Ok(minified_css)
}
```

## Database Schema

### White-Label Configuration
```sql
CREATE TABLE white_label_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Domain configuration
    custom_domain VARCHAR(255),
    domain_verified BOOLEAN DEFAULT FALSE,
    ssl_certificate_id VARCHAR(255),
    
    -- Branding configuration
    logo_url VARCHAR(500),
    favicon_url VARCHAR(500),
    primary_color VARCHAR(7),
    secondary_color VARCHAR(7),
    custom_css TEXT,
    
    -- Email branding
    email_logo_url VARCHAR(500),
    email_footer_text TEXT,
    email_templates JSONB DEFAULT '{}',
    
    -- Mobile app branding
    mobile_app_icon_url VARCHAR(500),
    mobile_splash_screen_url VARCHAR(500),
    
    -- Reseller configuration
    parent_tenant_id UUID REFERENCES tenants(id),
    reseller_level INTEGER DEFAULT 0,
    revenue_share_percentage DECIMAL(5,2),
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id),
    INDEX idx_white_label_domain (custom_domain),
    INDEX idx_white_label_parent (parent_tenant_id)
);

CREATE TABLE domain_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    verification_token VARCHAR(255) NOT NULL,
    verification_type VARCHAR(50) NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    INDEX idx_domain_verification_token (verification_token),
    INDEX idx_domain_verification_domain (domain)
);

CREATE TABLE ssl_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain VARCHAR(255) NOT NULL,
    certificate_pem TEXT NOT NULL,
    private_key_pem TEXT NOT NULL,
    chain_pem TEXT,
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    auto_renew BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(domain),
    INDEX idx_ssl_cert_expiry (expires_at)
);
```

## API Endpoints

### Domain Management
- `POST /api/white-label/domains` - Start custom domain setup workflow
- `GET /api/white-label/domains/{domain}/verification` - Get domain verification status
- `POST /api/white-label/domains/{domain}/verify` - Verify domain ownership
- `DELETE /api/white-label/domains/{domain}` - Remove custom domain

### Branding Management
- `POST /api/white-label/branding` - Start branding update workflow
- `GET /api/white-label/branding` - Get current branding configuration
- `POST /api/white-label/branding/assets` - Upload branding assets
- `GET /api/white-label/branding/preview` - Preview branding changes

### Reseller Management
- `POST /api/white-label/resellers` - Create reseller account
- `GET /api/white-label/resellers` - List reseller accounts
- `PUT /api/white-label/resellers/{id}/revenue-share` - Update revenue sharing
- `GET /api/white-label/resellers/{id}/analytics` - Get reseller analytics

## Key Benefits of Temporal-First White-Label

### 1. Reliable Domain Management
- **Domain verification** with automatic retry and timeout handling
- **SSL certificate provisioning** with automatic renewal workflows
- **DNS configuration** with rollback on failure
- **Load balancer updates** with health checking

### 2. Consistent Branding Application
- **Asset validation** and security scanning before upload
- **Parallel asset processing** for faster deployment
- **Rollback capability** for failed branding updates
- **Multi-platform consistency** across web, mobile, and email

### 3. Scalable Reseller Hierarchy
- **Nested tenant management** with inheritance rules
- **Revenue sharing automation** with accurate tracking
- **Support routing** based on reseller hierarchy
- **Analytics aggregation** across reseller levels

### 4. Enterprise-Grade Reliability
- **Workflow visibility** in Temporal UI for debugging
- **Automatic error recovery** for all white-label operations
- **State persistence** across service restarts
- **Comprehensive audit trail** for compliance

This Temporal-first approach makes white-label operations **reliable, scalable, and maintainable** while providing enterprise customers with the customization capabilities they need.