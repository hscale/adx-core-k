import { SubscriptionTier, TenantStatus, TenantRole, MemberStatus, InvitationStatus } from '../types';

// Format subscription tier for display
export const formatSubscriptionTier = (tier: SubscriptionTier): string => {
  switch (tier) {
    case SubscriptionTier.FREE:
      return 'Free';
    case SubscriptionTier.PROFESSIONAL:
      return 'Professional';
    case SubscriptionTier.ENTERPRISE:
      return 'Enterprise';
    default:
      return tier;
  }
};

// Format tenant status for display
export const formatTenantStatus = (status: TenantStatus): string => {
  switch (status) {
    case TenantStatus.ACTIVE:
      return 'Active';
    case TenantStatus.SUSPENDED:
      return 'Suspended';
    case TenantStatus.PENDING:
      return 'Pending';
    case TenantStatus.CANCELLED:
      return 'Cancelled';
    default:
      return status;
  }
};

// Format tenant role for display
export const formatTenantRole = (role: TenantRole): string => {
  switch (role) {
    case TenantRole.OWNER:
      return 'Owner';
    case TenantRole.ADMIN:
      return 'Admin';
    case TenantRole.MEMBER:
      return 'Member';
    case TenantRole.VIEWER:
      return 'Viewer';
    default:
      return role;
  }
};

// Format member status for display
export const formatMemberStatus = (status: MemberStatus): string => {
  switch (status) {
    case MemberStatus.ACTIVE:
      return 'Active';
    case MemberStatus.INVITED:
      return 'Invited';
    case MemberStatus.SUSPENDED:
      return 'Suspended';
    default:
      return status;
  }
};

// Format invitation status for display
export const formatInvitationStatus = (status: InvitationStatus): string => {
  switch (status) {
    case InvitationStatus.PENDING:
      return 'Pending';
    case InvitationStatus.ACCEPTED:
      return 'Accepted';
    case InvitationStatus.EXPIRED:
      return 'Expired';
    case InvitationStatus.CANCELLED:
      return 'Cancelled';
    default:
      return status;
  }
};

// Format file size
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

// Format storage quota
export const formatStorageQuota = (currentGB: number, maxGB: number): string => {
  const percentage = maxGB > 0 ? (currentGB / maxGB) * 100 : 0;
  return `${currentGB.toFixed(1)} GB / ${maxGB} GB (${percentage.toFixed(1)}%)`;
};

// Format user count
export const formatUserCount = (current: number, max: number): string => {
  return `${current} / ${max} users`;
};

// Format API calls
export const formatApiCalls = (current: number, max: number): string => {
  if (current >= 1000000) {
    return `${(current / 1000000).toFixed(1)}M / ${(max / 1000000).toFixed(1)}M calls`;
  } else if (current >= 1000) {
    return `${(current / 1000).toFixed(1)}K / ${(max / 1000).toFixed(1)}K calls`;
  }
  return `${current} / ${max} calls`;
};

// Format date relative to now
export const formatRelativeDate = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);
  
  if (diffInSeconds < 60) {
    return 'Just now';
  } else if (diffInSeconds < 3600) {
    const minutes = Math.floor(diffInSeconds / 60);
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  } else if (diffInSeconds < 86400) {
    const hours = Math.floor(diffInSeconds / 3600);
    return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  } else if (diffInSeconds < 2592000) {
    const days = Math.floor(diffInSeconds / 86400);
    return `${days} day${days > 1 ? 's' : ''} ago`;
  } else {
    return date.toLocaleDateString();
  }
};

// Format absolute date
export const formatAbsoluteDate = (dateString: string): string => {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
};

// Get status color class
export const getStatusColorClass = (status: TenantStatus | MemberStatus | InvitationStatus): string => {
  switch (status) {
    case TenantStatus.ACTIVE:
    case MemberStatus.ACTIVE:
    case InvitationStatus.ACCEPTED:
      return 'text-green-600 bg-green-100';
    case TenantStatus.SUSPENDED:
    case MemberStatus.SUSPENDED:
      return 'text-red-600 bg-red-100';
    case TenantStatus.PENDING:
    case MemberStatus.INVITED:
    case InvitationStatus.PENDING:
      return 'text-yellow-600 bg-yellow-100';
    case TenantStatus.CANCELLED:
    case InvitationStatus.CANCELLED:
    case InvitationStatus.EXPIRED:
      return 'text-gray-600 bg-gray-100';
    default:
      return 'text-gray-600 bg-gray-100';
  }
};

// Get role color class
export const getRoleColorClass = (role: TenantRole): string => {
  switch (role) {
    case TenantRole.OWNER:
      return 'text-purple-600 bg-purple-100';
    case TenantRole.ADMIN:
      return 'text-blue-600 bg-blue-100';
    case TenantRole.MEMBER:
      return 'text-green-600 bg-green-100';
    case TenantRole.VIEWER:
      return 'text-gray-600 bg-gray-100';
    default:
      return 'text-gray-600 bg-gray-100';
  }
};

// Get subscription tier color class
export const getSubscriptionTierColorClass = (tier: SubscriptionTier): string => {
  switch (tier) {
    case SubscriptionTier.FREE:
      return 'text-gray-600 bg-gray-100';
    case SubscriptionTier.PROFESSIONAL:
      return 'text-blue-600 bg-blue-100';
    case SubscriptionTier.ENTERPRISE:
      return 'text-purple-600 bg-purple-100';
    default:
      return 'text-gray-600 bg-gray-100';
  }
};

// Truncate text
export const truncateText = (text: string, maxLength: number): string => {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + '...';
};

// Generate initials from name
export const generateInitials = (name: string): string => {
  return name
    .split(' ')
    .map(word => word.charAt(0).toUpperCase())
    .slice(0, 2)
    .join('');
};