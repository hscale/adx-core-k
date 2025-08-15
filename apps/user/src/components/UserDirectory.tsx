import React, { useState, useEffect } from 'react';
import { Button, Card, Input } from '@adx-core/design-system';
import { Search, Filter, UserPlus, Mail, Phone, MoreVertical, Eye, Edit3, UserX, UserCheck, X } from 'lucide-react';
import { useUserSearch, useInviteUser, useDeactivateUser, useReactivateUser } from '../hooks';
import { formatUserDisplayName, formatUserInitials, formatLastLogin } from '../utils';
import { UserSearchFilters, User, CreateUserRequest } from '../types';

interface UserDirectoryProps {
  canManageUsers?: boolean;
}

export const UserDirectory: React.FC<UserDirectoryProps> = ({ canManageUsers = false }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [filters, setFilters] = useState<UserSearchFilters>({});
  const [showFilters, setShowFilters] = useState(false);
  const [showInviteModal, setShowInviteModal] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [showUserActions, setShowUserActions] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const pageSize = 20;

  const searchFilters = {
    ...filters,
    query: searchQuery || undefined,
  };

  const { data: searchResult, isLoading, refetch } = useUserSearch(searchFilters, page, pageSize);
  const inviteUserMutation = useInviteUser();
  const deactivateUserMutation = useDeactivateUser();
  const reactivateUserMutation = useReactivateUser();

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      setPage(1);
      refetch();
    }, 300);

    return () => clearTimeout(timer);
  }, [searchQuery, filters, refetch]);

  const handleSearch = (query: string) => {
    setSearchQuery(query);
  };

  const handleFilterChange = (newFilters: Partial<UserSearchFilters>) => {
    setFilters({ ...filters, ...newFilters });
  };

  const handleInviteUser = async (invitation: CreateUserRequest) => {
    try {
      await inviteUserMutation.mutateAsync(invitation);
      setShowInviteModal(false);
      refetch();
    } catch (error) {
      console.error('Failed to invite user:', error);
    }
  };

  const handleDeactivateUser = async (userId: string) => {
    try {
      await deactivateUserMutation.mutateAsync(userId);
      setShowUserActions(null);
      refetch();
    } catch (error) {
      console.error('Failed to deactivate user:', error);
    }
  };

  const handleReactivateUser = async (userId: string) => {
    try {
      await reactivateUserMutation.mutateAsync(userId);
      setShowUserActions(null);
      refetch();
    } catch (error) {
      console.error('Failed to reactivate user:', error);
    }
  };

  const availableRoles = ['admin', 'user', 'manager', 'viewer'];
  const availableDepartments = ['Engineering', 'Sales', 'Marketing', 'Support', 'HR', 'Finance'];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">User Directory</h2>
          <p className="text-gray-600">
            {searchResult?.total || 0} users found
          </p>
        </div>
        {canManageUsers && (
          <Button onClick={() => setShowInviteModal(true)}>
            <UserPlus className="w-4 h-4 mr-2" />
            Invite User
          </Button>
        )}
      </div>

      {/* Search and Filters */}
      <Card className="p-4">
        <div className="flex items-center space-x-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
            <Input
              placeholder="Search users by name, email, or role..."
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
              className="pl-10"
            />
          </div>
          <Button
            variant="outline"
            onClick={() => setShowFilters(!showFilters)}
          >
            <Filter className="w-4 h-4 mr-2" />
            Filters
          </Button>
        </div>

        {/* Advanced Filters */}
        {showFilters && (
          <div className="mt-4 pt-4 border-t border-gray-200">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Roles</label>
                <select
                  multiple
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  value={filters.roles || []}
                  onChange={(e) => {
                    const selectedRoles = Array.from(e.target.selectedOptions, option => option.value);
                    handleFilterChange({ roles: selectedRoles });
                  }}
                >
                  {availableRoles.map(role => (
                    <option key={role} value={role}>{role}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Departments</label>
                <select
                  multiple
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  value={filters.departments || []}
                  onChange={(e) => {
                    const selectedDepartments = Array.from(e.target.selectedOptions, option => option.value);
                    handleFilterChange({ departments: selectedDepartments });
                  }}
                >
                  {availableDepartments.map(dept => (
                    <option key={dept} value={dept}>{dept}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Status</label>
                <select
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  value={filters.isActive === undefined ? 'all' : filters.isActive ? 'active' : 'inactive'}
                  onChange={(e) => {
                    const value = e.target.value;
                    handleFilterChange({
                      isActive: value === 'all' ? undefined : value === 'active'
                    });
                  }}
                >
                  <option value="all">All Users</option>
                  <option value="active">Active Only</option>
                  <option value="inactive">Inactive Only</option>
                </select>
              </div>
            </div>
            <div className="mt-4 flex space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  setFilters({});
                  setSearchQuery('');
                }}
              >
                Clear Filters
              </Button>
            </div>
          </div>
        )}
      </Card>

      {/* User List */}
      <Card className="overflow-hidden">
        {isLoading ? (
          <div className="flex items-center justify-center p-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
        ) : searchResult?.users.length === 0 ? (
          <div className="text-center py-12">
            <Search className="w-12 h-12 text-gray-300 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 mb-2">No users found</h3>
            <p className="text-gray-600">Try adjusting your search criteria or filters.</p>
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {searchResult?.users.map((user) => (
              <div key={user.id} className="p-6 hover:bg-gray-50">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="w-12 h-12 bg-blue-600 rounded-full flex items-center justify-center text-white font-semibold">
                      {user.avatar ? (
                        <img src={user.avatar} alt={formatUserDisplayName(user)} className="w-12 h-12 rounded-full" />
                      ) : (
                        formatUserInitials(user)
                      )}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <h3 className="text-lg font-medium text-gray-900">
                          {formatUserDisplayName(user)}
                        </h3>
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                          user.isActive 
                            ? 'bg-green-100 text-green-800' 
                            : 'bg-red-100 text-red-800'
                        }`}>
                          {user.isActive ? 'Active' : 'Inactive'}
                        </span>
                      </div>
                      <div className="flex items-center space-x-4 mt-1 text-sm text-gray-600">
                        <div className="flex items-center space-x-1">
                          <Mail className="w-4 h-4" />
                          <span>{user.email}</span>
                        </div>
                        {user.phone && (
                          <div className="flex items-center space-x-1">
                            <Phone className="w-4 h-4" />
                            <span>{user.phone}</span>
                          </div>
                        )}
                      </div>
                      <div className="flex items-center space-x-4 mt-1 text-sm text-gray-500">
                        <span>Roles: {user.roles.join(', ')}</span>
                        <span>Last login: {formatLastLogin(user.lastLoginAt)}</span>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setSelectedUser(user)}
                    >
                      <Eye className="w-4 h-4 mr-1" />
                      View
                    </Button>
                    
                    {canManageUsers && (
                      <div className="relative">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => setShowUserActions(showUserActions === user.id ? null : user.id)}
                        >
                          <MoreVertical className="w-4 h-4" />
                        </Button>
                        
                        {showUserActions === user.id && (
                          <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg z-10 border border-gray-200">
                            <div className="py-1">
                              <button
                                className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                                onClick={() => {
                                  setSelectedUser(user);
                                  setShowUserActions(null);
                                }}
                              >
                                <Edit3 className="w-4 h-4 mr-2" />
                                Edit User
                              </button>
                              {user.isActive ? (
                                <button
                                  className="flex items-center w-full px-4 py-2 text-sm text-red-700 hover:bg-red-50"
                                  onClick={() => handleDeactivateUser(user.id)}
                                >
                                  <UserX className="w-4 h-4 mr-2" />
                                  Deactivate
                                </button>
                              ) : (
                                <button
                                  className="flex items-center w-full px-4 py-2 text-sm text-green-700 hover:bg-green-50"
                                  onClick={() => handleReactivateUser(user.id)}
                                >
                                  <UserCheck className="w-4 h-4 mr-2" />
                                  Reactivate
                                </button>
                              )}
                            </div>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Pagination */}
        {searchResult && searchResult.hasMore && (
          <div className="px-6 py-4 border-t border-gray-200">
            <div className="flex items-center justify-between">
              <p className="text-sm text-gray-700">
                Showing {((page - 1) * pageSize) + 1} to {Math.min(page * pageSize, searchResult.total)} of {searchResult.total} users
              </p>
              <div className="flex space-x-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(page - 1)}
                  disabled={page === 1}
                >
                  Previous
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(page + 1)}
                  disabled={!searchResult.hasMore}
                >
                  Next
                </Button>
              </div>
            </div>
          </div>
        )}
      </Card>

      {/* Invite User Modal */}
      {showInviteModal && (
        <InviteUserModal
          onClose={() => setShowInviteModal(false)}
          onInvite={handleInviteUser}
          availableRoles={availableRoles}
        />
      )}

      {/* User Detail Modal */}
      {selectedUser && (
        <UserDetailModal
          user={selectedUser}
          onClose={() => setSelectedUser(null)}
          canEdit={canManageUsers}
        />
      )}
    </div>
  );
};

// Invite User Modal Component
interface InviteUserModalProps {
  onClose: () => void;
  onInvite: (invitation: CreateUserRequest) => void;
  availableRoles: string[];
}

const InviteUserModal: React.FC<InviteUserModalProps> = ({ onClose, onInvite, availableRoles }) => {
  const [form, setForm] = useState<CreateUserRequest>({
    email: '',
    firstName: '',
    lastName: '',
    roles: [],
    tenantId: localStorage.getItem('current_tenant_id') || '',
    sendInvitation: true,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onInvite(form);
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Invite New User</h3>
        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            label="Email"
            type="email"
            value={form.email}
            onChange={(e) => setForm({ ...form, email: e.target.value })}
            required
          />
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="First Name"
              value={form.firstName}
              onChange={(e) => setForm({ ...form, firstName: e.target.value })}
              required
            />
            <Input
              label="Last Name"
              value={form.lastName}
              onChange={(e) => setForm({ ...form, lastName: e.target.value })}
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Roles</label>
            <select
              multiple
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              value={form.roles}
              onChange={(e) => {
                const selectedRoles = Array.from(e.target.selectedOptions, option => option.value);
                setForm({ ...form, roles: selectedRoles });
              }}
              required
            >
              {availableRoles.map(role => (
                <option key={role} value={role}>{role}</option>
              ))}
            </select>
          </div>
          <label className="flex items-center space-x-2">
            <input
              type="checkbox"
              checked={form.sendInvitation}
              onChange={(e) => setForm({ ...form, sendInvitation: e.target.checked })}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700">Send invitation email</span>
          </label>
          <div className="flex space-x-2">
            <Button type="submit" className="flex-1">
              Send Invitation
            </Button>
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

// User Detail Modal Component
interface UserDetailModalProps {
  user: User;
  onClose: () => void;
  canEdit: boolean;
}

const UserDetailModal: React.FC<UserDetailModalProps> = ({ user, onClose }) => {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[80vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-gray-900">User Details</h3>
          <Button variant="outline" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </div>
        
        <div className="space-y-6">
          <div className="flex items-center space-x-4">
            <div className="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center text-white text-xl font-semibold">
              {user.avatar ? (
                <img src={user.avatar} alt={formatUserDisplayName(user)} className="w-16 h-16 rounded-full" />
              ) : (
                formatUserInitials(user)
              )}
            </div>
            <div>
              <h4 className="text-xl font-semibold text-gray-900">{formatUserDisplayName(user)}</h4>
              <p className="text-gray-600">{user.email}</p>
              <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                user.isActive 
                  ? 'bg-green-100 text-green-800' 
                  : 'bg-red-100 text-red-800'
              }`}>
                {user.isActive ? 'Active' : 'Inactive'}
              </span>
            </div>
          </div>
          
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-3">
              <div>
                <span className="text-sm font-medium text-gray-700">Phone:</span>
                <p className="text-sm text-gray-600">{user.phone || 'Not provided'}</p>
              </div>
              <div>
                <span className="text-sm font-medium text-gray-700">Timezone:</span>
                <p className="text-sm text-gray-600">{user.timezone}</p>
              </div>
              <div>
                <span className="text-sm font-medium text-gray-700">Language:</span>
                <p className="text-sm text-gray-600">{user.language}</p>
              </div>
            </div>
            <div className="space-y-3">
              <div>
                <span className="text-sm font-medium text-gray-700">Roles:</span>
                <p className="text-sm text-gray-600">{user.roles.join(', ')}</p>
              </div>
              <div>
                <span className="text-sm font-medium text-gray-700">Last Login:</span>
                <p className="text-sm text-gray-600">{formatLastLogin(user.lastLoginAt)}</p>
              </div>
              <div>
                <span className="text-sm font-medium text-gray-700">Created:</span>
                <p className="text-sm text-gray-600">{new Date(user.createdAt).toLocaleDateString()}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default UserDirectory;