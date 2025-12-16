//! WebSocket client for sending/receiving chat messages and typing indicators.
//!
//! Runs on a background Tokio runtime and communicates with the UI through channels.

use chat_shared::protocol::{AckData, MessageEnvelope, TextMessageData, TypingData, PresenceData};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use uuid::Uuid;
use crate::services::session;

/// Events emitted by the WebSocket client.
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    /// Connection status changes surfaced to the UI.
    ConnectionState(ConnectionStatus),
    /// A chat message was received from the server.
    Message {
        conversation_id: String,
        message_id: String,
        sender_username: String,
        content: String,
        status: String,
        timestamp: u64,
    },
    /// An acknowledgement for a message we sent.
    Ack {
        message_id: Option<String>,
        status: String,
        conversation_id: Option<String>,
    },
    /// Typing indicator from the remote participant.
    Typing {
        sender_id: Option<String>,
        sender_username: String,
        recipient_id: String,
        is_typing: bool,
    },
    /// User online status update.
    Presence {
        user_id: String,
        username: String,
        is_online: bool,
        last_seen_at: u64,
    },
    /// Error surfaced to the UI.
    Error(String),
}

/// Connection lifecycle states used for UI feedback.
#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Reconnecting { retry_in_ms: u64 },
    Disconnected { reason: String },
}

/// Commands sent from UI into the WebSocket client.
pub enum WebSocketCommand {
    SendMessage {
        message_id: String,
        conversation_id: String,
        recipient_id: String,
        content: String,
    },
    SendTyping {
        recipient_id: String,
        is_typing: bool,
    },
    Disconnect,
}

/// Handle to interact with the WebSocket client.
#[derive(Clone)]
pub struct WebSocketClient {
    command_tx: mpsc::UnboundedSender<WebSocketCommand>,
}

impl WebSocketClient {
    /// Connect to the WebSocket server and start background processing.
    pub fn connect(
        websocket_url: String,
        token: String,
        event_tx: mpsc::UnboundedSender<WebSocketEvent>,
        runtime: &tokio::runtime::Runtime,
    ) -> Self {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel::<WebSocketCommand>();

        // Run the WebSocket loop on the provided runtime.
        runtime.spawn(async move {
            let mut pending: VecDeque<WebSocketCommand> = VecDeque::new();
            let mut attempt: usize = 0;
            loop {
                // Capture any queued commands before attempting a connection.
                while let Ok(cmd) = command_rx.try_recv() {
                    if matches!(cmd, WebSocketCommand::Disconnect) {
                        return;
                    }
                    pending.push_back(cmd);
                }

                let token_to_use = session::get_token().unwrap_or_else(|| token.clone());
                let connect_url = format!("{}?token={}", websocket_url, token_to_use);
                let _ = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Connecting));

                match connect_async(&connect_url).await {
                    Ok((ws_stream, _)) => {
                        attempt = 0;
                        let _ = event_tx
                            .send(WebSocketEvent::ConnectionState(ConnectionStatus::Connected));
                        let (mut ws_write, mut ws_read) = ws_stream.split();

                        if let Err(e) =
                            flush_queue(&mut ws_write, &mut pending, &event_tx).await
                        {
                            let _ = event_tx.send(WebSocketEvent::ConnectionState(
                                ConnectionStatus::Disconnected {
                                    reason: format!("Send failed: {}", e),
                                },
                            ));
                            continue;
                        }

                        // Main loop: fan-in reads and commands.
                        loop {
                            tokio::select! {
                                Some(cmd) = command_rx.recv() => {
                                    if matches!(cmd, WebSocketCommand::Disconnect) {
                                        let _ = ws_write.send(Message::Close(None)).await;
                                        return;
                                    }
                                    pending.push_back(cmd);
                                    if let Err(e) = flush_queue(&mut ws_write, &mut pending, &event_tx).await {
                                        let _ = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: format!("Send failed: {}", e) }));
                                        break;
                                    }
                                }
                                msg = ws_read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            handle_incoming_text(&text, &event_tx);
                                        }
                                        Some(Ok(Message::Ping(p))) => {
                                            let _ = ws_write.send(Message::Pong(p)).await;
                                        }
                                        Some(Ok(Message::Close(_))) => {
                                            let _ = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: "Server closed connection".to_string() }));
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            let _ = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: format!("WebSocket error: {}", e) }));
                                            break;
                                        }
                                        None => {
                                            let _ = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: "Connection dropped".to_string() }));
                                            break
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(WebSocketEvent::ConnectionState(
                            ConnectionStatus::Disconnected {
                                reason: format!("Connect failed: {}", e),
                            },
                        ));
                    }
                }

                // Exponential backoff on reconnect attempts.
                attempt += 1;
                let backoff = calculate_backoff(attempt);
                let _ = event_tx.send(WebSocketEvent::ConnectionState(
                    ConnectionStatus::Reconnecting {
                        retry_in_ms: backoff.as_millis() as u64,
                    },
                ));
                tokio::time::sleep(backoff).await;
            }
        });

        Self { command_tx }
    }

    /// Send a chat message.
    pub fn send_message(
        &self,
        message_id: String,
        conversation_id: String,
        recipient_id: String,
        content: String,
    ) -> Result<(), String> {
        self.command_tx
            .send(WebSocketCommand::SendMessage {
                message_id,
                conversation_id,
                recipient_id,
                content,
            })
            .map_err(|e| format!("Failed to queue send: {}", e))
    }

    /// Send typing indicator.
    pub fn send_typing(&self, recipient_id: String, is_typing: bool) -> Result<(), String> {
        self.command_tx
            .send(WebSocketCommand::SendTyping {
                recipient_id,
                is_typing,
            })
            .map_err(|e| format!("Failed to queue typing: {}", e))
    }

    /// Disconnect WebSocket
    pub fn disconnect(&self) -> Result<(), String> {
         self.command_tx
            .send(WebSocketCommand::Disconnect)
            .map_err(|e| format!("Failed to queue disconnect: {}", e))
    }
}

async fn flush_queue<S>(
    ws_write: &mut S,
    pending: &mut VecDeque<WebSocketCommand>,
    event_tx: &mpsc::UnboundedSender<WebSocketEvent>,
) -> Result<(), tokio_tungstenite::tungstenite::Error>
where
    S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    while let Some(cmd) = pending.front() {
        let payload = match cmd {
            WebSocketCommand::SendMessage {
                message_id,
                conversation_id,
                recipient_id,
                content,
            } => match serde_json::to_string(&build_message_envelope(
                message_id.clone(),
                conversation_id.clone(),
                recipient_id.clone(),
                content.clone(),
            )) {
                Ok(p) => p,
                Err(e) => {
                    let _ = event_tx
                        .send(WebSocketEvent::Error(format!("Serialize error: {}", e)));
                    pending.pop_front();
                    continue;
                }
            },
            WebSocketCommand::SendTyping {
                recipient_id,
                is_typing,
            } => match serde_json::to_string(&build_typing_envelope(
                recipient_id.clone(),
                *is_typing,
            )) {
                Ok(p) => p,
                Err(e) => {
                    let _ = event_tx
                        .send(WebSocketEvent::Error(format!("Serialize error: {}", e)));
                    pending.pop_front();
                    continue;
                }
            },
            WebSocketCommand::Disconnect => {
                pending.pop_front();
                let _ = ws_write.send(Message::Close(None)).await;
                return Ok(());
            }
        };

        ws_write.send(Message::Text(payload)).await?;
        pending.pop_front();
    }

    Ok(())
}

fn calculate_backoff(attempt: usize) -> Duration {
    let ranges = [
        (0.5f64, 1.5f64),
        (1.5, 3.5),
        (3.0, 7.0),
        (7.0, 15.0),
        (15.0, 30.0),
    ];

    let (min, max) = if attempt < ranges.len() {
        ranges[attempt]
    } else {
        (30.0, 60.0)
    };

    jitter_delay(min, max)
}

fn jitter_delay(min_secs: f64, max_secs: f64) -> Duration {
    let span = max_secs - min_secs;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    let ratio = (nanos / 1_000_000_000_f64).clamp(0.0, 1.0);
    let millis = (min_secs + span * ratio) * 1000.0;
    Duration::from_millis(millis.round() as u64)
}

fn build_message_envelope(
    message_id: String,
    conversation_id: String,
    recipient_id: String,
    content: String,
) -> MessageEnvelope {
    let data = TextMessageData {
        sender_id: None,
        sender_username: None,
        recipient_id,
        content,
        conversation_id: Some(conversation_id),
        status: None,
    };

    MessageEnvelope {
        id: message_id,
        msg_type: "message".to_string(),
        timestamp: current_timestamp_ms(),
        data: serde_json::to_value(data).unwrap_or_default(),
    }
}

fn build_typing_envelope(recipient_id: String, is_typing: bool) -> MessageEnvelope {
    let data = TypingData {
        sender_id: None,
        sender_username: None,
        recipient_id,
        is_typing,
    };

    MessageEnvelope {
        id: Uuid::new_v4().to_string(),
        msg_type: "typing".to_string(),
        timestamp: current_timestamp_ms(),
        data: serde_json::to_value(data).unwrap_or_default(),
    }
}

fn current_timestamp_ms() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as u64
}

fn handle_incoming_text(text: &str, event_tx: &mpsc::UnboundedSender<WebSocketEvent>) {
    let envelope: Result<MessageEnvelopeWire, _> = serde_json::from_str(text);
    let envelope = match envelope {
        Ok(v) => v,
        Err(_) => {
            let _ = event_tx.send(WebSocketEvent::Error("Invalid message payload".into()));
            return;
        }
    };

    match envelope.msg_type.as_str() {
        "ack" => {
            let ack: Result<AckData, _> = serde_json::from_value(envelope.data.clone());
            if let Ok(ack) = ack {
                let _ = event_tx.send(WebSocketEvent::Ack {
                    message_id: ack.message_id,
                    status: ack.status,
                    conversation_id: ack.conversation_id,
                });
            }
        }
        "message" => {
            // Deserialize into TextMessageData; fall back to minimal fields if unknown.
            let msg: Result<TextMessageData, _> = serde_json::from_value(envelope.data.clone());
            if let Ok(msg) = msg {
                let _ = event_tx.send(WebSocketEvent::Message {
                    conversation_id: msg
                        .conversation_id
                        .unwrap_or_else(|| "unknown".to_string()),
                    message_id: envelope.id,
                    sender_username: msg
                        .sender_username
                        .unwrap_or_else(|| "Unknown".to_string()),
                    content: msg.content,
                    status: msg.status.unwrap_or_else(|| "sent".to_string()),
                    timestamp: envelope.timestamp,
                });
            }
        }
        "typing" => {
            let typing: Result<TypingData, _> = serde_json::from_value(envelope.data.clone());
            if let Ok(typing) = typing {
                let _ = event_tx.send(WebSocketEvent::Typing {
                    sender_id: typing.sender_id,
                    sender_username: typing
                        .sender_username
                        .unwrap_or_else(|| "Unknown".to_string()),
                    recipient_id: typing.recipient_id,
                    is_typing: typing.is_typing,
                });
            }
        }
        "presence" => {
            let presence: Result<PresenceData, _> = serde_json::from_value(envelope.data.clone());
            if let Ok(presence) = presence {
                let _ = event_tx.send(WebSocketEvent::Presence {
                    user_id: presence.user_id,
                    username: presence.username,
                    is_online: presence.is_online,
                    last_seen_at: presence.last_seen_at,
                });
            }
        }
        _ => {
            // Ignore unknown types for now
        }
    }
}

#[derive(Debug, Deserialize)]
struct MessageEnvelopeWire {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}
