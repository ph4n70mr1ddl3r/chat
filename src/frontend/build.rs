fn main() {
    // Compile Slint UI files
    let slint_files = vec![
        "screens/login_screen.slint",
        "screens/signup_screen.slint",
        "screens/user_search_screen.slint",
        "screens/chat_screen.slint",
    ];
    
    for file in slint_files {
        slint_build::compile_with_config(
            file,
            slint_build::CompilerConfiguration::new()
                .with_include_paths(vec!["screens".into()]),
        )
        .unwrap_or_else(|_| panic!("Slint compilation failed for {}", file));
    }
}
