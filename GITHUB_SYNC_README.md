# Kiro GitHub Task Sync

Simple one-way synchronization from Kiro tasks to GitHub issues.

## Features

- **One-way sync**: Kiro tasks automatically create and update GitHub issues
- **File watching**: Real-time sync when task files change
- **Status tracking**: Completed tasks close corresponding GitHub issues
- **Configuration management**: Simple setup and configuration
- **Error handling**: Comprehensive error logging and retry logic

## Installation

```bash
# Install dependencies
npm install

# Build the project
npm run build
```

## Configuration

### Initial Setup

```bash
# Setup with GitHub token and repository
npm run start setup --token ghp_your_token_here --repo owner/repo-name

# Or use environment variable for token
export GITHUB_TOKEN=ghp_your_token_here
npm run start setup --repo owner/repo-name
```

### Configuration Options

The configuration is stored in `.kiro/settings/github.json`:

```json
{
  "enabled": true,
  "repository": "owner/repo-name",
  "labelPrefix": "kiro:",
  "syncOnSave": true,
  "apiUrl": "https://api.github.com",
  "maxRetries": 3,
  "retryDelay": 1000,
  "rateLimitBuffer": 100
}
```

## Usage

### CLI Commands

```bash
# Setup configuration
npm run start setup --token <token> --repo <owner/repo>

# Manual sync all tasks
npm run start sync

# Show sync status
npm run start status

# Enable/disable sync
npm run start enable
npm run start disable

# Start file watcher (runs continuously)
npm run start watch
```

### Development

```bash
# Development mode with hot reload
npm run dev

# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Lint code
npm run lint

# Type check
npm run type-check
```

## How It Works

1. **Task Detection**: Monitors `.kiro/specs/**/tasks.md` files for changes
2. **Task Parsing**: Extracts task information from markdown checkbox format
3. **GitHub Integration**: Creates/updates/closes GitHub issues based on task status
4. **State Management**: Tracks sync state to avoid unnecessary API calls
5. **Error Handling**: Logs errors and retries failed operations

## Task Format

The sync service recognizes tasks in this format:

```markdown
- [ ] 1. Task title
  - Task description or details
  - _Requirements: 1.1, 2.3_

- [-] 2. In progress task
  - This task is currently being worked on

- [x] 3. Completed task
  - This task has been completed
```

## GitHub Issue Mapping

- `[ ]` → Open GitHub issue
- `[-]` → Open GitHub issue (in progress)
- `[x]` → Closed GitHub issue

Each issue includes:
- Task title as issue title
- Task description and context as issue body
- Kiro task ID as label (e.g., `kiro:1.2`)
- Link back to source file and line number

## Requirements

- Node.js 18+
- GitHub personal access token with repository write permissions
- Kiro workspace with task files in `.kiro/specs/` directory

## Security

- GitHub tokens can be stored in environment variables
- Configuration file excludes tokens when using environment variables
- All API calls use HTTPS
- Rate limiting respects GitHub API limits