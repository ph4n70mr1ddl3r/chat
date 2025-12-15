//! WebSocket client for sending/receiving chat messages and typing indicators.
//!
//! Runs on a background Tokio runtime and communicates with the UI through channels.

use chat_shared::protocol::{AckData, MessageEnvelope, TextMessageData, TypingData};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::connect_async;
use uuid::Uuid;

/// Events emitted by the WebSocket client.
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
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
    /// Error surfaced to the UI.
    Error(String),
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
            let connect_url = format!("{}?token={}", websocket_url, token);
            let mut backoff_secs = 1u64;

            loop {
                match connect_async(&connect_url).await {
                    Ok((ws_stream, _)) => {
                        backoff_secs = 1; // reset backoff on success
                        let (mut ws_write, mut ws_read) = ws_stream.split();

                        // Main loop: fan-in reads and commands.
                        loop {
                            tokio::select! {
                                Some(cmd) = command_rx.recv() => {
                                    match cmd {
                                        WebSocketCommand::SendMessage { message_id, conversation_id, recipient_id, content } => {
                                            let envelope = build_message_envelope(message_id, conversation_id, recipient_id, content);
                                            let payload = match serde_json::to_string(&envelope) {
                                                Ok(p) => p,
                                                Err(e) => {
                                                    let _ = event_tx.send(WebSocketEvent::Error(format!("Serialize error: {}", e)));
                                                    continue;
                                                }
                                            };
                                            if let Err(e) = ws_write.send(Message::Text(payload)).await {
                                                let _ = event_tx.send(WebSocketEvent::Error(format!("Send failed: {}", e)));
                                                break;
                                            }
                                        }
                                        WebSocketCommand::SendTyping { recipient_id, is_typing } => {
                                            let envelope = build_typing_envelope(recipient_id, is_typing);
                                            let payload = match serde_json::to_string(&envelope) {
                                                Ok(p) => p,
                                                Err(e) => {
                                                    let _ = event_tx.send(WebSocketEvent::Error(format!("Serialize error: {}", e)));
                                                    continue;
                                                }
                                            };
                                            if let Err(e) = ws_write.send(Message::Text(payload)).await {
                                                let _ = event_tx.send(WebSocketEvent::Error(format!("Send failed: {}", e)));
                                                break;
                                            }
                                        }
                                        WebSocketCommand::Disconnect => {
                                            let _ = ws_write.send(Message::Close(None)).await;
                                            return;
                                        }
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
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            let _ = event_tx.send(WebSocketEvent::Error(format!("WebSocket error: {}", e)));
                                            break;
                                        }
                                        None => break,
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(WebSocketEvent::Error(format!("Connect failed: {}", e)));
                    }
                }

                // Exponential backoff on reconnect attempts.
                tokio::time::sleep(Duration::from_secs(backoff_secs.min(30))).await;
                backoff_secs = (backoff_secs * 2).min(30);
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
