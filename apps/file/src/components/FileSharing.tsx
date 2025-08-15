import React, { useState, useEffect } from 'react';
import { 
  Share2, 
  Link, 
  Mail, 
  Users, 
  Globe, 
  Copy, 
  Check, 
  Calendar, 
  Lock,
  Eye,
  Trash2,
  Plus
} from 'lucide-react';
import { Button, Input } from '@adx-core/design-system';
import { FileItem, FileShare, FilePermissionLevel } from '../types/file';
import { fileBFFClient } from '../services/fileBFFClient';
import { copyToClipboard } from '../utils/fileUtils';

interface FileSharingProps {
  file: FileItem;
  onClose?: () => void;
  className?: string;
}

export const FileSharing: React.FC<FileSharingProps> = ({
  file,
  onClose,
  className = '',
}) => {
  const [shares, setShares] = useState<FileShare[]>([]);
  const [loading, setLoading] = useState(true);
  const [showNewShare, setShowNewShare] = useState(false);
  const [copiedShareId, setCopiedShareId] = useState<string | null>(null);

  // Load existing shares
  useEffect(() => {
    const loadShares = async () => {
      try {
        const fileShares = await fileBFFClient.getFileShares(file.id);
        setShares(fileShares);
      } catch (error) {
        console.error('Failed to load shares:', error);
      } finally {
        setLoading(false);
      }
    };

    loadShares();
  }, [file.id]);

  const handleCopyLink = async (share: FileShare) => {
    if (share.shareUrl) {
      const success = await copyToClipboard(share.shareUrl);
      if (success) {
        setCopiedShareId(share.id);
        setTimeout(() => setCopiedShareId(null), 2000);
      }
    }
  };

  const handleRevokeShare = async (shareId: string) => {
    if (confirm('Are you sure you want to revoke this share?')) {
      try {
        await fileBFFClient.revokeShare(shareId);
        setShares(shares.filter(s => s.id !== shareId));
      } catch (error) {
        console.error('Failed to revoke share:', error);
      }
    }
  };

  // Removed unused functions to fix TypeScript warnings

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className={`file-sharing ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <Share2 className="w-6 h-6 text-blue-600" />
          <div>
            <h2 className="text-lg font-semibold text-gray-900">Share File</h2>
            <p className="text-sm text-gray-500">{file.name}</p>
          </div>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <Plus className="w-6 h-6 rotate-45" />
          </button>
        )}
      </div>

      {/* Quick Share Actions */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Button
          variant="outline"
          className="flex items-center justify-center space-x-2 p-4"
          onClick={() => setShowNewShare(true)}
        >
          <Link className="w-5 h-5" />
          <span>Create Link</span>
        </Button>
        
        <Button
          variant="outline"
          className="flex items-center justify-center space-x-2 p-4"
          onClick={() => setShowNewShare(true)}
        >
          <Mail className="w-5 h-5" />
          <span>Email Share</span>
        </Button>
        
        <Button
          variant="outline"
          className="flex items-center justify-center space-x-2 p-4"
          onClick={() => setShowNewShare(true)}
        >
          <Users className="w-5 h-5" />
          <span>Team Share</span>
        </Button>
      </div>

      {/* Existing Shares */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-gray-900">Active Shares</h3>
          <Button
            size="sm"
            onClick={() => setShowNewShare(true)}
          >
            <Plus className="w-4 h-4 mr-1" />
            New Share
          </Button>
        </div>

        {shares.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <Share2 className="w-12 h-12 mx-auto mb-4 text-gray-300" />
            <p>No active shares</p>
            <p className="text-sm">Create a share to collaborate with others</p>
          </div>
        ) : (
          <div className="space-y-3">
            {shares.map((share) => (
              <ShareItem
                key={share.id}
                share={share}
                onCopyLink={() => handleCopyLink(share)}
                onRevoke={() => handleRevokeShare(share.id)}
                copied={copiedShareId === share.id}
              />
            ))}
          </div>
        )}
      </div>

      {/* New Share Modal */}
      {showNewShare && (
        <NewShareModal
          file={file}
          onClose={() => setShowNewShare(false)}
          onShareCreated={(newShare) => {
            setShares([...shares, newShare]);
            setShowNewShare(false);
          }}
        />
      )}
    </div>
  );
};

interface ShareItemProps {
  share: FileShare;
  onCopyLink: () => void;
  onRevoke: () => void;
  copied: boolean;
}

const ShareItem: React.FC<ShareItemProps> = ({
  share,
  onCopyLink,
  onRevoke,
  copied,
}) => {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getShareTypeIcon = (shareType: string) => {
    switch (shareType) {
      case 'public':
        return <Globe className="w-4 h-4 text-blue-500" />;
      case 'email':
        return <Mail className="w-4 h-4 text-green-500" />;
      case 'team':
        return <Users className="w-4 h-4 text-purple-500" />;
      default:
        return <Link className="w-4 h-4 text-gray-500" />;
    }
  };

  return (
    <div className="border border-gray-200 rounded-lg p-4">
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-3">
          {getShareTypeIcon(share.shareType)}
          
          <div className="flex-1 min-w-0">
            <div className="flex items-center space-x-2">
              <p className="text-sm font-medium text-gray-900">
                {share.shareType === 'public' ? 'Public Link' : 
                 share.shareType === 'email' ? share.sharedWith : 
                 share.shareType === 'team' ? `Team: ${share.sharedWith}` : 
                 'Private Share'}
              </p>
              
              <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                share.permissions === 'admin' ? 'bg-red-100 text-red-800' :
                share.permissions === 'write' ? 'bg-blue-100 text-blue-800' :
                'bg-green-100 text-green-800'
              }`}>
                {share.permissions}
              </span>
              
              {share.password && (
                <Lock className="w-3 h-3 text-gray-400" />
              )}
              
              {!share.downloadAllowed && (
                <Eye className="w-3 h-3 text-gray-400" />
              )}
            </div>
            
            <div className="flex items-center space-x-4 mt-1 text-xs text-gray-500">
              <span>Created {formatDate(share.createdAt)}</span>
              {share.expiresAt && (
                <span className="flex items-center space-x-1">
                  <Calendar className="w-3 h-3" />
                  <span>Expires {formatDate(share.expiresAt)}</span>
                </span>
              )}
              <span>{share.accessCount} access{share.accessCount !== 1 ? 'es' : ''}</span>
            </div>
            
            {share.message && (
              <p className="text-xs text-gray-600 mt-2 italic">
                "{share.message}"
              </p>
            )}
          </div>
        </div>
        
        <div className="flex items-center space-x-2">
          {share.shareUrl && (
            <Button
              size="sm"
              variant="outline"
              onClick={onCopyLink}
              className="flex items-center space-x-1"
            >
              {copied ? (
                <Check className="w-3 h-3 text-green-500" />
              ) : (
                <Copy className="w-3 h-3" />
              )}
              <span className="text-xs">
                {copied ? 'Copied!' : 'Copy'}
              </span>
            </Button>
          )}
          
          <Button
            size="sm"
            variant="outline"
            onClick={onRevoke}
            className="text-red-600 hover:text-red-700"
          >
            <Trash2 className="w-3 h-3" />
          </Button>
        </div>
      </div>
    </div>
  );
};

interface NewShareModalProps {
  file: FileItem;
  onClose: () => void;
  onShareCreated: (share: FileShare) => void;
}

const NewShareModal: React.FC<NewShareModalProps> = ({
  file,
  onClose,
  onShareCreated,
}) => {
  const [shareType, setShareType] = useState<'public' | 'email' | 'team'>('public');
  const [permissions, setPermissions] = useState<FilePermissionLevel>('read');
  const [sharedWith, setSharedWith] = useState('');
  const [expiresAt, setExpiresAt] = useState('');
  const [password, setPassword] = useState('');
  const [downloadAllowed, setDownloadAllowed] = useState(true);
  const [message, setMessage] = useState('');
  const [creating, setCreating] = useState(false);

  const handleCreateShare = async () => {
    setCreating(true);
    
    try {
      const shareSettings = {
        shareType,
        permissions,
        sharedWith: shareType !== 'public' ? sharedWith : undefined,
        expiresAt: expiresAt || undefined,
        password: password ? true : undefined, // Convert to boolean
        downloadAllowed,
        message: message || undefined,
      };
      
      const newShare = await fileBFFClient.shareFile(file.id, shareSettings);
      onShareCreated(newShare);
    } catch (error) {
      console.error('Failed to create share:', error);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">Create New Share</h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <Plus className="w-6 h-6 rotate-45" />
          </button>
        </div>

        <div className="space-y-4">
          {/* Share Type */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Share Type
            </label>
            <div className="grid grid-cols-3 gap-2">
              {(['public', 'email', 'team'] as const).map((type) => (
                <button
                  key={type}
                  onClick={() => setShareType(type)}
                  className={`p-2 text-xs rounded border ${
                    shareType === type
                      ? 'border-blue-500 bg-blue-50 text-blue-700'
                      : 'border-gray-300 text-gray-700 hover:bg-gray-50'
                  }`}
                >
                  {type === 'public' && <Globe className="w-4 h-4 mx-auto mb-1" />}
                  {type === 'email' && <Mail className="w-4 h-4 mx-auto mb-1" />}
                  {type === 'team' && <Users className="w-4 h-4 mx-auto mb-1" />}
                  <div className="capitalize">{type}</div>
                </button>
              ))}
            </div>
          </div>

          {/* Share With */}
          {shareType !== 'public' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {shareType === 'email' ? 'Email Address' : 'Team Name'}
              </label>
              <Input
                type={shareType === 'email' ? 'email' : 'text'}
                value={sharedWith}
                onChange={(e) => setSharedWith(e.target.value)}
                placeholder={shareType === 'email' ? 'user@example.com' : 'Team name'}
              />
            </div>
          )}

          {/* Permissions */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Permissions
            </label>
            <select
              value={permissions}
              onChange={(e) => setPermissions(e.target.value as FilePermissionLevel)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="read">View Only</option>
              <option value="write">Can Edit</option>
              <option value="admin">Full Access</option>
            </select>
          </div>

          {/* Options */}
          <div className="space-y-3">
            <div className="flex items-center">
              <input
                type="checkbox"
                id="downloadAllowed"
                checked={downloadAllowed}
                onChange={(e) => setDownloadAllowed(e.target.checked)}
                className="mr-2"
              />
              <label htmlFor="downloadAllowed" className="text-sm text-gray-700">
                Allow downloads
              </label>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Expiration Date (optional)
              </label>
              <Input
                type="date"
                value={expiresAt}
                onChange={(e) => setExpiresAt(e.target.value)}
                min={new Date().toISOString().split('T')[0]}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Password Protection (optional)
              </label>
              <Input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter password"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Message (optional)
              </label>
              <textarea
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Add a message for recipients"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                rows={3}
              />
            </div>
          </div>

          {/* Actions */}
          <div className="flex justify-end space-x-3 pt-4">
            <Button variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateShare}
              disabled={creating || (shareType !== 'public' && !sharedWith.trim())}
            >
              {creating ? 'Creating...' : 'Create Share'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};