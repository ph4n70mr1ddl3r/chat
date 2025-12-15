use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;

slint::include_modules!();

#[derive(Clone, Debug)]
pub struct ConversationData {
    pub conversation_id: String,
    pub participant_id: String,
    pub participant_username: String,
    pub participant_is_online: bool,
    pub last_message: String,
    pub last_message_time: String,
    pub message_count: i32,
}

#[derive(Clone, Debug)]
pub struct MessageData {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_username: String,
    pub content: String,
    pub timestamp: String,
    pub is_own_message: bool,
    pub status: String,
}

pub struct ChatScreen {
    ui: ChatScreenComponent,
    current_user_id: String,
    runtime: Arc<Runtime>,
    conversations: Arc<Mutex<Vec<ConversationData>>>,
    messages: Arc<Mutex<Vec<MessageData>>>,
    selected_conversation_id: Arc<Mutex<Option<String>>>,
    selected_participant_id: Arc<Mutex<Option<String>>>,
    _typing_state: Arc<Mutex<bool>>,
    _typing_indicator_token: Arc<Mutex<String>>,
    websocket_client: Option<crate::services::WebSocketClient>,
    _event_handle: Option<std::thread::JoinHandle<()>>,
}

impl ChatScreen {
    pub fn new(user_id: String) -> Result<Self, Box<dyn std::error::Error>> {
        let ui = ChatScreenComponent::new()?;
        let current_user_id = user_id.clone();
        let runtime = Arc::new(Runtime::new()?);
        let conversations = Arc::new(Mutex::new(Vec::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));
        let selected_conversation_id = Arc::new(Mutex::new(None::<String>));
        let selected_participant_id = Arc::new(Mutex::new(None::<String>));
        let typing_state = Arc::new(Mutex::new(false));
        let typing_indicator_token = Arc::new(Mutex::new(String::new()));

        // WebSocket client bootstrap
        let ui_weak = ui.as_weak();
        let session_token = crate::services::session::get_token()
            .ok_or("No authentication token found")?;
        let ws_url =
            std::env::var("SERVER_WS_URL").unwrap_or_else(|_| "ws://localhost:8080/socket".to_string());
        let (event_tx, event_rx) = mpsc::unbounded_channel::<crate::services::WebSocketEvent>();
        let websocket_client =
            Some(crate::services::WebSocketClient::connect(ws_url, session_token, event_tx, &runtime));
        let event_handle = spawn_event_listener(
            event_rx,
            ui_weak.clone(),
            messages.clone(),
            selected_conversation_id.clone(),
            selected_participant_id.clone(),
            typing_indicator_token.clone(),
            current_user_id.clone(),
        );

        // Clone UI handles for callbacks
        let user_id_clone = user_id.clone();
        let runtime_for_select = runtime.clone();
        let conversations_for_select = conversations.clone();
        let messages_for_select = messages.clone();
        let selected_conv_for_select = selected_conversation_id.clone();
        let selected_participant_for_select = selected_participant_id.clone();
        
        // Set up conversation selection callback
        ui.on_conversation_selected(move |conversation_id| {
            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };
            
            let conv_id = conversation_id.to_string();
            let user_id = user_id_clone.clone();
            // Update selected conversation metadata
            if let Some(conv) = conversations_for_select
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.conversation_id == conv_id)
                .cloned()
            {
                *selected_conv_for_select.lock().unwrap() = Some(conv_id.clone());
                *selected_participant_for_select.lock().unwrap() = Some(conv.participant_id.clone());

                slint::invoke_from_event_loop({
                    let conv_id = conv_id.clone();
                        move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_selected_conversation_id(conv_id.clone().into());
                                ui.set_selected_participant_id(conv.participant_id.clone().into());
                                ui.set_selected_participant_username(conv.participant_username.clone().into());
                                ui.set_selected_participant_is_online(conv.participant_is_online);
                                ui.set_typing_indicator("".into());
                            }
                        }
                    })
                .ok();
            }
            
            runtime_for_select.spawn(async move {
                // Load messages for selected conversation
                match load_messages(&conv_id).await {
                    Ok(messages) => {
                        {
                            let mut cache = messages_for_select.lock().unwrap();
                            *cache = messages.clone();
                        }
                        render_messages_for_conversation(
                            ui_weak.clone(),
                            messages_for_select.clone(),
                            conv_id.clone(),
                        );
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
        let messages_send = messages.clone();
        let selected_participant_for_send = selected_participant_id.clone();
        let ws_for_send = websocket_client.clone();
        let typing_state_for_send = typing_state.clone();
        ui.on_send_message(move |content| {
            let ui = match ui_weak_send.upgrade() {
                Some(ui) => ui,
                None => return,
            };
            
            let conversation_id = ui.get_selected_conversation_id().to_string();
            if conversation_id.is_empty() {
                return;
            }
            
            let participant_id = selected_participant_for_send
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();
            if participant_id.is_empty() {
                return;
            }
            
            let message_content = content.to_string();
            let message_id = Uuid::new_v4().to_string();
            let timestamp = chrono::Utc::now().timestamp();

            // Add pending message locally for immediate feedback
            {
                let mut cache = messages_send.lock().unwrap();
                cache.push(MessageData {
                    message_id: message_id.clone(),
                    conversation_id: conversation_id.clone(),
                    sender_username: "You".to_string(),
                    content: message_content.clone(),
                    timestamp: format_timestamp(Some(timestamp)),
                    is_own_message: true,
                    status: "pending".to_string(),
                });
            }
            render_messages_for_conversation(
                ui_weak_send.clone(),
                messages_send.clone(),
                conversation_id.clone(),
            );
            
            if let Some(ws) = ws_for_send.as_ref() {
                if let Err(e) = ws.send_message(
                    message_id.clone(),
                    conversation_id.clone(),
                    participant_id.clone(),
                    message_content.clone(),
                ) {
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_send.upgrade() {
                            ui.set_error_message(format!("Failed to send message: {}", e).into());
                        }
                    })
                    .ok();
                }
            }

            // Reset typing indicator after sending a message
            {
                let mut state = typing_state_for_send.lock().unwrap();
                if *state {
                    *state = false;
                    if let Some(ws) = ws_for_send.as_ref() {
                        let _ = ws.send_typing(participant_id, false);
                    }
                }
            }
        });

        // Typing indicator callback
        let ui_weak_typing = ui.as_weak();
        let ws_for_typing = websocket_client.clone();
        let typing_state_for_cb = typing_state.clone();
        let selected_participant_for_typing = selected_participant_id.clone();
        ui.on_typing(move |is_typing| {
            let participant_id = selected_participant_for_typing
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();

            if participant_id.is_empty() {
                return;
            }

            let mut state = typing_state_for_cb.lock().unwrap();
            if *state == is_typing {
                return;
            }
            *state = is_typing;

            if let Some(ws) = ws_for_typing.as_ref() {
                if let Err(e) = ws.send_typing(participant_id.clone(), is_typing) {
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_typing.upgrade() {
                            ui.set_error_message(format!("Failed to send typing indicator: {}", e).into());
                        }
                    })
                    .ok();
                }
            }
        });
        
        // Load initial conversations
        let ui_weak_init = ui.as_weak();
        let conversations_init = conversations.clone();
        let runtime_for_init = runtime.clone();
        runtime_for_init.spawn(async move {
            match load_conversations().await {
                Ok(conversations) => {
                    let slint_conversations: Vec<crate::ConversationItem> = conversations
                        .iter()
                        .map(|c| crate::ConversationItem {
                            conversation_id: c.conversation_id.clone().into(),
                            participant_id: c.participant_id.clone().into(),
                            participant_username: c.participant_username.clone().into(),
                            participant_is_online: c.participant_is_online,
                            last_message: c.last_message.clone().into(),
                            last_message_time: c.last_message_time.clone().into(),
                            message_count: c.message_count,
                        })
                        .collect();
                    
                    {
                        let mut cache = conversations_init.lock().unwrap();
                        *cache = conversations;
                    }
                    
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
        
        Ok(Self {
            ui,
            current_user_id,
            runtime,
            conversations,
            messages,
            selected_conversation_id,
            selected_participant_id,
            _typing_state: typing_state,
            _typing_indicator_token: typing_indicator_token,
            websocket_client,
            _event_handle: Some(event_handle),
        })
    }
    
    pub fn show(&self) {
        self.ui.show().unwrap();
    }
    
    pub fn as_weak(&self) -> slint::Weak<ChatScreenComponent> {
        self.ui.as_weak()
    }
}

fn spawn_event_listener(
    mut event_rx: mpsc::UnboundedReceiver<crate::services::WebSocketEvent>,
    ui_weak: slint::Weak<ChatScreenComponent>,
    messages: Arc<Mutex<Vec<MessageData>>>,
    selected_conversation_id: Arc<Mutex<Option<String>>>,
    selected_participant_id: Arc<Mutex<Option<String>>>,
    typing_indicator_token: Arc<Mutex<String>>,
    current_user_id: String,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while let Some(event) = event_rx.blocking_recv() {
            match event {
                crate::services::WebSocketEvent::Message { conversation_id, message_id, sender_username, content, status, timestamp } => {
                    if selected_conversation_id.lock().unwrap().as_deref() != Some(&conversation_id) {
                        continue;
                    }

                    {
                        let mut cache = messages.lock().unwrap();
                        cache.push(MessageData {
                            message_id,
                            conversation_id: conversation_id.clone(),
                            sender_username,
                            content,
                            timestamp: format_timestamp(Some(timestamp as i64)),
                            is_own_message: false,
                            status,
                        });
                    }

                    render_messages_for_conversation(
                        ui_weak.clone(),
                        messages.clone(),
                        conversation_id.clone(),
                    );
                }
                crate::services::WebSocketEvent::Ack { message_id, status, conversation_id } => {
                    {
                        let mut cache = messages.lock().unwrap();
                        if let Some(msg) = cache.iter_mut().find(|m| {
                            Some(&m.message_id) == message_id.as_ref()
                                && conversation_id
                                    .as_ref()
                                    .map(|id| id == &m.conversation_id)
                                    .unwrap_or(true)
                        }) {
                            msg.status = status.clone();
                        }
                    }

                    let selected = selected_conversation_id.lock().unwrap().clone();
                    if let Some(selected_id) = selected {
                        render_messages_for_conversation(
                            ui_weak.clone(),
                            messages.clone(),
                            selected_id,
                        );
                    }
                }
                crate::services::WebSocketEvent::Typing { sender_id, sender_username, recipient_id, is_typing } => {
                    if recipient_id != current_user_id {
                        continue;
                    }

                    let participant_match = {
                        let selected = selected_participant_id.lock().unwrap();
                        match (sender_id.as_ref(), selected.as_ref()) {
                            (Some(sender), Some(active)) => sender == active,
                            _ => true,
                        }
                    };

                    if !participant_match {
                        continue;
                    }

                    let indicator_text = if is_typing {
                        format!("{} is typing...", sender_username)
                    } else {
                        String::new()
                    };

                    let token = Uuid::new_v4().to_string();
                    {
                        let mut guard = typing_indicator_token.lock().unwrap();
                        *guard = token.clone();
                    }

                    slint::invoke_from_event_loop({
                        let indicator_text = indicator_text.clone();
                        move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_typing_indicator(indicator_text.clone().into());
                            }
                        }
                    })
                    .ok();

                    if is_typing {
                        let ui_weak_clone = ui_weak.clone();
                        let typing_indicator_token = typing_indicator_token.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(2000));
                            let should_clear = {
                                let guard = typing_indicator_token.lock().unwrap();
                                *guard == token
                            };
                            if should_clear {
                                slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_weak_clone.upgrade() {
                                        ui.set_typing_indicator("".into());
                                    }
                                })
                                .ok();
                            }
                        });
                    }
                }
                crate::services::WebSocketEvent::Error(err) => {
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_error_message(err.clone().into());
                        }
                    })
                    .ok();
                }
            }
        }
    })
}

fn render_messages_for_conversation(
    ui_weak: slint::Weak<ChatScreenComponent>,
    messages: Arc<Mutex<Vec<MessageData>>>,
    conversation_id: String,
) {
    let ui_messages: Vec<crate::MessageItem> = messages
        .lock()
        .unwrap()
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .map(|m| crate::MessageItem {
            message_id: m.message_id.clone().into(),
            conversation_id: m.conversation_id.clone().into(),
            sender_username: m.sender_username.clone().into(),
            content: m.content.clone().into(),
            timestamp: m.timestamp.clone().into(),
            is_own_message: m.is_own_message,
            status: m.status.clone().into(),
        })
        .collect();

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = Rc::new(VecModel::from(ui_messages));
            ui.set_messages(ModelRc::from(model));
            ui.scroll_to_bottom();
        }
    })
    .ok();
}

// API call to load conversations
async fn load_conversations() -> Result<Vec<ConversationData>, Box<dyn std::error::Error>> {
    let base_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let token = crate::services::session::get_token()
        .ok_or("No authentication token found")?;
    
    #[derive(serde::Deserialize)]
    struct ApiConversation {
        conversation_id: String,
        participant_id: String,
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
            participant_id: c.participant_id,
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
            conversation_id: conversation_id.to_string(),
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
