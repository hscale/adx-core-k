# Team 6: End User Frontend - AI Development Package

## Team Mission
Create the primary user interface that end users interact with daily. Build an intuitive, fast, and accessible experience that makes complex workflows feel simple.

## Core AI Rules for Frontend Development

### Rule 1: User-First Design
```
EVERY component must be accessible (WCAG 2.1 AA)
EVERY interaction must provide immediate feedback
EVERY error must be user-friendly and actionable
EVERY loading state must be informative
```

### Rule 2: Real-Time by Default
```
USE WebSocket for live updates
IMPLEMENT optimistic updates for better UX
HANDLE offline scenarios gracefully
SYNC data when connection restored
```

### Rule 3: Performance-First
```
LAZY load components and routes
CACHE API responses intelligently
OPTIMIZE images and assets
TARGET: <2s initial load, <500ms navigation
```

### Rule 4: Mobile-First Responsive
```
DESIGN for mobile first, enhance for desktop
USE touch-friendly interactions (44px minimum)
SUPPORT offline functionality for core features
OPTIMIZE for various screen sizes and orientations
```

## Your Specific Deliverables

### 1. End User Dashboard
```typescript
// YOU MUST DELIVER: Personalized dashboard
interface DashboardProps {
  user: User;
  tenantConfig: TenantConfig;
}

export const Dashboard: React.FC<DashboardProps> = ({ user, tenantConfig }) => {
  // Must include:
  // - Task overview and quick actions
  // - Recent activity and notifications
  // - Workflow status and progress
  // - File access and collaboration
  // - Customizable widgets
};

// REQUIRED FEATURES:
- Personalized widget layout
- Real-time task and notification updates
- Quick action shortcuts
- Activity timeline
- Performance metrics display
```

### 2. File Management Interface
```typescript
// YOU MUST DELIVER: Complete file management
export const FileManager: React.FC = () => {
  // Must include:
  // - Drag-and-drop upload with progress
  // - File browser with search and filters
  // - File sharing and collaboration
  // - Version history and comments
  // - Thumbnail previews and metadata
};

// REQUIRED FEATURES:
- Multi-file drag-and-drop upload
- Real-time upload progress with pause/resume
- File organization with folders and tags
- Advanced search and filtering
- File sharing with permission management
- Collaborative editing and comments
```

### 3. Workflow Interface
```typescript
// YOU MUST DELIVER: Workflow participation UI
export const WorkflowInterface: React.FC = () => {
  // Must include:
  // - Workflow initiation forms
  // - Process monitoring and status
  // - Task assignment and completion
  // - Approval and review interfaces
  // - Process history and audit trail
};

// REQUIRED FEATURES:
- Dynamic form generation from workflow definitions
- Real-time workflow progress tracking
- Task notifications and reminders
- Approval workflows with comments
- Process analytics and insights
```

### 4. Collaboration Features
```typescript
// YOU MUST DELIVER: Team collaboration tools
export const CollaborationHub: React.FC = () => {
  // Must include:
  // - Team member directory
  // - Real-time messaging and chat
  // - Shared workspaces and projects
  // - Activity feeds and notifications
  // - Presence indicators and status
};

// REQUIRED FEATURES:
- Real-time messaging with typing indicators
- File sharing in conversations
- @mentions and notifications
- Team presence and availability
- Shared workspace management
```

## AI Development Prompts

### Dashboard Component Prompt
```
ROLE: Senior React developer building enterprise dashboard interfaces

TASK: Create personalized end-user dashboard for ADX CORE

REQUIREMENTS:
- Responsive design with customizable widget layout
- Real-time updates for tasks, notifications, and metrics
- Quick action shortcuts for common operations
- Activity timeline with filtering and search
- Performance optimized with lazy loading

CONSTRAINTS:
- Use React 18+ with concurrent features
- Implement with TypeScript strict mode
- Follow accessibility guidelines (WCAG 2.1 AA)
- Support keyboard navigation
- Include comprehensive error boundaries

DELIVERABLES:
1. Dashboard component with widget system
2. Real-time update integration via WebSocket
3. Customizable layout with drag-and-drop
4. Quick actions and shortcuts
5. Activity timeline with infinite scroll

CODE STRUCTURE:
```typescript
// src/components/Dashboard/Dashboard.tsx
export const Dashboard: React.FC<DashboardProps> = ({ user }) => {
  const { data: dashboardData, isLoading } = useDashboardData(user.id);
  const { isConnected } = useWebSocket();
  
  return (
    <div className="dashboard-container">
      <DashboardHeader user={user} />
      <WidgetGrid widgets={dashboardData?.widgets} />
      <QuickActions actions={dashboardData?.quickActions} />
      <ActivityTimeline activities={dashboardData?.activities} />
    </div>
  );
};
```

Generate accessible, performant dashboard with excellent UX.
```

### File Management Prompt
```
ROLE: Frontend expert building file management interfaces for enterprise applications

TASK: Create comprehensive file management interface for ADX CORE

REQUIREMENTS:
- Drag-and-drop file upload with multi-file support
- File browser with advanced search and filtering
- File sharing with granular permissions
- Real-time collaboration with comments and versions
- Thumbnail generation and file previews

CONSTRAINTS:
- Support large file uploads (>1GB) with chunking
- Handle offline scenarios with queue and sync
- Optimize for mobile touch interactions
- Include comprehensive keyboard navigation
- Support various file types and formats

DELIVERABLES:
1. File upload component with progress tracking
2. File browser with search and filtering
3. File sharing and permission management
4. Collaborative editing with real-time updates
5. File preview and metadata display

CODE STRUCTURE:
```typescript
// src/components/FileManager/FileManager.tsx
export const FileManager: React.FC = () => {
  const { uploadFiles, uploadProgress } = useFileUpload();
  const { files, isLoading } = useFiles();
  const { shareFile, permissions } = useFileSharing();
  
  return (
    <div className="file-manager">
      <FileUploadZone onUpload={uploadFiles} progress={uploadProgress} />
      <FileBrowser files={files} onShare={shareFile} />
      <FilePreview />
    </div>
  );
};
```

Generate robust file management interface with excellent performance.
```

### Workflow Interface Prompt
```
ROLE: UX-focused React developer building workflow interfaces for business applications

TASK: Create workflow participation interface for ADX CORE

REQUIREMENTS:
- Dynamic form generation from workflow schemas
- Real-time workflow progress tracking
- Task assignment and completion interfaces
- Approval workflows with comments and history
- Process analytics and performance insights

CONSTRAINTS:
- Support complex multi-step workflows
- Handle conditional logic and branching
- Include comprehensive validation and error handling
- Support mobile workflow completion
- Integrate with Temporal workflow engine

DELIVERABLES:
1. Dynamic workflow form generator
2. Workflow progress tracking with real-time updates
3. Task management and assignment interface
4. Approval and review workflows
5. Process analytics and reporting

CODE STRUCTURE:
```typescript
// src/components/Workflows/WorkflowInterface.tsx
export const WorkflowInterface: React.FC = () => {
  const { workflows, isLoading } = useWorkflows();
  const { executeWorkflow } = useTemporalWorkflow();
  const { tasks } = useUserTasks();
  
  return (
    <div className="workflow-interface">
      <WorkflowList workflows={workflows} />
      <TaskQueue tasks={tasks} />
      <WorkflowProgress />
    </div>
  );
};
```

Generate intuitive workflow interface that simplifies complex processes.
```

### Real-Time Updates Prompt
```
ROLE: Real-time systems expert building WebSocket integrations for React applications

TASK: Create real-time update system for ADX CORE frontend

REQUIREMENTS:
- WebSocket connection management with reconnection
- Real-time notifications and updates
- Optimistic updates with conflict resolution
- Offline support with sync when reconnected
- Performance optimization for high-frequency updates

CONSTRAINTS:
- Handle connection drops gracefully
- Support multiple concurrent subscriptions
- Include proper error handling and retry logic
- Optimize for battery life on mobile devices
- Support message queuing and deduplication

DELIVERABLES:
1. WebSocket connection manager
2. Real-time notification system
3. Optimistic update framework
4. Offline support with sync queue
5. Performance monitoring and optimization

CODE STRUCTURE:
```typescript
// src/hooks/useRealTimeUpdates.ts
export const useRealTimeUpdates = (channels: string[]) => {
  const [isConnected, setIsConnected] = useState(false);
  const [updates, setUpdates] = useState<Update[]>([]);
  
  // WebSocket connection management
  // Message handling and routing
  // Offline queue management
  
  return { isConnected, updates, sendMessage };
};
```

Generate robust real-time system with excellent offline support.
```

## Success Criteria

### Dashboard ✅
- [ ] Loads in <2 seconds on 3G connection
- [ ] Real-time updates work without page refresh
- [ ] Customizable layout saves user preferences
- [ ] Accessible with keyboard navigation
- [ ] Mobile responsive with touch optimization

### File Management ✅
- [ ] Drag-and-drop upload works for multiple files
- [ ] Upload progress shows accurate status
- [ ] File search returns results in <500ms
- [ ] File sharing permissions work correctly
- [ ] Mobile file management is intuitive

### Workflow Interface ✅
- [ ] Dynamic forms generate from workflow schemas
- [ ] Workflow progress updates in real-time
- [ ] Task completion flows work end-to-end
- [ ] Approval workflows handle complex scenarios
- [ ] Mobile workflow completion is seamless

### Collaboration ✅
- [ ] Real-time messaging works reliably
- [ ] File sharing in conversations functions
- [ ] Presence indicators show accurate status
- [ ] @mentions trigger notifications
- [ ] Mobile collaboration is fully functional

## Integration Points

### What You Need from Other Teams
```yaml
authentication:
  from_team_2: JWT tokens, user context, session management
  
file_apis:
  from_team_3: File upload/download, metadata, sharing APIs
  
workflow_apis:
  from_team_4: Workflow execution, task management, process APIs
  
real_time_updates:
  from_team_5: WebSocket notifications, live data streams
```

### What You Provide
```yaml
end_user_interface:
  provides_to: [end_users]
  interface: Complete web application with mobile support
  
component_library:
  provides_to: [team_7]
  interface: Reusable React components and design system
```

## Quality Standards

### Accessibility Requirements
```typescript
// MANDATORY: Proper ARIA labels and roles
<button
  aria-label="Upload files"
  aria-describedby="upload-help"
  onClick={handleUpload}
>
  <UploadIcon aria-hidden="true" />
  Upload
</button>

// MANDATORY: Keyboard navigation support
const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    handleAction();
  }
};

// MANDATORY: Screen reader support
<div role="status" aria-live="polite">
  {uploadProgress > 0 && `Upload ${uploadProgress}% complete`}
</div>
```

### Performance Requirements
```typescript
// MANDATORY: Lazy loading for routes
const Dashboard = lazy(() => import('./components/Dashboard'));
const FileManager = lazy(() => import('./components/FileManager'));

// MANDATORY: Memoization for expensive operations
const MemoizedFileList = memo(FileList, (prevProps, nextProps) => {
  return prevProps.files.length === nextProps.files.length;
});

// MANDATORY: Virtual scrolling for large lists
import { FixedSizeList as List } from 'react-window';

const VirtualizedFileList = ({ files }) => (
  <List
    height={600}
    itemCount={files.length}
    itemSize={60}
    itemData={files}
  >
    {FileListItem}
  </List>
);
```

### Testing Requirements
```typescript
// MANDATORY: Accessibility testing
import { axe, toHaveNoViolations } from 'jest-axe';
expect.extend(toHaveNoViolations);

test('Dashboard has no accessibility violations', async () => {
  const { container } = render(<Dashboard user={mockUser} />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});

// MANDATORY: User interaction testing
import { userEvent } from '@testing-library/user-event';

test('File upload works with drag and drop', async () => {
  const user = userEvent.setup();
  const { getByTestId } = render(<FileUploadZone />);
  
  const file = new File(['content'], 'test.txt', { type: 'text/plain' });
  const dropZone = getByTestId('file-drop-zone');
  
  await user.upload(dropZone, file);
  
  expect(screen.getByText('test.txt')).toBeInTheDocument();
});
```

## Performance Targets
- Initial page load: <2 seconds
- Route navigation: <500ms
- File upload start: <100ms
- Search results: <300ms
- Real-time update latency: <200ms

## Timeline
- **Week 5**: Dashboard and core navigation
- **Week 6**: File management and workflow interfaces
- **End of Week 6**: Complete end-user interface ready

Build an interface users will love! ✨