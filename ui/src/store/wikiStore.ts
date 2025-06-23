import { create } from 'zustand';
import { Wiki, WikiPage, WikiRole, wikiApi, PageInfo } from '../api/wiki';

// Helper to extract error message with details
const getErrorMessage = (error: any, fallback: string): string => {
  if (error.details) {
    return typeof error.details === 'string' ? error.details : JSON.stringify(error.details);
  }
  return error.message || fallback;
};

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
  joinWiki: (wiki_id: string, node_id?: string) => Promise<void>;
  inviteUser: (wiki_id: string, invitee_id: string) => Promise<void>;
  manageMember: (wiki_id: string, member_id: string, action: 'add' | 'remove' | 'update', role?: WikiRole) => Promise<void>;
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
      set({ error: getErrorMessage(error, 'Failed to load wikis'), isLoading: false });
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
      set({ error: getErrorMessage(error, 'Failed to load pages'), isLoading: false });
    }
  },

  loadPage: async (wiki_id: string, path: string) => {
    set({ isLoading: true, error: null });
    try {
      const page = await wikiApi.getPage(wiki_id, path);
      set({ currentPage: page, isLoading: false });
    } catch (error: any) {
      set({ error: getErrorMessage(error, 'Failed to load page'), isLoading: false });
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
      set({ error: getErrorMessage(error, 'Failed to save page'), isLoading: false });
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
      set({ error: getErrorMessage(error, 'Failed to create wiki'), isLoading: false });
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
      set({ error: getErrorMessage(error, 'Failed to create page'), isLoading: false });
    }
  },

  joinWiki: async (wiki_id: string, node_id?: string) => {
    set({ isLoading: true, error: null });
    try {
      await wikiApi.joinWiki(wiki_id, node_id);
      // Reload wikis list to include the newly joined wiki
      await get().loadWikis();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: getErrorMessage(error, 'Failed to join wiki'), isLoading: false });
    }
  },

  inviteUser: async (wiki_id: string, invitee_id: string) => {
    try {
      await wikiApi.inviteUser(wiki_id, invitee_id);
    } catch (error: any) {
      throw new Error(getErrorMessage(error, 'Failed to invite user'));
    }
  },

  manageMember: async (wiki_id: string, member_id: string, action: 'add' | 'remove' | 'update', role?: WikiRole) => {
    try {
      await wikiApi.manageMember(wiki_id, member_id, action, role);
      // Reload the current wiki to get updated member list
      const { currentWiki } = get();
      if (currentWiki && currentWiki.id === wiki_id) {
        const updatedWiki = await wikiApi.getWiki(wiki_id);
        set({ currentWiki: updatedWiki });
      }
    } catch (error: any) {
      throw new Error(getErrorMessage(error, 'Failed to manage member'));
    }
  },

  setError: (error) => {
    set({ error });
  },

  clearError: () => {
    set({ error: null });
  }
}));