import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import './WikiList.css';

export function WikiList() {
  const { wikis, loadWikis, createWiki, selectWiki, isLoading, error } = useWikiStore();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newWiki, setNewWiki] = useState({ name: '', description: '', is_public: false });

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

  if (isLoading) {
    return <div className="loading">Loading wikis...</div>;
  }

  return (
    <div className="wiki-list">
      <div className="wiki-list-header">
        <h2>My Wikis</h2>
        <button 
          className="create-wiki-btn"
          onClick={() => setShowCreateForm(true)}
        >
          Create New Wiki
        </button>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      {showCreateForm && (
        <div className="create-wiki-form">
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
              className="wiki-card" 
              onClick={() => selectWiki(wiki)}
            >
              <h3>{wiki.name}</h3>
              <p>{wiki.description}</p>
              <div className="wiki-meta">
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