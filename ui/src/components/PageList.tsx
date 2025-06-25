import React from 'react';
import { useWikiStore } from '../store/wikiStore';
import './PageList.css';

interface PageListProps {
  onPageSelect?: (path: string) => void;
}

export function PageList({ onPageSelect }: PageListProps) {
  const { pages, currentWiki, currentPage, loadPage } = useWikiStore();

  if (!currentWiki) return null;

  const handlePageClick = (path: string) => {
    if (onPageSelect) {
      onPageSelect(path);
    } else {
      loadPage(currentWiki.id, path);
    }
  };

  return (
    <div className="page-list">
      {pages.length === 0 ? (
        <div className="empty-pages">
          <p>No pages yet</p>
        </div>
      ) : (
        <ul>
          {pages.map((page) => (
            <li 
              key={page.path}
              className={currentPage?.path === page.path ? 'active' : ''}
              onClick={() => handlePageClick(page.path)}
            >
              <div className="page-item">
                <span className="page-name">{page.path}</span>
                <span className="page-meta">
                  Updated {new Date(page.updated_at).toLocaleDateString()}
                </span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}