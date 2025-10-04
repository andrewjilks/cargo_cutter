// file_manager.rs
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoToml {
    pub package: Package,
    pub dependencies: Option<toml::value::Table>,
    #[serde(flatten)]
    pub other_sections: toml::value::Table,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: Option<String>,
    #[serde(flatten)]
    pub other_fields: toml::value::Table,
}

pub struct FileManager {
    config: crate::config::AppConfig,
}

impl FileManager {
    pub fn new_with_config(config: crate::config::AppConfig) -> Self {
        FileManager { config }
    }

    pub fn new() -> Self {
        let config_manager = crate::config::ConfigManager::new();
        FileManager { 
            config: config_manager.get_config().clone()
        }
    }

    pub fn create_rust_file(&self, project_name: &str, file_path: &str, content: &str) -> Result<(), String> {
        let full_path = self.config.get_src_path(project_name).join(file_path);
        
        // Ensure directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::write(&full_path, content).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(&format!(" Created file: {}\n", full_path.display()), &self.config.theme);
        Ok(())
    }

    pub fn read_rust_file(&self, project_name: &str, file_path: &str) -> Result<String, String> {
        let full_path = self.config.get_src_path(project_name).join(file_path);
        fs::read_to_string(&full_path).map_err(|e| e.to_string())
    }

    pub fn open_file_in_notepad(&self, project_name: &str, file_path: &str) -> Result<(), String> {
        let full_path = self.config.get_src_path(project_name).join(file_path);
        
        if !full_path.exists() {
            return Err(format!("File does not exist: {}", full_path.display()));
        }

        AnsiTheme::print_info(&format!(" Opening '{}' in Notepad...\n", file_path), &self.config.theme);
        
        let status = Command::new("notepad.exe")
            .arg(&full_path)
            .status()
            .map_err(|e| format!("Failed to open Notepad: {}", e))?;
        
        if status.success() {
            AnsiTheme::print_success(" Notepad closed. File saved.\n", &self.config.theme);
            Ok(())
        } else {
            Err("Notepad exited with an error".to_string())
        }
    }

    pub fn read_cargo_toml(&self, project_name: &str) -> Result<CargoToml, String> {
        let cargo_path = self.config.get_cargo_toml_path(project_name);
        let content = fs::read_to_string(&cargo_path).map_err(|e| e.to_string())?;
        toml::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn open_cargo_toml_in_notepad(&self, project_name: &str) -> Result<(), String> {
        let cargo_path = self.config.get_cargo_toml_path(project_name);
        
        if !cargo_path.exists() {
            return Err("Cargo.toml does not exist".to_string());
        }

        AnsiTheme::print_info(" Opening Cargo.toml in Notepad...\n", &self.config.theme);
        
        let status = Command::new("notepad.exe")
            .arg(&cargo_path)
            .status()
            .map_err(|e| format!("Failed to open Notepad: {}", e))?;
        
        if status.success() {
            AnsiTheme::print_success(" Notepad closed. Cargo.toml saved.\n", &self.config.theme);
            Ok(())
        } else {
            Err("Notepad exited with an error".to_string())
        }
    }

    pub fn modify_cargo_toml<F>(&self, project_name: &str, modifier: F) -> Result<(), String> 
    where
        F: FnOnce(&mut CargoToml),
    {
        let cargo_path = self.config.get_cargo_toml_path(project_name);
        let content = fs::read_to_string(&cargo_path).map_err(|e| e.to_string())?;
        
        let mut cargo_toml: CargoToml = toml::from_str(&content).map_err(|e| e.to_string())?;
        modifier(&mut cargo_toml);
        
        let new_content = toml::to_string_pretty(&cargo_toml).map_err(|e| e.to_string())?;
        fs::write(&cargo_path, new_content).map_err(|e| e.to_string())?;
        
        AnsiTheme::print_success(&format!(" Updated Cargo.toml for project: {}\n", project_name), &self.config.theme);
        Ok(())
    }

    pub fn add_dependency(&self, project_name: &str, dep_name: &str, version: &str) -> Result<(), String> {
        self.modify_cargo_toml(project_name, |cargo_toml| {
            if cargo_toml.dependencies.is_none() {
                cargo_toml.dependencies = Some(toml::value::Table::new());
            }
            
            if let Some(ref mut deps) = cargo_toml.dependencies {
                deps.insert(dep_name.to_string(), toml::Value::String(version.to_string()));
            }
        })
    }

    // Manual version management functions
    pub fn set_version(&self, project_name: &str, new_version: &str) -> Result<(), String> {
        self.modify_cargo_toml(project_name, |cargo_toml| {
            cargo_toml.package.version = new_version.to_string();
            AnsiTheme::print_success(&format!(" Version set to: {}\n", new_version), &self.config.theme);
        })
    }

    pub fn increment_patch_version(&self, project_name: &str) -> Result<String, String> {
        let mut new_version = String::new();
    
        self.modify_cargo_toml(project_name, |cargo_toml| {
            let current_version = cargo_toml.package.version.clone();
            let parts: Vec<&str> = current_version.split('.').collect();
        
            if parts.len() == 3 {
                if let Ok(patch) = parts[2].parse::<u32>() {
                    new_version = format!("{}.{}.{}", parts[0], parts[1], patch + 1);
                    cargo_toml.package.version = new_version.clone();
                    AnsiTheme::print_success(&format!(" Version incremented: {} → {}\n", current_version, new_version), &self.config.theme);
                } else {
                    AnsiTheme::print_error(&format!(" Failed to parse patch version: {}\n", parts[2]), &self.config.theme);
                }
            } else {
                AnsiTheme::print_error(&format!(" Invalid version format: {}\n", current_version), &self.config.theme);
            }
        })?;
    
        Ok(new_version)
    }

    pub fn increment_minor_version(&self, project_name: &str) -> Result<String, String> {
        let mut new_version = String::new();
    
        self.modify_cargo_toml(project_name, |cargo_toml| {
            let current_version = cargo_toml.package.version.clone();
            let parts: Vec<&str> = current_version.split('.').collect();
        
            if parts.len() == 3 {
                if let Ok(minor) = parts[1].parse::<u32>() {
                    new_version = format!("{}.{}.0", parts[0], minor + 1);
                    cargo_toml.package.version = new_version.clone();
                    AnsiTheme::print_success(&format!(" Version incremented: {} → {}\n", current_version, new_version), &self.config.theme);
                } else {
                    AnsiTheme::print_error(&format!(" Failed to parse minor version: {}\n", parts[1]), &self.config.theme);
                }
            } else {
                AnsiTheme::print_error(&format!(" Invalid version format: {}\n", current_version), &self.config.theme);
            }
        })?;
    
        Ok(new_version)
    }

    pub fn increment_major_version(&self, project_name: &str) -> Result<String, String> {
        let mut new_version = String::new();
    
        self.modify_cargo_toml(project_name, |cargo_toml| {
            let current_version = cargo_toml.package.version.clone();
            let parts: Vec<&str> = current_version.split('.').collect();
        
            if parts.len() == 3 {
                if let Ok(major) = parts[0].parse::<u32>() {
                    new_version = format!("{}.0.0", major + 1);
                    cargo_toml.package.version = new_version.clone();
                    AnsiTheme::print_success(&format!(" Version incremented: {} → {}\n", current_version, new_version), &self.config.theme);
                } else {
                    AnsiTheme::print_error(&format!(" Failed to parse major version: {}\n", parts[0]), &self.config.theme);
                }
            } else {
                AnsiTheme::print_error(&format!(" Invalid version format: {}\n", current_version), &self.config.theme);
            }
        })?;
    
        Ok(new_version)
    }

    pub fn show_current_version(&self, project_name: &str) -> Result<(), String> {
        let cargo_toml = self.read_cargo_toml(project_name)?;
        AnsiTheme::print_info(&format!(" Current version: {}\n", cargo_toml.package.version), &self.config.theme);
        Ok(())
    }

    pub fn version_management_menu(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        loop {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_cyan(&format!("Version Management for: {}\n", project_name), theme);
            AnsiTheme::print_themed("1) Show current version\n", theme);
            AnsiTheme::print_themed("2) Set custom version\n", theme);
            AnsiTheme::print_themed("3) Increment patch version (x.y.Z → x.y.Z+1)\n", theme);
            AnsiTheme::print_themed("4) Increment minor version (x.Y.z → x.Y+1.0)\n", theme);
            AnsiTheme::print_themed("5) Increment major version (X.y.z → X+1.0.0)\n", theme);
            AnsiTheme::print_themed("6) Open Cargo.toml in Notepad\n", theme);
            AnsiTheme::print_themed("0) Back to main menu\n", theme);
            
            match Self::get_number_input("Select option: ", 0, 6, theme) {
                Some(1) => {
                    self.show_current_version(project_name)?;
                }
                Some(2) => {
                    AnsiTheme::print_themed("Enter new version (format: X.Y.Z): ", theme);
                    io::stdout().flush().unwrap();
                    
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let version = input.trim();
                    
                    if !version.is_empty() {
                        self.set_version(project_name, version)?;
                    }
                }
                Some(3) => {
                    match self.increment_patch_version(project_name) {
                        Ok(new_version) => {
                            AnsiTheme::print_success(&format!(" Patch version incremented to: {}\n", new_version), theme);
                        }
                        Err(e) => {
                            AnsiTheme::print_error(&format!(" Failed to increment patch version: {}\n", e), theme);
                        }
                    }
                }
                Some(4) => {
                    match self.increment_minor_version(project_name) {
                        Ok(new_version) => {
                            AnsiTheme::print_success(&format!(" Minor version incremented to: {}\n", new_version), theme);
                        }
                        Err(e) => {
                            AnsiTheme::print_error(&format!(" Failed to increment minor version: {}\n", e), theme);
                        }
                    }
                }
                Some(5) => {
                    match self.increment_major_version(project_name) {
                        Ok(new_version) => {
                            AnsiTheme::print_success(&format!(" Major version incremented to: {}\n", new_version), theme);
                        }
                        Err(e) => {
                            AnsiTheme::print_error(&format!(" Failed to increment major version: {}\n", e), theme);
                        }
                    }
                }
                Some(6) => {
                    self.open_cargo_toml_in_notepad(project_name)?;
                }
                Some(0) => break,
                _ => AnsiTheme::print_themed("Invalid selection.\n", theme),
            }
        }
        Ok(())
    }

    pub fn list_project_files(&self, project_name: &str) -> Result<Vec<(usize, PathBuf)>, String> {
        let src_path = self.config.get_src_path(project_name);
        
        let mut files = Vec::new();
        Self::walk_dir(&src_path, &mut files)?;
        
        // Number the files
        let numbered_files = files.into_iter().enumerate().map(|(i, path)| (i + 1, path)).collect();
        Ok(numbered_files)
    }

    pub fn select_file_from_list(&self, project_name: &str) -> Result<Option<String>, String> {
        let files = self.list_project_files(project_name)?;
        
        if files.is_empty() {
            AnsiTheme::print_warning(" No Rust files found in project.\n", &self.config.theme);
            return Ok(None);
        }

        AnsiTheme::print_themed("\n", &self.config.theme);
        AnsiTheme::print_blue("Rust files in project:\n", &self.config.theme);
        for (i, path) in &files {
            // Get relative path from src directory
            let relative_path = path.strip_prefix(self.config.get_src_path(project_name))
                .unwrap_or(path)
                .display();
            AnsiTheme::print_themed(&format!("{:2}) {}\n", i, relative_path), &self.config.theme);
        }

        AnsiTheme::print_themed(" 0) Cancel\n", &self.config.theme);
        
        let selection = Self::get_number_input("Select file by number: ", 0, files.len(), &self.config.theme);
        
        match selection {
            Some(0) => {
                AnsiTheme::print_themed("Selection cancelled.\n", &self.config.theme);
                Ok(None)
            }
            Some(index) if index <= files.len() => {
                let selected_path = files[index - 1].1.clone();
                // Convert to relative path from src directory
                let relative_path = selected_path.strip_prefix(self.config.get_src_path(project_name))
                    .unwrap_or(&selected_path)
                    .to_string_lossy()
                    .to_string();
                Ok(Some(relative_path))
            }
            _ => {
                AnsiTheme::print_themed("Invalid selection.\n", &self.config.theme);
                Ok(None)
            }
        }
    }

    // FILE DELETION METHODS
    pub fn delete_file(&self, project_name: &str, file_path: &str) -> Result<(), String> {
        let full_path = self.config.get_src_path(project_name).join(file_path);
        
        if !full_path.exists() {
            return Err(format!("File does not exist: {}", full_path.display()));
        }

        // Safety check: don't allow deleting files outside src directory
        let src_path = self.config.get_src_path(project_name);
        if !full_path.starts_with(&src_path) {
            return Err("Cannot delete files outside project src directory".to_string());
        }

        AnsiTheme::print_error(&format!(" Deleting file: {}\n", full_path.display()), &self.config.theme);
        
        fs::remove_file(&full_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(&format!(" File deleted: {}\n", full_path.display()), &self.config.theme);
        
        // Clean up empty directories
        self.cleanup_empty_directories(project_name, &full_path)?;
        
        Ok(())
    }

    pub fn delete_directory(&self, project_name: &str, dir_path: &str) -> Result<(), String> {
        let full_path = self.config.get_src_path(project_name).join(dir_path);
        
        if !full_path.exists() {
            return Err(format!("Directory does not exist: {}", full_path.display()));
        }

        if !full_path.is_dir() {
            return Err("Path is not a directory".to_string());
        }

        // Safety check: don't allow deleting directories outside src directory
        let src_path = self.config.get_src_path(project_name);
        if !full_path.starts_with(&src_path) {
            return Err("Cannot delete directories outside project src directory".to_string());
        }

        // Check if directory is empty
        let is_empty = fs::read_dir(&full_path)
            .map_err(|e| e.to_string())?
            .next()
            .is_none();

        if !is_empty {
            return Err("Directory is not empty. Delete files first or use force delete.".to_string());
        }

        AnsiTheme::print_error(&format!(" Deleting directory: {}\n", full_path.display()), &self.config.theme);
        
        fs::remove_dir(&full_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(&format!(" Directory deleted: {}\n", full_path.display()), &self.config.theme);
        
        Ok(())
    }

    pub fn force_delete_directory(&self, project_name: &str, dir_path: &str) -> Result<(), String> {
        let full_path = self.config.get_src_path(project_name).join(dir_path);
        
        if !full_path.exists() {
            return Err(format!("Directory does not exist: {}", full_path.display()));
        }

        if !full_path.is_dir() {
            return Err("Path is not a directory".to_string());
        }

        // Safety check: don't allow deleting directories outside src directory
        let src_path = self.config.get_src_path(project_name);
        if !full_path.starts_with(&src_path) {
            return Err("Cannot delete directories outside project src directory".to_string());
        }

        // Recursively delete directory and all contents
        AnsiTheme::print_error(&format!(" Recursively deleting directory: {}\n", full_path.display()), &self.config.theme);
        
        fs::remove_dir_all(&full_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(&format!(" Directory and all contents deleted: {}\n", full_path.display()), &self.config.theme);
        
        Ok(())
    }

    /// Helper method to clean up empty directories after file deletion
    fn cleanup_empty_directories(&self, project_name: &str, deleted_file_path: &PathBuf) -> Result<(), String> {
        if let Some(mut current_dir) = deleted_file_path.parent() {
            let src_path = self.config.get_src_path(project_name);
            
            // Walk up the directory tree until we reach src directory
            while current_dir != src_path && current_dir.starts_with(&src_path) {
                // Check if directory is empty
                if let Ok(mut entries) = fs::read_dir(current_dir) {
                    if entries.next().is_none() {
                        // Directory is empty, remove it
                        AnsiTheme::print_warning(&format!(" Removing empty directory: {}\n", current_dir.display()), &self.config.theme);
                        fs::remove_dir(current_dir).map_err(|e| e.to_string())?;
                        AnsiTheme::print_success(" Empty directory removed\n", &self.config.theme);
                    } else {
                        // Directory is not empty, stop cleanup
                        break;
                    }
                }
                
                // Move to parent directory
                if let Some(parent) = current_dir.parent() {
                    current_dir = parent;
                } else {
                    break;
                }
            }
        }
        
        Ok(())
    }

    pub fn file_operations_menu(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        loop {
            AnsiTheme::print_themed("\n", theme);
            AnsiTheme::print_cyan(&format!("File Operations for: {}\n", project_name), theme);
            AnsiTheme::print_themed("1) List files\n", theme);
            AnsiTheme::print_themed("2) Create file\n", theme);
            AnsiTheme::print_themed("3) Edit file (Notepad)\n", theme);
            AnsiTheme::print_themed("4) Delete file\n", theme);
            AnsiTheme::print_themed("5) Delete empty directory\n", theme);
            AnsiTheme::print_themed("6) Force delete directory (recursive)\n", theme);
            AnsiTheme::print_themed("0) Back to main menu\n", theme);
            
            match Self::get_number_input("Select option: ", 0, 6, theme) {
                Some(1) => {
                    match self.list_project_files(project_name) {
                        Ok(files) => {
                            AnsiTheme::print_themed("\n", theme);
                            AnsiTheme::print_blue("Rust files in project:\n", theme);
                            for (i, path) in files {
                                let relative_path = path.strip_prefix(self.config.get_src_path(project_name))
                                    .unwrap_or(&path)
                                    .display();
                                AnsiTheme::print_themed(&format!("{:2}) {}\n", i, relative_path), theme);
                            }
                        }
                        Err(e) => AnsiTheme::print_error(&format!(" Failed to list files: {}\n", e), theme),
                    }
                }
                Some(2) => {
                    AnsiTheme::print_themed("Enter file path (e.g., 'main.rs', 'utils/mod.rs'): ", theme);
                    io::stdout().flush().unwrap();
                    
                    let mut path_input = String::new();
                    io::stdin().read_line(&mut path_input).unwrap();
                    let file_path = path_input.trim();
                    
                    if file_path.is_empty() {
                        AnsiTheme::print_themed("File path cannot be empty.\n", theme);
                        continue;
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
                    
                    if let Err(e) = self.create_rust_file(project_name, file_path, &content) {
                        AnsiTheme::print_error(&format!(" Failed to create file: {}\n", e), theme);
                    }
                }
                Some(3) => {
                    if let Ok(Some(file_path)) = self.select_file_from_list(project_name) {
                        if let Err(e) = self.open_file_in_notepad(project_name, &file_path) {
                            AnsiTheme::print_error(&format!(" Failed to open file: {}\n", e), theme);
                        }
                    }
                }
                Some(4) => {
                    if let Ok(Some(file_path)) = self.select_file_from_list(project_name) {
                        AnsiTheme::print_warning(&format!(" Are you sure you want to delete '{}'? (y/N): ", file_path), theme);
                        io::stdout().flush().unwrap();
                        
                        let mut confirm = String::new();
                        io::stdin().read_line(&mut confirm).unwrap();
                        
                        if confirm.trim().eq_ignore_ascii_case("y") {
                            match self.delete_file(project_name, &file_path) {
                                Ok(()) => AnsiTheme::print_success(" File deleted successfully\n", theme),
                                Err(e) => AnsiTheme::print_error(&format!(" Failed to delete file: {}\n", e), theme),
                            }
                        } else {
                            AnsiTheme::print_themed("Deletion cancelled.\n", theme);
                        }
                    }
                }
                Some(5) => {
                    AnsiTheme::print_themed("Enter directory path to delete (must be empty): ", theme);
                    io::stdout().flush().unwrap();
                    
                    let mut dir_input = String::new();
                    io::stdin().read_line(&mut dir_input).unwrap();
                    let dir_path = dir_input.trim();
                    
                    if !dir_path.is_empty() {
                        match self.delete_directory(project_name, dir_path) {
                            Ok(()) => AnsiTheme::print_success(" Directory deleted successfully\n", theme),
                            Err(e) => AnsiTheme::print_error(&format!(" Failed to delete directory: {}\n", e), theme),
                        }
                    }
                }
                Some(6) => {
                    AnsiTheme::print_themed("Enter directory path to force delete (recursive - DANGEROUS): ", theme);
                    io::stdout().flush().unwrap();
                    
                    let mut dir_input = String::new();
                    io::stdin().read_line(&mut dir_input).unwrap();
                    let dir_path = dir_input.trim();
                    
                    if !dir_path.is_empty() {
                        AnsiTheme::print_error(&format!(" Are you ABSOLUTELY sure? This cannot be undone! (type 'YES' to confirm): ",), theme);
                        io::stdout().flush().unwrap();
                        
                        let mut confirm = String::new();
                        io::stdin().read_line(&mut confirm).unwrap();
                        
                        if confirm.trim() == "YES" {
                            match self.force_delete_directory(project_name, dir_path) {
                                Ok(()) => AnsiTheme::print_success(" Directory and all contents deleted\n", theme),
                                Err(e) => AnsiTheme::print_error(&format!(" Failed to delete directory: {}\n", e), theme),
                            }
                        } else {
                            AnsiTheme::print_themed("Force deletion cancelled.\n", theme);
                        }
                    }
                }
                Some(0) => break,
                _ => AnsiTheme::print_themed("Invalid selection.\n", theme),
            }
        }
        Ok(())
    }

    fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                
                if path.is_dir() {
                    Self::walk_dir(&path, files)?;
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    fn get_number_input(prompt: &str, min: usize, max: usize, theme: &ThemeConfig) -> Option<usize> {
        AnsiTheme::print_themed(prompt, theme);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("cancel") {
            return None;
        }

        match input.parse::<usize>() {
            Ok(num) if num >= min && num <= max => Some(num),
            _ => {
                AnsiTheme::print_themed(&format!("Please enter a number between {} and {}.\n", min, max), theme);
                None
            }
        }
    }
}