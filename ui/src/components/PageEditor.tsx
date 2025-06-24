import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';
import './PageEditor.css';

export function PageEditor() {
  const { currentPage, currentWiki, savePage, loadPage, isLoading } = useWikiStore();
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

  const handleLinkClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const target = e.target as HTMLElement;
    if (target.tagName === 'A') {
      e.preventDefault();
      const href = target.getAttribute('href');
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
    }
  };

  return (
    <div className="page-editor">
      {!isEditing && (
        <button 
          className="edit-overlay-btn"
          onClick={() => setIsEditing(true)}
          title="Edit page"
        >
          ✏️
        </button>
      )}
      
      {isEditing && (
        <div className="editor-toolbar">
          <button onClick={handleSave} disabled={isLoading}>
            {isLoading ? 'Saving...' : 'Save'}
          </button>
          <button onClick={() => setShowPreview(!showPreview)}>
            {showPreview ? 'Edit' : 'Preview'}
          </button>
          <button onClick={handleCancel}>Cancel</button>
        </div>
      )}
      
      <div className="editor-content">
        {isEditing && !showPreview ? (
          <textarea
            className="markdown-editor"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Write your content in Markdown..."
          />
        ) : (
          <div className="markdown-preview" onClick={handleLinkClick}>
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
  if (!text || text.trim() === '') {
    return '<p>This page is empty. Click Edit to add content.</p>';
  }

  // First, handle code blocks to prevent them from being parsed
  const codeBlocks: string[] = [];
  let parsed = text.replace(/```[\s\S]*?```/g, (match) => {
    const index = codeBlocks.length;
    codeBlocks.push(match);
    return `__CODE_BLOCK_${index}__`;
  });

  // Parse headers with IDs for anchor links
  parsed = parsed.replace(/^### (.*$)/gim, (match, heading) => {
    const id = heading.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
    return `<h3 id="${id}">${heading}</h3>`;
  });
  parsed = parsed.replace(/^## (.*$)/gim, (match, heading) => {
    const id = heading.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
    return `<h2 id="${id}">${heading}</h2>`;
  });
  parsed = parsed.replace(/^# (.*$)/gim, (match, heading) => {
    const id = heading.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
    return `<h1 id="${id}">${heading}</h1>`;
  });
  
  // Parse inline formatting
  parsed = parsed.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  parsed = parsed.replace(/\*([^*]+)\*/g, '<em>$1</em>');
  parsed = parsed.replace(/`([^`]+)`/g, '<code>$1</code>');
  
  // Parse links - handle both internal and external
  parsed = parsed.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (match, text, href) => {
    // Check if it's an external link
    if (href.match(/^https?:\/\//) || href.match(/^mailto:/) || href.match(/^tel:/)) {
      return `<a href="${href}" target="_blank" rel="noopener noreferrer">${text}</a>`;
    }
    // Internal link
    return `<a href="${href}">${text}</a>`;
  });
  
  // Parse lists
  parsed = parsed.replace(/^\* (.+)$/gim, '<li>$1</li>');
  parsed = parsed.replace(/(<li>.*<\/li>)/s, '<ul>$1</ul>');
  
  // Split into paragraphs
  const paragraphs = parsed.split(/\n\n+/);
  parsed = paragraphs
    .map(p => {
      // Don't wrap headers, lists, or code blocks in paragraphs
      if (p.match(/^<[hul]|^__CODE_BLOCK_/)) {
        return p;
      }
      // Replace single line breaks with <br> within paragraphs
      return `<p>${p.replace(/\n/g, '<br>')}</p>`;
    })
    .join('\n');

  // Restore code blocks
  codeBlocks.forEach((block, index) => {
    const code = block.replace(/^```(\w*)\n?/, '').replace(/\n?```$/, '');
    const escaped = code.replace(/</g, '&lt;').replace(/>/g, '&gt;');
    parsed = parsed.replace(`__CODE_BLOCK_${index}__`, `<pre><code>${escaped}</code></pre>`);
  });

  return parsed;
}