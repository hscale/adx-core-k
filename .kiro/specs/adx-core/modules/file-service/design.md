# File Service - Technical Design

## Architecture Overview

The File Service provides a simple, secure abstraction over multiple storage providers with comprehensive file management capabilities.

```
┌─────────────────────────────────────────────────────────────┐
│                    File Service                             │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Storage        │   File          │    Sharing &            │
│  Abstraction    │   Management    │    Permissions          │
│                 │                 │                         │
│ • Multi-Provider│ • Upload/Download│ • Link Sharing         │
│ • Local/S3/GCS  │ • Versioning    │ • User Permissions     │
│ • Failover      │ • Metadata      │ • External Sharing     │
│ • Migration     │ • Thumbnails    │ • Access Logging       │
└─────────────────┴─────────────────┴─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Storage     │    │   PostgreSQL  │    │     Redis     │
│   Providers   │    │   (Metadata)  │    │   (Cache)     │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Core Components

### 1. Storage Provider Abstraction

**Storage Provider Trait**
```rust
#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<UploadResult, StorageError>;
    async fn download(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    async fn exists(&self, key: &str) -> Result<bool, StorageError>;
    async fn get_metadata(&self, key: &str) -> Result<StorageMetadata, StorageError>;
    async fn list(&self, prefix: &str) -> Result<Vec<StorageObject>, StorageError>;
}
```

**Supported Providers**
- **Local FileSystem** - For development and small deployments
- **AWS S3** - Primary cloud storage option
- **Google Cloud Storage** - Alternative cloud option
- **Azure Blob Storage** - Enterprise cloud option

### 2. File Management Service

**File Operations**
- Multipart upload for large files (>100MB)
- Automatic virus scanning on upload
- File type validation and restrictions
- Thumbnail and preview generation
- File versioning and history

**Metadata Management**
- File information (name, size, type, checksum)
- Upload/modification timestamps
- User and tenant association
- Custom tags and descriptions
- Search indexing

### 3. File Sharing System

**Permission Levels**
- **View** - Can view file metadata and thumbnails
- **Download** - Can download file content
- **Edit** - Can upload new versions
- **Admin** - Can manage permissions and sharing

**Sharing Methods**
- **User Sharing** - Share with specific platform users
- **Team Sharing** - Share with team members
- **External Links** - Password-protected public links
- **Email Sharing** - Send secure links via email

## Database Schema

### Files Table
```sql
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    
    -- File information
    name VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    
    -- Storage information
    storage_provider VARCHAR(50) NOT NULL,
    storage_key VARCHAR(500) NOT NULL,
    storage_path VARCHAR(1000) NOT NULL,
    
    -- File properties
    visibility file_visibility NOT NULL DEFAULT 'private',
    tags TEXT[],
    description TEXT,
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_file_id UUID REFERENCES files(id),
    is_current_version BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Lifecycle
    expires_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT files_size_positive CHECK (size_bytes > 0),
    CONSTRAINT files_version_positive CHECK (version > 0),
    CONSTRAINT files_checksum_format CHECK (checksum ~ '^[a-f0-9]{64}$'),
    
    -- Indexes
    INDEX idx_files_tenant_owner (tenant_id, owner_id),
    INDEX idx_files_storage (storage_provider, storage_key),
    INDEX idx_files_parent (parent_file_id, is_current_version),
    INDEX idx_files_tags USING GIN(tags)
);
```

### File Shares Table
```sql
CREATE TABLE file_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    
    -- Sharing details
    shared_by UUID NOT NULL,
    shared_with UUID, -- NULL for public shares
    share_type share_type NOT NULL,
    permissions JSONB NOT NULL DEFAULT '{"read": true}',
    
    -- Link sharing
    share_token VARCHAR(64) UNIQUE,
    password_hash VARCHAR(255),
    
    -- Limits and expiration
    expires_at TIMESTAMPTZ,
    download_limit INTEGER,
    download_count INTEGER NOT NULL DEFAULT 0,
    
    -- Audit
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT file_shares_token_format CHECK (share_token ~ '^[a-zA-Z0-9_-]{64}$'),
    CONSTRAINT file_shares_download_limit_positive CHECK (download_limit IS NULL OR download_limit > 0),
    
    -- Indexes
    INDEX idx_file_shares_file (file_id),
    INDEX idx_file_shares_token (share_token),
    INDEX idx_file_shares_shared_with (shared_with)
);
```

## API Endpoints

### File Operations
- `POST /files/upload` - Upload file with multipart support
- `GET /files/{id}` - Get file metadata
- `GET /files/{id}/download` - Download file content
- `GET /files/{id}/preview` - Get file preview/thumbnail
- `PUT /files/{id}` - Update file metadata
- `DELETE /files/{id}` - Delete file
- `GET /files` - List files with filtering and pagination

### File Sharing
- `POST /files/{id}/share` - Create file share
- `GET /files/{id}/shares` - List file shares
- `PUT /files/shares/{share_id}` - Update share permissions
- `DELETE /files/shares/{share_id}` - Remove file share
- `GET /shared/{token}` - Access shared file by token

### File Management
- `POST /files/{id}/versions` - Upload new version
- `GET /files/{id}/versions` - List file versions
- `POST /files/{id}/restore/{version}` - Restore file version
- `POST /files/batch/delete` - Batch delete files
- `POST /files/batch/move` - Batch move files

## Storage Provider Implementations

### AWS S3 Provider
```rust
pub struct S3StorageProvider {
    client: S3Client,
    bucket: String,
    region: Region,
    encryption: Option<ServerSideEncryption>,
}

impl StorageProvider for S3StorageProvider {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<UploadResult, StorageError> {
        let request = PutObjectRequest {
            bucket: self.bucket.clone(),
            key: key.to_string(),
            body: Some(data.to_vec().into()),
            server_side_encryption: self.encryption.clone(),
            ..Default::default()
        };
        
        let result = self.client.put_object(request).await?;
        
        Ok(UploadResult {
            key: key.to_string(),
            etag: result.e_tag,
            size: data.len() as u64,
            uploaded_at: Utc::now(),
        })
    }
    
    async fn download(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            key: key.to_string(),
            ..Default::default()
        };
        
        let result = self.client.get_object(request).await?;
        let body = result.body.ok_or(StorageError::EmptyResponse)?;
        
        let mut data = Vec::new();
        body.into_async_read().read_to_end(&mut data).await?;
        
        Ok(data)
    }
}
```

### Local FileSystem Provider
```rust
pub struct LocalStorageProvider {
    base_path: PathBuf,
    permissions: u32,
}

impl StorageProvider for LocalStorageProvider {
    async fn upload(&self, key: &str, data: &[u8]) -> Result<UploadResult, StorageError> {
        let file_path = self.base_path.join(key);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Write file
        tokio::fs::write(&file_path, data).await?;
        
        // Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(self.permissions);
            tokio::fs::set_permissions(&file_path, permissions).await?;
        }
        
        Ok(UploadResult {
            key: key.to_string(),
            etag: None,
            size: data.len() as u64,
            uploaded_at: Utc::now(),
        })
    }
    
    async fn download(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let file_path = self.base_path.join(key);
        let data = tokio::fs::read(file_path).await?;
        Ok(data)
    }
}
```

## Security Considerations

### File Upload Security
- Virus scanning with ClamAV or cloud services
- File type validation based on content, not extension
- Size limits enforced at multiple levels
- Malicious file detection and quarantine

### Access Control
- All file access requires authentication
- Permission checks at API and storage level
- Audit logging for all file operations
- Secure token generation for sharing

### Data Protection
- Encryption at rest for all storage providers
- Encryption in transit with TLS 1.3
- Secure key management for encryption
- Data residency compliance options

## Performance Optimizations

### Caching Strategy
- File metadata cached in Redis (5-minute TTL)
- Thumbnail cache with CDN integration
- Download URL caching for cloud providers
- Permission cache for frequent access checks

### Upload Optimization
- Multipart upload for files >100MB
- Parallel chunk uploads
- Resume capability for interrupted uploads
- Client-side compression for text files

### Download Optimization
- Direct download URLs from cloud providers
- CDN integration for static content
- Range request support for large files
- Compression for text-based files

## Monitoring and Observability

### Metrics
- Upload/download success rates and performance
- Storage usage per tenant and user
- File sharing activity and access patterns
- Storage provider performance and costs
- Virus scan results and security events

### Logging
- All file operations with user attribution
- Storage provider API calls and responses
- Security events and access violations
- Performance metrics and slow operations
- Cost tracking and usage analytics

### Alerts
- Storage quota approaching limits
- High error rates from storage providers
- Security threats detected in uploads
- Performance degradation alerts
- Cost budget overruns

## Testing Strategy

### Unit Tests
- Storage provider implementations
- File metadata operations
- Permission checking logic
- Sharing token generation and validation
- File processing and validation

### Integration Tests
- End-to-end file upload/download flows
- Storage provider failover testing
- File sharing and permission enforcement
- Virus scanning integration
- Performance benchmarking

### Security Tests
- File upload security validation
- Access control enforcement
- Malicious file detection
- Token security and expiration
- Data encryption verification

## Deployment Considerations

### Storage Configuration
- Environment-specific storage providers
- Encryption key management
- Backup and replication strategies
- Cost optimization settings

### Scaling Strategy
- Horizontal scaling for file processing
- CDN integration for global distribution
- Storage provider load balancing
- Auto-scaling based on usage patterns

### Disaster Recovery
- Cross-region storage replication
- Backup verification and testing
- Recovery time objectives (RTO)
- Data integrity validation