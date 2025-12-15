use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
use tokio::sync::mpsc;
use std::time::Duration;

slint::include_modules!();

#[derive(Clone, Debug)]
pub struct UserSearchResult {
    pub user_id: String,
    pub username: String,
    pub is_online: bool,
}

pub struct UserSearchScreen {
    ui: UserSearchScreenComponent,
    search_tx: mpsc::Sender<String>,
}

impl UserSearchScreen {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ui = UserSearchScreenComponent::new()?;
        let (search_tx, mut search_rx) = mpsc::channel::<String>(100);
        
        // Clone UI handle for callbacks
        let ui_weak = ui.as_weak();
        
        // Set up search callback with debouncing
        let search_tx_clone = search_tx.clone();
        ui.on_search_changed(move |query| {
            let tx = search_tx_clone.clone();
            let query_str = query.to_string();
            tokio::spawn(async move {
                // Debounce: wait 300ms before actually searching
                tokio::time::sleep(Duration::from_millis(300)).await;
                let _ = tx.send(query_str).await;
            });
        });
        
        // Set up user selected callback (start conversation)
        let ui_weak_clone = ui.as_weak();
        ui.on_user_selected(move |user_id| {
            let ui = match ui_weak_clone.upgrade() {
                Some(ui) => ui,
                None => return,
            };
            
            let user_id_str = user_id.to_string();
            tokio::spawn(async move {
                match start_conversation(&user_id_str).await {
                    Ok(conversation_id) => {
                        // TODO: Navigate to chat screen with conversation_id
                        println!("Conversation started: {}", conversation_id);
                    }
                    Err(e) => {
                        // Show error
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_clone.upgrade() {
                                ui.set_error_message(format!("Failed to start conversation: {}", e).into());
                            }
                        }).ok();
                    }
                }
            });
        });
        
        // Background task to handle search requests
        tokio::spawn(async move {
            while let Some(query) = search_rx.recv().await {
                if query.len() < 1 {
                    continue;
                }
                
                let ui = match ui_weak.upgrade() {
                    Some(ui) => ui,
                    None => break,
                };
                
                // Set loading state
                ui.set_is_searching(true);
                ui.set_error_message("".into());
                
                // Perform search (call backend API)
                match search_users(&query).await {
                    Ok(results) => {
                        // Convert to Slint model
                        let slint_results: Vec<crate::UserSearchResult> = results
                            .iter()
                            .map(|r| crate::UserSearchResult {
                                user_id: r.user_id.clone().into(),
                                username: r.username.clone().into(),
                                is_online: r.is_online,
                            })
                            .collect();
                        
                        let model = Rc::new(VecModel::from(slint_results));
                        ui.set_search_results(ModelRc::from(model));
                        ui.set_is_searching(false);
                    }
                    Err(e) => {
                        ui.set_error_message(format!("Search failed: {}", e).into());
                        ui.set_is_searching(false);
                    }
                }
            }
        });
        
        Ok(Self { ui, search_tx })
    }
    
    pub fn show(&self) {
        self.ui.show().unwrap();
    }
    
    pub fn as_weak(&self) -> slint::Weak<UserSearchScreenComponent> {
        self.ui.as_weak()
    }
}

// API call to search users
async fn search_users(query: &str) -> Result<Vec<UserSearchResult>, Box<dyn std::error::Error>> {
    // Get base URL from environment or use default
    let base_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    // Get auth token from session storage
    let token = crate::services::session::get_token()
        .ok_or("No authentication token found")?;
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/users/search", base_url))
        .query(&[("q", query), ("limit", "10")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Search failed with status: {}", response.status()).into());
    }
    
    #[derive(serde::Deserialize)]
    struct ApiUserResult {
        #[serde(rename = "userId")]
        user_id: String,
        username: String,
        #[serde(rename = "isOnline")]
        is_online: bool,
    }
    
    let api_results: Vec<ApiUserResult> = response.json().await?;
    
    Ok(api_results
        .into_iter()
        .map(|r| UserSearchResult {
            user_id: r.user_id,
            username: r.username,
            is_online: r.is_online,
        })
        .collect())
}

// API call to start a conversation
async fn start_conversation(other_user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let base_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let token = crate::services::session::get_token()
        .ok_or("No authentication token found")?;
    
    #[derive(serde::Serialize)]
    struct StartConversationRequest {
        other_user_id: String,
    }
    
    #[derive(serde::Deserialize)]
    struct ConversationResponse {
        conversation_id: String,
    }
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/conversations/start", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&StartConversationRequest {
            other_user_id: other_user_id.to_string(),
        })
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to start conversation: {}", response.status()).into());
    }
    
    let result: ConversationResponse = response.json().await?;
    Ok(result.conversation_id)
}
