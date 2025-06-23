import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import { PageEditor } from './PageEditor';
import { PageList } from './PageList';
import './WikiPage.css';

export function WikiPage() {
  const { currentWiki, currentPage, selectWiki, createPage } = useWikiStore();
  const [showCreatePage, setShowCreatePage] = useState(false);
  const [newPagePath, setNewPagePath] = useState('');

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
        </div>
      </div>
      
      <div className="wiki-content">
        <aside className="wiki-sidebar">
          <div className="sidebar-header">
            <h3>Pages</h3>
            <button 
              className="create-page-btn"
              onClick={() => setShowCreatePage(true)}
            >
              + New Page
            </button>
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
            <PageEditor />
          ) : (
            <div className="empty-page">
              <p>Select a page from the sidebar or create a new one.</p>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}