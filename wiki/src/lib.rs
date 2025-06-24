use hyperprocess_macro::hyperprocess;
use hyperware_process_lib::{get_state, our, println, Address};
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
    created_by: String, // Node ID of creator (e.g., "alice.os")
    created_at: String,
    members: HashMap<String, WikiRole>, // Keys are node IDs (e.g., "alice.os")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageVersion {
    content: Vec<u8>,
    updated_by: String, // Node ID of updater (e.g., "alice.os")
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WikiPage {
    path: String,
    wiki_id: String,
    current_version: PageVersion,
    yrs_doc: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WikiInvite {
    id: String,
    wiki_id: String,
    wiki_name: String,
    inviter_id: String, // Node ID of inviter (e.g., "alice.os")
    invitee_id: String, // Node ID of invitee (e.g., "bob.os")
    created_at: String,
    expires_at: String,
    status: InviteStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum InviteStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct WikiState {
    node_id: String, // Our node ID (e.g., "alice.os"), NOT the full address
    wikis: HashMap<String, Wiki>,
    pages: HashMap<String, WikiPage>,
    my_memberships: Vec<WikiMembership>,
    invites: HashMap<String, WikiInvite>,
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
    node_id: Option<String>, // Node where the wiki exists (for remote wikis)
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
    member_id: String, // Node ID (e.g., "alice.os")
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

#[derive(Deserialize)]
struct FindWikisByUserRequest {
    username: String, // Node ID (e.g., "alice.os")
}

#[derive(Deserialize)]
struct GetPublicWikiRequest {
    wiki_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WikiMessage {
    FindWikisByUser { username: String },
    GetPublicWiki { wiki_id: String },
    JoinPublicWiki { wiki_id: String, user_id: String },
    GetWikiData { wiki_id: String },
    GetWikiPages { wiki_id: String },
    GetWikiPage { wiki_id: String, path: String },
    CreatePage { wiki_id: String, path: String, initial_content: String, user_id: String },
    UpdatePage { wiki_id: String, path: String, content: String, user_id: String },
    SendInvite { invite: WikiInvite, wiki: Wiki },
    InviteResponse { invite_id: String, status: InviteStatus, invitee_id: String },
    RoleUpdate { wiki_id: String, member_id: String, new_role: WikiRole },
    SearchPages { wiki_id: String, query: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WikiResponse {
    WikiList(Vec<serde_json::Value>),
    WikiInfo(serde_json::Value),
    WikiData(Wiki),
    PageList(Vec<PageSummary>),
    PageData(PageInfo),
    SearchResults(Vec<SearchResult>),
    Success(bool),
    Error(String),
}

#[derive(Deserialize)]
struct InviteUserRequest {
    wiki_id: String,
    invitee_id: String, // Node ID (e.g., "alice.os")
}

#[derive(Deserialize)]
struct RespondToInviteRequest {
    invite_id: String,
    accept: bool,
}

// WebSocket notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum WsNotification {
    WikiListUpdated,
    WikiUpdated { wiki_id: String },
    PageListUpdated { wiki_id: String },
    PageUpdated { wiki_id: String, path: String },
    RoleUpdated { wiki_id: String, new_role: WikiRole },
}

// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SuccessResponse {
    success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateWikiResponse {
    wiki_id: String,
    wiki: Wiki,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreatePageResponse {
    success: bool,
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InviteUserResponse {
    invite_id: String,
    success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RespondToInviteResponse {
    success: bool,
    status: InviteStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageInfo {
    path: String,
    wiki_id: String,
    content: String,
    updated_by: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageSummary {
    path: String,
    updated_by: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    path: String,
    updated_by: String,
    updated_at: String,
    snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InviteInfo {
    id: String,
    wiki_id: String,
    wiki_name: String,
    inviter_id: String,
    created_at: String,
    expires_at: String,
    is_expired: bool,
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
        // Extract just the node part from the full address
        let our_address = our();
        self.node_id = our_address.node().to_string();

        if let Some(ref state) = get_state() {
            if let Ok(WikiState { wikis, pages, my_memberships, invites, .. }) = rmp_serde::from_slice(state) {
                self.wikis = wikis;
                self.pages = pages;
                self.my_memberships = my_memberships;
                self.invites = invites;
            }
        }

        println!("Wiki node initialized with node ID: {}", self.node_id);
        println!("Full address: {}", our_address);
    }

    #[remote]
    async fn handle_wiki_message(&mut self, body: Vec<u8>) -> Result<Vec<u8>, String> {
        let body_str = String::from_utf8(body)
            .map_err(|e| format!("Failed to convert body to string: {}", e))?;
        let message: WikiMessage = match serde_json::from_str(&body_str) {
            Ok(msg) => msg,
            Err(e) => {
                let error_response = WikiResponse::Error(format!("Failed to parse message: {}", e));
                return Ok(serde_json::to_string(&error_response).unwrap().into_bytes());
            }
        };

        let response = match message {
            WikiMessage::FindWikisByUser { username } => {
                // Find all public wikis that have this user as a member
                let matching_wikis: Vec<_> = self.wikis
                    .values()
                    .filter(|wiki| wiki.is_public && wiki.members.contains_key(&username))
                    .map(|wiki| serde_json::json!({
                        "id": wiki.id,
                        "name": wiki.name,
                        "description": wiki.description,
                        "is_public": wiki.is_public,
                        "member_count": wiki.members.len(),
                        "user_role": wiki.members.get(&username),
                        "node_id": self.node_id
                    }))
                    .collect();

                WikiResponse::WikiList(matching_wikis)
            }
            WikiMessage::GetPublicWiki { wiki_id } => {
                match self.wikis.get(&wiki_id) {
                    Some(wiki) if wiki.is_public => {
                        let wiki_info = serde_json::json!({
                            "id": wiki.id,
                            "name": wiki.name,
                            "description": wiki.description,
                            "is_public": wiki.is_public,
                            "member_count": wiki.members.len(),
                            "node_id": self.node_id
                        });
                        WikiResponse::WikiInfo(wiki_info)
                    }
                    Some(_) => WikiResponse::Error("Wiki is not public".to_string()),
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::JoinPublicWiki { wiki_id, user_id } => {
                match self.wikis.get_mut(&wiki_id) {
                    Some(wiki) if wiki.is_public => {
                        // Add the user as a member with Writer role for public wikis
                        // This allows collaboration on public wikis
                        wiki.members.insert(user_id.clone(), WikiRole::Writer);
                        println!("User {} joined wiki {} as Writer", user_id, wiki_id);
                        WikiResponse::Success(true)
                    }
                    Some(_) => WikiResponse::Error("Wiki is not public".to_string()),
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::GetWikiData { wiki_id } => {
                match self.wikis.get(&wiki_id) {
                    Some(wiki) => {
                        // Allow access to wiki data for both public and private wikis
                        // TODO: In production, verify requester is a member for private wikis
                        WikiResponse::WikiData(wiki.clone())
                    }
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::GetWikiPages { wiki_id } => {
                match self.wikis.get(&wiki_id) {
                    Some(wiki) => {
                        // For now, allow access to pages if wiki is public
                        // TODO: In the future, check if the requester is a member for private wikis
                        if wiki.is_public {
                            let pages: Vec<PageSummary> = self.pages
                                .iter()
                                .filter(|(_, page)| page.wiki_id == wiki_id)
                                .map(|(_, page)| PageSummary {
                                    path: page.path.clone(),
                                    updated_by: page.current_version.updated_by.clone(),
                                    updated_at: page.current_version.updated_at.clone(),
                                })
                                .collect();
                            WikiResponse::PageList(pages)
                        } else {
                            // For private wikis, we'll allow access for now
                            // In a production system, we'd verify the requester is a member
                            let pages: Vec<PageSummary> = self.pages
                                .iter()
                                .filter(|(_, page)| page.wiki_id == wiki_id)
                                .map(|(_, page)| PageSummary {
                                    path: page.path.clone(),
                                    updated_by: page.current_version.updated_by.clone(),
                                    updated_at: page.current_version.updated_at.clone(),
                                })
                                .collect();
                            WikiResponse::PageList(pages)
                        }
                    }
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::GetWikiPage { wiki_id, path } => {
                match self.wikis.get(&wiki_id) {
                    Some(_wiki) => {
                        // Allow access to pages for both public and private wikis
                        // TODO: In production, verify requester is a member for private wikis
                        let page_key = format!("{}:{}", wiki_id, path);
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

                            WikiResponse::PageData(PageInfo {
                                path: page.path.clone(),
                                wiki_id: page.wiki_id.clone(),
                                content,
                                updated_by: page.current_version.updated_by.clone(),
                                updated_at: page.current_version.updated_at.clone(),
                            })
                        } else {
                            WikiResponse::PageData(PageInfo {
                                path: path.clone(),
                                wiki_id: wiki_id.clone(),
                                content: String::new(),
                                updated_by: String::new(),
                                updated_at: String::new(),
                            })
                        }
                    }
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::CreatePage { wiki_id, path, initial_content, user_id } => {
                match self.wikis.get_mut(&wiki_id) {
                    Some(wiki) => {
                        // Check if user has write permissions
                        match wiki.members.get(&user_id) {
                            Some(role) if matches!(role, WikiRole::Writer | WikiRole::Admin | WikiRole::SuperAdmin) => {
                                let page_key = format!("{}:{}", wiki_id, path);

                                // Create CRDT document
                                let doc = Doc::new();
                                let text = doc.get_or_insert_text("content");
                                {
                                    let mut txn = doc.transact_mut();
                                    text.insert(&mut txn, 0, &initial_content);
                                    txn.commit();
                                }

                                let mut encoder = EncoderV1::new();
                                doc.transact().encode_state_as_update(&yrs::StateVector::default(), &mut encoder);
                                let update = encoder.to_vec();

                                let page = WikiPage {
                                    path: path.clone(),
                                    wiki_id: wiki_id.clone(),
                                    current_version: PageVersion {
                                        content: update.clone(),
                                        updated_by: user_id.clone(),
                                        updated_at: Utc::now().to_rfc3339(),
                                    },
                                    yrs_doc: update,
                                };

                                self.pages.insert(page_key.clone(), page);
                                self.active_docs.insert(page_key, doc);

                                WikiResponse::Success(true)
                            }
                            _ => WikiResponse::Error("Insufficient permissions".to_string()),
                        }
                    }
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::UpdatePage { wiki_id, path, content, user_id } => {
                match self.wikis.get(&wiki_id) {
                    Some(wiki) => {
                        // Check if user has write permissions
                        match wiki.members.get(&user_id) {
                            Some(role) if matches!(role, WikiRole::Writer | WikiRole::Admin | WikiRole::SuperAdmin) => {
                                let page_key = format!("{}:{}", wiki_id, path);

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

                                // Replace content
                                {
                                    let mut txn = doc.transact_mut();
                                    let current_len = text.len(&txn);
                                    if current_len > 0 {
                                        text.remove_range(&mut txn, 0, current_len);
                                    }
                                    text.insert(&mut txn, 0, &content);
                                    txn.commit();
                                }

                                // Encode the update
                                let mut encoder = EncoderV1::new();
                                doc.transact().encode_state_as_update(&yrs::StateVector::default(), &mut encoder);
                                let update = encoder.to_vec();

                                if let Some(page) = self.pages.get_mut(&page_key) {
                                    page.current_version = PageVersion {
                                        content: update.clone(),
                                        updated_by: user_id.clone(),
                                        updated_at: Utc::now().to_rfc3339(),
                                    };
                                    page.yrs_doc = update;
                                }

                                WikiResponse::Success(true)
                            }
                            _ => WikiResponse::Error("Insufficient permissions".to_string()),
                        }
                    }
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
            WikiMessage::SendInvite { invite, wiki } => {
                // Check if the invite is for this user
                if invite.invitee_id != self.node_id {
                    WikiResponse::Error("This invite is not for this node".to_string())
                } else {
                    // Store the invite
                    self.invites.insert(invite.id.clone(), invite.clone());

                    // Store the wiki data - but if it's from another node, store with remote ID
                    if invite.inviter_id != self.node_id {
                        // Store with remote reference ID to ensure P2P operations
                        let mut remote_wiki = wiki;
                        remote_wiki.id = format!("{}@{}", remote_wiki.id, invite.inviter_id);
                        self.wikis.insert(remote_wiki.id.clone(), remote_wiki);
                    } else {
                        // Local wiki - store normally
                        self.wikis.insert(wiki.id.clone(), wiki);
                    }
                    WikiResponse::Success(true)
                }
            }
            WikiMessage::InviteResponse { invite_id, status, invitee_id } => {
                // Update the invite status on the inviter's node
                if let Some(invite) = self.invites.get_mut(&invite_id) {
                    invite.status = status.clone();

                    // If accepted, update the wiki membership
                    if status == InviteStatus::Accepted {
                        if let Some(wiki) = self.wikis.get_mut(&invite.wiki_id) {
                            wiki.members.insert(invitee_id, WikiRole::Reader);
                        }
                    }
                }
                WikiResponse::Success(true)
            }
            WikiMessage::RoleUpdate { wiki_id, member_id, new_role } => {
                // Handle role update notification
                if member_id == self.node_id {
                    // Update our local membership record
                    if let Some(membership) = self.my_memberships.iter_mut()
                        .find(|m| m.wiki_id == wiki_id || m.wiki_id.starts_with(&format!("{}@", wiki_id))) {
                        let old_role = membership.role.clone();
                        membership.role = new_role.clone();
                        println!("Your role in wiki {} has been updated from {:?} to {:?}", wiki_id, old_role, new_role);
                    }

                    // Update the wiki - check for remote wikis stored with @ suffix
                    // First try to find the wiki with a remote reference
                    let mut wiki_found = false;
                    for (stored_wiki_id, wiki) in self.wikis.iter_mut() {
                        if stored_wiki_id == &wiki_id || stored_wiki_id.starts_with(&format!("{}@", wiki_id)) {
                            wiki.members.insert(member_id.clone(), new_role.clone());
                            wiki_found = true;
                            break;
                        }
                    }

                    if !wiki_found {
                        println!("Wiki {} not found locally for role update", wiki_id);
                    }
                }
                WikiResponse::Success(true)
            }
            WikiMessage::SearchPages { wiki_id, query } => {
                // Search pages in the wiki
                match self.wikis.get(&wiki_id) {
                    Some(_wiki) => {
                        // Allow search for both public and private wikis
                        // In production, should verify requester is a member for private wikis
                        let query_lower = query.to_lowercase();
                        let mut results: Vec<SearchResult> = Vec::new();

                        for (page_key, page) in &self.pages {
                            if page.wiki_id != wiki_id {
                                continue;
                            }

                            // Search in path and content
                            let path_matches = page.path.to_lowercase().contains(&query_lower);
                            
                            // Decode CRDT content to get actual text
                            let content = if let Some(doc) = self.active_docs.get(page_key) {
                                // Use existing doc
                                let text = doc.get_or_insert_text("content");
                                let content_string = text.get_string(&doc.transact());
                                content_string
                            } else {
                                // Create temporary doc to decode content
                                let doc = Doc::new();
                                {
                                    let mut txn = doc.transact_mut();
                                    if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                                        let _ = txn.apply_update(update);
                                    }
                                }
                                let text = doc.get_or_insert_text("content");
                                let content_string = text.get_string(&doc.transact());
                                content_string
                            };
                            
                            let content_lower = content.to_lowercase();
                            let content_matches = content_lower.contains(&query_lower);

                            if path_matches || content_matches {
                                // Create a snippet around the match
                                let snippet = if content_matches {
                                    if let Some(pos) = content_lower.find(&query_lower) {
                                        let start = pos.saturating_sub(50);
                                        let end = (pos + query_lower.len() + 50).min(content.len());
                                        let mut snippet_text = content[start..end].to_string();
                                        if start > 0 {
                                            snippet_text = format!("...{}", snippet_text);
                                        }
                                        if end < content.len() {
                                            snippet_text = format!("{}...", snippet_text);
                                        }
                                        snippet_text
                                    } else {
                                        content.chars().take(100).collect::<String>() + "..."
                                    }
                                } else {
                                    // If match is only in path, show beginning of content
                                    content.chars().take(100).collect::<String>() + "..."
                                };

                                results.push(SearchResult {
                                    path: page.path.clone(),
                                    updated_by: page.current_version.updated_by.clone(),
                                    updated_at: page.current_version.updated_at.clone(),
                                    snippet,
                                });
                            }
                        }

                        WikiResponse::SearchResults(results)
                    }
                    None => WikiResponse::Error("Wiki not found".to_string()),
                }
            }
        };

        Ok(serde_json::to_string(&response).unwrap().into_bytes())
    }


    #[http]
    async fn create_wiki(&mut self, body: String) -> Result<String, String> {
        let req: CreateWikiRequest = serde_json::from_str(&body)
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

        Ok(serde_json::to_string(&serde_json::json!({
            "wiki_id": wiki_id,
            "wiki": wiki
        })).unwrap())
    }

    #[http]
    async fn list_wikis(&mut self) -> Result<String, String> {
        let mut all_wikis = Vec::new();

        for membership in &self.my_memberships {
            // First check if the wiki exists locally
            let base_wiki_id = if membership.wiki_id.contains('@') {
                membership.wiki_id.split('@').next().unwrap_or(&membership.wiki_id)
            } else {
                &membership.wiki_id
            };

            // Check if we have the wiki stored locally
            if let Some(wiki) = self.wikis.get(&membership.wiki_id) {
                // Found wiki with exact membership ID (could be wiki_id or wiki_id@node)
                all_wikis.push(wiki.clone());
                continue;
            }

            // Also check base wiki ID for backwards compatibility
            if let Some(wiki) = self.wikis.get(base_wiki_id) {
                all_wikis.push(wiki.clone());
                continue;
            }

            // Wiki doesn't exist locally, check if this is a remote wiki reference
            if membership.wiki_id.contains('@') {
                // Parse the remote wiki reference
                let parts: Vec<&str> = membership.wiki_id.split('@').collect();
                if parts.len() == 2 {
                    let wiki_id = parts[0];
                    let node_id = parts[1];

                    // Try to fetch the actual wiki data from the remote node
                    let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                    let message = WikiMessage::GetWikiData {
                        wiki_id: wiki_id.to_string(),
                    };

                    if let Ok(message_body) = serde_json::to_string(&message).map(|s| s.into_bytes()) {
                        match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                            Ok(Ok(response_bytes)) => {
                                if let Ok(response_str) = String::from_utf8(response_bytes) {
                                    if let Ok(WikiResponse::WikiData(mut wiki)) = serde_json::from_str::<WikiResponse>(&response_str) {
                                        // Override the ID to include the remote node reference
                                        wiki.id = format!("{}@{}", wiki.id, node_id);
                                        all_wikis.push(wiki);
                                        continue;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    // Fallback if we can't fetch the data
                    let remote_wiki = Wiki {
                        id: wiki_id.to_string(),
                        name: format!("Remote Wiki on {}", node_id),
                        description: format!("Wiki hosted on node {}", node_id),
                        is_public: true,
                        created_by: node_id.to_string(),
                        created_at: membership.joined_at.clone(),
                        members: HashMap::new(),
                    };
                    all_wikis.push(remote_wiki);
                }
            }
        }

        Ok(serde_json::to_string(&all_wikis).unwrap())
    }

    #[http]
    async fn get_wiki(&mut self, body: String) -> Result<String, String> {
        let req: GetWikiRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // First, check if this wiki exists locally
        if let Some(wiki) = self.wikis.get(&req.wiki_id) {
            // It's a local wiki, check permission and return it
            self.check_permission(&req.wiki_id, WikiRole::Reader)?;
            return Ok(serde_json::to_string(&wiki).unwrap());
        }

        // Not a local wiki, check if we have a membership for a remote wiki
        let membership = self.my_memberships.iter()
            .find(|m| m.wiki_id == req.wiki_id || m.wiki_id.starts_with(&format!("{}@", req.wiki_id)));

        if let Some(membership) = membership {
            // Check if this is a remote wiki reference
            if membership.wiki_id.contains('@') {
                let parts: Vec<&str> = membership.wiki_id.split('@').collect();
                if parts.len() == 2 {
                    let wiki_id = parts[0];
                    let node_id = parts[1];

                    // Fetch the actual wiki data from the remote node
                    let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                    let message = WikiMessage::GetWikiData {
                        wiki_id: wiki_id.to_string(),
                    };

                    let message_body = serde_json::to_string(&message)
                        .map_err(|e| format!("Failed to serialize message: {}", e))?
                        .into_bytes();

                    match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                        Ok(Ok(response_bytes)) => {
                            let response_str = String::from_utf8(response_bytes)
                                .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                            match serde_json::from_str::<WikiResponse>(&response_str) {
                                Ok(WikiResponse::WikiData(mut wiki)) => {
                                    // Override the ID to include the remote node reference
                                    wiki.id = format!("{}@{}", wiki.id, node_id);
                                    return Ok(serde_json::to_string(&wiki).unwrap());
                                }
                                _ => {
                                    // Fallback to basic representation
                                    let remote_wiki = Wiki {
                                        id: wiki_id.to_string(),
                                        name: format!("Remote Wiki on {}", node_id),
                                        description: format!("Wiki hosted on node {}", node_id),
                                        is_public: true,
                                        created_by: node_id.to_string(),
                                        created_at: membership.joined_at.clone(),
                                        members: HashMap::from([(self.node_id.clone(), membership.role.clone())]),
                                    };
                                    return Ok(serde_json::to_string(&remote_wiki).unwrap());
                                }
                            }
                        }
                        _ => {
                            // Fallback to basic representation if we can't reach the remote node
                            let remote_wiki = Wiki {
                                id: wiki_id.to_string(),
                                name: format!("Remote Wiki on {}", node_id),
                                description: format!("Wiki hosted on node {}", node_id),
                                is_public: true,
                                created_by: node_id.to_string(),
                                created_at: membership.joined_at.clone(),
                                members: HashMap::from([(self.node_id.clone(), membership.role.clone())]),
                            };
                            return Ok(serde_json::to_string(&remote_wiki).unwrap());
                        }
                    }
                }
            }
        }

        // Wiki not found
        Err("Wiki not found".to_string())
    }

    #[http]
    async fn join_wiki(&mut self, body: String) -> Result<String, String> {
        let req: JoinWikiRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        // Check if this is a remote wiki
        if let Some(remote_node_id) = &req.node_id {
            if remote_node_id != &self.node_id {
                // This is a remote wiki - fetch its info first
                let target_address = Address::new(remote_node_id, ("wiki", "wiki", "sys"));

                // Create message to get public wiki info
                let message = WikiMessage::GetPublicWiki {
                    wiki_id: req.wiki_id.clone(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                // Query the remote node for wiki info
                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::WikiInfo(wiki_json)) => {
                                // Parse the wiki info to verify it exists and is public
                                if let Ok(wiki_info) = serde_json::from_value::<serde_json::Value>(wiki_json) {
                                    if let Some(is_public) = wiki_info.get("is_public").and_then(|v| v.as_bool()) {
                                        if !is_public {
                                            return Err("Cannot join private wiki on remote node".to_string());
                                        }
                                    }

                                    // Now send a join request to actually join the wiki
                                    let join_message = WikiMessage::JoinPublicWiki {
                                        wiki_id: req.wiki_id.clone(),
                                        user_id: self.node_id.clone(),
                                    };

                                    let join_body = serde_json::to_string(&join_message)
                                        .map_err(|e| format!("Failed to serialize join message: {}", e))?
                                        .into_bytes();

                                    match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, join_body).await {
                                        Ok(Ok(join_response)) => {
                                            let join_response_str = String::from_utf8(join_response)
                                                .map_err(|e| format!("Failed to convert join response to string: {}", e))?;
                                            match serde_json::from_str::<WikiResponse>(&join_response_str) {
                                                Ok(WikiResponse::Success(true)) => {
                                                    // Store the membership with remote node reference
                                                    // For public wikis, we get Writer role
                                                    self.my_memberships.push(WikiMembership {
                                                        wiki_id: format!("{}@{}", req.wiki_id, remote_node_id),
                                                        role: WikiRole::Writer,
                                                        joined_at: Utc::now().to_rfc3339(),
                                                    });

                                                    println!("Successfully joined remote wiki {} on node {}", req.wiki_id, remote_node_id);
                                                    return Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap());
                                                }
                                                _ => {
                                                    return Err("Failed to join remote wiki".to_string());
                                                }
                                            }
                                        }
                                        _ => {
                                            return Err("Failed to join remote wiki".to_string());
                                        }
                                    }
                                }
                            }
                            Ok(WikiResponse::Error(err)) => {
                                return Err(format!("Remote node error: {}", err));
                            }
                            _ => {
                                return Err("Unexpected response from remote node".to_string());
                            }
                        }
                    }
                    Ok(Err(err)) => {
                        return Err(format!("Remote node returned error: {}", err));
                    }
                    Err(e) => {
                        return Err(format!("Failed to contact remote node: {:?}", e));
                    }
                }

                return Err("Failed to join remote wiki".to_string());
            }
        }

        // Local wiki join
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

        Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap())
    }

    #[http]
    async fn leave_wiki(&mut self, body: String) -> Result<String, String> {
        let req: LeaveWikiRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        if let Some(wiki) = self.wikis.get_mut(&req.wiki_id) {
            wiki.members.remove(&self.node_id);
        }

        self.my_memberships.retain(|m| m.wiki_id != req.wiki_id);

        Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap())
    }

    #[http]
    async fn update_wiki_settings(&mut self, body: String) -> Result<String, String> {
        let req: UpdateWikiRequest = serde_json::from_str(&body)
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

        Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap())
    }

    #[http]
    async fn manage_member(&mut self, body: String) -> Result<String, String> {
        let req: ManageMemberRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        self.check_permission(&req.wiki_id, WikiRole::Admin)?;

        let wiki = self.wikis.get_mut(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;

        // Store the previous role for comparison
        let previous_role = wiki.members.get(&req.member_id).cloned();
        let wiki_name = wiki.name.clone();
        let wiki_id = wiki.id.clone();

        match req.action.as_str() {
            "add" => {
                if let Some(role) = req.role.clone() {
                    wiki.members.insert(req.member_id.clone(), role.clone());

                    // Send notification to the new member
                    if req.member_id != self.node_id {
                        let target_address = Address::new(&req.member_id, ("wiki", "wiki", "sys"));
                        let message = WikiMessage::RoleUpdate {
                            wiki_id: wiki_id.clone(),
                            member_id: req.member_id.clone(),
                            new_role: role,
                        };

                        if let Ok(message_body) = serde_json::to_string(&message).map(|s| s.into_bytes()) {
                            // Fire and forget notification
                            let _ = caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await;
                        }
                    }
                }
            }
            "remove" => {
                wiki.members.remove(&req.member_id);

                // Notify the removed member
                if req.member_id != self.node_id && previous_role.is_some() {
                    // For removed members, we can send a special notification
                    // Note: They won't have access to the wiki anymore, but they should know they were removed
                    println!("Member {} removed from wiki {}", req.member_id, wiki_name);
                }
            }
            "update" => {
                if let Some(role) = req.role.clone() {
                    wiki.members.insert(req.member_id.clone(), role.clone());

                    // Send notification if role actually changed
                    if req.member_id != self.node_id && previous_role.as_ref() != Some(&role) {
                        let target_address = Address::new(&req.member_id, ("wiki", "wiki", "sys"));
                        let message = WikiMessage::RoleUpdate {
                            wiki_id: wiki_id.clone(),
                            member_id: req.member_id.clone(),
                            new_role: role,
                        };

                        if let Ok(message_body) = serde_json::to_string(&message).map(|s| s.into_bytes()) {
                            // Fire and forget notification
                            let _ = caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await;
                        }
                    }
                }
            }
            _ => return Err("Invalid action".to_string()),
        }

        Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap())
    }

    #[http]
    async fn create_page(&mut self, body: String) -> Result<String, String> {
        let req: CreatePageRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // Check if this is a remote wiki
        if req.wiki_id.contains('@') {
            let parts: Vec<&str> = req.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];

                // Send create page request to remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::CreatePage {
                    wiki_id: wiki_id.to_string(),
                    path: req.path.clone(),
                    initial_content: req.initial_content.clone(),
                    user_id: self.node_id.clone(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::Success(true)) => {
                                return Ok(serde_json::to_string(&CreatePageResponse {
                                    success: true,
                                    path: req.path.clone(),
                                }).unwrap());
                            }
                            Ok(WikiResponse::Error(err)) => {
                                return Err(format!("Remote error: {}", err));
                            }
                            _ => {
                                return Err("Unexpected response from remote node".to_string());
                            }
                        }
                    }
                    _ => {
                        return Err("Failed to create page on remote wiki".to_string());
                    }
                }
            }
        }

        // Local wiki handling
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

        Ok(serde_json::to_string(&CreatePageResponse {
            success: true,
            path: req.path,
        }).unwrap())
    }

    #[http]
    async fn update_page(&mut self, body: String) -> Result<String, String> {
        let req: UpdatePageRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // Check if this is a remote wiki
        if req.wiki_id.contains('@') {
            let parts: Vec<&str> = req.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];

                // Send update page request to remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::UpdatePage {
                    wiki_id: wiki_id.to_string(),
                    path: req.path.clone(),
                    content: req.content.clone(),
                    user_id: self.node_id.clone(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::Success(true)) => {
                                return Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap());
                            }
                            Ok(WikiResponse::Error(err)) => {
                                return Err(format!("Remote error: {}", err));
                            }
                            _ => {
                                return Err("Unexpected response from remote node".to_string());
                            }
                        }
                    }
                    _ => {
                        return Err("Failed to update page on remote wiki".to_string());
                    }
                }
            }
        }

        // Local wiki handling
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

        Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap())
    }

    #[http]
    async fn get_page(&mut self, body: String) -> Result<String, String> {
        let req: GetPageRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // Check if this is for a remote wiki
        if req.wiki_id.contains('@') {
            let parts: Vec<&str> = req.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];

                // Fetch page from remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::GetWikiPage {
                    wiki_id: wiki_id.to_string(),
                    path: req.path.clone(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::PageData(page_info)) => {
                                return Ok(serde_json::to_string(&page_info).unwrap());
                            }
                            Ok(WikiResponse::Error(err)) => {
                                return Err(format!("Remote error: {}", err));
                            }
                            _ => {
                                return Err("Failed to fetch page from remote wiki".to_string());
                            }
                        }
                    }
                    _ => {
                        return Err("Failed to contact remote wiki".to_string());
                    }
                }
            }
        }

        // Local wiki
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

            Ok(serde_json::to_string(&PageInfo {
                path: page.path.clone(),
                wiki_id: page.wiki_id.clone(),
                content,
                updated_by: page.current_version.updated_by.clone(),
                updated_at: page.current_version.updated_at.clone(),
            }).unwrap())
        } else {
            Ok(serde_json::to_string(&PageInfo {
                path: req.path,
                wiki_id: req.wiki_id,
                content: String::new(),
                updated_by: String::new(),
                updated_at: String::new(),
            }).unwrap())
        }
    }

    #[http]
    async fn list_pages(&mut self, body: String) -> Result<String, String> {
        let req: ListPagesRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // Check if this is for a remote wiki
        if req.wiki_id.contains('@') {
            let parts: Vec<&str> = req.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];

                // Fetch pages from remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::GetWikiPages {
                    wiki_id: wiki_id.to_string(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::PageList(pages)) => {
                                return Ok(serde_json::to_string(&pages).unwrap());
                            }
                            Ok(WikiResponse::Error(err)) => {
                                return Err(format!("Remote error: {}", err));
                            }
                            _ => {
                                return Ok(serde_json::to_string(&Vec::<PageSummary>::new()).unwrap());
                            }
                        }
                    }
                    _ => {
                        return Ok(serde_json::to_string(&Vec::<PageSummary>::new()).unwrap());
                    }
                }
            }
        }

        // Local wiki
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;

        let pages: Vec<PageSummary> = self.pages
            .iter()
            .filter(|(_, page)| page.wiki_id == req.wiki_id)
            .map(|(_, page)| PageSummary {
                path: page.path.clone(),
                updated_by: page.current_version.updated_by.clone(),
                updated_at: page.current_version.updated_at.clone(),
            })
            .collect();

        Ok(serde_json::to_string(&pages).unwrap())
    }

    #[http]
    async fn delete_page(&mut self, body: String) -> Result<String, String> {
        let req: DeletePageRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        self.check_permission(&req.wiki_id, WikiRole::Writer)?;

        let page_key = format!("{}:{}", req.wiki_id, req.path);

        self.pages.remove(&page_key);
        self.active_docs.remove(&page_key);

        Ok(serde_json::to_string(&SuccessResponse { success: true }).unwrap())
    }

    #[http]
    async fn search_pages(&mut self, body: String) -> Result<String, String> {
        #[derive(Deserialize)]
        struct SearchRequest {
            wiki_id: String,
            query: String,
        }

        let req: SearchRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // For remote wikis, perform the search remotely
        if req.wiki_id.contains('@') {
            let parts: Vec<&str> = req.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];

                // Send search request to remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::SearchPages {
                    wiki_id: wiki_id.to_string(),
                    query: req.query.clone(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::SearchResults(results)) => {
                                return Ok(serde_json::to_string(&results).unwrap());
                            }
                            Ok(WikiResponse::Error(err)) => {
                                return Err(format!("Remote search error: {}", err));
                            }
                            _ => {
                                return Err("Unexpected response from remote node".to_string());
                            }
                        }
                    }
                    _ => {
                        return Err("Failed to search remote wiki".to_string());
                    }
                }
            }
        }

        // For local wikis, check permissions and search directly
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;

        let message = WikiMessage::SearchPages {
            wiki_id: req.wiki_id,
            query: req.query,
        };
        
        // Process the message directly instead of calling handle_wiki_message
        match message {
            WikiMessage::SearchPages { wiki_id, query } => {
                match self.wikis.get(&wiki_id) {
                    Some(_) => {
                        let query_lower = query.to_lowercase();
                        let mut results: Vec<SearchResult> = Vec::new();

                        for (page_key, page) in &self.pages {
                            if page.wiki_id != wiki_id {
                                continue;
                            }

                            // Search in path and content
                            let path_matches = page.path.to_lowercase().contains(&query_lower);
                            
                            // Decode CRDT content to get actual text
                            let content = if let Some(doc) = self.active_docs.get(page_key) {
                                // Use existing doc
                                let text = doc.get_or_insert_text("content");
                                let content_string = text.get_string(&doc.transact());
                                content_string
                            } else {
                                // Create temporary doc to decode content
                                let doc = Doc::new();
                                {
                                    let mut txn = doc.transact_mut();
                                    if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                                        let _ = txn.apply_update(update);
                                    }
                                }
                                let text = doc.get_or_insert_text("content");
                                let content_string = text.get_string(&doc.transact());
                                content_string
                            };
                            
                            let content_lower = content.to_lowercase();
                            let content_matches = content_lower.contains(&query_lower);

                            if path_matches || content_matches {
                                // Create a snippet around the match
                                let snippet = if content_matches {
                                    if let Some(pos) = content_lower.find(&query_lower) {
                                        let start = pos.saturating_sub(50);
                                        let end = (pos + query_lower.len() + 50).min(content.len());
                                        let mut snippet_text = content[start..end].to_string();
                                        if start > 0 {
                                            snippet_text = format!("...{}", snippet_text);
                                        }
                                        if end < content.len() {
                                            snippet_text = format!("{}...", snippet_text);
                                        }
                                        snippet_text
                                    } else {
                                        content.chars().take(100).collect::<String>() + "..."
                                    }
                                } else {
                                    // If match is only in path, show beginning of content
                                    content.chars().take(100).collect::<String>() + "..."
                                };

                                results.push(SearchResult {
                                    path: page.path.clone(),
                                    updated_by: page.current_version.updated_by.clone(),
                                    updated_at: page.current_version.updated_at.clone(),
                                    snippet,
                                });
                            }
                        }

                        Ok(serde_json::to_string(&results).unwrap())
                    }
                    None => Err("Wiki not found".to_string()),
                }
            }
            _ => Err("Invalid message type".to_string()),
        }
    }
    
    #[http]
    async fn search_pages_disabled(&mut self, body: String) -> Result<String, String> {
        let req: SearchRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // For remote wikis, perform the search remotely
        if req.wiki_id.contains('@') {
            let parts: Vec<&str> = req.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];

                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::SearchPages {
                    wiki_id: wiki_id.to_string(),
                    query: req.query.clone(),
                };

                let message_body = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?
                    .into_bytes();

                match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                    Ok(Ok(response_bytes)) => {
                        let response_str = String::from_utf8(response_bytes)
                            .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                        match serde_json::from_str::<WikiResponse>(&response_str) {
                            Ok(WikiResponse::SearchResults(results)) => {
                                return Ok(serde_json::to_string(&results).unwrap());
                            }
                            Ok(WikiResponse::Error(e)) => return Err(e),
                            _ => return Err("Unexpected response from remote search".to_string()),
                        }
                    }
                    _ => return Err("Failed to search remote wiki".to_string()),
                }
            }
        }

        // For local wikis, check permissions
        self.check_permission(&req.wiki_id, WikiRole::Reader)?;

        let query_lower = req.query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();

        // Search through all pages in the wiki
        for (page_key, page) in &self.pages {
            if page.wiki_id != req.wiki_id {
                continue;
            }

            // Search in path and content
            let path_matches = page.path.to_lowercase().contains(&query_lower);
            
            // Decode CRDT content to get actual text
            let content = if let Some(doc) = self.active_docs.get(page_key) {
                // Use existing doc
                let text = doc.get_or_insert_text("content");
                let txn = doc.transact();
                text.get_string(&txn)
            } else {
                // Create temporary doc to decode content
                let doc = Doc::new();
                {
                    let mut txn = doc.transact_mut();
                    if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                        let _ = txn.apply_update(update);
                    }
                }
                let text = doc.get_or_insert_text("content");
                let txn = doc.transact();
                text.get_string(&txn)
            };
            
            let content_lower = content.to_lowercase();
            let content_matches = content_lower.contains(&query_lower);

            if path_matches || content_matches {
                // Create a snippet around the match
                let snippet = if content_matches {
                    if let Some(pos) = content_lower.find(&query_lower) {
                        let start = pos.saturating_sub(50);
                        let end = (pos + query_lower.len() + 50).min(content.len());
                        let mut snippet_text = content[start..end].to_string();
                        if start > 0 {
                            snippet_text = format!("...{}", snippet_text);
                        }
                        if end < content.len() {
                            snippet_text = format!("{}...", snippet_text);
                        }
                        snippet_text
                    } else {
                        content.chars().take(100).collect::<String>() + "..."
                    }
                } else {
                    // If match is only in path, show beginning of content
                    content.chars().take(100).collect::<String>() + "..."
                };

                results.push(SearchResult {
                    path: page.path.clone(),
                    updated_by: page.current_version.updated_by.clone(),
                    updated_at: page.current_version.updated_at.clone(),
                    snippet,
                });
            }
        }

        Ok(serde_json::to_string(&results).unwrap())
    }

    #[http]
    async fn find_wikis_by_user(&mut self, body: String) -> Result<String, String> {
        let req: FindWikisByUserRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        println!("Finding wikis for user: {}", req.username);

        // First, check local wikis
        let all_wikis: Vec<serde_json::Value> = self.wikis
            .values()
            .filter(|wiki| wiki.is_public && wiki.members.contains_key(&req.username))
            .map(|wiki| serde_json::json!({
                "id": wiki.id,
                "name": wiki.name,
                "description": wiki.description,
                "is_public": wiki.is_public,
                "member_count": wiki.members.len(),
                "user_role": wiki.members.get(&req.username),
                "node_id": self.node_id
            }))
            .collect();

        println!("Found {} local wikis", all_wikis.len());

        // If the username is our own node, we already have the local results
        if req.username == self.node_id {
            println!("Username is our own node, skipping remote query");
            return Ok(serde_json::to_string(&all_wikis).unwrap());
        }

        // Try to query the target node for their public wikis
        // The username is expected to be a node ID (e.g., "alice.os")
        // We'll construct the wiki process address from it
        let target_address = Address::new(&req.username, ("wiki", "wiki", "sys"));

        println!("Querying node {} for public wikis", target_address);

        // Create P2P message
        let message = WikiMessage::FindWikisByUser {
            username: req.username.clone(),
        };

        // Serialize the message
        let message_body = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?
            .into_bytes();

        // Use the generated RPC function which properly wraps the message
        match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
            Ok(Ok(response_bytes)) => {
                // Deserialize the response
                let response_str = String::from_utf8(response_bytes)
                    .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                match serde_json::from_str::<WikiResponse>(&response_str) {
                    Ok(WikiResponse::WikiList(wikis)) => {
                        println!("Received {} wikis from {}", wikis.len(), target_address);
                        let mut combined_wikis = all_wikis;
                        combined_wikis.extend(wikis);
                        println!("Returning {} total wikis", combined_wikis.len());
                        Ok(serde_json::to_string(&combined_wikis).unwrap())
                    }
                    Ok(WikiResponse::Error(err)) => {
                        println!("Remote node returned error: {}", err);
                        Ok(serde_json::to_string(&all_wikis).unwrap())
                    }
                    Ok(_) => {
                        println!("Unexpected response type from remote node");
                        Ok(serde_json::to_string(&all_wikis).unwrap())
                    }
                    Err(e) => {
                        println!("Failed to deserialize response: {}", e);
                        Ok(serde_json::to_string(&all_wikis).unwrap())
                    }
                }
            }
            Ok(Err(err)) => {
                println!("Remote node returned error: {}", err);
                Ok(serde_json::to_string(&all_wikis).unwrap())
            }
            Err(e) => {
                println!("Failed to query node {}: {:?}", target_address, e);
                Ok(serde_json::to_string(&all_wikis).unwrap())
            }
        }
    }

    #[http]
    async fn get_public_wiki(&mut self, body: String) -> Result<String, String> {
        let req: GetPublicWikiRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        let wiki = self.wikis.get(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;

        if !wiki.is_public {
            return Err("Wiki is not public".to_string());
        }

        let wiki_info = serde_json::json!({
            "id": wiki.id,
            "name": wiki.name,
            "description": wiki.description,
            "is_public": wiki.is_public,
            "member_count": wiki.members.len(),
        });

        Ok(serde_json::to_string(&wiki_info).unwrap())
    }

    #[http]
    async fn search_wiki(&mut self, body: String) -> Result<String, String> {
        let req: SearchRequest = serde_json::from_str(&body)
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
                results.push(SearchResult {
                    path: page.path.clone(),
                    updated_by: page.current_version.updated_by.clone(),
                    updated_at: page.current_version.updated_at.clone(),
                    snippet: content.chars().take(200).collect::<String>(),
                });
            }
        }

        Ok(serde_json::to_string(&results).unwrap())
    }

    #[http]
    async fn invite_user(&mut self, body: String) -> Result<String, String> {
        let req: InviteUserRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        self.check_permission(&req.wiki_id, WikiRole::Admin)?;

        let wiki = self.wikis.get(&req.wiki_id)
            .ok_or_else(|| "Wiki not found".to_string())?;

        // Check if user is already a member
        // req.invitee_id should be a node ID (e.g., "alice.os"), not a full address
        if wiki.members.contains_key(&req.invitee_id) {
            return Err("User is already a member of this wiki".to_string());
        }

        // Check if there's already a pending invite
        let pending_exists = self.invites.values().any(|inv| {
            inv.wiki_id == req.wiki_id &&
            inv.invitee_id == req.invitee_id &&
            inv.status == InviteStatus::Pending
        });

        if pending_exists {
            return Err("An invite for this user already exists".to_string());
        }

        let invite_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        let invite = WikiInvite {
            id: invite_id.clone(),
            wiki_id: req.wiki_id.clone(),
            wiki_name: wiki.name.clone(),
            inviter_id: self.node_id.clone(),
            invitee_id: req.invitee_id.clone(),
            created_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
            status: InviteStatus::Pending,
        };

        self.invites.insert(invite_id.clone(), invite.clone());

        // Send the invite to the invitee via P2P
        let target_address = Address::new(&req.invitee_id, ("wiki", "wiki", "sys"));
        let message = WikiMessage::SendInvite {
            invite: invite.clone(),
            wiki: wiki.clone(),
        };

        let message_body = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize invite message: {}", e))?
            .into_bytes();

        match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
            Ok(Ok(response_bytes)) => {
                let response_str = String::from_utf8(response_bytes)
                    .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                match serde_json::from_str::<WikiResponse>(&response_str) {
                    Ok(WikiResponse::Success(true)) => {
                        Ok(serde_json::to_string(&InviteUserResponse {
                            invite_id,
                            success: true,
                        }).unwrap())
                    }
                    Ok(WikiResponse::Error(err)) => {
                        // Remove the invite from our local storage if sending failed
                        self.invites.remove(&invite_id);
                        Err(format!("Failed to send invite: {}", err))
                    }
                    _ => {
                        self.invites.remove(&invite_id);
                        Err("Failed to send invite to user".to_string())
                    }
                }
            }
            _ => {
                self.invites.remove(&invite_id);
                Err("Failed to reach invitee node".to_string())
            }
        }
    }

    #[http]
    async fn respond_to_invite(&mut self, body: String) -> Result<String, String> {
        let req: RespondToInviteRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;
        let invite = self.invites.get_mut(&req.invite_id)
            .ok_or_else(|| "Invite not found".to_string())?;

        // Check if the invite is for this user
        if invite.invitee_id != self.node_id {
            return Err("This invite is not for you".to_string());
        }

        // Check if invite is still pending
        if invite.status != InviteStatus::Pending {
            return Err("Invite has already been processed".to_string());
        }

        // Check if invite has expired
        let expires_at = chrono::DateTime::parse_from_rfc3339(&invite.expires_at)
            .map_err(|e| format!("Invalid expiry date: {}", e))?;
        if Utc::now() > expires_at {
            invite.status = InviteStatus::Expired;
            return Err("Invite has expired".to_string());
        }

        if req.accept {
            // Store membership with remote reference if this wiki is from another node
            let membership_wiki_id = if invite.inviter_id != self.node_id {
                format!("{}@{}", invite.wiki_id, invite.inviter_id)
            } else {
                invite.wiki_id.clone()
            };

            self.my_memberships.push(WikiMembership {
                wiki_id: membership_wiki_id.clone(),
                role: WikiRole::Reader,
                joined_at: Utc::now().to_rfc3339(),
            });

            // Update wiki member list - check both with and without @ suffix
            if let Some(wiki) = self.wikis.get_mut(&membership_wiki_id) {
                // Remote wiki stored with @ suffix
                wiki.members.insert(self.node_id.clone(), WikiRole::Reader);
            } else if let Some(wiki) = self.wikis.get_mut(&invite.wiki_id) {
                // Local wiki (we're the owner)
                wiki.members.insert(self.node_id.clone(), WikiRole::Reader);
            }

            invite.status = InviteStatus::Accepted;
        } else {
            invite.status = InviteStatus::Rejected;
        }

        // Send notification back to inviter
        let inviter_id = invite.inviter_id.clone();
        let invite_id = invite.id.clone();
        let invite_status = invite.status.clone();
        let invitee_id = self.node_id.clone();

        // Send async notification to inviter
        let target_address = Address::new(&inviter_id, ("wiki", "wiki", "sys"));
        let message = WikiMessage::InviteResponse {
            invite_id,
            status: invite_status.clone(),
            invitee_id,
        };

        if let Ok(message_body) = serde_json::to_string(&message).map(|s| s.into_bytes()) {
            // Fire and forget - we don't wait for the response
            let _ = caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await;
        }

        Ok(serde_json::to_string(&RespondToInviteResponse {
            success: true,
            status: invite_status,
        }).unwrap())
    }

    #[http]
    async fn list_my_invites(&mut self) -> Result<String, String> {
        let my_invites: Vec<InviteInfo> = self.invites
            .values()
            .filter(|inv| inv.invitee_id == self.node_id)
            .filter(|inv| inv.status == InviteStatus::Pending)
            .map(|inv| {
                // Check if expired
                let is_expired = chrono::DateTime::parse_from_rfc3339(&inv.expires_at)
                    .map(|exp| Utc::now() > exp)
                    .unwrap_or(false);

                InviteInfo {
                    id: inv.id.clone(),
                    wiki_id: inv.wiki_id.clone(),
                    wiki_name: inv.wiki_name.clone(),
                    inviter_id: inv.inviter_id.clone(),
                    created_at: inv.created_at.clone(),
                    expires_at: inv.expires_at.clone(),
                    is_expired,
                }
            })
            .collect();

        Ok(serde_json::to_string(&my_invites).unwrap())
    }

    #[http]
    async fn search_all_wikis(&mut self, body: String) -> Result<String, String> {
        #[derive(Deserialize)]
        struct SearchAllRequest {
            query: String,
        }

        let req: SearchAllRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        let query_lower = req.query.to_lowercase();

        #[derive(Serialize)]
        struct GlobalSearchResult {
            wiki_id: String,
            wiki_name: String,
            path: String,
            updated_by: String,
            updated_at: String,
            snippet: String,
        }

        let mut all_results: Vec<GlobalSearchResult> = Vec::new();

        // Search in local wikis where user has access
        for (wiki_id, wiki) in &self.wikis {
            // Check if user has access to this wiki
            let has_access = wiki.is_public ||
                             wiki.members.contains_key(&self.node_id) ||
                             self.my_memberships.iter().any(|m| m.wiki_id == *wiki_id || m.wiki_id.starts_with(&format!("{}@", wiki_id)));

            if !has_access {
                continue;
            }

            // Search pages in this wiki directly
            let mut results: Vec<SearchResult> = Vec::new();

            for (page_key, page) in &self.pages {
                if page.wiki_id != *wiki_id {
                    continue;
                }

                // Search in path and content
                let path_matches = page.path.to_lowercase().contains(&query_lower);
                
                // Decode CRDT content to get actual text
                let content = if let Some(doc) = self.active_docs.get(page_key) {
                    // Use existing doc
                    let text = doc.get_or_insert_text("content");
                    let content_string = text.get_string(&doc.transact());
                    content_string
                } else {
                    // Create temporary doc to decode content
                    let doc = Doc::new();
                    {
                        let mut txn = doc.transact_mut();
                        if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                            let _ = txn.apply_update(update);
                        }
                    }
                    let text = doc.get_or_insert_text("content");
                    let content_string = text.get_string(&doc.transact());
                    content_string
                };
                
                let content_lower = content.to_lowercase();
                let content_matches = content_lower.contains(&query_lower);

                if path_matches || content_matches {
                    // Create a snippet around the match
                    let snippet = if content_matches {
                        if let Some(pos) = content_lower.find(&query_lower) {
                            let start = pos.saturating_sub(50);
                            let end = (pos + query_lower.len() + 50).min(content.len());
                            let mut snippet_text = content[start..end].to_string();
                            if start > 0 {
                                snippet_text = format!("...{}", snippet_text);
                            }
                            if end < content.len() {
                                snippet_text = format!("{}...", snippet_text);
                            }
                            snippet_text
                        } else {
                            content.chars().take(100).collect::<String>() + "..."
                        }
                    } else {
                        // If match is only in path, show beginning of content
                        content.chars().take(100).collect::<String>() + "..."
                    };

                    results.push(SearchResult {
                        path: page.path.clone(),
                        updated_by: page.current_version.updated_by.clone(),
                        updated_at: page.current_version.updated_at.clone(),
                        snippet,
                    });
                }
            }

            // Add results from this wiki to global results
            for result in results {
                all_results.push(GlobalSearchResult {
                    wiki_id: wiki_id.clone(),
                    wiki_name: wiki.name.clone(),
                    path: result.path,
                    updated_by: result.updated_by,
                    updated_at: result.updated_at,
                    snippet: result.snippet,
                });
            }
        }

        // Search remote wikis that user is a member of
        for membership in &self.my_memberships {
            // Skip local wikis (already searched above)
            if !membership.wiki_id.contains('@') {
                continue;
            }
            
            // Parse remote wiki reference
            let parts: Vec<&str> = membership.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];
                
                // Send search request to remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::SearchPages {
                    wiki_id: wiki_id.to_string(),
                    query: req.query.clone(),
                };
                
                if let Ok(message_body) = serde_json::to_string(&message).map(|s| s.into_bytes()) {
                    match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                        Ok(Ok(response_bytes)) => {
                            if let Ok(response_str) = String::from_utf8(response_bytes) {
                                if let Ok(WikiResponse::SearchResults(results)) = serde_json::from_str::<WikiResponse>(&response_str) {
                                    // Get wiki name from stored remote wiki
                                    let wiki_name = if let Some(wiki) = self.wikis.get(&membership.wiki_id) {
                                        wiki.name.clone()
                                    } else {
                                        format!("Remote Wiki on {}", node_id)
                                    };
                                    
                                    // Add remote results to global results
                                    for result in results {
                                        all_results.push(GlobalSearchResult {
                                            wiki_id: membership.wiki_id.clone(),
                                            wiki_name: wiki_name.clone(),
                                            path: result.path,
                                            updated_by: result.updated_by,
                                            updated_at: result.updated_at,
                                            snippet: result.snippet,
                                        });
                                    }
                                }
                            }
                        }
                        _ => {} // Ignore errors from remote searches
                    }
                }
            }
        }

        Ok(serde_json::to_string(&all_results).unwrap())
    }
    
    #[http]
    async fn search_all_wikis_disabled(&mut self, body: String) -> Result<String, String> {
        #[derive(Deserialize)]
        struct SearchAllRequest {
            query: String,
        }

        let req: SearchAllRequest = serde_json::from_str(&body)
            .map_err(|e| format!("Invalid request: {}", e))?;

        let query_lower = req.query.to_lowercase();

        #[derive(Serialize)]
        struct GlobalSearchResult {
            wiki_id: String,
            wiki_name: String,
            path: String,
            updated_by: String,
            updated_at: String,
            snippet: String,
        }

        let mut all_results: Vec<GlobalSearchResult> = Vec::new();

        // Search in local wikis where user has access
        for (wiki_id, wiki) in &self.wikis {
            // Check if user has access to this wiki
            let has_access = wiki.is_public ||
                             wiki.members.contains_key(&self.node_id) ||
                             self.my_memberships.iter().any(|m| m.wiki_id == *wiki_id || m.wiki_id.starts_with(&format!("{}@", wiki_id)));

            if !has_access {
                continue;
            }

            // Search pages in this wiki
            for (_, page) in &self.pages {
                if page.wiki_id != *wiki_id {
                    continue;
                }

                // Search in path and content
                let path_matches = page.path.to_lowercase().contains(&query_lower);
                
                // Get page key
                let page_key_str = format!("{}:{}", wiki_id, page.path);
                
                // Decode CRDT content to get actual text
                let content = if let Some(doc) = self.active_docs.get(&page_key_str) {
                    // Use existing doc
                    let text = doc.get_or_insert_text("content");
                    text.get_string(&doc.transact())
                } else {
                    // Create temporary doc to decode content
                    let doc = Doc::new();
                    {
                        let mut txn = doc.transact_mut();
                        if let Ok(update) = yrs::Update::decode_v1(&page.yrs_doc) {
                            let _ = txn.apply_update(update);
                        }
                    }
                    let text = doc.get_or_insert_text("content");
                    let content_string = text.get_string(&doc.transact());
                    content_string
                };
                
                let content_lower = content.to_lowercase();
                let content_matches = content_lower.contains(&query_lower);

                if path_matches || content_matches {
                    // Create a snippet around the match
                    let snippet = if content_matches {
                        if let Some(pos) = content_lower.find(&query_lower) {
                            let start = pos.saturating_sub(50);
                            let end = (pos + query_lower.len() + 50).min(content.len());
                            let mut snippet_text = content[start..end].to_string();
                            if start > 0 {
                                snippet_text = format!("...{}", snippet_text);
                            }
                            if end < content.len() {
                                snippet_text = format!("{}...", snippet_text);
                            }
                            snippet_text
                        } else {
                            content.chars().take(100).collect::<String>() + "..."
                        }
                    } else {
                        // If match is only in path, show beginning of content
                        content.chars().take(100).collect::<String>() + "..."
                    };

                    all_results.push(GlobalSearchResult {
                        wiki_id: wiki_id.clone(),
                        wiki_name: wiki.name.clone(),
                        path: page.path.clone(),
                        updated_by: page.current_version.updated_by.clone(),
                        updated_at: page.current_version.updated_at.clone(),
                        snippet,
                    });
                }
            }
        }

        // Search remote wikis that user is a member of
        for membership in &self.my_memberships {
            // Skip local wikis (already searched above)
            if !membership.wiki_id.contains('@') {
                continue;
            }
            
            // Parse remote wiki reference
            let parts: Vec<&str> = membership.wiki_id.split('@').collect();
            if parts.len() == 2 {
                let wiki_id = parts[0];
                let node_id = parts[1];
                
                // Send search request to remote node
                let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
                let message = WikiMessage::SearchPages {
                    wiki_id: wiki_id.to_string(),
                    query: req.query.clone(),
                };
                
                if let Ok(message_body) = serde_json::to_string(&message).map(|s| s.into_bytes()) {
                    match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
                        Ok(Ok(response_bytes)) => {
                            if let Ok(response_str) = String::from_utf8(response_bytes) {
                                if let Ok(WikiResponse::SearchResults(results)) = serde_json::from_str::<WikiResponse>(&response_str) {
                                    // Get wiki name from remote node
                                    let wiki_name = if let Ok(wiki_data) = self.get_remote_wiki_data(wiki_id, node_id).await {
                                        wiki_data.name
                                    } else {
                                        format!("Remote Wiki on {}", node_id)
                                    };
                                    
                                    // Add remote results to global results
                                    for result in results {
                                        all_results.push(GlobalSearchResult {
                                            wiki_id: membership.wiki_id.clone(),
                                            wiki_name: wiki_name.clone(),
                                            path: result.path,
                                            updated_by: result.updated_by,
                                            updated_at: result.updated_at,
                                            snippet: result.snippet,
                                        });
                                    }
                                }
                            }
                        }
                        _ => {} // Ignore errors from remote searches
                    }
                }
            }
        }

        Ok(serde_json::to_string(&all_results).unwrap())
    }

}

impl WikiState {
    async fn get_remote_wiki_data(&self, wiki_id: &str, node_id: &str) -> Result<Wiki, String> {
        let target_address = Address::new(node_id, ("wiki", "wiki", "sys"));
        let message = WikiMessage::GetWikiData {
            wiki_id: wiki_id.to_string(),
        };
        
        let message_body = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?
            .into_bytes();
        
        match caller_utils::wiki::handle_wiki_message_remote_rpc(&target_address, message_body).await {
            Ok(Ok(response_bytes)) => {
                let response_str = String::from_utf8(response_bytes)
                    .map_err(|e| format!("Failed to convert response to string: {}", e))?;
                match serde_json::from_str::<WikiResponse>(&response_str) {
                    Ok(WikiResponse::WikiData(wiki)) => Ok(wiki),
                    _ => Err("Failed to get wiki data".to_string()),
                }
            }
            _ => Err("Failed to contact remote node".to_string()),
        }
    }
    
    fn check_permission(&self, wiki_id: &str, required_role: WikiRole) -> Result<(), String> {
        // For remote wikis (format: wiki_id@node_id), check our membership
        if wiki_id.contains('@') {
            // For remote wikis, check if we have a membership
            let has_membership = self.my_memberships.iter()
                .any(|m| m.wiki_id == wiki_id || m.wiki_id.starts_with(&format!("{}@", wiki_id)));

            if has_membership {
                // For remote wikis, we assume reader permissions
                // More sophisticated permission checks would require P2P communication
                match required_role {
                    WikiRole::Reader => return Ok(()),
                    _ => return Err("Cannot modify remote wiki".to_string()),
                }
            } else {
                return Err("Not a member of this wiki".to_string());
            }
        }

        // Local wiki check
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
