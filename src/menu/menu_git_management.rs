// menu_git_management.rs
use std::io::{self, Write};
use crate::git_manager::GitManager;
use crate::projects;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub fn git_management_menu(git_manager: &GitManager, current_project: &Option<String>, theme: &ThemeConfig) {
    let project = match current_project {
        Some(p) => p.clone(),
        None => match projects::select_project() {
            Some(p) => p,
            None => return,
        },
    };

    // Check if git is initialized
    if !git_manager.is_git_initialized(&project) {
        AnsiTheme::print_warning(&format!(" Git not initialized for project: {}\n", project), theme);
        AnsiTheme::print_themed("Initialize Git repository? (y/N): ", theme);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if input.trim().eq_ignore_ascii_case("y") {
            match git_manager.initialize_git(&project, theme) {
                Ok(()) => AnsiTheme::print_success(" Git initialized successfully\n", theme),
                Err(e) => {
                    AnsiTheme::print_error(&format!(" Failed to initialize git: {}\n", e), theme);
                    return;
                }
            }
        } else {
            AnsiTheme::print_themed("Git initialization cancelled.\n", theme);
            return;
        }
    }

    loop {
        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_green(&format!("Git Management - {}\n", project), theme);
        AnsiTheme::print_themed("1) Status\n", theme);
        AnsiTheme::print_themed("2) Add specific files\n", theme);
        AnsiTheme::print_themed("3) Add all changes\n", theme);
        AnsiTheme::print_themed("4) Commit changes\n", theme);
        AnsiTheme::print_themed("5) Quick Commit & Push\n", theme);
        AnsiTheme::print_themed("6) Push to remote\n", theme);
        AnsiTheme::print_themed("7) Pull from remote\n", theme);
        AnsiTheme::print_themed("8) Manage remotes\n", theme);
        AnsiTheme::print_themed("9) Create version tag\n", theme);
        AnsiTheme::print_themed("10) View commit history\n", theme);
        AnsiTheme::print_themed("B) Back to main menu\n", theme);

        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => show_status(git_manager, &project, theme),
            "2" => add_specific_files_menu(git_manager, &project, theme),
            "3" => add_all_changes(git_manager, &project, theme),
            "4" => commit_menu(git_manager, &project, theme),
            "5" => quick_commit_push_menu(git_manager, &project, theme),
            "6" => push_menu(git_manager, &project, theme),
            "7" => pull_menu(git_manager, &project, theme),
            "8" => remotes_menu(git_manager, &project, theme),
            "9" => create_tag_menu(git_manager, &project, theme),
            "10" => view_history_menu(git_manager, &project, theme),
            "B" | "b" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

// Update ALL helper function signatures to include theme parameter
fn show_status(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    match git_manager.get_status(project) {
        Ok(status) => {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_cyan(&format!("Git Status - Branch: {}\n", status.current_branch), theme);
            
            if status.has_changes {
                AnsiTheme::print_yellow("Changes to be committed:\n", theme);
                for file in status.files {
                    AnsiTheme::print_themed(&format!(" {} {} {}\n", 
                        file.status, 
                        "→", 
                        file.file_path), theme);
                }
            } else {
                AnsiTheme::print_success(" No changes to commit\n", theme);
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn add_specific_files_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    let status = match git_manager.get_status(project) {
        Ok(s) => s,
        Err(e) => {
            AnsiTheme::print_error(&format!("Error: {}\n", e), theme);
            return;
        }
    };

    if !status.has_changes {
        AnsiTheme::print_blue(" No changes to add\n", theme);
        return;
    }

    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Select files to add:\n", theme);
    for (i, file) in status.files.iter().enumerate() {
        AnsiTheme::print_themed(&format!("{:2}) {} {}\n", i + 1, file.status, file.file_path), theme);
    }
    AnsiTheme::print_themed(" A) Add all files\n", theme);
    AnsiTheme::print_themed(" C) Cancel\n", theme);

    AnsiTheme::print_themed("Enter choice (number, A, or C): ", theme);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    match choice {
        "A" | "a" => {
            match git_manager.add_all(project) {
                Ok(()) => AnsiTheme::print_success(" All files added\n", theme),
                Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
            }
        }
        "C" | "c" => {
            AnsiTheme::print_themed("Cancelled.\n", theme);
        }
        _ => {
            if let Ok(num) = choice.parse::<usize>() {
                if num > 0 && num <= status.files.len() {
                    let selected_file = &status.files[num - 1];
                    match git_manager.add_files(project, &[selected_file.file_path.clone()]) {
                        Ok(()) => AnsiTheme::print_success(&format!(" File '{}' added\n", selected_file.file_path), theme),
                        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
                    }
                } else {
                    AnsiTheme::print_themed("Invalid selection.\n", theme);
                }
            } else {
                AnsiTheme::print_themed("Invalid choice.\n", theme);
            }
        }
    }
}

fn add_all_changes(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    match git_manager.add_all(project) {
        Ok(()) => AnsiTheme::print_success(" All changes added\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn commit_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Enter commit message:\n", theme);
    
    let mut message = String::new();
    io::stdin().read_line(&mut message).unwrap();
    let message = message.trim();

    if message.is_empty() {
        AnsiTheme::print_themed("Commit message cannot be empty.\n", theme);
        return;
    }

    match git_manager.commit(project, message) {
        Ok(()) => AnsiTheme::print_success(" Changes committed successfully\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn quick_commit_push_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    // Get current version from Cargo.toml for commit message
    let file_manager = crate::file_manager::FileManager::new();
    let version = match file_manager.read_cargo_toml(project) {
        Ok(cargo_toml) => cargo_toml.package.version,
        Err(_) => "unknown".to_string(),
    };

    let commit_message = format!("Release version v{}", version);
    
    AnsiTheme::print_cyan("Quick Commit & Push\n", theme);
    AnsiTheme::print_themed(&format!("Commit message: {}\n", commit_message), theme);
    
    AnsiTheme::print_themed("Proceed? (y/N): ", theme);
    io::stdout().flush().unwrap();
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    
    if !confirm.trim().eq_ignore_ascii_case("y") {
        AnsiTheme::print_themed("Cancelled.\n", theme);
        return;
    }

    // Add all changes
    if let Err(e) = git_manager.add_all(project) {
        AnsiTheme::print_error(&format!("Error adding changes: {}\n", e), theme);
        return;
    }

    // Commit
    if let Err(e) = git_manager.commit(project, &commit_message) {
        AnsiTheme::print_error(&format!("Error committing: {}\n", e), theme);
        return;
    }

    // Push to default remote (origin/main)
    match git_manager.push(project, "origin", "main") {
        Ok(()) => AnsiTheme::print_success(" Successfully committed and pushed!\n", theme),
        Err(e) => {
            AnsiTheme::print_warning(&format!(" Commit successful but push failed: {}\n", e), theme);
            AnsiTheme::print_themed("You may need to set up remote first.\n", theme);
        }
    }
}

fn push_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    let remotes = match git_manager.get_remotes(project) {
        Ok(r) => r,
        Err(e) => {
            AnsiTheme::print_error(&format!("Error getting remotes: {}\n", e), theme);
            return;
        }
    };

    if remotes.is_empty() {
        AnsiTheme::print_warning(" No remotes configured\n", theme);
        AnsiTheme::print_themed("Use 'Manage remotes' to add a remote repository.\n", theme);
        return;
    }

    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Select remote to push to:\n", theme);
    for (i, remote) in remotes.iter().enumerate() {
        if remote.kind == "(fetch)" {
            AnsiTheme::print_themed(&format!("{:2}) {} - {}\n", i + 1, remote.name, remote.url), theme);
        }
    }

    AnsiTheme::print_themed("Enter remote number: ", theme);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let remote_num: usize = match input.trim().parse() {
        Ok(n) if n > 0 && n <= remotes.len() => n,
        _ => {
            AnsiTheme::print_themed("Invalid selection.\n", theme);
            return;
        }
    };

    let selected_remote = &remotes[remote_num - 1];
    
    AnsiTheme::print_themed("Enter branch name [main]: ", theme);
    io::stdout().flush().unwrap();
    
    let mut branch_input = String::new();
    io::stdin().read_line(&mut branch_input).unwrap();
    let branch = if branch_input.trim().is_empty() {
        "main".to_string()
    } else {
        branch_input.trim().to_string()
    };

    match git_manager.push(project, &selected_remote.name, &branch) {
        Ok(()) => AnsiTheme::print_success(" Push successful\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn pull_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    let remotes = match git_manager.get_remotes(project) {
        Ok(r) => r,
        Err(e) => {
            AnsiTheme::print_error(&format!("Error getting remotes: {}\n", e), theme);
            return;
        }
    };

    if remotes.is_empty() {
        AnsiTheme::print_warning(" No remotes configured\n", theme);
        return;
    }

    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Select remote to pull from:\n", theme);
    for (i, remote) in remotes.iter().enumerate() {
        if remote.kind == "(fetch)" {
            AnsiTheme::print_themed(&format!("{:2}) {} - {}\n", i + 1, remote.name, remote.url), theme);
        }
    }

    AnsiTheme::print_themed("Enter remote number: ", theme);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let remote_num: usize = match input.trim().parse() {
        Ok(n) if n > 0 && n <= remotes.len() => n,
        _ => {
            AnsiTheme::print_themed("Invalid selection.\n", theme);
            return;
        }
    };

    let selected_remote = &remotes[remote_num - 1];
    
    AnsiTheme::print_themed("Enter branch name [main]: ", theme);
    io::stdout().flush().unwrap();
    
    let mut branch_input = String::new();
    io::stdin().read_line(&mut branch_input).unwrap();
    let branch = if branch_input.trim().is_empty() {
        "main".to_string()
    } else {
        branch_input.trim().to_string()
    };

    match git_manager.pull(project, &selected_remote.name, &branch) {
        Ok(()) => AnsiTheme::print_success(" Pull successful\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn remotes_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    loop {
        let remotes = match git_manager.get_remotes(project) {
            Ok(r) => r,
            Err(e) => {
                AnsiTheme::print_error(&format!("Error getting remotes: {}\n", e), theme);
                return;
            }
        };

        AnsiTheme::print_themed("\n", theme);
        AnsiTheme::print_cyan("Remote Repositories:\n", theme);
        if remotes.is_empty() {
            AnsiTheme::print_themed("No remotes configured.\n", theme);
        } else {
            for remote in &remotes {
                AnsiTheme::print_themed(&format!(" - {}: {} ({})\n", remote.name, remote.url, remote.kind), theme);
            }
        }

        AnsiTheme::print_themed("\n1) Add remote\n", theme);
        AnsiTheme::print_themed("2) Back\n", theme);

        AnsiTheme::print_themed("Enter choice: ", theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "1" => add_remote_menu(git_manager, project, theme),
            "2" => break,
            _ => AnsiTheme::print_themed("Invalid choice.\n", theme),
        }
    }
}

fn add_remote_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Add Remote Repository:\n", theme);
    
    AnsiTheme::print_themed("Enter remote name [origin]: ", theme);
    io::stdout().flush().unwrap();
    let mut name_input = String::new();
    io::stdin().read_line(&mut name_input).unwrap();
    let name = if name_input.trim().is_empty() {
        "origin".to_string()
    } else {
        name_input.trim().to_string()
    };

    AnsiTheme::print_themed("Enter remote URL: ", theme);
    io::stdout().flush().unwrap();
    let mut url_input = String::new();
    io::stdin().read_line(&mut url_input).unwrap();
    let url = url_input.trim();

    if url.is_empty() {
        AnsiTheme::print_themed("URL cannot be empty.\n", theme);
        return;
    }

    match git_manager.add_remote(project, &name, url) {
        Ok(()) => AnsiTheme::print_success(" Remote added successfully\n", theme),
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn create_tag_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    // Get current version from Cargo.toml
    let file_manager = crate::file_manager::FileManager::new();
    let version = match file_manager.read_cargo_toml(project) {
        Ok(cargo_toml) => cargo_toml.package.version,
        Err(_) => {
            AnsiTheme::print_themed("Enter tag name: ", theme);
            io::stdout().flush().unwrap();
            let mut version_input = String::new();
            io::stdin().read_line(&mut version_input).unwrap();
            version_input.trim().to_string()
        }
    };

    let tag_name = format!("v{}", version);
    let message = format!("Release version {}", version);

    AnsiTheme::print_cyan(&format!("Create version tag: {}\n", tag_name), theme);
    
    match git_manager.create_tag(project, &tag_name, Some(&message)) {
        Ok(()) => {
            AnsiTheme::print_success(" Tag created successfully\n", theme);
            
            // Ask to push tags
            AnsiTheme::print_themed("Push tags to remote? (y/N): ", theme);
            io::stdout().flush().unwrap();
            
            let mut push_confirm = String::new();
            io::stdin().read_line(&mut push_confirm).unwrap();
            
            if push_confirm.trim().eq_ignore_ascii_case("y") {
                if let Err(e) = git_manager.push_tags(project, "origin") {
                    AnsiTheme::print_warning(&format!(" Failed to push tags: {}\n", e), theme);
                } else {
                    AnsiTheme::print_success(" Tags pushed successfully\n", theme);
                }
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}

fn view_history_menu(git_manager: &GitManager, project: &str, theme: &ThemeConfig) {
    AnsiTheme::print_themed("\n", theme);
    AnsiTheme::print_cyan("Recent Commits:\n", theme);
    
    match git_manager.get_log(project, 10) {
        Ok(commits) => {
            if commits.is_empty() {
                AnsiTheme::print_themed("No commits yet.\n", theme);
            } else {
                for commit in commits {
                    // Convert the hash slice to a String to fix the size issue
                    let short_hash = commit.hash[..7].to_string();
                    AnsiTheme::print_themed(&format!(" {} {}\n", short_hash, commit.message), theme);
                }
            }
        }
        Err(e) => AnsiTheme::print_error(&format!("Error: {}\n", e), theme),
    }
}