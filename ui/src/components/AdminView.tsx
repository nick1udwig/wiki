import React, { useState, useEffect } from 'react';
import { Wiki, WikiRole, DeletedPageSummary, wikiApi } from '../api/wiki';
import { useWikiStore } from '../store/wikiStore';
import './AdminView.css';

interface AdminViewProps {
  wiki: Wiki;
  onClose: () => void;
}

export function AdminView({ wiki, onClose }: AdminViewProps) {
  const { inviteUser, manageMember, loadPages } = useWikiStore();
  const [inviteUsername, setInviteUsername] = useState('');
  const [isInviting, setIsInviting] = useState(false);
  const [inviteError, setInviteError] = useState<string | null>(null);
  const [deletedPages, setDeletedPages] = useState<DeletedPageSummary[]>([]);
  const [showDeletedPages, setShowDeletedPages] = useState(false);
  const [isLoadingDeleted, setIsLoadingDeleted] = useState(false);
  const [restoringPage, setRestoringPage] = useState<string | null>(null);

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

  const loadDeletedPages = async () => {
    setIsLoadingDeleted(true);
    try {
      const deleted = await wikiApi.listDeletedPages(wiki.id);
      setDeletedPages(deleted);
    } catch (error) {
      console.error('Failed to load deleted pages:', error);
    } finally {
      setIsLoadingDeleted(false);
    }
  };

  const handleRestorePage = async (page: DeletedPageSummary) => {
    if (!confirm(`Are you sure you want to restore "${page.path}"?`)) {
      return;
    }

    setRestoringPage(page.deleted_key);
    try {
      await wikiApi.restoreDeletedPage(wiki.id, page.path, page.deleted_key);
      // Remove from deleted list
      setDeletedPages(deletedPages.filter(p => p.deleted_key !== page.deleted_key));
      // Reload pages in the wiki
      await loadPages(wiki.id);
    } catch (error: any) {
      alert(`Failed to restore page: ${error.message || 'Unknown error'}`);
    } finally {
      setRestoringPage(null);
    }
  };

  useEffect(() => {
    if (showDeletedPages && deletedPages.length === 0 && !isLoadingDeleted) {
      loadDeletedPages();
    }
  }, [showDeletedPages]);

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
          
          {/* Deleted Pages Section */}
          <div className="admin-section">
            <div className="section-header">
              <h3>Deleted Pages</h3>
              <button 
                className="toggle-btn"
                onClick={() => setShowDeletedPages(!showDeletedPages)}
                title={showDeletedPages ? "Hide deleted pages" : "Show deleted pages"}
              >
                {showDeletedPages ? '−' : '+'} {showDeletedPages ? 'Hide' : 'Show'}
              </button>
            </div>
            
            {showDeletedPages && (
              <>
                {isLoadingDeleted ? (
                  <div className="loading">Loading deleted pages...</div>
                ) : deletedPages.length === 0 ? (
                  <div className="empty-message">No deleted pages</div>
                ) : (
                  <div className="deleted-pages-list">
                    {deletedPages.map((page) => (
                      <div key={page.deleted_key} className="deleted-page-item">
                        <div className="deleted-page-info">
                          <span className="page-path">{page.path}</span>
                          <span className="deleted-meta">
                            Deleted by {page.deleted_by} on {new Date(page.deleted_at).toLocaleString()}
                          </span>
                        </div>
                        <button
                          className="restore-btn"
                          onClick={() => handleRestorePage(page)}
                          disabled={restoringPage === page.deleted_key}
                        >
                          {restoringPage === page.deleted_key ? 'Restoring...' : 'Restore'}
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}