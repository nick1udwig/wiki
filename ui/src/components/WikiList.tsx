import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import { SearchBox } from './SearchBox';
import { wikiApi, WikiInfo } from '../api/wiki';
import './WikiList.css';

export function WikiList() {
  const { wikis, loadWikis, createWiki, selectWiki, joinWiki, loadPage, isLoading, error, setSidebarCollapsed } = useWikiStore();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showJoinForm, setShowJoinForm] = useState(false);
  const [newWiki, setNewWiki] = useState({ name: '', description: '', is_public: false });
  const [joinUsername, setJoinUsername] = useState('');
  const [foundWikis, setFoundWikis] = useState<WikiInfo[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);

  useEffect(() => {
    loadWikis();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newWiki.name.trim()) return;
    
    await createWiki(newWiki.name, newWiki.description, newWiki.is_public);
    setShowCreateForm(false);
    setNewWiki({ name: '', description: '', is_public: false });
  };

  const handleSearchUser = async () => {
    if (!joinUsername.trim()) return;
    
    setSearchLoading(true);
    try {
      const wikis = await wikiApi.findWikisByUser(joinUsername);
      setFoundWikis(wikis);
    } catch (err) {
      console.error('Failed to find wikis:', err);
      setFoundWikis([]);
    } finally {
      setSearchLoading(false);
    }
  };

  const handleJoinWiki = async (wikiId: string, nodeId?: string) => {
    await joinWiki(wikiId, nodeId);
    setShowJoinForm(false);
    setJoinUsername('');
    setFoundWikis([]);
  };

  if (isLoading) {
    return <div className="loading">Loading wikis...</div>;
  }

  return (
    <div className="wiki-list">
      <div className="wiki-list-header">
        <div className="wiki-list-controls">
          <button 
            className="create-wiki-btn icon-btn"
            onClick={() => setShowCreateForm(true)}
            title="Create New Wiki"
          >
            +
          </button>
          <button 
            className="join-wiki-btn icon-btn"
            onClick={() => setShowJoinForm(true)}
            title="Join Wiki"
          >
            🤝
          </button>
        </div>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      {showCreateForm && (
        <div className="create-wiki-modal" onClick={() => setShowCreateForm(false)}>
          <div className="create-wiki-form" onClick={(e) => e.stopPropagation()}>
            <form onSubmit={handleCreate}>
              <h3>Create New Wiki</h3>
            <div className="form-group">
              <label htmlFor="wiki-name">Name</label>
              <input
                id="wiki-name"
                type="text"
                placeholder="Wiki Name"
                value={newWiki.name}
                onChange={(e) => setNewWiki({ ...newWiki, name: e.target.value })}
                required
                autoFocus
              />
            </div>
            <div className="form-group">
              <label htmlFor="wiki-description">Description</label>
              <textarea
                id="wiki-description"
                placeholder="Description"
                value={newWiki.description}
                onChange={(e) => setNewWiki({ ...newWiki, description: e.target.value })}
                rows={3}
              />
            </div>
            <div className="form-group checkbox">
              <label>
                <input
                  type="checkbox"
                  checked={newWiki.is_public}
                  onChange={(e) => setNewWiki({ ...newWiki, is_public: e.target.checked })}
                />
                Public Wiki
              </label>
            </div>
            <div className="form-actions">
              <button type="submit">Create</button>
              <button type="button" onClick={() => setShowCreateForm(false)}>Cancel</button>
            </div>
          </form>
          </div>
        </div>
      )}

      {showJoinForm && (
        <div className="create-wiki-form">
          <h3>Join a Wiki</h3>
          <div className="form-group">
            <label htmlFor="username">Search by Username</label>
            <input
              id="username"
              type="text"
              value={joinUsername}
              onChange={(e) => setJoinUsername(e.target.value)}
              placeholder="Enter username to find their public wikis"
            />
          </div>
          <div className="form-actions">
            <button onClick={handleSearchUser} disabled={searchLoading}>
              {searchLoading ? 'Searching...' : 'Search'}
            </button>
            <button type="button" onClick={() => {
              setShowJoinForm(false);
              setJoinUsername('');
              setFoundWikis([]);
            }}>Cancel</button>
          </div>
          
          {foundWikis.length > 0 && (
            <div className="found-wikis">
              <h4>Found Wikis:</h4>
              {foundWikis.map((wiki) => (
                <div key={wiki.id} className="found-wiki-item">
                  <div>
                    <strong>{wiki.name}</strong>
                    <p>{wiki.description}</p>
                    <small>{wiki.member_count} members</small>
                  </div>
                  <button onClick={() => handleJoinWiki(wiki.id, wiki.node_id)}>
                    Join
                  </button>
                </div>
              ))}
            </div>
          )}
          
          {searchLoading === false && joinUsername && foundWikis.length === 0 && (
            <p style={{ marginTop: '1rem', color: 'var(--text-secondary)' }}>
              No public wikis found for user "{joinUsername}"
            </p>
          )}
        </div>
      )}
      
      <div className="wiki-grid">
        {wikis.length === 0 ? (
          <div className="empty-state">
            <p>No wikis yet. Create your first wiki to get started!</p>
          </div>
        ) : (
          wikis.map((wiki) => (
            <div 
              key={wiki.id} 
              className="wiki-item" 
              onClick={() => {
                setSidebarCollapsed(false); // Open sidebar when selecting wiki from home
                selectWiki(wiki);
              }}
            >
              <div className="wiki-item-primary">
                <h3 className="wiki-name">{wiki.name}</h3>
                <span className="wiki-owner">{wiki.created_by}</span>
              </div>
              <div className="wiki-item-secondary">
                <span className="wiki-badge">{wiki.is_public ? 'Public' : 'Private'}</span>
                <span className="wiki-members">
                  {Object.keys(wiki.members).length} members
                </span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}