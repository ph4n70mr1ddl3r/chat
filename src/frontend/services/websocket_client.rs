//! WebSocket client for sending/receiving chat messages and typing indicators.
//!
//! Runs on a background Tokio runtime and communicates with the UI through channels.

use crate::services::session;
use chat_shared::protocol::{AckData, MessageEnvelope, PresenceData, TextMessageData, TypingData};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

fn max_reconnect_attempts() -> usize {
    std::env::var("MAX_RECONNECT_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// Maximum number of pending messages in the queue before applying backpressure.
/// When this limit is reached, the oldest messages will be dropped to prevent
/// memory exhaustion and alert the user.
const MAX_PENDING_QUEUE_SIZE: usize = 100;

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
    /// Heartbeat response received (server acknowledged ping)
    HeartbeatReceived,
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
                    push_to_queue(&mut pending, cmd, &event_tx);
                }

                let token_to_use = session::get_token().unwrap_or_else(|| token.clone());
                if let Err(e) = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Connecting)) {
                    tracing::error!("Failed to send connecting state: {}", e);
                }

                let request = match Request::builder()
                    .uri(&websocket_url)
                    .header("Sec-WebSocket-Protocol", format!("jwt.{}", token_to_use))
                    .body(())
                {
                    Ok(req) => req,
                    Err(e) => {
                        let _ = event_tx.send(WebSocketEvent::Error(format!("Failed to build request: {e}")));
                        let _ = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: "Failed to build request".to_string() }));
                        continue;
                    }
                };

                match tokio_tungstenite::connect_async(request).await {
                    Ok((ws_stream, _)) => {
                        attempt = 0;
                        if let Err(e) = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Connected)) {
                            tracing::error!("Failed to send connected state: {}", e);
                        }
                        let (mut ws_write, mut ws_read) = ws_stream.split();

                        if let Err(e) =
                            flush_queue(&mut ws_write, &mut pending, &event_tx).await
                        {
                            if let Err(send_err) = event_tx.send(WebSocketEvent::ConnectionState(
                                ConnectionStatus::Disconnected {
                                    reason: format!("Send failed: {e}"),
                                },
                            )) {
                                tracing::error!("Failed to send disconnected state: {}", send_err);
                            }
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
                                    push_to_queue(&mut pending, cmd, &event_tx);
                                    if let Err(e) = flush_queue(&mut ws_write, &mut pending, &event_tx).await {
                                        if let Err(send_err) = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: format!("Send failed: {e}") })) {
                                            tracing::error!("Failed to send disconnected state: {}", send_err);
                                        }
                                        break;
                                    }
                                }
                                msg = ws_read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            handle_incoming_text(&text, &event_tx);
                                        }
                                        Some(Ok(Message::Ping(p))) => {
                                            if let Err(e) = ws_write.send(Message::Pong(p)).await {
                                                tracing::error!("Failed to send pong: {}", e);
                                            }
                                        }
                                        Some(Ok(Message::Pong(_))) => {
                                            if let Err(e) = event_tx.send(WebSocketEvent::HeartbeatReceived) {
                                                tracing::error!("Failed to send heartbeat received event: {}", e);
                                            }
                                        }
                                        Some(Ok(Message::Close(_))) => {
                                            if let Err(e) = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: "Server closed connection".to_string() })) {
                                                tracing::error!("Failed to send disconnected state: {}", e);
                                            }
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            if let Err(send_err) = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: format!("WebSocket error: {e}") })) {
                                                tracing::error!("Failed to send disconnected state: {}", send_err);
                                            }
                                            break;
                                        }
                                        None => {
                                            if let Err(e) = event_tx.send(WebSocketEvent::ConnectionState(ConnectionStatus::Disconnected { reason: "Connection dropped".to_string() })) {
                                                tracing::error!("Failed to send disconnected state: {}", e);
                                            }
                                            break
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if let Err(send_err) = event_tx.send(WebSocketEvent::ConnectionState(
                            ConnectionStatus::Disconnected {
                                reason: format!("Connect failed: {e}"),
                            },
                        )) {
                            tracing::error!("Failed to send disconnected state: {}", send_err);
                        }
                    }
                }

                // Exponential backoff on reconnect attempts.
                attempt += 1;
                if attempt >= max_reconnect_attempts() {
                    if let Err(e) = event_tx.send(WebSocketEvent::Error(
                        "Max reconnect attempts reached".to_string(),
                    )) {
                        tracing::error!("Failed to send error event: {}", e);
                    }
                    if let Err(e) = event_tx.send(WebSocketEvent::ConnectionState(
                        ConnectionStatus::Disconnected {
                            reason: "Max reconnect attempts exceeded".to_string(),
                        },
                    )) {
                        tracing::error!("Failed to send disconnected state: {}", e);
                    }
                    return;
                }
                let backoff = calculate_backoff(attempt);
                if let Err(e) = event_tx.send(WebSocketEvent::ConnectionState(
                    ConnectionStatus::Reconnecting {
                        retry_in_ms: backoff.as_millis() as u64,
                    },
                )) {
                    tracing::error!("Failed to send reconnecting state: {}", e);
                }
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
            .map_err(|e| format!("Failed to queue send: {e}"))
    }

    /// Send typing indicator.
    pub fn send_typing(&self, recipient_id: String, is_typing: bool) -> Result<(), String> {
        self.command_tx
            .send(WebSocketCommand::SendTyping {
                recipient_id,
                is_typing,
            })
            .map_err(|e| format!("Failed to queue typing: {e}"))
    }

    /// Disconnect WebSocket
    pub fn disconnect(&self) -> Result<(), String> {
        self.command_tx
            .send(WebSocketCommand::Disconnect)
            .map_err(|e| format!("Failed to queue disconnect: {e}"))
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
            } => match build_message_envelope(
                message_id.clone(),
                conversation_id.clone(),
                recipient_id.clone(),
                content.clone(),
            ) {
                Ok(envelope) => match serde_json::to_string(&envelope) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Serialize error: {e}");
                        if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!("Serialize error: {e}"))) {
                            tracing::error!("Failed to send error event: {}", send_err);
                        }
                        pending.pop_front();
                        continue;
                    }
                },
                Err(e) => {
                    tracing::error!("Build envelope error: {e}");
                    if let Err(send_err) = event_tx.send(WebSocketEvent::Error(e)) {
                        tracing::error!("Failed to send error event: {}", send_err);
                    }
                    pending.pop_front();
                    continue;
                }
            },
            WebSocketCommand::SendTyping {
                recipient_id,
                is_typing,
            } => match build_typing_envelope(recipient_id.clone(), *is_typing) {
                Ok(envelope) => match serde_json::to_string(&envelope) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Serialize error: {e}");
                        if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!("Serialize error: {e}"))) {
                            tracing::error!("Failed to send error event: {}", send_err);
                        }
                        pending.pop_front();
                        continue;
                    }
                },
                Err(e) => {
                    tracing::error!("Build typing envelope error: {e}");
                    if let Err(send_err) = event_tx.send(WebSocketEvent::Error(e)) {
                        tracing::error!("Failed to send error event: {}", send_err);
                    }
                    pending.pop_front();
                    continue;
                }
            },
            WebSocketCommand::Disconnect => {
                pending.pop_front();
                if let Err(e) = ws_write.send(Message::Close(None)).await {
                    tracing::error!("Failed to send close message: {}", e);
                }
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
    let span = (max_secs - min_secs).max(0.0);
    let ratio = rand::random::<f64>();
    let millis = (min_secs + span * ratio) * 1000.0;
    Duration::from_millis(millis.round() as u64)
}

fn build_message_envelope(
    message_id: String,
    conversation_id: String,
    recipient_id: String,
    content: String,
) -> Result<MessageEnvelope, String> {
    let data = TextMessageData {
        sender_id: None,
        sender_username: None,
        recipient_id,
        content,
        conversation_id: Some(conversation_id),
        status: None,
    };

    let data_value =
        serde_json::to_value(data).map_err(|e| format!("Serialize message data: {e}"))?;

    Ok(MessageEnvelope {
        id: message_id,
        msg_type: "message".to_string(),
        timestamp: current_timestamp_ms(),
        data: data_value,
    })
}

fn build_typing_envelope(recipient_id: String, is_typing: bool) -> Result<MessageEnvelope, String> {
    let data = TypingData {
        sender_id: None,
        sender_username: None,
        recipient_id,
        is_typing,
    };

    let data_value =
        serde_json::to_value(data).map_err(|e| format!("Serialize typing data: {e}"))?;

    Ok(MessageEnvelope {
        id: Uuid::new_v4().to_string(),
        msg_type: "typing".to_string(),
        timestamp: current_timestamp_ms(),
        data: data_value,
    })
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_else(|_| {
            tracing::error!("System clock error - time is before Unix epoch");
            0
        })
}

/// Push a command to the pending queue with backpressure handling.
/// 
/// When the queue reaches MAX_PENDING_QUEUE_SIZE, the oldest message is dropped
/// and an error event is sent to notify the user. This prevents unbounded memory
/// growth when the server is slow or the connection is unstable.
fn push_to_queue(
    pending: &mut VecDeque<WebSocketCommand>,
    cmd: WebSocketCommand,
    event_tx: &mpsc::UnboundedSender<WebSocketEvent>,
) {
    if pending.len() >= MAX_PENDING_QUEUE_SIZE {
        // Drop oldest message to make room for new one
        if let Some(dropped) = pending.pop_front() {
            tracing::warn!(
                "Message queue full ({} messages), dropping oldest message: {:?}",
                MAX_PENDING_QUEUE_SIZE,
                dropped
            );
            if let Err(e) = event_tx.send(WebSocketEvent::Error(
                format!("Connection slow - message queue full ({} messages), dropped oldest message. \
                         Consider waiting for delivery confirmation before sending more messages.",
                        MAX_PENDING_QUEUE_SIZE)
            )) {
                tracing::error!("Failed to send queue full error: {}", e);
            }
        }
    }
    pending.push_back(cmd);
}

fn handle_incoming_text(text: &str, event_tx: &mpsc::UnboundedSender<WebSocketEvent>) {
    let envelope: Result<MessageEnvelopeWire, _> = serde_json::from_str(text);
    let envelope = match envelope {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Invalid message payload: {e}");
            if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!(
                "Invalid message payload: {e}"
            ))) {
                tracing::error!("Failed to send error event: {}", send_err);
            }
            return;
        }
    };

    match envelope.msg_type.as_str() {
        "ack" => {
            let ack: Result<AckData, _> = serde_json::from_value(envelope.data.clone());
            match ack {
                Ok(ack) => {
                    if let Err(e) = event_tx.send(WebSocketEvent::Ack {
                        message_id: ack.message_id,
                        status: ack.status,
                        conversation_id: ack.conversation_id,
                    }) {
                        tracing::error!("Failed to send ack event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse ack: {e}");
                    if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!("Failed to parse ack: {e}"))) {
                        tracing::error!("Failed to send error event: {}", send_err);
                    }
                }
            }
        }
        "message" => {
            let msg: Result<TextMessageData, _> = serde_json::from_value(envelope.data.clone());
            match msg {
                Ok(msg) => {
                    if let Err(e) = event_tx.send(WebSocketEvent::Message {
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
                    }) {
                        tracing::error!("Failed to send message event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse message: {e}");
                    if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!(
                        "Failed to parse message: {e}"
                    ))) {
                        tracing::error!("Failed to send error event: {}", send_err);
                    }
                }
            }
        }
        "typing" => {
            let typing: Result<TypingData, _> = serde_json::from_value(envelope.data.clone());
            match typing {
                Ok(typing) => {
                    if let Err(e) = event_tx.send(WebSocketEvent::Typing {
                        sender_id: typing.sender_id,
                        sender_username: typing
                            .sender_username
                            .unwrap_or_else(|| "Unknown".to_string()),
                        recipient_id: typing.recipient_id,
                        is_typing: typing.is_typing,
                    }) {
                        tracing::error!("Failed to send typing event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse typing: {e}");
                    if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!(
                        "Failed to parse typing: {e}"
                    ))) {
                        tracing::error!("Failed to send error event: {}", send_err);
                    }
                }
            }
        }
        "presence" => {
            let presence: Result<PresenceData, _> = serde_json::from_value(envelope.data.clone());
            match presence {
                Ok(presence) => {
                    if let Err(e) = event_tx.send(WebSocketEvent::Presence {
                        user_id: presence.user_id,
                        username: presence.username,
                        is_online: presence.is_online,
                        last_seen_at: presence.last_seen_at,
                    }) {
                        tracing::error!("Failed to send presence event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse presence: {e}");
                    if let Err(send_err) = event_tx.send(WebSocketEvent::Error(format!(
                        "Failed to parse presence: {e}"
                    ))) {
                        tracing::error!("Failed to send error event: {}", send_err);
                    }
                }
            }
        }
        _ => {
            tracing::error!("Unknown message type: {}", envelope.msg_type);
            if let Err(e) = event_tx.send(WebSocketEvent::Error(format!(
                "Unknown message type: {}",
                envelope.msg_type
            ))) {
                tracing::error!("Failed to send error event: {}", e);
            }
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
