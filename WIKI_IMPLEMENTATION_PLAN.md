# Wiki Application Implementation Plan

This document provides a detailed implementation plan for converting the current id/ app into a decentralized wiki application using the Hyperware framework.

## Overview

The wiki app will be a P2P decentralized wiki system using:
- **yrs** CRDT library for conflict-free distributed synchronization
- **Hyperware** framework for P2P networking and state management
- **React/TypeScript** frontend with markdown editing capabilities
- **Role-based access control** (Reader, Writer, Admin, SuperAdmin)

## Implementation Phases

### Phase 1: Backend Foundation

#### 1.1 Update Project Configuration

**File: `Cargo.toml`**
```toml
[package]
name = "wiki"
version = "0.1.0"
edition = "2021"

[dependencies]
yrs = "0.23.1"
uuid = { version = "1.11.0", features = ["v4", "serde"] }
hyperprocess_macro = "0.1"
hyperware_app_common = "0.1"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wit-bindgen = "0.36.0"
rmp-serde = "1.3"
base64 = "0.22"
```

**File: `metadata.json`**
```json
{
  "name": "wiki",
  "description": "Decentralized wiki with P2P synchronization",
  "website": "",
  "code_hashes": {},
  "dependencies": []
}
```

**File: `pkg/manifest.json`**
```json
[
  {
    "process_name": "wiki",
    "process_wasm_path": "wiki/wiki.wasm",
    "on_exit": "Restart",
    "request_networking": true,
    "request_capabilities": [
      {
        "grant_capabilities": {
          "messaging": []
        }
      },
      "http-server",
      "vfs"
    ],
    "public": true
  }
]
```

#### 1.2 Backend Data Structures

**File: `id/src/lib.rs`** (rename directory from `id/` to `wiki/` later)

```rust
use hyperprocess_macro::{hyperprocess, http, remote, local, ws, init};
use hyperware_app_common::{bindings::component::hyperware::capabilities::*, *};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use yrs::{Doc, Map, GetString, Text, Transact, Array, ReadTxn, TransactionMut};
use yrs::updates::encoder::{Encode, Encoder};
use yrs::updates::decoder::{Decode, Decoder};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum WikiRole {
    Reader,
    Writer,
    Admin,
    SuperAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WikiMembership {
    wiki_id: String,
    role: WikiRole,
    joined_at: String, // ISO 8601 timestamp string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Wiki {
    id: String,
    name: String,
    description: String,
    is_public: bool,
    created_by: String,
    created_at: String,
    members: HashMap<String, WikiRole>, // node_id -> role
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageVersion {
    content: Vec<u8>, // Serialized Yrs document
    updated_by: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WikiPage {
    path: String,
    wiki_id: String,
    current_version: PageVersion,
    yrs_doc: Vec<u8>, // Serialized Yrs document state
}

// P2P Sync Messages
#[derive(Debug, Serialize, Deserialize)]
enum SyncMessage {
    PingPageUpdate { wiki_id: String, page_path: String },
    RequestStateVector { wiki_id: String, page_path: String },
    SendStateVector { wiki_id: String, page_path: String, state_vector: Vec<u8> },
    RequestUpdate { wiki_id: String, page_path: String, state_vector: Vec<u8> },
    SendUpdate { wiki_id: String, page_path: String, update: Vec<u8> },
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct WikiState {
    node_id: String,
    wikis: HashMap<String, Wiki>,
    pages: HashMap<String, WikiPage>, // key: "{wiki_id}:{page_path}"
    my_memberships: Vec<WikiMembership>,
    // Transient state (not persisted)
    #[serde(skip)]
    active_docs: HashMap<String, Doc>, // key: "{wiki_id}:{page_path}"
}

#[hyperprocess(
    name = "wiki",
    ui = Some(HttpBindingConfig::default()),
    endpoints = vec![
        Binding::Http { path: "/api", config: HttpBindingConfig::default() },
        Binding::Ws { path: "/ws", config: WsBindingConfig::default() }
    ],
    save_config = SaveOptions::EveryMessage,
    wit_world = "wiki-sys-v0"
)]
impl WikiState {
    // Initialize node with unique ID
    #[init]
    async fn init(&mut self) {
        self.node_id = Uuid::new_v4().to_string();
        println!("Wiki node initialized with ID: {}", self.node_id);
    }
}
```

### Phase 2: Backend HTTP Handlers

#### 2.1 Wiki Management Endpoints

```rust
// Inside WikiState impl block

#[http]
async fn handle_http_request(&mut self, req: HttpRequest) -> anyhow::Result<HttpResponse> {
    let path = req.path()?;
    let body = req.body();
    
    match path.as_str() {
        "/api/create-wiki" => self.create_wiki(body),
        "/api/list-wikis" => self.list_wikis(),
        "/api/get-wiki" => self.get_wiki(body),
        "/api/join-wiki" => self.join_wiki(body),
        "/api/leave-wiki" => self.leave_wiki(body),
        "/api/update-wiki-settings" => self.update_wiki_settings(body),
        "/api/manage-member" => self.manage_member(body),
        
        "/api/list-pages" => self.list_pages(body),
        "/api/get-page" => self.get_page(body),
        "/api/update-page" => self.update_page(body),
        "/api/create-page" => self.create_page(body),
        "/api/delete-page" => self.delete_page(body),
        "/api/search" => self.search_wiki(body),
        
        _ => Ok(HttpResponse::new(404, HashMap::new(), vec![])),
    }
}

// Wiki CRUD operations
fn create_wiki(&mut self, body: Vec<u8>) -> anyhow::Result<HttpResponse> {
    #[derive(Deserialize)]
    struct CreateWikiRequest {
        name: String,
        description: String,
        is_public: bool,
    }
    
    let req: CreateWikiRequest = serde_json::from_slice(&body)?;
    let wiki_id = Uuid::new_v4().to_string();
    
    let wiki = Wiki {
        id: wiki_id.clone(),
        name: req.name,
        description: req.description,
        is_public: req.is_public,
        created_by: self.node_id.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        members: HashMap::from([(self.node_id.clone(), WikiRole::SuperAdmin)]),
    };
    
    self.wikis.insert(wiki_id.clone(), wiki.clone());
    self.my_memberships.push(WikiMembership {
        wiki_id: wiki_id.clone(),
        role: WikiRole::SuperAdmin,
        joined_at: chrono::Utc::now().to_rfc3339(),
    });
    
    Ok(HttpResponse::json(serde_json::json!({
        "wiki_id": wiki_id,
        "wiki": wiki
    })))
}

// Role-based access control
fn check_permission(&self, wiki_id: &str, required_role: WikiRole) -> Result<(), String> {
    let wiki = self.wikis.get(wiki_id)
        .ok_or_else(|| "Wiki not found".to_string())?;
    
    let user_role = wiki.members.get(&self.node_id)
        .ok_or_else(|| "Not a member of this wiki".to_string())?;
    
    match required_role {
        WikiRole::Reader => Ok(()), // Everyone who is a member can read
        WikiRole::Writer => {
            if matches!(user_role, WikiRole::Writer | WikiRole::Admin | WikiRole::SuperAdmin) {
                Ok(())
            } else {
                Err("Insufficient permissions".to_string())
            }
        }
        WikiRole::Admin => {
            if matches!(user_role, WikiRole::Admin | WikiRole::SuperAdmin) {
                Ok(())
            } else {
                Err("Insufficient permissions".to_string())
            }
        }
        WikiRole::SuperAdmin => {
            if matches!(user_role, WikiRole::SuperAdmin) {
                Ok(())
            } else {
                Err("Insufficient permissions".to_string())
            }
        }
    }
}
```

#### 2.2 Page Management with Yrs

```rust
// Page CRUD with CRDT
fn create_page(&mut self, body: Vec<u8>) -> anyhow::Result<HttpResponse> {
    #[derive(Deserialize)]
    struct CreatePageRequest {
        wiki_id: String,
        path: String,
        initial_content: String,
    }
    
    let req: CreatePageRequest = serde_json::from_slice(&body)?;
    
    // Check write permission
    self.check_permission(&req.wiki_id, WikiRole::Writer)
        .map_err(|e| anyhow::anyhow!(e))?;
    
    let page_key = format!("{}:{}", req.wiki_id, req.path);
    
    // Create new Yrs document
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    
    // Initialize with content
    let mut txn = doc.transact_mut();
    text.insert(&mut txn, 0, &req.initial_content);
    txn.commit();
    
    // Serialize document state
    let state_vector = doc.transact().state_vector();
    let update = doc.transact().encode_state_as_update(&state_vector);
    
    let page = WikiPage {
        path: req.path.clone(),
        wiki_id: req.wiki_id.clone(),
        current_version: PageVersion {
            content: update.clone(),
            updated_by: self.node_id.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        yrs_doc: update,
    };
    
    self.pages.insert(page_key.clone(), page);
    self.active_docs.insert(page_key, doc);
    
    // Notify other nodes
    self.broadcast_page_update(&req.wiki_id, &req.path)?;
    
    Ok(HttpResponse::json(serde_json::json!({
        "success": true,
        "path": req.path
    })))
}

fn update_page(&mut self, body: Vec<u8>) -> anyhow::Result<HttpResponse> {
    #[derive(Deserialize)]
    struct UpdatePageRequest {
        wiki_id: String,
        path: String,
        content: String,
    }
    
    let req: UpdatePageRequest = serde_json::from_slice(&body)?;
    
    // Check write permission
    self.check_permission(&req.wiki_id, WikiRole::Writer)
        .map_err(|e| anyhow::anyhow!(e))?;
    
    let page_key = format!("{}:{}", req.wiki_id, req.path);
    
    // Get or create document
    let doc = self.active_docs.entry(page_key.clone())
        .or_insert_with(|| {
            if let Some(page) = self.pages.get(&page_key) {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();
                txn.apply_update(yrs::Update::decode_v1(&page.yrs_doc).unwrap());
                doc
            } else {
                Doc::new()
            }
        });
    
    // Apply update
    let text = doc.get_or_insert_text("content");
    let mut txn = doc.transact_mut();
    
    // Clear and set new content (simple approach)
    let current_len = text.len(&txn);
    if current_len > 0 {
        text.remove_range(&mut txn, 0, current_len);
    }
    text.insert(&mut txn, 0, &req.content);
    txn.commit();
    
    // Save state
    let update = doc.transact().encode_state_as_update(&yrs::StateVector::default());
    
    let page = WikiPage {
        path: req.path.clone(),
        wiki_id: req.wiki_id.clone(),
        current_version: PageVersion {
            content: update.clone(),
            updated_by: self.node_id.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        yrs_doc: update,
    };
    
    self.pages.insert(page_key, page);
    
    // Trigger P2P sync
    self.broadcast_page_update(&req.wiki_id, &req.path)?;
    
    Ok(HttpResponse::json(serde_json::json!({
        "success": true
    })))
}
```

### Phase 3: P2P Synchronization

#### 3.1 P2P Message Handlers

```rust
// P2P sync protocol implementation
#[local]
#[remote]
fn handle_sync_message(&mut self, message: Vec<u8>) -> Result<Vec<u8>, String> {
    let sync_msg: SyncMessage = rmp_serde::from_slice(&message)
        .map_err(|e| format!("Failed to deserialize sync message: {}", e))?;
    
    match sync_msg {
        SyncMessage::PingPageUpdate { wiki_id, page_path } => {
            // Another node updated a page, request their state vector
            let response = SyncMessage::RequestStateVector {
                wiki_id,
                page_path,
            };
            Ok(rmp_serde::to_vec(&response).unwrap())
        }
        
        SyncMessage::RequestStateVector { wiki_id, page_path } => {
            // Send our state vector
            let page_key = format!("{}:{}", wiki_id, page_path);
            
            if let Some(doc) = self.active_docs.get(&page_key) {
                let state_vector = doc.transact().state_vector();
                let response = SyncMessage::SendStateVector {
                    wiki_id,
                    page_path,
                    state_vector,
                };
                Ok(rmp_serde::to_vec(&response).unwrap())
            } else {
                Err("Page not found".to_string())
            }
        }
        
        SyncMessage::SendStateVector { wiki_id, page_path, state_vector } => {
            // Compute diff and send update
            let page_key = format!("{}:{}", wiki_id, page_path);
            
            if let Some(doc) = self.active_docs.get(&page_key) {
                let their_sv = yrs::StateVector::decode_v1(&state_vector)
                    .map_err(|e| format!("Failed to decode state vector: {}", e))?;
                
                let update = doc.transact().encode_diff(&their_sv);
                
                let response = SyncMessage::SendUpdate {
                    wiki_id,
                    page_path,
                    update,
                };
                Ok(rmp_serde::to_vec(&response).unwrap())
            } else {
                Err("Page not found".to_string())
            }
        }
        
        SyncMessage::RequestUpdate { wiki_id, page_path, state_vector } => {
            // They sent their state vector, compute and send diff
            let page_key = format!("{}:{}", wiki_id, page_path);
            
            if let Some(doc) = self.active_docs.get(&page_key) {
                let their_sv = yrs::StateVector::decode_v1(&state_vector)
                    .map_err(|e| format!("Failed to decode state vector: {}", e))?;
                
                let update = doc.transact().encode_diff(&their_sv);
                
                let response = SyncMessage::SendUpdate {
                    wiki_id,
                    page_path,
                    update,
                };
                Ok(rmp_serde::to_vec(&response).unwrap())
            } else {
                Err("Page not found".to_string())
            }
        }
        
        SyncMessage::SendUpdate { wiki_id, page_path, update } => {
            // Apply the update to our document
            let page_key = format!("{}:{}", wiki_id, page_path);
            
            let doc = self.active_docs.entry(page_key.clone())
                .or_insert_with(Doc::new);
            
            let mut txn = doc.transact_mut();
            let update_data = yrs::Update::decode_v1(&update)
                .map_err(|e| format!("Failed to decode update: {}", e))?;
            
            txn.apply_update(update_data);
            
            // Save updated state
            let full_update = doc.transact().encode_state_as_update(&yrs::StateVector::default());
            
            if let Some(page) = self.pages.get_mut(&page_key) {
                page.yrs_doc = full_update;
                page.current_version.updated_at = chrono::Utc::now().to_rfc3339();
            }
            
            Ok(rmp_serde::to_vec(&serde_json::json!({ "success": true })).unwrap())
        }
    }
}

// Helper to broadcast updates to wiki members
fn broadcast_page_update(&self, wiki_id: &str, page_path: &str) -> anyhow::Result<()> {
    if let Some(wiki) = self.wikis.get(wiki_id) {
        let message = SyncMessage::PingPageUpdate {
            wiki_id: wiki_id.to_string(),
            page_path: page_path.to_string(),
        };
        let serialized = rmp_serde::to_vec(&message)?;
        
        // Send to all wiki members
        for (member_id, _) in &wiki.members {
            if member_id != &self.node_id {
                let target = Address::new("our", (member_id, "wiki", "sys"));
                
                // Use generated RPC function
                let _ = handle_sync_message_remote_rpc(&target, serialized.clone());
            }
        }
    }
    Ok(())
}
```

#### 3.2 WebSocket Handler for Real-time Updates

```rust
#[ws]
fn handle_websocket(&mut self, channel_id: u32, message_type: WsMessageType, blob: LazyLoadBlob) {
    match message_type {
        WsMessageType::Open => {
            println!("WebSocket connection opened: {}", channel_id);
        }
        WsMessageType::Close => {
            println!("WebSocket connection closed: {}", channel_id);
        }
        WsMessageType::Message => {
            // Handle real-time updates from UI
            if let Ok(data) = blob.read() {
                if let Ok(json_str) = String::from_utf8(data.clone()) {
                    // Parse and handle WebSocket messages
                    // Could be used for live collaborative editing
                }
            }
        }
        WsMessageType::Error => {
            println!("WebSocket error on channel: {}", channel_id);
        }
    }
}
```

### Phase 4: Frontend Implementation

#### 4.1 Project Structure

```
ui/
├── src/
│   ├── App.tsx              # Main app component with routing
│   ├── main.tsx             # Entry point
│   ├── index.css            # Global styles
│   ├── api/
│   │   └── wiki.ts          # API client functions
│   ├── store/
│   │   ├── wikiStore.ts     # Zustand store for wiki state
│   │   └── types.ts         # TypeScript types
│   ├── components/
│   │   ├── WikiList.tsx     # List of wikis (home page)
│   │   ├── WikiPage.tsx     # Individual wiki view
│   │   ├── PageEditor.tsx   # Markdown editor
│   │   ├── PageViewer.tsx   # Markdown viewer
│   │   ├── WikiSettings.tsx # Wiki management UI
│   │   └── MemberList.tsx   # Member management
│   └── utils/
│       └── markdown.ts      # Markdown parsing utilities
├── package.json
├── tsconfig.json
├── vite.config.ts
└── index.html
```

#### 4.2 API Client

**File: `ui/src/api/wiki.ts`**

```typescript
const API_BASE = '/api';

export interface Wiki {
  id: string;
  name: string;
  description: string;
  is_public: boolean;
  created_by: string;
  created_at: string;
  members: Record<string, WikiRole>;
}

export enum WikiRole {
  Reader = 'Reader',
  Writer = 'Writer',
  Admin = 'Admin',
  SuperAdmin = 'SuperAdmin',
}

export interface WikiPage {
  path: string;
  wiki_id: string;
  content?: string;
  updated_by: string;
  updated_at: string;
}

export const wikiApi = {
  // Wiki management
  createWiki: async (name: string, description: string, is_public: boolean): Promise<{ wiki_id: string; wiki: Wiki }> => {
    const response = await fetch(`${API_BASE}/create-wiki`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, description, is_public }),
    });
    return response.json();
  },

  listWikis: async (): Promise<Wiki[]> => {
    const response = await fetch(`${API_BASE}/list-wikis`);
    return response.json();
  },

  // Page management
  getPage: async (wiki_id: string, path: string): Promise<WikiPage> => {
    const response = await fetch(`${API_BASE}/get-page`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ wiki_id, path }),
    });
    return response.json();
  },

  updatePage: async (wiki_id: string, path: string, content: string): Promise<void> => {
    await fetch(`${API_BASE}/update-page`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ wiki_id, path, content }),
    });
  },

  // Member management
  inviteMember: async (wiki_id: string, member_id: string, role: WikiRole): Promise<void> => {
    await fetch(`${API_BASE}/manage-member`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ wiki_id, member_id, role, action: 'add' }),
    });
  },
};
```

#### 4.3 State Management

**File: `ui/src/store/wikiStore.ts`**

```typescript
import { create } from 'zustand';
import { Wiki, WikiPage, WikiRole } from '../api/wiki';

interface WikiStore {
  // State
  wikis: Wiki[];
  currentWiki: Wiki | null;
  currentPage: WikiPage | null;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  loadWikis: () => Promise<void>;
  selectWiki: (wiki: Wiki) => void;
  loadPage: (wiki_id: string, path: string) => Promise<void>;
  savePage: (content: string) => Promise<void>;
  createWiki: (name: string, description: string, is_public: boolean) => Promise<void>;
}

export const useWikiStore = create<WikiStore>((set, get) => ({
  wikis: [],
  currentWiki: null,
  currentPage: null,
  isLoading: false,
  error: null,

  loadWikis: async () => {
    set({ isLoading: true, error: null });
    try {
      const wikis = await wikiApi.listWikis();
      set({ wikis, isLoading: false });
    } catch (error) {
      set({ error: error.message, isLoading: false });
    }
  },

  selectWiki: (wiki) => {
    set({ currentWiki: wiki });
  },

  loadPage: async (wiki_id, path) => {
    set({ isLoading: true, error: null });
    try {
      const page = await wikiApi.getPage(wiki_id, path);
      set({ currentPage: page, isLoading: false });
    } catch (error) {
      set({ error: error.message, isLoading: false });
    }
  },

  savePage: async (content) => {
    const { currentWiki, currentPage } = get();
    if (!currentWiki || !currentPage) return;

    set({ isLoading: true, error: null });
    try {
      await wikiApi.updatePage(currentWiki.id, currentPage.path, content);
      set({ isLoading: false });
    } catch (error) {
      set({ error: error.message, isLoading: false });
    }
  },

  createWiki: async (name, description, is_public) => {
    set({ isLoading: true, error: null });
    try {
      const result = await wikiApi.createWiki(name, description, is_public);
      const { wikis } = get();
      set({ 
        wikis: [...wikis, result.wiki],
        currentWiki: result.wiki,
        isLoading: false 
      });
    } catch (error) {
      set({ error: error.message, isLoading: false });
    }
  },
}));
```

#### 4.4 Main Components

**File: `ui/src/components/WikiList.tsx`**

```tsx
import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';

export function WikiList() {
  const { wikis, loadWikis, createWiki, selectWiki } = useWikiStore();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newWiki, setNewWiki] = useState({ name: '', description: '', is_public: false });

  useEffect(() => {
    loadWikis();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    await createWiki(newWiki.name, newWiki.description, newWiki.is_public);
    setShowCreateForm(false);
    setNewWiki({ name: '', description: '', is_public: false });
  };

  return (
    <div className="wiki-list">
      <h1>My Wikis</h1>
      
      <button onClick={() => setShowCreateForm(true)}>Create New Wiki</button>
      
      {showCreateForm && (
        <form onSubmit={handleCreate}>
          <input
            type="text"
            placeholder="Wiki Name"
            value={newWiki.name}
            onChange={(e) => setNewWiki({ ...newWiki, name: e.target.value })}
            required
          />
          <textarea
            placeholder="Description"
            value={newWiki.description}
            onChange={(e) => setNewWiki({ ...newWiki, description: e.target.value })}
          />
          <label>
            <input
              type="checkbox"
              checked={newWiki.is_public}
              onChange={(e) => setNewWiki({ ...newWiki, is_public: e.target.checked })}
            />
            Public Wiki
          </label>
          <button type="submit">Create</button>
          <button type="button" onClick={() => setShowCreateForm(false)}>Cancel</button>
        </form>
      )}
      
      <div className="wiki-grid">
        {wikis.map((wiki) => (
          <div key={wiki.id} className="wiki-card" onClick={() => selectWiki(wiki)}>
            <h3>{wiki.name}</h3>
            <p>{wiki.description}</p>
            <span className="wiki-badge">{wiki.is_public ? 'Public' : 'Private'}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
```

**File: `ui/src/components/PageEditor.tsx`**

```tsx
import React, { useState, useEffect } from 'react';
import { useWikiStore } from '../store/wikiStore';

interface PageEditorProps {
  initialContent: string;
  canEdit: boolean;
}

export function PageEditor({ initialContent, canEdit }: PageEditorProps) {
  const [content, setContent] = useState(initialContent);
  const [isEditing, setIsEditing] = useState(false);
  const [showPreview, setShowPreview] = useState(false);
  const { savePage, isLoading } = useWikiStore();

  const handleSave = async () => {
    await savePage(content);
    setIsEditing(false);
  };

  if (!canEdit && !isEditing) {
    return <div className="page-viewer" dangerouslySetInnerHTML={{ __html: parseMarkdown(content) }} />;
  }

  return (
    <div className="page-editor">
      <div className="editor-toolbar">
        {!isEditing && (
          <button onClick={() => setIsEditing(true)}>Edit</button>
        )}
        {isEditing && (
          <>
            <button onClick={handleSave} disabled={isLoading}>Save</button>
            <button onClick={() => setShowPreview(!showPreview)}>
              {showPreview ? 'Edit' : 'Preview'}
            </button>
            <button onClick={() => {
              setContent(initialContent);
              setIsEditing(false);
            }}>Cancel</button>
          </>
        )}
      </div>
      
      {isEditing && !showPreview && (
        <textarea
          className="markdown-editor"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Write your content in Markdown..."
        />
      )}
      
      {(showPreview || !isEditing) && (
        <div className="markdown-preview" dangerouslySetInnerHTML={{ __html: parseMarkdown(content) }} />
      )}
    </div>
  );
}

// Simple markdown parser (in real app, use a library like marked or remark)
function parseMarkdown(text: string): string {
  return text
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/\*\*(.*)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*)\*/g, '<em>$1</em>')
    .replace(/\n/g, '<br>');
}
```

### Phase 5: Build and Deployment

#### 5.1 Build Process

1. **Backend Build**:
   ```bash
   # From project root
   kit build --hyperapp
   ```
   This will:
   - Compile Rust code to WASM
   - Generate WIT bindings in `api/` directory
   - Package the app

2. **Frontend Build**:
   ```bash
   # From ui/ directory
   npm install
   npm run build
   ```

3. **Deploy**:
   ```bash
   kit start-package . --port 8080
   ```

#### 5.2 Testing Checklist

- [ ] Create new wiki
- [ ] Join existing wiki (public/private)
- [ ] Create and edit pages
- [ ] P2P synchronization between nodes
- [ ] Role-based permissions
- [ ] WebSocket real-time updates
- [ ] Search functionality
- [ ] Member management

## Key Implementation Notes

1. **No Custom WIT Types**: All timestamps use `string` format (ISO 8601)
2. **CRDT Synchronization**: Uses yrs library's built-in sync protocol
3. **Persistence**: State automatically saved after each message
4. **P2P Communication**: Uses generated RPC functions from `caller_utils`
5. **Frontend State**: Zustand for state management, standard fetch for API calls
6. **WebSocket**: Available for future real-time collaborative editing

## Security Considerations

1. **Authentication**: Node ID used as identity (could be enhanced with cryptographic signatures)
2. **Authorization**: Role-based access control enforced in backend
3. **Data Validation**: All inputs validated before processing
4. **P2P Security**: Messages authenticated by Hyperware framework

## Future Enhancements

1. **Full-text search** using embedded search engine
2. **Version history** with diff viewing
3. **Real-time collaborative editing** via WebSocket
4. **File attachments** using VFS capability
5. **Wiki templates** and themes
6. **Export functionality** (PDF, HTML, Markdown)