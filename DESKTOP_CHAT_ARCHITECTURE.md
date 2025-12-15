# Desktop Chat Application - Architecture & Implementation Guide
## Rust + Slint + WebSocket for Windows

**Date:** December 15, 2025  
**Technology Stack:** Rust 1.75+, Slint 1.14+, Tokio, Tungstenite  
**Target Platform:** Windows

---

## 1. Slint Architecture for Chat UIs

### Decision
Implement a **hierarchical component composition pattern** with:
- Root container managing window state
- Separated UI sections as sub-components (main chat area, sidebar, conversation list)
- Stateful model-driven components for message display
- Reusable atomic components (message bubble, user item, input field)

### Rationale
- **Separation of Concerns**: Each component has a single responsibility (UI rendering vs. data management)
- **Reusability**: Atomic components can be composed into complex UIs without duplication
- **Maintainability**: Clear hierarchy makes it easier to reason about data flow and state updates
- **Performance**: Slint's reactive property system efficiently re-renders only changed elements

### Implementation Considerations

#### Component Hierarchy Structure
```
AppWindow (export)
├── TopBar (window controls, user info)
├── SplitLayout (HorizontalLayout)
│   ├── SidebarPanel
│   │   ├── ConversationList
│   │   │   └── ConversationItem (for each conversation)
│   │   └── UserList
│   │       └── UserItem (for each user)
│   └── ChatPanel
│       ├── MessageArea
│       │   └── MessageBubble (for each message)
│       └── InputSection
│           ├── TextField (text input)
│           └── SendButton
```

#### Key Slint Patterns

**1. Separate UI Markup from Business Logic**
- Place `.slint` files in a dedicated `ui/` directory
- Keep component definitions focused on layout and styling
- Use callbacks to communicate with Rust code
- Use globals for application-wide state (user info, theme)

```slint
// ui/components/message-bubble.slint
export component MessageBubble {
    in property <string> text;
    in property <string> author;
    in property <bool> is-own-message;
    in property <string> timestamp;
    
    Rectangle {
        background: is-own-message ? #4a90e2 : #e8e8e8;
        BorderRadius { radius: 8px; }
        
        VerticalLayout {
            padding: 8px;
            Text { text: author; font-weight: bold; }
            Text { text: text; }
            Text { text: timestamp; font-size: 11px; }
        }
    }
}
```

**2. Use Model-Driven Lists**
- Leverage Slint's `Model` trait and `for` loops in markup
- Property arrays automatically bind to UI lists
- The renderer handles virtualization for large lists automatically

```slint
// ui/panels/chat-panel.slint
export component ChatPanel {
    in property <[MessageData]> messages <=> message-model.data;
    
    VerticalLayout {
        message-list := ListView {
            for message in messages: MessageBubble {
                text: message.text;
                author: message.author;
                is-own-message: message.is-own;
                timestamp: message.timestamp;
            }
        }
    }
}
```

**3. Stateful Component Pattern**
- Use root component properties to manage UI state
- Bind properties bidirectionally (`<=>`) for two-way updates
- Callbacks trigger Rust-side logic

```slint
export component AppWindow {
    // Application state
    in property <[ConversationData]> conversations;
    in property <[UserData]> online-users;
    in property <ConversationData> current-conversation;
    
    // UI state (should be ephemeral, not persisted)
    property <bool> sidebar-expanded: true;
    property <string> search-query: "";
    
    // Callbacks to Rust
    callback send-message(string);
    callback select-conversation(ConversationData);
    callback search-conversations(string);
}
```

### Performance Considerations
1. **Virtualization**: Slint's ListView automatically virtualizes - only visible items are rendered
2. **Reactive Updates**: Changes to properties only re-render affected components
3. **Avoid Expensive Computations in Bindings**: Keep bindings simple; move complex logic to Rust
4. **Image Caching**: Use `Image` component with proper resource management; avoid loading from disk repeatedly
5. **Animations**: Keep animations lightweight; use state transitions instead of continuous polling

---

## 2. WebSocket Client Integration

### Decision
Implement a **multi-threaded async architecture** with:
- Dedicated WebSocket thread using Tokio
- Message queue (MPSC channel) for UI→Network communication
- Event callback system for Network→UI communication using `invoke_from_event_loop()`
- Connection state machine with automatic reconnection

### Rationale
- **Non-blocking UI**: Network operations don't block the UI event loop
- **Clean Separation**: Networking logic isolated from UI
- **Type Safety**: Rust's type system catches errors at compile time
- **Async Efficiency**: Tokio handles thousands of concurrent connections efficiently

### Implementation Considerations

#### Architecture Overview
```
┌─────────────────────────────────────────┐
│         Slint UI Event Loop             │
│    (runs on main thread)                │
└────────────────┬────────────────────────┘
                 │ invoke_from_event_loop
                 ▼
        ┌───────────────────┐
        │  UI Message Queue │
        │  (MPSC receiver)  │
        └───────────────────┘
                 │
        ┌────────┴──────────┐
        │                   │
    ┌───▼────┐      ┌──────▼──────┐
    │ Update │      │ Update      │
    │ Visible│      │ Hidden      │
    │ State  │      │ State       │
    │        │      │ (history)   │
    └────────┘      └─────────────┘

┌─────────────────────────────────────────┐
│    WebSocket Worker Thread (Tokio)      │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  WebSocket Connection Handler   │   │
│  │  - Connect/Disconnect           │   │
│  │  - Send messages                │   │
│  │  - Receive & parse messages     │   │
│  │  - Ping/pong keepalive          │   │
│  │  - Reconnection logic           │   │
│  └─────────────────────────────────┘   │
│              │                          │
│  ┌──────────┴──────────────┐           │
│  │ Command Channel (MPSC)  │           │
│  │ (receiver)              │           │
│  └─────────────────────────┘           │
│              │                          │
│  ┌──────────┴──────────────────┐       │
│  │ Event Broadcast (broadcast) │       │
│  │ (send to UI)                │       │
│  └─────────────────────────────┘       │
└─────────────────────────────────────────┘
```

#### WebSocket Client Module
```rust
// src/network/mod.rs
pub mod ws_client;
pub mod message;
pub mod state;

use tokio::sync::{mpsc, broadcast};
use std::sync::Arc;

pub struct WebSocketClient {
    command_tx: mpsc::Sender<ClientCommand>,
    event_rx: broadcast::Receiver<ClientEvent>,
    state: Arc<ConnectionState>,
}

#[derive(Clone, Debug)]
pub enum ClientCommand {
    Connect { url: String },
    Disconnect,
    SendMessage { content: String, conversation_id: String },
    Authenticate { token: String },
}

#[derive(Clone, Debug)]
pub enum ClientEvent {
    Connected,
    Disconnected { reason: String },
    MessageReceived { 
        id: String, 
        author: String, 
        content: String,
        timestamp: i64,
        conversation_id: String,
    },
    UserJoined { user_id: String, username: String },
    UserLeft { user_id: String },
    TypingIndicator { user_id: String, is_typing: bool },
    Error { message: String },
    Reconnecting { attempt: u32 },
}

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting(u32),
    Failed(String),
}

impl WebSocketClient {
    pub fn new() -> Self { /* ... */ }
    
    pub async fn run(self) { /* ... */ }
    
    pub fn send_command(&self, cmd: ClientCommand) -> Result<()> {
        self.command_tx.try_send(cmd)?;
        Ok(())
    }
    
    pub fn subscribe_events(&self) -> broadcast::Receiver<ClientEvent> {
        self.event_rx.subscribe()
    }
}
```

#### Integration with Slint UI
```rust
// src/ui_bridge.rs
use slint::Weak;
use tokio::sync::broadcast;

pub struct UIBridge {
    ui_handle: Weak<AppWindow>,
    ws_client: Arc<WebSocketClient>,
    event_rx: broadcast::Receiver<ClientEvent>,
}

impl UIBridge {
    pub fn new(
        ui_handle: Weak<AppWindow>,
        ws_client: Arc<WebSocketClient>,
    ) -> Self {
        let event_rx = ws_client.subscribe_events();
        Self {
            ui_handle,
            ws_client,
            event_rx,
        }
    }
    
    pub async fn start_event_loop(mut self) {
        while let Ok(event) = self.event_rx.recv().await {
            // Use invoke_from_event_loop to update UI from async context
            match event {
                ClientEvent::MessageReceived { 
                    id, author, content, timestamp, conversation_id 
                } => {
                    let ui = self.ui_handle.clone();
                    let msg_data = MessageData {
                        id: id.into(),
                        author: author.into(),
                        text: content.into(),
                        timestamp: timestamp.into(),
                        is_own: false,
                    };
                    
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui.upgrade() {
                            // Update UI state
                            let mut messages = ui.get_messages();
                            messages.push(msg_data);
                            ui.set_messages(messages);
                            
                            // Auto-scroll to bottom
                            ui.invoke_scroll_to_bottom();
                        }
                    }).ok();
                }
                ClientEvent::Connected => {
                    let ui = self.ui_handle.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_connection_status("Connected");
                        }
                    }).ok();
                }
                ClientEvent::Error { message } => {
                    let ui = self.ui_handle.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_error_message(message.into());
                        }
                    }).ok();
                }
                _ => {}
            }
        }
    }
}
```

#### Main Application Setup
```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // Create WebSocket client
    let ws_client = Arc::new(WebSocketClient::new());
    
    // Spawn WebSocket worker thread
    let ws_client_clone = ws_client.clone();
    tokio::spawn(async move {
        ws_client_clone.run().await;
    });
    
    // Initialize UI
    let ui = AppWindow::new()?;
    let weak_ui = ui.as_weak();
    
    // Create UI bridge
    let bridge = UIBridge::new(weak_ui.clone(), ws_client.clone());
    
    // Start event loop (converts UI events to network commands)
    tokio::spawn(async move {
        bridge.start_event_loop().await;
    });
    
    // Slint UI event handlers
    let ws_client_clone = ws_client.clone();
    ui.on_send_message(move |msg| {
        let client = ws_client_clone.clone();
        tokio::spawn(async move {
            let cmd = ClientCommand::SendMessage {
                content: msg.to_string(),
                conversation_id: "current".to_string(),
            };
            client.send_command(cmd).ok();
        });
    });
    
    // Connect button handler
    let ws_client_clone = ws_client.clone();
    ui.on_connect(move || {
        let client = ws_client_clone.clone();
        tokio::spawn(async move {
            client.send_command(ClientCommand::Connect {
                url: "ws://localhost:8080".to_string(),
            }).ok();
        });
    });
    
    // Run UI
    ui.run()?;
    Ok(())
}
```

### Performance Considerations
1. **Message Batching**: Queue multiple messages before sending to avoid network overhead
2. **Backpressure Handling**: Drop lowest-priority events if queue fills up (e.g., typing indicators)
3. **Connection Pooling**: Reuse single WebSocket connection; avoid reconnects for each operation
4. **Keepalive**: Implement ping/pong frames to detect stale connections
5. **Memory Management**: Limit in-memory message history; use database for persistent storage

---

## 3. State Management in Slint Apps

### Decision
Implement a **layered state architecture**:
- **Persistent State**: User data, conversation history → database/files
- **Application State**: Active user info, auth token → Globals (singletons)
- **UI State**: Sidebar expanded, search query, scroll position → Component properties
- **Transient State**: Typing indicators, presence info → Broadcast channels

### Rationale
- **Clear Boundaries**: Different layers handle different concerns
- **Efficient Updates**: Only affected layers re-render when state changes
- **Testability**: Each layer can be tested independently
- **Scalability**: Supports complex applications without state explosion

### Implementation Considerations

#### Global Singletons Pattern
```slint
// ui/globals.slint
export global ApplicationState {
    // User authentication state
    in property <string> current-user-id;
    in property <string> current-username;
    in property <string> auth-token;
    in property <bool> is-authenticated;
    
    // Application-wide settings
    in property <string> theme: "light";
    in property <string> language: "en";
    in property <bool> notifications-enabled: true;
    
    // Connection state
    in property <string> connection-status: "disconnected"; // disconnected, connecting, connected
    in property <string> error-message: "";
    
    // Callbacks for state changes
    callback on-logout();
    callback on-theme-changed(string);
}

export global NotificationCenter {
    callback show-notification(string, string, int);
    callback show-error(string);
    callback show-warning(string);
}
```

#### Rust State Container
```rust
// src/state/mod.rs
pub mod auth;
pub mod conversation;
pub mod user;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    auth: Arc<RwLock<AuthState>>,
    conversations: Arc<RwLock<ConversationState>>,
    users: Arc<RwLock<UserState>>,
}

pub struct AuthState {
    current_user_id: String,
    username: String,
    auth_token: String,
    is_authenticated: bool,
}

pub struct ConversationState {
    conversations: HashMap<String, ConversationData>,
    current_conversation_id: Option<String>,
    message_cache: LruCache<String, Vec<MessageData>>, // for performance
}

pub struct UserState {
    online_users: HashSet<String>,
    user_info: HashMap<String, UserData>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth: Arc::new(RwLock::new(AuthState::default())),
            conversations: Arc::new(RwLock::new(ConversationState::new())),
            users: Arc::new(RwLock::new(UserState::new())),
        }
    }
    
    pub async fn login(&self, username: &str, password: &str) -> Result<String> {
        // Authentication logic
        let token = authenticate(username, password).await?;
        
        let mut auth = self.auth.write().await;
        auth.auth_token = token.clone();
        auth.username = username.to_string();
        auth.is_authenticated = true;
        
        Ok(token)
    }
    
    pub async fn get_conversations(&self) -> Vec<ConversationData> {
        let conversations = self.conversations.read().await;
        conversations.conversations.values().cloned().collect()
    }
    
    pub async fn add_message(&self, conv_id: &str, msg: MessageData) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        conversations
            .message_cache
            .get_mut(conv_id)
            .ok_or(anyhow::anyhow!("Conversation not found"))?
            .push(msg);
        Ok(())
    }
}
```

#### Syncing State Between Rust and Slint
```rust
// src/ui_state_sync.rs
use slint::{Weak, ComponentHandle};

pub struct StateSync {
    ui_handle: Weak<AppWindow>,
    app_state: Arc<AppState>,
}

impl StateSync {
    pub async fn sync_auth_state(&self) {
        let auth = self.app_state.auth.read().await;
        if let Some(ui) = self.ui_handle.upgrade() {
            let global = ui.global::<ApplicationState>();
            global.set_current_user_id(auth.current_user_id.clone().into());
            global.set_current_username(auth.username.clone().into());
            global.set_is_authenticated(auth.is_authenticated);
        }
    }
    
    pub async fn sync_conversations(&self) {
        let conversations = self.app_state.get_conversations().await;
        let conv_data: Vec<ConversationData> = conversations
            .into_iter()
            .map(|c| ConversationData {
                id: c.id.into(),
                name: c.name.into(),
                last_message: c.last_message.into(),
                unread_count: c.unread_count as i32,
            })
            .collect();
        
        if let Some(ui) = self.ui_handle.upgrade() {
            ui.set_conversations(conv_data.into());
        }
    }
    
    pub async fn sync_messages(&self, conversation_id: &str) {
        let messages = self.app_state
            .get_messages(conversation_id)
            .await
            .unwrap_or_default();
            
        let msg_data: Vec<MessageData> = messages
            .into_iter()
            .map(|m| MessageData {
                id: m.id.into(),
                author: m.author.into(),
                text: m.content.into(),
                timestamp: m.timestamp.into(),
                is_own: m.is_own_message,
            })
            .collect();
        
        if let Some(ui) = self.ui_handle.upgrade() {
            ui.set_messages(msg_data.into());
        }
    }
}
```

#### Reactive Message Display
```slint
// ui/panels/message-list.slint
import { ScrollView } from "std-widgets.slint";

export component MessageList {
    in property <[MessageData]> messages;
    in property <string> current-user-id;
    
    callback message-selected(MessageData);
    
    ScrollView {
        VerticalLayout {
            spacing: 4px;
            padding: 8px;
            
            for msg[i] in messages: MessageBubble {
                text: msg.text;
                author: msg.author;
                timestamp: msg.timestamp;
                is-own-message: msg.author == current-user-id;
                
                // Add visual separator between users
                Rectangle {
                    height: msg[i] != messages[i-1] 
                        && messages[i-1].author != msg.author 
                        ? 4px : 0px;
                    background: transparent;
                }
            }
        }
    }
}
```

### Performance Considerations
1. **Lazy Loading**: Load conversation history on demand, not all at once
2. **Message Pagination**: Keep only N recent messages in memory; paginate older ones
3. **Caching Strategy**: Use LRU cache for frequently accessed conversations
4. **Debouncing**: Debounce rapid state updates (e.g., typing indicators) before UI sync
5. **Lock Contention**: Use fine-grained locks per data structure, not a global lock

---

## 4. Message Rendering & Performance

### Decision
Implement a **virtual scrolling architecture** with:
- Dynamic message grouping (by date/sender)
- Lazy image loading with placeholder support
- Message deduplication and caching
- Efficient text rendering with line breaking

### Rationale
- **Handles Large Datasets**: Virtual scrolling renders only visible messages
- **Responsive Scrolling**: Smooth 60 FPS scrolling even with thousands of messages
- **Low Memory Footprint**: Avoids keeping all messages in memory
- **Fast Loading**: Incremental message loading as user scrolls

### Implementation Considerations

#### Virtual List Component
```slint
// ui/components/virtual-message-list.slint
export component VirtualMessageList {
    in property <[MessageData]> messages;
    in property <int> visible-item-count: 20;
    
    // Slint automatically virtualizes ListView
    ListView {
        for msg in messages: MessageBubble {
            text: msg.text;
            author: msg.author;
            timestamp: msg.timestamp;
            is-own-message: msg.author == current-user-id;
            
            // Dynamic height based on content
            height: msg.text.length > 100 ? 120px : 60px;
        }
    }
}
```

**Why Slint's ListView is efficient:**
- Only renders items currently visible in viewport
- Automatic viewport management
- O(1) scroll performance regardless of list size
- Lazy layout computation for off-screen items

#### Message Grouping Strategy
```rust
// src/ui/message_grouping.rs
pub struct MessageGroup {
    pub date: NaiveDate,
    pub messages: Vec<MessageData>,
}

pub fn group_messages_by_date(
    messages: &[MessageData],
) -> Vec<(String, Vec<MessageData>)> {
    use chrono::NaiveDate;
    use std::collections::BTreeMap;
    
    let mut groups: BTreeMap<NaiveDate, Vec<MessageData>> = BTreeMap::new();
    
    for msg in messages {
        let date = NaiveDate::from_timestamp_millis(msg.timestamp.parse::<i64>().unwrap_or(0))
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        groups.entry(date).or_insert_with(Vec::new).push(msg.clone());
    }
    
    groups
        .into_iter()
        .map(|(date, msgs)| {
            let label = format_date_label(date);
            (label, msgs)
        })
        .collect()
}

fn format_date_label(date: NaiveDate) -> String {
    let today = chrono::Local::now().naive_local().date();
    match (today - date).num_days() {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        days if days < 7 => format!("{} days ago", days),
        _ => date.format("%B %d, %Y").to_string(),
    }
}
```

#### Lazy Image Loading
```slint
// ui/components/lazy-image.slint
export component LazyImage {
    in property <string> src;
    in property <int> width: 200;
    in property <int> height: 200;
    
    // Show placeholder while loading
    property <bool> is-loaded: false;
    
    Image {
        source: src;
        width: root.width;
        height: root.height;
        
        // Fallback for broken images
        // Note: Slint doesn't have built-in image error handling,
        // so we rely on placeholder visibility
    }
    
    // Placeholder (shown while image loads)
    Rectangle {
        visible: !is-loaded;
        background: #f0f0f0;
        
        Text {
            text: "Loading...";
            color: #999;
        }
    }
}
```

#### Text Rendering Optimization
```slint
// ui/components/rich-text-bubble.slint
export component RichTextBubble {
    in property <string> text;
    in property <int> max-width: 400;
    in property <[string]> mentioned-users;
    
    Rectangle {
        background: #f0f0f0;
        border-radius: 8px;
        padding: 8px;
        
        Text {
            text: root.text;
            // Critical for performance:
            // - Limit text length per message
            // - Use ellipsis for long text
            // - Avoid complex markup (use simple text only)
            wrap: word-wrap;
            font-size: 14px;
            color: #333;
            
            // Note: Emoji rendering might be slow - keep emoji count low
        }
    }
}
```

#### Rust-side Message Cache
```rust
// src/cache/message_cache.rs
use lru::LruCache;
use std::num::NonZeroUsize;
use std::collections::HashMap;

pub struct MessageCache {
    // Cache per conversation
    caches: HashMap<String, LruCache<u64, Vec<MessageData>>>,
    // Conversation ID -> list of cached page offsets
    pagination: HashMap<String, Vec<u64>>,
}

impl MessageCache {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
            pagination: HashMap::new(),
        }
    }
    
    pub fn get_or_fetch(
        &mut self,
        conv_id: &str,
        page: u64,
        page_size: usize,
    ) -> Option<Vec<MessageData>> {
        self.caches
            .entry(conv_id.to_string())
            .or_insert_with(|| {
                LruCache::new(NonZeroUsize::new(10).unwrap()) // Keep 10 pages in memory
            })
            .get(&page)
            .cloned()
    }
    
    pub fn cache_messages(
        &mut self,
        conv_id: &str,
        page: u64,
        messages: Vec<MessageData>,
    ) {
        self.caches
            .entry(conv_id.to_string())
            .or_insert_with(|| LruCache::new(NonZeroUsize::new(10).unwrap()))
            .put(page, messages);
    }
}
```

#### Scroll Position Tracking
```rust
// src/ui/scroll_manager.rs
pub struct ScrollManager {
    conversation_id: String,
    last_scroll_position: f32,
    auto_scroll_enabled: bool,
}

impl ScrollManager {
    pub fn on_scroll(&mut self, position: f32) {
        self.last_scroll_position = position;
        
        // Detect if user scrolled up (want older messages)
        if position < 10.0 {
            // Trigger load more
        }
        
        // Detect if user is at bottom (should auto-scroll new messages)
        self.auto_scroll_enabled = position > 90.0;
    }
    
    pub fn should_auto_scroll(&self) -> bool {
        self.auto_scroll_enabled
    }
    
    pub fn restore_scroll_position(&self) -> f32 {
        self.last_scroll_position
    }
}
```

### Performance Considerations
1. **Memory Limit**: Cap messages in memory to 1000-5000 depending on message size
2. **Network Paging**: Load messages in pages (50-100 per page)
3. **Debounce Scroll Events**: Only process scroll every 200ms
4. **Image Caching**: Use a disk-based image cache for avatars
5. **Text Metrics**: Compute text height once, cache for rendering

---

## 5. Desktop Platform Integration (Windows)

### Decision
Implement a **Windows-native integration layer** with:
- System tray icon and context menu
- Desktop notifications via Windows Runtime API
- Window management (restore, minimize, close)
- Hotkeys for quick access
- File drag-drop support for file sharing

### Rationale
- **Native Feel**: Users expect Windows apps to behave like Windows apps
- **Accessibility**: Integrates with Windows accessibility features
- **System Integration**: Works with Windows notification center, taskbar
- **User Productivity**: Quick access via hotkeys and tray icon

### Implementation Considerations

#### System Tray Integration
```rust
// src/platform/windows/tray.rs
use tray_icon::{TrayIconBuilder, TrayIcon, menu::Menu};
use winit::window::Window;

pub struct TrayManager {
    tray_icon: Option<TrayIcon>,
}

impl TrayManager {
    pub fn new(window: &Window) -> Result<Self> {
        // Create context menu
        let menu = Menu::new();
        
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Chat App")
            .with_icon(load_icon("assets/icon.png")?)
            .build()?;
        
        Ok(Self {
            tray_icon: Some(tray_icon),
        })
    }
    
    pub fn show_tooltip(&self, message: &str) {
        // Show hover tooltip with message preview
    }
    
    pub fn set_notification_badge(&self, count: u32) {
        // Update taskbar badge with unread count
    }
}
```

#### Desktop Notifications
```rust
// src/platform/windows/notifications.rs
use notify_rust::Notification;
use windows::Win32::UI::Notifications::*;

pub struct NotificationManager;

impl NotificationManager {
    pub fn show_message_notification(
        author: &str,
        message: &str,
        conversation_id: &str,
    ) -> Result<()> {
        // Use Windows Toast Notifications for better integration
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Com::*;
            
            // Initialize COM
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)?;
            }
            
            // Create toast notification
            let toast = Notification::new()
                .summary(author)
                .body(message)
                .timeout(notify_rust::Timeout::Milliseconds(5000));
            
            toast.show()?;
        }
        
        Ok(())
    }
    
    pub fn show_connection_status(status: &str) -> Result<()> {
        Notification::new()
            .summary("Chat Application")
            .body(status)
            .timeout(notify_rust::Timeout::Milliseconds(3000))
            .show()?;
        
        Ok(())
    }
}
```

#### Hotkey Management
```rust
// src/platform/windows/hotkeys.rs
use hotkey_rs::{Hotkey, Modifiers};
use std::sync::Arc;
use std::sync::mpsc;

pub struct HotKeyManager {
    hotkeys: Vec<Hotkey>,
}

impl HotKeyManager {
    pub fn new(tx: mpsc::Sender<HotkeyEvent>) -> Result<Self> {
        let mut hotkeys = Vec::new();
        
        // Ctrl+Alt+C: Focus chat window
        let hotkey1 = Hotkey::new(
            Some(Modifiers::CTRL | Modifiers::ALT),
            hotkey_rs::keys::Keys::C,
        );
        
        hotkey1.register(move || {
            tx.send(HotkeyEvent::FocusChatWindow).ok();
        })?;
        
        hotkeys.push(hotkey1);
        
        // Ctrl+Alt+M: Mute/unmute audio
        let hotkey2 = Hotkey::new(
            Some(Modifiers::CTRL | Modifiers::ALT),
            hotkey_rs::keys::Keys::M,
        );
        
        hotkey2.register(move || {
            tx.send(HotkeyEvent::ToggleMute).ok();
        })?;
        
        hotkeys.push(hotkey2);
        
        Ok(Self { hotkeys })
    }
}

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    FocusChatWindow,
    ToggleMute,
    QuickReply,
}
```

#### File Drag-Drop Support
```slint
// ui/panels/input-area.slint
export component InputArea {
    in property <bool> has-file: false;
    
    DropArea {
        drop(files) => {
            if files.len() > 0 {
                root.has-file = true;
                root.on-files-dropped(files);
            }
        }
    }
    
    callback on-files-dropped([string]);
}
```

```rust
// src/ui/file_handler.rs
use slint::ComponentHandle;
use std::path::PathBuf;

pub struct FileHandler;

impl FileHandler {
    pub async fn handle_dropped_files(
        files: Vec<PathBuf>,
        conversation_id: String,
        ws_client: Arc<WebSocketClient>,
    ) -> Result<()> {
        for file_path in files {
            // Check file size
            let metadata = std::fs::metadata(&file_path)?;
            if metadata.len() > 50 * 1024 * 1024 {
                // Skip files > 50MB
                continue;
            }
            
            // Upload file
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            
            let file_data = std::fs::read(&file_path)?;
            
            ws_client.send_command(ClientCommand::SendFile {
                conversation_id,
                filename: file_name.to_string(),
                data: file_data,
            })?;
        }
        
        Ok(())
    }
}
```

#### Window Management
```rust
// src/platform/windows/window_manager.rs
use slint::Window;

pub struct WindowManager {
    window: Window,
}

impl WindowManager {
    pub fn minimize(&self) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::*;
            
            if let Some(hwnd) = self.get_hwnd() {
                unsafe {
                    ShowWindow(hwnd, SW_MINIMIZE);
                }
            }
        }
    }
    
    pub fn restore(&self) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::*;
            
            if let Some(hwnd) = self.get_hwnd() {
                unsafe {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
            }
        }
    }
    
    pub fn always_on_top(&self, enabled: bool) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::*;
            
            if let Some(hwnd) = self.get_hwnd() {
                unsafe {
                    let flags = if enabled { HWND_TOPMOST } else { HWND_NOTOPMOST };
                    SetWindowPos(hwnd, flags, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    fn get_hwnd(&self) -> Option<HWND> {
        use windows::Win32::Foundation::HWND;
        // Extract HWND from Slint window handle
        // This requires access to raw window handle
        None
    }
}
```

### Performance Considerations
1. **Notification Throttling**: Limit notifications to max 1 per second per conversation
2. **Tray Icon Updates**: Batch badge updates; don't update on every message
3. **Hotkey Overhead**: Register only essential hotkeys (2-3 max)
4. **File Operations**: Use async I/O for file uploads/downloads
5. **Memory for Tray**: Keep tray icon small (< 64KB when loaded)

---

## Summary: Decision Matrix

| Aspect | Decision | Key Benefit | Trade-off |
|--------|----------|------------|-----------|
| **Architecture** | Hierarchical components + Globals | Clean separation | Slight learning curve |
| **Networking** | Tokio + MPSC channels | Non-blocking, responsive | Added complexity |
| **State** | Layered (persistent/app/UI) | Clear boundaries | More code upfront |
| **Rendering** | Virtual lists + grouping | Handles thousands of messages | Need lazy loading logic |
| **Platform** | Win32 APIs via windows crate | Native feel | Windows-only features |

---

## Technology Stack Recommendations

### Core Dependencies
```toml
[dependencies]
slint = { version = "1.14", features = ["backend-winit"] }
tokio = { version = "1", features = ["full"] }
tungstenite = "0.28"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
lru = "0.12"
chrono = "0.4"
log = "0.4"
env_logger = "0.11"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_UI_Notifications"] }
notify-rust = "4.11"
tray-icon = "0.1"
hotkey-rs = "0.2"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
```

---

## Next Steps

1. **Create project structure** following the recommended directory layout
2. **Implement WebSocket client** with connection state machine
3. **Build Slint UI components** following hierarchical pattern
4. **Integrate UI with networking** via bridges and event loops
5. **Add Windows-specific features** (tray, notifications, hotkeys)
6. **Performance testing** with 5K+ messages and rapid scrolling
7. **User testing** on various Windows versions (10, 11)

---

## Additional Resources

- **Slint Documentation**: https://slint.dev/docs
- **Tokio Runtime Guide**: https://tokio.rs/tokio/tutorial
- **Tungstenite WebSocket**: https://docs.rs/tungstenite/latest/tungstenite/
- **Windows Crate**: https://docs.rs/windows/latest/windows/
- **Rust Best Practices**: https://doc.rust-lang.org/book/

