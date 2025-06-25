import React, { useState } from 'react';
import { SearchBox } from './SearchBox';
import { wikiApi, GlobalSearchResult } from '../api/wiki';
import { useWikiStore } from '../store/wikiStore';
import './GlobalSearch.css';

export function GlobalSearch() {
  const { selectWiki, loadPage, loadWikis, wikis, setSidebarCollapsed } = useWikiStore();

  const handleGlobalSearch = async (query: string): Promise<any[]> => {
    const results = await wikiApi.searchAllWikis(query);
    // Convert GlobalSearchResult to the format expected by SearchBox
    return results.map(r => ({
      path: r.path,
      updated_by: r.updated_by,
      updated_at: r.updated_at,
      snippet: r.snippet,
      wiki_id: r.wiki_id,
      wiki_name: r.wiki_name
    }));
  };

  const handleSelectResult = async (path: string, result: any) => {
    try {
      // First, check if we already have this wiki in our list
      let wiki = wikis.find(w => w.id === result.wiki_id);
      
      if (!wiki) {
        // If not, try to load it
        const loadedWiki = await wikiApi.getWiki(result.wiki_id);
        await selectWiki(loadedWiki);
      } else {
        await selectWiki(wiki);
      }
      
      // Then load the specific page
      await loadPage(result.wiki_id, path);
      
      // Collapse sidebar on mobile or always for better UX
      setSidebarCollapsed(true);
    } catch (error) {
      console.error('Failed to navigate to search result:', error);
      // If loading the wiki fails, at least try to reload the wiki list
      await loadWikis();
    }
  };

  return (
    <div className="global-search">
      <SearchBox 
        onSearch={handleGlobalSearch}
        onSelectResult={handleSelectResult}
        placeholder="Search all wikis..."
      />
    </div>
  );
}