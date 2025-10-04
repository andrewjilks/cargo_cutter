// menu_project_creator.rs
use std::io::{self, Write};
use crate::project_creator::ProjectCreator;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub fn create_project_menu(project_creator: &ProjectCreator, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_green("Create New Rust Project\n", theme);
    
    AnsiTheme::print_themed("Select project type:\n", theme);
    AnsiTheme::print_themed("1) Basic Binary Project\n", theme);
    AnsiTheme::print_themed("2) Library Project\n", theme);
    AnsiTheme::print_themed("3) CLI Application\n", theme);
    AnsiTheme::print_themed("B) Back to main menu\n", theme);

    AnsiTheme::print_themed("Enter choice: ", theme);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    let project_type = match choice {
        "1" => "basic",
        "2" => "library", 
        "3" => "cli",
        "B" | "b" => return,
        _ => {
            AnsiTheme::print_themed("Invalid choice.\n", theme);
            return;
        }
    };

    AnsiTheme::print_themed("Enter project name: ", theme);
    io::stdout().flush().unwrap();

    let mut name_input = String::new();
    io::stdin().read_line(&mut name_input).unwrap();
    let project_name = name_input.trim();

    if project_name.is_empty() {
        AnsiTheme::print_themed("Project name cannot be empty.\n", theme);
        return;
    }

    if !project_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        AnsiTheme::print_themed("Project name can only contain alphanumeric characters, underscores, and hyphens.\n", theme);
        return;
    }

    match project_creator.create_project_with_template(project_name, project_type, theme) {
        Ok(()) => {
            AnsiTheme::print_success(" Project created successfully!\n", theme);
            
            AnsiTheme::print_themed("Open project in file manager? (y/N): ", theme);
            io::stdout().flush().unwrap();
            
            let mut open_choice = String::new();
            io::stdin().read_line(&mut open_choice).unwrap();
            
            if open_choice.trim().eq_ignore_ascii_case("y") {
                AnsiTheme::print_themed("Use 'File Management' menu to work with your new project.\n", theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}