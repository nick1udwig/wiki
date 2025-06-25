import React, { useState, useEffect } from 'react';
import { PageHistory as PageHistoryType, PageVersion, VersionDiff, wikiApi } from '../api/wiki';
import { useWikiStore } from '../store/wikiStore';
import './PageHistory.css';

interface PageHistoryProps {
  wiki_id: string;
  path: string;
  onClose: () => void;
}

export function PageHistory({ wiki_id, path, onClose }: PageHistoryProps) {
  const [history, setHistory] = useState<PageHistoryType | null>(null);
  const [selectedVersions, setSelectedVersions] = useState<[string?, string?]>([]);
  const [viewMode, setViewMode] = useState<'list' | 'view' | 'diff'>('list');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewContent, setViewContent] = useState<string>('');
  const [versionDiff, setVersionDiff] = useState<VersionDiff | null>(null);
  const [isLoadingDiff, setIsLoadingDiff] = useState(false);

  useEffect(() => {
    loadHistory();
  }, [wiki_id, path]);

  useEffect(() => {
    if (viewMode === 'diff' && selectedVersions.length === 2 && selectedVersions[0] && selectedVersions[1]) {
      loadDiff(selectedVersions[0], selectedVersions[1]);
    }
  }, [selectedVersions, viewMode]);

  const loadHistory = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const pageHistory = await wikiApi.getPageHistory(wiki_id, path);
      setHistory(pageHistory);
    } catch (err: any) {
      setError(err.message || 'Failed to load page history');
    } finally {
      setIsLoading(false);
    }
  };

  const loadDiff = async (version1Id: string, version2Id: string) => {
    setIsLoadingDiff(true);
    try {
      const diff = await wikiApi.getVersionDiff(wiki_id, path, version1Id, version2Id);
      setVersionDiff(diff);
    } catch (err: any) {
      setError(err.message || 'Failed to load version diff');
    } finally {
      setIsLoadingDiff(false);
    }
  };


  const selectVersion = (versionId: string) => {
    if (viewMode === 'view') {
      setSelectedVersions([versionId]);
      const version = history?.versions.find(v => v.version_id === versionId);
      if (version) {
        setViewContent(version.content); // Content is already decoded
      }
    } else if (viewMode === 'diff') {
      const [first] = selectedVersions;
      if (!first) {
        setSelectedVersions([versionId]);
      } else if (first !== versionId) {
        setSelectedVersions([first, versionId]);
      } else {
        setSelectedVersions([]);
      }
    }
  };

  const isVersionSelected = (versionId: string) => {
    return selectedVersions.includes(versionId);
  };

  const renderDiff = () => {
    if (selectedVersions.length !== 2 || !history) {
      return <div className="diff-placeholder">Select two versions to compare</div>;
    }

    if (isLoadingDiff) {
      return <div className="loading">Loading diff...</div>;
    }

    if (!versionDiff) {
      return <div className="diff-placeholder">Select two versions to compare</div>;
    }

    const [v1Id, v2Id] = selectedVersions;
    const v1 = history.versions.find(v => v.version_id === v1Id);
    const v2 = history.versions.find(v => v.version_id === v2Id);

    if (!v1 || !v2) return null;

    return (
      <div className="diff-view">
        <div className="diff-header">
          <div className="diff-version">
            <h4>Version from {new Date(v1.updated_at).toLocaleString()}</h4>
            <span>by {v1.updated_by}</span>
          </div>
          <div className="diff-version">
            <h4>Version from {new Date(v2.updated_at).toLocaleString()}</h4>
            <span>by {v2.updated_by}</span>
          </div>
        </div>
        <div className="diff-content">
          {versionDiff.diff_lines.map((line, index) => (
            <div
              key={index}
              className={`diff-line diff-line-${line.line_type.toLowerCase()}`}
            >
              <span className="line-number line-number-old">
                {line.line_number_old || ''}
              </span>
              <span className="line-number line-number-new">
                {line.line_number_new || ''}
              </span>
              <span className="line-content">
                {line.line_type === 'Added' && '+ '}
                {line.line_type === 'Removed' && '- '}
                {line.line_type === 'Unchanged' && '  '}
                {line.content}
              </span>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderContent = () => {
    if (isLoading) {
      return <div className="loading">Loading history...</div>;
    }

    if (error) {
      return <div className="error-message">{error}</div>;
    }

    if (!history || history.versions.length === 0) {
      return <div className="empty-history">No version history available</div>;
    }

    switch (viewMode) {
      case 'list':
        return (
          <div className="history-list">
            {history.versions.map((version, index) => (
              <div
                key={version.version_id}
                className={`history-item ${version.version_id === history.current_version_id ? 'current' : ''}`}
              >
                <div className="version-info">
                  <div className="version-header">
                    <span className="version-number">Version {history.versions.length - index}</span>
                    {version.version_id === history.current_version_id && (
                      <span className="current-badge">Current</span>
                    )}
                  </div>
                  <div className="version-meta">
                    <span className="version-author">by {version.updated_by}</span>
                    <span className="version-date">{new Date(version.updated_at).toLocaleString()}</span>
                  </div>
                  {version.commit_message && (
                    <div className="version-message">{version.commit_message}</div>
                  )}
                </div>
                <div className="version-actions">
                  <button
                    className="view-btn"
                    onClick={() => {
                      setViewMode('view');
                      selectVersion(version.version_id);
                    }}
                  >
                    View
                  </button>
                </div>
              </div>
            ))}
          </div>
        );

      case 'view':
        return (
          <div className="version-view">
            <div className="view-header">
              <button onClick={() => setViewMode('list')}>← Back to list</button>
              <button
                onClick={() => setViewMode('diff')}
                className="diff-mode-btn"
              >
                Compare versions
              </button>
            </div>
            <div className="version-list-sidebar">
              {history.versions.map((version, index) => (
                <div
                  key={version.version_id}
                  className={`version-item ${isVersionSelected(version.version_id) ? 'selected' : ''}`}
                  onClick={() => selectVersion(version.version_id)}
                >
                  <span>Version {history.versions.length - index}</span>
                  <span className="version-date">{new Date(version.updated_at).toLocaleDateString()}</span>
                </div>
              ))}
            </div>
            <div className="version-content">
              {selectedVersions[0] ? (
                <div>
                  <h3>Version content</h3>
                  <pre>{viewContent}</pre>
                </div>
              ) : (
                <div className="select-version-prompt">Select a version to view</div>
              )}
            </div>
          </div>
        );

      case 'diff':
        return (
          <div className="diff-mode">
            <div className="view-header">
              <button onClick={() => setViewMode('list')}>← Back to list</button>
              <button
                onClick={() => setViewMode('view')}
                className="view-mode-btn"
              >
                View single version
              </button>
            </div>
            <div className="version-list-sidebar">
              {history.versions.map((version, index) => (
                <div
                  key={version.version_id}
                  className={`version-item ${isVersionSelected(version.version_id) ? 'selected' : ''}`}
                  onClick={() => selectVersion(version.version_id)}
                >
                  <span>Version {history.versions.length - index}</span>
                  <span className="version-date">{new Date(version.updated_at).toLocaleDateString()}</span>
                </div>
              ))}
            </div>
            <div className="diff-container">
              {renderDiff()}
            </div>
          </div>
        );
    }
  };

  return (
    <div className="page-history-overlay" onClick={onClose}>
      <div className="page-history-container" onClick={(e) => e.stopPropagation()}>
        <div className="page-history-header">
          <h2>Version History - {path}</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        <div className="page-history-content">
          {renderContent()}
        </div>
      </div>
    </div>
  );
}