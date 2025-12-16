//! Chat application frontend GUI
//!
//! This is the main entry point for the desktop chat GUI built with Slint.

mod screens;
mod services;
pub mod ui; // Public so screens can use it

use screens::chat_screen::ChatScreen;
use screens::login_screen::LoginScreen;
use screens::settings_screen::SettingsScreen;
use services::SessionManager;
use std::cell::RefCell;

struct AppState {
    login_screen: Option<LoginScreen>,
    chat_screen: Option<ChatScreen>,
    settings_screen: Option<SettingsScreen>,
}

thread_local! {
    static APP_STATE: RefCell<AppState> = const {
        RefCell::new(AppState {
            login_screen: None,
            chat_screen: None,
            settings_screen: None,
        })
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting chat GUI");

    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let session_manager = SessionManager::new();

    // Check for existing session
    match session_manager.get_session() {
        Ok(Some(session)) => {
            tracing::info!("Found existing session for user: {}", session.username);

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            if now >= session.expires_at {
                tracing::warn!("Session expired, showing login screen");
                show_login(base_url);
            } else {
                tracing::info!("Valid session found, showing chat screen");
                show_chat(session.user_id, base_url);
            }
        }
        _ => {
            tracing::info!("No existing session, showing login screen");
            show_login(base_url);
        }
    }

    slint::run_event_loop()?;
    Ok(())
}

fn show_login(base_url: String) {
    let url_for_chat = base_url.clone();

    let login = LoginScreen::new(
        base_url,
        Box::new(move |user_id| {
            show_chat(user_id, url_for_chat.clone());
        }),
    );

    login.show();

    APP_STATE.with(|state| {
        let mut state_ref = state.borrow_mut();
        state_ref.chat_screen = None;
        state_ref.settings_screen = None;
        state_ref.login_screen = Some(login);
    });
}

fn show_chat(user_id: String, base_url: String) {
    let base_url_logout = base_url.clone();
    let base_url_settings = base_url.clone();
    let user_id_settings = user_id.clone();

    match ChatScreen::new(
        user_id,
        // On Logout
        Box::new(move || {
            show_login(base_url_logout.clone());
        }),
        // On Settings
        Box::new(move || {
            show_settings(user_id_settings.clone(), base_url_settings.clone());
        }),
    ) {
        Ok(chat) => {
            chat.show();
            APP_STATE.with(|state| {
                let mut state_ref = state.borrow_mut();
                state_ref.login_screen = None;
                state_ref.settings_screen = None;
                state_ref.chat_screen = Some(chat);
            });
        }
        Err(e) => {
            tracing::error!("Failed to initialize chat screen: {}", e);
            show_login(base_url);
        }
    }
}

fn show_settings(user_id: String, base_url: String) {
    let base_url_back = base_url.clone();
    let base_url_deleted = base_url.clone();
    let user_id_back = user_id.clone();

    // We need username for settings. Session has it.
    let username = crate::services::session::get_session_manager()
        .get_current_session()
        .map(|s| s.username)
        .unwrap_or_else(|| "User".to_string());

    let settings = SettingsScreen::new(
        username,
        // On Back
        Box::new(move || {
            show_chat(user_id_back.clone(), base_url_back.clone());
        }),
        // On Account Deleted (Logout)
        Box::new(move || {
            show_login(base_url_deleted.clone());
        }),
    );

    settings.show();

    APP_STATE.with(|state| {
        let mut state_ref = state.borrow_mut();
        state_ref.chat_screen = None;
        state_ref.login_screen = None;
        state_ref.settings_screen = Some(settings);
    });
}
