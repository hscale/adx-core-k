# ADX Core GitHub Task Sync

This system automatically syncs Kiro tasks from `.kiro/specs/adx-core/tasks.md` to GitHub issues, providing seamless project management integration.

## Features

- **Comprehensive Task Sync**: Syncs all 43 ADX Core implementation tasks
- **Status Tracking**: Automatically opens/closes issues based on task completion status
- **Smart Labeling**: Applies relevant labels for phases, components, requirements, and status
- **Incremental Updates**: Only updates issues when task content changes
- **Rate Limit Handling**: Respects GitHub API rate limits with automatic retry
- **Multi-tenant Architecture Aware**: Labels tasks by architectural components

## Quick Start

### 1. Setup GitHub Sync

```bash
# Set environment variables
export GITHUB_TOKEN="your_github_personal_access_token"
export GITHUB_REPOSITORY="your-org/your-repo"

# Run setup
npm run setup-github
```

### 2. Sync All Tasks

```bash
# Sync all ADX Core tasks to GitHub issues
npm run sync-tasks
```

### 3. Sync Individual Task Completion

```bash
# Sync specific completed task
npm run sync-task-completion
```

## Configuration

The system uses `.kiro/settings/github.json` for configuration:

```json
{
  "enabled": true,
  "repository": "your-org/your-repo",
  "labelPrefix": "adx-core:",
  "syncOnSave": true,
  "apiUrl": "https://api.github.com",
  "maxRetries": 3,
  "retryDelay": 1000,
  "rateLimitBuffer": 100
}
```

### Environment Variables

- `GITHUB_TOKEN`: GitHub personal access token with `repo` permissions
- `GITHUB_REPOSITORY`: Target repository in format `owner/repo`

## Task Parsing

The system parses tasks from `.kiro/specs/adx-core/tasks.md` with the following format:

```markdown
- [x] 1. Project Structure and Workspace Setup
  - Create root `adx-core/` directory with Rust workspace structure
  - Initialize workspace `Cargo.toml` with microservices members
  - _Requirements: 3.1 (Temporal-first backend microservices), 13.1 (Team autonomy)_
```

### Task Status Detection

- `[x]` = Completed task (issue will be closed)
- `[ ]` = Not started task (issue will be open)
- `[~]` = In progress task (issue will be open with in-progress label)

## GitHub Issue Structure

### Issue Title Format
```
✅ [adx-core] 1: Project Structure and Workspace Setup
🔄 [adx-core] 8: Auth Service Database Layer
📋 [adx-core] 19: API Gateway Implementation
```

### Labels Applied

**Core Labels:**
- `adx-core:1` - Task identifier
- `spec:adx-core` - Specification name
- `status:completed` - Task status
- `phase:1-2` - Implementation phase

**Component Labels:**
- `component:temporal` - Temporal workflow tasks
- `component:database` - Database-related tasks
- `component:auth` - Authentication tasks
- `component:tenant` - Multi-tenancy tasks
- `component:frontend` - Frontend microservice tasks
- `component:api` - API Gateway tasks
- `component:testing` - Testing tasks

**Requirement Labels:**
- `requirement:3.1` - Temporal-first backend microservices
- `requirement:2.1` - Multi-tenant architecture
- `requirement:8.1` - Frontend microservices architecture

## Architecture Integration

### ADX Core Task Categories

The sync system understands ADX Core's architecture and applies appropriate labels:

**Phase 1-2: Foundation (Tasks 1-7)**
- Project structure and workspace setup
- Temporal infrastructure setup
- Database and caching infrastructure
- Shared library foundation
- Temporal SDK integration
- Database migrations and schema setup
- Auth service HTTP server implementation

**Phase 3: Tenant Service (Tasks 8-13)**
- Auth service database layer
- Auth service Temporal worker mode
- Authentication activities implementation
- Tenant service dual-mode implementation
- Tenant management Temporal workflows
- Tenant activities and RBAC

**Phase 4: User and File Services (Tasks 14-18)**
- User service dual-mode implementation
- User management Temporal workflows
- File service dual-mode implementation
- File processing Temporal workflows
- File storage activities

**Phase 5: API Gateway (Tasks 19-21)**
- API Gateway implementation
- Cross-service workflow orchestration
- Workflow monitoring and management

**Phase 6-12: Frontend, BFF, and Advanced Features (Tasks 22-43)**
- Frontend microservices foundation
- BFF services implementation
- User experience and AI integration
- Testing and quality assurance
- Enterprise features and production readiness

## Workflow Integration

### Temporal-First Architecture Support

The sync system recognizes Temporal-first patterns and labels accordingly:

- **Workflow Tasks**: Tasks implementing Temporal workflows
- **Activity Tasks**: Tasks implementing Temporal activities
- **Dual-Mode Tasks**: Tasks implementing both HTTP endpoints and Temporal workers
- **Cross-Service Tasks**: Tasks coordinating multiple microservices

### Multi-Tenant Architecture Support

Tasks are labeled based on multi-tenancy concerns:

- **Tenant Isolation**: Database, application, and workflow-level isolation
- **Tenant Management**: Provisioning, monitoring, and lifecycle management
- **Cross-Tenant Operations**: Secure data sharing and migration

## Error Handling

The system includes comprehensive error handling:

- **Rate Limit Management**: Automatic backoff and retry
- **Connection Issues**: Retry with exponential backoff
- **Authentication Errors**: Clear error messages and guidance
- **Validation Errors**: Detailed validation feedback

## Monitoring and Logging

All operations are logged with structured logging:

```typescript
logger.info('Synced task', {
  taskId: '1',
  issueNumber: 123,
  status: 'completed',
  issueUrl: 'https://github.com/org/repo/issues/123'
});
```

## Development

### Adding New Task Types

To support new task types, update the label generation logic:

```typescript
// In generateLabels method
if (titleLower.includes('new-component')) {
  labels.push('component:new-component');
}
```

### Extending Requirements

Add new requirements to the description mapping:

```typescript
const descriptions: Record<string, string> = {
  '16.1': 'New requirement description',
  // ... existing requirements
};
```

## Troubleshooting

### Common Issues

**Authentication Failed**
```bash
# Check token permissions
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
```

**Repository Not Found**
```bash
# Verify repository format
echo $GITHUB_REPOSITORY  # Should be "owner/repo"
```

**Rate Limit Exceeded**
- The system automatically handles rate limits
- Check rate limit status: `curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit`

**Task Parsing Issues**
- Ensure tasks follow the expected markdown format
- Check for proper checkbox syntax: `- [x]` or `- [ ]`
- Verify task numbering is sequential

### Debug Mode

Enable debug logging:

```bash
DEBUG=* npm run sync-tasks
```

## Contributing

When adding new features:

1. Update task parsing logic in `parseTasks()`
2. Extend label generation in `generateLabels()`
3. Update requirement descriptions in `getRequirementDescription()`
4. Add tests for new functionality
5. Update this documentation

## License

MIT License - see LICENSE file for details.