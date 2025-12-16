//! Login screen UI and logic

use crate::services::{HttpClient, SessionManager};
use crate::ui::LoginScreenComponent;
use slint::ComponentHandle;
use std::sync::Arc;

/// Login screen controller
#[allow(dead_code)]
pub struct LoginScreen {
    ui: LoginScreenComponent,
    http_client: Arc<HttpClient>,
    session_manager: Arc<SessionManager>,
}

impl LoginScreen {
    pub fn new(
        base_url: String,
        on_login_success: Box<dyn Fn(String) + Send + Sync>,
        on_navigate_to_signup: Box<dyn Fn() + Send + Sync>,
    ) -> Self {
        let ui = LoginScreenComponent::new().unwrap();
        let http_client = Arc::new(HttpClient::new(base_url));
        let session_manager = Arc::new(SessionManager::new());

        let ui_weak = ui.as_weak();
        let client = http_client.clone();
        let session_mgr = session_manager.clone();
        let callback = Arc::new(on_login_success);
        let signup_callback = Arc::new(on_navigate_to_signup);

        ui.on_login(move || {
            let ui_handle = ui_weak.unwrap();
            let username = ui_handle.get_username().to_string();
            let password = ui_handle.get_password().to_string();

            // Validate inputs
            if username.is_empty() {
                ui_handle.set_error_message("Username cannot be empty".into());
                return;
            }

            if password.is_empty() {
                ui_handle.set_error_message("Password cannot be empty".into());
                return;
            }

            // Clear previous error
            ui_handle.set_error_message("".into());
            ui_handle.set_is_loading(true);

            // Call backend login endpoint in background thread
            let ui_weak_inner = ui_weak.clone();
            let http_client = client.clone();
            let session_manager = session_mgr.clone();
            let success_cb = callback.clone();

            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                match runtime.block_on(http_client.login(username.clone(), password.clone())) {
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

                        let user_id = response.user_id.clone();

                        // Success! Update UI from event loop
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_loading(false);
                                ui.set_error_message("".into());
                                ui.hide().unwrap(); // Hide login screen
                                success_cb(user_id); // Trigger callback
                            }
                        })
                        .ok();
                    }
                    Err(e) => {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_loading(false);
                                ui.set_error_message(e.into());
                            }
                        })
                        .ok();
                    }
                }
            });
        });

        let ui_weak_signup = ui.as_weak();
        ui.on_navigate_to_signup(move || {
            if let Some(ui) = ui_weak_signup.upgrade() {
                ui.hide().unwrap();
            }
            signup_callback();
        });

        Self {
            ui,
            http_client,
            session_manager,
        }
    }

    pub fn show(&self) {
        self.ui.show().unwrap();
    }
}
