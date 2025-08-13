import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { 
  SendIcon, 
  UserPlusIcon, 
  MailIcon, 
  XIcon,
  RefreshCwIcon,
  TrashIcon,
} from 'lucide-react';
import { 
  useCurrentTenant, 
  useTenantInvitations, 
  useInviteMember,
  useCancelInvitation,
  useResendInvitation,
} from '../hooks';
import { inviteMemberSchema } from '../utils';
import { 
  TenantRole, 
  TenantInvitation as TenantInvitationType,
  InvitationStatus,
} from '../types';
import {
  formatTenantRole,
  formatInvitationStatus,
  formatRelativeDate,
  getStatusColorClass,
  getRoleColorClass,
} from '../utils';

interface TenantInvitationProps {
  className?: string;
}

interface InviteFormData {
  email: string;
  role: TenantRole;
  message: string;
}

export const TenantInvitation: React.FC<TenantInvitationProps> = ({
  className = '',
}) => {
  const [showInviteForm, setShowInviteForm] = useState(false);
  const { data: currentTenant } = useCurrentTenant();
  const { data: invitations, isLoading } = useTenantInvitations(currentTenant?.id || '');
  const inviteMemberMutation = useInviteMember();
  const cancelInvitationMutation = useCancelInvitation();
  const resendInvitationMutation = useResendInvitation();

  const {
    register,
    handleSubmit,
    formState: { errors },
    reset,
  } = useForm<InviteFormData>({
    resolver: zodResolver(inviteMemberSchema),
    defaultValues: {
      email: '',
      role: TenantRole.MEMBER,
      message: '',
    },
  });

  const onSubmit = async (data: InviteFormData) => {
    if (!currentTenant) return;

    try {
      await inviteMemberMutation.mutateAsync({
        tenantId: currentTenant.id,
        request: {
          email: data.email,
          role: data.role,
          message: data.message || undefined,
        },
      });
      reset();
      setShowInviteForm(false);
    } catch (error) {
      console.error('Failed to invite member:', error);
    }
  };

  const handleCancelInvitation = async (invitation: TenantInvitationType) => {
    if (!currentTenant) return;

    try {
      await cancelInvitationMutation.mutateAsync({
        tenantId: currentTenant.id,
        invitationId: invitation.id,
      });
    } catch (error) {
      console.error('Failed to cancel invitation:', error);
    }
  };

  const handleResendInvitation = async (invitation: TenantInvitationType) => {
    if (!currentTenant) return;

    try {
      await resendInvitationMutation.mutateAsync({
        tenantId: currentTenant.id,
        invitationId: invitation.id,
      });
    } catch (error) {
      console.error('Failed to resend invitation:', error);
    }
  };

  if (!currentTenant) {
    return (
      <div className={`text-center py-8 ${className}`}>
        <p className="text-gray-500">No tenant selected</p>
      </div>
    );
  }

  const pendingInvitations = invitations?.filter(
    inv => inv.status === InvitationStatus.PENDING
  ) || [];

  const expiredInvitations = invitations?.filter(
    inv => inv.status === InvitationStatus.EXPIRED
  ) || [];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Team Invitations</h3>
          <p className="text-sm text-gray-500">
            Invite new members to join your tenant
          </p>
        </div>
        <button
          onClick={() => setShowInviteForm(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          <UserPlusIcon className="w-4 h-4 mr-2" />
          Invite Member
        </button>
      </div>

      {/* Invite Form Modal */}
      {showInviteForm && (
        <div className="fixed inset-0 z-50 overflow-y-auto">
          <div className="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
            <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" />
            
            <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
              <form onSubmit={handleSubmit(onSubmit)}>
                <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-medium text-gray-900">
                      Invite Team Member
                    </h3>
                    <button
                      type="button"
                      onClick={() => setShowInviteForm(false)}
                      className="text-gray-400 hover:text-gray-600"
                    >
                      <XIcon className="w-5 h-5" />
                    </button>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label htmlFor="email" className="block text-sm font-medium text-gray-700">
                        Email Address
                      </label>
                      <input
                        type="email"
                        id="email"
                        {...register('email')}
                        className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                        placeholder="colleague@example.com"
                      />
                      {errors.email && (
                        <p className="mt-1 text-sm text-red-600">{errors.email.message}</p>
                      )}
                    </div>

                    <div>
                      <label htmlFor="role" className="block text-sm font-medium text-gray-700">
                        Role
                      </label>
                      <select
                        id="role"
                        {...register('role')}
                        className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                      >
                        <option value={TenantRole.VIEWER}>Viewer - Read-only access</option>
                        <option value={TenantRole.MEMBER}>Member - Standard access</option>
                        <option value={TenantRole.ADMIN}>Admin - Full management access</option>
                      </select>
                      {errors.role && (
                        <p className="mt-1 text-sm text-red-600">{errors.role.message}</p>
                      )}
                    </div>

                    <div>
                      <label htmlFor="message" className="block text-sm font-medium text-gray-700">
                        Personal Message (Optional)
                      </label>
                      <textarea
                        id="message"
                        rows={3}
                        {...register('message')}
                        className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                        placeholder="Welcome to our team! We're excited to have you join us."
                      />
                      {errors.message && (
                        <p className="mt-1 text-sm text-red-600">{errors.message.message}</p>
                      )}
                    </div>
                  </div>
                </div>

                <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
                  <button
                    type="submit"
                    disabled={inviteMemberMutation.isPending}
                    className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {inviteMemberMutation.isPending ? (
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                    ) : (
                      <SendIcon className="w-4 h-4 mr-2" />
                    )}
                    Send Invitation
                  </button>
                  <button
                    type="button"
                    onClick={() => setShowInviteForm(false)}
                    className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>
        </div>
      )}

      {/* Pending Invitations */}
      {pendingInvitations.length > 0 && (
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h4 className="text-base font-medium text-gray-900 mb-4">
              Pending Invitations ({pendingInvitations.length})
            </h4>
            <div className="space-y-3">
              {pendingInvitations.map((invitation) => (
                <div
                  key={invitation.id}
                  className="flex items-center justify-between p-3 border border-gray-200 rounded-lg"
                >
                  <div className="flex items-center space-x-3">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                        <MailIcon className="w-4 h-4 text-gray-600" />
                      </div>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-900">
                        {invitation.email}
                      </p>
                      <div className="flex items-center space-x-2 text-xs text-gray-500">
                        <span
                          className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getRoleColorClass(
                            invitation.role
                          )}`}
                        >
                          {formatTenantRole(invitation.role)}
                        </span>
                        <span>•</span>
                        <span>Invited {formatRelativeDate(invitation.invitedAt)}</span>
                        <span>•</span>
                        <span>Expires {formatRelativeDate(invitation.expiresAt)}</span>
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button
                      onClick={() => handleResendInvitation(invitation)}
                      disabled={resendInvitationMutation.isPending}
                      className="inline-flex items-center px-2 py-1 border border-gray-300 shadow-sm text-xs font-medium rounded text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <RefreshCwIcon className="w-3 h-3 mr-1" />
                      Resend
                    </button>
                    <button
                      onClick={() => handleCancelInvitation(invitation)}
                      disabled={cancelInvitationMutation.isPending}
                      className="inline-flex items-center px-2 py-1 border border-red-300 shadow-sm text-xs font-medium rounded text-red-700 bg-white hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      <TrashIcon className="w-3 h-3 mr-1" />
                      Cancel
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Expired Invitations */}
      {expiredInvitations.length > 0 && (
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h4 className="text-base font-medium text-gray-900 mb-4">
              Expired Invitations ({expiredInvitations.length})
            </h4>
            <div className="space-y-3">
              {expiredInvitations.map((invitation) => (
                <div
                  key={invitation.id}
                  className="flex items-center justify-between p-3 border border-gray-200 rounded-lg bg-gray-50"
                >
                  <div className="flex items-center space-x-3">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                        <MailIcon className="w-4 h-4 text-gray-400" />
                      </div>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-500">
                        {invitation.email}
                      </p>
                      <div className="flex items-center space-x-2 text-xs text-gray-400">
                        <span
                          className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getStatusColorClass(
                            invitation.status
                          )}`}
                        >
                          {formatInvitationStatus(invitation.status)}
                        </span>
                        <span>•</span>
                        <span>Expired {formatRelativeDate(invitation.expiresAt)}</span>
                      </div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleResendInvitation(invitation)}
                    disabled={resendInvitationMutation.isPending}
                    className="inline-flex items-center px-2 py-1 border border-gray-300 shadow-sm text-xs font-medium rounded text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <RefreshCwIcon className="w-3 h-3 mr-1" />
                    Resend
                  </button>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Empty State */}
      {isLoading ? (
        <div className="bg-white shadow rounded-lg p-6">
          <div className="animate-pulse space-y-4">
            <div className="h-4 bg-gray-200 rounded w-1/4"></div>
            <div className="space-y-3">
              <div className="h-4 bg-gray-200 rounded"></div>
              <div className="h-4 bg-gray-200 rounded w-3/4"></div>
            </div>
          </div>
        </div>
      ) : (
        !pendingInvitations.length && !expiredInvitations.length && (
          <div className="bg-white shadow rounded-lg">
            <div className="px-4 py-5 sm:p-6 text-center">
              <MailIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h4 className="text-base font-medium text-gray-900 mb-2">
                No invitations sent
              </h4>
              <p className="text-sm text-gray-500 mb-4">
                Start building your team by inviting new members to join your tenant.
              </p>
              <button
                onClick={() => setShowInviteForm(true)}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                <UserPlusIcon className="w-4 h-4 mr-2" />
                Send First Invitation
              </button>
            </div>
          </div>
        )
      )}
    </div>
  );
};