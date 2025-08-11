# Kiro GitHub Sync Hook Documentation

## Overview

The ADX Core GitHub Task Sync Hook provides automatic synchronization between Kiro task specifications and GitHub issues. When the `tasks.md` file is modified, the hook intelligently analyzes changes and updates corresponding GitHub issues with comprehensive project management integration.

## Hook Configuration

### Location
`.kiro/hooks/kiro-github-sync.kiro.hook`

### Trigger
The hook activates when `.kiro/specs/adx-core/tasks.md` is edited, providing real-time sync between Kiro specifications and GitHub project management.

### Hook Definition
```json
{
  "enabled": true,
  "name": "ADX Core GitHub Task Sync",
  "description": "Automatically syncs ADX Core tasks to GitHub issues...",
  "version": "2.0",
  "when": {
    "type": "fileEdited",
    "patterns": [".kiro/specs/adx-core/tasks.md"]
  },
  "then": {
    "type": "askAgent",
    "prompt": "Comprehensive GitHub sync with architectural awareness..."
  }
}
```

## Hook Execution Flow

### 1. File Change Detection
- Monitors `.kiro/specs/adx-core/tasks.md` for modifications
- Captures change type (created, modified, deleted)
- Records timestamp and file path

### 2. Intelligent Analysis
- Parses all 43 ADX Core tasks
- Identifies status changes (completed ↔ pending)
- Detects new tasks or description updates
- Analyzes architectural component changes

### 3. GitHub Synchronization
- Creates new GitHub issues for new tasks
- Updates existing issues with latest information
- Closes issues for completed tasks
- Reopens issues for tasks marked as pending
- Applies comprehensive labeling system

### 4. Manager Reporting
- Provides sync summary with statistics
- Reports status changes and new issues
- Shows component and phase breakdown
- Offers troubleshooting guidance

## Available Execution Modes

### 1. Interactive Mode (Default)
```bash
npm run hook-github-sync
```
- Performs dry-run analysis first
- Asks for confirmation before syncing
- Provides detailed progress feedback
- Suitable for manual execution

### 2. Automatic Mode
```bash
npm run hook-github-sync-auto
```
- Skips confirmation prompts
- Performs sync automatically
- Suitable for CI/CD integration
- Used by Kiro hook system

### 3. Dry-Run Mode
```bash
npm run hook-github-sync-dry-run
```
- Analyzes tasks without GitHub changes
- Shows what would be synced
- Provides statistics and breakdown
- Safe for testing and analysis

### 4. Direct Hook Execution
```bash
tsx kiro-hook-github-sync.ts --file=.kiro/specs/adx-core/tasks.md
```
- Simulates hook execution with context
- Includes file path and change information
- Useful for testing hook behavior

## GitHub Integration Features

### Issue Management
- **Create Issues**: New GitHub issues for all tasks
- **Update Issues**: Sync changes in descriptions and requirements
- **Status Tracking**: Automatically close/reopen based on completion
- **Label Management**: Apply comprehensive architectural labels

### Labeling System
The hook applies intelligent labels based on ADX Core architecture:

**Core Labels:**
- `adx-core:1` through `adx-core:43` - Task identifiers
- `spec:adx-core` - Specification name
- `status:completed|in_progress|not_started` - Current status
- `phase:1-2` through `phase:12` - Implementation phases

**Component Labels:**
- `component:temporal` - Temporal workflow tasks
- `component:auth` - Authentication tasks
- `component:tenant` - Multi-tenancy tasks
- `component:user` - User management tasks
- `component:file` - File management tasks
- `component:frontend` - Frontend microservice tasks
- `component:bff` - Backend for Frontend tasks
- `component:database` - Database tasks
- `component:api` - API Gateway tasks
- `component:testing` - Testing tasks
- `component:module` - Module system tasks
- `component:ai` - AI integration tasks

**Requirement Labels:**
- `requirement:3.1` - Temporal-first backend microservices
- `requirement:2.1` - Multi-tenant architecture
- `requirement:8.1` - Frontend microservices architecture
- Plus 20+ additional architectural requirements

### Issue Content
Each GitHub issue includes:
- **Smart Title**: `✅ [adx-core] 1: Project Structure and Workspace Setup`
- **Detailed Description**: Implementation guidelines and requirements
- **Status Indicator**: Visual status with emojis
- **Architecture Context**: Temporal-first, multi-tenant, microservices guidance
- **Requirement Mapping**: Links to architectural requirements
- **Implementation Guidelines**: Best practices and patterns

## Configuration Management

### Setup Requirements
```bash
# Required environment variables
export GITHUB_TOKEN="your_github_personal_access_token"
export GITHUB_REPOSITORY="your-org/your-repo"

# Initial setup
npm run setup-github
```

### Configuration File
`.kiro/settings/github.json`:
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

### Token Permissions
The GitHub token requires:
- `repo` - Full repository access
- `issues` - Create and modify issues
- `metadata` - Read repository metadata

## Architecture Awareness

### Temporal-First Design
The hook understands ADX Core's Temporal-first architecture:
- Identifies workflow vs. activity tasks
- Recognizes dual-mode service patterns
- Maps cross-service orchestration requirements
- Labels workflow-related tasks appropriately

### Multi-Tenant Architecture
Intelligent multi-tenancy support:
- Labels tenant isolation tasks
- Identifies RBAC and security requirements
- Tracks tenant lifecycle workflows
- Maps tenant management operations

### Microservices Architecture
Frontend and backend microservices awareness:
- Maps micro-frontend boundaries
- Identifies BFF optimization opportunities
- Tracks team autonomy requirements
- Labels vertical slice ownership

## Error Handling and Recovery

### Common Issues and Solutions

**GitHub Authentication Failed**
```
❌ GitHub authentication failed
💡 Solutions:
   1. Check GITHUB_TOKEN environment variable
   2. Ensure token has "repo" permissions
   3. Run: npm run setup-github
```

**Repository Not Found**
```
❌ Repository not found
💡 Solutions:
   1. Check GITHUB_REPOSITORY format ("owner/repo")
   2. Verify repository exists
   3. Ensure token has access to repository
```

**Rate Limit Exceeded**
```
❌ GitHub rate limit exceeded
💡 The hook automatically handles rate limits:
   - Waits for rate limit reset
   - Uses exponential backoff
   - Provides progress updates
```

**Configuration Issues**
```
❌ GitHub sync not configured
💡 Run setup:
   npm run setup-github
```

### Automatic Recovery
- **Rate Limiting**: Automatic backoff and retry
- **Network Issues**: Exponential backoff with retry
- **Partial Failures**: Continue with remaining tasks
- **Configuration Errors**: Clear error messages and guidance

## Monitoring and Logging

### Structured Logging
All operations are logged with context:
```typescript
logger.info('Hook executed', {
  filePath: '.kiro/specs/adx-core/tasks.md',
  changeType: 'modified',
  tasksAnalyzed: 43,
  issuesUpdated: 5,
  issuesClosed: 2
});
```

### Progress Reporting
- Real-time sync progress
- Task-by-task status updates
- Error reporting with context
- Success confirmation with statistics

### Manager Dashboard
After sync completion:
- Total tasks synced
- Status change summary
- New issues created
- Component breakdown
- Phase progress update

## Integration with Kiro IDE

### Automatic Execution
- Triggered by file save in Kiro IDE
- Runs in background without interruption
- Provides notifications on completion
- Integrates with Kiro's agent system

### Agent Integration
The hook works with Kiro's AI agent to:
- Analyze task changes intelligently
- Provide architectural context
- Generate appropriate issue descriptions
- Apply correct labeling based on content

### Development Workflow
1. **Edit tasks.md** in Kiro IDE
2. **Save file** - Hook automatically triggers
3. **Agent analyzes** changes and architectural context
4. **GitHub sync** updates issues automatically
5. **Manager notification** with sync summary

## Best Practices

### For Managers
1. **Review sync results** after major task updates
2. **Use GitHub labels** to organize team work
3. **Monitor phase progress** through issue status
4. **Assign issues** to team members for tracking

### For Developers
1. **Keep task descriptions** clear and detailed
2. **Update task status** promptly in tasks.md
3. **Include architectural context** in task descriptions
4. **Follow ADX Core patterns** in implementation

### For Teams
1. **Use component labels** to filter relevant work
2. **Track cross-service dependencies** through requirements
3. **Monitor workflow tasks** for Temporal implementation
4. **Coordinate through GitHub issues** and Kiro specs

## Troubleshooting

### Debug Mode
Enable detailed logging:
```bash
DEBUG=* npm run hook-github-sync
```

### Test Hook Execution
```bash
# Test without GitHub changes
npm run hook-github-sync-dry-run

# Test with specific file
tsx kiro-hook-github-sync.ts --file=.kiro/specs/adx-core/tasks.md --dry-run
```

### Validate Configuration
```bash
# Check configuration
npm run setup-github

# Test GitHub connection
npm run sync-tasks-dry-run
```

### Reset Sync State
If issues occur, you can:
1. **Reconfigure**: `npm run setup-github`
2. **Full resync**: `npm run sync-tasks`
3. **Analyze first**: `npm run sync-tasks-dry-run`

## Future Enhancements

### Planned Features
- **Incremental sync** - Only update changed tasks
- **Conflict resolution** - Handle concurrent edits
- **Webhook integration** - Bi-directional sync
- **Team assignment** - Automatic assignee mapping
- **Milestone integration** - Phase-based milestones

### Extensibility
The hook system is designed for extension:
- **Custom labeling** rules
- **Additional repositories** support
- **Different task formats** parsing
- **Integration with other tools** (Jira, Linear, etc.)

This comprehensive hook system ensures seamless integration between Kiro task specifications and GitHub project management, providing managers with real-time visibility into ADX Core implementation progress while maintaining architectural alignment and team coordination.