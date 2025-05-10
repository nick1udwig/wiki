use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use hyperprocess_macro::hyperprocess;
use hyperware_process_lib::logging::{init_logging, Level};
use hyperware_process_lib::Address;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::{Doc, GetString, ReadTxn, Text, Transact, Update};
use yrs::updates::decoder::Decode;
use chrono::Utc;

// --------------------- Core Data Structures ---------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WikiRole {
    Reader,
    Writer,
    Admin,
    SuperAdmin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiMembership {
    user_id: String, // Node ID
    role: WikiRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wiki {
    id: String, // Unique ID for the wiki
    name: String,
    // Each page path maps to its own Yrs document for CRDT sync.
    // The Vec<u8> will store the serialized Yrs Doc state for persistence.
    pages_data: HashMap<String, Vec<u8>>,
    #[serde(skip)] // Yrs Docs are not directly serializable in this form for state.
    pages: HashMap<String, Doc>, // Runtime representation
    members: HashMap<String, WikiMembership>, // Node ID to membership
    is_public: bool,
    owner_id: String, // Node ID of the superadmin
    // For Yrs, we may need to track state vectors for peers
    peer_state_vectors: HashMap<String, Vec<u8>>, // peer_node_id -> state_vector
    created_at: String, // ISO 8601 format string: no custom WIT types
}

impl Wiki {
    // Helper to get or create a Yrs Doc for a page
    pub fn get_or_create_page_doc(&mut self, path: &str) -> &mut Doc {
        self.pages.entry(path.to_string()).or_insert_with(Doc::new)
    }

    // Method to load pages_data into runtime pages
    pub fn hydrate_pages(&mut self) {
        for (path, data) in &self.pages_data {
            if !data.is_empty() {
                match Update::decode_v1(data) {
                    Ok(update) => {
                        let doc = Doc::new();
                        let _ = doc.transact_mut().apply_update(update);
                        self.pages.insert(path.clone(), doc);
                    }
                    Err(_) => {
                        // Log error: failed to deserialize page data
                        println!("Error: Failed to hydrate page {}", path);
                        // Create a new Doc if hydration fails
                        self.pages.insert(path.clone(), Doc::new());
                    }
                }
            } else {
                // If no data, just create a new empty Doc
                self.pages.insert(path.clone(), Doc::new());
            }
        }
    }

    // Method to prepare pages for serialization
    pub fn dehydrate_pages(&mut self) {
        for (path, doc) in &self.pages {
            let state_vector = doc.transact().state_vector();
            let update = doc.transact().encode_state_as_update_v1(&state_vector);
            self.pages_data.insert(path.clone(), update);
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct WikiState {
    node_id: String,
    wikis: HashMap<String, Wiki>, // wiki_id -> Wiki
    known_peers: HashSet<String>, // For P2P communication
}

// --------------------- API Request/Response Types ---------------------

#[derive(Serialize, Deserialize, Debug)]
struct CreateWikiRequest {
    name: String,
    is_public: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct WikiResponse {
    id: String,
    name: String,
    owner_id: String,
    is_public: bool,
    members: Vec<WikiMembership>, // Simplified for response
    pages: Vec<String>, // List of page paths
    created_at: String, // ISO 8601 timestamp string
}

#[derive(Serialize, Deserialize, Debug)]
struct PageContentResponse {
    path: String,
    content: String, // Markdown content from Yrs doc
    last_modified_by: String,
    last_modified_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UpdatePageRequest {
    wiki_id: String,
    path: String,
    // The content here is the new full markdown text from the user.
    // The backend will use this to update the Yrs doc.
    new_content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenericResponse {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddPeerRequest {
    peer_node_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ManageUserRequest {
    wiki_id: String,
    user_id: String,
    role: WikiRole,
}

// --------------------- P2P Message Types ---------------------

#[derive(Serialize, Deserialize, Debug)]
enum P2PMessage {
    PingPageUpdate { // Sent when a local page is updated
        wiki_id: String,
        page_path: String,
        sender_node_id: String,
    },
    RequestStateVector { // Sent by receiver to the editor
        wiki_id: String,
        page_path: String,
        sender_node_id: String,
    },
    SendUpdate { // Sent by editor to peers that need updating
        wiki_id: String,
        page_path: String,
        update: Vec<u8>, // Encoded Yrs update
        sender_node_id: String,
    },
    ShareWikiData { // For sharing entire wiki state
        wiki_id: String,
        wiki_data: Vec<u8>, // Serialized wiki
        sender_node_id: String,
    },
}

// --------------------- Wiki Implementation ---------------------

#[hyperprocess(
    name = "wiki",
    ui = Some(HttpBindingConfig::default()),
    endpoints = vec![
        Binding::Http {
            path: "/api",
            config: HttpBindingConfig::default(),
        },
        Binding::Ws {
            path: "/ws",
            config: WsBindingConfig::default(),
        }
    ],
    save_config = SaveOptions::EveryNMessage(5),
    wit_world = "process-v1",
)]
impl WikiState {
    #[init]
    async fn initialize(&mut self) {
        // Initialize logging
        init_logging(Level::INFO, Level::DEBUG, None, None, None).unwrap();
        
        // Set the node_id
        // In a real environment, we'd get this from the framework
        self.node_id = Uuid::new_v4().to_string();
        
        println!("Wiki process initialized for node: {}", self.node_id);

        // Hydrate Yrs Docs from persisted state
        for wiki in self.wikis.values_mut() {
            wiki.hydrate_pages();
        }
    }

    // Helper to manually persist state
    fn save_state_if_needed(&mut self) {
        // Before saving, ensure Yrs Docs are serialized into pages_data
        for wiki in self.wikis.values_mut() {
            wiki.dehydrate_pages();
        }
        // The hyperprocess framework handles actual saving based on save_config
    }

    // Helper to get a mutable wiki by ID
    fn get_wiki_mut(&mut self, wiki_id: &str) -> Result<&mut Wiki, String> {
        self.wikis.get_mut(wiki_id).ok_or_else(|| format!("Wiki with ID '{}' not found", wiki_id))
    }

    // Helper to get an immutable wiki by ID
    fn get_wiki(&self, wiki_id: &str) -> Result<&Wiki, String> {
        self.wikis.get(wiki_id).ok_or_else(|| format!("Wiki with ID '{}' not found", wiki_id))
    }

    // --------------------- HTTP Handlers ---------------------

    #[http]
    async fn create_wiki(&mut self, body: CreateWikiRequest) -> Result<WikiResponse, String> {
        let wiki_id = Uuid::new_v4().to_string();
        let owner_id = self.node_id.clone();

        if owner_id.is_empty() {
            return Err("Node ID not set. Cannot create wiki.".to_string());
        }

        let mut members = HashMap::new();
        members.insert(owner_id.clone(), WikiMembership {
            user_id: owner_id.clone(),
            role: WikiRole::SuperAdmin,
        });

        let created_at = Utc::now().to_rfc3339();

        let new_wiki = Wiki {
            id: wiki_id.clone(),
            name: body.name.clone(),
            pages_data: HashMap::new(),
            pages: HashMap::new(),
            members,
            is_public: body.is_public,
            owner_id: owner_id.clone(),
            peer_state_vectors: HashMap::new(),
            created_at,
        };

        self.wikis.insert(wiki_id.clone(), new_wiki);
        self.save_state_if_needed();

        println!("Created wiki: {} by {}", wiki_id, owner_id);

        let wiki = self.wikis.get(&wiki_id).unwrap();
        
        Ok(WikiResponse {
            id: wiki_id,
            name: body.name,
            owner_id,
            is_public: body.is_public,
            members: wiki.members.values().cloned().collect(),
            pages: Vec::new(),
            created_at: wiki.created_at.clone(),
        })
    }

    #[http]
    async fn get_wiki_details(&self, wiki_id_req: String) -> Result<WikiResponse, String> {
        let wiki = self.get_wiki(&wiki_id_req)?;

        Ok(WikiResponse {
            id: wiki.id.clone(),
            name: wiki.name.clone(),
            owner_id: wiki.owner_id.clone(),
            is_public: wiki.is_public,
            members: wiki.members.values().cloned().collect(),
            pages: wiki.pages_data.keys().cloned().collect(),
            created_at: wiki.created_at.clone(),
        })
    }

    #[http]
    async fn list_wikis(&self) -> Result<Vec<WikiResponse>, String> {
        Ok(self.wikis.values().map(|wiki| {
            WikiResponse {
                id: wiki.id.clone(),
                name: wiki.name.clone(),
                owner_id: wiki.owner_id.clone(),
                is_public: wiki.is_public,
                members: wiki.members.values().cloned().collect(),
                pages: wiki.pages_data.keys().cloned().collect(),
                created_at: wiki.created_at.clone(),
            }
        }).collect())
    }

    #[http]
    async fn get_page_content(&mut self, wiki_id_req: String, page_path_req: String) -> Result<PageContentResponse, String> {
        let wiki = self.get_wiki_mut(&wiki_id_req)?;
        let doc = wiki.get_or_create_page_doc(&page_path_req);
        
        // Get the content from the Yrs doc
        let text_content = doc.get_or_insert_text("content");
        let current_content = text_content.get_string(&doc.transact());

        // Placeholder metadata - in a real system we'd track this with each edit
        let last_modified_by = "unknown".to_string();
        let last_modified_at = Utc::now().to_rfc3339();

        Ok(PageContentResponse {
            path: page_path_req,
            content: current_content,
            last_modified_by,
            last_modified_at,
        })
    }

    #[http]
    async fn update_page_content(&mut self, body: UpdatePageRequest) -> Result<GenericResponse, String> {
        let editor_node_id = self.node_id.clone();
        let wiki_id = body.wiki_id.clone(); // Clone to avoid borrow issues
        let page_path = body.path.clone(); // Clone to avoid borrow issues
        
        if editor_node_id.is_empty() {
            return Err("Node ID not set. Cannot update page.".to_string());
        }

        // Check permissions
        {
            let wiki = self.get_wiki(&wiki_id)?;
            
            // Verify permissions - only Writer+ can edit
            let role = match wiki.members.get(&editor_node_id) {
                Some(membership) => &membership.role,
                None => return Err("You are not a member of this wiki.".to_string()),
            };
            
            if !matches!(role, WikiRole::Writer | WikiRole::Admin | WikiRole::SuperAdmin) {
                return Err("You don't have permission to edit this wiki.".to_string());
            }
        }
        
        // Apply changes
        {
            let wiki = self.get_wiki_mut(&wiki_id)?;
            let doc = wiki.get_or_create_page_doc(&page_path);
            let text_content = doc.get_or_insert_text("content");

            // Apply changes to the Yrs Doc
            let mut txn = doc.transact_mut();
            let current_len = text_content.len(&txn);
            // Delete the existing text by removing all characters
            if current_len > 0 {
                text_content.remove_range(&mut txn, 0, current_len);
            }
            text_content.insert(&mut txn, 0, &body.new_content);
            // txn is committed when it goes out of scope
        }
        
        // Save state and notify peers in separate blocks 
        self.save_state_if_needed();

        // Notify peers about the update
        println!("Page {}/{} updated. Triggering P2P sync.", wiki_id, page_path);
        self.initiate_p2p_page_sync(&wiki_id, &page_path, &editor_node_id).await;

        Ok(GenericResponse {
            success: true,
            message: format!("Page '{}/{}' updated successfully.", wiki_id, page_path),
        })
    }

    // --------------------- P2P Sync Methods ---------------------

    #[http]
    async fn add_peer(&mut self, body: AddPeerRequest) -> Result<GenericResponse, String> {
        if body.peer_node_id.is_empty() || body.peer_node_id == self.node_id {
            return Err("Invalid peer node ID.".to_string());
        }
        self.known_peers.insert(body.peer_node_id.clone());
        self.save_state_if_needed();
        
        Ok(GenericResponse {
            success: true,
            message: format!("Peer '{}' added.", body.peer_node_id),
        })
    }

    // Initiates the P2P sync by sending a ping to all known peers
    async fn initiate_p2p_page_sync(&mut self, wiki_id: &str, page_path: &str, editor_node_id: &str) {
        // Step 1: Create a PingPageUpdate message
        let p2p_message = P2PMessage::PingPageUpdate {
            wiki_id: wiki_id.to_string(),
            page_path: page_path.to_string(),
            sender_node_id: editor_node_id.to_string(),
        };

        let Ok(serialized_message) = serde_json::to_vec(&p2p_message) else {
            eprintln!("Failed to serialize PingPageUpdate message");
            return;
        };

        // Step 2: Send the ping to all known peers
        for peer_id in &self.known_peers {
            // Skip self
            if peer_id == editor_node_id {
                continue;
            }

            // Construct target address for the peer
            let target_process_id_str = "wiki:wiki:os";
            let Ok(target_process_id) = hyperware_process_lib::ProcessId::from_str(target_process_id_str) else {
                eprintln!("Failed to parse target process ID: {}", target_process_id_str);
                continue;
            };

            let target_address = Address::new(peer_id.clone(), target_process_id);

            println!("Sending PingPageUpdate to peer: {} for page {}/{}", peer_id, wiki_id, page_path);
            
            // Fire and forget the ping
            if let Err(err) = hyperware_process_lib::Request::new()
                .target(target_address)
                .body(serialized_message.clone())
                .send() {
                eprintln!("Failed to send PingPageUpdate to {}: {:?}", peer_id, err);
            }
        }
    }

    // Remote handler for P2P messages
    #[remote]
    async fn handle_p2p_message(&mut self, source: Address, body_bytes: Vec<u8>) -> Result<(), String> {
        let p2p_message: P2PMessage = match serde_json::from_slice(&body_bytes) {
            Ok(msg) => msg,
            Err(e) => return Err(format!("Failed to deserialize P2PMessage: {}", e)),
        };

        let sender_node_id = source.node().to_string();
        println!("Received P2P message from {}: {:?}", sender_node_id, p2p_message);

        match p2p_message {
            // Step 1: Received notification that a page was updated
            P2PMessage::PingPageUpdate { wiki_id, page_path, sender_node_id: editor_node_id } => {
                // Make sure we have this wiki and we're a member
                if !self.wikis.contains_key(&wiki_id) {
                    return Err(format!("Wiki with ID '{}' not found", wiki_id));
                }
                
                // Step 2: Send RequestStateVector back to the editor
                let request_sv_msg = P2PMessage::RequestStateVector {
                    wiki_id: wiki_id.clone(),
                    page_path: page_path.clone(),
                    sender_node_id: self.node_id.clone(),
                };
                
                // Serialize and send the request
                let Ok(serialized_msg) = serde_json::to_vec(&request_sv_msg) else {
                    return Err("Failed to serialize RequestStateVector".to_string());
                };

                let target_process_id_str = "wiki:wiki:os";
                let Ok(target_process_id) = hyperware_process_lib::ProcessId::from_str(target_process_id_str) else {
                    return Err(format!("Failed to parse target process ID: {}", target_process_id_str));
                };

                let target_address = Address::new(editor_node_id, target_process_id);
                
                if let Err(err) = hyperware_process_lib::Request::new()
                    .target(target_address)
                    .body(serialized_msg)
                    .send() {
                    return Err(format!("Failed to send RequestStateVector: {:?}", err));
                }
                
                println!("Sent RequestStateVector to editor for {}/{}", wiki_id, page_path);
            },
            
            // Step 3: Editor received request for state vector - compute diff and send update
            P2PMessage::RequestStateVector { wiki_id, page_path, sender_node_id: peer_node_id } => {
                // Get the local wiki and page doc
                let wiki = self.get_wiki_mut(&wiki_id)?;
                let doc = wiki.get_or_create_page_doc(&page_path);
                
                // Create an update to send to the peer
                let state_vector = doc.transact().state_vector();
                let update = doc.transact().encode_state_as_update_v1(&state_vector);
                
                // Create the update message
                let send_update_msg = P2PMessage::SendUpdate {
                    wiki_id: wiki_id.clone(), 
                    page_path: page_path.clone(),
                    update,
                    sender_node_id: self.node_id.clone(),
                };
                
                // Serialize and send the update
                let Ok(serialized_msg) = serde_json::to_vec(&send_update_msg) else {
                    return Err("Failed to serialize SendUpdate".to_string());
                };

                let target_process_id_str = "wiki:wiki:os";
                let Ok(target_process_id) = hyperware_process_lib::ProcessId::from_str(target_process_id_str) else {
                    return Err(format!("Failed to parse target process ID: {}", target_process_id_str));
                };

                let target_address = Address::new(peer_node_id, target_process_id);
                
                if let Err(err) = hyperware_process_lib::Request::new()
                    .target(target_address)
                    .body(serialized_msg)
                    .send() {
                    return Err(format!("Failed to send update: {:?}", err));
                }
                
                println!("Sent update to peer for {}/{}", wiki_id, page_path);
            },
            
            // Step 4: Peer received update from editor - apply to local doc
            P2PMessage::SendUpdate { wiki_id, page_path, update, sender_node_id: _ } => {
                // Get the local wiki and page doc
                let wiki = self.get_wiki_mut(&wiki_id)?;
                let doc = wiki.get_or_create_page_doc(&page_path);
                
                // Apply the update
                let update_obj = match Update::decode_v1(&update) {
                    Ok(update) => update,
                    Err(e) => return Err(format!("Failed to decode update: {}", e)),
                };
                
                // Apply the update to our local doc
                let _ = doc.transact_mut().apply_update(update_obj);
                self.save_state_if_needed();
                
                println!("Applied update to local doc for {}/{}", wiki_id, page_path);
            },
            
            // Handle receiving full wiki data (e.g., when joining a wiki)
            P2PMessage::ShareWikiData { wiki_id, wiki_data, sender_node_id } => {
                // Deserialize the wiki
                let wiki: Wiki = match serde_json::from_slice(&wiki_data) {
                    Ok(wiki) => wiki,
                    Err(e) => return Err(format!("Failed to deserialize wiki data: {}", e)),
                };
                
                // Store the wiki
                self.wikis.insert(wiki_id.clone(), wiki);
                
                // Hydrate the pages
                if let Some(wiki) = self.wikis.get_mut(&wiki_id) {
                    wiki.hydrate_pages();
                }
                
                self.save_state_if_needed();
                println!("Received and applied full wiki data for {} from {}", wiki_id, sender_node_id);
            },
        }
        
        Ok(())
    }

    // --------------------- User Role Management ---------------------

    // Helper to check if a user has a specific role
    fn check_permission(&self, wiki_id: &str, user_id: &str, required_roles: &[WikiRole]) -> Result<(), String> {
        let wiki = self.get_wiki(wiki_id)?;
        
        let membership = wiki.members.get(user_id)
            .ok_or_else(|| "User not a member of this wiki.".to_string())?;
            
        if required_roles.contains(&membership.role) {
            Ok(())
        } else {
            Err("Permission denied.".to_string())
        }
    }
    
    // Helper to check if a user is a SuperAdmin
    fn is_super_admin(&self, wiki_id: &str, user_id: &str) -> bool {
        self.get_wiki(wiki_id)
            .ok()
            .and_then(|wiki| wiki.members.get(user_id))
            .map_or(false, |mem| matches!(mem.role, WikiRole::SuperAdmin))
    }

    #[http]
    async fn add_user_to_wiki(&mut self, body: ManageUserRequest) -> Result<GenericResponse, String> {
        // Check if requester has permission (Admin or SuperAdmin)
        let requester_id = self.node_id.clone();
        self.check_permission(&body.wiki_id, &requester_id, &[WikiRole::Admin, WikiRole::SuperAdmin])?;

        // First check permissions before getting mutable access
        if matches!(body.role, WikiRole::SuperAdmin) && !self.is_super_admin(&body.wiki_id, &requester_id) {
            return Err("Only a SuperAdmin can assign SuperAdmin role.".to_string());
        }
        
        // Now get mutable access to the wiki
        let wiki = self.get_wiki_mut(&body.wiki_id)?;
        
        // Check if user is already a member
        if wiki.members.contains_key(&body.user_id) {
            return Err("User already a member.".to_string());
        }

        // Add the user
        wiki.members.insert(body.user_id.clone(), WikiMembership {
            user_id: body.user_id.clone(),
            role: body.role,
        });
        
        self.save_state_if_needed();
        
        Ok(GenericResponse { 
            success: true, 
            message: format!("User {} added to wiki {}.", body.user_id, body.wiki_id)
        })
    }

    #[http]
    async fn remove_user_from_wiki(&mut self, body: ManageUserRequest) -> Result<GenericResponse, String> {
        // Check if requester has permission (Admin or SuperAdmin)
        let requester_id = self.node_id.clone();
        self.check_permission(&body.wiki_id, &requester_id, &[WikiRole::Admin, WikiRole::SuperAdmin])?;

        // First check permissions - get the target user role
        let is_target_super_admin;
        {
            let wiki = self.get_wiki(&body.wiki_id)?;
            if !wiki.members.contains_key(&body.user_id) {
                return Err("User is not a member.".to_string());
            }
            is_target_super_admin = wiki.members.get(&body.user_id)
                .map_or(false, |mem| matches!(mem.role, WikiRole::SuperAdmin));
        }
        
        // Check if requester can remove a SuperAdmin
        if is_target_super_admin && !self.is_super_admin(&body.wiki_id, &requester_id) {
            return Err("Only a SuperAdmin can remove a SuperAdmin.".to_string());
        }
        
        // Now get mutable access to the wiki and remove the user
        let wiki = self.get_wiki_mut(&body.wiki_id)?;
        wiki.members.remove(&body.user_id);
        
        self.save_state_if_needed();
        
        Ok(GenericResponse { 
            success: true, 
            message: format!("User {} removed from wiki {}.", body.user_id, body.wiki_id)
        })
    }

    #[http]
    async fn change_user_role(&mut self, body: ManageUserRequest) -> Result<GenericResponse, String> {
        // Check if requester has permission (Admin or SuperAdmin)
        let requester_id = self.node_id.clone();
        self.check_permission(&body.wiki_id, &requester_id, &[WikiRole::Admin, WikiRole::SuperAdmin])?;

        // First check permissions - get the target user role
        let is_target_super_admin;
        {
            let wiki = self.get_wiki(&body.wiki_id)?;
            if !wiki.members.contains_key(&body.user_id) {
                return Err("User is not a member.".to_string());
            }
            is_target_super_admin = wiki.members.get(&body.user_id)
                .map_or(false, |mem| matches!(mem.role, WikiRole::SuperAdmin));
        }
        
        // Check permission - SuperAdmin can only be modified by SuperAdmin
        if is_target_super_admin && !self.is_super_admin(&body.wiki_id, &requester_id) {
            return Err("Only a SuperAdmin can modify a SuperAdmin.".to_string());
        }
        
        // Check permission - only SuperAdmin can promote to SuperAdmin
        if matches!(body.role, WikiRole::SuperAdmin) && !self.is_super_admin(&body.wiki_id, &requester_id) {
            return Err("Only a SuperAdmin can assign SuperAdmin role.".to_string());
        }
        
        // Now get mutable access to the wiki and change the role
        let wiki = self.get_wiki_mut(&body.wiki_id)?;
        let member_to_modify = wiki.members.get_mut(&body.user_id)
            .ok_or_else(|| "User is not a member.".to_string())?;
        member_to_modify.role = body.role;
        
        self.save_state_if_needed();
        
        Ok(GenericResponse { 
            success: true, 
            message: format!("User {}'s role changed in wiki {}.", body.user_id, body.wiki_id)
        })
    }

    #[http]
    async fn set_wiki_public(&mut self, wiki_id_req: String, is_public_req: bool) -> Result<GenericResponse, String> {
        // Check if requester has permission (Admin or SuperAdmin)
        let requester_id = self.node_id.clone();
        self.check_permission(&wiki_id_req, &requester_id, &[WikiRole::Admin, WikiRole::SuperAdmin])?;

        let wiki = self.get_wiki_mut(&wiki_id_req)?;
        wiki.is_public = is_public_req;
        
        self.save_state_if_needed();
        
        Ok(GenericResponse { 
            success: true, 
            message: format!("Wiki {} visibility set to public: {}.", wiki_id_req, is_public_req)
        })
    }
}