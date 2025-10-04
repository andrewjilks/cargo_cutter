// project_creator.rs
use std::fs;
use std::process::Command;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub struct ProjectCreator {
    config: crate::config::AppConfig,
}

impl ProjectCreator {
    pub fn new_with_config(config: crate::config::AppConfig) -> Self {
        ProjectCreator { config }
    }

    pub fn new() -> Self {
        let config_manager = crate::config::ConfigManager::new();
        ProjectCreator { 
            config: config_manager.get_config().clone()
        }
    }

    pub fn create_new_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        
        // Check if project already exists
        if project_path.exists() {
            return Err(format!("Project '{}' already exists!", project_name));
        }

        AnsiTheme::print_themed(&format!("Creating new Rust project: {}\n", project_name), theme);
        
        // Create project directory
        fs::create_dir_all(&project_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created project directory\n", theme);

        // Create src directory
        let src_path = self.config.get_src_path(project_name);
        fs::create_dir_all(&src_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created src directory\n", theme);

        // Create Cargo.toml
        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

"#,
            project_name
        );

        let cargo_toml_path = self.config.get_cargo_toml_path(project_name);
        fs::write(&cargo_toml_path, cargo_toml_content).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created Cargo.toml\n", theme);

        // Create main.rs with basic template
        let main_rs_content = r#"fn main() {
    println!("Hello, world!");
}
"#;

        let main_rs_path = src_path.join("main.rs");
        fs::write(&main_rs_path, main_rs_content).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created main.rs\n", theme);

        // Initialize git repository (optional)
        if let Ok(status) = Command::new("git")
            .current_dir(&project_path)
            .arg("init")
            .status()
        {
            if status.success() {
                AnsiTheme::print_success(" Initialized git repository\n", theme);
            }
        }

        AnsiTheme::print_success(&format!(" Project '{}' created successfully!\n", project_name), theme);
        AnsiTheme::print_themed(&format!(" Project location: {}\n", project_path.display()), theme);

        Ok(())
    }

    pub fn create_library_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        
        // Check if project already exists
        if project_path.exists() {
            return Err(format!("Project '{}' already exists!", project_name));
        }

        AnsiTheme::print_themed(&format!("Creating new Rust library project: {}\n", project_name), theme);
        
        // Create project directory
        fs::create_dir_all(&project_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created project directory\n", theme);

        // Create src directory
        let src_path = self.config.get_src_path(project_name);
        fs::create_dir_all(&src_path).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created src directory\n", theme);

        // Create Cargo.toml
        let cargo_toml_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
name = "{}"
path = "src/lib.rs"

"#,
            project_name, project_name
        );

        let cargo_toml_path = self.config.get_cargo_toml_path(project_name);
        fs::write(&cargo_toml_path, cargo_toml_content).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created Cargo.toml\n", theme);

        // Create lib.rs with basic template
        let lib_rs_content = r#"pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
"#;

        let lib_rs_path = src_path.join("lib.rs");
        fs::write(&lib_rs_path, lib_rs_content).map_err(|e| e.to_string())?;
        AnsiTheme::print_success(" Created lib.rs\n", theme);

        AnsiTheme::print_success(&format!(" Library project '{}' created successfully!\n", project_name), theme);
        AnsiTheme::print_themed(&format!(" Project location: {}\n", project_path.display()), theme);

        Ok(())
    }

    pub fn create_project_with_template(&self, project_name: &str, template: &str, theme: &ThemeConfig) -> Result<(), String> {
        match template {
            "basic" => self.create_new_project(project_name, theme),
            "library" => self.create_library_project(project_name, theme),
            "cli" => self.create_cli_project(project_name, theme),
            _ => Err(format!("Unknown template: {}", template))
        }
    }

    fn create_cli_project(&self, project_name: &str, theme: &ThemeConfig) -> Result<(), String> {
        let project_path = self.config.get_project_path(project_name);
        
        if project_path.exists() {
            return Err(format!("Project '{}' already exists!", project_name));
        }

        AnsiTheme::print_themed(&format!("Creating CLI project: {}\n", project_name), theme);
        
        // Create basic project structure
        self.create_new_project(project_name, theme)?;

        // Add common CLI dependencies
        let cargo_toml_path = self.config.get_cargo_toml_path(project_name);
        let mut cargo_content = fs::read_to_string(&cargo_toml_path).map_err(|e| e.to_string())?;
        
        // Add CLI dependencies
        cargo_content.push_str("\n[dependencies]\nclap = { version = \"4.0\", features = [\"derive\"] }\nanyhow = \"1.0\"\n");
        
        fs::write(&cargo_toml_path, cargo_content).map_err(|e| e.to_string())?;

        // Create a more advanced main.rs for CLI
        let main_rs_content = r#"use clap::Parser;

/// A simple CLI application
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to greet
    name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.name {
        Some(name) => println!("Hello, {}!", name),
        None => println!("Hello, world!"),
    }

    Ok(())
}
"#;

        let main_rs_path = self.config.get_src_path(project_name).join("main.rs");
        fs::write(&main_rs_path, main_rs_content).map_err(|e| e.to_string())?;

        AnsiTheme::print_success(" Added CLI dependencies and template\n", theme);
        AnsiTheme::print_success(&format!(" CLI project '{}' created successfully!\n", project_name), theme);

        Ok(())
    }
}