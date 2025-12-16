//! Settings screen logic

use crate::ui::SettingsScreenComponent;
use slint::ComponentHandle;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct SettingsScreen {
    ui: SettingsScreenComponent,
    runtime: Arc<Runtime>,
}

impl SettingsScreen {
    pub fn new(
        username: String,
        on_back: Box<dyn Fn() + Send + Sync>,
        on_account_deleted: Box<dyn Fn() + Send + Sync>,
    ) -> Self {
        let ui = SettingsScreenComponent::new().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());

        ui.set_username(username.into());

        let ui_weak = ui.as_weak();
        let back_cb = Arc::new(on_back);
        ui.on_back(move || {
            back_cb();
        });

        let ui_weak = ui.as_weak();
        let runtime_clone = runtime.clone();
        ui.on_change_password(move || {
            let ui = ui_weak.unwrap();
            let current = ui.get_current_password().to_string();
            let new = ui.get_new_password().to_string();

            ui.set_is_loading(true);
            ui.set_error_message("".into());
            ui.set_success_message("".into());

            let ui_weak_inner = ui_weak.clone();
            runtime_clone.spawn(async move {
                match api_change_password(&current, &new).await {
                    Ok(_) => {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_loading(false);
                                ui.set_success_message("Password changed successfully".into());
                                ui.set_current_password("".into());
                                ui.set_new_password("".into());
                            }
                        })
                        .ok();
                    }
                    Err(e) => {
                        let err_msg = format!("Failed: {}", e);
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_loading(false);
                                ui.set_error_message(err_msg.into());
                            }
                        })
                        .ok();
                    }
                }
            });
        });

        let ui_weak = ui.as_weak();
        let runtime_clone = runtime.clone();
        let deleted_cb = Arc::new(on_account_deleted);
        ui.on_delete_account(move || {
            let ui = ui_weak.unwrap();
            let password = ui.get_delete_password().to_string();

            ui.set_is_loading(true);
            ui.set_error_message("".into());

            let ui_weak_inner = ui_weak.clone();
            let deleted_cb_inner = deleted_cb.clone();

            runtime_clone.spawn(async move {
                match api_delete_account(&password).await {
                    Ok(_) => {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_loading(false);
                                ui.hide().unwrap();
                                deleted_cb_inner();
                            }
                        })
                        .ok();
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to delete account: {}", e);
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_loading(false);
                                ui.set_error_message(err_msg.into());
                            }
                        })
                        .ok();
                    }
                }
            });
        });

        Self { ui, runtime }
    }

    pub fn show(&self) {
        self.ui.show().unwrap();
    }
}

async fn api_change_password(current: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let token = crate::services::session::get_token().ok_or("No token")?;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/user/change-password", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "current_password": current,
            "new_password": new
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Error: {}", res.status()).into());
    }
    Ok(())
}

async fn api_delete_account(password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let token = crate::services::session::get_token().ok_or("No token")?;

    let client = reqwest::Client::new();
    let res = client
        .delete(format!("{}/user/me", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "password": password
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Error: {}", res.status()).into());
    }

    // Also clear session locally
    let _ = crate::services::session::get_session_manager()
        .clear_session()
        .await;

    Ok(())
}
