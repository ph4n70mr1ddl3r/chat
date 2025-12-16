//! User search screen logic

use crate::ui::{UserSearchResult, UserSearchScreenComponent};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct UserSearchScreen {
    ui: UserSearchScreenComponent,
    runtime: Arc<Runtime>,
    _debounce_timer: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl UserSearchScreen {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ui = UserSearchScreenComponent::new()?;
        let runtime = Arc::new(Runtime::new()?);
        let debounce_timer = Arc::new(Mutex::new(None::<tokio::task::JoinHandle<()>>));

        let ui_weak = ui.as_weak();
        let runtime_clone = runtime.clone();
        let debounce_clone = debounce_timer.clone();

        ui.on_search_changed(move |query| {
            let ui_weak = ui_weak.clone();
            let runtime = runtime_clone.clone();
            let debounce = debounce_clone.clone();
            let query = query.to_string();

            // Cancel previous timer
            if let Some(handle) = debounce.lock().unwrap().take() {
                handle.abort();
            }

            // Set loading state immediately if query is not empty
            if !query.is_empty() {
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_is_searching(true);
                        ui.set_error_message("".into());
                    }
                })
                .ok();
            } else {
                // Clear results if empty
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_is_searching(false);
                        ui.set_search_results(ModelRc::from(Rc::new(VecModel::from(vec![]))));
                    }
                })
                .ok();
                return;
            }

            // Start new timer
            let ui_weak_inner = ui_weak.clone();
            let handle = runtime.spawn(async move {
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Perform search
                match search_users_api(&query).await {
                    Ok(results) => {
                        let slint_results: Vec<UserSearchResult> = results
                            .into_iter()
                            .map(|r| UserSearchResult {
                                user_id: r.user_id.into(),
                                username: r.username.into(),
                                is_online: r.is_online,
                            })
                            .collect();

                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_searching(false);
                                let model = Rc::new(VecModel::from(slint_results));
                                ui.set_search_results(ModelRc::from(model));
                            }
                        })
                        .ok();
                    }
                    Err(e) => {
                        let err_msg = format!("Search failed: {}", e);
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_is_searching(false);
                                ui.set_error_message(err_msg.into());
                            }
                        })
                        .ok();
                    }
                }
            });

            *debounce.lock().unwrap() = Some(handle);
        });

        // Navigation callbacks (stubs to be wired by caller if needed, or exposed)
        // Currently UserSearchScreen logic is self-contained except navigation
        // But main.rs doesn't wire UserSearchScreen yet?
        // Wait, UserSearchScreen is modal? Or separate screen?
        // In ChatScreen, we have `search_users` callback.
        // It should show UserSearchScreen.
        // But `main.rs` doesn't know about `UserSearchScreen` (I didn't add it to AppState).
        // I should add it to AppState in main.rs if I want to use it.
        // But for now, let's fix compilation.

        Ok(Self {
            ui,
            runtime,
            _debounce_timer: debounce_timer,
        })
    }

    pub fn show(&self) {
        self.ui.show().unwrap();
    }

    pub fn as_weak(&self) -> slint::Weak<UserSearchScreenComponent> {
        self.ui.as_weak()
    }
}

async fn search_users_api(query: &str) -> Result<Vec<ApiUserResult>, Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let token = crate::services::session::get_token().ok_or("No token")?;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/users/search", base_url))
        .query(&[("q", query), ("limit", "20")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Error: {}", res.status()).into());
    }

    let results: Vec<ApiUserResult> = res.json().await?;
    Ok(results)
}

#[derive(serde::Deserialize)]
struct ApiUserResult {
    user_id: String,
    username: String,
    is_online: bool,
}
