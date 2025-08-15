import { format, formatDistanceToNow, parseISO } from 'date-fns';
import { Workflow, WorkflowStatus, WorkflowPriority } from '../types';

export const formatWorkflowDuration = (durationMs: number): string => {
  if (durationMs < 1000) {
    return `${durationMs}ms`;
  }
  
  const seconds = Math.floor(durationMs / 1000);
  if (seconds < 60) {
    return `${seconds}s`;
  }
  
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    const remainingSeconds = seconds % 60;
    return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`;
  }
  
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
};

export const formatWorkflowTimestamp = (timestamp: string): string => {
  try {
    const date = parseISO(timestamp);
    return format(date, 'MMM dd, yyyy HH:mm:ss');
  } catch {
    return 'Invalid date';
  }
};

export const formatRelativeTime = (timestamp: string): string => {
  try {
    const date = parseISO(timestamp);
    return formatDistanceToNow(date, { addSuffix: true });
  } catch {
    return 'Unknown time';
  }
};

export const getWorkflowStatusColor = (status: WorkflowStatus): string => {
  switch (status) {
    case WorkflowStatus.RUNNING:
      return 'blue';
    case WorkflowStatus.COMPLETED:
      return 'green';
    case WorkflowStatus.FAILED:
      return 'red';
    case WorkflowStatus.PENDING:
      return 'yellow';
    case WorkflowStatus.CANCELLED:
      return 'gray';
    case WorkflowStatus.TIMED_OUT:
      return 'orange';
    default:
      return 'gray';
  }
};

export const getWorkflowStatusIcon = (status: WorkflowStatus): string => {
  switch (status) {
    case WorkflowStatus.RUNNING:
      return '⚡';
    case WorkflowStatus.COMPLETED:
      return '✅';
    case WorkflowStatus.FAILED:
      return '❌';
    case WorkflowStatus.PENDING:
      return '⏳';
    case WorkflowStatus.CANCELLED:
      return '🚫';
    case WorkflowStatus.TIMED_OUT:
      return '⏰';
    default:
      return '❓';
  }
};

export const getPriorityColor = (priority: WorkflowPriority): string => {
  switch (priority) {
    case WorkflowPriority.CRITICAL:
      return 'red';
    case WorkflowPriority.HIGH:
      return 'orange';
    case WorkflowPriority.NORMAL:
      return 'blue';
    case WorkflowPriority.LOW:
      return 'gray';
    default:
      return 'gray';
  }
};

export const getPriorityIcon = (priority: WorkflowPriority): string => {
  switch (priority) {
    case WorkflowPriority.CRITICAL:
      return '🔴';
    case WorkflowPriority.HIGH:
      return '🟠';
    case WorkflowPriority.NORMAL:
      return '🔵';
    case WorkflowPriority.LOW:
      return '⚪';
    default:
      return '⚪';
  }
};

export const calculateSuccessRate = (workflows: Workflow[]): number => {
  if (workflows.length === 0) return 0;
  
  const completedWorkflows = workflows.filter(
    w => w.status === WorkflowStatus.COMPLETED
  ).length;
  
  return Math.round((completedWorkflows / workflows.length) * 100);
};

export const calculateAverageDuration = (workflows: Workflow[]): number => {
  const completedWorkflows = workflows.filter(
    w => w.status === WorkflowStatus.COMPLETED && w.duration
  );
  
  if (completedWorkflows.length === 0) return 0;
  
  const totalDuration = completedWorkflows.reduce(
    (sum, w) => sum + (w.duration || 0), 
    0
  );
  
  return Math.round(totalDuration / completedWorkflows.length);
};

export const groupWorkflowsByStatus = (workflows: Workflow[]) => {
  return workflows.reduce((groups, workflow) => {
    const status = workflow.status;
    if (!groups[status]) {
      groups[status] = [];
    }
    groups[status].push(workflow);
    return groups;
  }, {} as Record<WorkflowStatus, Workflow[]>);
};

export const groupWorkflowsByType = (workflows: Workflow[]) => {
  return workflows.reduce((groups, workflow) => {
    const type = workflow.type;
    if (!groups[type]) {
      groups[type] = [];
    }
    groups[type].push(workflow);
    return groups;
  }, {} as Record<string, Workflow[]>);
};

export const filterWorkflowsByDateRange = (
  workflows: Workflow[],
  startDate: string,
  endDate: string
): Workflow[] => {
  const start = parseISO(startDate);
  const end = parseISO(endDate);
  
  return workflows.filter(workflow => {
    const workflowDate = parseISO(workflow.startedAt);
    return workflowDate >= start && workflowDate <= end;
  });
};

export const sortWorkflows = (
  workflows: Workflow[],
  sortBy: string,
  sortOrder: 'asc' | 'desc' = 'desc'
): Workflow[] => {
  return [...workflows].sort((a, b) => {
    let aValue: any;
    let bValue: any;
    
    switch (sortBy) {
      case 'startedAt':
        aValue = new Date(a.startedAt).getTime();
        bValue = new Date(b.startedAt).getTime();
        break;
      case 'duration':
        aValue = a.duration || 0;
        bValue = b.duration || 0;
        break;
      case 'type':
        aValue = a.type;
        bValue = b.type;
        break;
      case 'status':
        aValue = a.status;
        bValue = b.status;
        break;
      case 'priority':
        aValue = a.metadata.priority;
        bValue = b.metadata.priority;
        break;
      default:
        return 0;
    }
    
    if (aValue < bValue) {
      return sortOrder === 'asc' ? -1 : 1;
    }
    if (aValue > bValue) {
      return sortOrder === 'asc' ? 1 : -1;
    }
    return 0;
  });
};

export const isWorkflowActive = (workflow: Workflow): boolean => {
  return workflow.status === WorkflowStatus.RUNNING || 
         workflow.status === WorkflowStatus.PENDING;
};

export const canCancelWorkflow = (workflow: Workflow): boolean => {
  return workflow.status === WorkflowStatus.RUNNING || 
         workflow.status === WorkflowStatus.PENDING;
};

export const canRetryWorkflow = (workflow: Workflow): boolean => {
  return workflow.status === WorkflowStatus.FAILED ||
         workflow.status === WorkflowStatus.TIMED_OUT;
};

export const getWorkflowTypeDisplayName = (type: string): string => {
  return type
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
};

export const estimateTimeRemaining = (workflow: Workflow): number | null => {
  if (!workflow.progress || workflow.progress.percentage === 0) {
    return null;
  }
  
  const elapsed = Date.now() - new Date(workflow.startedAt).getTime();
  const totalEstimated = elapsed / (workflow.progress.percentage / 100);
  const remaining = totalEstimated - elapsed;
  
  return Math.max(0, remaining);
};

export const formatEstimatedTimeRemaining = (timeMs: number): string => {
  if (timeMs < 60000) { // Less than 1 minute
    return 'Less than 1 minute';
  }
  
  const minutes = Math.floor(timeMs / 60000);
  if (minutes < 60) {
    return `About ${minutes} minute${minutes === 1 ? '' : 's'}`;
  }
  
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  
  if (remainingMinutes === 0) {
    return `About ${hours} hour${hours === 1 ? '' : 's'}`;
  }
  
  return `About ${hours}h ${remainingMinutes}m`;
};