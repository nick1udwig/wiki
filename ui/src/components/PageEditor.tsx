import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import './PageEditor.css';

export function PageEditor() {
  const { currentPage, savePage, isLoading } = useWikiStore();
  const [content, setContent] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [showPreview, setShowPreview] = useState(false);

  useEffect(() => {
    if (currentPage) {
      setContent(currentPage.content || '');
    }
  }, [currentPage]);

  if (!currentPage) return null;

  const handleSave = async () => {
    await savePage(content);
    setIsEditing(false);
  };

  const handleCancel = () => {
    setContent(currentPage.content || '');
    setIsEditing(false);
  };

  return (
    <div className="page-editor">
      <div className="editor-header">
        <h2>{currentPage.path}</h2>
        <div className="editor-toolbar">
          {!isEditing && (
            <button onClick={() => setIsEditing(true)}>Edit</button>
          )}
          {isEditing && (
            <>
              <button onClick={handleSave} disabled={isLoading}>
                {isLoading ? 'Saving...' : 'Save'}
              </button>
              <button onClick={() => setShowPreview(!showPreview)}>
                {showPreview ? 'Edit' : 'Preview'}
              </button>
              <button onClick={handleCancel}>Cancel</button>
            </>
          )}
        </div>
      </div>
      
      <div className="editor-content">
        {isEditing && !showPreview ? (
          <textarea
            className="markdown-editor"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Write your content in Markdown..."
          />
        ) : (
          <div className="markdown-preview">
            <div dangerouslySetInnerHTML={{ __html: parseMarkdown(content) }} />
          </div>
        )}
      </div>
      
      <div className="page-meta">
        <span>Last updated: {new Date(currentPage.updated_at).toLocaleString()}</span>
        <span>By: {currentPage.updated_by}</span>
      </div>
    </div>
  );
}

// Simple markdown parser (in production, use a library like marked or remark)
function parseMarkdown(text: string): string {
  return text
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/\*\*(.*)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*)\*/g, '<em>$1</em>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br>')
    .split('</p><p>')
    .map(p => `<p>${p}</p>`)
    .join('');
}