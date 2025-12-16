use crate::services::ConnectionStatus;
use crate::ui::{ChatScreenComponent, ConversationItem, MessageItem};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;

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
    pub fn new(
        user_id: String,
        on_logout: Box<dyn Fn() + Send + Sync>,
        on_settings: Box<dyn Fn() + Send + Sync>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ui = ChatScreenComponent::new()?;
        let current_user_id = user_id.clone();
        let runtime = Arc::new(Runtime::new()?);
        let conversations = Arc::new(Mutex::new(Vec::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));
        let selected_conversation_id = Arc::new(Mutex::new(None::<String>));
        let selected_participant_id = Arc::new(Mutex::new(None::<String>));
        let typing_state = Arc::new(Mutex::new(false));
        let typing_indicator_token = Arc::new(Mutex::new(String::new()));
        ui.set_connection_status("Connecting...".into());
        ui.set_connection_online(false);
        ui.set_show_error_dialog(false);
        ui.set_error_dialog_title("Something went wrong".into());
        ui.set_error_dialog_message("".into());

        // WebSocket client bootstrap
        let ui_weak = ui.as_weak();
        let session_token =
            crate::services::session::get_token().ok_or("No authentication token found")?;
        let ws_url = std::env::var("SERVER_WS_URL")
            .unwrap_or_else(|_| "ws://localhost:8080/socket".to_string());
        let (event_tx, event_rx) = mpsc::unbounded_channel::<crate::services::WebSocketEvent>();
        let websocket_client = Some(crate::services::WebSocketClient::connect(
            ws_url,
            session_token,
            event_tx,
            &runtime,
        ));
        let event_handle = spawn_event_listener(
            event_rx,
            ui_weak.clone(),
            conversations.clone(),
            messages.clone(),
            selected_conversation_id.clone(),
            selected_participant_id.clone(),
            typing_indicator_token.clone(),
            current_user_id.clone(),
            runtime.clone(),
            websocket_client.clone(),
        );

        // Logout callback
        let ui_weak_logout = ui.as_weak();
        let runtime_for_logout = runtime.clone();
        let ws_for_logout = websocket_client.clone();
        let logout_cb = Arc::new(on_logout);

        ui.on_logout(move || {
            let ui_weak = ui_weak_logout.clone();
            let runtime = runtime_for_logout.clone();
            let logout_cb_inner = logout_cb.clone();
            let ws_client = ws_for_logout.clone();

            runtime.spawn(async move {
                // 1. Call API logout
                let _ = api_logout().await;

                // 2. Clear session locally
                let _ = crate::services::session::get_session_manager()
                    .clear_session()
                    .await;

                // 3. Disconnect WebSocket
                if let Some(ws) = ws_client.as_ref() {
                    ws.disconnect();
                }

                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.hide().unwrap(); // Hide chat screen
                        logout_cb_inner(); // Trigger callback
                    }
                })
                .ok();
            });
        });

        // Settings callback
        let ui_weak_settings = ui.as_weak();
        let settings_cb = Arc::new(on_settings);
        ui.on_open_settings(move || {
            let ui_weak = ui_weak_settings.clone();
            let settings_cb_inner = settings_cb.clone();

            // Just hide and call callback, no async needed here strictly speaking,
            // but for consistency we use invoke_from_event_loop if we were in async.
            // Here we are in UI callback (main thread).
            // But we want to ensure clean state.
            if let Some(ui) = ui_weak.upgrade() {
                ui.hide().unwrap();
                settings_cb_inner();
            }
        });

        // Clone UI handles for callbacks
        let user_id_clone = user_id.clone();
        let runtime_for_select = runtime.clone();
        let conversations_for_select = conversations.clone();
        let messages_for_select = messages.clone();
        let selected_conv_for_select = selected_conversation_id.clone();
        let selected_participant_for_select = selected_participant_id.clone();
        let ui_weak_select = ui.as_weak();

        // Set up conversation selection callback
        ui.on_conversation_selected(move |conversation_id| {
            let ui_weak = ui_weak_select.clone();
            let runtime = runtime_for_select.clone();
            let conversations = conversations_for_select.clone();
            let messages = messages_for_select.clone();
            let selected_conv = selected_conv_for_select.clone();
            let selected_participant = selected_participant_for_select.clone();
            let user_id = user_id_clone.clone();

            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            let conv_id = conversation_id.to_string();

            // Update selected conversation metadata
            if let Some(conv) = conversations
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.conversation_id == conv_id)
                .cloned()
            {
                *selected_conv.lock().unwrap() = Some(conv_id.clone());
                *selected_participant.lock().unwrap() = Some(conv.participant_id.clone());

                slint::invoke_from_event_loop({
                    let conv_id = conv_id.clone();
                    let ui_weak = ui_weak.clone();
                    move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_selected_conversation_id(conv_id.clone().into());
                            ui.set_selected_participant_id(conv.participant_id.clone().into());
                            ui.set_selected_participant_username(
                                conv.participant_username.clone().into(),
                            );
                            ui.set_selected_participant_is_online(conv.participant_is_online);
                            ui.set_typing_indicator("".into());
                        }
                    }
                })
                .ok();
            }

            runtime.spawn(async move {
                // Load messages for selected conversation
                match load_messages(&conv_id).await {
                    Ok(msgs) => {
                        {
                            let mut cache = messages.lock().unwrap();
                            *cache = msgs.clone();
                        }
                        render_messages_for_conversation(
                            ui_weak.clone(),
                            messages.clone(),
                            conv_id.clone(),
                        );
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to load messages: {}", e);
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_error_message(err_msg.into());
                            }
                        })
                        .ok();
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
            let ui_weak = ui_weak_send.clone();
            let messages = messages_send.clone();
            let selected_participant = selected_participant_for_send.clone();
            let ws_client = ws_for_send.clone();
            let typing_state = typing_state_for_send.clone();

            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            let conversation_id = ui.get_selected_conversation_id().to_string();
            if conversation_id.is_empty() {
                return;
            }

            let participant_id = selected_participant
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
                let mut cache = messages.lock().unwrap();
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
                ui_weak.clone(),
                messages.clone(),
                conversation_id.clone(),
            );

            if let Some(ws) = ws_client.as_ref() {
                if let Err(e) = ws.send_message(
                    message_id.clone(),
                    conversation_id.clone(),
                    participant_id.clone(),
                    message_content.clone(),
                ) {
                    let err_msg = format!("Failed to send message: {}", e);
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_error_message(err_msg.into());
                        }
                    })
                    .ok();
                }
            }

            // Reset typing indicator after sending a message
            {
                let mut state = typing_state.lock().unwrap();
                if *state {
                    *state = false;
                    if let Some(ws) = ws_client.as_ref() {
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
            let ui_weak = ui_weak_typing.clone();
            let ws_client = ws_for_typing.clone();
            let typing_state = typing_state_for_cb.clone();
            let selected_participant = selected_participant_for_typing.clone();

            let participant_id = selected_participant
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();

            if participant_id.is_empty() {
                return;
            }

            let mut state = typing_state.lock().unwrap();
            if *state == is_typing {
                return;
            }
            *state = is_typing;

            if let Some(ws) = ws_client.as_ref() {
                if let Err(e) = ws.send_typing(participant_id.clone(), is_typing) {
                    let err_msg = format!("Failed to send typing indicator: {}", e);
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_error_message(err_msg.into());
                        }
                    })
                    .ok();
                }
            }
        });

        // Search callbacks
        let ui_weak_search = ui.as_weak();
        let runtime_for_search = runtime.clone();
        ui.on_search_in_conversation(move |query| {
            let ui_weak = ui_weak_search.clone();
            let runtime = runtime_for_search.clone();

            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            let conversation_id = ui.get_selected_conversation_id().to_string();
            if conversation_id.is_empty() {
                return;
            }

            ui.set_is_search_active(true);
            ui.set_search_query(query.clone().into());

            runtime.spawn(async move {
                match search_messages(&conversation_id, &query).await {
                    Ok(messages) => {
                        let ui_messages: Vec<MessageItem> = messages
                            .iter()
                            .map(|m| MessageItem {
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
                            }
                        })
                        .ok();
                    }
                    Err(e) => {
                        let err_msg = format!("Search failed: {}", e);
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_error_message(err_msg.into());
                            }
                        })
                        .ok();
                    }
                }
            });
        });

        let ui_weak_clear = ui.as_weak();
        let messages_for_clear = messages.clone();
        ui.on_clear_search(move || {
            let ui_weak = ui_weak_clear.clone();
            let messages = messages_for_clear.clone();

            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            let conversation_id = ui.get_selected_conversation_id().to_string();
            if conversation_id.is_empty() {
                return;
            }

            ui.set_is_search_active(false);
            ui.set_search_query("".into());

            // Restore original messages
            render_messages_for_conversation(ui_weak.clone(), messages.clone(), conversation_id);
        });

        // Load initial conversations
        let ui_weak_init = ui.as_weak();
        let conversations_init = conversations.clone();
        let runtime_for_init = runtime.clone();
        runtime_for_init.spawn(async move {
            match load_conversations().await {
                Ok(conversations) => {
                    let slint_conversations: Vec<ConversationItem> = conversations
                        .iter()
                        .map(|c| ConversationItem {
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

                    let ui_weak = ui_weak_init.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            let model = Rc::new(VecModel::from(slint_conversations));
                            ui.set_conversations(ModelRc::from(model));
                        }
                    })
                    .ok();
                }
                Err(e) => {
                    let err_msg = format!("Failed to load conversations: {}", e);
                    let ui_weak = ui_weak_init.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_error_message(err_msg.into());
                        }
                    })
                    .ok();
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
    conversations: Arc<Mutex<Vec<ConversationData>>>,
    messages: Arc<Mutex<Vec<MessageData>>>,
    selected_conversation_id: Arc<Mutex<Option<String>>>,
    selected_participant_id: Arc<Mutex<Option<String>>>,
    typing_indicator_token: Arc<Mutex<String>>,
    current_user_id: String,
    runtime: Arc<Runtime>,
    websocket_client: Option<crate::services::WebSocketClient>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while let Some(event) = event_rx.blocking_recv() {
            match event {
                crate::services::WebSocketEvent::ConnectionState(state) => {
                    let ui_for_status = ui_weak.clone();
                    match state {
                        ConnectionStatus::Connecting => {
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_for_status.upgrade() {
                                    ui.set_connection_status("Connecting...".into());
                                    ui.set_connection_online(false);
                                }
                            })
                            .ok();
                        }
                        ConnectionStatus::Reconnecting { retry_in_ms } => {
                            let status_text = format!(
                                "Reconnecting... retry in {}s",
                                (retry_in_ms as f64 / 1000.0).ceil() as u64
                            );
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_for_status.upgrade() {
                                    ui.set_connection_status(status_text.clone().into());
                                    ui.set_connection_online(false);
                                }
                            })
                            .ok();
                        }
                        ConnectionStatus::Connected => {
                            let ui_for_status = ui_weak.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_for_status.upgrade() {
                                    ui.set_connection_status("Connected".into());
                                    ui.set_connection_online(true);
                                    ui.set_error_message("".into());
                                    ui.set_show_error_dialog(false);
                                    ui.set_error_dialog_message("".into());
                                }
                            })
                            .ok();

                            // Refresh conversations/messages and retry any pending sends.
                            let conversations_refresh = conversations.clone();
                            let messages_refresh = messages.clone();
                            let selected_conv_refresh = selected_conversation_id.clone();
                            let ui_refresh = ui_weak.clone();
                            let ws_for_resend = websocket_client.clone();
                            let runtime_refresh = runtime.clone();

                            runtime_refresh.spawn(async move {
                                if let Ok(fresh_conversations) = load_conversations().await {
                                    {
                                        let mut cache = conversations_refresh.lock().unwrap();
                                        *cache = fresh_conversations.clone();
                                    }
                                    render_conversations(
                                        ui_refresh.clone(),
                                        conversations_refresh.clone(),
                                    );
                                }

                                if let Some(active_conv) =
                                    selected_conv_refresh.lock().unwrap().clone()
                                {
                                    if let Ok(fresh_messages) = load_messages(&active_conv).await {
                                        {
                                            let mut cache = messages_refresh.lock().unwrap();
                                            *cache = fresh_messages.clone();
                                        }
                                        render_messages_for_conversation(
                                            ui_refresh.clone(),
                                            messages_refresh.clone(),
                                            active_conv,
                                        );
                                    }
                                }

                                if let Some(ws) = ws_for_resend {
                                    resend_pending_messages(
                                        &ws,
                                        &messages_refresh,
                                        &conversations_refresh,
                                    );
                                }
                            });
                        }
                        ConnectionStatus::Disconnected { reason } => {
                            let ui_for_status = ui_weak.clone();
                            slint::invoke_from_event_loop({
                                let reason = reason.clone();
                                move || {
                                    if let Some(ui) = ui_for_status.upgrade() {
                                        ui.set_connection_status(
                                            format!("Disconnected: {}", reason).into(),
                                        );
                                        ui.set_connection_online(false);
                                        ui.set_error_dialog_title("Disconnected".into());
                                        ui.set_error_dialog_message(reason.clone().into());
                                        ui.set_show_error_dialog(true);
                                    }
                                }
                            })
                            .ok();
                        }
                    }
                }
                crate::services::WebSocketEvent::Presence {
                    user_id,
                    username: _,
                    is_online,
                    last_seen_at: _,
                } => {
                    // Update conversation list
                    let mut needs_refresh = false;
                    {
                        let mut cache = conversations.lock().unwrap();
                        for conv in cache.iter_mut() {
                            if conv.participant_id == user_id {
                                conv.participant_is_online = is_online;
                                needs_refresh = true;
                            }
                        }
                    }

                    if needs_refresh {
                        render_conversations(ui_weak.clone(), conversations.clone());
                    }

                    // Update selected conversation status
                    let selected_participant = selected_participant_id.lock().unwrap().clone();
                    if let Some(active_user_id) = selected_participant {
                        if active_user_id == user_id {
                            let ui_weak_update = ui_weak.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_update.upgrade() {
                                    ui.set_selected_participant_is_online(is_online);
                                }
                            })
                            .ok();
                        }
                    }
                }
                crate::services::WebSocketEvent::Message {
                    conversation_id,
                    message_id,
                    sender_username,
                    content,
                    status,
                    timestamp,
                } => {
                    if selected_conversation_id.lock().unwrap().as_deref() != Some(&conversation_id)
                    {
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

                    let is_searching = ui_weak
                        .upgrade()
                        .map(|ui| ui.get_is_search_active())
                        .unwrap_or(false);
                    if !is_searching {
                        render_messages_for_conversation(
                            ui_weak.clone(),
                            messages.clone(),
                            conversation_id.clone(),
                        );
                    }
                }
                crate::services::WebSocketEvent::Ack {
                    message_id,
                    status,
                    conversation_id,
                } => {
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
                        let is_searching = ui_weak
                            .upgrade()
                            .map(|ui| ui.get_is_search_active())
                            .unwrap_or(false);
                        if !is_searching {
                            render_messages_for_conversation(
                                ui_weak.clone(),
                                messages.clone(),
                                selected_id,
                            );
                        }
                    }
                }
                crate::services::WebSocketEvent::Typing {
                    sender_id,
                    sender_username,
                    recipient_id,
                    is_typing,
                } => {
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

                    let ui_weak_typing = ui_weak.clone();
                    slint::invoke_from_event_loop({
                        let indicator_text = indicator_text.clone();
                        move || {
                            if let Some(ui) = ui_weak_typing.upgrade() {
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
                    let ui_weak_err = ui_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_err.upgrade() {
                            ui.set_error_message(err.clone().into());
                            ui.set_error_dialog_title("Connection issue".into());
                            ui.set_error_dialog_message(err.clone().into());
                            ui.set_show_error_dialog(true);
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
    let ui_messages: Vec<MessageItem> = messages
        .lock()
        .unwrap()
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .map(|m| MessageItem {
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
            ui.invoke_scroll_to_bottom();
        }
    })
    .ok();
}

fn render_conversations(
    ui_weak: slint::Weak<ChatScreenComponent>,
    conversations: Arc<Mutex<Vec<ConversationData>>>,
) {
    let snapshot = conversations.lock().unwrap().clone();
    let slint_conversations: Vec<ConversationItem> = snapshot
        .into_iter()
        .map(|c| ConversationItem {
            conversation_id: c.conversation_id.into(),
            participant_id: c.participant_id.into(),
            participant_username: c.participant_username.into(),
            participant_is_online: c.participant_is_online,
            last_message: c.last_message.into(),
            last_message_time: c.last_message_time.into(),
            message_count: c.message_count,
        })
        .collect();

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = Rc::new(VecModel::from(slint_conversations));
            ui.set_conversations(ModelRc::from(model));
        }
    })
    .ok();
}

fn resend_pending_messages(
    websocket_client: &crate::services::WebSocketClient,
    messages: &Arc<Mutex<Vec<MessageData>>>,
    conversations: &Arc<Mutex<Vec<ConversationData>>>,
) {
    let participants: HashMap<String, String> = conversations
        .lock()
        .unwrap()
        .iter()
        .map(|c| (c.conversation_id.clone(), c.participant_id.clone()))
        .collect();

    let pending: Vec<MessageData> = messages
        .lock()
        .unwrap()
        .iter()
        .filter(|m| m.is_own_message && m.status == "pending")
        .cloned()
        .collect();

    for msg in pending {
        if let Some(recipient) = participants.get(&msg.conversation_id) {
            let _ = websocket_client.send_message(
                msg.message_id.clone(),
                msg.conversation_id.clone(),
                recipient.clone(),
                msg.content.clone(),
            );
        }
    }
}

fn format_timestamp(timestamp: Option<i64>) -> String {
    match timestamp {
        Some(ts) => {
            use chrono::{DateTime, Local, Utc};
            let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_default();
            let local: DateTime<Local> = dt.into();
            local.format("%I:%M %p").to_string()
        }
        None => "".to_string(),
    }
}

// API call to logout
async fn api_logout() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let token = match crate::services::session::get_token() {
        Some(t) => t,
        None => return Ok(()), // Already logged out or no token
    };

    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{}/auth/logout", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    Ok(())
}

// API call to load conversations
async fn load_conversations() -> Result<Vec<ConversationData>, Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let token = crate::services::session::get_token().ok_or("No authentication token found")?;

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
async fn load_messages(
    conversation_id: &str,
) -> Result<Vec<MessageData>, Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let token = crate::services::session::get_token().ok_or("No authentication token found")?;

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
        .get(format!(
            "{}/conversations/{}/messages",
            base_url, conversation_id
        ))
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

// API call to search messages in a conversation
async fn search_messages(
    conversation_id: &str,
    query: &str,
) -> Result<Vec<MessageData>, Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let token = crate::services::session::get_token().ok_or("No authentication token found")?;

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
        .get(format!(
            "{}/conversations/{}/search",
            base_url, conversation_id
        ))
        .query(&[("q", query), ("limit", "50")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to search messages: {}", response.status()).into());
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
