import React, { useState } from 'react';
import { 
  UsersIcon, 
  MoreVerticalIcon, 
  TrashIcon,
  CrownIcon,
  ShieldIcon,
  UserIcon,
  EyeIcon,
} from 'lucide-react';
import { 
  useCurrentTenant, 
  useTenantMembers, 
  useUpdateMember,
  useRemoveMember,
} from '../hooks';
import { 
  TenantMember, 
  TenantRole, 
} from '../types';
import {
  formatTenantRole,
  formatMemberStatus,
  formatRelativeDate,
  getStatusColorClass,
  getRoleColorClass,
  generateInitials,
} from '../utils';

interface TenantMembershipProps {
  className?: string;
}

interface MemberActionsProps {
  member: TenantMember;
  onUpdateRole: (memberId: string, role: TenantRole) => void;
  onRemoveMember: (memberId: string) => void;
  canManage: boolean;
}

const MemberActions: React.FC<MemberActionsProps> = ({
  member,
  onUpdateRole,
  onRemoveMember,
  canManage,
}) => {
  const [isOpen, setIsOpen] = useState(false);

  if (!canManage || member.role === TenantRole.OWNER) {
    return null;
  }

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="p-1 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100"
      >
        <MoreVerticalIcon className="w-4 h-4" />
      </button>

      {isOpen && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
          />
          <div className="absolute right-0 z-20 mt-1 w-48 bg-white border border-gray-200 rounded-md shadow-lg">
            <div className="py-1">
              <div className="px-3 py-2 text-xs font-medium text-gray-500 uppercase tracking-wide">
                Change Role
              </div>
              {Object.values(TenantRole)
                .filter(role => role !== TenantRole.OWNER && role !== member.role)
                .map((role) => (
                  <button
                    key={role}
                    onClick={() => {
                      onUpdateRole(member.id, role);
                      setIsOpen(false);
                    }}
                    className="flex items-center w-full px-3 py-2 text-sm text-gray-700 hover:bg-gray-100"
                  >
                    {role === TenantRole.ADMIN && <ShieldIcon className="w-4 h-4 mr-2" />}
                    {role === TenantRole.MEMBER && <UserIcon className="w-4 h-4 mr-2" />}
                    {role === TenantRole.VIEWER && <EyeIcon className="w-4 h-4 mr-2" />}
                    Make {formatTenantRole(role)}
                  </button>
                ))}
              <div className="border-t border-gray-200 my-1" />
              <button
                onClick={() => {
                  onRemoveMember(member.id);
                  setIsOpen(false);
                }}
                className="flex items-center w-full px-3 py-2 text-sm text-red-700 hover:bg-red-50"
              >
                <TrashIcon className="w-4 h-4 mr-2" />
                Remove Member
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export const TenantMembership: React.FC<TenantMembershipProps> = ({
  className = '',
}) => {
  const { data: currentTenant } = useCurrentTenant();
  const { data: members, isLoading } = useTenantMembers(currentTenant?.id || '');
  const updateMemberMutation = useUpdateMember();
  const removeMemberMutation = useRemoveMember();

  const handleUpdateRole = async (memberId: string, role: TenantRole) => {
    if (!currentTenant) return;

    try {
      await updateMemberMutation.mutateAsync({
        tenantId: currentTenant.id,
        memberId,
        request: { role },
      });
    } catch (error) {
      console.error('Failed to update member role:', error);
    }
  };

  const handleRemoveMember = async (memberId: string) => {
    if (!currentTenant) return;

    const confirmed = window.confirm(
      'Are you sure you want to remove this member? This action cannot be undone.'
    );

    if (!confirmed) return;

    try {
      await removeMemberMutation.mutateAsync({
        tenantId: currentTenant.id,
        memberId,
      });
    } catch (error) {
      console.error('Failed to remove member:', error);
    }
  };

  if (!currentTenant) {
    return (
      <div className={`text-center py-8 ${className}`}>
        <p className="text-gray-500">No tenant selected</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className={`bg-white shadow rounded-lg p-6 ${className}`}>
        <div className="animate-pulse space-y-4">
          <div className="h-6 bg-gray-200 rounded w-1/4"></div>
          <div className="space-y-3">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="flex items-center space-x-3">
                <div className="w-10 h-10 bg-gray-200 rounded-full"></div>
                <div className="flex-1 space-y-2">
                  <div className="h-4 bg-gray-200 rounded w-1/3"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/4"></div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (!members || members.length === 0) {
    return (
      <div className={`bg-white shadow rounded-lg ${className}`}>
        <div className="px-4 py-5 sm:p-6 text-center">
          <UsersIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h4 className="text-base font-medium text-gray-900 mb-2">
            No members found
          </h4>
          <p className="text-sm text-gray-500">
            This tenant doesn't have any members yet.
          </p>
        </div>
      </div>
    );
  }

  // Sort members by role priority and then by name
  const sortedMembers = [...members].sort((a, b) => {
    const rolePriority = {
      [TenantRole.OWNER]: 0,
      [TenantRole.ADMIN]: 1,
      [TenantRole.MEMBER]: 2,
      [TenantRole.VIEWER]: 3,
    };
    
    const aPriority = rolePriority[a.role];
    const bPriority = rolePriority[b.role];
    
    if (aPriority !== bPriority) {
      return aPriority - bPriority;
    }
    
    return a.name.localeCompare(b.name);
  });

  // Check if current user can manage members (owner or admin)
  const currentUserMember = members.find(m => m.userId === 'current-user-id'); // This should come from auth context
  const canManageMembers = currentUserMember?.role === TenantRole.OWNER || 
                          currentUserMember?.role === TenantRole.ADMIN;

  const getRoleIcon = (role: TenantRole) => {
    switch (role) {
      case TenantRole.OWNER:
        return <CrownIcon className="w-4 h-4" />;
      case TenantRole.ADMIN:
        return <ShieldIcon className="w-4 h-4" />;
      case TenantRole.MEMBER:
        return <UserIcon className="w-4 h-4" />;
      case TenantRole.VIEWER:
        return <EyeIcon className="w-4 h-4" />;
      default:
        return <UserIcon className="w-4 h-4" />;
    }
  };

  return (
    <div className={`bg-white shadow rounded-lg ${className}`}>
      <div className="px-4 py-5 sm:p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg font-medium text-gray-900">
              Team Members ({members.length})
            </h3>
            <p className="text-sm text-gray-500">
              Manage your team members and their roles
            </p>
          </div>
        </div>

        <div className="space-y-4">
          {sortedMembers.map((member) => (
            <div
              key={member.id}
              className="flex items-center justify-between p-4 border border-gray-200 rounded-lg hover:bg-gray-50"
            >
              <div className="flex items-center space-x-4">
                <div className="flex-shrink-0">
                  <div className="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
                    <span className="text-sm font-medium text-blue-600">
                      {generateInitials(member.name)}
                    </span>
                  </div>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center space-x-2">
                    <p className="text-sm font-medium text-gray-900 truncate">
                      {member.name}
                    </p>
                    {member.role === TenantRole.OWNER && (
                      <span className="text-yellow-500" title="Owner">
                        <CrownIcon className="w-4 h-4" />
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 truncate">
                    {member.email}
                  </p>
                  <div className="flex items-center space-x-3 mt-1">
                    <span
                      className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getRoleColorClass(
                        member.role
                      )}`}
                    >
                      {getRoleIcon(member.role)}
                      <span className="ml-1">{formatTenantRole(member.role)}</span>
                    </span>
                    <span
                      className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getStatusColorClass(
                        member.status
                      )}`}
                    >
                      {formatMemberStatus(member.status)}
                    </span>
                    {member.lastActiveAt && (
                      <span className="text-xs text-gray-400">
                        Last active {formatRelativeDate(member.lastActiveAt)}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                {member.joinedAt && (
                  <div className="text-right">
                    <p className="text-xs text-gray-500">
                      Joined {formatRelativeDate(member.joinedAt)}
                    </p>
                  </div>
                )}
                <MemberActions
                  member={member}
                  onUpdateRole={handleUpdateRole}
                  onRemoveMember={handleRemoveMember}
                  canManage={canManageMembers}
                />
              </div>
            </div>
          ))}
        </div>

        {/* Role Legend */}
        <div className="mt-6 pt-6 border-t border-gray-200">
          <h4 className="text-sm font-medium text-gray-700 mb-3">Role Permissions</h4>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div className="space-y-2">
              <div className="flex items-center space-x-2">
                <CrownIcon className="w-4 h-4 text-yellow-500" />
                <span className="font-medium">Owner</span>
                <span className="text-gray-500">- Full control, billing access</span>
              </div>
              <div className="flex items-center space-x-2">
                <ShieldIcon className="w-4 h-4 text-blue-500" />
                <span className="font-medium">Admin</span>
                <span className="text-gray-500">- Manage members, settings</span>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-center space-x-2">
                <UserIcon className="w-4 h-4 text-green-500" />
                <span className="font-medium">Member</span>
                <span className="text-gray-500">- Standard access</span>
              </div>
              <div className="flex items-center space-x-2">
                <EyeIcon className="w-4 h-4 text-gray-500" />
                <span className="font-medium">Viewer</span>
                <span className="text-gray-500">- Read-only access</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};