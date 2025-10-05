// menu.rs (refactored with Python management and theme support)
use std::io::{self, Write};
use crate::projects;
use crate::analyzer;
use crate::file_manager::FileManager;
use crate::project_creator::ProjectCreator;
use crate::build_system::BuildSystem; 
use crate::self_update::SelfUpdater;
use crate::git_manager::GitManager;
use crate::ansi_theme::AnsiTheme;
pub use editor::open_file_in_editor;

// Import the new menu modules
mod editor;
mod menu_build_run;
mod menu_project_creator;
mod menu_file_management;
mod menu_cargo_management;
mod menu_git_management;
mod menu_python_management;

// Re-export the menu functions
pub use menu_build_run::build_run_menu;
pub use menu_project_creator::create_project_menu;
pub use menu_file_management::file_management_menu;
pub use menu_cargo_management::cargo_management_menu;
pub use menu_git_management::git_management_menu;
pub use menu_python_management::python_management_menu;

pub fn main_menu_with_config(config: crate::config::AppConfig) {
    let file_manager = FileManager::new_with_config(config.clone());
    let project_creator = ProjectCreator::new_with_config(config.clone()); 
    let build_system = BuildSystem::new(
        Some(config.log_file.clone()), 
        config.verbose_logging
    );
    let git_manager = GitManager::new_with_config(config.clone());
    let python_manager = crate::python_manager::PythonManager::new_with_config(config.clone());
    let config_manager = crate::config::ConfigManager::new();
    let mut current_project: Option<String> = None;

    loop {
        AnsiTheme::print_themed("\n", &config.theme);
        AnsiTheme::print_cyan("Main Menu:", &config.theme);
        AnsiTheme::print_themed("\n", &config.theme);
        
        if let Some(project) = &current_project {
            AnsiTheme::print_themed("Current Project: ", &config.theme);
            AnsiTheme::print_cyan(project, &config.theme);
            AnsiTheme::print_themed("\n", &config.theme);
        }
        
        AnsiTheme::print_themed("Workspace: ", &config.theme);
        AnsiTheme::print_themed(&config.workspace_path, &config.theme);
        AnsiTheme::print_themed("\n", &config.theme);
        
        AnsiTheme::print_themed("1) Analyze Rust Projects (Rust Analyzer)\n", &config.theme);
        AnsiTheme::print_themed("2) List Rust Projects\n", &config.theme);
        AnsiTheme::print_themed("3) File Management\n", &config.theme);
        AnsiTheme::print_themed("4) Cargo.toml Management\n", &config.theme);
        AnsiTheme::print_themed("5) Create New Rust Project\n", &config.theme);
        AnsiTheme::print_themed("6) Build & Run Projects\n", &config.theme);
        AnsiTheme::print_themed("7) Self-Update Terminal\n", &config.theme);
        AnsiTheme::print_themed("8) Git Management\n", &config.theme);
        AnsiTheme::print_themed("9) Version Management\n", &config.theme);
        AnsiTheme::print_themed("10) Select Current Project\n", &config.theme);
        AnsiTheme::print_themed("11) Open Build Log File\n", &config.theme);
        AnsiTheme::print_themed("12) Python Management\n", &config.theme);
        AnsiTheme::print_themed("13) Configuration Settings\n", &config.theme);
        AnsiTheme::print_themed("14) Open File in Text Editor\n", &config.theme);
        AnsiTheme::print_themed("Q) Quit\n", &config.theme);

        AnsiTheme::print_themed("Enter choice: ", &config.theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                if let Some(project) = projects::select_project_with_config(&config) {
                    current_project = Some(project.clone());
                    analyzer::analyze_project(&project, &config.theme);
                }
            }
            "2" => projects::list_projects_with_config(&config),
            "3" => file_management_menu(&file_manager, &current_project, &config.theme),
            "4" => cargo_management_menu(&file_manager, &current_project, &config.theme),
            "5" => create_project_menu(&project_creator, &config.theme),
            "6" => {
                let project = match &current_project {
                    Some(p) => p.clone(),
                    None => match projects::select_project_with_config(&config) {
                        Some(p) => p,
                        None => continue,
                    },
                };
                build_run_menu(&build_system, &project, &config.theme);
            }
            "7" => {
                let updater = SelfUpdater::new();
                updater.check_for_updates(&config.theme);
            }
            "8" => git_management_menu(&git_manager, &current_project, &config.theme),
            "9" => {
                if let Some(project) = &current_project {
                    if let Err(e) = file_manager.version_management_menu(project, &config.theme) {
                        AnsiTheme::print_error(&format!("Error: {}\n", e), &config.theme);
                    }
                } else {
                    AnsiTheme::print_error(" No project selected.\n", &config.theme);
                }
            }
            "10" => {
                if let Some(project) = projects::select_project_with_config(&config) {
                    current_project = Some(project);
                }
            }
            "11" => {
                let build_system = BuildSystem::new(
                    Some(config.log_file.clone()), 
                    config.verbose_logging
                );
                 match build_system.open_log_file(&config.theme) {
                    Ok(()) => AnsiTheme::print_success(" Log file opened\n", &config.theme),
                    Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), &config.theme),
                }
            }
            "12" => python_management_menu(&python_manager, &current_project, &config.theme),
            "13" => {
                config_menu(&config_manager, &config.theme);
            }
            "14" => {
	        if let Some(project) = &current_project {
    	            let file_manager = FileManager::new_with_config(config.clone());
	            if let Ok(Some(file_path)) = file_manager.select_editor_file_from_list(project) {
	                let full_path = config.get_project_path(project).join(file_path);
	                if let Err(e) = editor::open_file_in_editor(full_path, config.theme.clone()) {
	                    AnsiTheme::print_error(&format!("Editor error: {}\n", e), &config.theme);
	                }
 	            }
	        } else {
	            AnsiTheme::print_error(" No project selected.\n", &config.theme);
	        }
	    }
            "Q" | "q" => {
                AnsiTheme::print_themed("Goodbye!\n", &config.theme);
                break;
            }
            _ => AnsiTheme::print_themed("Invalid choice.\n", &config.theme),
        }
    }
}

// Updated config menu with theme support
fn config_menu(config_manager: &crate::config::ConfigManager, theme: &crate::config::ThemeConfig) {
    use std::io::{self, Write};
    
    loop {
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_cyan("Configuration Settings", theme);
        AnsiTheme::print_themed("\n", theme);
        
        config_manager.show_config(theme);
        AnsiTheme::print_themed("\n", theme);
        
        AnsiTheme::print_themed("1) Edit Workspace Path\n", theme);
        AnsiTheme::print_themed("2) Show Current Configuration\n", theme);
        AnsiTheme::print_themed("3) Open Config File in Editor\n", theme);
        AnsiTheme::print_themed("4) Change Theme Colors\n", theme);
        AnsiTheme::print_themed("B) Back to Main Menu\n", theme);
        
        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();
        
        match choice {
            "1" => {
                let mut config_manager = crate::config::ConfigManager::new();
                if let Err(e) = config_manager.edit_config_interactive(theme) {
                    AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
                }
            }
            "2" => {
                config_manager.show_config(theme);
            }
            "3" => {
                // Open config file in default editor
                let config_path = crate::config::ConfigManager::get_config_path();
                if config_path.exists() {
                    #[cfg(target_os = "windows")]
                    let _ = std::process::Command::new("notepad.exe")
                        .arg(&config_path)
                        .status();
                    
                    #[cfg(target_family = "unix")]
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&config_path)
                        .status();
                    
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open")
                        .arg(&config_path)
                        .status();
                } else {
                    AnsiTheme::print_warning(" Config file not found\n", theme);
                }
            }
            "4" => {
                change_theme_menu(&mut crate::config::ConfigManager::new(), theme);
            }
            "B" | "b" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

fn change_theme_menu(config_manager: &mut crate::config::ConfigManager, theme: &crate::config::ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Theme Configuration", theme);
    AnsiTheme::print_themed("\n", theme);
    
    AnsiTheme::print_themed("Available colors: default, black, white, red, green, blue, yellow, magenta, cyan\n", theme);
    
    // Background color
    AnsiTheme::print_themed("Enter background color [default]: ", theme);
    io::stdout().flush().unwrap();
    let mut bg_input = String::new();
    io::stdin().read_line(&mut bg_input).unwrap();
    let bg_color = parse_color_input(&bg_input);
    
    // Foreground color  
    AnsiTheme::print_themed("Enter foreground color [default]: ", theme);
    io::stdout().flush().unwrap();
    let mut fg_input = String::new();
    io::stdin().read_line(&mut fg_input).unwrap();
    let fg_color = parse_color_input(&fg_input);

    // Logo color
    AnsiTheme::print_themed("Enter logo color [red]: ", theme);
    io::stdout().flush().unwrap();
    let mut logo_input = String::new();
    io::stdin().read_line(&mut logo_input).unwrap();
    let logo_color = if logo_input.trim().is_empty() {
        crate::config::TerminalColor::Red // Default logo color
    } else {
        parse_color_input(&logo_input)
    };
    
    if let Err(e) = config_manager.update_config(|config| {
        config.theme.background_color = bg_color.clone();
        config.theme.foreground_color = fg_color.clone();
        config.theme.logo_color = logo_color.clone();
    }) {
        AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
        return;
    }
    
    // Apply changes immediately
    crate::config::apply_theme(&config_manager.get_config().theme);
    AnsiTheme::print_success(" Theme updated and applied!\n", theme);
    AnsiTheme::print_themed(&format!("Background: {:?}, Foreground: {:?}, Logo: {:?}\n", bg_color, fg_color, logo_color), theme);
}

fn parse_color_input(input: &str) -> crate::config::TerminalColor {
    match input.trim().to_lowercase().as_str() {
        "" => crate::config::TerminalColor::Default,
        "black" => crate::config::TerminalColor::Black,
        "white" => crate::config::TerminalColor::White,
        "red" => crate::config::TerminalColor::Red,
        "green" => crate::config::TerminalColor::Green, 
        "blue" => crate::config::TerminalColor::Blue,
        "yellow" => crate::config::TerminalColor::Yellow,
        "magenta" => crate::config::TerminalColor::Magenta,
        "cyan" => crate::config::TerminalColor::Cyan,
        _ => {
            AnsiTheme::print_warning(&format!(" Unknown color '{}', using default\n", input.trim()), &crate::config::ThemeConfig::default());
            crate::config::TerminalColor::Default
        }
    }
}

pub fn main_menu() {
    let config = crate::config::AppConfig::default();
    main_menu_with_config(config);
}