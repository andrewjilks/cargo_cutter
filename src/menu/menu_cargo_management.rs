// menu_cargo_management.rs
use std::io::{self, Write};
use crate::file_manager::FileManager;
use crate::projects;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub fn cargo_management_menu(file_manager: &FileManager, current_project: &Option<String>, theme: &ThemeConfig) {
    let project = match current_project {
        Some(p) => p.clone(),
        None => match projects::select_project() {
            Some(p) => p,
            None => return,
        },
    };

    loop {
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_green(&format!("Cargo.toml Management - {}\n", project), theme);
        AnsiTheme::print_themed("1) View Cargo.toml info\n", theme);
        AnsiTheme::print_themed("2) Open Cargo.toml in Notepad (View/Edit)\n", theme);
        AnsiTheme::print_themed("3) Add dependency (quick)\n", theme);
        AnsiTheme::print_themed("4) Update package version (quick)\n", theme);
        AnsiTheme::print_themed("B) Back to main menu\n", theme);

        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                match file_manager.read_cargo_toml(&project) {
                    Ok(cargo_toml) => {
                        AnsiTheme::print_themed("\n", theme);
                        AnsiTheme::print_blue("Package Information:\n", theme);
                        AnsiTheme::print_themed(&format!("Name: {}\n", cargo_toml.package.name), theme);
                        AnsiTheme::print_themed(&format!("Version: {}\n", cargo_toml.package.version), theme);
                        if let Some(edition) = cargo_toml.package.edition {
                            AnsiTheme::print_themed(&format!("Edition: {}\n", edition), theme);
                        }
                        if let Some(deps) = cargo_toml.dependencies {
                            AnsiTheme::print_themed("\n", theme);
                            AnsiTheme::print_blue("Dependencies:\n", theme);
                            for (name, version) in deps {
                                AnsiTheme::print_themed(&format!(" - {} = {}\n", name, version), theme);
                            }
                        }
                    }
                    Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
                }
            }
            "2" => {
                match file_manager.open_cargo_toml_in_notepad(&project) {
                    Ok(()) => AnsiTheme::print_success(" Cargo.toml editing complete.\n", theme),
                    Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
                }
            }
            "3" => add_dependency_menu(file_manager, &project, theme),
            "4" => update_version_menu(file_manager, &project, theme),
            "B" | "b" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

fn add_dependency_menu(file_manager: &FileManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_green("Add dependency:\n", theme);
    AnsiTheme::print_themed("Enter dependency name: ", theme);
    io::stdout().flush().unwrap();

    let mut name_input = String::new();
    io::stdin().read_line(&mut name_input).unwrap();
    let dep_name = name_input.trim();

    AnsiTheme::print_themed("Enter version: ", theme);
    io::stdout().flush().unwrap();

    let mut version_input = String::new();
    io::stdin().read_line(&mut version_input).unwrap();
    let version = version_input.trim();

    match file_manager.add_dependency(project, dep_name, version) {
        Ok(()) => AnsiTheme::print_success(&format!(" Dependency '{}' added successfully!\n", dep_name), theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn update_version_menu(file_manager: &FileManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_yellow("Update package version:\n", theme);
    AnsiTheme::print_themed("Enter new version: ", theme);
    io::stdout().flush().unwrap();

    let mut version_input = String::new();
    io::stdin().read_line(&mut version_input).unwrap();
    let new_version = version_input.trim();

    match file_manager.modify_cargo_toml(project, |cargo_toml| {
        cargo_toml.package.version = new_version.to_string();
    }) {
        Ok(()) => AnsiTheme::print_success(&format!(" Version updated to {}!\n", new_version), theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}