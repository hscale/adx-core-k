# Kiro to GitHub Task Sync - Design Document

## Overview

This design provides a simple one-way synchronization from Kiro tasks to GitHub issues. The system monitors Kiro task files for changes and automatically creates or updates corresponding GitHub issues using the GitHub REST API. The implementation is lightweight and focused on the core use case of keeping GitHub issues in sync with Kiro task progress.

## Architecture

### Simple Architecture

```mermaid
graph TB
    subgraph "Kiro Workspace"
        TaskFiles[Task Files (.md)]
        Config[.kiro/settings/github.json]
        Watcher[File Watcher]
    end
    
    subgraph "GitHub Sync Service"
        Parser[Task Parser]
        Syncer[GitHub Syncer]
        API[GitHub REST API Client]
    end
    
    subgraph "GitHub"
        Issues[GitHub Issues]
        Repo[Repository]
    end
    
    TaskFiles --> Watcher
    Watcher --> Parser
    Parser --> Syncer
    Config --> Syncer
    Syncer --> API
    API --> Issues
```

### Components

1. **File Watcher**: Monitors Kiro task files for changes
2. **Task Parser**: Extracts task information from markdown files
3. **GitHub Syncer**: Manages the sync logic and state
4. **GitHub API Client**: Handles GitHub REST API calls

## Components and Interfaces

### 1. Configuration

Simple JSON configuration stored in `.kiro/settings/github.json`:

```json
{
  "enabled": true,
  "token": "ghp_xxxxxxxxxxxxxxxxxxxx",
  "repository": "owner/repo-name",
  "labelPrefix": "kiro:",
  "syncOnSave": true
}
```

### 2. Task Parser

```typescript
interface KiroTask {
  id: string;
  title: string;
  description?: string;
  status: 'not_started' | 'in_progress' | 'completed';
  filePath: string;
  lineNumber: number;
}

class TaskParser {
  parseTaskFile(filePath: string): KiroTask[] {
    // Parse markdown file and extract tasks
    // Look for checkbox patterns: - [ ], - [-], - [x]
    // Extract task ID, title, and status
  }
  
  extractTaskId(taskLine: string): string {
    // Extract task ID from patterns like "1.1 Task Title" or "Task Title"
  }
  
  generateTaskDescription(task: KiroTask, context: string): string {
    // Generate GitHub issue description with context
    // Include link back to Kiro file and line number
  }
}
```

### 3. GitHub API Client

```typescript
interface GitHubIssue {
  id: number;
  number: number;
  title: string;
  body: string;
  state: 'open' | 'closed';
  labels: string[];
}

class GitHubClient {
  constructor(private token: string, private repository: string) {}
  
  async createIssue(title: string, body: string, labels: string[]): Promise<GitHubIssue> {
    // POST /repos/{owner}/{repo}/issues
  }
  
  async updateIssue(issueNumber: number, title: string, body: string): Promise<GitHubIssue> {
    // PATCH /repos/{owner}/{repo}/issues/{issue_number}
  }
  
  async closeIssue(issueNumber: number): Promise<GitHubIssue> {
    // PATCH /repos/{owner}/{repo}/issues/{issue_number} with state: 'closed'
  }
  
  async findIssueByLabel(label: string): Promise<GitHubIssue | null> {
    // GET /repos/{owner}/{repo}/issues?labels={label}
  }
}
```

### 4. GitHub Syncer

```typescript
interface SyncState {
  taskId: string;
  githubIssueNumber: number;
  lastSynced: Date;
  lastHash: string; // Hash of task content to detect changes
}

class GitHubSyncer {
  private syncState: Map<string, SyncState> = new Map();
  private stateFile = '.kiro/.github-sync-state.json';
  
  constructor(
    private githubClient: GitHubClient,
    private config: GitHubConfig
  ) {
    this.loadSyncState();
  }
  
  async syncTask(task: KiroTask): Promise<void> {
    const taskHash = this.hashTask(task);
    const existingState = this.syncState.get(task.id);
    
    if (existingState && existingState.lastHash === taskHash) {
      return; // No changes, skip sync
    }
    
    const label = `${this.config.labelPrefix}${task.id}`;
    const issueTitle = task.title;
    const issueBody = this.generateIssueBody(task);
    
    if (existingState) {
      // Update existing issue
      await this.updateExistingIssue(existingState, task, issueTitle, issueBody);
    } else {
      // Create new issue
      await this.createNewIssue(task, issueTitle, issueBody, label);
    }
    
    this.updateSyncState(task.id, taskHash);
  }
  
  private async createNewIssue(task: KiroTask, title: string, body: string, label: string): Promise<void> {
    const issue = await this.githubClient.createIssue(title, body, [label]);
    
    this.syncState.set(task.id, {
      taskId: task.id,
      githubIssueNumber: issue.number,
      lastSynced: new Date(),
      lastHash: this.hashTask(task)
    });
  }
  
  private async updateExistingIssue(state: SyncState, task: KiroTask, title: string, body: string): Promise<void> {
    if (task.status === 'completed') {
      await this.githubClient.closeIssue(state.githubIssueNumber);
    } else {
      await this.githubClient.updateIssue(state.githubIssueNumber, title, body);
    }
    
    state.lastSynced = new Date();
    state.lastHash = this.hashTask(task);
  }
  
  private generateIssueBody(task: KiroTask): string {
    let body = task.description || '';
    
    // Add metadata
    body += '\n\n---\n';
    body += `**Kiro Task ID:** ${task.id}\n`;
    body += `**Status:** ${task.status}\n`;
    body += `**Source:** ${task.filePath}:${task.lineNumber}\n`;
    body += `**Last Synced:** ${new Date().toISOString()}\n`;
    
    return body;
  }
  
  private hashTask(task: KiroTask): string {
    return crypto
      .createHash('md5')
      .update(JSON.stringify({
        title: task.title,
        description: task.description,
        status: task.status
      }))
      .digest('hex');
  }
  
  private loadSyncState(): void {
    try {
      const data = fs.readFileSync(this.stateFile, 'utf8');
      const stateArray = JSON.parse(data);
      this.syncState = new Map(stateArray.map((item: any) => [item.taskId, item]));
    } catch (error) {
      // State file doesn't exist or is invalid, start fresh
      this.syncState = new Map();
    }
  }
  
  private saveSyncState(): void {
    const stateArray = Array.from(this.syncState.values());
    fs.writeFileSync(this.stateFile, JSON.stringify(stateArray, null, 2));
  }
}
```

### 5. File Watcher

```typescript
class KiroFileWatcher {
  private watcher: chokidar.FSWatcher;
  
  constructor(
    private taskParser: TaskParser,
    private githubSyncer: GitHubSyncer
  ) {}
  
  start(): void {
    this.watcher = chokidar.watch('.kiro/specs/**/tasks.md', {
      ignored: /node_modules/,
      persistent: true
    });
    
    this.watcher
      .on('change', (path) => this.handleFileChange(path))
      .on('add', (path) => this.handleFileChange(path));
  }
  
  stop(): void {
    if (this.watcher) {
      this.watcher.close();
    }
  }
  
  private async handleFileChange(filePath: string): Promise<void> {
    try {
      const tasks = this.taskParser.parseTaskFile(filePath);
      
      for (const task of tasks) {
        await this.githubSyncer.syncTask(task);
      }
    } catch (error) {
      console.error(`Error syncing tasks from ${filePath}:`, error);
    }
  }
}
```

## Data Models

### Task Representation

```typescript
interface KiroTask {
  id: string;           // Extracted from task line (e.g., "1.1", "2.3")
  title: string;        // Task title text
  description?: string; // Additional context from sub-bullets
  status: TaskStatus;   // Parsed from checkbox state
  filePath: string;     // Source file path
  lineNumber: number;   // Line number in file
  specName: string;     // Spec name (extracted from file path)
}

enum TaskStatus {
  NOT_STARTED = 'not_started',  // - [ ]
  IN_PROGRESS = 'in_progress',  // - [-]
  COMPLETED = 'completed'       // - [x]
}
```

### Sync State

```typescript
interface SyncState {
  taskId: string;
  githubIssueNumber: number;
  lastSynced: Date;
  lastHash: string;
}
```

## Error Handling

### Simple Error Handling

```typescript
class SyncError extends Error {
  constructor(
    message: string,
    public taskId: string,
    public cause?: Error
  ) {
    super(message);
    this.name = 'SyncError';
  }
}

class GitHubSyncer {
  async syncTask(task: KiroTask): Promise<void> {
    try {
      // Sync logic here
    } catch (error) {
      console.error(`Failed to sync task ${task.id}:`, error);
      
      // Log error but continue with other tasks
      this.logError(new SyncError(
        `Failed to sync task ${task.id}`,
        task.id,
        error as Error
      ));
    }
  }
  
  private logError(error: SyncError): void {
    const errorLog = {
      timestamp: new Date().toISOString(),
      taskId: error.taskId,
      message: error.message,
      cause: error.cause?.message
    };
    
    // Append to error log file
    fs.appendFileSync('.kiro/.github-sync-errors.log', 
      JSON.stringify(errorLog) + '\n'
    );
  }
}
```

## Testing Strategy

### Unit Testing

```typescript
describe('TaskParser', () => {
  it('should parse tasks from markdown file', () => {
    const content = `
# Implementation Plan

- [ ] 1. Setup project structure
  - Create basic folder structure
  - Initialize configuration
  - _Requirements: 1.1_

- [-] 2. Implement core features
  - Build main functionality
  - _Requirements: 2.1_

- [x] 3. Testing
  - Write unit tests
  - _Requirements: 3.1_
    `;
    
    const parser = new TaskParser();
    const tasks = parser.parseContent(content, 'test.md');
    
    expect(tasks).toHaveLength(3);
    expect(tasks[0].id).toBe('1');
    expect(tasks[0].status).toBe(TaskStatus.NOT_STARTED);
    expect(tasks[1].status).toBe(TaskStatus.IN_PROGRESS);
    expect(tasks[2].status).toBe(TaskStatus.COMPLETED);
  });
});

describe('GitHubSyncer', () => {
  it('should create new issue for new task', async () => {
    const mockClient = new MockGitHubClient();
    const syncer = new GitHubSyncer(mockClient, mockConfig);
    
    const task: KiroTask = {
      id: '1.1',
      title: 'Test Task',
      status: TaskStatus.NOT_STARTED,
      filePath: 'test.md',
      lineNumber: 5,
      specName: 'test-spec'
    };
    
    await syncer.syncTask(task);
    
    expect(mockClient.createIssue).toHaveBeenCalledWith(
      'Test Task',
      expect.stringContaining('Kiro Task ID: 1.1'),
      ['kiro:1.1']
    );
  });
});
```

This simplified design focuses on the core requirement: one-way sync from Kiro tasks to GitHub issues with minimal complexity and configuration.