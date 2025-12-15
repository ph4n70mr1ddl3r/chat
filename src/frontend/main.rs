//! Chat application frontend GUI
//!
//! This is the main entry point for the desktop chat GUI built with Slint.

mod screens;
mod services;

use services::SessionManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting chat GUI");

    // Check for existing session
    let session_manager = SessionManager::new();
    
    match session_manager.get_session() {
        Ok(Some(session)) => {
            tracing::info!("Found existing session for user: {}", session.username);
            
            // Check if token needs refresh
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            
            if now >= session.expires_at {
                tracing::warn!("Session expired, showing login screen");
                show_login_screen();
            } else if now + 300 >= session.expires_at {
                // Token expires in < 5 minutes, should refresh
                tracing::info!("Token expiring soon, will need refresh");
                show_chat_screen();
            } else {
                tracing::info!("Valid session found, showing chat screen");
                show_chat_screen();
            }
        }
        Ok(None) => {
            tracing::info!("No existing session, showing login screen");
            show_login_screen();
        }
        Err(e) => {
            tracing::warn!("Failed to load session: {}, showing login screen", e);
            show_login_screen();
        }
    }

    Ok(())
}

fn show_login_screen() {
    // TODO: Show actual login screen
    println!("Would show login screen here");
}

fn show_chat_screen() {
    // TODO: Show actual chat screen
    println!("Would show chat screen here (auto-logged in)");
}
