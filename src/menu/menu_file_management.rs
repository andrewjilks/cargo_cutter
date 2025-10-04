// menu_file_management.rs (updated)
use std::io::{self, Write};
use crate::file_manager::FileManager;
use crate::projects;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub fn file_management_menu(file_manager: &FileManager, current_project: &Option<String>, theme: &ThemeConfig) {
    let project = match current_project {
        Some(p) => p.clone(),
        None => match projects::select_project() {
            Some(p) => p,
            None => return,
        },
    };

    loop {
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_green(&format!("File Management - {}\n", project), theme);
        AnsiTheme::print_themed("1) List Rust files in project\n", theme);
        AnsiTheme::print_themed("2) Create new Rust file\n", theme);
        AnsiTheme::print_themed("3) Open Rust file in Notepad (View/Edit)\n", theme);
        AnsiTheme::print_themed("4) Delete Rust file\n", theme);
        AnsiTheme::print_themed("5) Advanced File Operations\n", theme);
        AnsiTheme::print_themed("B) Back to main menu\n", theme);

        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                match file_manager.list_project_files(&project) {
                    Ok(files) => {
                        AnsiTheme::print_themed("\n", theme);
                        AnsiTheme::print_blue("Rust files in project:\n", theme);
                        for (i, path) in files {
                            let relative_path = path.strip_prefix(format!("D:\\RustProjects\\{}\\src", project))
                                .unwrap_or(&path)
                                .display();
                            AnsiTheme::print_themed(&format!("{:2}) {}\n", i, relative_path), theme);
                        }
                    }
                    Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
                }
            }
            "2" => create_rust_file_menu(file_manager, &project, theme),
            "3" => open_rust_file_menu(file_manager, &project, theme),
            "4" => delete_rust_file_menu(file_manager, &project, theme),
            "5" => {
                if let Err(e) = file_manager.file_operations_menu(&project, theme) {
                    AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
                }
            }
            "B" | "b" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

fn create_rust_file_menu(file_manager: &FileManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_green("Create new Rust file:\n", theme);
    AnsiTheme::print_themed("Enter file path (e.g., 'main.rs', 'utils/mod.rs'): ", theme);
    io::stdout().flush().unwrap();

    let mut path_input = String::new();
    io::stdin().read_line(&mut path_input).unwrap();
    let file_path = path_input.trim();

    if file_path.is_empty() {
        AnsiTheme::print_themed("File path cannot be empty.\n", theme);
        return;
    }

    AnsiTheme::print_themed("Enter file content (end with 'EOF' on a new line):\n", theme);
    let mut content = String::new();
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        
        if line.trim() == "EOF" {
            break;
        }
        content.push_str(&line);
    }

    match file_manager.create_rust_file(project, file_path, &content) {
        Ok(()) => {
            AnsiTheme::print_success(" File created successfully!\n", theme);
            
            AnsiTheme::print_themed("Open file in Notepad? (y/N): ", theme);
            io::stdout().flush().unwrap();
            
            let mut open_choice = String::new();
            io::stdin().read_line(&mut open_choice).unwrap();
            
            if open_choice.trim().eq_ignore_ascii_case("y") {
                if let Err(e) = file_manager.open_file_in_notepad(project, file_path) {
                    AnsiTheme::print_error(&format!("Error opening file: {}\n", e), theme);
                }
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn open_rust_file_menu(file_manager: &FileManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_blue("Open Rust file in Notepad:\n", theme);
    
    let file_path = match file_manager.select_file_from_list(project) {
        Ok(Some(path)) => path,
        Ok(None) => return,
        Err(e) => {
            AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
            return;
        }
    };

    match file_manager.open_file_in_notepad(project, &file_path) {
        Ok(()) => AnsiTheme::print_success(" File editing complete.\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn delete_rust_file_menu(file_manager: &FileManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_red("Delete Rust file:\n", theme);
    
    let file_path = match file_manager.select_file_from_list(project) {
        Ok(Some(path)) => path,
        Ok(None) => return,
        Err(e) => {
            AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
            return;
        }
    };

    AnsiTheme::print_error(&format!(" You are about to delete: {}\n", file_path), theme);
    AnsiTheme::print_warning(" Are you sure? This cannot be undone! (y/N): ", theme);
    io::stdout().flush().unwrap();

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();

    if confirm.trim().eq_ignore_ascii_case("y") {
        match file_manager.delete_file(project, &file_path) {
            Ok(()) => AnsiTheme::print_success(" File deleted successfully!\n", theme),
            Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
        }
    } else {
        AnsiTheme::print_themed("Deletion cancelled.\n", theme);
    }
}