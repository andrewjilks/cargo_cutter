// projects.rs
use std::fs;
use std::io::{self, Write};
use crate::config::AppConfig;
use crate::ansi_theme::AnsiTheme;

pub fn list_projects_with_config(config: &AppConfig) {
    let workspace = config.get_workspace_path();

    AnsiTheme::print_themed("\n", &config.theme);
    AnsiTheme::print_themed(&format!("Rust Projects in workspace ({}):\n", workspace.display()), &config.theme);

    if let Ok(entries) = fs::read_dir(&workspace) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if entry.path().join("Cargo.toml").exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        AnsiTheme::print_themed(&format!(" - {}\n", name), &config.theme);
                    }
                }
            }
        }
    } else {
        AnsiTheme::print_warning(&format!(" Workspace directory not found: {}\n", workspace.display()), &config.theme);
        AnsiTheme::print_themed("Update the workspace path in configuration settings.\n", &config.theme);
    }
}

pub fn select_project_with_config(config: &AppConfig) -> Option<String> {
    let workspace = config.get_workspace_path();
    let mut projects = vec![];

    if let Ok(entries) = fs::read_dir(&workspace) {
        for entry in entries.flatten() {
            if entry.path().is_dir() && entry.path().join("Cargo.toml").exists() {
                if let Some(name) = entry.file_name().to_str() {
                    projects.push(name.to_string());
                }
            }
        }
    } else {
        AnsiTheme::print_warning(&format!(" Workspace directory not found: {}\n", workspace.display()), &config.theme);
        AnsiTheme::print_themed("Update the workspace path in configuration settings.\n", &config.theme);
        return None;
    }

    if projects.is_empty() {
        AnsiTheme::print_themed("No Rust projects found in workspace.\n", &config.theme);
        return None;
    }

    AnsiTheme::print_themed("\nSelect a project:\n", &config.theme);
    for (i, project) in projects.iter().enumerate() {
        AnsiTheme::print_themed(&format!("{}: {}\n", i + 1, project), &config.theme);
    }

    AnsiTheme::print_themed("Enter number (or Q to cancel): ", &config.theme);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.eq_ignore_ascii_case("q") {
        return None;
    }

    if let Ok(num) = input.parse::<usize>() {
        if num > 0 && num <= projects.len() {
            let selected = projects[num - 1].clone();
            AnsiTheme::print_themed(&format!("Selected project: {}\n", selected), &config.theme);
            return Some(selected);
        }
    }

    AnsiTheme::print_themed("Invalid selection.\n", &config.theme);
    None
}

// Keep original functions for backward compatibility
pub fn list_projects() {
    let default_config = AppConfig::default();
    list_projects_with_config(&default_config);
}

pub fn select_project() -> Option<String> {
    let default_config = AppConfig::default();
    select_project_with_config(&default_config)
}