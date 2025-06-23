use hyperprocess_macro::hyperprocess;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yrs::{Doc, GetString, Text, Transact, ReadTxn};
use yrs::updates::encoder::{Encoder, EncoderV1};
use yrs::updates::decoder::Decode;
use uuid::Uuid;
use chrono::Utc;

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
    joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Wiki {
    id: String,
    name: String,
    description: String,
    is_public: bool,
    created_by: String,
    created_at: String,
    members: HashMap<String, WikiRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageVersion {
    content: Vec<u8>,
    updated_by: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WikiPage {
    path: String,
    wiki_id: String,
    current_version: PageVersion,
    yrs_doc: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct WikiState {
    node_id: String,
    wikis: HashMap<String, Wiki>,
    pages: HashMap<String, WikiPage>,
    my_memberships: Vec<WikiMembership>,
    #[serde(skip)]
    active_docs: HashMap<String, Doc>,
}

#[derive(Deserialize)]
struct CreateWikiRequest {
    name: String,
    description: String,
    is_public: bool,
}

#[derive(Deserialize)]
struct GetWikiRequest {
    wiki_id: String,
}

#[derive(Deserialize)]
struct JoinWikiRequest {
    wiki_id: String,
    join_code: Option<String>,
}

#[derive(Deserialize)]
struct LeaveWikiRequest {
    wiki_id: String,
}

#[derive(Deserialize)]
struct UpdateWikiRequest {
    wiki_id: String,
    name: Option<String>,
    description: Option<String>,
    is_public: Option<bool>,
}

#[derive(Deserialize)]
struct ManageMemberRequest {
    wiki_id: String,
    member_id: String,
    action: String,
    role: Option<WikiRole>,
}

#[derive(Deserialize)]
struct CreatePageRequest {
    wiki_id: String,
    path: String,
    initial_content: String,
}

#[derive(Deserialize)]
struct UpdatePageRequest {
    wiki_id: String,
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct GetPageRequest {
    wiki_id: String,
    path: String,
}

#[derive(Deserialize)]
struct ListPagesRequest {
    wiki_id: String,
}

#[derive(Deserialize)]
struct DeletePageRequest {
    wiki_id: String,
    path: String,
}

#[derive(Deserialize)]
struct SearchRequest {
    wiki_id: String,
    query: String,
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
    #[init]
    async fn init(&mut self) {
        self.node_id = Uuid::new_v4().to_string();
        println!("Wiki node initialized with ID: {}", self.node_id);
    }

    #[http]
    async fn create_wiki(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: CreateWikiRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let wiki_id = Uuid::new_v4().to_string();
        
        let wiki = Wiki {
            id: wiki_id.clone(),
            name: req.name,
            description: req.description,
            is_public: req.is_public,
            created_by: self.node_id.clone(),
            created_at: Utc::now().to_rfc3339(),
            members: HashMap::from([(self.node_id.clone(), WikiRole::SuperAdmin)]),
        };
        
        self.wikis.insert(wiki_id.clone(), wiki.clone());
        self.my_memberships.push(WikiMembership {
            wiki_id: wiki_id.clone(),
            role: WikiRole::SuperAdmin,
            joined_at: Utc::now().to_rfc3339(),
        });
        
        Ok(serde_json::to_vec(&serde_json::json!({
            "wiki_id": wiki_id,
            "wiki": wiki
        })).unwrap())
    }

    #[http]
    async fn list_wikis(&mut self) -> Result<Vec<u8>, String> {
        let my_wikis: Vec<&Wiki> = self.my_memberships
            .iter()
            .filter_map(|m| self.wikis.get(&m.wiki_id))
            .collect();
        
        Ok(serde_json::to_vec(&my_wikis).unwrap())
    }

    #[http]
    async fn get_wiki(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: GetWikiRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let wiki = self.wikis.get(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;
        
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;
        
        Ok(serde_json::to_vec(&wiki).unwrap())
    }

    #[http]
    async fn join_wiki(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: JoinWikiRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        let wiki = self.wikis.get_mut(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;
        
        if !wiki.is_public && req.join_code.is_none() {
            return Err("Private wiki requires join code".to_string());
        }
        
        wiki.members.insert(self.node_id.clone(), WikiRole::Reader);
        
        self.my_memberships.push(WikiMembership {
            wiki_id: req.wiki_id.clone(),
            role: WikiRole::Reader,
            joined_at: Utc::now().to_rfc3339(),
        });
        
        Ok(serde_json::to_vec(&serde_json::json!({ "success": true })).unwrap())
    }

    #[http]
    async fn leave_wiki(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: LeaveWikiRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        if let Some(wiki) = self.wikis.get_mut(&req.wiki_id) {
            wiki.members.remove(&self.node_id);
        }
        
        self.my_memberships.retain(|m| m.wiki_id != req.wiki_id);
        
        Ok(serde_json::to_vec(&serde_json::json!({ "success": true })).unwrap())
    }

    #[http]
    async fn update_wiki_settings(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: UpdateWikiRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Admin)?;
        
        let wiki = self.wikis.get_mut(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;
        
        if let Some(name) = req.name {
            wiki.name = name;
        }
        if let Some(description) = req.description {
            wiki.description = description;
        }
        if let Some(is_public) = req.is_public {
            wiki.is_public = is_public;
        }
        
        Ok(serde_json::to_vec(&serde_json::json!({ "success": true })).unwrap())
    }

    #[http]
    async fn manage_member(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: ManageMemberRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Admin)?;
        
        let wiki = self.wikis.get_mut(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;
        
        match req.action.as_str() {
            "add" => {
                if let Some(role) = req.role {
                    wiki.members.insert(req.member_id, role);
                }
            }
            "remove" => {
                wiki.members.remove(&req.member_id);
            }
            "update" => {
                if let Some(role) = req.role {
                    wiki.members.insert(req.member_id, role);
                }
            }
            _ => return Err("Invalid action".to_string()),
        }
        
        Ok(serde_json::to_vec(&serde_json::json!({ "success": true })).unwrap())
    }

    #[http]
    async fn create_page(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: CreatePageRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Writer)?;
        
        let page_key = format!("{}:{}", req.wiki_id, req.path);
        
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        
        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 0, &req.initial_content);
            txn.commit();
        }
        
        let mut encoder = EncoderV1::new();
        doc.transact().encode_state_as_update(&yrs::StateVector::default(), &mut encoder);
        let update = encoder.to_vec();
        
        let page = WikiPage {
            path: req.path.clone(),
            wiki_id: req.wiki_id.clone(),
            current_version: PageVersion {
                content: update.clone(),
                updated_by: self.node_id.clone(),
                updated_at: Utc::now().to_rfc3339(),
            },
            yrs_doc: update,
        };
        
        self.pages.insert(page_key.clone(), page);
        self.active_docs.insert(page_key, doc);
        
        Ok(serde_json::to_vec(&serde_json::json!({
            "success": true,
            "path": req.path
        })).unwrap())
    }

    #[http]
    async fn update_page(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: UpdatePageRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Writer)?;
        
        let page_key = format!("{}:{}", req.wiki_id, req.path);
        
        let doc = self.active_docs.entry(page_key.clone())
            .or_insert_with(|| {
                if let Some(page) = self.pages.get(&page_key) {
                    let new_doc = Doc::new();
                    {
                        let mut txn = new_doc.transact_mut();
                        if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                            let _ = txn.apply_update(update);
                        }
                    }
                    new_doc
                } else {
                    Doc::new()
                }
            });
        
        let text = doc.get_or_insert_text("content");
        {
            let mut txn = doc.transact_mut();
            
            let current_len = text.len(&txn);
            if current_len > 0 {
                text.remove_range(&mut txn, 0, current_len);
            }
            text.insert(&mut txn, 0, &req.content);
            txn.commit();
        }
        
        let mut encoder = EncoderV1::new();
        doc.transact().encode_state_as_update(&yrs::StateVector::default(), &mut encoder);
        let update = encoder.to_vec();
        
        let page = WikiPage {
            path: req.path.clone(),
            wiki_id: req.wiki_id.clone(),
            current_version: PageVersion {
                content: update.clone(),
                updated_by: self.node_id.clone(),
                updated_at: Utc::now().to_rfc3339(),
            },
            yrs_doc: update,
        };
        
        self.pages.insert(page_key, page);
        
        Ok(serde_json::to_vec(&serde_json::json!({
            "success": true
        })).unwrap())
    }

    #[http]
    async fn get_page(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: GetPageRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;
        
        let page_key = format!("{}:{}", req.wiki_id, req.path);
        
        if let Some(page) = self.pages.get(&page_key) {
            let doc = self.active_docs.entry(page_key.clone())
                .or_insert_with(|| {
                    let new_doc = Doc::new();
                    {
                        let mut txn = new_doc.transact_mut();
                        if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                            let _ = txn.apply_update(update);
                        }
                    }
                    new_doc
                });
            
            let text = doc.get_or_insert_text("content");
            let content = text.get_string(&doc.transact());
            
            Ok(serde_json::to_vec(&serde_json::json!({
                "path": page.path,
                "wiki_id": page.wiki_id,
                "content": content,
                "updated_by": page.current_version.updated_by,
                "updated_at": page.current_version.updated_at
            })).unwrap())
        } else {
            Ok(serde_json::to_vec(&serde_json::json!({
                "path": req.path,
                "wiki_id": req.wiki_id,
                "content": "",
                "updated_by": "",
                "updated_at": ""
            })).unwrap())
        }
    }

    #[http]
    async fn list_pages(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: ListPagesRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;
        
        let pages: Vec<_> = self.pages
            .iter()
            .filter(|(_, page)| page.wiki_id == req.wiki_id)
            .map(|(_, page)| serde_json::json!({
                "path": page.path,
                "updated_by": page.current_version.updated_by,
                "updated_at": page.current_version.updated_at
            }))
            .collect();
        
        Ok(serde_json::to_vec(&pages).unwrap())
    }

    #[http]
    async fn delete_page(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: DeletePageRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Writer)?;
        
        let page_key = format!("{}:{}", req.wiki_id, req.path);
        
        self.pages.remove(&page_key);
        self.active_docs.remove(&page_key);
        
        Ok(serde_json::to_vec(&serde_json::json!({ "success": true })).unwrap())
    }

    #[http]
    async fn search_wiki(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let req: SearchRequest = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;
        
        let query_lower = req.query.to_lowercase();
        let mut results = Vec::new();
        
        for (key, page) in &self.pages {
            if page.wiki_id != req.wiki_id {
                continue;
            }
            
            let doc = self.active_docs.entry(key.clone())
                .or_insert_with(|| {
                    let new_doc = Doc::new();
                    {
                        let mut txn = new_doc.transact_mut();
                        if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                            let _ = txn.apply_update(update);
                        }
                    }
                    new_doc
                });
            
            let text = doc.get_or_insert_text("content");
            let content = text.get_string(&doc.transact()).to_lowercase();
            
            if content.contains(&query_lower) || page.path.to_lowercase().contains(&query_lower) {
                results.push(serde_json::json!({
                    "path": page.path,
                    "updated_by": page.current_version.updated_by,
                    "updated_at": page.current_version.updated_at,
                    "snippet": content.chars().take(200).collect::<String>()
                }));
            }
        }
        
        Ok(serde_json::to_vec(&results).unwrap())
    }

}

impl WikiState {
    fn check_permission(&self, wiki_id: &str, required_role: WikiRole) -> Result<(), String> {
        let wiki = self.wikis.get(wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;
        
        let user_role = wiki.members.get(&self.node_id)
            .ok_or_else(|| "Not a member of this wiki".to_string())?;
        
        match required_role {
            WikiRole::Reader => Ok(()),
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
}