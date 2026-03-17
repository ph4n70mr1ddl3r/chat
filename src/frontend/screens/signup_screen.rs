//! Signup screen UI and logic

use crate::services::HttpClient;
use crate::services::SessionManager;
use crate::ui::SignupScreenComponent;
use slint::ComponentHandle;
use std::sync::Arc;

fn validate_password_strength(password: &str) -> Result<(), String> {
    let len = password.len();

    if len < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    if len > 128 {
        return Err("Password must be at most 128 characters".to_string());
    }

    if !password.chars().any(char::is_uppercase) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !password.chars().any(char::is_lowercase) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !password.chars().any(char::is_numeric) {
        return Err("Password must contain at least one digit".to_string());
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

/// Signup screen controller
pub struct SignupScreen {
    ui: SignupScreenComponent,
}

impl SignupScreen {
    pub fn new(
        base_url: String,
        on_signup_success: Box<dyn Fn(String) + Send + Sync>,
        on_navigate_to_login: Box<dyn Fn() + Send + Sync>,
    ) -> Self {
        let ui = SignupScreenComponent::new().expect("Failed to create signup screen UI");
        let http_client = Arc::new(HttpClient::new(base_url));
        let session_manager = Arc::new(SessionManager::new());

        let ui_weak = ui.as_weak();
        let client = http_client.clone();
        let session_mgr = session_manager.clone();
        let success_callback = Arc::new(on_signup_success);
        ui.on_signup(move || {
            tracing::debug!("Signup button clicked");
            let ui_handle = if let Some(ui) = ui_weak.upgrade() {
                ui
            } else {
                tracing::warn!("UI weak reference failed to upgrade");
                return;
            };
            let username = ui_handle.get_username().to_string();
            let password = ui_handle.get_password().to_string();
            let confirm_password = ui_handle.get_confirm_password().to_string();
            tracing::debug!("Got form values - username: {}", username);

            // Validate inputs
            if username.is_empty() {
                tracing::debug!("Username is empty");
                ui_handle.set_error_message("Username cannot be empty".into());
                return;
            }

            if password != confirm_password {
                tracing::debug!("Passwords don't match");
                ui_handle.set_error_message("Passwords do not match".into());
                return;
            }

            if let Err(e) = validate_password_strength(&password) {
                tracing::debug!("Password validation failed: {}", e);
                ui_handle.set_error_message(e.into());
                return;
            }

            tracing::debug!("Validation passed, spawning signup thread");
            // Clear previous error
            ui_handle.set_error_message("".into());

            // Call backend signup endpoint in background thread
            let ui_weak_inner = ui_weak.clone();
            let http_client = client.clone();
            let session_manager = session_mgr.clone();
            let success_cb = success_callback.clone();
            std::thread::spawn(move || {
                tracing::debug!("Signup thread started");
                let runtime = match tokio::runtime::Runtime::new() {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        tracing::error!("Failed to create async runtime: {}", e);
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_error_message("Failed to initialize network".into());
                            }
                        })
                        .ok();
                        return;
                    }
                };
                tracing::debug!("Calling signup API for user: {}", username);
                match runtime.block_on(http_client.signup(username.clone(), password.clone())) {
                    Ok(response) => {
                        tracing::info!("Signup successful for user: {}", response.username);

                        // Save session to disk
                        if let Err(e) = session_manager.save_session_sync(
                            &response.user_id,
                            &response.token,
                            &response.username,
                            response.expires_in as i64,
                            &response.csrf_token,
                        ) {
                            tracing::error!("Failed to save session: {}", e);
                        }

                        let user_id = response.user_id.clone();

                        // Success! Navigate to chat screen
                        tracing::info!("Navigating to chat screen for user: {}", user_id);
                        slint::invoke_from_event_loop(move || {
                            success_cb(user_id);
                            // Note: Don't hide the window here - show_chat will clean up
                        })
                        .ok();
                    }
                    Err(e) => {
                        tracing::error!("Signup failed: {}", e);
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

        let login_callback = Arc::new(on_navigate_to_login);
        ui.on_navigate_to_login(move || {
            tracing::debug!("Navigate to login clicked");
            login_callback();
            // Note: Don't hide signup window here - show_login will clean up
        });

        Self { ui }
    }

    pub fn show(&self) {
        self.ui.show().expect("Failed to show signup screen");
    }
}
