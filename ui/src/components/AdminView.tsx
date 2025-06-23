import React, { useState } from 'react';
import { Wiki, WikiRole } from '../api/wiki';
import { useWikiStore } from '../store/wikiStore';
import './AdminView.css';

interface AdminViewProps {
  wiki: Wiki;
  onClose: () => void;
}

export function AdminView({ wiki, onClose }: AdminViewProps) {
  const { inviteUser, manageMember } = useWikiStore();
  const [inviteUsername, setInviteUsername] = useState('');
  const [isInviting, setIsInviting] = useState(false);
  const [inviteError, setInviteError] = useState<string | null>(null);

  const handleInviteUser = async () => {
    if (!inviteUsername.trim()) return;
    
    setIsInviting(true);
    setInviteError(null);
    
    try {
      await inviteUser(wiki.id, inviteUsername.trim());
      setInviteUsername('');
    } catch (error: any) {
      setInviteError(error.message || 'Failed to send invite');
    } finally {
      setIsInviting(false);
    }
  };

  const handleChangeRole = async (memberId: string, newRole: WikiRole) => {
    try {
      await manageMember(wiki.id, memberId, 'update', newRole);
    } catch (error) {
      console.error('Failed to update member role:', error);
    }
  };

  const handleRemoveMember = async (memberId: string) => {
    if (confirm(`Are you sure you want to remove this member?`)) {
      try {
        await manageMember(wiki.id, memberId, 'remove');
      } catch (error) {
        console.error('Failed to remove member:', error);
      }
    }
  };

  // Get current user's node ID
  const nodeId = window.our?.node || '';
  const currentUserRole = wiki.members[nodeId];
  const isCurrentUserSuperAdmin = currentUserRole === 'SuperAdmin';

  // Sort members by role priority
  const memberEntries = Object.entries(wiki.members).sort(([, roleA], [, roleB]) => {
    const rolePriority: Record<WikiRole, number> = {
      'SuperAdmin': 0,
      'Admin': 1,
      'Writer': 2,
      'Reader': 3,
    };
    return rolePriority[roleA] - rolePriority[roleB];
  });

  return (
    <div className="admin-view-overlay" onClick={onClose}>
      <div className="admin-view-container" onClick={(e) => e.stopPropagation()}>
        <div className="admin-view-header">
          <h2>Admin Panel - {wiki.name}</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="admin-view-content">
          {/* Invite User Section */}
          <div className="admin-section">
            <h3>Invite User</h3>
            <div className="invite-form">
              <input
                type="text"
                placeholder="Enter username (e.g., alice.os)"
                value={inviteUsername}
                onChange={(e) => setInviteUsername(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleInviteUser()}
              />
              <button
                onClick={handleInviteUser}
                disabled={isInviting || !inviteUsername.trim()}
              >
                {isInviting ? 'Sending...' : 'Send Invite'}
              </button>
            </div>
            {inviteError && (
              <div className="error-message">{inviteError}</div>
            )}
          </div>

          {/* Members Management Section */}
          <div className="admin-section">
            <h3>Members ({memberEntries.length})</h3>
            <div className="members-list">
              {memberEntries.map(([memberId, role]) => (
                <div key={memberId} className="member-row">
                  <div className="member-info">
                    <span className="member-id">{memberId}</span>
                    <span className={`role-badge role-${role.toLowerCase()}`}>
                      {role}
                    </span>
                  </div>
                  
                  {memberId !== nodeId && ( // Can't modify own role
                    <div className="member-actions">
                      <select
                        value={role}
                        onChange={(e) => handleChangeRole(memberId, e.target.value as WikiRole)}
                        disabled={!isCurrentUserSuperAdmin && role === 'Admin'}
                      >
                        <option value="Reader">Reader</option>
                        <option value="Writer">Writer</option>
                        <option value="Admin">Admin</option>
                        {isCurrentUserSuperAdmin && (
                          <option value="SuperAdmin">Super Admin</option>
                        )}
                      </select>
                      
                      <button
                        className="remove-btn"
                        onClick={() => handleRemoveMember(memberId)}
                        disabled={role === 'SuperAdmin' || (!isCurrentUserSuperAdmin && role === 'Admin')}
                        title="Remove member"
                      >
                        Remove
                      </button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Wiki Settings Section */}
          <div className="admin-section">
            <h3>Wiki Settings</h3>
            <div className="wiki-settings">
              <div className="setting-row">
                <label>Wiki ID:</label>
                <code>{wiki.id}</code>
              </div>
              <div className="setting-row">
                <label>Created by:</label>
                <span>{wiki.created_by}</span>
              </div>
              <div className="setting-row">
                <label>Created at:</label>
                <span>{new Date(wiki.created_at).toLocaleString()}</span>
              </div>
              <div className="setting-row">
                <label>Visibility:</label>
                <span className={`visibility-badge ${wiki.is_public ? 'public' : 'private'}`}>
                  {wiki.is_public ? 'Public' : 'Private'}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}