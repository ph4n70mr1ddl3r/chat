//! Signup screen UI and logic

use crate::services::HttpClient;
use crate::services::SessionManager;
use crate::ui::SignupScreenComponent;
use slint::ComponentHandle;
use std::sync::Arc;

/// Signup screen controller
pub struct SignupScreen {
    ui: SignupScreenComponent,
    http_client: Arc<HttpClient>,
    session_manager: Arc<SessionManager>,
}

impl SignupScreen {
    pub fn new(base_url: String) -> Self {
        let ui = SignupScreenComponent::new().unwrap();
        let http_client = Arc::new(HttpClient::new(base_url));
        let session_manager = Arc::new(SessionManager::new());

        let ui_weak = ui.as_weak();
        let client = http_client.clone();
        let session_mgr = session_manager.clone();
        ui.on_signup(move || {
            let ui_handle = ui_weak.unwrap();
            let username = ui_handle.get_username().to_string();
            let password = ui_handle.get_password().to_string();
            let confirm_password = ui_handle.get_confirm_password().to_string();

            // Validate inputs
            if password != confirm_password {
                ui_handle.set_error_message("Passwords do not match".into());
                return;
            }

            if username.is_empty() {
                ui_handle.set_error_message("Username cannot be empty".into());
                return;
            }

            if password.len() < 8 {
                ui_handle.set_error_message("Password must be at least 8 characters".into());
                return;
            }

            // Clear previous error
            ui_handle.set_error_message("".into());

            // Call backend signup endpoint in background thread
            let ui_weak_inner = ui_weak.clone();
            let http_client = client.clone();
            let session_manager = session_mgr.clone();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                match runtime.block_on(http_client.signup(username.clone(), password.clone())) {
                    Ok(response) => {
                        // Save session to disk
                        if let Err(e) = session_manager.save_session_sync(
                            &response.user_id,
                            &response.token,
                            &response.username,
                            response.expires_in as i64,
                        ) {
                            eprintln!("Failed to save session: {}", e);
                        }

                        // Success! Update UI from event loop
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_error_message(
                                    format!("Account created! Welcome, {}", response.username)
                                        .into(),
                                );
                                // TODO: Navigate to chat screen after successful signup
                            }
                        })
                        .ok();
                    }
                    Err(e) => {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_error_message(e.into());
                            }
                        })
                        .ok();
                    }
                }
            });
        });

        let ui_weak = ui.as_weak();
        ui.on_navigate_to_login(move || {
            // TODO: Navigate to login screen
            println!("Navigate to login");
        });

        Self {
            ui,
            http_client,
            session_manager,
        }
    }

    pub fn show(&self) {
        self.ui.run().unwrap();
    }
}
