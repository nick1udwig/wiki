import React from 'react';
import ReactMarkdown from 'react-markdown';
import { useWikiStore } from '../store/wikiStore';
import './PageViewer.css';

export function PageViewer() {
  const { currentPage } = useWikiStore();

  if (!currentPage) {
    return (
      <div className="page-viewer empty">
        <p>Select a page from the sidebar to start reading.</p>
      </div>
    );
  }

  return (
    <div className="page-viewer">
      <div className="page-viewer-content">
        <ReactMarkdown>
          {currentPage.content || ''}
        </ReactMarkdown>
      </div>
      
      <div className="page-footer">
        <div className="page-meta">
          <span>Last updated by {currentPage.updated_by}</span>
          <span>{new Date(currentPage.updated_at).toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}