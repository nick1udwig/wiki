import React, { useState } from 'react';
import { SearchBox } from './SearchBox';
import { wikiApi, GlobalSearchResult } from '../api/wiki';
import { useWikiStore } from '../store/wikiStore';
import './GlobalSearch.css';

export function GlobalSearch() {
  const { selectWiki, loadPage } = useWikiStore();

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

  const handleSelectResult = (path: string, result: any) => {
    // Load the wiki and then the page
    selectWiki({ id: result.wiki_id, name: result.wiki_name } as any);
    loadPage(result.wiki_id, path);
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