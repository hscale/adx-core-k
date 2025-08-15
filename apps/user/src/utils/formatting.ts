import { User, UserActivity } from '../types';

// Format user display name
export const formatUserDisplayName = (user: User): string => {
  if (user.displayName) {
    return user.displayName;
  }
  return `${user.firstName} ${user.lastName}`.trim();
};

// Format user initials
export const formatUserInitials = (user: User): string => {
  const firstName = user.firstName?.charAt(0)?.toUpperCase() || '';
  const lastName = user.lastName?.charAt(0)?.toUpperCase() || '';
  return `${firstName}${lastName}`;
};

// Format user roles
export const formatUserRoles = (roles: string[]): string => {
  if (roles.length === 0) return 'No roles';
  if (roles.length === 1) return roles[0];
  if (roles.length === 2) return roles.join(' and ');
  return `${roles.slice(0, -1).join(', ')}, and ${roles[roles.length - 1]}`;
};

// Format last login time
export const formatLastLogin = (lastLoginAt?: string): string => {
  if (!lastLoginAt) return 'Never';
  
  const date = new Date(lastLoginAt);
  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  const diffInMinutes = Math.floor(diffInMs / (1000 * 60));
  const diffInHours = Math.floor(diffInMinutes / 60);
  const diffInDays = Math.floor(diffInHours / 24);

  if (diffInMinutes < 1) return 'Just now';
  if (diffInMinutes < 60) return `${diffInMinutes} minute${diffInMinutes === 1 ? '' : 's'} ago`;
  if (diffInHours < 24) return `${diffInHours} hour${diffInHours === 1 ? '' : 's'} ago`;
  if (diffInDays < 7) return `${diffInDays} day${diffInDays === 1 ? '' : 's'} ago`;
  
  return date.toLocaleDateString();
};

// Format activity description
export const formatActivityDescription = (activity: UserActivity): string => {
  switch (activity.type) {
    case 'login':
      return 'Signed in';
    case 'logout':
      return 'Signed out';
    case 'profile_update':
      return 'Updated profile';
    case 'password_change':
      return 'Changed password';
    case 'workflow_execution':
      return `Executed workflow: ${activity.metadata?.workflowType || 'Unknown'}`;
    case 'file_upload':
      return `Uploaded file: ${activity.metadata?.fileName || 'Unknown'}`;
    default:
      return activity.description || 'Unknown activity';
  }
};

// Format activity timestamp
export const formatActivityTimestamp = (timestamp: string): string => {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  const diffInMinutes = Math.floor(diffInMs / (1000 * 60));
  const diffInHours = Math.floor(diffInMinutes / 60);
  const diffInDays = Math.floor(diffInHours / 24);

  if (diffInMinutes < 1) return 'Just now';
  if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
  if (diffInHours < 24) return `${diffInHours}h ago`;
  if (diffInDays < 7) return `${diffInDays}d ago`;
  
  return date.toLocaleDateString();
};

// Format file size
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// Format percentage
export const formatPercentage = (value: number, total: number): string => {
  if (total === 0) return '0%';
  const percentage = (value / total) * 100;
  return `${Math.round(percentage)}%`;
};

// Format quota usage
export const formatQuotaUsage = (used: number, limit: number): string => {
  const percentage = formatPercentage(used, limit);
  return `${used.toLocaleString()} / ${limit.toLocaleString()} (${percentage})`;
};

// Format timezone
export const formatTimezone = (timezone: string): string => {
  try {
    const formatter = new Intl.DateTimeFormat('en', {
      timeZone: timezone,
      timeZoneName: 'short',
    });
    const parts = formatter.formatToParts(new Date());
    const timeZoneName = parts.find(part => part.type === 'timeZoneName')?.value;
    return `${timezone} (${timeZoneName})`;
  } catch {
    return timezone;
  }
};

// Truncate text
export const truncateText = (text: string, maxLength: number): string => {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength - 3) + '...';
};