// main.rs
mod menu;
mod projects;
mod analyzer;
mod file_manager;
mod project_creator;
mod build_system;
mod self_update;
mod git_manager;
mod config;
mod python_manager;
mod ansi_theme;  

fn main() {
    // ENABLE ANSI SUPPORT FIRST
    ansi_theme::AnsiTheme::enable_ansi_support();

    // Initialize configuration first
    let config_manager = config::ConfigManager::new();
    let config = config_manager.get_config().clone();

    // APPLY THEME AT STARTUP - This sets the persistent background and foreground
    ansi_theme::AnsiTheme::apply_theme(&config.theme);

    let current_dir = std::env::current_dir().unwrap();
    
    // Use theme-aware printing for ALL text
    ansi_theme::AnsiTheme::print_themed(&format!("Current directory: {}\n", current_dir.display()), &config.theme);
    
    // Style the entire "Workspace" line with theme colors
    ansi_theme::AnsiTheme::print_themed("Workspace: ", &config.theme);
    ansi_theme::AnsiTheme::print_themed(&config.workspace_path, &config.theme);
    ansi_theme::AnsiTheme::print_themed("\n", &config.theme);
    
    let logo = r#"
       ___ ___    _   ___   _____ ___ ___ __  __ ___ _  _   _   _ 
      / __| _ \  /_\ | _ ) |_   _| __| _ \  \/  |_ _| \| | /_\ | |
     | (__|   / / _ \| _ \   | | | _||   / |\/| || || .` |/ _ \| |__
      \___|_|_\/_/ \_\___/   |_| |___|_|_\_|  |_|___|_|\_/_/ \_\____| (NOW WITH THEMES)
    "#;

    // Use themed logo color
    ansi_theme::AnsiTheme::print_logo(logo, &config.theme);
    ansi_theme::AnsiTheme::print_themed("\n", &config.theme);
    
    // Usage blurb - all themed!
    ansi_theme::AnsiTheme::print_themed("Rust Dev Terminal\n", &config.theme);
    ansi_theme::AnsiTheme::print_themed(&format!("Version: {}\n", env!("CARGO_PKG_VERSION")), &config.theme);
    ansi_theme::AnsiTheme::print_themed("Author: Andrew Jilks\n", &config.theme);
    ansi_theme::AnsiTheme::print_themed("Description: Self-updating Program development environment\n", &config.theme);

    ansi_theme::AnsiTheme::print_themed("===============================\n", &config.theme);
    ansi_theme::AnsiTheme::print_themed("  Rust Dev Mode Ready!\n", &config.theme);
    ansi_theme::AnsiTheme::print_themed("===============================\n", &config.theme);

    // DEBUG/TEXT TEST SECTION - Now using theme properly
    //ansi_theme::AnsiTheme::print_themed("\n=== Debug/Text Test ===\n", &config.theme);
    
    // Test 1: Plain text using theme colors
    //ansi_theme::AnsiTheme::print_themed("Plain text: This should appear in your theme's foreground color\n", &config.theme);
    
    // Test 2: Colored text using theme background
    //ansi_theme::AnsiTheme::print_green("Green text: This should be green with theme background\n", &config.theme);
    //ansi_theme::AnsiTheme::print_blue("Blue text: This should be blue with theme background\n", &config.theme);
    //ansi_theme::AnsiTheme::print_red("Red text: This should be red with theme background\n", &config.theme);
    
    //ansi_theme::AnsiTheme::print_themed("=== End Debug/Text Test ===\n\n", &config.theme);

    // Start interactive menu loop
    menu::main_menu_with_config(config);

    // RESET TERMINAL COLORS ON EXIT
    config::reset_terminal_colors();
}