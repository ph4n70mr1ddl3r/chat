use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
use tokio::sync::mpsc;

slint::include_modules!();

#[derive(Clone, Debug)]
pub struct ConversationData {
    pub conversation_id: String,
    pub participant_username: String,
    pub participant_is_online: bool,
    pub last_message: String,
    pub last_message_time: String,
    pub message_count: i32,
}

#[derive(Clone, Debug)]
pub struct MessageData {
    pub message_id: String,
    pub sender_username: String,
    pub content: String,
    pub timestamp: String,
    pub is_own_message: bool,
    pub status: String,
}

pub struct ChatScreen {
    ui: ChatScreenComponent,
    current_user_id: String,
}

impl ChatScreen {
    pub fn new(user_id: String) -> Result<Self, Box<dyn std::error::Error>> {
        let ui = ChatScreenComponent::new()?;
        let current_user_id = user_id.clone();
        
        // Clone UI handles for callbacks
        let ui_weak = ui.as_weak();
        let user_id_clone = user_id.clone();
        
        // Set up conversation selection callback
        ui.on_conversation_selected(move |conversation_id| {
            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };
            
            let conv_id = conversation_id.to_string();
            let user_id = user_id_clone.clone();
            
            tokio::spawn(async move {
                // Load messages for selected conversation
                match load_messages(&conv_id).await {
                    Ok(messages) => {
                        // Convert to Slint model
                        let slint_messages: Vec<crate::MessageItem> = messages
                            .iter()
                            .map(|m| crate::MessageItem {
                                message_id: m.message_id.clone().into(),
                                sender_username: m.sender_username.clone().into(),
                                content: m.content.clone().into(),
                                timestamp: m.timestamp.clone().into(),
                                is_own_message: m.is_own_message,
                                status: m.status.clone().into(),
                            })
                            .collect();
                        
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                let model = Rc::new(VecModel::from(slint_messages));
                                ui.set_messages(ModelRc::from(model));
                                ui.set_selected_conversation_id(conv_id.into());
                            }
                        }).ok();
                    }
                    Err(e) => {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_error_message(format!("Failed to load messages: {}", e).into());
                            }
                        }).ok();
                    }
                }
            });
        });
        
        // Set up send message callback
        let ui_weak_send = ui.as_weak();
        let user_id_send = user_id.clone();
        ui.on_send_message(move |content| {
            let ui = match ui_weak_send.upgrade() {
                Some(ui) => ui,
                None => return,
            };
            
            let conversation_id = ui.get_selected_conversation_id().to_string();
            if conversation_id.is_empty() {
                return;
            }
            
            let message_content = content.to_string();
            
            tokio::spawn(async move {
                match send_message(&conversation_id, &message_content).await {
                    Ok(_) => {
                        // Message sent successfully - will be added via WebSocket
                        println!("Message sent successfully");
                    }
                    Err(e) => {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_send.upgrade() {
                                ui.set_error_message(format!("Failed to send message: {}", e).into());
                            }
                        }).ok();
                    }
                }
            });
        });
        
        // Load initial conversations
        let ui_weak_init = ui.as_weak();
        let user_id_init = user_id.clone();
        tokio::spawn(async move {
            match load_conversations().await {
                Ok(conversations) => {
                    let slint_conversations: Vec<crate::ConversationItem> = conversations
                        .iter()
                        .map(|c| crate::ConversationItem {
                            conversation_id: c.conversation_id.clone().into(),
                            participant_username: c.participant_username.clone().into(),
                            participant_is_online: c.participant_is_online,
                            last_message: c.last_message.clone().into(),
                            last_message_time: c.last_message_time.clone().into(),
                            message_count: c.message_count,
                        })
                        .collect();
                    
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_init.upgrade() {
                            let model = Rc::new(VecModel::from(slint_conversations));
                            ui.set_conversations(ModelRc::from(model));
                        }
                    }).ok();
                }
                Err(e) => {
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_init.upgrade() {
                            ui.set_error_message(format!("Failed to load conversations: {}", e).into());
                        }
                    }).ok();
                }
            }
        });
        
        Ok(Self { ui, current_user_id })
    }
    
    pub fn show(&self) {
        self.ui.show().unwrap();
    }
    
    pub fn as_weak(&self) -> slint::Weak<ChatScreenComponent> {
        self.ui.as_weak()
    }
}

// API call to load conversations
async fn load_conversations() -> Result<Vec<ConversationData>, Box<dyn std::error::Error>> {
    let base_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let token = crate::services::session::get_token()
        .ok_or("No authentication token found")?;
    
    #[derive(serde::Deserialize)]
    struct ApiConversation {
        conversation_id: String,
        participant_username: String,
        participant_is_online: bool,
        last_message_at: Option<i64>,
        message_count: i32,
    }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/conversations", base_url))
        .query(&[("limit", "20"), ("offset", "0")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to load conversations: {}", response.status()).into());
    }
    
    let api_conversations: Vec<ApiConversation> = response.json().await?;
    
    Ok(api_conversations
        .into_iter()
        .map(|c| ConversationData {
            conversation_id: c.conversation_id,
            participant_username: c.participant_username,
            participant_is_online: c.participant_is_online,
            last_message: "".to_string(), // TODO: Get from messages
            last_message_time: format_timestamp(c.last_message_at),
            message_count: c.message_count,
        })
        .collect())
}

// API call to load messages for a conversation
async fn load_messages(conversation_id: &str) -> Result<Vec<MessageData>, Box<dyn std::error::Error>> {
    let base_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let token = crate::services::session::get_token()
        .ok_or("No authentication token found")?;
    
    let session = crate::services::session::get_session_manager()
        .get_current_session()
        .ok_or("No session found")?;
    
    #[derive(serde::Deserialize)]
    struct ApiMessage {
        id: String,
        sender_id: String,
        sender_username: Option<String>,
        content: String,
        created_at: i64,
        status: String,
    }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/conversations/{}/messages", base_url, conversation_id))
        .query(&[("limit", "100"), ("offset", "0")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to load messages: {}", response.status()).into());
    }
    
    let api_messages: Vec<ApiMessage> = response.json().await?;
    
    Ok(api_messages
        .into_iter()
        .map(|m| MessageData {
            message_id: m.id.clone(),
            sender_username: m.sender_username.unwrap_or_else(|| "Unknown".to_string()),
            content: m.content,
            timestamp: format_timestamp(Some(m.created_at)),
            is_own_message: m.sender_id == session.user_id,
            status: m.status,
        })
        .collect())
}

// API call to send a message
async fn send_message(conversation_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let token = crate::services::session::get_token()
        .ok_or("No authentication token found")?;
    
    #[derive(serde::Serialize)]
    struct SendMessageRequest {
        content: String,
    }
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/conversations/{}/messages", base_url, conversation_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&SendMessageRequest {
            content: content.to_string(),
        })
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to send message: {}", response.status()).into());
    }
    
    Ok(())
}

// Helper to format timestamp
fn format_timestamp(timestamp: Option<i64>) -> String {
    match timestamp {
        Some(ts) => {
            use chrono::{DateTime, Utc, Local};
            let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_default();
            let local: DateTime<Local> = dt.into();
            local.format("%I:%M %p").to_string()
        }
        None => "".to_string(),
    }
}
