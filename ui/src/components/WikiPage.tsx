import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import { PageEditor } from './PageEditor';
import { PageViewer } from './PageViewer';
import { PageList } from './PageList';
import { AdminView } from './AdminView';
import { SearchBox } from './SearchBox';
import { wikiApi, WikiRole } from '../api/wiki';
import './WikiPage.css';

export function WikiPage() {
  const { currentWiki, currentPage, selectWiki, createPage, loadPages, loadWiki, loadPage, sidebarCollapsed, setSidebarCollapsed } = useWikiStore();
  const [showAdminView, setShowAdminView] = useState(false);
  const [isEditMode, setIsEditMode] = useState<boolean | null>(null);


  if (!currentWiki) {
    return null;
  }

  const handleCreatePage = async () => {
    // Create a new untitled page
    await createPage('temp', `# New Note\n\nStart writing your content here...`);
    
    // Collapse sidebar on mobile after creating page
    if (isMobile) {
      setSidebarCollapsed(true);
    }
  };

  const handleToggleEditMode = async () => {
    // Refresh wiki data to get latest role before toggling
    if (currentWiki) {
      await loadWiki(currentWiki.id);
    }
    setIsEditMode(!isEditMode);
  };

  const handleOpenAdminView = async () => {
    // Refresh wiki data to get latest members before opening
    if (currentWiki) {
      await loadWiki(currentWiki.id);
    }
    setShowAdminView(true);
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
  
  // Set default edit mode for writers when a page is loaded
  useEffect(() => {
    if (currentPage && isEditMode === null && canWrite) {
      setIsEditMode(true);
    }
  }, [currentPage, canWrite, isEditMode]);
  
  // Check if on mobile
  const isMobile = window.innerWidth <= 768;

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
              onClick={handleToggleEditMode}
            >
              {isEditMode ? '👁 View' : '✏️ Edit'}
            </button>
          )}
          {isAdmin && !isRemoteWiki && (
            <button
              className="admin-btn"
              onClick={handleOpenAdminView}
            >
              Admin Panel
            </button>
          )}
        </div>
      </div>

      
      <div className="wiki-content">
        <aside className={`wiki-sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
          <div className="sidebar-header">
            <h3>Pages</h3>
            {canEdit && (
              <button 
                className="create-page-btn"
                onClick={handleCreatePage}
              >
                + New Page
              </button>
            )}
          </div>
          
          <div className="sidebar-search">
            <SearchBox
              onSearch={(query) => wikiApi.searchPages(currentWiki.id, query)}
              onSelectResult={(path) => {
                loadPage(currentWiki.id, path);
                // Collapse sidebar on mobile after selecting page
                if (isMobile) {
                  setSidebarCollapsed(true);
                }
              }}
              placeholder="Search in this wiki..."
            />
          </div>
          
          
          <PageList onPageSelect={(path) => {
            loadPage(currentWiki.id, path);
            // Collapse sidebar on mobile after selecting page
            if (isMobile) {
              setSidebarCollapsed(true);
            }
          }} />
        </aside>
        <button 
          className="sidebar-toggle"
          onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
          aria-label={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {sidebarCollapsed ? '→' : '←'}
        </button>
        
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
        {/* Mobile overlay */}
        <div 
          className={`sidebar-overlay ${!sidebarCollapsed ? 'active' : ''}`}
          onClick={() => setSidebarCollapsed(true)}
        />
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