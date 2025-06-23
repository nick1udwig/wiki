import { create } from 'zustand';
import { Wiki, WikiPage, WikiRole, wikiApi, PageInfo } from '../api/wiki';

interface WikiStore {
  // State
  wikis: Wiki[];
  currentWiki: Wiki | null;
  currentPage: WikiPage | null;
  pages: PageInfo[];
  isLoading: boolean;
  error: string | null;
  
  // Actions
  loadWikis: () => Promise<void>;
  selectWiki: (wiki: Wiki | null) => void;
  loadPage: (wiki_id: string, path: string) => Promise<void>;
  savePage: (content: string) => Promise<void>;
  createWiki: (name: string, description: string, is_public: boolean) => Promise<void>;
  createPage: (path: string, initialContent: string) => Promise<void>;
  loadPages: (wiki_id: string) => Promise<void>;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useWikiStore = create<WikiStore>((set, get) => ({
  wikis: [],
  currentWiki: null,
  currentPage: null,
  pages: [],
  isLoading: false,
  error: null,

  loadWikis: async () => {
    set({ isLoading: true, error: null });
    try {
      const wikis = await wikiApi.listWikis();
      set({ wikis, isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load wikis', isLoading: false });
    }
  },

  selectWiki: (wiki) => {
    set({ currentWiki: wiki, currentPage: null, pages: [] });
    if (wiki) {
      get().loadPages(wiki.id);
    }
  },

  loadPages: async (wiki_id: string) => {
    set({ isLoading: true, error: null });
    try {
      const pages = await wikiApi.listPages(wiki_id);
      set({ pages, isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load pages', isLoading: false });
    }
  },

  loadPage: async (wiki_id: string, path: string) => {
    set({ isLoading: true, error: null });
    try {
      const page = await wikiApi.getPage(wiki_id, path);
      set({ currentPage: page, isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load page', isLoading: false });
    }
  },

  savePage: async (content: string) => {
    const { currentWiki, currentPage } = get();
    if (!currentWiki || !currentPage) return;

    set({ isLoading: true, error: null });
    try {
      await wikiApi.updatePage(currentWiki.id, currentPage.path, content);
      set({ 
        isLoading: false,
        currentPage: { ...currentPage, content }
      });
    } catch (error: any) {
      set({ error: error.message || 'Failed to save page', isLoading: false });
    }
  },

  createWiki: async (name: string, description: string, is_public: boolean) => {
    set({ isLoading: true, error: null });
    try {
      const result = await wikiApi.createWiki(name, description, is_public);
      const { wikis } = get();
      set({ 
        wikis: [...wikis, result.wiki],
        currentWiki: result.wiki,
        isLoading: false 
      });
    } catch (error: any) {
      set({ error: error.message || 'Failed to create wiki', isLoading: false });
    }
  },

  createPage: async (path: string, initialContent: string) => {
    const { currentWiki } = get();
    if (!currentWiki) return;

    set({ isLoading: true, error: null });
    try {
      await wikiApi.createPage(currentWiki.id, path, initialContent);
      // Reload pages list
      await get().loadPages(currentWiki.id);
      // Load the newly created page
      await get().loadPage(currentWiki.id, path);
    } catch (error: any) {
      set({ error: error.message || 'Failed to create page', isLoading: false });
    }
  },

  setError: (error) => {
    set({ error });
  },

  clearError: () => {
    set({ error: null });
  }
}));