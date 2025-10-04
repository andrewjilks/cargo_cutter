// self_update.rs (CONVERTED)
use std::env;
use std::fs::{self, rename, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub struct SelfUpdater;

impl SelfUpdater {
    pub fn new() -> Self {
        SelfUpdater
    }

    // Add the missing method
    pub fn check_for_updates(&self, theme: &ThemeConfig) {
        AnsiTheme::print_themed("Checking for updates...\n", theme);
        // For now, just redirect to perform_self_update
        // In a real implementation, you might check a remote repository first
        match self.perform_self_update(theme) {
            Ok(()) => AnsiTheme::print_themed("Update process started...\n", theme),
            Err(e) => AnsiTheme::print_error(&format!("Error during self-update: {}\n", e), theme),
        }
    }

    pub fn perform_self_update(&self, theme: &ThemeConfig) -> Result<(), String> {
        AnsiTheme::print_themed("Starting self-update process...\n", theme);
        
        let build_system = crate::build_system::BuildSystem::new(Some("self_update.log".to_string()), true);

        let current_exe: PathBuf = env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;

        AnsiTheme::print_themed(&format!("Current executable: {}\n", current_exe.display()), theme);

        // Step 0: Read current version and increment it
        let (new_version, mut cargo_restore) = self.increment_version(theme)
            .map_err(|e| format!("Failed to increment version: {}", e))?;
        
        AnsiTheme::print_themed(&format!("New version will be: {}\n", new_version), theme);

        // Step 1: Create backup of entire project
        self.create_backup(&current_exe, &new_version, theme)
            .map_err(|e| format!("Failed to create backup: {}", e))?;

        // Step 2: On Windows, rename the current exe to free up the path for the new build
        if cfg!(target_os = "windows") {
            let stem = current_exe.file_stem().unwrap().to_str().unwrap();
            let ext = current_exe.extension().map_or("", |e| e.to_str().unwrap());
            let old_exe = current_exe.with_file_name(format!("{}-old.{}", stem, ext));
            
            AnsiTheme::print_themed(&format!("Renaming current executable to: {}\n", old_exe.display()), theme);
            
            // Try to remove old file if it exists
            let _ = fs::remove_file(&old_exe);
            
            rename(&current_exe, &old_exe)
                .map_err(|e| format!("Failed to rename current executable: {}", e))?;
            
            AnsiTheme::print_success("Renamed current executable for update\n", theme);
        }
        
        // Step 3: Build ourselves (now the path is free on Windows)
        AnsiTheme::print_themed("Building new version...\n", theme);
        match build_system.build_self(theme) {
            Ok(result) => {
                if !result.success {
                    return Err(format!("Self-build failed: {}", result.stderr));
                }
                AnsiTheme::print_success("Self-build successful\n", theme);
                
                // Show build output if there were warnings/errors
                if !result.stderr.is_empty() {
                    AnsiTheme::print_themed("Build output:\n", theme);
                    AnsiTheme::print_themed(&result.stderr, theme);
                }
            }
            Err(e) => return Err(format!("Self-build error: {}", e)),
        }

        // Step 4: Get the new binary path
        let new_binary_path = build_system.get_self_binary_path(theme)
            .map_err(|e| format!("Failed to get new binary path: {}", e))?;
        
        if !new_binary_path.exists() {
            return Err(format!("New binary not found at: {}", new_binary_path.display()));
        }

        AnsiTheme::print_success(&format!("New binary ready: {}\n", new_binary_path.display()), theme);

        // Step 5: Commit the Cargo.toml changes (prevent restore)
        cargo_restore.commit();

        // Step 6: Restart with the new binary
        AnsiTheme::print_themed("Restarting with new version...\n", theme);

        let mut command = Command::new(&new_binary_path);
        command.args(env::args().skip(1));
        
        AnsiTheme::print_themed(&format!("Command: {:?}\n", command), theme);
        
        command.spawn()
            .map_err(|e| format!("Failed to spawn new process: {}", e))?;

        AnsiTheme::print_success("New process spawned successfully, exiting...\n", theme);
        exit(0);
    }

    /// Increments the version in Cargo.toml and returns the new version
    fn increment_version(&self, theme: &ThemeConfig) -> Result<(String, CargoTomlRestore), String> {
        let cargo_toml_path = self.find_cargo_toml()
            .ok_or("Could not find Cargo.toml".to_string())?;
        
        AnsiTheme::print_themed(&format!("Found Cargo.toml at: {}\n", cargo_toml_path.display()), theme);
        
        let content = fs::read_to_string(&cargo_toml_path)
            .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;
        
        // Parse and increment version
        let mut in_package_section = false;
        let mut new_content = String::new();
        let mut _current_version = None;
        let mut new_version = None;
        
        for line in content.lines() {
            if line.trim() == "[package]" {
                in_package_section = true;
                new_content.push_str(line);
                new_content.push('\n');
                continue;
            }
            
            if in_package_section && line.trim().starts_with('[') && line.trim().ends_with(']') {
                in_package_section = false;
            }
            
            if in_package_section && line.trim().starts_with("version =") {
                // Extract current version
                let version_str = line.trim();
                let start_idx = version_str.find('"').ok_or("Invalid version format")? + 1;
                let end_idx = version_str.rfind('"').ok_or("Invalid version format")?;
                let version = &version_str[start_idx..end_idx];
                
                _current_version = Some(version.to_string());
                
                // Increment patch version
                let new_ver = self.increment_patch_version(version)
                    .map_err(|e| format!("Failed to increment version {}: {}", version, e))?;
                
                new_version = Some(new_ver.clone());
                let new_line = format!("version = \"{}\"", new_ver);
                new_content.push_str(&new_line);
                new_content.push('\n');
                
                AnsiTheme::print_success(&format!("Version updated: {} → {}\n", version, new_ver), theme);
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
        
        let new_version = new_version.ok_or("Could not find version in Cargo.toml".to_string())?;
        
        // Create backup of original Cargo.toml
        let backup_path = cargo_toml_path.with_extension("toml.backup");
        fs::copy(&cargo_toml_path, &backup_path)
            .map_err(|e| format!("Failed to backup Cargo.toml: {}", e))?;
        
        // Write updated Cargo.toml
        fs::write(&cargo_toml_path, new_content)
            .map_err(|e| format!("Failed to write updated Cargo.toml: {}", e))?;
        
        AnsiTheme::print_success("Cargo.toml updated successfully\n", theme);
        
        // Create restore guard
        let backup_guard = CargoTomlRestore::new(cargo_toml_path, backup_path);
        
        Ok((new_version, backup_guard))
    }
    
    /// Increments the patch version (major.minor.patch)
    fn increment_patch_version(&self, version: &str) -> Result<String, String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version));
        }
        
        let patch: u32 = parts[2].parse()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;
        
        let new_version = format!("{}.{}.{}", parts[0], parts[1], patch + 1);
        Ok(new_version)
    }
    
    /// Creates a backup zip of the entire project
    fn create_backup(&self, current_exe: &Path, new_version: &str, theme: &ThemeConfig) -> Result<(), String> {
        let backups_dir = self.get_backups_dir()?;
        
        // Create backups directory if it doesn't exist
        fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;

        AnsiTheme::print_themed(&format!("Backups directory: {}\n", backups_dir.display()), theme);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();
        
        let exe_name = current_exe.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid executable name")?;
        
        let backup_filename = format!("{}-backup-v{}-{}.zip", exe_name, new_version, timestamp);
        let backup_path = backups_dir.join(&backup_filename);
        
        AnsiTheme::print_themed(&format!("Creating full project backup: {}\n", backup_path.display()), theme);
        
        // Get project root directory
        let project_root = self.find_cargo_toml()
            .and_then(|p| p.parent().map(PathBuf::from))
            .ok_or("Could not find project root directory".to_string())?;
        
        AnsiTheme::print_themed(&format!("Project root: {}\n", project_root.display()), theme);
        
        // Create zip file
        let file = File::create(&backup_path)
            .map_err(|e| format!("Failed to create backup file: {}", e))?;
        
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        
        // Walk through project directory and add all relevant files
        self.add_directory_to_zip(&mut zip, &project_root, &project_root, &options)
            .map_err(|e| format!("Failed to add directory to zip: {}", e))?;
        
        zip.finish()
            .map_err(|e| format!("Failed to finalize backup zip: {}", e))?;

        // Verify the backup was created
        let backup_metadata = fs::metadata(&backup_path)
            .map_err(|e| format!("Failed to verify backup creation: {}", e))?;
        
        AnsiTheme::print_success(&format!("Full project backup created: {} ({} bytes)\n", backup_path.display(), backup_metadata.len()), theme);
        
        Ok(())
    }
    
    /// Recursively adds a directory to the zip file
    fn add_directory_to_zip(
        &self,
        zip: &mut zip::ZipWriter<File>,
        base_path: &Path,
        current_path: &Path,
        options: &zip::write::FileOptions,
    ) -> Result<(), String> {
        let entries = fs::read_dir(current_path)
            .map_err(|e| format!("Failed to read directory {}: {}", current_path.display(), e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            let metadata = entry.metadata()
                .map_err(|e| format!("Failed to get metadata for {}: {}", path.display(), e))?;
            
            // Calculate relative path for zip
            let relative_path = path.strip_prefix(base_path)
                .map_err(|e| format!("Failed to calculate relative path: {}", e))?;
            
            if metadata.is_dir() {
                // Skip target directory and .git directory to avoid huge backups
                if path.file_name().and_then(|s| s.to_str()) == Some("target") 
                    || path.file_name().and_then(|s| s.to_str()) == Some(".git") {
                    AnsiTheme::print_themed(&format!("Skipping directory: {}\n", path.display()), &ThemeConfig::default());
                    continue;
                }
                
                // Add directory entry to zip
                if !relative_path.as_os_str().is_empty() {
                    let dir_name = format!("{}/", relative_path.to_string_lossy());
                    zip.add_directory(&dir_name, *options)
                        .map_err(|e| format!("Failed to add directory to zip: {}", e))?;
                }
                
                // Recursively add contents
                self.add_directory_to_zip(zip, base_path, &path, options)?;
            } else {
                // Skip large or unnecessary files
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "zip" || ext == "backup" {
                        AnsiTheme::print_themed(&format!("Skipping backup file: {}\n", path.display()), &ThemeConfig::default());
                        continue;
                    }
                }
                
                // Skip files in target directory (in case we encounter any)
                if path.components().any(|c| c.as_os_str() == "target") {
                    continue;
                }
                
                // Add file to zip
                let file_data = fs::read(&path)
                    .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
                
                zip.start_file(relative_path.to_string_lossy().as_ref(), *options)
                    .map_err(|e| format!("Failed to start zip entry: {}", e))?;
                
                zip.write_all(&file_data)
                    .map_err(|e| format!("Failed to write file to zip: {}", e))?;
                
                AnsiTheme::print_themed(&format!("Added to backup: {}\n", relative_path.display()), &ThemeConfig::default());
            }
        }
        
        Ok(())
    }
    
    /// Finds the Cargo.toml file by searching from current directory upwards
    fn find_cargo_toml(&self) -> Option<PathBuf> {
        let mut current_dir = env::current_dir().ok()?;
        
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return Some(cargo_toml);
            }
            
            if !current_dir.pop() {
                break;
            }
        }
        
        None
    }
    
    /// Gets the backups directory path
    fn get_backups_dir(&self) -> Result<PathBuf, String> {
        let mut dir = if cfg!(target_os = "windows") {
            env::var("LOCALAPPDATA")
                .map(PathBuf::from)
                .or_else(|_| env::var("APPDATA").map(PathBuf::from))
                .map_err(|_| "Could not find AppData directory".to_string())?
        } else {
            dirs::home_dir()
                .ok_or("Could not find home directory".to_string())?
        };
        
        let exe_name = env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "app".to_string());
        
        dir.push(&exe_name);
        dir.push("backups");
        
        AnsiTheme::print_themed(&format!("Backups directory resolved to: {}\n", dir.display()), &ThemeConfig::default());
        Ok(dir)
    }
}

/// Helper struct to restore Cargo.toml backup if something goes wrong
struct CargoTomlRestore {
    original_path: PathBuf,
    backup_path: PathBuf,
    should_restore: bool,
}

impl CargoTomlRestore {
    fn new(original_path: PathBuf, backup_path: PathBuf) -> Self {
        Self {
            original_path,
            backup_path,
            should_restore: true,
        }
    }
    
    fn commit(&mut self) {
        self.should_restore = false;
        AnsiTheme::print_themed("Cargo.toml changes committed\n", &ThemeConfig::default());
    }
}

impl Drop for CargoTomlRestore {
    fn drop(&mut self) {
        if self.should_restore {
            AnsiTheme::print_themed("Restoring original Cargo.toml from backup\n", &ThemeConfig::default());
            let _ = fs::copy(&self.backup_path, &self.original_path);
            let _ = fs::remove_file(&self.backup_path);
        } else {
            AnsiTheme::print_themed("Keeping updated Cargo.toml, removing backup\n", &ThemeConfig::default());
            let _ = fs::remove_file(&self.backup_path);
        }
    }
}