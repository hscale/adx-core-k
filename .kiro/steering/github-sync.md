---
inclusion: manual
---

# GitHub Task Sync Development Guidelines

## Technology Stack

This project uses a different, simpler stack than ADX CORE:

- **Language**: TypeScript/Node.js for simplicity and GitHub API compatibility
- **Architecture**: Single-purpose utility, not microservices
- **File Watching**: chokidar for monitoring Kiro task files
- **GitHub Integration**: REST API with personal access tokens
- **Storage**: Simple JSON files for state management
- **Testing**: Jest for unit tests, simple integration tests

## Key Dependencies

```json
{
  "chokidar": "^3.5.3",        // File watching
  "@octokit/rest": "^20.0.2",  // GitHub API client
  "commander": "^11.1.0",      // CLI interface
  "chalk": "^5.3.0",           // Colored console output
  "fs-extra": "^11.1.1",       // Enhanced file operations
  "crypto": "built-in",        // Content hashing
  "jest": "^29.7.0"            // Testing framework
}
```

## Architecture Principles

### Keep It Simple
- **Single Purpose**: Only sync Kiro tasks to GitHub issues (one-way)
- **No Complex State**: Simple JSON file for tracking sync state
- **Direct API Calls**: No workflow orchestration needed
- **Minimal Configuration**: Just GitHub token and repository

### File-Based Operation
- **Watch Task Files**: Monitor `.kiro/specs/**/tasks.md` for changes
- **Parse Markdown**: Extract tasks from checkbox patterns
- **State Persistence**: Store sync state in `.kiro/.github-sync-state.json`
- **Error Logging**: Simple log file for troubleshooting

### GitHub Integration Patterns
- **Personal Access Tokens**: Simpler than GitHub Apps for this use case
- **Issue Labels**: Use `kiro:taskId` labels to track synced issues
- **Issue Metadata**: Include Kiro context in issue descriptions
- **Graceful Failures**: Log errors but continue with other tasks

## Development Patterns

### Error Handling
```typescript
// Simple error handling - log and continue
try {
  await syncTask(task);
} catch (error) {
  console.error(`Failed to sync task ${task.id}:`, error);
  logError(task.id, error);
  // Continue with next task
}
```

### Configuration Management
```typescript
// Simple JSON config in .kiro/settings/github.json
interface GitHubConfig {
  enabled: boolean;
  token: string;
  repository: string;
  labelPrefix: string;
  syncOnSave: boolean;
}
```

### State Management
```typescript
// Simple in-memory state with JSON persistence
class SyncStateManager {
  private state: Map<string, SyncState> = new Map();
  
  load() { /* Load from .kiro/.github-sync-state.json */ }
  save() { /* Save to .kiro/.github-sync-state.json */ }
  get(taskId: string) { /* Get sync state for task */ }
  set(taskId: string, state: SyncState) { /* Update sync state */ }
}
```

## Testing Strategy

### Unit Tests
- Test task parsing with various markdown formats
- Test GitHub API client with mocked responses
- Test sync logic with different scenarios
- Test configuration validation

### Integration Tests
- Test with real GitHub repository (use test repo)
- Test file watching with temporary files
- Test end-to-end sync flow

### No Complex Testing
- No workflow testing (not using workflows)
- No microservice testing (single utility)
- No cross-service integration (standalone tool)

## CLI Design

### Simple Commands
```bash
kiro github setup     # Initial configuration
kiro github sync      # Manual sync trigger
kiro github status    # Show sync status
kiro github enable    # Enable auto-sync
kiro github disable   # Disable auto-sync
```

### User-Friendly Output
- Use chalk for colored output
- Show progress for sync operations
- Clear error messages with suggestions
- Helpful status information

## File Structure

```
github-sync/
├── src/
│   ├── cli.ts           # Command-line interface
│   ├── config.ts        # Configuration management
│   ├── parser.ts        # Task file parsing
│   ├── github.ts        # GitHub API client
│   ├── syncer.ts        # Main sync logic
│   ├── watcher.ts       # File watching
│   └── types.ts         # TypeScript interfaces
├── tests/
│   ├── parser.test.ts   # Parser unit tests
│   ├── github.test.ts   # GitHub client tests
│   └── integration.test.ts # End-to-end tests
├── package.json
├── tsconfig.json
└── README.md
```

## Development Guidelines

### Code Style
- Use TypeScript strict mode
- Prefer async/await over promises
- Use descriptive variable names
- Keep functions small and focused
- Add JSDoc comments for public APIs

### Error Messages
- Be specific about what went wrong
- Suggest solutions when possible
- Include relevant context (task ID, file path)
- Use consistent formatting

### Performance
- Only sync changed tasks (use content hashing)
- Respect GitHub API rate limits
- Use efficient file watching (avoid polling)
- Minimize memory usage for large task files

This is a utility tool, not an enterprise platform. Keep it simple, reliable, and focused on the core use case.