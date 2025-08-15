import React, { useState } from 'react';
import { 
  Shield, 
  User, 
  Users, 
  Globe,
  Trash2,
  Plus,
  Search,
  X
} from 'lucide-react';
import { Button, Input } from '@adx-core/design-system';
import { FileItem, FilePermissions as FilePermissionsType, FilePermissionLevel } from '../types/file';
import { fileBFFClient } from '../services/fileBFFClient';

interface FilePermissionsProps {
  file: FileItem;
  onClose?: () => void;
  onPermissionsUpdated?: (permissions: FilePermissionsType) => void;
  className?: string;
}

export const FilePermissions: React.FC<FilePermissionsProps> = ({
  file,
  onClose,
  onPermissionsUpdated,
  className = '',
}) => {
  const [permissions, setPermissions] = useState<FilePermissionsType>(file.permissions);
  const [loading, setLoading] = useState(false);
  const [showAddUser, setShowAddUser] = useState(false);
  const [showAddTeam, setShowAddTeam] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [hasChanges, setHasChanges] = useState(false);

  const updatePermissions = async () => {
    setLoading(true);
    try {
      const updatedPermissions = await fileBFFClient.updateFilePermissions(file.id, permissions);
      setPermissions(updatedPermissions);
      setHasChanges(false);
      onPermissionsUpdated?.(updatedPermissions);
    } catch (error) {
      console.error('Failed to update permissions:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleUserPermissionChange = (userId: string, level: FilePermissionLevel) => {
    const newPermissions = {
      ...permissions,
      permissions: {
        ...permissions.permissions,
        [userId]: level,
      },
    };
    setPermissions(newPermissions);
    setHasChanges(true);
  };

  const handleTeamPermissionChange = (teamId: string, level: FilePermissionLevel) => {
    const newPermissions = {
      ...permissions,
      teamPermissions: {
        ...permissions.teamPermissions,
        [teamId]: level,
      },
    };
    setPermissions(newPermissions);
    setHasChanges(true);
  };

  const handleRemoveUserPermission = (userId: string) => {
    const newPermissions = { ...permissions.permissions };
    delete newPermissions[userId];
    setPermissions({
      ...permissions,
      permissions: newPermissions,
    });
    setHasChanges(true);
  };

  const handleRemoveTeamPermission = (teamId: string) => {
    const newTeamPermissions = { ...permissions.teamPermissions };
    delete newTeamPermissions[teamId];
    setPermissions({
      ...permissions,
      teamPermissions: newTeamPermissions,
    });
    setHasChanges(true);
  };

  const handlePublicAccessChange = (enabled: boolean) => {
    setPermissions({
      ...permissions,
      publicAccess: enabled ? {
        enabled: true,
        level: 'read',
        downloadAllowed: true,
      } : undefined,
    });
    setHasChanges(true);
  };

  const handleInheritanceChange = (inherit: boolean) => {
    setPermissions({
      ...permissions,
      inheritFromParent: inherit,
    });
    setHasChanges(true);
  };

  // Removed unused functions to fix TypeScript warnings

  // Filter users and teams based on search
  const filteredUsers = Object.entries(permissions.permissions).filter(([userId]) =>
    userId.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredTeams = Object.entries(permissions.teamPermissions || {}).filter(([teamId]) =>
    teamId.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className={`file-permissions ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <Shield className="w-6 h-6 text-blue-600" />
          <div>
            <h2 className="text-lg font-semibold text-gray-900">File Permissions</h2>
            <p className="text-sm text-gray-500">{file.name}</p>
          </div>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <X className="w-6 h-6" />
          </button>
        )}
      </div>

      {/* Owner */}
      <div className="mb-6">
        <h3 className="text-sm font-medium text-gray-900 mb-3">Owner</h3>
        <div className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
          <div className="flex items-center space-x-3">
            <User className="w-5 h-5 text-gray-500" />
            <div>
              <p className="text-sm font-medium text-gray-900">{permissions.owner}</p>
              <p className="text-xs text-gray-500">File owner</p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Shield className="w-4 h-4 text-red-500" />
            <span className="text-sm font-medium text-red-600">Owner</span>
          </div>
        </div>
      </div>

      {/* Inheritance */}
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-medium text-gray-900">Inherit from Parent</h3>
            <p className="text-xs text-gray-500">Use permissions from parent folder</p>
          </div>
          <button
            onClick={() => handleInheritanceChange(!permissions.inheritFromParent)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              permissions.inheritFromParent ? 'bg-blue-600' : 'bg-gray-200'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                permissions.inheritFromParent ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>
      </div>

      {/* Public Access */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h3 className="text-sm font-medium text-gray-900">Public Access</h3>
            <p className="text-xs text-gray-500">Allow anyone with the link to access</p>
          </div>
          <button
            onClick={() => handlePublicAccessChange(!permissions.publicAccess?.enabled)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              permissions.publicAccess?.enabled ? 'bg-blue-600' : 'bg-gray-200'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                permissions.publicAccess?.enabled ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {permissions.publicAccess?.enabled && (
          <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
            <div className="flex items-center space-x-2 mb-2">
              <Globe className="w-4 h-4 text-yellow-600" />
              <span className="text-sm font-medium text-yellow-800">Public Access Enabled</span>
            </div>
            <div className="flex items-center space-x-4 text-xs">
              <label className="flex items-center space-x-2">
                <input
                  type="radio"
                  name="publicLevel"
                  checked={permissions.publicAccess.level === 'read'}
                  onChange={() => setPermissions({
                    ...permissions,
                    publicAccess: { ...permissions.publicAccess!, level: 'read' }
                  })}
                />
                <span>View only</span>
              </label>
              <label className="flex items-center space-x-2">
                <input
                  type="radio"
                  name="publicLevel"
                  checked={permissions.publicAccess.level === 'write'}
                  onChange={() => setPermissions({
                    ...permissions,
                    publicAccess: { ...permissions.publicAccess!, level: 'write' }
                  })}
                />
                <span>Can edit</span>
              </label>
            </div>
          </div>
        )}
      </div>

      {/* Search */}
      <div className="mb-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <Input
            placeholder="Search users and teams..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
      </div>

      {/* User Permissions */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-medium text-gray-900">User Permissions</h3>
          <Button
            size="sm"
            variant="outline"
            onClick={() => setShowAddUser(true)}
          >
            <Plus className="w-4 h-4 mr-1" />
            Add User
          </Button>
        </div>

        <div className="space-y-2">
          {filteredUsers.length === 0 ? (
            <p className="text-sm text-gray-500 text-center py-4">
              {searchQuery ? 'No users found' : 'No user permissions set'}
            </p>
          ) : (
            filteredUsers.map(([userId, level]) => (
              <PermissionItem
                key={userId}
                id={userId}
                type="user"
                level={level}
                onLevelChange={(newLevel) => handleUserPermissionChange(userId, newLevel)}
                onRemove={() => handleRemoveUserPermission(userId)}
              />
            ))
          )}
        </div>
      </div>

      {/* Team Permissions */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-medium text-gray-900">Team Permissions</h3>
          <Button
            size="sm"
            variant="outline"
            onClick={() => setShowAddTeam(true)}
          >
            <Plus className="w-4 h-4 mr-1" />
            Add Team
          </Button>
        </div>

        <div className="space-y-2">
          {filteredTeams.length === 0 ? (
            <p className="text-sm text-gray-500 text-center py-4">
              {searchQuery ? 'No teams found' : 'No team permissions set'}
            </p>
          ) : (
            filteredTeams.map(([teamId, level]) => (
              <PermissionItem
                key={teamId}
                id={teamId}
                type="team"
                level={level}
                onLevelChange={(newLevel) => handleTeamPermissionChange(teamId, newLevel)}
                onRemove={() => handleRemoveTeamPermission(teamId)}
              />
            ))
          )}
        </div>
      </div>

      {/* Actions */}
      {hasChanges && (
        <div className="flex justify-end space-x-3 pt-4 border-t border-gray-200">
          <Button
            variant="outline"
            onClick={() => {
              setPermissions(file.permissions);
              setHasChanges(false);
            }}
          >
            Cancel
          </Button>
          <Button
            onClick={updatePermissions}
            disabled={loading}
          >
            {loading ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      )}

      {/* Add User Modal */}
      {showAddUser && (
        <AddPermissionModal
          type="user"
          onClose={() => setShowAddUser(false)}
          onAdd={(id, level) => {
            handleUserPermissionChange(id, level);
            setShowAddUser(false);
          }}
        />
      )}

      {/* Add Team Modal */}
      {showAddTeam && (
        <AddPermissionModal
          type="team"
          onClose={() => setShowAddTeam(false)}
          onAdd={(id, level) => {
            handleTeamPermissionChange(id, level);
            setShowAddTeam(false);
          }}
        />
      )}
    </div>
  );
};

interface PermissionItemProps {
  id: string;
  type: 'user' | 'team';
  level: FilePermissionLevel;
  onLevelChange: (level: FilePermissionLevel) => void;
  onRemove: () => void;
}

const PermissionItem: React.FC<PermissionItemProps> = ({
  id,
  type,
  level,
  onLevelChange,
  onRemove,
}) => {
  const getIcon = () => {
    return type === 'user' ? 
      <User className="w-4 h-4 text-gray-500" /> : 
      <Users className="w-4 h-4 text-gray-500" />;
  };

  const getPermissionColor = (permLevel: FilePermissionLevel) => {
    switch (permLevel) {
      case 'admin':
        return 'text-red-600 bg-red-50 border-red-200';
      case 'write':
        return 'text-blue-600 bg-blue-50 border-blue-200';
      case 'read':
        return 'text-green-600 bg-green-50 border-green-200';
      default:
        return 'text-gray-600 bg-gray-50 border-gray-200';
    }
  };

  return (
    <div className="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
      <div className="flex items-center space-x-3">
        {getIcon()}
        <div>
          <p className="text-sm font-medium text-gray-900">{id}</p>
          <p className="text-xs text-gray-500 capitalize">{type}</p>
        </div>
      </div>

      <div className="flex items-center space-x-2">
        <select
          value={level}
          onChange={(e) => onLevelChange(e.target.value as FilePermissionLevel)}
          className={`text-xs px-2 py-1 rounded border ${getPermissionColor(level)}`}
        >
          <option value="read">View</option>
          <option value="write">Edit</option>
          <option value="admin">Admin</option>
        </select>

        <button
          onClick={onRemove}
          className="p-1 text-gray-400 hover:text-red-600"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};

interface AddPermissionModalProps {
  type: 'user' | 'team';
  onClose: () => void;
  onAdd: (id: string, level: FilePermissionLevel) => void;
}

const AddPermissionModal: React.FC<AddPermissionModalProps> = ({
  type,
  onClose,
  onAdd,
}) => {
  const [id, setId] = useState('');
  const [level, setLevel] = useState<FilePermissionLevel>('read');

  const handleAdd = () => {
    if (id.trim()) {
      onAdd(id.trim(), level);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">Add {type === 'user' ? 'User' : 'Team'}</h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {type === 'user' ? 'User ID or Email' : 'Team Name'}
            </label>
            <Input
              type="text"
              value={id}
              onChange={(e) => setId(e.target.value)}
              placeholder={type === 'user' ? 'user@example.com' : 'Team name'}
              autoFocus
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Permission Level
            </label>
            <select
              value={level}
              onChange={(e) => setLevel(e.target.value as FilePermissionLevel)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="read">View Only</option>
              <option value="write">Can Edit</option>
              <option value="admin">Full Access</option>
            </select>
          </div>

          <div className="flex justify-end space-x-3 pt-4">
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button
              onClick={handleAdd}
              disabled={!id.trim()}
            >
              Add {type === 'user' ? 'User' : 'Team'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};