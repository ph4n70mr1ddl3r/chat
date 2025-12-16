# Rust + Slint Desktop Chat - Quick Reference & Patterns

**Date:** December 15, 2025  
**Status:** Comprehensive Research Complete

---

## Overview

This is a quick-reference companion to the comprehensive architecture guides. For deep dives, see:
- **[DESKTOP_CHAT_ARCHITECTURE.md](./DESKTOP_CHAT_ARCHITECTURE.md)** - Full implementation guide for all 5 topics
- **[RUST_REALTIME_CHAT_GUIDE.md](./RUST_REALTIME_CHAT_GUIDE.md)** - Backend architecture patterns

---

## Quick Decision Matrix

### 1. Slint Component Architecture
| Decision | Pattern | When to Use |
|----------|---------|------------|
| **Hierarchical Composition** | Root → Sections → Components | All apps (recommended default) |
| **Flat Components** | No nesting | Simple apps only |
| **Custom Components** | Reusable, self-contained | Recurring UI patterns |

**Recommendation**: Hierarchical with atomic components

```slint
export component AppWindow {
    in property <[MessageData]> messages;
    callback send-message(string);
    
    VerticalLayout {
        TopBar { }
        SplitLayout {
            Sidebar { }
            ChatPanel { }
        }
    }
}
```

---

### 2. WebSocket + Async Integration
| Approach | Pros | Cons | Best For |
|----------|------|------|----------|
| **Tokio MPSC + Broadcast** | Type-safe, clean | More boilerplate | Production apps |
| **Async Channels** | Simple | Less control | Simple apps |
| **Raw Futures** | Maximum control | Complex | Low-level optimization |

**Recommendation**: Tokio MPSC + Broadcast for all sizes

```rust
// Command channel (UI → Network)
let (tx, mut rx) = mpsc::channel(100);

// Event broadcast (Network → UI)
let (event_tx, event_rx) = broadcast::channel(1000);

// In UI callback
ui.on_send_message(move |msg| {
    tokio::spawn({
        let tx = tx.clone();
        async move {
            tx.send(ClientCommand::SendMessage(msg.to_string())).await.ok();
        }
    });
});
```

---

### 3. State Management Architecture
| Layer | Storage | Update Pattern | Example |
|-------|---------|-----------------|---------|
| **Persistent** | Disk/DB | Sync to disk | User profile, settings |
| **Application** | Arc<RwLock<>> + Globals | Through handlers | Current user, auth token |
| **UI** | Component properties | Reactive bindings | Sidebar expanded, search text |
| **Transient** | Channels | Events | Typing indicators, presence |

**Pattern**: Layered state with clear boundaries

```rust
// Persistent layer
let user_data = load_from_disk("user.json")?;

// Application layer
let app_state = Arc::new(RwLock::new(AppData {
    user: user_data,
    conversations: Vec::new(),
}));

// UI layer (set via Slint)
ui.set_current_user(user_data.name.into());
ui.set_conversations(conv_data.into());
```

---

### 4. Message List Virtualization
| Strategy | Memory | Scroll FPS | Setup Complexity |
|----------|--------|-----------|-------------------|
| **Slint ListView** | O(visible items) | 60 FPS | Minimal |
| **Custom Virtual List** | O(visible + buffer) | 60 FPS+ | Complex |
| **All in Memory** | O(total items) | Variable | Simple |

**Recommendation**: Use Slint ListView (automatic virtualization)

```slint
// Slint handles virtualization automatically
ListView {
    for message in messages: MessageBubble {
        text: message.text;
    }
}
```

---

### 5. Windows Integration
| Feature | Approach | Complexity | Priority |
|---------|----------|-----------|----------|
| **System Tray** | tray-icon crate | Low | High |
| **Notifications** | notify-rust | Low | Medium |
| **Hotkeys** | hotkey-rs | Low | Optional |
| **Window Management** | windows-rs | Medium | Medium |

**Recommendation**: Implement in this order

```rust
// 1. System Tray (easiest)
let tray = TrayIconBuilder::new().build()?;

// 2. Notifications
Notification::new().summary("New message").show()?;

// 3. Hotkeys (if needed)
let hotkey = Hotkey::new(Modifiers::CTRL | Modifiers::ALT, Keys::C);
```

---

## Common Patterns

### Pattern 1: UI-to-Network Communication
```rust
// ✓ Correct pattern
ui.on_send_message(move |text| {
    let tx = command_tx.clone();
    tokio::spawn(async move {
        tx.send(ClientCommand::SendMessage(text.to_string())).await.ok();
    });
});
```

### Pattern 2: Network-to-UI Communication
```rust
// ✓ Correct pattern
tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        let ui = ui_weak.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                // Update UI safely
                ui.set_messages(new_messages);
            }
        }).ok();
    }
});
```

### Pattern 3: State Updates
```rust
// ✓ Correct pattern
let state = app_state.clone();
tokio::spawn(async move {
    let mut data = state.data.write().await;
    data.conversations.push(new_conv);
    drop(data); // Release lock before sync
    
    sync_to_ui(&ui, &state).await;
});
```

### Pattern 4: Error Handling
```rust
// ✓ Correct pattern
match ws.send_command(cmd) {
    Ok(_) => log::debug!("Command sent"),
    Err(e) => {
        log::error!("Failed to send: {}", e);
        let ui = ui_weak.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui.upgrade() {
                ui.set_error(format!("Error: {}", e).into());
            }
        }).ok();
    }
}
```

---

## Performance Quick Checklist

- [ ] UI updates use `invoke_from_event_loop()`
- [ ] No blocking operations in main thread
- [ ] WebSocket client spawned in separate task
- [ ] Message list uses Slint ListView (not VerticalLayout)
- [ ] Memory cache for avatars/images
- [ ] Debounce rapid events (typing, scroll)
- [ ] Use weak references to UI handle
- [ ] Connection pooling for WebSocket
- [ ] Async file I/O for uploads
- [ ] Batch message updates instead of individual

### Memory Targets
```
Idle:          < 150 MB
Active chat:   < 500 MB
Long running:  < 1 GB
```

### Latency Targets
```
Message send:       < 100 ms
UI response:        < 16 ms
Reconnect:          < 3 s
Typing indicator:   < 200 ms
```

---

## Build & Run

```bash
# Install dependencies
rustup update

# Development build
cargo build
SLINT_INSPECTOR=true cargo run  # With UI inspector

# Release build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy --all-targets
cargo fmt --check
```

---

## Debugging Tips

### Enable Logging
```rust
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();

log::debug!("User sent: {:?}", msg);
log::warn!("Reconnecting...");
log::error!("Failed: {}", err);
```

### Common Issues

| Problem | Cause | Solution |
|---------|-------|----------|
| UI freezes | Long operation on main thread | Move to async task |
| Weak ref upgrade fails | UI dropped | Add null check |
| Memory leak | Circular refs | Use Weak, not clone_strong |
| Queue full | Too many messages | Increase channel size or drop events |
| Jittery scroll | Heavy work while scrolling | Use debouncing |
| Reconnect loop | Wrong state handling | Check connection state machine |

---

## Project Structure

```
my-chat-app/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── network/
│   │   ├── mod.rs
│   │   ├── ws_client.rs     # WebSocket implementation
│   │   ├── message.rs       # Protocol messages
│   │   └── state.rs         # Connection state
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── bridge.rs        # UI ↔ Network bridge
│   │   ├── state_sync.rs    # State → UI sync
│   │   └── handlers.rs      # UI event handlers
│   ├── state/
│   │   ├── mod.rs
│   │   ├── app.rs           # Application state
│   │   ├── auth.rs          # Auth state
│   │   └── conversation.rs  # Conversation state
│   └── platform/
│       ├── mod.rs
│       └── windows/         # Windows-specific code
│           ├── mod.rs
│           ├── tray.rs
│           ├── notifications.rs
│           └── window.rs
├── ui/
│   ├── app.slint            # Main window
│   ├── components/
│   │   ├── message.slint
│   │   ├── sidebar.slint
│   │   └── input.slint
│   └── styles/
│       └── theme.slint
├── Cargo.toml
└── build.rs                 # Slint compilation
```

---

## Cargo.toml Template

```toml
[package]
name = "chat-app"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
slint = { version = "1.14", features = ["backend-winit"] }
tokio = { version = "1", features = ["full"] }
tungstenite = "0.28"
tokio-tungstenite = "0.28"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
lru = "0.12"
chrono = "0.4"
log = "0.4"
env_logger = "0.11"
anyhow = "1"
thiserror = "2"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation"
] }
notify-rust = "4.11"
tray-icon = "0.1"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"

[build-dependencies]
slint-build = "1.14"
```

---

## Testing Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = MessageData {
            id: "1".into(),
            text: "Hello".into(),
            author: "Alice".into(),
            timestamp: 0,
            is_own: false,
        };
        assert_eq!(msg.author, "Alice");
    }

    #[tokio::test]
    async fn test_connection_state() {
        let state = ConnectionState::Disconnected;
        assert!(matches!(state, ConnectionState::Disconnected));
    }
}
```

---

## Key Takeaways

1. **Use Slint's built-in ListView** - virtualization is automatic
2. **Separate async work** - keep main thread free for UI
3. **Weak references** - prevent reference cycles with UI
4. **Channel communication** - type-safe cross-thread messaging
5. **Windows APIs safely** - use `windows` crate properly
6. **Test early** - set up testing infrastructure from start

---

## Resources

- **Official Documentation**
  - Slint: https://slint.dev/docs
  - Tokio: https://tokio.rs/tokio/tutorial
  - Tungstenite: https://docs.rs/tungstenite/latest/
  
- **Examples**
  - Slint examples: https://github.com/slint-ui/slint/tree/master/examples
  - Tokio tutorials: https://tokio.rs
  
- **Community**
  - Rust Discord: https://discord.gg/rust-lang
  - Slint Chat: https://chat.slint.dev

---

## Next Steps

1. **Start with main app structure** - set up Cargo project and Slint integration
2. **Implement WebSocket client** - get baseline connection working
3. **Build basic UI** - message list and input area
4. **Add state management** - sync Rust ↔ Slint
5. **Polish UI** - styling, animations, responsive layout
6. **Add Windows features** - tray, notifications (optional but recommended)
7. **Performance testing** - load test with 5K+ messages
8. **User testing** - collect feedback and iterate

**Estimated Timeline**: 2-3 weeks for MVP, 6-8 weeks for production-ready

