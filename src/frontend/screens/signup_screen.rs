//! Signup screen UI and logic

slint::include_modules!();

/// Signup screen controller
pub struct SignupScreen {
    ui: SignupScreen,
}

impl SignupScreen {
    pub fn new() -> Self {
        let ui = SignupScreen::new().unwrap();
        
        let ui_weak = ui.as_weak();
        ui.on_signup(move || {
            let ui = ui_weak.unwrap();
            let username = ui.get_username();
            let password = ui.get_password();
            let confirm_password = ui.get_confirm_password();
            
            // Validate inputs
            if password != confirm_password {
                ui.set_error_message("Passwords do not match".into());
                return;
            }
            
            // Clear previous error
            ui.set_error_message("".into());
            
            // TODO: Call backend signup endpoint
            ui.set_error_message("Signup not implemented yet".into());
        });
        
        let ui_weak = ui.as_weak();
        ui.on_navigate_to_login(move || {
            // TODO: Navigate to login screen
            println!("Navigate to login");
        });
        
        Self { ui }
    }
    
    pub fn show(&self) {
        self.ui.run().unwrap();
    }
}