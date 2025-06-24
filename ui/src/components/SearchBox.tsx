import React, { useState, useEffect, useRef } from 'react';
import { SearchResult } from '../api/wiki';
import './SearchBox.css';

interface SearchBoxProps {
  onSearch: (query: string) => Promise<SearchResult[]>;
  onSelectResult: (path: string) => void;
  placeholder?: string;
}

export function SearchBox({ onSearch, onSelectResult, placeholder = "Search pages..." }: SearchBoxProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [showResults, setShowResults] = useState(false);
  const searchRef = useRef<HTMLDivElement>(null);
  const debounceTimeout = useRef<NodeJS.Timeout>();

  useEffect(() => {
    // Click outside handler
    const handleClickOutside = (event: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(event.target as Node)) {
        setShowResults(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  useEffect(() => {
    if (query.trim().length === 0) {
      setResults([]);
      setShowResults(false);
      setIsSearching(false);
      return;
    }

    // Debounce search
    clearTimeout(debounceTimeout.current);
    debounceTimeout.current = setTimeout(async () => {
      setIsSearching(true);
      
      // Create a timeout promise
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error('Search timeout')), 5000);
      });
      
      try {
        // Race between search and timeout
        const searchResults = await Promise.race([
          onSearch(query),
          timeoutPromise
        ]);
        setResults(searchResults);
        setShowResults(searchResults.length > 0);
      } catch (error) {
        console.error('Search failed:', error);
        setResults([]);
        setShowResults(false);
      } finally {
        setIsSearching(false);
      }
    }, 300);

    return () => {
      clearTimeout(debounceTimeout.current);
      setIsSearching(false);
    };
  }, [query, onSearch]);

  const handleSelectResult = (path: string) => {
    onSelectResult(path);
    setQuery('');
    setShowResults(false);
  };

  return (
    <div className="search-box" ref={searchRef}>
      <div className="search-input-wrapper">
        <input
          type="text"
          className="search-input"
          placeholder={placeholder}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => results.length > 0 && setShowResults(true)}
        />
        {isSearching && <span className="search-spinner">🔍</span>}
      </div>
      
      {showResults && (
        <div className="search-results">
          {results.map((result, index) => (
            <div
              key={`${result.path}-${index}`}
              className="search-result-item"
              onClick={() => handleSelectResult(result.path)}
            >
              <div className="search-result-path">{result.path}</div>
              <div className="search-result-snippet">{result.snippet}</div>
              <div className="search-result-meta">
                Updated by {result.updated_by} • {new Date(result.updated_at).toLocaleDateString()}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}