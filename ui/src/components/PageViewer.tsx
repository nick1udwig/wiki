import React, { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { useWikiStore } from '../store/wikiStore';
import { PageHistory } from './PageHistory';
import './PageViewer.css';

export function PageViewer() {
  const { currentPage, currentWiki, loadPage, deletePage } = useWikiStore();
  const [showHistory, setShowHistory] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  if (!currentPage) {
    return (
      <div className="page-viewer empty">
        <p>Select a page from the sidebar to start reading.</p>
      </div>
    );
  }

  const handleLinkClick = (href: string | undefined) => {
    if (!href) return;

    // Check if it's an internal link (no protocol or starts with #)
    if (!href.match(/^https?:\/\//) && !href.match(/^mailto:/) && !href.match(/^tel:/)) {
      // Internal link
      const [pagePath, anchor] = href.split('#');
      if (currentWiki && pagePath) {
        loadPage(currentWiki.id, pagePath);
      }
      if (anchor) {
        // Scroll to anchor after page loads
        setTimeout(() => {
          const element = document.getElementById(anchor);
          if (element) {
            element.scrollIntoView({ behavior: 'smooth' });
          }
        }, 100);
      }
    } else {
      // External link - open in new tab
      window.open(href, '_blank', 'noopener,noreferrer');
    }
  };

  const handleDelete = async () => {
    if (!currentWiki || !currentPage) return;
    
    if (confirm(`Are you sure you want to delete "${currentPage.path}"? This page can be restored from the admin panel.`)) {
      setIsDeleting(true);
      try {
        await deletePage(currentWiki.id, currentPage.path);
      } catch (error) {
        // Error is already shown by the store
        setIsDeleting(false);
      }
    }
  };

  return (
    <div className="page-viewer">
      <div className="page-viewer-content">
        <ReactMarkdown
          components={{
            a: ({ href, children }) => (
              <a
                href={href}
                onClick={(e) => {
                  e.preventDefault();
                  handleLinkClick(href);
                }}
              >
                {children}
              </a>
            ),
            h1: ({ children }) => {
              const id = String(children).toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
              return <h1 id={id}>{children}</h1>;
            },
            h2: ({ children }) => {
              const id = String(children).toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
              return <h2 id={id}>{children}</h2>;
            },
            h3: ({ children }) => {
              const id = String(children).toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
              return <h3 id={id}>{children}</h3>;
            }
          }}
        >
          {currentPage.content || ''}
        </ReactMarkdown>
      </div>
      
      <div className="page-footer">
        <div className="page-meta">
          <span>Last updated by {currentPage.updated_by}</span>
          <span>{new Date(currentPage.updated_at).toLocaleString()}</span>
          <div className="page-actions">
            <button 
              className="history-btn"
              onClick={() => setShowHistory(true)}
              title="View version history"
            >
              <span className="history-icon">🕐</span>
            </button>
            <button 
              className="delete-btn"
              onClick={handleDelete}
              disabled={isDeleting}
              title="Delete page"
            >
              {isDeleting ? '...' : '🗑️'}
            </button>
          </div>
        </div>
      </div>
      
      {showHistory && currentWiki && (
        <PageHistory
          wiki_id={currentWiki.id}
          path={currentPage.path}
          onClose={() => setShowHistory(false)}
        />
      )}
    </div>
  );
}