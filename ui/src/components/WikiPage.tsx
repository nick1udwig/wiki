import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import { PageEditor } from './PageEditor';
import { PageViewer } from './PageViewer';
import { PageList } from './PageList';
import { AdminView } from './AdminView';
import { wikiApi, WikiRole } from '../api/wiki';
import './WikiPage.css';

export function WikiPage() {
  const { currentWiki, currentPage, selectWiki, createPage } = useWikiStore();
  const [showCreatePage, setShowCreatePage] = useState(false);
  const [newPagePath, setNewPagePath] = useState('');
  const [showAdminView, setShowAdminView] = useState(false);
  const [isEditMode, setIsEditMode] = useState(false);

  if (!currentWiki) {
    return null;
  }

  const handleCreatePage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newPagePath.trim()) return;
    
    await createPage(newPagePath, `# ${newPagePath}\n\nStart writing your content here...`);
    setShowCreatePage(false);
    setNewPagePath('');
  };


  // Check user permissions
  const nodeId = window.our?.node || '';
  const currentUserRole = currentWiki.members[nodeId];
  const isAdmin = currentUserRole === 'Admin' || currentUserRole === 'SuperAdmin';
  const canWrite = currentUserRole === 'Writer' || currentUserRole === 'Admin' || currentUserRole === 'SuperAdmin';
  
  // A wiki is "remote" if it has @ in the ID, indicating it's accessed via another node
  const isRemoteWiki = currentWiki.id.includes('@');
  
  // Writers can now edit remote wikis through P2P
  const canEdit = canWrite;

  return (
    <div className="wiki-page">
      <div className="wiki-header">
        <button 
          className="back-button"
          onClick={() => selectWiki(null)}
        >
          ← Back to Wikis
        </button>
        <h1>{currentWiki.name}</h1>
        <div className="wiki-info">
          <span className="wiki-badge">{currentWiki.is_public ? 'Public' : 'Private'}</span>
          {currentUserRole && !isAdmin && (
            <span className={`role-badge role-${currentUserRole.toLowerCase()}`}>
              {currentUserRole}
            </span>
          )}
          {canEdit && currentPage && (
            <button
              className="mode-toggle-btn"
              onClick={() => setIsEditMode(!isEditMode)}
            >
              {isEditMode ? '👁 View' : '✏️ Edit'}
            </button>
          )}
          {isAdmin && !isRemoteWiki && (
            <button
              className="admin-btn"
              onClick={() => setShowAdminView(true)}
            >
              Admin Panel
            </button>
          )}
        </div>
      </div>

      
      <div className="wiki-content">
        <aside className="wiki-sidebar">
          <div className="sidebar-header">
            <h3>Pages</h3>
            {canEdit && (
              <button 
                className="create-page-btn"
                onClick={() => setShowCreatePage(true)}
              >
                + New Page
              </button>
            )}
          </div>
          
          {showCreatePage && (
            <form onSubmit={handleCreatePage} className="create-page-form">
              <input
                type="text"
                placeholder="Page name"
                value={newPagePath}
                onChange={(e) => setNewPagePath(e.target.value)}
                autoFocus
              />
              <div className="form-actions">
                <button type="submit">Create</button>
                <button type="button" onClick={() => setShowCreatePage(false)}>Cancel</button>
              </div>
            </form>
          )}
          
          <PageList />
        </aside>
        
        <main className="wiki-main">
          {currentPage ? (
            isEditMode && canEdit ? (
              <PageEditor />
            ) : (
              <PageViewer />
            )
          ) : (
            <div className="empty-page">
              <p>Select a page from the sidebar or create a new one.</p>
            </div>
          )}
        </main>
      </div>
      
      {showAdminView && (
        <AdminView
          wiki={currentWiki}
          onClose={() => setShowAdminView(false)}
        />
      )}
    </div>
  );
}