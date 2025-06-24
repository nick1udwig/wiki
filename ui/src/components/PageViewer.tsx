import React from 'react';
import ReactMarkdown from 'react-markdown';
import { useWikiStore } from '../store/wikiStore';
import './PageViewer.css';

export function PageViewer() {
  const { currentPage, currentWiki, loadPage } = useWikiStore();

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
        </div>
      </div>
    </div>
  );
}